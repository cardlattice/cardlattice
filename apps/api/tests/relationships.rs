use axum::{body::Body, http::Request};
use cardlattice_api::{AppState, config::Config};
use http_body_util::BodyExt;
use ingestion::bundle::{self, Bundle};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::{path::Path, sync::Arc};
use tower::ServiceExt;

fn write(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, content).unwrap();
}
fn fixture(root: &Path) -> Bundle {
    for (path, data) in [
        ("pokemon/game.yaml", "name: Pokémon\n"),
        (
            "pokemon/sets/laboratory.yaml",
            "name: Synthetic Laboratory\nprintedIdentifiers: [{language: en, region: US, code: SYN7}]\n",
        ),
        ("pokemon/sets/promos.yaml", "name: Synthetic Promos\n"),
        (
            "pokemon/products/laboratory.yaml",
            "name: Synthetic laboratory box\nrelatedSets: [pokemon-laboratory]\n",
        ),
        (
            "pokemon/collections/laboratory.yaml",
            "name: Synthetic laboratory and related promos\nrelatedSets: [pokemon-laboratory]\n",
        ),
        (
            "pokemon/artworks/seed.yaml",
            "name: Synthetic seed illustration\nillustrators: [Example artist]\n",
        ),
        (
            "pokemon/cards/laboratory/en/US/001.yaml",
            "name: Synthetic Seed\nproducts: [pokemon-product-laboratory]\ncollections: [pokemon-collection-laboratory]\nartwork: pokemon-artwork-seed\npokemon: {attacks: [{name: Tackle, cost: [Colorless], damage: '10'}]}\n",
        ),
        (
            "pokemon/cards/promos/en/US/001.yaml",
            "name: Synthetic Seed promo\nproducts: [pokemon-product-laboratory]\ncollections: [pokemon-collection-laboratory]\nartwork: pokemon-artwork-seed\npokemon: {attacks: [{name: Tackle, cost: [Psychic, Colorless], damage: '30'}]}\n",
        ),
    ] {
        write(root, path, data);
    }
    load(root)
}
fn load(root: &Path) -> Bundle {
    Bundle {
        schema_version: 1,
        source_commit: "a".repeat(40),
        records: ingestion::source_tree::load(root).unwrap(),
    }
}
#[test]
fn relationships_preserve_sets_rules_and_artwork_independently() {
    let root = tempfile::tempdir().unwrap();
    let bundle = fixture(root.path());
    bundle.validate().unwrap();
    let printings: Vec<_> = bundle
        .records
        .iter()
        .filter(|r| r.draft.kind == catalogue::Kind::Printing)
        .collect();
    assert_ne!(printings[0].draft.set_id, printings[1].draft.set_id);
    assert_ne!(printings[0].draft.card_id, printings[1].draft.card_id);
    assert_eq!(
        printings[0].draft.attributes["artwork"],
        printings[1].draft.attributes["artwork"]
    );
    let cards: Vec<_> = bundle
        .records
        .iter()
        .filter(|r| r.draft.kind == catalogue::Kind::Card)
        .collect();
    assert_ne!(
        cards[0].draft.attributes["attacks"],
        cards[1].draft.attributes["attacks"]
    );
    for (field, value) in [
        ("products", json!(["pokemon-product-typo"])),
        ("products", json!(["pokemon-artwork-seed"])),
        (
            "collections",
            json!([
                "pokemon-collection-laboratory",
                "pokemon-collection-laboratory"
            ]),
        ),
        ("artwork", json!("pokemon-product-laboratory")),
    ] {
        let mut invalid = bundle.clone();
        invalid
            .records
            .iter_mut()
            .find(|r| r.draft.kind == catalogue::Kind::Printing)
            .unwrap()
            .draft
            .attributes[field] = value;
        assert!(invalid.validate().is_err(), "{field}");
    }
    let mut invalid = bundle.clone();
    let game = invalid
        .records
        .iter()
        .find(|r| r.draft.kind == catalogue::Kind::Game)
        .unwrap()
        .clone();
    let mut other = game;
    other.id = "another".into();
    other.draft.slug = Some("another".into());
    other.draft.name = "Another".into();
    invalid.records.push(other);
    invalid
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Artwork)
        .unwrap()
        .draft
        .game_id = Some("another".into());
    assert!(invalid.validate().is_err());
}
#[test]
fn malformed_scoped_set_identifiers_and_misplaced_fields_fail() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    for marks in [
        "[{language: en}]",
        "[{language: en, code: LAB}, {language: en, code: OTHER}]",
        "[{language: en, region: usa, code: LAB}]",
    ] {
        write(
            root.path(),
            "pokemon/sets/laboratory.yaml",
            &format!("name: Synthetic Laboratory\nprintedIdentifiers: {marks}\n"),
        );
        assert!(ingestion::source_tree::load(root.path()).is_err());
    }
    fixture(root.path());
    write(
        root.path(),
        "pokemon/artworks/seed.yaml",
        "name: Synthetic art\nrelatedSets: [pokemon-laboratory]\n",
    );
    assert!(ingestion::source_tree::load(root.path()).is_err());
}
async fn get(state: &AppState, path: &str) -> Value {
    let path = if let Some((base, query)) = path.split_once('?') {
        let mut pairs: Vec<_> = query.split('&').collect();
        pairs.sort();
        format!("{base}?{}", pairs.join("&"))
    } else {
        path.into()
    };
    let response = cardlattice_api::router(state.clone())
        .oneshot(Request::builder().uri(&path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{path}");
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
#[sqlx::test(migrations = "../../migrations")]
async fn indexing_browsing_rewiring_and_retirement_are_consistent(pool: PgPool) {
    let root = tempfile::tempdir().unwrap();
    let mut input = fixture(root.path());
    assert_eq!(
        bundle::index(&pool, &input, true).await.unwrap().inserted,
        10
    );
    assert_eq!(
        bundle::index(&pool, &input, true).await.unwrap().unchanged,
        10
    );
    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(Config {
            database_url: String::new(),
            bind: "127.0.0.1:0".into(),
            contributor_token: "test-contributor-capability-not-real".into(),
            moderator_token: "test-moderator-capability-not-real".into(),
            metrics_token: "test-metrics-capability-not-real".into(),
            publishing: publishing::PublishingConfig {
                directory: root.path().into(),
                public_origin: "http://localhost".into(),
                cloudflare: None,
            },
        }),
        metrics: observability::Metrics::default(),
    };
    for (collection, id, filter) in [
        ("products", "pokemon-product-laboratory", "productId"),
        (
            "collections",
            "pokemon-collection-laboratory",
            "collectionId",
        ),
        ("artworks", "pokemon-artwork-seed", "artworkId"),
    ] {
        let detail = get(&state, &format!("/v1/games/pokemon/{collection}/{id}")).await;
        assert_eq!(detail["printings"].as_array().unwrap().len(), 2);
        let url = format!("/v1/games/pokemon/printings?{filter}={id}");
        assert_eq!(detail["printingsUrl"], url);
        assert_eq!(get(&state, &url).await["data"].as_array().unwrap().len(), 2);
        let page = get(
            &state,
            &format!("/v1/games/pokemon/printings?{filter}={id}&limit=1"),
        )
        .await;
        assert!(page["nextCursor"].is_string());
    }
    assert_eq!(
        get(&state, "/v1/games/pokemon/sets?q=syn7").await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE catalogue_records SET deleted_at=now() WHERE id='pokemon-artwork-seed'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(
        tx.commit().await.is_err(),
        "linked artwork cannot be retired"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE catalogue_records SET attributes=jsonb_set(attributes,'{artwork}','\"pokemon-product-laboratory\"') WHERE id='pokemon-promos-en-US-001'").execute(&mut *tx).await.unwrap();
    assert!(
        tx.commit().await.is_err(),
        "wrong-kind links must fail in the database too"
    );
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE catalogue_records SET attributes=jsonb_set(attributes,'{artwork}','\"missing-artwork\"') WHERE id='pokemon-promos-en-US-001'").execute(&mut *tx).await.unwrap();
    assert!(tx.commit().await.is_err(), "dangling links must fail");
    // Removing all references and the target is allowed in one authoritative import.
    for entry in &mut input.records {
        if entry.draft.kind == catalogue::Kind::Printing {
            entry
                .draft
                .attributes
                .as_object_mut()
                .unwrap()
                .remove("artwork");
        }
    }
    input
        .records
        .retain(|e| e.draft.kind != catalogue::Kind::Artwork);
    assert_eq!(bundle::index(&pool, &input, true).await.unwrap().retired, 1);
    let paths: Vec<Value> = sqlx::query_scalar("SELECT paths FROM cache_invalidation_outbox")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(paths.iter().any(|paths| {
        paths
            .as_array()
            .unwrap()
            .contains(&json!("/v1/games/pokemon/artworks/pokemon-artwork-seed"))
    }));
    assert_eq!(
        get(
            &state,
            "/v1/games/pokemon/printings?artworkId=pokemon-artwork-seed"
        )
        .await["data"],
        json!([])
    );
}
