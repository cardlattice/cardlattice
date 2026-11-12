use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ValidationError(pub String);

/// URL-safe semantic keys; no identifier aliases or legacy lookup path.
pub fn parse_id(value: &str) -> Result<String, ValidationError> {
    if value.is_empty()
        || value.len() > 200
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err(ValidationError(
            "Identifiers must use 1–200 letters, numbers, hyphens, dots or underscores".into(),
        ));
    }
    Ok(value.into())
}

/// Opaque request/audit tokens have no catalogue identity semantics.
pub fn operation_id() -> String {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).expect("operating system entropy");
    format!(
        "event-{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

pub fn slug(value: &str) -> String {
    normalize(value)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn normalize(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Game,
    Series,
    Set,
    Card,
    Printing,
    Product,
    Collection,
    Artwork,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::Series => "series",
            Self::Set => "set",
            Self::Card => "card",
            Self::Printing => "printing",
            Self::Product => "product",
            Self::Collection => "collection",
            Self::Artwork => "artwork",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = CatalogueRecord)]
pub struct Record {
    pub id: String,
    pub kind: String,
    pub game_id: Option<String>,
    pub slug: Option<String>,
    pub name: String,
    pub series_id: Option<String>,
    pub set_id: Option<String>,
    pub card_id: Option<String>,
    pub collector_number: Option<String>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub rarity: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub attributes: Value,
    pub synthetic: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceInput {
    pub description: String,
    pub reference_url: Option<String>,
    pub method: SourceMethod,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceMethod {
    OriginalContribution,
    PhysicalInspection,
    PermittedOfficialReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Draft {
    pub kind: Kind,
    pub name: String,
    pub game_id: Option<String>,
    pub slug: Option<String>,
    pub series_id: Option<String>,
    pub set_id: Option<String>,
    pub card_id: Option<String>,
    pub collector_number: Option<String>,
    pub language: Option<String>,
    pub region: Option<String>,
    pub rarity: Option<String>,
    pub release_date: Option<NaiveDate>,
    #[serde(default = "empty_object")]
    pub attributes: Value,
    #[serde(default)]
    pub synthetic: bool,
    pub source: Option<SourceInput>,
}
fn empty_object() -> Value {
    serde_json::json!({})
}

pub mod pokemon;
pub mod relationships;
pub use pokemon::PokemonAttributes;

impl Draft {
    /// New catalogue identities are readable. Existing edits retain their explicit key.
    pub fn record_id(&self) -> Result<String, ValidationError> {
        let game = self.game_id.as_deref().unwrap_or_default();
        let id = match self.kind {
            Kind::Game => self.slug.clone().unwrap_or_default(),
            Kind::Series => format!("{game}-series-{}", slug(&self.name)),
            Kind::Set => format!("{game}-{}", slug(&self.name)),
            Kind::Product | Kind::Collection | Kind::Artwork => {
                format!("{game}-{}-{}", self.kind.as_str(), slug(&self.name))
            }
            Kind::Card => self
                .attributes
                .get("identityScope")
                .and_then(Value::as_str)
                .map(|scope| format!("card-{scope}"))
                .unwrap_or_else(|| format!("{game}-card-{}", slug(&self.name))),
            Kind::Printing => format!(
                "{}-{}-{}-{}",
                self.set_id.as_deref().unwrap_or_default(),
                self.language.as_deref().unwrap_or_default(),
                self.region.as_deref().unwrap_or("global"),
                self.collector_number.as_deref().unwrap_or_default()
            ),
        };
        parse_id(&id)
    }
    pub fn validate(&self, game_slug: Option<&str>) -> Result<(), ValidationError> {
        let fail = |s: &str| Err(ValidationError(s.into()));
        for id in [&self.game_id, &self.series_id, &self.set_id, &self.card_id]
            .into_iter()
            .flatten()
        {
            parse_id(id)?;
        }
        if self.name != self.name.trim()
            || self.name.trim().is_empty()
            || self.name.chars().count() > 200
            || self.name.chars().any(char::is_control)
        {
            return fail("Name must contain 1–200 printable characters");
        }
        if self.kind != Kind::Game && self.game_id.is_none() {
            return fail("gameId is required");
        }
        if self.kind == Kind::Game
            && self.slug.as_ref().is_none_or(|s| {
                s.is_empty()
                    || s.len() > 60
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            })
        {
            return fail("Game slug must use lowercase ASCII letters, numbers and hyphens");
        }
        if self.kind == Kind::Printing
            && (self.card_id.is_none()
                || self.set_id.is_none()
                || self
                    .collector_number
                    .as_ref()
                    .is_none_or(|s| s.trim().is_empty())
                || self.language.is_none())
        {
            return fail("Printings require cardId, setId, collectorNumber and language");
        }
        if self.kind != Kind::Printing
            && (self.card_id.is_some() || self.set_id.is_some() || self.collector_number.is_some())
        {
            return fail("Printing identity fields are only valid for printings");
        }
        if self.kind != Kind::Set && self.series_id.is_some() {
            return fail("seriesId is only valid for sets");
        }
        if self.kind == Kind::Game && self.game_id.is_some() {
            return fail("Games cannot have a parent game");
        }
        for value in [
            &self.collector_number,
            &self.region,
            &self.rarity,
            &self.slug,
        ]
        .into_iter()
        .flatten()
        {
            if value.trim().is_empty()
                || value != value.trim()
                || value.chars().count() > 100
                || value.chars().any(char::is_control)
            {
                return fail("Metadata fields must be at most 100 bytes and printable");
            }
        }
        if self.region.as_ref().is_some_and(|r| !valid_region(r)) {
            return fail("Use an uppercase two-letter region code");
        }
        if let Some(lang) = &self.language {
            validate_language(lang)?;
        }
        if !self.attributes.is_object() || self.attributes.to_string().len() > 8192 {
            return fail("Attributes must be an object of at most 8 KiB");
        }
        if game_slug == Some("pokemon")
            && self.kind == Kind::Card
            && self.attributes != empty_object()
        {
            let attributes: PokemonAttributes = serde_json::from_value(self.attributes.clone())
                .map_err(|error| {
                    ValidationError(format!("Invalid typed Pokémon attributes: {error}"))
                })?;
            attributes.validate()?;
        }
        if game_slug == Some("pokemon") {
            if let Some(rarity) = &self.rarity {
                serde_json::from_value::<pokemon::vocabulary::Rarity>(Value::String(
                    rarity.clone(),
                ))
                .map_err(|error| ValidationError(format!("Invalid Pokémon rarity: {error}")))?;
            }
            if self.kind == Kind::Printing {
                let attributes: pokemon::PrintingMetadata =
                    serde_json::from_value(self.attributes.clone()).map_err(|error| {
                        ValidationError(format!("Invalid Pokémon printing attributes: {error}"))
                    })?;
                attributes.validate()?;
            }
        }
        relationships::references(self)?;
        if let Some(source) = &self.source {
            if source.description.trim().len() < 10
                || source.description.len() > 2000
                || source.license != "CC0-1.0"
            {
                return fail(
                    "Provide source description (10–2000 bytes) and CC0-1.0 factual metadata dedication",
                );
            }
            if source
                .reference_url
                .as_ref()
                .is_some_and(|url| !url.starts_with("https://") || url.len() > 2000)
            {
                return fail("Reference URLs must use HTTPS and be at most 2000 bytes");
            }
        }
        Ok(())
    }
    pub fn duplicate_key(&self) -> String {
        let identity = match self.kind {
            Kind::Game => format!("game:{}", self.slug.as_deref().unwrap_or_default()),
            Kind::Printing => format!(
                "printing:{:?}:{:?}:{}:{:?}:{:?}:{:?}",
                self.game_id,
                self.set_id,
                normalize(self.collector_number.as_deref().unwrap_or_default()),
                self.language,
                self.region,
                self.release_date
            ),
            // Reviewed localisations may have identical names and rules text.
            Kind::Card if self.attributes.get("identityGroup").is_some() => format!(
                "localised-card:{:?}:{:?}:{}:{}",
                self.game_id,
                self.language,
                normalize(&self.name),
                self.attributes
            ),
            Kind::Card => format!(
                "card:{:?}:{}:{}",
                self.game_id,
                normalize(&self.name),
                self.attributes
            ),
            Kind::Set => format!(
                "set:{:?}:{:?}:{}:{:?}:{:?}:{}",
                self.game_id,
                self.series_id,
                normalize(&self.name),
                self.region,
                self.release_date,
                self.attributes
            ),
            _ => format!(
                "{}:{:?}:{:?}:{}",
                self.kind.as_str(),
                self.game_id,
                self.series_id,
                normalize(&self.name)
            ),
        };
        format!("{:x}", Sha256::digest(identity.as_bytes()))
    }
}

pub fn valid_region(region: &str) -> bool {
    region.len() == 2 && region.bytes().all(|c| c.is_ascii_uppercase())
}
/// Supported canonical BCP 47 subset: language, optional script, optional region.
pub fn validate_language(language: &str) -> Result<(), ValidationError> {
    let mut parts = language.split('-');
    let primary = parts.next().unwrap_or("");
    let mut rest: Vec<_> = parts.collect();
    if rest.first().is_some_and(|s| {
        s.len() == 4
            && s.as_bytes()[0].is_ascii_uppercase()
            && s.bytes().skip(1).all(|b| b.is_ascii_lowercase())
    }) {
        rest.remove(0);
    }
    if !(2..=3).contains(&primary.len())
        || !primary.bytes().all(|b| b.is_ascii_lowercase())
        || rest.len() > 1
        || rest.first().is_some_and(|s| {
            !valid_region(s) && !(s.len() == 3 && s.bytes().all(|b| b.is_ascii_digit()))
        })
    {
        return Err(ValidationError(
            "Use a canonical language tag such as en, en-US or zh-Hant".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListQuery {
    pub limit: Option<u16>,
    pub cursor: Option<String>,
    pub q: Option<String>,
    pub language: Option<String>,
    pub sort: Option<String>,
    pub set_id: Option<String>,
    pub card_id: Option<String>,
    pub product_id: Option<String>,
    pub collection_id: Option<String>,
    pub artwork_id: Option<String>,
    pub rarity: Option<String>,
    pub game: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub key: String,
    pub id: String,
    pub scope: String,
}

impl ListQuery {
    pub fn validate(&self) -> Result<(), ValidationError> {
        for id in [
            &self.set_id,
            &self.card_id,
            &self.product_id,
            &self.collection_id,
            &self.artwork_id,
        ]
        .into_iter()
        .flatten()
        {
            parse_id(id)?;
        }
        if self.limit.is_some_and(|v| v == 0 || v > 100) {
            return Err(ValidationError("limit must be between 1 and 100".into()));
        }
        if self.q.as_ref().is_some_and(|v| {
            v.chars().count() > 120
                || v.split_whitespace().count() > 8
                || v.chars().any(char::is_control)
        }) {
            return Err(ValidationError(
                "Search allows at most 120 characters and 8 terms".into(),
            ));
        }
        if self
            .sort
            .as_ref()
            .is_some_and(|s| s != "name" && s != "newest")
        {
            return Err(ValidationError("sort must be name or newest".into()));
        }
        if self.rarity.as_ref().is_some_and(|v| v.len() > 100)
            || self.game.as_ref().is_some_and(|v| v.len() > 60)
        {
            return Err(ValidationError("Filter is too long".into()));
        }
        if let Some(lang) = &self.language {
            validate_language(lang)?;
        }
        Ok(())
    }
    pub fn scope(&self, path: &str) -> String {
        let mut query = self.clone();
        query.cursor = None;
        query.limit = None;
        format!(
            "{:x}",
            Sha256::digest(format!(
                "{path}:{}",
                serde_json::to_string(&query).unwrap_or_default()
            ))
        )
    }
    pub fn decode_cursor(&self, path: &str) -> Result<Option<Cursor>, ValidationError> {
        self.cursor
            .as_ref()
            .map(|encoded| {
                if encoded.len() > 2048 {
                    return Err(ValidationError("Cursor is too long".into()));
                }
                let bytes = URL_SAFE_NO_PAD
                    .decode(encoded)
                    .map_err(|_| ValidationError("Invalid cursor".into()))?;
                let cursor: Cursor = serde_json::from_slice(&bytes)
                    .map_err(|_| ValidationError("Invalid cursor".into()))?;
                parse_id(&cursor.id)?;
                if cursor.scope != self.scope(path) || cursor.key.len() > 400 {
                    return Err(ValidationError(
                        "Cursor does not belong to this query".into(),
                    ));
                }
                Ok(cursor)
            })
            .transpose()
    }
}

pub fn encode_cursor(key: String, id: String, scope: String) -> String {
    URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&Cursor { key, id, scope }).expect("serializable cursor"))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub data: Vec<Record>,
    pub next_cursor: Option<String>,
    pub limit: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_name_card_attributes_preserve_identity() {
        let mut draft: Draft = serde_json::from_value(serde_json::json!({
            "kind":"card","name":"Synthetic shared name","gameId":operation_id(),
            "attributes":{"category":"pokemon","hp":40},"synthetic":true,
            "source":{"description":"Original synthetic identity test","method":"original_contribution","license":"CC0-1.0"}
        })).unwrap();
        let original = draft.duplicate_key();
        draft.name = "  SYNTHETIC shared name  ".into();
        assert_eq!(original, draft.duplicate_key());
        draft.attributes["hp"] = serde_json::json!(80);
        assert_ne!(original, draft.duplicate_key());
    }
    #[test]
    fn unicode_normalization() {
        assert_eq!(normalize("  ＣＡＲＤ  Cafe\u{301}  "), "card café");
        assert_eq!(normalize("ピカチュウ"), "ピカチュウ");
    }
    #[test]
    fn cursor_scope_and_limits() {
        let mut query = ListQuery::default();
        query.cursor = Some(encode_cursor(
            "café".into(),
            operation_id(),
            query.scope("/cards"),
        ));
        assert!(query.decode_cursor("/cards").unwrap().is_some());
        assert!(query.decode_cursor("/sets").is_err());
        query.limit = Some(101);
        assert!(query.validate().is_err());
        query.limit = Some(10);
        query.q = Some("a ".repeat(9));
        assert!(query.validate().is_err());
    }
    #[test]
    fn rejects_invalid_languages() {
        for lang in ["", "x", "en--us", "en_US", "en/uk"] {
            assert!(validate_language(lang).is_err());
        }
        assert!(validate_language("zh-Hant").is_ok());
    }
}
