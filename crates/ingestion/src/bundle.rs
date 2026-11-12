//! Offline CI compiler and transactional consumer for the independent data repository.
use crate::{revision_and_outbox, write_record};
use catalogue::{Draft, Kind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

const MAX_BYTES: usize = 64 * 1024 * 1024;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid catalogue: {0}")]
    Invalid(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Ingestion(#[from] crate::IngestionError),
    #[error(transparent)]
    Validation(#[from] catalogue::ValidationError),
}
fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalId {
    pub namespace: String,
    pub value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub draft: Draft,
    pub external_ids: Vec<ExternalId>,
}
#[derive(Debug, Clone)]
pub struct Bundle {
    pub schema_version: u32,
    pub source_commit: String,
    pub records: Vec<Entry>,
}
// Contributor artifacts contain facts and a Git commit, not source-evidence records.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireBundle {
    schema_version: u32,
    source_commit: String,
    records: Vec<WireEntry>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireEntry {
    id: String,
    metadata: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    external_ids: Vec<ExternalId>,
}
impl Serialize for Bundle {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut records = Vec::new();
        for entry in &self.records {
            let mut metadata =
                serde_json::to_value(&entry.draft).map_err(serde::ser::Error::custom)?;
            metadata.as_object_mut().unwrap().retain(|key, value| {
                key != "source"
                    && !value.is_null()
                    && !(key == "synthetic" && value == false)
                    && !(key == "attributes" && value == &serde_json::json!({}))
            });
            records.push(WireEntry {
                id: entry.id.clone(),
                metadata,
                external_ids: entry.external_ids.clone(),
            });
        }
        WireBundle {
            schema_version: self.schema_version,
            source_commit: self.source_commit.clone(),
            records,
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Bundle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = WireBundle::deserialize(deserializer)?;
        let mut records = Vec::new();
        for entry in wire.records {
            let mut metadata = entry.metadata;
            let object = metadata
                .as_object_mut()
                .ok_or_else(|| serde::de::Error::custom("Record metadata must be an object"))?;
            if object.contains_key("source") {
                return Err(serde::de::Error::custom(
                    "Inline artifact sources are forbidden",
                ));
            }
            records.push(Entry {
                id: entry.id,
                draft: serde_json::from_value(metadata).map_err(serde::de::Error::custom)?,
                external_ids: entry.external_ids,
            });
        }
        Ok(Self {
            schema_version: wire.schema_version,
            source_commit: wire.source_commit,
            records,
        })
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub source_commit: String,
    pub filename: String,
    pub sha256: String,
    pub byte_size: usize,
    pub counts: BTreeMap<String, usize>,
}
fn rank(kind: Kind) -> u8 {
    match kind {
        Kind::Game => 0,
        Kind::Series => 1,
        Kind::Set => 2,
        Kind::Product | Kind::Collection | Kind::Artwork => 3,
        Kind::Card => 4,
        Kind::Printing => 5,
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: &Path) -> Result<Vec<u8>, Error> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("File exceeds 64 MiB"));
    }
    Ok(bytes)
}
impl Bundle {
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1
            || self.source_commit.len() != 40
            || !self.source_commit.bytes().all(|b| b.is_ascii_hexdigit())
            || self.records.len() > 100_000
        {
            return Err(invalid("Unsupported schema, source commit or record count"));
        }
        let entries: HashMap<_, _> = self.records.iter().map(|e| (e.id.clone(), e)).collect();
        if entries.len() != self.records.len() {
            return Err(invalid("Duplicate record identifier"));
        }
        let mut duplicates = HashSet::new();
        let mut external = HashSet::new();
        for entry in &self.records {
            let draft = &entry.draft;
            let game = draft
                .game_id
                .clone()
                .and_then(|id| entries.get(&id))
                .and_then(|e| e.draft.slug.as_deref());
            draft.validate(game)?;
            if catalogue::parse_id(&entry.id).is_err()
                || draft.synthetic
                || draft.name != draft.name.trim()
                || !duplicates.insert(draft.duplicate_key())
            {
                return Err(invalid(&format!(
                    "{}: invalid identifier, synthetic record, whitespace or duplicate identity",
                    entry.id
                )));
            }
            for (parent, kind) in [
                (draft.game_id.clone(), Kind::Game),
                (draft.series_id.clone(), Kind::Series),
                (draft.set_id.clone(), Kind::Set),
                (draft.card_id.clone(), Kind::Card),
            ] {
                if let Some(id) = parent {
                    let target = entries
                        .get(&id)
                        .ok_or_else(|| invalid("Missing parent record"))?;
                    if target.draft.kind != kind
                        || (kind != Kind::Game && target.draft.game_id != draft.game_id)
                    {
                        return Err(invalid("Wrong parent kind or game"));
                    }
                }
            }
            for (id, kind) in catalogue::relationships::references(draft)? {
                let target = entries.get(&id).ok_or_else(|| {
                    invalid(&format!("{}: missing relationship target {id}", entry.id))
                })?;
                if target.draft.kind != kind || target.draft.game_id != draft.game_id {
                    return Err(invalid(&format!(
                        "{}: wrong relationship kind or game for {id}",
                        entry.id
                    )));
                }
            }
            for id in &entry.external_ids {
                if id.namespace.is_empty()
                    || id.namespace.len() > 100
                    || id.value.is_empty()
                    || id.value.len() > 1000
                    || !external.insert((&id.namespace, &id.value))
                {
                    return Err(invalid("Invalid or duplicate external identifier"));
                }
            }
        }
        Ok(())
    }
    pub fn counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for entry in &self.records {
            *counts.entry(entry.draft.kind.as_str().into()).or_default() += 1;
        }
        counts
    }
}
/// Deterministic: no network, database, random identifiers or wall-clock timestamps.
pub fn build(source: &Path, output: &Path, source_commit: &str) -> Result<Manifest, Error> {
    let mut records = super::source_tree::load(source)?;
    records.sort_by_key(|e| (rank(e.draft.kind), e.id.clone()));
    for record in &mut records {
        record
            .external_ids
            .sort_by(|a, b| (&a.namespace, &a.value).cmp(&(&b.namespace, &b.value)));
    }
    let bundle = Bundle {
        schema_version: 1,
        source_commit: source_commit.into(),
        records,
    };
    bundle.validate()?;
    let bytes = serde_json::to_vec(&bundle)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("Compiled catalogue exceeds 64 MiB"));
    }
    let sha256 = hash(&bytes);
    let manifest = Manifest {
        schema_version: 1,
        source_commit: source_commit.into(),
        filename: format!("catalogue-{sha256}.json"),
        sha256,
        byte_size: bytes.len(),
        counts: bundle.counts(),
    };
    std::fs::create_dir_all(output)?;
    std::fs::write(output.join(&manifest.filename), bytes)?;
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}
pub fn load(manifest_path: &Path) -> Result<Bundle, Error> {
    Ok(load_artifact(manifest_path)?.0)
}
pub fn load_artifact(manifest_path: &Path) -> Result<(Bundle, Manifest, Vec<u8>), Error> {
    let manifest: Manifest = serde_json::from_slice(&read(manifest_path)?)?;
    if manifest.schema_version != 1
        || manifest.sha256.len() != 64
        || !manifest.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || manifest.filename != format!("catalogue-{}.json", manifest.sha256)
    {
        return Err(invalid("Invalid manifest or artifact filename"));
    }
    let bytes = read(
        &manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(&manifest.filename),
    )?;
    if bytes.len() != manifest.byte_size || hash(&bytes) != manifest.sha256 {
        return Err(invalid("Artifact checksum or size mismatch"));
    }
    let bundle: Bundle = serde_json::from_slice(&bytes)?;
    bundle.validate()?;
    if bundle.source_commit != manifest.source_commit || bundle.counts() != manifest.counts {
        return Err(invalid("Manifest does not describe this catalogue"));
    }
    Ok((bundle, manifest, bytes))
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub applied: bool,
    pub inserted: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub retired: usize,
}

/// The caller selects a trusted CI artifact. Checksums detect corruption, not authorship.
/// Index updates, audit revisions and invalidation commit together. Existing records
/// may be adopted only when identical; later local moderation conflicts fail closed.
pub async fn index(pool: &PgPool, bundle: &Bundle, apply: bool) -> Result<Summary, Error> {
    bundle.validate()?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("LOCK TABLE catalogue_records IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *tx)
        .await?;
    let mut summary = Summary {
        applied: apply,
        ..Default::default()
    };
    let mut revisions = Vec::new();
    let mut entries: Vec<_> = bundle.records.iter().collect();
    entries.sort_by_key(|e| (rank(e.draft.kind), e.id.clone()));
    let ids: Vec<_> = entries.iter().map(|e| e.id.clone()).collect();
    for entry in entries {
        let before: Option<Value> = sqlx::query_scalar(
            "SELECT to_jsonb(r)-'search_document' FROM catalogue_records r WHERE id=$1",
        )
        .bind(&entry.id)
        .fetch_optional(&mut *tx)
        .await?;
        let saved: Option<Value> = sqlx::query_scalar(
            "SELECT indexed_record FROM catalogue_index_records WHERE record_id=$1",
        )
        .bind(&entry.id)
        .fetch_optional(&mut *tx)
        .await?;
        if before.as_ref().is_some_and(|r| !r["deleted_at"].is_null()) {
            return Err(invalid("Retired identifier cannot be resurrected"));
        }
        if let (Some(saved), Some(before)) = (&saved, &before)
            && saved != before
        {
            return Err(invalid(
                "Record changed outside the data repository; reconcile before indexing",
            ));
        }
        let same = if before.is_some() {
            let record: catalogue::Record =
                sqlx::query_as("SELECT * FROM catalogue_records WHERE id=$1")
                    .bind(&entry.id)
                    .fetch_one(&mut *tx)
                    .await?;
            if record.kind != entry.draft.kind.as_str()
                || record.game_id != entry.draft.game_id
                || record.slug != entry.draft.slug
            {
                return Err(invalid("Record kind, game and slug are immutable"));
            }
            let source: Value = sqlx::query_scalar("SELECT jsonb_build_object('description',s.description,'referenceUrl',s.reference_url,'method',s.method,'license',s.license) FROM sources s WHERE EXISTS(SELECT 1 FROM provenance p WHERE p.source_id=s.id AND p.record_id=$1) ORDER BY s.recorded_at DESC,s.id DESC LIMIT 1").bind(&entry.id).fetch_optional(&mut *tx).await?.unwrap_or(Value::Null);
            let mut current = serde_json::to_value(record)?;
            for key in ["id", "createdAt", "updatedAt"] {
                current.as_object_mut().unwrap().remove(key);
            }
            current["source"] = source;
            let same = current == serde_json::to_value(&entry.draft)?;
            if !same && saved.is_none() {
                return Err(invalid(
                    "Existing record differs from the first indexed catalogue",
                ));
            }
            same
        } else {
            false
        };
        let existing_external: Vec<(String,String)> = sqlx::query_as("SELECT namespace,value FROM external_identifiers WHERE record_id=$1 ORDER BY namespace,value").bind(&entry.id).fetch_all(&mut *tx).await?;
        let mut desired_external: Vec<_> = entry
            .external_ids
            .iter()
            .map(|e| (e.namespace.clone(), e.value.clone()))
            .collect();
        desired_external.sort();
        if before.is_some() && saved.is_none() && existing_external != desired_external {
            return Err(invalid("Existing external identifiers differ"));
        }
        if !same || existing_external != desired_external {
            write_record(&mut tx, &entry.id, &entry.draft).await?;
            sqlx::query("DELETE FROM external_identifiers WHERE record_id=$1")
                .bind(&entry.id)
                .execute(&mut *tx)
                .await?;
            for id in &entry.external_ids {
                sqlx::query(
                    "INSERT INTO external_identifiers(record_id,namespace,value) VALUES($1,$2,$3)",
                )
                .bind(&entry.id)
                .bind(&id.namespace)
                .bind(&id.value)
                .execute(&mut *tx)
                .await?;
            }
            revisions.push(revision_and_outbox(&mut tx, &entry.id, None, before.clone()).await?);
            if before.is_some() {
                summary.updated += 1;
            } else {
                summary.inserted += 1;
            }
        } else {
            summary.unchanged += 1;
        }
        sqlx::query("INSERT INTO catalogue_index_records(record_id,indexed_record,source_commit) SELECT id,to_jsonb(r)-'search_document',$2 FROM catalogue_records r WHERE id=$1 ON CONFLICT(record_id) DO UPDATE SET indexed_record=EXCLUDED.indexed_record,source_commit=EXCLUDED.source_commit").bind(&entry.id).bind(&bundle.source_commit).execute(&mut *tx).await?;
    }
    let retired: Vec<(String,Value,Value)> = sqlx::query_as("SELECT r.id,to_jsonb(r)-'search_document',i.indexed_record FROM catalogue_index_records i JOIN catalogue_records r ON r.id=i.record_id WHERE NOT(r.id=ANY($1)) AND r.deleted_at IS NULL ORDER BY CASE r.kind WHEN 'printing' THEN 0 WHEN 'card' THEN 1 WHEN 'product' THEN 2 WHEN 'collection' THEN 2 WHEN 'artwork' THEN 2 WHEN 'set' THEN 3 WHEN 'series' THEN 4 ELSE 5 END,r.id").bind(&ids).fetch_all(&mut *tx).await?;
    for (id, before, saved) in retired {
        if before != saved {
            return Err(invalid(
                "Locally changed record cannot be retired by an artifact",
            ));
        }
        sqlx::query("UPDATE catalogue_records SET deleted_at=clock_timestamp(),updated_at=clock_timestamp() WHERE id=$1").bind(&id).execute(&mut *tx).await?;
        revisions.push(revision_and_outbox(&mut tx, &id, None, Some(before)).await?);
        sqlx::query("DELETE FROM catalogue_index_records WHERE record_id=$1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        summary.retired += 1;
    }
    if let Some(revision) = revisions.last() {
        let paths: Value = sqlx::query_scalar("SELECT jsonb_agg(DISTINCT p.value ORDER BY p.value) FROM cache_invalidation_outbox o CROSS JOIN LATERAL jsonb_array_elements(o.paths) p WHERE o.revision=ANY($1)").bind(&revisions).fetch_one(&mut *tx).await?;
        sqlx::query("DELETE FROM cache_invalidation_outbox WHERE revision=ANY($1)")
            .bind(&revisions)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO cache_invalidation_outbox(id,revision,paths) VALUES($1,$2,$3)")
            .bind(catalogue::operation_id())
            .bind(revision)
            .bind(paths)
            .execute(&mut *tx)
            .await?;
    }
    if apply {
        tx.commit().await?;
    } else {
        tx.rollback().await?;
    }
    Ok(summary)
}
