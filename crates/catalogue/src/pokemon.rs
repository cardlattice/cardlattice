//! Pokémon-specific rules and physical printing details. Missing fields remain unknown.
use crate::ValidationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod vocabulary;
use vocabulary::{
    AbilityKind, CardSize, Category, CostModifier, Edition, EnergyType, Finish, FoilPattern,
    LegendPart, Power, PrintingError, RegulationMark, Stage, Subtype, VUnionPart,
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
    /// Printed modifier beside the base energy cost; conditional wording remains in text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_modifier: Option<CostModifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Explicit printed power banner, independent of the localised attack name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<Power>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ability {
    pub name: String,
    pub text: String,
    pub kind: AbilityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<Power>,
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
    /// Numeric level printed on this card; LV.X is a separate mechanic.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(minimum = 1, maximum = 100)]
    pub level: Option<u8>,
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
            || self.level.is_some_and(|v| v == 0 || v > 100)
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
        // Tools can grant printed attacks or abilities to the attached Pokémon.
        let pokemon_tool = self.category == Some(Category::Trainer)
            && self
                .subtypes
                .as_ref()
                .is_some_and(|values| values.contains(&Subtype::PokemonTool));
        let technical_machine = self.category == Some(Category::Trainer)
            && self
                .subtypes
                .as_ref()
                .is_some_and(|values| values.contains(&Subtype::TechnicalMachine));
        if matches!(self.category, Some(Category::Trainer | Category::Energy))
            && ((!trainer_pokemon && self.hp.is_some())
                || (!trainer_pokemon && !pokemon_tool && self.abilities.is_some())
                || self.stage.is_some()
                || self.evolves_from.is_some()
                || (!pokemon_tool && !technical_machine && self.attacks.is_some())
                || self.weaknesses.is_some()
                || self.resistances.is_some()
                || self.retreat_cost.is_some()
                || self.national_dex.is_some()
                || self.species.is_some()
                || self.level.is_some()
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
    /// Verified physical printing errors, separate from the card's rules text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printing_errors: Option<Vec<PrintingError>>,
    /// Physical size of this distinct printing when its rules differ from other sizes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<CardSize>,
    /// Physical half; pair membership is represented by a shared collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend_part: Option<LegendPart>,
    /// Physical quadrant; assembly membership is represented by a shared collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_union_part: Option<VUnionPart>,
    /// Derived from the source filename when collectorNumber is explicitly null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unnumbered_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_numbers: Option<AdditionalNumbers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finishes: Option<Vec<Finish>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulation_mark: Option<RegulationMark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_denominator: Option<PrintedDenominator>,
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
            .printing_errors
            .as_ref()
            .is_some_and(|v| v.len() > 10 || !unique(v))
            || self
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
            || self
                .printed_denominator
                .as_ref()
                .is_some_and(|v| !v.valid())
            || self.additional_numbers.as_ref().is_some_and(|v| !v.valid())
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
                if self.size.is_some() && variant.size.is_some() && self.size != variant.size {
                    return Err(ValidationError(
                        "Variant size must match its distinct printing".into(),
                    ));
                }
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
                if let Some(finish) = variant.finish
                    && !finishes.contains(&finish)
                {
                    finishes.push(finish);
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

/// Additional printed numbering never replaces the primary collector number.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdditionalNumbers {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(pattern = "^(0[1-9]|[12][0-9]|30)/30$")]
    pub pikachu30th: Option<String>,
    /// Additional ID explicitly labelled on the physical card, not its collector number.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(pattern = "^[A-Za-z0-9][A-Za-z0-9-]{0,31}$")]
    pub printed_id: Option<String>,
}
impl AdditionalNumbers {
    fn valid(&self) -> bool {
        if self.pikachu30th.is_none() && self.printed_id.is_none() {
            return false;
        }
        let badge_valid = self.pikachu30th.as_ref().is_none_or(|badge| {
            let Some((number, denominator)) = badge.split_once('/') else {
                return false;
            };
            denominator == "30"
                && number.len() == 2
                && number.bytes().all(|b| b.is_ascii_digit())
                && number.parse::<u8>().is_ok_and(|v| (1..=30).contains(&v))
        });
        badge_valid
            && self.printed_id.as_ref().is_none_or(|id| {
                !id.is_empty()
                    && id.len() <= 32
                    && id.as_bytes()[0].is_ascii_alphanumeric()
                    && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            })
    }
}

/// Preserve subset denominators such as H32 and RC25 without changing numeric JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrintedDenominator {
    Number(u16),
    Code(String),
}
impl utoipa::PartialSchema for PrintedDenominator {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        serde_json::from_value(serde_json::json!({"oneOf":[
            {"type":"integer","minimum":1,"maximum":9999},
            {"type":"string","pattern":"^[A-Z]{1,8}[1-9][0-9]{0,3}$"}
        ]}))
        .expect("valid printed denominator schema")
    }
}
impl ToSchema for PrintedDenominator {}
impl PrintedDenominator {
    fn valid(&self) -> bool {
        match self {
            Self::Number(value) => (1..=9999).contains(value),
            Self::Code(value) => {
                let letters = value.bytes().take_while(u8::is_ascii_uppercase).count();
                let digits = &value[letters..];
                (1..=8).contains(&letters)
                    && (1..=4).contains(&digits.len())
                    && !digits.starts_with('0')
                    && digits.bytes().all(|c| c.is_ascii_digit())
            }
        }
    }
}

/// Each entry is an observed combination; never infer an edition × finish cross product.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrintingVariant {
    /// Exact printed flavour text for this physical variant, overriding the card's default bio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flavour_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<CardSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foil_pattern: Option<FoilPattern>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish: Option<Finish>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<Edition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products: Option<Vec<String>>,
}
impl PrintingVariant {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.finish.is_none()
            && self.flavour_text.is_none()
            && self.size.is_none()
            && self.foil_pattern.is_none()
            && self.edition.is_none()
            && self.stamps.as_ref().is_none_or(Vec::is_empty)
            && self.products.as_ref().is_none_or(Vec::is_empty)
        {
            return Err(ValidationError(
                "A variant needs an observed finish, foil pattern, size, edition, stamp or product"
                    .into(),
            ));
        }
        if self.foil_pattern.is_some() && self.finish == Some(Finish::NonHolo) {
            return Err(ValidationError(
                "A foil pattern cannot describe a non-holo variant".into(),
            ));
        }
        if self
            .flavour_text
            .as_ref()
            .is_some_and(|value| !text(value, 2000))
        {
            return Err(ValidationError(
                "Invalid printed variant flavour text".into(),
            ));
        }
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
    fn printed_cost_modifier_round_trips_and_rejects_aliases() {
        let value = json!({"name":"Original attack","cost":["Colorless"],"costModifier":"+"});
        let attack: Attack = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(attack).unwrap(), value);
        for invalid in ["plus", "-", "", " +"] {
            let mut candidate = value.clone();
            candidate["costModifier"] = json!(invalid);
            assert!(serde_json::from_value::<Attack>(candidate).is_err());
        }
    }

    #[test]
    fn printed_power_markers_round_trip_on_attacks_and_abilities() {
        let value = json!({"category":"pokemon","attacks":[{"name":"Original attack","cost":[],"power":"VSTAR"}],"abilities":[{"name":"Original ability","text":"Original effect.","kind":"Ability","power":"VSTAR"}]});
        let attributes: PokemonAttributes = serde_json::from_value(value.clone()).unwrap();
        attributes.validate().unwrap();
        let serialised = serde_json::to_value(attributes).unwrap();
        assert_eq!(serialised["attacks"], value["attacks"]);
        assert_eq!(serialised["abilities"], value["abilities"]);
        for field in ["attacks", "abilities"] {
            for invalid in ["vstar", "V-Star", "Unknown"] {
                let mut candidate = value.clone();
                candidate[field][0]["power"] = json!(invalid);
                assert!(serde_json::from_value::<PokemonAttributes>(candidate).is_err());
            }
        }
    }

    #[test]
    fn pokemon_tools_can_grant_printed_vstar_abilities() {
        let value = json!({"category":"trainer","subtypes":["Item","Pokémon Tool"],"abilities":[{"name":"Original power","text":"Original effect.","kind":"Ability","power":"VSTAR"}]});
        let attributes: PokemonAttributes = serde_json::from_value(value.clone()).unwrap();
        attributes.validate().unwrap();
        for patch in [
            json!({"subtypes":["Item"]}),
            json!({"subtypes":["Supporter"]}),
            json!({"category":"energy"}),
            json!({"hp":30}),
            json!({"stage":"Basic"}),
        ] {
            let mut candidate = value.clone();
            for (key, value) in patch.as_object().unwrap() {
                candidate[key] = value.clone();
            }
            let attributes: PokemonAttributes = serde_json::from_value(candidate).unwrap();
            assert!(attributes.validate().is_err());
        }
    }

    #[test]
    fn observed_editions_do_not_require_guessed_finishes() {
        let value = json!({"variants":{"first-edition":{"edition":"first-edition"}}});
        let metadata: PrintingMetadata = serde_json::from_value(value.clone()).unwrap();
        metadata.validate().unwrap();
        assert_eq!(serde_json::to_value(metadata).unwrap(), value);
        for invalid in [
            json!({"variants":{"unknown":{}}}),
            json!({"variants":{"unknown":{"stamps":[],"products":[]}}}),
            json!({"variants":{"first":{"edition":"first-edition"},"duplicate":{"edition":"first-edition"}}}),
            json!({"finishes":["holo"],"variants":{"first":{"edition":"first-edition"}}}),
        ] {
            let metadata: PrintingMetadata = serde_json::from_value(invalid).unwrap();
            assert!(metadata.validate().is_err());
        }
    }

    #[test]
    fn technical_machine_tools_accept_attacks_without_pokemon_only_stats() {
        let tool = json!({"category":"trainer","subtypes":["Pokémon Tool"],"attacks":[{"name":"Original synthetic attack","cost":["Colorless","Colorless"],"damage":"20"}]});
        let attributes: PokemonAttributes = serde_json::from_value(tool.clone()).unwrap();
        attributes.validate().unwrap();
        for patch in [
            json!({"category":"energy"}),
            json!({"subtypes":["Item"]}),
            json!({"subtypes":["Supporter"]}),
            json!({"subtypes":null}),
            json!({"hp":60}),
            json!({"stage":"Basic"}),
            json!({"retreatCost":0}),
            json!({"attacks":[{"name":" ","cost":[]}]}),
        ] {
            let mut invalid = tool.clone();
            for (key, value) in patch.as_object().unwrap() {
                invalid[key] = value.clone();
            }
            let attributes: PokemonAttributes = serde_json::from_value(invalid).unwrap();
            assert!(attributes.validate().is_err());
        }
    }

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
        let value = json!({"category":"pokemon", "level":12, "flavourText":"Original synthetic flavour text.", "species":"Original synthetic species", "heightMetres":0.3, "weightKilograms":0.1});
        let attributes: PokemonAttributes = serde_json::from_value(value.clone()).unwrap();
        attributes.validate().unwrap();
        assert_eq!(
            serde_json::to_value(attributes).unwrap()["heightMetres"],
            0.3
        );
        for patch in [
            json!({"level":0}),
            json!({"level":101}),
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
    fn additional_printed_numbering_is_scoped_and_bounded() {
        for number in ["01/30", "09/30", "30/30"] {
            let metadata: PrintingMetadata = serde_json::from_value(json!({
                "additionalNumbers":{"pikachu30th":number}
            }))
            .unwrap();
            metadata.validate().unwrap();
        }
        for number in ["00/30", "31/30", "1/30", "01/31", "+1/30", "０1/30"] {
            let metadata: PrintingMetadata = serde_json::from_value(json!({
                "additionalNumbers":{"pikachu30th":number}
            }))
            .unwrap();
            assert!(metadata.validate().is_err());
        }
        for value in [json!({"pikachu30th":9}), json!({"pikachu": "01/30"})] {
            assert!(serde_json::from_value::<AdditionalNumbers>(value).is_err());
        }
        assert!(
            !serde_json::from_value::<AdditionalNumbers>(json!({}))
                .unwrap()
                .valid()
        );
    }

    #[test]
    fn literal_denominators_and_printed_ids_preserve_scalar_types() {
        for value in [
            json!({"printedDenominator":147}),
            json!({"printedDenominator":"H32"}),
            json!({"printedDenominator":"RC25"}),
            json!({"additionalNumbers":{"printedId":"Y-02"}}),
        ] {
            let metadata: PrintingMetadata = serde_json::from_value(value.clone()).unwrap();
            metadata.validate().unwrap();
            assert_eq!(serde_json::to_value(metadata).unwrap(), value);
        }
        for value in [
            json!({"printedDenominator":0}),
            json!({"printedDenominator":"147"}),
            json!({"printedDenominator":"RC0"}),
            json!({"printedDenominator":"RC025"}),
            json!({"printedDenominator":" RC25"}),
            json!({"printedDenominator":"é32"}),
            json!({"additionalNumbers":{"printedId":" Y-02"}}),
        ] {
            let parsed = serde_json::from_value::<PrintingMetadata>(value);
            assert!(parsed.is_err() || parsed.unwrap().validate().is_err());
        }
    }

    #[test]
    fn historical_technical_machines_grant_attacks_but_not_pokemon_stats() {
        let tool = json!({"category":"trainer","subtypes":["Technical Machine"],"attacks":[{"name":"Original synthetic attack","cost":["Colorless"],"damage":"20"}]});
        serde_json::from_value::<PokemonAttributes>(tool.clone())
            .unwrap()
            .validate()
            .unwrap();
        for patch in [
            json!({"category":"pokemon"}),
            json!({"category":"energy"}),
            json!({"subtypes":["Item"]}),
            json!({"abilities":[]}),
            json!({"hp":30}),
            json!({"stage":"Baby"}),
        ] {
            let mut changed = tool.clone();
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
        serde_json::from_value::<PokemonAttributes>(json!({"category":"pokemon","stage":"Baby"}))
            .unwrap()
            .validate()
            .unwrap();
    }

    #[test]
    fn physical_sizes_distinguish_observed_variants() {
        let value = json!({"variants":{"standard":{"size":"standard","finish":"non-holo","stamps":["winner"]},"jumbo":{"size":"jumbo","finish":"non-holo","stamps":["winner"]}}});
        let metadata: PrintingMetadata = serde_json::from_value(value.clone()).unwrap();
        metadata.validate().unwrap();
        assert_eq!(serde_json::to_value(metadata).unwrap(), value);
        let mut duplicate = value;
        duplicate["variants"]["jumbo"]["size"] = json!("standard");
        assert!(
            serde_json::from_value::<PrintingMetadata>(duplicate)
                .unwrap()
                .validate()
                .is_err()
        );
        let mismatched = json!({"size":"jumbo","variants":{"normal":{"size":"standard"}}});
        assert!(
            serde_json::from_value::<PrintingMetadata>(mismatched)
                .unwrap()
                .validate()
                .is_err()
        );
    }

    #[test]
    fn observed_foil_patterns_distinguish_variants_and_reject_non_holo() {
        let value = json!({"variants":{"first":{"finish":"holo","foilPattern":"cosmos"},"second":{"finish":"holo","foilPattern":"cracked-ice"}}});
        let metadata: PrintingMetadata = serde_json::from_value(value.clone()).unwrap();
        metadata.validate().unwrap();
        assert_eq!(serde_json::to_value(metadata).unwrap(), value);
        let invalid = json!({"variants":{"first":{"finish":"non-holo","foilPattern":"cosmos"}}});
        assert!(
            serde_json::from_value::<PrintingMetadata>(invalid)
                .unwrap()
                .validate()
                .is_err()
        );
    }

    #[test]
    fn printed_text_variants_and_graphic_errors_preserve_literal_observations() {
        let value = json!({"printingErrors":["stage-2-evolution-frame"],"variants":{"first":{"finish":"reverse-holo","foilPattern":"mirror","flavourText":"Original synthetic first printing."},"corrected":{"finish":"reverse-holo","foilPattern":"mirror","flavourText":"Original synthetic corrected printing."}}});
        let metadata: PrintingMetadata = serde_json::from_value(value.clone()).unwrap();
        metadata.validate().unwrap();
        assert_eq!(serde_json::to_value(metadata).unwrap(), value);
        for pattern in ["tinsel", "energy-symbol", "sheen"] {
            let observed = json!({"variants":{"foil":{"finish":"holo","foilPattern":pattern}}});
            serde_json::from_value::<PrintingMetadata>(observed)
                .unwrap()
                .validate()
                .unwrap();
        }
        for invalid in [
            json!({"printingErrors":["stage-2-evolution-frame","stage-2-evolution-frame"]}),
            json!({"variants":{"error":{"flavourText":" "}}}),
        ] {
            assert!(
                serde_json::from_value::<PrintingMetadata>(invalid)
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
            json!({"category":"trainer", "subtypes":["Supporter", "TAG TEAM"]}),
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
            json!({"category":"energy", "subtypes":["TAG TEAM"]}),
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
