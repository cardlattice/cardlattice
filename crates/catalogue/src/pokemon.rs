//! Pokémon-specific rules and physical printing details. Missing fields remain unknown.
use crate::ValidationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod vocabulary;
use vocabulary::{
    AbilityKind, Category, Edition, EnergyType, Finish, RegulationMark, Stage, Subtype,
};

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value == value.trim() && value.chars().count() <= max
}
fn unique<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, value)| !values[..i].contains(value))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attack {
    pub name: String,
    /// Ordered energy symbols; an empty list means a printed free attack.
    pub cost: Vec<EnergyType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ability {
    pub name: String,
    pub text: String,
    pub kind: AbilityKind,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeModifier {
    #[serde(rename = "type")]
    pub energy_type: EnergyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PokemonAttributes {
    pub category: Option<Category>,
    /// Provisional identity bounded to a printing until shared rules identity is reviewed.
    pub identity_scope: Option<String>,
    /// Reviewed identity shared by localised rules records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_group: Option<String>,
    pub hp: Option<u16>,
    /// Printed localised Pokédex flavour text, not an inferred species description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flavour_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_metres: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kilograms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<EnergyType>>,
    pub stage: Option<Stage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolves_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attacks: Option<Vec<Attack>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abilities: Option<Vec<Ability>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weaknesses: Option<Vec<TypeModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resistances: Option<Vec<TypeModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retreat_cost: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtypes: Option<Vec<Subtype>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub national_dex: Option<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trainers: Option<Vec<String>>,
}
impl PokemonAttributes {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let invalid = || ValidationError("Invalid typed Pokémon attributes".into());
        if self.hp.is_some_and(|v| v == 0 || v > 1000)
            || self
                .types
                .as_ref()
                .is_some_and(|v| v.len() > 4 || !unique(v))
            || self.evolves_from.as_ref().is_some_and(|v| !text(v, 200))
            || self.retreat_cost.is_some_and(|v| v > 10)
            || self.flavour_text.as_ref().is_some_and(|v| !text(v, 2000))
            || self.species.as_ref().is_some_and(|v| !text(v, 200))
            || [self.height_metres, self.weight_kilograms]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || !(0.0..=1_000_000.0).contains(&v))
        {
            return Err(invalid());
        }
        if let (Some(category), Some(subtypes)) = (self.category, &self.subtypes)
            && subtypes
                .iter()
                .any(|subtype| !subtype.allowed_for(category))
        {
            return Err(invalid());
        }
        // Fossil Items are Trainers with printed HP and, sometimes, Abilities.
        let trainer_pokemon = self.category == Some(Category::Trainer)
            && self.hp.is_some()
            && self
                .subtypes
                .as_ref()
                .is_none_or(|v| v.is_empty() || v == &[Subtype::Item]);
        if matches!(self.category, Some(Category::Trainer | Category::Energy))
            && ((!trainer_pokemon && (self.hp.is_some() || self.abilities.is_some()))
                || self.stage.is_some()
                || self.evolves_from.is_some()
                || self.attacks.is_some()
                || self.weaknesses.is_some()
                || self.resistances.is_some()
                || self.retreat_cost.is_some()
                || self.national_dex.is_some()
                || self.species.is_some()
                || self.height_metres.is_some()
                || self.weight_kilograms.is_some())
        {
            return Err(invalid());
        }
        if self.attacks.as_ref().is_some_and(|attacks| {
            attacks.len() > 6
                || attacks.iter().any(|attack| {
                    !text(&attack.name, 200)
                        || attack.cost.len() > 10
                        || attack.text.as_ref().is_some_and(|v| !text(v, 2000))
                        || attack.damage.as_ref().is_some_and(|v| {
                            if v == "?" {
                                return false;
                            }
                            let digits = v.trim_end_matches(['+', '×', '-']);
                            v.chars().count() > 12
                                || digits.is_empty()
                                || !digits.bytes().all(|b| b.is_ascii_digit())
                                || v.chars().count() - digits.chars().count() > 1
                        })
                })
        }) {
            return Err(invalid());
        }
        if self.abilities.as_ref().is_some_and(|abilities| {
            abilities.len() > 6
                || abilities
                    .iter()
                    .any(|v| !text(&v.name, 200) || !text(&v.text, 2000))
        }) {
            return Err(invalid());
        }
        for modifiers in [&self.weaknesses, &self.resistances].into_iter().flatten() {
            if modifiers.len() > 4
                || modifiers.iter().any(|v| {
                    v.value.as_ref().is_some_and(|value| {
                        value.chars().count() > 8
                            || !matches!(value.chars().next(), Some('×' | '+' | '-'))
                            || value.chars().skip(1).count() == 0
                            || !value.chars().skip(1).all(|c| c.is_ascii_digit())
                    })
                })
            {
                return Err(invalid());
            }
        }
        if self
            .subtypes
            .as_ref()
            .is_some_and(|values| values.len() > 10 || !unique(values))
            || self.trainers.as_ref().is_some_and(|values| {
                values.len() > 10 || !unique(values) || values.iter().any(|v| !text(v, 100))
            })
        {
            return Err(invalid());
        }
        if self
            .rules
            .as_ref()
            .is_some_and(|v| v.len() > 10 || v.iter().any(|s| !text(s, 2000)))
            || self.national_dex.as_ref().is_some_and(|v| {
                v.len() > 6
                    || v.iter().any(|n| *n == 0 || *n > 9999)
                    || v.iter().enumerate().any(|(i, n)| v[..i].contains(n))
            })
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrintingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finishes: Option<Vec<Finish>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulation_mark: Option<RegulationMark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_denominator: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub illustrators: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<std::collections::BTreeMap<String, PrintingVariant>>,
}
impl PrintingMetadata {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self
            .finishes
            .as_ref()
            .is_some_and(|v| v.len() > 3 || !unique(v))
            || self.stamps.as_ref().is_some_and(|v| {
                v.len() > 10
                    || !unique(v)
                    || v.iter().any(|s| {
                        s.is_empty()
                            || s.len() > 100
                            || !s
                                .bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                    })
            })
            || [&self.products, &self.collections]
                .into_iter()
                .flatten()
                .any(|v| !crate::relationships::references_valid(v))
            || self
                .artwork
                .as_ref()
                .is_some_and(|v| crate::parse_id(v).is_err())
            || self.printed_denominator.is_some_and(|v| v == 0 || v > 9999)
            || self
                .illustrators
                .as_ref()
                .is_some_and(|v| v.len() > 10 || !unique(v) || v.iter().any(|s| !text(s, 200)))
        {
            return Err(ValidationError(
                "Invalid printing finishes, stamps or printed details".into(),
            ));
        }
        if let Some(variants) = &self.variants {
            if variants.is_empty() || variants.len() > 50 {
                return Err(ValidationError("Expected 1–50 explicit variants".into()));
            }
            let mut seen = Vec::new();
            let mut finishes = Vec::new();
            for (key, variant) in variants {
                if key.is_empty()
                    || key.len() > 100
                    || !key
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                {
                    return Err(ValidationError("Invalid variant key".into()));
                }
                variant.validate()?;
                let mut comparison = variant.clone();
                for values in [&mut comparison.stamps, &mut comparison.products]
                    .into_iter()
                    .flatten()
                {
                    values.sort();
                }
                if seen.contains(&comparison) {
                    return Err(ValidationError("Duplicate variant combination".into()));
                }
                seen.push(comparison);
                if !finishes.contains(&variant.finish) {
                    finishes.push(variant.finish);
                }
            }
            if self.finishes.as_ref().is_some_and(|v| {
                v.len() != finishes.len() || v.iter().any(|f| !finishes.contains(f))
            }) {
                return Err(ValidationError(
                    "finishes must match explicit variant finishes".into(),
                ));
            }
        }
        Ok(())
    }
}

/// Each entry is an observed combination; never infer an edition × finish cross product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrintingVariant {
    pub finish: Finish,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<Edition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<String>>,
}
impl PrintingVariant {
    fn validate(&self) -> Result<(), ValidationError> {
        PrintingMetadata {
            stamps: self.stamps.clone(),
            products: self.products.clone(),
            ..Default::default()
        }
        .validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fossil_items_accept_printed_hp_and_abilities_without_relaxing_other_categories() {
        let fossil = json!({"category":"trainer","subtypes":["Item"],"hp":60,"abilities":[{"name":"Domed Armor","kind":"Ability","text":"This Pokémon takes 30 less damage from attacks (after applying Weakness and Resistance)."}]});
        let attributes: PokemonAttributes = serde_json::from_value(fossil.clone()).unwrap();
        attributes.validate().unwrap();
        for patch in [
            json!({"category":"energy"}),
            json!({"subtypes":["Supporter"]}),
            json!({"hp":null}),
            json!({"attacks":[]}),
            json!({"stage":"Basic"}),
        ] {
            let mut invalid = fossil.clone();
            for (key, value) in patch.as_object().unwrap() {
                invalid[key] = value.clone();
            }
            let attributes: PokemonAttributes = serde_json::from_value(invalid).unwrap();
            assert!(attributes.validate().is_err());
        }
    }

    #[test]
    fn localised_printed_pokedex_metadata_is_typed() {
        let value = json!({"category":"pokemon", "flavourText":"Original synthetic flavour text.", "species":"Original synthetic species", "heightMetres":0.3, "weightKilograms":0.1});
        let attributes: PokemonAttributes = serde_json::from_value(value.clone()).unwrap();
        attributes.validate().unwrap();
        assert_eq!(
            serde_json::to_value(attributes).unwrap()["heightMetres"],
            0.3
        );
        for patch in [
            json!({"heightMetres":-1}),
            json!({"weightKilograms":1000001}),
            json!({"species":" "}),
            json!({"flavourText":" "}),
            json!({"category":"trainer"}),
        ] {
            let mut changed = value.clone();
            changed
                .as_object_mut()
                .unwrap()
                .extend(patch.as_object().unwrap().clone());
            assert!(
                serde_json::from_value::<PokemonAttributes>(changed)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn historical_printed_values_and_category_compatibility() {
        for value in [
            json!({"category":"trainer", "hp":10}),
            json!({"category":"pokemon", "attacks":[{"name":"Super Fang", "cost":[], "damage":"?"}], "weaknesses":[{"type":"Fighting"}]}),
            json!({"category":"energy", "subtypes":["Special Energy", "ACE SPEC"]}),
        ] {
            serde_json::from_value::<PokemonAttributes>(value)
                .unwrap()
                .validate()
                .unwrap();
        }
        for value in [
            json!({"category":"pokemon", "subtypes":["Item"]}),
            json!({"category":"trainer", "subtypes":["ex"]}),
            json!({"category":"energy", "subtypes":["Supporter"]}),
        ] {
            assert!(
                serde_json::from_value::<PokemonAttributes>(value)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn enum_spellings_are_strict_at_deserialization() {
        for attributes in [
            json!({"stage":"basic"}),
            json!({"stage":"Stage1"}),
            json!({"stage":"Stage 3"}),
            json!({"subtypes":["item"]}),
            json!({"subtypes":["Suporter"]}),
            json!({"category":"Pokémon"}),
            json!({"types":["grass"]}),
            json!({"attacks":[{"name":"Hit","cost":["Electric"]}]}),
            json!({"weaknesses":[{"type":"fire","value":"×2"}]}),
            json!({"abilities":[{"name":"Effect","text":"Draw.","kind":"ability"}]}),
        ] {
            assert!(
                serde_json::from_value::<PokemonAttributes>(attributes.clone()).is_err(),
                "{attributes}"
            );
        }
        for attributes in [
            json!({"finishes":["normal"]}),
            json!({"regulationMark":"j"}),
            json!({"regulationMark":"Z"}),
        ] {
            assert!(
                serde_json::from_value::<PrintingMetadata>(attributes.clone()).is_err(),
                "{attributes}"
            );
        }
    }

    #[test]
    fn enums_preserve_distinct_mechanics_and_repeated_attack_costs() {
        let value = json!({"stage":"Stage 1","category":"pokemon","subtypes":["ex","EX"],"types":["Grass"],"attacks":[{"name":"Hit","cost":["Grass","Grass"]}]});
        let attributes: PokemonAttributes = serde_json::from_value(value).unwrap();
        attributes.validate().unwrap();
        assert_eq!(attributes.stage, Some(Stage::Stage1));
        assert_eq!(
            attributes.subtypes,
            Some(vec![Subtype::Ex, Subtype::LegacyEx])
        );
        assert_eq!(
            serde_json::to_value(attributes).unwrap()["attacks"][0]["cost"],
            json!(["Grass", "Grass"])
        );
        let duplicate: PokemonAttributes =
            serde_json::from_value(json!({"subtypes":["Item","Item"]})).unwrap();
        assert!(duplicate.validate().is_err());
        let empty: PokemonAttributes = serde_json::from_value(json!({})).unwrap();
        empty.validate().unwrap();
    }

    #[test]
    fn game_specific_validation_covers_drafts_and_printing_attributes() {
        let mut draft: crate::Draft = serde_json::from_value(json!({
            "kind":"printing", "name":"Synthetic", "gameId":"pokemon",
            "cardId":"pokemon-card-example", "setId":"pokemon-example",
            "collectorNumber":"001", "language":"en", "attributes":{}, "rarity":"Common"
        }))
        .unwrap();
        draft.validate(Some("pokemon")).unwrap();
        for rarity in ["Comon", "common", "SIR", "Unknown"] {
            draft.rarity = Some(rarity.into());
            let error = draft.validate(Some("pokemon")).unwrap_err().to_string();
            assert!(error.contains("rarity"), "{error}");
            // Other games retain their own vocabulary; Pokémon rules are not global.
            draft.validate(Some("another-game")).unwrap();
        }
        draft.rarity = None;
        for attributes in [
            json!({"finishes":["normal"]}),
            json!({"finishes":["holo","holo"]}),
            json!({"regulationMark":"Z"}),
        ] {
            draft.attributes = attributes;
            assert!(draft.validate(Some("pokemon")).is_err());
        }
    }
}
