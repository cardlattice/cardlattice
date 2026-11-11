//! Serve the exact CI artifact; the origin never recompiles it into another dataset.
use ingestion::bundle::{self, Summary};
use sqlx::PgPool;
use std::path::Path;

pub async fn import(
    pool: &PgPool,
    manifest_path: &Path,
    output: &Path,
    apply: bool,
) -> anyhow::Result<Summary> {
    let (bundle, manifest, bytes) = bundle::load_artifact(manifest_path)?;
    if apply {
        tokio::fs::create_dir_all(output).await?;
        let temporary = output.join(format!(".{}.tmp", catalogue::operation_id()));
        tokio::fs::write(&temporary, &bytes).await?;
        tokio::fs::rename(temporary, output.join(&manifest.filename)).await?;
    }
    let summary = bundle::index(pool, &bundle, apply).await?;
    if apply {
        let mut tx = pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(71002)")
            .execute(&mut *tx)
            .await?;
        let revision: i64 =
            sqlx::query_scalar("SELECT COALESCE(MAX(revision),0) FROM catalogue_revisions")
                .fetch_one(&mut *tx)
                .await?;
        let inserted = sqlx::query("INSERT INTO publication_manifests(id,revision,release_date,schema_version,sha256,byte_size,record_count,filename) VALUES($1,$2,(now() AT TIME ZONE 'UTC')::date,1,$3,$4,$5,$6) ON CONFLICT DO NOTHING")
            .bind(format!("catalogue-{}",manifest.sha256)).bind(revision).bind(&manifest.sha256).bind(manifest.byte_size as i64).bind(bundle.records.len() as i64).bind(&manifest.filename).execute(&mut *tx).await?.rows_affected();
        if inserted > 0 && revision > 0 {
            sqlx::query(
                "INSERT INTO cache_invalidation_outbox(id,revision,paths) VALUES($1,$2,$3)",
            )
            .bind(catalogue::operation_id())
            .bind(revision)
            .bind(serde_json::json!(["/v1/datasets", "/v1/status"]))
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }
    Ok(summary)
}
