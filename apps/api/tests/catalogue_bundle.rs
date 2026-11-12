use ingestion::bundle::{self, Bundle, Entry};
use sqlx::PgPool;

fn input() -> Bundle {
    let fixtures: Vec<ingestion::Fixture> =
        serde_json::from_str(include_str!("../../../data/development.json")).unwrap();
    Bundle {
        schema_version: 1,
        source_commit: "a".repeat(40),
        records: fixtures
            .into_iter()
            .map(|f| {
                let mut draft = f.draft;
                // Original synthetic test content is marked publishable only inside this test.
                draft.synthetic = false;
                Entry {
                    id: f.id,
                    draft,
                    external_ids: vec![],
                }
            })
            .collect(),
    }
}

#[test]
fn reproducible_build_checks_manifest_integrity_and_relations() {
    let source = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let mut bundle = source_fixture(source.path());
    let manifest = bundle::build(source.path(), first.path(), &bundle.source_commit).unwrap();
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        "# A contributor comment does not change the artifact\nrarity: null\nname: Prism Seed\n",
    );
    let again = bundle::build(source.path(), second.path(), &bundle.source_commit).unwrap();
    assert_eq!(manifest.sha256, again.sha256);
    let loaded = bundle::load(&first.path().join("manifest.json")).unwrap();
    assert_eq!(loaded.records.len(), bundle.records.len());
    std::fs::write(first.path().join(manifest.filename), b"{}").unwrap();
    assert!(bundle::load(&first.path().join("manifest.json")).is_err());
    bundle.records[0].draft.synthetic = true;
    assert!(bundle.validate().is_err());
    bundle = input();
    bundle.records.push(bundle.records[0].clone());
    assert!(bundle.validate().is_err());
    bundle = input();
    bundle
        .records
        .retain(|e| e.draft.kind != catalogue::Kind::Game);
    assert!(bundle.validate().is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexing_is_atomic_idempotent_and_detects_local_conflicts(pool: PgPool) {
    let mut bundle = input();
    let card = bundle
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap();
    card.draft.attributes["attacks"] = serde_json::json!([
        {"name": "Prism Echo", "cost": ["Psychic", "Psychic"], "damage": "20×"}
    ]);
    let card_id = card.id.clone();
    let printing = bundle
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Printing)
        .unwrap();
    printing.draft.attributes = serde_json::json!({"finishes": ["holo"], "stamps": ["laboratory-anniversary"], "printedDenominator": 128});
    let printing_id = printing.id.clone();
    let dry = bundle::index(&pool, &bundle, false).await.unwrap();
    assert_eq!(dry.inserted, bundle.records.len());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM catalogue_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let applied = bundle::index(&pool, &bundle, true).await.unwrap();
    assert_eq!(applied.inserted, bundle.records.len());
    let attributes: serde_json::Value =
        sqlx::query_scalar("SELECT attributes FROM catalogue_records WHERE id=$1")
            .bind(&card_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attributes["attacks"][0]["damage"], "20×");
    assert_eq!(
        attributes["attacks"][0]["cost"],
        serde_json::json!(["Psychic", "Psychic"])
    );
    let attributes: serde_json::Value =
        sqlx::query_scalar("SELECT attributes FROM catalogue_records WHERE id=$1")
            .bind(&printing_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attributes["finishes"], serde_json::json!(["holo"]));
    assert_eq!(
        attributes["stamps"],
        serde_json::json!(["laboratory-anniversary"])
    );
    let repeat = bundle::index(&pool, &bundle, true).await.unwrap();
    assert_eq!(repeat.unchanged, bundle.records.len());
    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM cache_invalidation_outbox")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events, 1);
    let card = bundle
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap();
    let id = card.id.clone();
    card.draft.name = "Original Flabébé ♀ correction".into();
    bundle.source_commit = "b".repeat(40);
    assert_eq!(
        bundle::index(&pool, &bundle, true).await.unwrap().updated,
        1
    );
    sqlx::query("UPDATE catalogue_records SET updated_at=clock_timestamp() WHERE id=$1")
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(bundle::index(&pool, &bundle, true).await.is_err());
    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM cache_invalidation_outbox")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn retirement_removes_only_managed_records_and_first_adoption_is_safe(pool: PgPool) {
    let bundle = input();
    let mut tx = pool.begin().await.unwrap();
    for entry in &bundle.records {
        ingestion::write_record(&mut tx, &entry.id, &entry.draft)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    let mut conflict = bundle.clone();
    conflict.records[0].draft.name = "Different preexisting record".into();
    assert!(bundle::index(&pool, &conflict, true).await.is_err());
    assert_eq!(
        bundle::index(&pool, &bundle, true).await.unwrap().unchanged,
        bundle.records.len()
    );
    let empty = Bundle {
        records: vec![],
        ..bundle
    };
    assert!(bundle::index(&pool, &empty, false).await.unwrap().retired > 0);
    assert!(bundle::index(&pool, &empty, true).await.unwrap().retired > 0);
    assert_eq!(bundle::index(&pool, &empty, true).await.unwrap().retired, 0);
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexed_download_is_the_exact_ci_artifact(pool: PgPool) {
    let source = tempfile::tempdir().unwrap();
    let built = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let bundle = source_fixture(source.path());
    let manifest = bundle::build(source.path(), built.path(), &bundle.source_commit).unwrap();
    let path = built.path().join("manifest.json");
    cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), false)
        .await
        .unwrap();
    assert!(!downloads.path().join(&manifest.filename).exists());
    cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), true)
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(built.path().join(&manifest.filename)).unwrap(),
        std::fs::read(downloads.path().join(&manifest.filename)).unwrap()
    );
    assert_eq!(
        cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), true)
            .await
            .unwrap()
            .unchanged,
        bundle.records.len()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM publication_manifests")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

fn write(root: &std::path::Path, path: &str, content: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
fn source_fixture(root: &std::path::Path) -> Bundle {
    write(root, "pokemon/game.yaml", "name: Pokémon\n");
    write(
        root,
        "pokemon/sets/laboratory.yaml",
        "name: Laboratory\nreleaseDate: '2026-01-01'\n",
    );
    write(
        root,
        "pokemon/cards/laboratory/en/US/001.yaml",
        "name: Prism Seed\n",
    );
    Bundle {
        schema_version: 1,
        source_commit: "a".repeat(40),
        records: ingestion::source_tree::load(root).unwrap(),
    }
}

#[test]
fn source_tree_inherits_context_without_evidence() {
    let source = tempfile::tempdir().unwrap();
    let bundle = source_fixture(source.path());
    bundle.validate().unwrap();
    let printing = bundle
        .records
        .iter()
        .find(|e| e.draft.kind == catalogue::Kind::Printing)
        .unwrap();
    assert_eq!(printing.id, "pokemon-laboratory-en-US-001");
    assert_eq!(printing.draft.collector_number.as_deref(), Some("001"));
    assert_eq!(
        printing.draft.release_date.unwrap().to_string(),
        "2026-01-01"
    );
    let wire = serde_json::to_value(&bundle).unwrap();
    assert!(wire.get("sources").is_none());
    assert!(bundle.records.iter().all(|r| r.draft.source.is_none()));
    assert!(
        wire["records"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["metadata"].get("source").is_none())
    );
    let roundtrip: Bundle = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(roundtrip.records).unwrap(),
        serde_json::to_value(&bundle.records).unwrap()
    );
    let mut bad = wire.clone();
    bad["records"][0]["source"] = serde_json::json!("missing");
    assert!(serde_json::from_value::<Bundle>(bad).is_err());
    let mut bad = wire;
    bad["records"][0]["metadata"]["source"] = serde_json::json!({});
    assert!(serde_json::from_value::<Bundle>(bad).is_err());
}

#[test]
fn source_tree_rejects_mistakes_with_file_diagnostics() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    let card = "pokemon/cards/laboratory/en/US/001.yaml";
    for text in [
        "name: One\nname: Two\n",
        "name: Prism Seed\nrairty: Common\n",
        "name: Prism Seed\nsource: missing\n",
        "name: !custom Prism Seed\n",
        "!custom name: Prism Seed\n",
        "name: &card Prism Seed\n",
        "name: Prism Seed\n---\nname: Different\n",
        "name: Prism Seed\npokemon:\n  hp: 5000\n",
        "name: Prism Seed\nattributes:\n  identityScope: manual\n",
        "name: Prism Seed\n<<: {rarity: Common}\n",
    ] {
        write(source.path(), card, text);
        let error = ingestion::source_tree::load(source.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(card), "{error}");
    }
    write(source.path(), card, "name: Prism Seed\n");
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/002.yaml",
        "name: Another\ncollectorNumber: '001'\n",
    );
    let mut duplicate = Bundle {
        records: ingestion::source_tree::load(source.path()).unwrap(),
        ..source_fixture(tempfile::tempdir().unwrap().path())
    };
    assert!(duplicate.validate().is_err());
    duplicate
        .records
        .retain(|e| e.draft.kind != catalogue::Kind::Game);
    assert!(duplicate.validate().is_err());
}

#[test]
fn source_tree_supports_reviewed_reprints() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/identities/prism.yaml",
        "localisations:\n  en:\n    name: Prism Seed\n    pokemon:\n      category: pokemon\n      hp: 70\n      types: [Lightning]\n",
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        "name: Prism Seed\nidentity: pokemon-card-prism\n",
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/002.yaml",
        "name: Prism Seed\nidentity: pokemon-card-prism\n",
    );
    let bundle = Bundle {
        schema_version: 1,
        source_commit: "a".repeat(40),
        records: ingestion::source_tree::load(source.path()).unwrap(),
    };
    bundle.validate().unwrap();
    assert_eq!(
        bundle
            .records
            .iter()
            .filter(|e| e.draft.kind == catalogue::Kind::Card)
            .count(),
        1
    );
    assert_eq!(
        bundle
            .records
            .iter()
            .filter(|e| e.draft.kind == catalogue::Kind::Printing)
            .count(),
        2
    );
}

#[test]
fn source_tree_rejects_stray_files_and_symlinks() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(source.path(), "ignored.json", "{}");
    assert!(ingestion::source_tree::load(source.path()).is_err());
    std::fs::remove_file(source.path().join("ignored.json")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("game.yaml", source.path().join("pokemon/linked.yaml")).unwrap();
        assert!(
            ingestion::source_tree::load(source.path())
                .unwrap_err()
                .to_string()
                .contains("Symlinks")
        );
    }
}

#[test]
fn source_tree_inherits_release_overrides_and_rejects_duplicate_patterns() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/release.yaml",
        "releaseDate: '2026-02-01'\nexternalIds:\n  - namespace: example\n    prefix: 'test:'\n    suffix: ':en'\n",
    );
    let records = ingestion::source_tree::load(source.path()).unwrap();
    let printing = records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Printing)
        .unwrap();
    assert_eq!(printing.external_ids[0].value, "test:001:en");
    assert_eq!(
        printing.draft.release_date.unwrap().to_string(),
        "2026-02-01"
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/release.yaml",
        "externalIds:\n  - namespace: example\n    prefix: 'test:'\n    suffix: ''\n  - namespace: example\n    prefix: 'other:'\n    suffix: ''\n",
    );
    assert!(
        ingestion::source_tree::load(source.path())
            .unwrap_err()
            .to_string()
            .contains("Duplicate external identifier")
    );
}

#[test]
fn source_tree_preserves_gameplay_and_resolves_printing_defaults() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/sets/laboratory.yaml",
        r#"name: Laboratory
stamps: [laboratory-anniversary]
regulationMark: J
printedDenominator: 128
finishes: [holo]
"#,
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        r#"name: Prism Seed
stamps: []
finishes: [non-holo, reverse-holo]
pokemon:
  category: pokemon
  hp: 60
  types: [Grass]
  stage: Basic
  attacks:
    - name: Two sparks
      cost: [Lightning, Lightning, Colorless]
      damage: "20×"
    - name: Rest
      cost: []
      text: Heal 10 damage from this Pokémon.
  abilities: []
  retreatCost: 0
  weaknesses: [{type: Fire, value: "×2"}]
  resistances: []
"#,
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/002.yaml",
        "name: Prism Leaf\n",
    );
    let records = ingestion::source_tree::load(source.path()).unwrap();
    let card = &records
        .iter()
        .find(|r| r.id == "card-pokemon-laboratory-en-US-001")
        .unwrap()
        .draft;
    assert_eq!(
        card.attributes["attacks"][0]["cost"],
        serde_json::json!(["Lightning", "Lightning", "Colorless"])
    );
    assert_eq!(card.attributes["attacks"][0]["damage"], "20×");
    assert!(card.attributes["attacks"][1].get("damage").is_none());
    assert_eq!(card.attributes["attacks"][1]["cost"], serde_json::json!([]));
    assert_eq!(card.attributes["retreatCost"], 0);
    let printing = &records
        .iter()
        .find(|r| r.id == "pokemon-laboratory-en-US-001")
        .unwrap()
        .draft;
    assert_eq!(printing.attributes["stamps"], serde_json::json!([]));
    assert_eq!(
        printing.attributes["finishes"],
        serde_json::json!(["non-holo", "reverse-holo"])
    );
    assert_eq!(printing.attributes["regulationMark"], "J");
    assert_eq!(printing.attributes["printedDenominator"], 128);
    let inherited = &records
        .iter()
        .find(|r| r.id == "pokemon-laboratory-en-US-002")
        .unwrap()
        .draft;
    assert_eq!(
        inherited.attributes["stamps"],
        serde_json::json!(["laboratory-anniversary"])
    );
    assert_eq!(
        inherited.attributes["finishes"],
        serde_json::json!(["holo"])
    );
    let unknown = &records
        .iter()
        .find(|r| r.id == "card-pokemon-laboratory-en-US-002")
        .unwrap()
        .draft;
    assert!(unknown.attributes.get("attacks").is_none());
}

#[test]
fn source_tree_rejects_invalid_rules_and_printing_metadata() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    let path = "pokemon/cards/laboratory/en/US/001.yaml";
    for fields in [
        "finishes: [shiny]",
        "finishes: [holo, holo]",
        "regulationMark: JJ",
        "printedDenominator: 0",
        "pokemon: {hp: 0}",
        "pokemon: {category: trainer, attacks: []}",
        "pokemon: {attacks: [{name: Hit, cost: [Electric]}]}",
        "pokemon: {attacks: [{name: Hit, cost: [], damage: '20++'}]}",
        "pokemon: {attacks: [{name: Hit, cost: [], damage: 20}]}",
        "pokemon: {weaknesses: [{type: Fire, value: '2'}]}",
        "pokemon: {trainers: [hop, hop]}",
        "finishes: [holo]\nprintingAttributes: {finishes: [non-holo]}",
    ] {
        write(
            source.path(),
            path,
            &format!("name: Prism Seed\n{fields}\n"),
        );
        let error = ingestion::source_tree::load(source.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(path), "{fields}: {error}");
    }
    write(source.path(), path, "name: Prism Seed\n");
    write(
        source.path(),
        "pokemon/sets/laboratory.yaml",
        "name: Laboratory\ndefaults: {finishes: [shiny]}\n",
    );
    assert!(ingestion::source_tree::load(source.path()).is_err());
}

#[test]
fn openapi_exposes_the_same_enum_values_as_the_compiler() {
    let spec = cardlattice_api::openapi::document();
    let vocabulary = catalogue::pokemon::vocabulary::document();
    for (definition, schema) in [
        ("category", "Category"),
        ("energyType", "EnergyType"),
        ("stage", "Stage"),
        ("subtype", "Subtype"),
        ("rarity", "Rarity"),
        ("finish", "Finish"),
        ("abilityKind", "AbilityKind"),
        ("regulationMark", "RegulationMark"),
    ] {
        assert_eq!(
            spec["components"]["schemas"][schema]["enum"], vocabulary[definition],
            "{schema}"
        );
    }
}

#[test]
fn source_tree_enum_validation_cannot_be_bypassed_by_attribute_locations() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    let path = "pokemon/cards/laboratory/en/US/001.yaml";
    for fields in [
        "rarity: Comon",
        "rarity: common",
        "rarity: SIR",
        "pokemon: {stage: Stage1}",
        "attributes: {stage: basic}",
        "pokemon: {subtypes: [Suporter]}",
        "attributes: {subtypes: [item]}",
        "printingAttributes: {finishes: [normal]}",
        "printingAttributes: {regulationMark: Z}",
    ] {
        write(
            source.path(),
            path,
            &format!("name: Prism Seed\n{fields}\n"),
        );
        let error = ingestion::source_tree::load(source.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(path), "{fields}: {error}");
    }
}

#[test]
fn source_tree_inheritance_distinguishes_missing_cleared_and_empty() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/sets/laboratory.yaml",
        "name: Laboratory\nprintedDenominator: 100\nregulationMark: J\nfinishes: [holo]\nstamps: [anniversary]\nreleaseDate: 2026-01-01\n",
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/release.yaml",
        "printedDenominator: 101\nfinishes: [non-holo]\nreleaseDate: 2026-02-01\n",
    );
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        "name: Prism Seed\nprintedDenominator: null\nregulationMark: null\nstamps: []\nreleaseDate: null\n",
    );
    let bundle = Bundle {
        records: ingestion::source_tree::load(source.path()).unwrap(),
        ..input()
    };
    bundle.validate().unwrap();
    let printing = &bundle
        .records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Printing)
        .unwrap()
        .draft;
    assert!(printing.release_date.is_none());
    assert_eq!(
        printing.attributes["finishes"],
        serde_json::json!(["non-holo"])
    );
    assert_eq!(printing.attributes["stamps"], serde_json::json!([]));
    assert_eq!(
        printing.attributes.get("printedDenominator"),
        Some(&serde_json::Value::Null)
    );
    assert_eq!(
        printing.attributes.get("regulationMark"),
        Some(&serde_json::Value::Null)
    );
    assert!(
        !printing
            .attributes
            .as_object()
            .unwrap()
            .contains_key("illustrators")
    );
    let card = &bundle
        .records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap()
        .draft;
    assert!(card.attributes.get("types").is_none());
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        "name: Prism Seed\npokemon: {types: []}\n",
    );
    let records = ingestion::source_tree::load(source.path()).unwrap();
    let printing = &records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Printing)
        .unwrap()
        .draft;
    assert_eq!(printing.attributes["printedDenominator"], 101);
    assert_eq!(printing.release_date.unwrap().to_string(), "2026-02-01");
    let card = &records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap()
        .draft;
    assert_eq!(card.attributes["types"], serde_json::json!([]));
}

#[test]
fn source_tree_shared_identity_requires_each_printing_language() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/identities/prism.yaml",
        "localisations:\n  en:\n    name: Prism\n    pokemon: {category: pokemon, attacks: [{name: Rest, cost: []}]}\n  nl:\n    name: Prism\n    pokemon: {category: pokemon, attacks: [{name: Rust, cost: []}]}\n",
    );
    for language in ["en", "nl"] {
        write(
            source.path(),
            &format!("pokemon/cards/laboratory/{language}/US/001.yaml"),
            "name: Prism\nidentity: pokemon-card-prism\n",
        );
    }
    let bundle = Bundle {
        records: ingestion::source_tree::load(source.path()).unwrap(),
        ..input()
    };
    bundle.validate().unwrap();
    for (language, attack) in [("en", "Rest"), ("nl", "Rust")] {
        let card = bundle
            .records
            .iter()
            .find(|r| r.id == format!("pokemon-card-prism-{language}"))
            .unwrap();
        assert_eq!(card.draft.attributes["identityGroup"], "pokemon-card-prism");
        assert_eq!(card.draft.attributes["attacks"][0]["name"], attack);
        let printing = bundle
            .records
            .iter()
            .find(|r| r.id == format!("pokemon-laboratory-{language}-US-001"))
            .unwrap();
        assert_eq!(printing.draft.card_id.as_deref(), Some(card.id.as_str()));
    }
    write(
        source.path(),
        "pokemon/cards/laboratory/fr/US/001.yaml",
        "name: Prism\nidentity: pokemon-card-prism\n",
    );
    assert!(
        ingestion::source_tree::load(source.path())
            .unwrap_err()
            .to_string()
            .contains("Missing identity localisation")
    );
}

#[test]
fn source_tree_variants_validate_combinations_and_references() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    let card = "pokemon/cards/laboratory/en/US/001.yaml";
    write(
        source.path(),
        card,
        "name: Prism Seed\nfinishes: [holo]\nvariants:\n  first-edition:\n    finish: holo\n    edition: first-edition\n    products: [pokemon-product-box]\n",
    );
    let mut bundle = Bundle {
        records: ingestion::source_tree::load(source.path()).unwrap(),
        ..input()
    };
    assert!(bundle.validate().is_err());
    write(source.path(), "pokemon/products/box.yaml", "name: Box\n");
    bundle.records = ingestion::source_tree::load(source.path()).unwrap();
    bundle.validate().unwrap();
    for fields in [
        "finishes: [non-holo]\nvariants: {first: {finish: holo}}",
        "variants: {first: {finish: holo}, duplicate: {finish: holo}}",
        "variants: {first: {finish: holo, edition: First}}",
        "variants: {first: {finish: holo, id: first}}",
        "finishes: null",
    ] {
        write(
            source.path(),
            card,
            &format!("name: Prism Seed\n{fields}\n"),
        );
        assert!(
            ingestion::source_tree::load(source.path()).is_err(),
            "{fields}"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexing_preserves_clearing_and_indexes_variant_products(pool: PgPool) {
    let source = tempfile::tempdir().unwrap();
    let mut bundle = source_fixture(source.path());
    write(
        source.path(),
        "pokemon/sets/laboratory.yaml",
        "name: Laboratory\nproducts: [pokemon-product-box]\nartwork: null\n",
    );
    write(source.path(), "pokemon/products/box.yaml", "name: Box\n");
    write(
        source.path(),
        "pokemon/cards/laboratory/en/US/001.yaml",
        "name: Prism Seed\nartwork: null\nproducts: []\nvariants: {first: {finish: holo, edition: first-edition, products: [pokemon-product-box]}}\n",
    );
    bundle.records = ingestion::source_tree::load(source.path()).unwrap();
    bundle::index(&pool, &bundle, true).await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM catalogue_links WHERE target_id='pokemon-product-box'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2); // Set plus the printing's explicit variant.
    let attrs: serde_json::Value =
        sqlx::query_scalar("SELECT attributes FROM catalogue_records WHERE kind='printing'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attrs.get("artwork"), Some(&serde_json::Value::Null));
    assert_eq!(attrs["products"], serde_json::json!([]));
}

#[sqlx::test(migrations = false)]
async fn migration_preserves_existing_catalogue_rows(pool: PgPool) {
    // Exercise an upgrade with real rows, not only empty-database migrations.
    for migration in sqlx::migrate!("../../migrations")
        .iter()
        .filter(|m| m.version < 8)
    {
        sqlx::raw_sql(migration.sql.clone())
            .execute(&pool)
            .await
            .unwrap();
    }
    let mut bundle = input();
    bundle::index(&pool, &bundle, true).await.unwrap();
    let card = bundle
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap();
    let new_key = card.draft.duplicate_key();
    sqlx::raw_sql(include_str!(
        "../../../migrations/0008_printing_metadata.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let actual: String =
        sqlx::query_scalar("SELECT duplicate_key FROM catalogue_records WHERE id=$1")
            .bind(&card.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(actual, new_key);
}

#[test]
fn shared_identity_localisations_with_identical_text_are_distinct() {
    let source = tempfile::tempdir().unwrap();
    source_fixture(source.path());
    write(
        source.path(),
        "pokemon/identities/prism.yaml",
        "localisations:\n  en:\n    name: Prism\n    pokemon: {category: pokemon, hp: 60}\n  nl:\n    name: Prism\n    pokemon: {category: pokemon, hp: 60}\n",
    );
    let bundle = Bundle {
        records: ingestion::source_tree::load(source.path()).unwrap(),
        ..input()
    };
    bundle.validate().unwrap();
    let keys: std::collections::HashSet<_> = bundle
        .records
        .iter()
        .filter(|r| r.draft.attributes.get("identityGroup").is_some())
        .map(|r| r.draft.duplicate_key())
        .collect();
    assert_eq!(keys.len(), 2);
}
