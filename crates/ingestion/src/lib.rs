use catalogue::{Draft, Kind, normalize};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};
use std::io::Cursor;
use uuid::Uuid;

pub mod official_pokemon;
pub mod pokemon_cards;

#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Validation(#[from] catalogue::ValidationError),
    #[error("Invalid fixture: {0}")]
    Fixture(#[from] serde_json::Error),
    #[error("Scan must be a valid PNG or JPEG, at most 8 MiB and 24 megapixels")]
    InvalidScan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fixture {
    pub id: Uuid,
    pub draft: Draft,
}

/// Optional OCR adapters return candidates only; no adapter can publish records.
pub trait ScanExtractor: Send + Sync {
    fn extract(&self, image: &[u8]) -> Result<ScanCandidates, IngestionError>;
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCandidates {
    pub fields: Value,
    pub requires_human_review: bool,
    pub engine: String,
}
pub struct ManualExtractor;
impl ScanExtractor for ManualExtractor {
    fn extract(&self, _: &[u8]) -> Result<ScanCandidates, IngestionError> {
        Ok(ScanCandidates {
            fields: json!({}),
            requires_human_review: true,
            engine: "manual".into(),
        })
    }
}

pub fn validate_scan(bytes: &[u8]) -> Result<(&'static str, String), IngestionError> {
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err(IngestionError::InvalidScan);
    }
    let format = image::guess_format(bytes).map_err(|_| IngestionError::InvalidScan)?;
    let mime = match format {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        _ => return Err(IngestionError::InvalidScan),
    };
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(6000);
    limits.max_image_height = Some(6000);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|_| IngestionError::InvalidScan)?;
    if u64::from(decoded.width()) * u64::from(decoded.height()) > 24_000_000 {
        return Err(IngestionError::InvalidScan);
    }
    Ok((mime, format!("{:x}", Sha256::digest(bytes))))
}

pub async fn write_record(
    connection: &mut PgConnection,
    id: Uuid,
    draft: &Draft,
) -> Result<(), IngestionError> {
    let game_slug: Option<String> = match draft.game_id {
        Some(game) => {
            sqlx::query_scalar("SELECT slug FROM games WHERE id = $1")
                .bind(game)
                .fetch_optional(&mut *connection)
                .await?
        }
        None => None,
    };
    if draft.kind != Kind::Game && game_slug.is_none() {
        return Err(catalogue::ValidationError("Game does not exist".into()).into());
    }
    draft.validate(game_slug.as_deref())?;
    sqlx::query("INSERT INTO catalogue_records (id,kind,game_id,slug,name,normalized_name,series_id,set_id,card_id,collector_number,language,region,rarity,release_date,attributes,synthetic,duplicate_key) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,normalized_name=EXCLUDED.normalized_name,series_id=EXCLUDED.series_id,set_id=EXCLUDED.set_id,card_id=EXCLUDED.card_id,collector_number=EXCLUDED.collector_number,language=EXCLUDED.language,region=EXCLUDED.region,rarity=EXCLUDED.rarity,release_date=EXCLUDED.release_date,attributes=EXCLUDED.attributes,synthetic=EXCLUDED.synthetic,duplicate_key=EXCLUDED.duplicate_key,updated_at=clock_timestamp()")
        .bind(id).bind(draft.kind.as_str()).bind(draft.game_id).bind(&draft.slug).bind(draft.name.trim()).bind(normalize(&draft.name)).bind(draft.series_id).bind(draft.set_id).bind(draft.card_id).bind(&draft.collector_number).bind(&draft.language).bind(&draft.region).bind(&draft.rarity).bind(draft.release_date).bind(&draft.attributes).bind(draft.synthetic).bind(draft.duplicate_key()).execute(&mut *connection).await?;
    let source = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO sources(id,description,reference_url,method,license) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(source)
    .bind(&draft.source.description)
    .bind(&draft.source.reference_url)
    .bind(serde_json::to_value(&draft.source.method)?.as_str())
    .bind(&draft.source.license)
    .execute(&mut *connection)
    .await?;
    let fields = serde_json::to_value(draft)?;
    for field in fields
        .as_object()
        .into_iter()
        .flat_map(|v| v.keys())
        .filter(|k| k.as_str() != "source")
    {
        sqlx::query("INSERT INTO provenance(record_id,source_id,field_path) VALUES($1,$2,$3)")
            .bind(id)
            .bind(source)
            .bind(format!("$.{field}"))
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

pub async fn revision_and_outbox(
    connection: &mut PgConnection,
    id: Uuid,
    contribution: Option<Uuid>,
    before: Option<Value>,
) -> Result<i64, sqlx::Error> {
    let after: Value = sqlx::query_scalar(
        "SELECT to_jsonb(r) - 'search_document' FROM catalogue_records r WHERE id=$1",
    )
    .bind(id)
    .fetch_one(&mut *connection)
    .await?;
    let revision: i64 = sqlx::query_scalar("INSERT INTO catalogue_revisions(record_id,contribution_id,before_data,after_data) VALUES($1,$2,$3,$4) RETURNING revision").bind(id).bind(contribution).bind(&before).bind(&after).fetch_one(&mut *connection).await?;
    let game: Option<String> = sqlx::query_scalar("SELECT slug FROM catalogue_records WHERE kind='game' AND id=(SELECT COALESCE(game_id,id) FROM catalogue_records WHERE id=$1)").bind(id).fetch_optional(&mut *connection).await?;
    let mut paths = vec![
        "/v1/games".to_string(),
        "/v1/stats".into(),
        "/v1/datasets".into(),
    ];
    if let Some(game) = game {
        paths.push(format!("/v1/games/{game}"));
        for collection in ["series", "sets", "cards", "printings"] {
            paths.push(format!("/v1/games/{game}/{collection}"));
        }
        let collection = match after["kind"].as_str() {
            Some("card") => "cards",
            Some("printing") => "printings",
            Some("set") => "sets",
            Some("series") => "series",
            _ => "",
        };
        if !collection.is_empty() {
            paths.push(format!("/v1/games/{game}/{collection}/{id}"));
        }
        for (field, collection) in [("card_id", "cards"), ("set_id", "sets")] {
            if let Some(parent) = after[field].as_str() {
                paths.push(format!("/v1/games/{game}/{collection}/{parent}"));
            }
            if let Some(parent) = before.as_ref().and_then(|v| v[field].as_str()) {
                paths.push(format!("/v1/games/{game}/{collection}/{parent}"));
            }
        }
    }
    paths.sort();
    paths.dedup();
    sqlx::query("INSERT INTO cache_invalidation_outbox(id,revision,paths) VALUES($1,$2,$3)")
        .bind(Uuid::now_v7())
        .bind(revision)
        .bind(json!(paths))
        .execute(connection)
        .await?;
    Ok(revision)
}

pub async fn seed(pool: &PgPool, content: &str) -> Result<usize, IngestionError> {
    let fixtures: Vec<Fixture> = serde_json::from_str(content)?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    let mut inserted = 0;
    for fixture in fixtures {
        if sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM catalogue_records WHERE id=$1)",
        )
        .bind(fixture.id)
        .fetch_one(&mut *tx)
        .await?
        {
            continue;
        }
        write_record(&mut tx, fixture.id, &fixture.draft).await?;
        revision_and_outbox(&mut tx, fixture.id, None, None).await?;
        inserted += 1;
    }
    tx.commit().await?;
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsafe_uploads_rejected() {
        assert!(validate_scan(b"<svg onload=alert(1)>").is_err());
        assert!(validate_scan(&[0; 20]).is_err());
    }
    #[test]
    fn manual_fallback_needs_review() {
        let candidates = ManualExtractor.extract(&[]).unwrap();
        assert!(candidates.requires_human_review);
        assert_eq!(candidates.fields, json!({}));
    }
}
