//! Bounded discovery of factual set metadata from the publisher's public press site.
//! Discovery never writes to the catalogue; publication requires an explicitly reviewed batch.
use crate::{IngestionError, revision_and_outbox, write_record};
use catalogue::{Draft, Kind, Record, SourceInput, SourceMethod};
use chrono::{DateTime, NaiveDate, Utc};
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::{collections::HashSet, time::Duration};

pub const TERMS_URL: &str = "https://press.pokemon.com/en/Assets-Use-Terms";
const PRODUCT_PREFIX: &str = "https://press.pokemon.com/en/products/";
const ROBOTS_URL: &str = "https://press.pokemon.com/robots.txt";
// Reviewed on 2026-11-02: wildcard agent, Crawl-Delay: 5, product pages allowed.
// A changed policy must be reviewed rather than silently accepted.
const ROBOTS_SHA256: &str = "6c7653b64327ac1c3088010c73a20786174046146f48a9e3d9a8d29f3f22d766";
const MAX_PAGES: usize = 24;
const MAX_BYTES: usize = 2 * 1024 * 1024;
const EXTERNAL_NAMESPACE: &str = "pokemon-official-press-product-url";

#[derive(Debug, thiserror::Error)]
pub enum OfficialImportError {
    #[error("Official source validation failed: {0}")]
    Invalid(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Ingestion(#[from] IngestionError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanPlan {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetCandidate {
    pub name: String,
    pub release_date: NaiveDate,
    pub source_url: String,
    pub retrieved_at: DateTime<Utc>,
    pub content_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfficialBatch {
    pub schema_version: u32,
    pub reviewed_by: Option<String>,
    pub terms_url: String,
    pub sets: Vec<SetCandidate>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub applied: bool,
    pub game_created: bool,
    pub game_promoted: bool,
    pub inserted_sets: usize,
    pub updated_sets: usize,
    pub unchanged_sets: usize,
    pub retired_synthetic_records: usize,
    pub rejected_synthetic_contributions: usize,
}

fn invalid(message: &str) -> OfficialImportError {
    OfficialImportError::Invalid(message.into())
}

fn validate_url(url: &str) -> Result<(), OfficialImportError> {
    let slug = url
        .strip_prefix(PRODUCT_PREFIX)
        .ok_or_else(|| invalid("Only official English product pages are allowed"))?;
    if slug.is_empty()
        || slug.len() > 200
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(invalid("Invalid official product path"));
    }
    Ok(())
}

fn field_text(node: &Handle) -> String {
    let mut pending = vec![node.clone()];
    let mut text = String::new();
    while let Some(node) = pending.pop() {
        if let NodeData::Text { contents } = &node.data {
            text.push_str(&contents.borrow());
        }
        pending.extend(node.children.borrow().iter().rev().cloned());
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Read only the product title and explicit release-date fields, never images or prose.
pub fn parse_product(html: &str, source_url: &str) -> Result<SetCandidate, OfficialImportError> {
    validate_url(source_url)?;
    let document = parse_document(RcDom::default(), Default::default()).one(html);
    let mut pending = vec![document.document.clone()];
    let mut titles = Vec::new();
    let mut dates = Vec::new();
    while let Some(node) = pending.pop() {
        if let NodeData::Element { attrs, .. } = &node.data {
            let attrs = attrs.borrow();
            if attrs.iter().any(|attribute| {
                attribute.name.local.as_ref() == "class"
                    && ["infoBoxData", "prod-name"].iter().all(|class| {
                        attribute
                            .value
                            .split_ascii_whitespace()
                            .any(|value| value == *class)
                    })
            }) {
                titles.push(field_text(&node));
            }
            if attrs.iter().any(|attribute| {
                attribute.name.local.as_ref() == "itemprop"
                    && attribute.value.as_ref() == "releaseDate"
            }) {
                dates.push(field_text(&node));
            }
        }
        pending.extend(node.children.borrow().iter().rev().cloned());
    }
    if titles.len() != 1 || dates.len() != 1 {
        return Err(invalid(
            "Missing or ambiguous product fields; page may be blocked or changed",
        ));
    }
    let title = &titles[0];
    let name = title
        .strip_prefix("Pokémon TCG: ")
        .ok_or_else(|| invalid("Page is not an identified Pokémon TCG product"))?
        .to_owned();
    let release_date = NaiveDate::parse_from_str(&dates[0], "%B %e, %Y")
        .map_err(|_| invalid("Release date must be a full, unambiguous calendar date"))?;
    Ok(SetCandidate {
        name,
        release_date,
        source_url: source_url.into(),
        retrieved_at: Utc::now(),
        content_sha256: format!("{:x}", Sha256::digest(html.as_bytes())),
    })
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<String, OfficialImportError> {
    let mut response = client.get(url).send().await?.error_for_status()?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(invalid(
            "Redirects and non-200 source responses require manual review",
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > MAX_BYTES {
            return Err(invalid("Source exceeds the 2 MiB response limit"));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| invalid("Source is not UTF-8"))
}

pub async fn scan(plan: &ScanPlan) -> Result<OfficialBatch, OfficialImportError> {
    if plan.urls.is_empty() || plan.urls.len() > MAX_PAGES {
        return Err(invalid("Select between 1 and 24 reviewed product URLs"));
    }
    let mut urls = HashSet::new();
    for url in &plan.urls {
        validate_url(url)?;
        if !urls.insert(url) {
            return Err(invalid("Duplicate URL in scan plan"));
        }
    }
    let client = reqwest::Client::builder()
        .user_agent("Cardlattice/0.1 (official set metadata research; no artwork downloads)")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .build()?;
    let robots = fetch(&client, ROBOTS_URL).await?;
    if format!("{:x}", Sha256::digest(robots.as_bytes())) != ROBOTS_SHA256 {
        return Err(invalid(
            "robots.txt changed; review its access rules and crawl delay before scanning",
        ));
    }
    let mut sets = Vec::new();
    for url in &plan.urls {
        // Honor the publisher's wildcard Crawl-Delay, including after robots.txt.
        tokio::time::sleep(Duration::from_secs(5)).await;
        let html = fetch(&client, url).await?;
        sets.push(parse_product(&html, url)?);
    }
    Ok(OfficialBatch {
        schema_version: 1,
        reviewed_by: None,
        terms_url: TERMS_URL.into(),
        sets,
    })
}

fn source(candidate: &SetCandidate) -> SourceInput {
    SourceInput {
        description: format!(
            "The Pokémon Company International official product information. Factual title and US release date transcribed for an independent, non-commercial catalogue. Retrieved {}; page SHA-256 {}. Informational-use guidelines: {}. CC0 applies only to Cardlattice's original factual metadata contribution, not the publisher's text, artwork, or trademarks.",
            candidate.retrieved_at.to_rfc3339(),
            candidate.content_sha256,
            TERMS_URL
        ),
        reference_url: Some(candidate.source_url.clone()),
        method: SourceMethod::PermittedOfficialReference,
        license: "CC0-1.0".into(),
    }
}

fn draft(candidate: &SetCandidate, game_id: &str) -> Draft {
    Draft {
        kind: Kind::Set,
        name: candidate.name.clone(),
        game_id: Some(game_id.into()),
        slug: None,
        series_id: None,
        set_id: None,
        card_id: None,
        collector_number: None,
        language: Some("en".into()),
        region: Some("US".into()),
        rarity: None,
        release_date: Some(candidate.release_date),
        attributes: json!({}),
        synthetic: false,
        source: Some(source(candidate)),
    }
}

/// One atomic, idempotent import. The default rolls back after validating every mutation.
pub async fn import(
    pool: &PgPool,
    batch: &OfficialBatch,
    apply: bool,
    retire_demo: bool,
) -> Result<ImportSummary, OfficialImportError> {
    if batch.schema_version != 1
        || batch.terms_url != TERMS_URL
        || batch.sets.is_empty()
        || batch.sets.len() > MAX_PAGES
    {
        return Err(invalid("Unsupported or empty official source batch"));
    }
    if apply
        && batch
            .reviewed_by
            .as_ref()
            .is_none_or(|reviewer| reviewer.trim().len() < 3 || reviewer.len() > 120)
    {
        return Err(invalid(
            "Inspect candidates and source terms, then record reviewedBy before applying",
        ));
    }
    let mut urls = HashSet::new();
    for candidate in &batch.sets {
        validate_url(&candidate.source_url)?;
        if !urls.insert(&candidate.source_url)
            || candidate.content_sha256.len() != 64
            || !candidate
                .content_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid("Duplicate source or invalid page checksum"));
        }
    }
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    let mut summary = ImportSummary {
        applied: apply,
        ..Default::default()
    };
    let mut revisions = Vec::new();
    let existing: Option<Record> =
        sqlx::query_as("SELECT * FROM games WHERE slug='pokemon' FOR UPDATE")
            .fetch_optional(&mut *tx)
            .await?;
    let game_id = existing
        .as_ref()
        .map(|record| record.id.clone())
        .unwrap_or_else(|| "pokemon".into());
    if existing.as_ref().is_none_or(|record| record.synthetic) {
        let mut game = draft(&batch.sets[0], &game_id);
        game.kind = Kind::Game;
        game.name = existing
            .as_ref()
            .map(|record| record.name.clone())
            .unwrap_or_else(|| "Pokémon".into());
        game.game_id = None;
        game.slug = Some("pokemon".into());
        game.language = None;
        game.region = None;
        game.release_date = None;
        game.attributes = existing
            .as_ref()
            .map(|record| record.attributes.clone())
            .unwrap_or_else(|| json!({}));
        let before: Option<Value> = sqlx::query_scalar(
            "SELECT to_jsonb(r)-'search_document' FROM catalogue_records r WHERE id=$1",
        )
        .bind(&game_id)
        .fetch_optional(&mut *tx)
        .await?;
        write_record(&mut tx, &game_id, &game).await?;
        revisions.push(revision_and_outbox(&mut tx, &game_id, None, before).await?);
        summary.game_created = existing.is_none();
        summary.game_promoted = existing.is_some();
    }
    for candidate in &batch.sets {
        let desired = draft(candidate, &game_id);
        let existing: Option<Record> = sqlx::query_as("SELECT r.* FROM catalogue_records r JOIN external_identifiers e ON e.record_id=r.id WHERE e.namespace=$1 AND e.value=$2 FOR UPDATE OF r")
            .bind(EXTERNAL_NAMESPACE).bind(&candidate.source_url).fetch_optional(&mut *tx).await?;
        let id = existing
            .as_ref()
            .map(|record| record.id.clone())
            .unwrap_or(desired.record_id().map_err(|e| invalid(&e.to_string()))?);
        if existing.is_none()
            && sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM catalogue_records WHERE id=$1)",
            )
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?
        {
            return Err(invalid(
                "Semantic set identity exists without its expected source mapping",
            ));
        }
        if let Some(record) = &existing {
            let deleted: bool = sqlx::query_scalar(
                "SELECT deleted_at IS NOT NULL FROM catalogue_records WHERE id=$1",
            )
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;
            if deleted || record.kind != "set" || record.game_id != Some(game_id.clone()) {
                return Err(invalid(
                    "Existing source identity conflicts with an active Pokémon set",
                ));
            }
            // Preserve community-added attributes and series links: this importer owns only
            // name, release date, language/region and the synthetic flag.
            if record.name == desired.name
                && record.release_date == desired.release_date
                && record.language == desired.language
                && record.region == desired.region
                && !record.synthetic
            {
                summary.unchanged_sets += 1;
                continue;
            }
        }
        let mut desired = desired;
        if let Some(record) = &existing {
            desired.attributes = record.attributes.clone();
            desired.series_id = record.series_id.clone();
            desired.rarity = record.rarity.clone();
        }
        let before: Option<Value> = sqlx::query_scalar(
            "SELECT to_jsonb(r)-'search_document' FROM catalogue_records r WHERE id=$1",
        )
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await?;
        write_record(&mut tx, &id, &desired).await?;
        sqlx::query("INSERT INTO external_identifiers(record_id,namespace,value) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(&id).bind(EXTERNAL_NAMESPACE).bind(&candidate.source_url).execute(&mut *tx).await?;
        revisions.push(revision_and_outbox(&mut tx, &id, None, before).await?);
        if existing.is_some() {
            summary.updated_sets += 1;
        } else {
            summary.inserted_sets += 1;
        }
    }
    if retire_demo {
        // Children first. Existing DB constraints abort the whole transaction if any
        // real record still depends on a demo parent; nothing unrelated is cascaded away.
        let records: Vec<(String, Value)> = sqlx::query_as("SELECT id,to_jsonb(r)-'search_document' FROM catalogue_records r WHERE synthetic AND deleted_at IS NULL ORDER BY CASE kind WHEN 'printing' THEN 0 WHEN 'card' THEN 1 WHEN 'set' THEN 2 WHEN 'series' THEN 3 ELSE 4 END,id FOR UPDATE")
            .fetch_all(&mut *tx).await?;
        for (id, before) in records {
            sqlx::query("UPDATE catalogue_records SET deleted_at=clock_timestamp(),updated_at=clock_timestamp() WHERE id=$1").bind(&id).execute(&mut *tx).await?;
            revisions.push(revision_and_outbox(&mut tx, &id, None, Some(before)).await?);
            summary.retired_synthetic_records += 1;
        }
        let pending: Vec<String> = sqlx::query_scalar("SELECT id FROM contributions WHERE status='pending' AND payload->>'synthetic'='true' FOR UPDATE").fetch_all(&mut *tx).await?;
        for id in pending {
            let reviewer = format!(
                "{:x}",
                Sha256::digest(batch.reviewed_by.as_deref().unwrap_or("dry-run").as_bytes())
            );
            sqlx::query("INSERT INTO review_decisions(id,contribution_id,reviewer_hash,decision,reason) VALUES($1,$2,$3,'reject','Operator retired synthetic development data during official set import')")
                .bind(catalogue::operation_id()).bind(&id).bind(reviewer).execute(&mut *tx).await?;
            sqlx::query("UPDATE contributions SET status='rejected',reviewed_at=clock_timestamp() WHERE id=$1").bind(&id).execute(&mut *tx).await?;
            summary.rejected_synthetic_contributions += 1;
        }
    }
    if apply {
        if !revisions.is_empty() || summary.rejected_synthetic_contributions > 0 {
            let import_id = catalogue::operation_id();
            let batch_value = json!(batch);
            let digest = format!("{:x}", Sha256::digest(batch_value.to_string().as_bytes()));
            sqlx::query("INSERT INTO catalogue_imports(id,batch_sha256,reviewed_by,source_policy_url,batch,summary) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(&import_id).bind(digest).bind(&batch.reviewed_by).bind(&batch.terms_url).bind(batch_value).bind(json!(summary)).execute(&mut *tx).await?;
            for revision in revisions {
                sqlx::query(
                    "INSERT INTO catalogue_import_revisions(import_id,revision) VALUES($1,$2)",
                )
                .bind(&import_id)
                .bind(revision)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
    } else {
        tx.rollback().await?;
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    const URL: &str = "https://press.pokemon.com/en/products/Synthetic-parser-test";
    #[test]
    fn only_factual_fields_are_extracted() {
        let page = r#"<h1>Unrelated heading</h1><span class="infoBoxData prod-name">Pok&#233;mon TCG: Original &amp; Synthetic</span><div itemprop=releaseDate> January 2, 2026 </div><img src="never-fetch.png"><p>Never import marketing prose.</p>"#;
        let parsed = parse_product(page, URL).unwrap();
        assert_eq!(parsed.name, "Original & Synthetic");
        assert_eq!(parsed.release_date.to_string(), "2026-01-02");
        assert_eq!(parsed.content_sha256.len(), 64);
    }
    #[test]
    fn blocked_ambiguous_and_foreign_pages_fail_closed() {
        assert!(parse_product("<iframe>Access challenge</iframe>", URL).is_err());
        assert!(validate_url("https://example.com/en/products/test").is_err());
        assert!(validate_url("https://press.pokemon.com/en/products/../Files/Download").is_err());
        assert!(
            validate_url("https://press.pokemon.com/en/products/test?redirect=elsewhere").is_err()
        );
        let partial = r#"<span class="infoBoxData prod-name">Pokémon TCG: Synthetic</span><div itemprop=releaseDate>2026</div>"#;
        assert!(parse_product(partial, URL).is_err());
        let duplicate = r#"<span class="infoBoxData prod-name">Pokémon TCG: Synthetic</span><span class="infoBoxData prod-name">Pokémon TCG: Ambiguous</span><div itemprop=releaseDate>January 2, 2026</div>"#;
        assert!(parse_product(duplicate, URL).is_err());
    }
}
