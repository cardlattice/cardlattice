use cardlattice_api::{AppState, config::Config};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init();
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    if command == "openapi" {
        println!(
            "{}",
            serde_json::to_string_pretty(&cardlattice_api::openapi::document())?
        );
        return Ok(());
    }
    let database_url = cardlattice_api::config::database_url()?;
    let pool = cardlattice_api::connect(&database_url).await?;
    match command.as_str() {
        "migrate" => {
            sqlx::migrate!("../../migrations").run(&pool).await?;
            return Ok(());
        }
        "seed" => {
            let count =
                ingestion::seed(&pool, include_str!("../../../data/development.json")).await?;
            tracing::info!(count, "original synthetic fixtures inserted");
            return Ok(());
        }
        "serve" => {}
        _ => anyhow::bail!("Commands: serve, migrate, seed, openapi"),
    }
    let config = Arc::new(Config::from_env()?);
    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    let state = AppState {
        pool,
        config,
        metrics: observability::Metrics::default(),
    };
    let worker = tokio::spawn(cardlattice_api::background(state.clone()));
    tracing::info!(address=%listener.local_addr()?,"Cardlattice API listening");
    axum::serve(
        listener,
        cardlattice_api::router(state)
            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    worker.abort();
    Ok(())
}
