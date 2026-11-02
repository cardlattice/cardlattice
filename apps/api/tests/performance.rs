//! Opt-in measurements against a disposable PostgreSQL database; never a latency claim.
use axum::http::HeaderMap;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::time::Instant;

fn report(name: &str, mut values: Vec<f64>) {
    values.sort_by(f64::total_cmp);
    let at = |percent: f64| values[((values.len() - 1) as f64 * percent).round() as usize];
    println!(
        "{}",
        json!({"benchmark":name,"samples":values.len(),"unit":"milliseconds","p50":at(0.50),"p95":at(0.95),"p99":at(0.99)})
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "Opt-in: generates 20,000 original synthetic records in a disposable database"]
async fn measured_catalogue_workloads(pool: PgPool) {
    ingestion::seed(&pool, include_str!("../../../data/development.json"))
        .await
        .unwrap();
    sqlx::query("INSERT INTO catalogue_records(id,kind,game_id,name,normalized_name,synthetic,duplicate_key) SELECT ('01910000-0000-7000-8000-'||lpad(n::text,12,'0'))::uuid,'card','01900000-0000-7000-8000-000000000001','Original benchmark prism '||n,'original benchmark prism '||n,true,'original-benchmark-'||n FROM generate_series(1,20000) n").execute(&pool).await.unwrap();
    sqlx::query("VACUUM (ANALYZE) catalogue_records")
        .execute(&pool)
        .await
        .unwrap();
    let mut lookups = Vec::new();
    let mut searches = Vec::new();
    let mut validators = Vec::new();
    let payload = json!({"data":{"name":"Original benchmark record","synthetic":true,"attributes":{"category":"pokemon","hp":40,"types":["grass"]}}});
    let modified = Utc::now();
    for sample in 0..550 {
        let started = Instant::now();
        let _:catalogue::Record=sqlx::query_as("SELECT * FROM catalogue_records WHERE id='01910000-0000-7000-8000-000000010000' AND deleted_at IS NULL").fetch_one(&pool).await.unwrap();
        if sample >= 50 {
            lookups.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        let started = Instant::now();
        let _:Vec<catalogue::Record>=sqlx::query_as("SELECT * FROM catalogue_records WHERE deleted_at IS NULL AND search_document @@ plainto_tsquery('simple','prism 10000') ORDER BY normalized_name COLLATE \"C\",id LIMIT 20").fetch_all(&pool).await.unwrap();
        if sample >= 50 {
            searches.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        let started = Instant::now();
        std::hint::black_box(cardlattice_api::cache::cached_json(
            &payload,
            &HeaderMap::new(),
            modified,
            86400,
        ));
        if sample >= 50 {
            validators.push(started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    println!(
        "{}",
        json!({"environment":"local process plus PostgreSQL","records":20007,"warmupSamples":50,"concurrency":1,"cdnMeasured":false,"timestamp":Utc::now()})
    );
    report("primary_key_card_lookup", lookups);
    report("indexed_full_text_search", searches);
    report("json_etag_response_generation", validators);
    let plan:Vec<(String,)>=sqlx::query_as("EXPLAIN SELECT * FROM catalogue_records WHERE deleted_at IS NULL AND search_document @@ plainto_tsquery('simple','prism 10000') LIMIT 20").fetch_all(&pool).await.unwrap();
    assert!(
        plan.iter().any(|(line,)| line.contains("Index")),
        "Indexed benchmark must actually use an index: {plan:?}"
    );
    for (line,) in plan {
        println!("{line}");
    }
}
