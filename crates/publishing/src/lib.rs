use catalogue::Record;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone)]
pub struct PublishingConfig {
    pub directory: PathBuf,
    pub public_origin: String,
    pub cloudflare: Option<CloudflareConfig>,
}
#[derive(Clone)]
pub struct CloudflareConfig {
    pub zone_id: String,
    pub api_token: String,
}
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Cache purge request failed")]
    Purge,
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: Uuid,
    pub revision: i64,
    pub schema_version: i32,
    pub sha256: String,
    pub byte_size: i64,
    pub record_count: i64,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}

pub fn retry_delay(attempt: i32) -> i64 {
    (5_i64 * 2_i64.pow(attempt.clamp(0, 10) as u32)).min(3600)
}

/// Repeatable-read snapshot, streamed from PostgreSQL. Artifacts are renamed atomically.
pub async fn publish_snapshot(
    pool: &PgPool,
    config: &PublishingConfig,
) -> Result<Manifest, PublishError> {
    tokio::fs::create_dir_all(&config.directory).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await?;
    let revision: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(revision),0) FROM catalogue_revisions")
            .fetch_one(&mut *tx)
            .await?;
    if let Some(manifest) =
        sqlx::query_as::<_, Manifest>("SELECT * FROM publication_manifests WHERE revision=$1")
            .bind(revision)
            .fetch_optional(&mut *tx)
            .await?
    {
        return Ok(manifest);
    }
    let id = Uuid::now_v7();
    let temporary = config.directory.join(format!(".{id}.tmp"));
    let mut file = tokio::fs::File::create(&temporary).await?;
    let mut hasher = Sha256::new();
    let mut count = 0_i64;
    let mut byte_size = 0_i64;
    {
        let mut rows = sqlx::query_as::<_, Record>(
            "SELECT * FROM catalogue_records WHERE deleted_at IS NULL ORDER BY id",
        )
        .fetch(&mut *tx);
        while let Some(record) = rows.try_next().await? {
            let mut bytes = serde_json::to_vec(&record)?;
            bytes.push(b'\n');
            hasher.update(&bytes);
            file.write_all(&bytes).await?;
            count += 1;
            byte_size += bytes.len() as i64;
        }
    }
    file.sync_all().await?;
    drop(file);
    let digest = format!("{:x}", hasher.finalize());
    let filename = format!("catalogue-r{revision}-{digest}.ndjson");
    tokio::fs::rename(&temporary, config.directory.join(&filename)).await?;
    let manifest = sqlx::query_as::<_, Manifest>("INSERT INTO publication_manifests(id,revision,sha256,byte_size,record_count,filename) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(revision) DO UPDATE SET revision=EXCLUDED.revision RETURNING *")
        .bind(id).bind(revision).bind(digest).bind(byte_size).bind(count).bind(filename).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(manifest)
}

pub async fn purge(config: &PublishingConfig, paths: &[String]) -> Result<(), PublishError> {
    let Some(cloudflare) = &config.cloudflare else {
        return Ok(());
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| PublishError::Purge)?;
    // URL purge is available without Enterprise cache tags. No per-request Worker.
    for chunk in paths.chunks(30) {
        let urls: Vec<_> = chunk
            .iter()
            .map(|path| format!("{}{path}", config.public_origin.trim_end_matches('/')))
            .collect();
        let response = client
            .post(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                cloudflare.zone_id
            ))
            .bearer_auth(&cloudflare.api_token)
            .json(&serde_json::json!({"files":urls}))
            .send()
            .await
            .map_err(|_| PublishError::Purge)?;
        if !response.status().is_success() {
            return Err(PublishError::Purge);
        }
        let result: Value = response.json().await.map_err(|_| PublishError::Purge)?;
        if result["success"] != true {
            return Err(PublishError::Purge);
        }
    }
    Ok(())
}

/// Row lock makes concurrent workers safe; crash rolls back the claim for retry.
pub async fn process_one(pool: &PgPool, config: &PublishingConfig) -> Result<bool, PublishError> {
    process_one_using(pool, config, &ConfiguredPurger(config.clone())).await
}

pub trait CachePurger: Send + Sync {
    fn purge(
        &self,
        paths: &[String],
    ) -> impl std::future::Future<Output = Result<(), PublishError>> + Send;
}
pub struct ConfiguredPurger(pub PublishingConfig);
impl CachePurger for ConfiguredPurger {
    async fn purge(&self, paths: &[String]) -> Result<(), PublishError> {
        purge(&self.0, paths).await
    }
}

pub async fn process_one_using(
    pool: &PgPool,
    config: &PublishingConfig,
    purger: &impl CachePurger,
) -> Result<bool, PublishError> {
    let mut tx = pool.begin().await?;
    let row: Option<(Uuid, Value, i32)> = sqlx::query_as("SELECT id,paths,attempts FROM cache_invalidation_outbox WHERE status='pending' AND next_attempt_at <= now() ORDER BY revision FOR UPDATE SKIP LOCKED LIMIT 1").fetch_optional(&mut *tx).await?;
    let Some((id, paths, attempts)) = row else {
        return Ok(false);
    };
    let result = async {
        publish_snapshot(pool, config).await?;
        let paths: Vec<String> = serde_json::from_value(paths)?;
        purger.purge(&paths).await
    }
    .await;
    match result {
        Ok(()) => {
            sqlx::query("UPDATE cache_invalidation_outbox SET status='completed',attempts=attempts+1,last_error=NULL,completed_at=clock_timestamp() WHERE id=$1").bind(id).execute(&mut *tx).await?;
        }
        Err(error) => {
            tracing::warn!(outbox_id=%id, %error, "publication or invalidation failed");
            sqlx::query("UPDATE cache_invalidation_outbox SET status=$2,attempts=attempts+1,last_error=$3,next_attempt_at=clock_timestamp()+make_interval(secs=>$4) WHERE id=$1")
                .bind(id).bind(if attempts + 1 >= 10 { "failed" } else { "pending" }).bind(error.to_string()).bind(retry_delay(attempts) as f64).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    Ok(true)
}

pub async fn replay(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("UPDATE cache_invalidation_outbox SET status='pending',attempts=0,next_attempt_at=clock_timestamp(),last_error=NULL WHERE id=$1 AND status='failed'").bind(id).execute(pool).await?.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_is_bounded() {
        assert_eq!(retry_delay(0), 5);
        assert_eq!(retry_delay(1), 10);
        assert_eq!(retry_delay(20), 3600);
    }
}
