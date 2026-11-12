//! Canonical spellings for supported Pokémon metadata. Extend deliberately; no aliases.
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Derive parsing, API schemas and the compiler's vocabulary export from one list.
macro_rules! vocabulary {
    ($name:ident { $($variant:ident => $value:tt),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }
        impl $name {
            pub const VALUES: &'static [&'static str] = &[$($value),+];
        }
    };
}

vocabulary! { Category {
    Pokemon => "pokemon",
    Trainer => "trainer",
    Energy => "energy",
} }

vocabulary! { EnergyType {
    Grass => "Grass",
    Fire => "Fire",
    Water => "Water",
    Lightning => "Lightning",
    Psychic => "Psychic",
    Fighting => "Fighting",
    Darkness => "Darkness",
    Metal => "Metal",
    Colorless => "Colorless",
    Dragon => "Dragon",
    Fairy => "Fairy",
} }

vocabulary! { Stage {
    Basic => "Basic",
    Stage1 => "Stage 1",
    Stage2 => "Stage 2",
    Restored => "Restored",
    Break => "BREAK",
    Mega => "MEGA",
    Vmax => "VMAX",
    Vstar => "VSTAR",
    VUnion => "V-UNION",
} }

vocabulary! { Subtype {
    Ex => "ex",
    LegacyEx => "EX",
    Gx => "GX",
    V => "V",
    Vmax => "VMAX",
    Vstar => "VSTAR",
    VUnion => "V-UNION",
    Break => "BREAK",
    Mega => "Mega",
    TagTeam => "TAG TEAM",
    Tera => "Tera",
    Radiant => "Radiant",
    PrismStar => "Prism Star",
    UltraBeast => "Ultra Beast",
    SingleStrike => "Single Strike",
    RapidStrike => "Rapid Strike",
    FusionStrike => "Fusion Strike",
    Ancient => "Ancient",
    Future => "Future",
    Item => "Item",
    Supporter => "Supporter",
    Stadium => "Stadium",
    PokemonTool => "Pokémon Tool",
    AceSpec => "ACE SPEC",
    BasicEnergy => "Basic Energy",
    SpecialEnergy => "Special Energy",
} }

vocabulary! { Rarity {
    PrintedC => "C",
    PrintedU => "U",
    PrintedR => "R",
    PrintedRR => "RR",
    PrintedSR => "SR",
    PrintedAR => "AR",
    PrintedSAR => "SAR",
    Common => "Common",
    Uncommon => "Uncommon",
    Rare => "Rare",
    DoubleRare => "Double Rare",
    UltraRare => "Ultra Rare",
    IllustrationRare => "Illustration Rare",
    SpecialIllustrationRare => "Special Illustration Rare",
    HyperRare => "Hyper Rare",
    ShinyRare => "Shiny Rare",
    ShinyUltraRare => "Shiny Ultra Rare",
    MegaHyperRare => "Mega Hyper Rare",
    FuturisticRare => "Futuristic Rare",
    PikachuRare => "Pikachu Rare",
    AceSpecRare => "ACE SPEC Rare",
} }

vocabulary! { Finish {
    NonHolo => "non-holo",
    Holo => "holo",
    ReverseHolo => "reverse-holo",
} }

vocabulary! { Edition {
    FirstEdition => "first-edition",
    Unlimited => "unlimited",
} }

impl Subtype {
    pub fn allowed_for(self, category: Category) -> bool {
        match self {
            Self::Item | Self::Supporter | Self::Stadium | Self::PokemonTool => {
                category == Category::Trainer
            }
            Self::AceSpec => matches!(category, Category::Trainer | Category::Energy),
            Self::BasicEnergy | Self::SpecialEnergy => category == Category::Energy,
            Self::PrismStar
            | Self::SingleStrike
            | Self::RapidStrike
            | Self::FusionStrike
            | Self::Ancient
            | Self::Future => true,
            _ => category == Category::Pokemon,
        }
    }
}

vocabulary! { AbilityKind {
    Ability => "Ability",
    PokemonPower => "Pokémon Power",
    PokePower => "Poké-Power",
    PokeBody => "Poké-Body",
} }

vocabulary! { RegulationMark {
    A => "A",
    B => "B",
    C => "C",
    D => "D",
    E => "E",
    F => "F",
    G => "G",
    H => "H",
    I => "I",
    J => "J",
} }

/// Used by the data repository to reject schema/compiler vocabulary drift.
pub fn document() -> serde_json::Value {
    serde_json::json!({
        "category": Category::VALUES,
        "energyType": EnergyType::VALUES,
        "stage": Stage::VALUES,
        "subtype": Subtype::VALUES,
        "rarity": Rarity::VALUES,
        "finish": Finish::VALUES,
        "edition": Edition::VALUES,
        "abilityKind": AbilityKind::VALUES,
        "regulationMark": RegulationMark::VALUES,
    })
}
