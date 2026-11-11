//! Contributor-facing files. Directory context supplies identity; no executable data.
use crate::bundle::{Entry, Error, ExternalId};
use catalogue::{Draft, Kind, pokemon::PrintingMetadata};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecordFile {
    name: String,
    language: Option<String>,
    region: Option<String>,
    release_date: Option<NaiveDate>,
    series: Option<String>,
    defaults: Option<PrintingMetadata>,
    #[serde(default = "object")]
    attributes: Value,
    #[serde(default)]
    external_ids: Vec<ExternalId>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReleaseFile {
    release_date: Option<NaiveDate>,
    #[serde(default)]
    external_ids: Vec<ExternalIdPattern>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExternalIdPattern {
    namespace: String,
    prefix: String,
    suffix: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PrintingFile {
    name: String,
    /// Explicit reviewed identity for reprints; otherwise generate a provisional identity.
    identity: Option<String>,
    collector_number: Option<String>,
    rarity: Option<String>,
    release_date: Option<NaiveDate>,
    #[serde(default = "object")]
    attributes: Value,
    #[serde(default = "object")]
    printing_attributes: Value,
    pokemon: Option<catalogue::PokemonAttributes>,
    finishes: Option<Vec<catalogue::pokemon::vocabulary::Finish>>,
    stamps: Option<Vec<String>>,
    regulation_mark: Option<catalogue::pokemon::vocabulary::RegulationMark>,
    printed_total: Option<u16>,
    illustrators: Option<Vec<String>>,
    #[serde(default)]
    external_ids: Vec<ExternalId>,
}
fn object() -> Value {
    json!({})
}
fn invalid(path: &Path, message: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("{}: {message}", path.display()))
}
fn decode<T: serde::de::DeserializeOwned>(path: &Path, bytes: &[u8]) -> Result<T, Error> {
    let text = std::str::from_utf8(bytes).map_err(|e| invalid(path, e))?;
    let mut budget = serde_saphyr::Budget::default();
    budget.max_aliases = 0;
    budget.max_anchors = 0;
    budget.max_depth = 32;
    budget.max_events = 20_000;
    let mut options = serde_saphyr::Options::default();
    options.duplicate_keys = serde_saphyr::DuplicateKeyPolicy::Error;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::Error;
    options.reject_unsupported_tags = true;
    options.reject_tagged_field_names = true;
    options.strict_booleans = true;
    options.budget = Some(budget);
    // Preserve YAML scalar types before decoding the domain model. Direct typed
    // decoding permits number-to-string coercion, which would hide damaged identifiers.
    let value: Value =
        serde_saphyr::from_str_with_options(text, options).map_err(|e| invalid(path, e))?;
    serde_json::from_value(value).map_err(|e| invalid(path, e))
}
fn key(path: &Path, key: &str) -> Result<(), Error> {
    catalogue::parse_id(key).map_err(|e| invalid(path, e))?;
    if key == "." || key == ".." {
        return Err(invalid(path, "Invalid path component"));
    }
    Ok(())
}
// Fail closed on stray files, symlinks and excessive trees, including unused inputs.
fn walk(
    root: &Path,
    path: &Path,
    depth: usize,
    total: &mut usize,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), Error> {
    if depth > 10 || files.len() >= 100_000 {
        return Err(invalid(path, "Catalogue tree exceeds bounds"));
    }
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(invalid(path, "Symlinks are forbidden"));
    }
    if meta.is_dir() {
        for entry in std::fs::read_dir(path)? {
            walk(root, &entry?.path(), depth + 1, total, files)?;
        }
    } else if meta.is_file() && path.extension().is_some_and(|s| s == "yaml") {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        *total += bytes.len();
        if bytes.len() > 1024 * 1024 || *total > 64 * 1024 * 1024 {
            return Err(invalid(path, "Catalogue input exceeds size bounds"));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|e| invalid(path, e))?
            .to_path_buf();
        files.insert(relative, bytes);
    } else {
        return Err(invalid(
            path,
            "Only YAML files and directories belong in catalogue",
        ));
    }
    Ok(())
}
fn draft(kind: Kind, name: String, game: Option<&str>) -> Draft {
    Draft {
        kind,
        name,
        game_id: game.map(str::to_owned),
        slug: None,
        series_id: None,
        set_id: None,
        card_id: None,
        collector_number: None,
        language: None,
        region: None,
        rarity: None,
        release_date: None,
        attributes: object(),
        synthetic: false,
        source: None,
    }
}
/// Load a strict tree and expand its inherited context into the common catalogue core.
pub fn load(root: &Path) -> Result<Vec<Entry>, Error> {
    let mut files = BTreeMap::new();
    walk(root, root, 0, &mut 0, &mut files)?;
    let mut records = Vec::new();
    let mut sets = BTreeMap::new();
    let mut releases = BTreeMap::new();
    let mut printing_files = Vec::new();
    for (path, bytes) in &files {
        let parts: Vec<_> = path.iter().map(|s| s.to_str().unwrap_or("")).collect();
        for part in &parts {
            key(path, part)?;
            if let Some(stem) = part.strip_suffix(".yaml") {
                key(path, stem)?;
            }
        }
        let (kind, game, id) = match parts.as_slice() {
            [game, "game.yaml"] => (Kind::Game, *game, game.to_string()),
            [game, "series", file] => (
                Kind::Series,
                *game,
                format!("{game}-series-{}", file.trim_end_matches(".yaml")),
            ),
            [game, "identities", file] => (
                Kind::Card,
                *game,
                format!("{game}-card-{}", file.trim_end_matches(".yaml")),
            ),
            [game, "sets", file] => (
                Kind::Set,
                *game,
                format!("{game}-{}", file.trim_end_matches(".yaml")),
            ),
            [_, "cards", _, _, _, "release.yaml"] => {
                let release: ReleaseFile = decode(path, bytes)?;
                let mut namespaces = HashSet::new();
                if release
                    .external_ids
                    .iter()
                    .any(|id| !namespaces.insert(&id.namespace))
                {
                    return Err(invalid(
                        path,
                        "Duplicate external identifier pattern namespace",
                    ));
                }
                releases.insert(path.parent().unwrap().to_path_buf(), release);
                continue;
            }
            [_, "cards", _, _, _, _] => {
                printing_files.push(path);
                continue;
            }
            _ => return Err(invalid(path, "Unrecognised catalogue path")),
        };
        let file: RecordFile = decode(path, bytes)?;
        if file.defaults.is_some() && kind != Kind::Set {
            return Err(invalid(path, "Printing defaults belong on set files only"));
        }
        let defaults = file.defaults.unwrap_or_default();
        defaults.validate().map_err(|e| invalid(path, e))?;
        let mut record = draft(kind, file.name, (kind != Kind::Game).then_some(game));
        if kind == Kind::Game {
            record.slug = Some(game.into());
        }
        record.language = file.language;
        record.region = file.region;
        record.release_date = file.release_date;
        record.series_id = file.series;
        record.attributes = file.attributes;
        record.validate(Some(game)).map_err(|e| invalid(path, e))?;
        if kind == Kind::Set {
            sets.insert(id.clone(), (record.release_date, defaults));
        }
        records.push(Entry {
            id,
            draft: record,
            external_ids: file.external_ids,
        });
    }
    let mut used_releases = HashSet::new();
    for path in printing_files {
        let parts: Vec<_> = path.iter().map(|s| s.to_str().unwrap()).collect();
        let game = parts[0];
        let set = format!("{game}-{}", parts[2]);
        let language = parts[3];
        let region = parts[4];
        let number = parts[5].trim_end_matches(".yaml");
        let (set_date, set_defaults) = sets
            .get(&set)
            .ok_or_else(|| invalid(path, "Missing set file"))?;
        let release_path = path.parent().unwrap();
        let defaults = ReleaseFile::default();
        let release = releases.get(release_path).unwrap_or(&defaults);
        used_releases.insert(release_path.to_path_buf());
        let mut file: PrintingFile = decode(path, &files[path])?;
        for pattern in &release.external_ids {
            if !file
                .external_ids
                .iter()
                .any(|id| id.namespace == pattern.namespace)
            {
                file.external_ids.push(ExternalId {
                    namespace: pattern.namespace.clone(),
                    value: format!(
                        "{}{}{}",
                        pattern.prefix,
                        file.collector_number.as_deref().unwrap_or(number),
                        pattern.suffix
                    ),
                });
            }
        }
        if let Some(pokemon) = file.pokemon.take() {
            if game != "pokemon" || file.attributes != object() {
                return Err(invalid(
                    path,
                    "pokemon requires the Pokémon game and cannot be mixed with attributes",
                ));
            }
            file.attributes = serde_json::to_value(pokemon)?;
            file.attributes
                .as_object_mut()
                .unwrap()
                .retain(|_, value| !value.is_null());
        }
        let id = format!("{set}-{language}-{region}-{number}");
        let card = if let Some(identity) = file.identity {
            if file.attributes != object() {
                return Err(invalid(
                    path,
                    "Edit attributes on the referenced identity, not its printing",
                ));
            }
            identity
        } else {
            let card_id = format!("card-{id}");
            let mut card = draft(Kind::Card, file.name.clone(), Some(game));
            card.language = Some(language.into());
            card.region = (region != "global").then(|| region.into());
            let mut attributes = file.attributes;
            let attrs = attributes
                .as_object_mut()
                .ok_or_else(|| invalid(path, "attributes must be an object"))?;
            if attrs.contains_key("identityScope") {
                return Err(invalid(
                    path,
                    "identityScope is generated; use identity for a reviewed shared identity",
                ));
            }
            attrs.insert("identityScope".into(), json!(id));
            card.attributes = attributes;
            card.validate(Some(game)).map_err(|e| invalid(path, e))?;
            records.push(Entry {
                id: card_id.clone(),
                draft: card,
                external_ids: vec![],
            });
            card_id
        };
        let mut record = draft(Kind::Printing, file.name, Some(game));
        record.set_id = Some(set);
        record.card_id = Some(card);
        record.collector_number = Some(file.collector_number.unwrap_or_else(|| number.into()));
        record.language = Some(language.into());
        record.region = (region != "global").then(|| region.into());
        record.rarity = file.rarity;
        record.release_date = file.release_date.or(release.release_date).or(*set_date);
        let details = PrintingMetadata {
            finishes: file.finishes,
            stamps: file.stamps,
            regulation_mark: file.regulation_mark,
            printed_total: file.printed_total,
            illustrators: file.illustrators,
        }
        .inherit(set_defaults);
        details.validate().map_err(|e| invalid(path, e))?;
        record.attributes = file.printing_attributes;
        let attributes = record
            .attributes
            .as_object_mut()
            .ok_or_else(|| invalid(path, "printingAttributes must be an object"))?;
        for (key, value) in serde_json::to_value(details)?.as_object().unwrap() {
            if attributes.insert(key.clone(), value.clone()).is_some() {
                return Err(invalid(path, format!("Duplicate printing field {key}")));
            }
        }
        record.validate(Some(game)).map_err(|e| invalid(path, e))?;
        records.push(Entry {
            id,
            draft: record,
            external_ids: file.external_ids,
        });
    }
    for path in releases.keys() {
        if !used_releases.contains(path) {
            return Err(invalid(path, "Release has no printing files"));
        }
    }
    if records.is_empty() {
        return Err(invalid(root, "Catalogue is empty"));
    }
    Ok(records)
}
