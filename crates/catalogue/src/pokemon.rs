//! Pokémon-specific rules and physical printing details. Missing fields remain unknown.
use crate::ValidationError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const TYPES: &[&str] = &[
    "Grass",
    "Fire",
    "Water",
    "Lightning",
    "Psychic",
    "Fighting",
    "Darkness",
    "Metal",
    "Colorless",
    "Dragon",
    "Fairy",
];
fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value == value.trim() && value.chars().count() <= max
}
fn unique(values: &[String]) -> bool {
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
    pub cost: Vec<String>,
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
    pub kind: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeModifier {
    #[serde(rename = "type")]
    pub energy_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PokemonAttributes {
    pub category: Option<String>,
    /// Provisional identity bounded to a printing until shared rules identity is reviewed.
    pub identity_scope: Option<String>,
    pub hp: Option<u16>,
    #[serde(default)]
    pub types: Vec<String>,
    pub stage: Option<String>,
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
    pub subtypes: Option<Vec<String>>,
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
        if self
            .category
            .as_deref()
            .is_some_and(|v| !["pokemon", "trainer", "energy"].contains(&v))
            || self.hp.is_some_and(|v| v == 0 || v > 1000)
            || self.types.len() > 4
            || !unique(&self.types)
            || self.types.iter().any(|v| !TYPES.contains(&v.as_str()))
            || self.stage.as_ref().is_some_and(|v| !text(v, 40))
            || self.evolves_from.as_ref().is_some_and(|v| !text(v, 200))
            || self.retreat_cost.is_some_and(|v| v > 10)
        {
            return Err(invalid());
        }
        if matches!(self.category.as_deref(), Some("trainer" | "energy"))
            && (self.hp.is_some()
                || self.stage.is_some()
                || self.evolves_from.is_some()
                || self.attacks.is_some()
                || self.abilities.is_some()
                || self.weaknesses.is_some()
                || self.resistances.is_some()
                || self.retreat_cost.is_some()
                || self.national_dex.is_some())
        {
            return Err(invalid());
        }
        if self.attacks.as_ref().is_some_and(|attacks| {
            attacks.len() > 6
                || attacks.iter().any(|attack| {
                    !text(&attack.name, 200)
                        || attack.cost.len() > 10
                        || attack.cost.iter().any(|v| !TYPES.contains(&v.as_str()))
                        || attack.text.as_ref().is_some_and(|v| !text(v, 2000))
                        || attack.damage.as_ref().is_some_and(|v| {
                            let digits = v.trim_end_matches(['+', '×', '-']);
                            v.len() > 12
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
                || abilities.iter().any(|v| {
                    !text(&v.name, 200)
                        || !text(&v.text, 2000)
                        || !["Ability", "Pokémon Power", "Poké-Power", "Poké-Body"]
                            .contains(&v.kind.as_str())
                })
        }) {
            return Err(invalid());
        }
        for modifiers in [&self.weaknesses, &self.resistances].into_iter().flatten() {
            if modifiers.len() > 4
                || modifiers.iter().any(|v| {
                    !TYPES.contains(&v.energy_type.as_str())
                        || v.value.chars().count() > 8
                        || !matches!(v.value.chars().next(), Some('×' | '+' | '-'))
                        || v.value.chars().skip(1).count() == 0
                        || !v.value.chars().skip(1).all(|c| c.is_ascii_digit())
                })
            {
                return Err(invalid());
            }
        }
        for values in [&self.subtypes, &self.trainers].into_iter().flatten() {
            if values.len() > 10 || !unique(values) || values.iter().any(|v| !text(v, 100)) {
                return Err(invalid());
            }
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
    pub finishes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regulation_mark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub printed_total: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub illustrators: Option<Vec<String>>,
}
impl PrintingMetadata {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.finishes.as_ref().is_some_and(|v| {
            v.is_empty()
                || v.len() > 3
                || !unique(v)
                || v.iter()
                    .any(|s| !["non-holo", "holo", "reverse-holo"].contains(&s.as_str()))
        }) || self.stamps.as_ref().is_some_and(|v| {
            v.len() > 10
                || !unique(v)
                || v.iter().any(|s| {
                    s.is_empty()
                        || s.len() > 100
                        || !s
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                })
        }) || self
            .regulation_mark
            .as_ref()
            .is_some_and(|v| v.len() != 1 || !v.bytes().all(|b| b.is_ascii_uppercase()))
            || self.printed_total.is_some_and(|v| v == 0 || v > 9999)
            || self.illustrators.as_ref().is_some_and(|v| {
                v.is_empty() || v.len() > 10 || !unique(v) || v.iter().any(|s| !text(s, 200))
            })
        {
            return Err(ValidationError(
                "Invalid printing finishes, stamps or printed details".into(),
            ));
        }
        Ok(())
    }
    /// A supplied list replaces the default; [] explicitly clears inherited stamps.
    pub fn inherit(self, defaults: &Self) -> Self {
        Self {
            finishes: self.finishes.or_else(|| defaults.finishes.clone()),
            stamps: self.stamps.or_else(|| defaults.stamps.clone()),
            regulation_mark: self
                .regulation_mark
                .or_else(|| defaults.regulation_mark.clone()),
            printed_total: self.printed_total.or(defaults.printed_total),
            illustrators: self.illustrators.or_else(|| defaults.illustrators.clone()),
        }
    }
}
