//! Relationships do not change a printing's canonical set or rules identity.
use crate::{Draft, Kind, ValidationError, parse_id, validate_language};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty()
        && value == value.trim()
        && value.chars().count() <= max
        && !value.chars().any(char::is_control)
}
pub fn references_valid(values: &[String]) -> bool {
    values.len() <= 50
        && values
            .iter()
            .enumerate()
            .all(|(i, v)| parse_id(v).is_ok() && !values[..i].contains(v))
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrintedIdentifier {
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Description of the printed symbol, not an image URL or canonical key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}
impl PrintedIdentifier {
    pub fn validate_all(values: &[Self]) -> Result<(), ValidationError> {
        if values.len() > 50
            || values.iter().enumerate().any(|(i, v)| {
                validate_language(&v.language).is_err()
                    || v.region
                        .as_ref()
                        .is_some_and(|r| r.len() != 2 || !r.bytes().all(|b| b.is_ascii_uppercase()))
                    || (v.code.is_none() && v.symbol.is_none())
                    || v.code.as_ref().is_some_and(|c| {
                        c.is_empty()
                            || c.len() > 30
                            || !c.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    })
                    || v.symbol.as_ref().is_some_and(|s| !text(s, 200))
                    || values[..i]
                        .iter()
                        .any(|p| p.language == v.language && p.region == v.region)
            })
        {
            return Err(ValidationError(
                "Invalid or duplicate scoped printed set identifier".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_sets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkMetadata {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub illustrators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
/// Validate typed attributes and return explicit same-game record references.
pub fn references(draft: &Draft) -> Result<Vec<(String, Kind)>, ValidationError> {
    let invalid = |error| ValidationError(format!("Invalid relationship metadata: {error}"));
    let mut refs = Vec::new();
    match draft.kind {
        Kind::Set => {
            if let Some(value) = draft.attributes.get("printedIdentifiers") {
                let marks: Vec<PrintedIdentifier> =
                    serde_json::from_value(value.clone()).map_err(invalid)?;
                PrintedIdentifier::validate_all(&marks)?;
            }
        }
        Kind::Product | Kind::Collection => {
            let value: GroupMetadata =
                serde_json::from_value(draft.attributes.clone()).map_err(invalid)?;
            if !references_valid(&value.related_sets)
                || value.description.as_ref().is_some_and(|v| !text(v, 2000))
            {
                return Err(ValidationError(
                    "Invalid product/collection metadata".into(),
                ));
            }
            refs.extend(value.related_sets.into_iter().map(|id| (id, Kind::Set)));
        }
        Kind::Artwork => {
            let value: ArtworkMetadata =
                serde_json::from_value(draft.attributes.clone()).map_err(invalid)?;
            if value.illustrators.len() > 10
                || value
                    .illustrators
                    .iter()
                    .enumerate()
                    .any(|(i, v)| !text(v, 200) || value.illustrators[..i].contains(v))
                || value.description.as_ref().is_some_and(|v| !text(v, 2000))
            {
                return Err(ValidationError("Invalid artwork metadata".into()));
            }
        }
        Kind::Printing => {
            for (field, kind) in [
                ("products", Kind::Product),
                ("collections", Kind::Collection),
            ] {
                if let Some(value) = draft.attributes.get(field) {
                    let values: Vec<String> =
                        serde_json::from_value(value.clone()).map_err(invalid)?;
                    if !references_valid(&values) {
                        return Err(ValidationError(format!("Invalid {field} references")));
                    }
                    refs.extend(values.into_iter().map(|id| (id, kind)));
                }
            }
            if let Some(value) = draft.attributes.get("artwork") {
                let id: String = serde_json::from_value(value.clone()).map_err(invalid)?;
                parse_id(&id)?;
                refs.push((id, Kind::Artwork));
            }
        }
        _ => {}
    }
    Ok(refs)
}
