//! Reviewed, factual checklist imports. No artwork, catalogue dumps or name-based merging.
use crate::{official_pokemon::OfficialImportError, revision_and_outbox, write_record};
use catalogue::{Draft, Kind, Record, SourceInput, SourceMethod};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

pub const TERMS_URL: &str = "https://www.pokemon.com/us/legal/";
const NAMESPACE: &str = "pokemon-official-numbered-checklist";
const PREFIX: &str = "https://d1i787aglh9bmb.cloudfront.net/assets/img/me-expansions/";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CardCandidate {
    pub collector_number: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Checklist {
    pub set_source_url: String,
    pub source_url: String,
    pub retrieved_at: DateTime<Utc>,
    pub content_sha256: String,
    pub scope: String,
    pub expected_count: usize,
    pub cards: Vec<CardCandidate>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CardBatch {
    pub schema_version: u32,
    pub reviewed_by: Option<String>,
    pub terms_url: String,
    pub sets: Vec<Checklist>,
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub applied: bool,
    pub inserted_cards: usize,
    pub inserted_printings: usize,
    pub unchanged_printings: usize,
}
fn invalid(message: &str) -> OfficialImportError {
    OfficialImportError::Invalid(message.into())
}
impl CardBatch {
    pub fn validate(&self) -> Result<(), OfficialImportError> {
        if self.schema_version != 1
            || self.terms_url != TERMS_URL
            || self
                .reviewed_by
                .as_ref()
                .is_none_or(|s| s.trim().len() < 3 || s.len() > 200)
            || self.sets.is_empty()
            || self.sets.len() > 24
        {
            return Err(invalid(
                "A bounded, explicitly reviewed checklist batch is required",
            ));
        }
        let mut sources = HashSet::new();
        for set in &self.sets {
            let path = set.source_url.strip_prefix(PREFIX).ok_or_else(|| {
                invalid("Only reviewed publisher-hosted checklist PDFs are allowed")
            })?;
            if !path.ends_with(".pdf")
                || path.contains("..")
                || !path
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"/-_.".contains(&b))
                || !set
                    .set_source_url
                    .starts_with("https://press.pokemon.com/en/products/")
                || !sources.insert(&set.set_source_url)
                || set.scope != "main-numbered-checklist"
                || set.expected_count == 0
                || set.expected_count > 500
                || set.cards.len() != set.expected_count
                || set.content_sha256.len() != 64
                || !set.content_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || set.retrieved_at > Utc::now()
            {
                return Err(invalid("Invalid checklist source, scope, count or digest"));
            }
            for (index, card) in set.cards.iter().enumerate() {
                if card.collector_number != format!("{:03}", index + 1)
                    || card.name.trim().is_empty()
                    || card.name != card.name.trim()
                    || card.name.chars().count() > 200
                    || card.name.chars().any(char::is_control)
                {
                    return Err(invalid(
                        "Checklist must have unique consecutive numbers and valid names",
                    ));
                }
            }
        }
        Ok(())
    }
}

pub async fn import(
    pool: &PgPool,
    batch: &CardBatch,
    apply: bool,
) -> Result<Summary, OfficialImportError> {
    batch.validate()?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    let mut summary = Summary {
        applied: apply,
        ..Default::default()
    };
    let mut revisions = Vec::new();
    for checklist in &batch.sets {
        let set: Record = sqlx::query_as("SELECT s.* FROM sets s JOIN external_identifiers e ON e.record_id=s.id JOIN games g ON g.id=s.game_id WHERE e.namespace='pokemon-official-press-product-url' AND e.value=$1 AND g.slug='pokemon' AND NOT s.synthetic AND NOT g.synthetic FOR UPDATE OF s")
            .bind(&checklist.set_source_url).fetch_optional(&mut *tx).await?.ok_or_else(|| invalid("Import the referenced real Pokémon set before its checklist"))?;
        let source = SourceInput {
            description: format!(
                "Names and collector numbers independently transcribed from the publisher's English checklist, main numbered section only. Retrieved {}; PDF SHA-256 {}. Energy symbols expanded to text after visual review. Rules, rarity, variants and artwork are not inferred. Card identities are provisional per printing pending rules review. CC0 covers only our factual metadata contribution, not the PDF, artwork or trademarks.",
                checklist.retrieved_at.to_rfc3339(),
                checklist.content_sha256
            ),
            reference_url: Some(checklist.source_url.clone()),
            method: SourceMethod::PermittedOfficialReference,
            license: "CC0-1.0".into(),
        };
        for candidate in &checklist.cards {
            // The publisher URL is a secondary lookup key, never a canonical record ID.
            let key = format!(
                "{}#main:{}:en:US",
                checklist.set_source_url, candidate.collector_number
            );
            let existing: Option<Record> = sqlx::query_as("SELECT r.* FROM catalogue_records r JOIN external_identifiers e ON e.record_id=r.id WHERE e.namespace=$1 AND e.value=$2 AND r.deleted_at IS NULL FOR UPDATE OF r")
                .bind(NAMESPACE).bind(&key).fetch_optional(&mut *tx).await?;
            if let Some(existing) = existing {
                if existing.kind != "printing"
                    || existing.set_id != Some(set.id)
                    || existing.name != candidate.name
                    || existing.collector_number.as_ref() != Some(&candidate.collector_number)
                    || existing.language.as_deref() != Some("en")
                    || existing.region.as_deref() != Some("US")
                    || existing.synthetic
                {
                    return Err(invalid(
                        "Existing checklist identity differs; use the correction review workflow",
                    ));
                }
                summary.unchanged_printings += 1;
                continue;
            }
            let card_id = Uuid::now_v7();
            let printing_id = Uuid::now_v7();
            let mut draft = Draft {
                kind: Kind::Card,
                name: candidate.name.clone(),
                game_id: set.game_id,
                slug: None,
                series_id: None,
                set_id: None,
                card_id: None,
                collector_number: None,
                language: Some("en".into()),
                region: Some("US".into()),
                rarity: None,
                release_date: None,
                attributes: json!({"identityScope": printing_id}),
                synthetic: false,
                source: source.clone(),
            };
            write_record(&mut tx, card_id, &draft).await?;
            revisions.push(revision_and_outbox(&mut tx, card_id, None, None).await?);
            draft.kind = Kind::Printing;
            draft.set_id = Some(set.id);
            draft.card_id = Some(card_id);
            draft.collector_number = Some(candidate.collector_number.clone());
            draft.release_date = set.release_date;
            draft.attributes = json!({});
            write_record(&mut tx, printing_id, &draft).await?;
            revisions.push(revision_and_outbox(&mut tx, printing_id, None, None).await?);
            sqlx::query(
                "INSERT INTO external_identifiers(record_id,namespace,value) VALUES($1,$2,$3)",
            )
            .bind(printing_id)
            .bind(NAMESPACE)
            .bind(key)
            .execute(&mut *tx)
            .await?;
            summary.inserted_cards += 1;
            summary.inserted_printings += 1;
        }
    }
    if !revisions.is_empty() {
        // All these events are uncommitted and invisible to workers. Consolidate their
        // deduplicated paths atomically, retaining every individual catalogue revision.
        // The existing purger splits URLs into 30-URL requests and retries failures.
        let paths: Value = sqlx::query_scalar("SELECT jsonb_agg(DISTINCT p.value ORDER BY p.value) FROM cache_invalidation_outbox o CROSS JOIN LATERAL jsonb_array_elements(o.paths) p WHERE o.revision=ANY($1)")
            .bind(&revisions).fetch_one(&mut *tx).await?;
        sqlx::query("DELETE FROM cache_invalidation_outbox WHERE revision=ANY($1)")
            .bind(&revisions)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO cache_invalidation_outbox(id,revision,paths) VALUES($1,$2,$3)")
            .bind(Uuid::now_v7())
            .bind(revisions.last())
            .bind(paths)
            .execute(&mut *tx)
            .await?;
        let import_id = Uuid::now_v7();
        let value = json!(batch);
        let digest = format!("{:x}", Sha256::digest(value.to_string().as_bytes()));
        sqlx::query("INSERT INTO catalogue_imports(id,batch_sha256,reviewed_by,source_policy_url,batch,summary) VALUES($1,$2,$3,$4,$5,$6)").bind(import_id).bind(digest).bind(&batch.reviewed_by).bind(&batch.terms_url).bind(value).bind(json!(summary)).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO catalogue_import_revisions(import_id,revision) SELECT $1,unnest($2::bigint[])").bind(import_id).bind(&revisions).execute(&mut *tx).await?;
    }
    if apply {
        tx.commit().await?;
    } else {
        tx.rollback().await?;
    }
    Ok(summary)
}
