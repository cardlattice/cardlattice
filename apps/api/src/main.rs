use cardlattice_api::{AppState, config::Config};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = arguments.first().map(String::as_str).unwrap_or("serve");
    if command == "build-catalogue" {
        anyhow::ensure!(
            arguments.len() == 4,
            "Usage: build-catalogue <source-directory> <output-directory> <git-commit>"
        );
        let manifest = ingestion::bundle::build(
            std::path::Path::new(&arguments[1]),
            std::path::Path::new(&arguments[2]),
            &arguments[3],
        )?;
        println!("{}", serde_json::to_string_pretty(&manifest)?);
        return Ok(());
    }
    if command == "openapi" {
        println!(
            "{}",
            serde_json::to_string_pretty(&cardlattice_api::openapi::document())?
        );
        return Ok(());
    }
    if command == "scan-pokemon-sets" {
        anyhow::ensure!(
            arguments.len() == 3,
            "Usage: scan-pokemon-sets <plan.json> <candidates.json>"
        );
        let plan: ingestion::official_pokemon::ScanPlan =
            serde_json::from_slice(&tokio::fs::read(&arguments[1]).await?)?;
        let candidates = ingestion::official_pokemon::scan(&plan).await?;
        // Create a new file: never overwrite a previously reviewed batch.
        use tokio::io::AsyncWriteExt;
        let mut output = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&arguments[2])
            .await?;
        output
            .write_all(serde_json::to_string_pretty(&candidates)?.as_bytes())
            .await?;
        tracing::info!(
            count = candidates.sets.len(),
            "official facts collected; review candidates and source terms before import"
        );
        return Ok(());
    }
    let database_url = cardlattice_api::config::database_url()?;
    let pool = cardlattice_api::connect(&database_url).await?;
    match command {
        "index-catalogue" => {
            anyhow::ensure!(
                arguments.len() == 2 || (arguments.len() == 3 && arguments[2] == "--apply"),
                "Usage: index-catalogue <manifest.json> [--apply]"
            );
            let directory =
                std::env::var("DATASET_DIRECTORY").unwrap_or_else(|_| ".local/datasets".into());
            let summary = cardlattice_api::catalogue_index::import(
                &pool,
                std::path::Path::new(&arguments[1]),
                std::path::Path::new(&directory),
                arguments.len() == 3,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            return Ok(());
        }
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
        "import-pokemon-cards" => {
            anyhow::ensure!(
                arguments.len() == 2 || (arguments.len() == 3 && arguments[2] == "--apply"),
                "Usage: import-pokemon-cards <reviewed.json> [--apply]"
            );
            let bytes = tokio::fs::read(&arguments[1]).await?;
            anyhow::ensure!(bytes.len() <= 2 * 1024 * 1024, "Import batch exceeds 2 MiB");
            let batch = serde_json::from_slice(&bytes)?;
            let summary =
                ingestion::pokemon_cards::import(&pool, &batch, arguments.len() == 3).await?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            return Ok(());
        }
        "import-pokemon-sets" => {
            anyhow::ensure!(
                arguments.len() >= 2
                    && arguments[2..]
                        .iter()
                        .all(|argument| argument == "--apply" || argument == "--retire-demo"),
                "Usage: import-pokemon-sets <reviewed.json> [--apply] [--retire-demo]"
            );
            let bytes = tokio::fs::read(&arguments[1]).await?;
            anyhow::ensure!(bytes.len() <= 1024 * 1024, "Import batch exceeds 1 MiB");
            let batch = serde_json::from_slice(&bytes)?;
            let summary = ingestion::official_pokemon::import(
                &pool,
                &batch,
                arguments.iter().any(|argument| argument == "--apply"),
                arguments.iter().any(|argument| argument == "--retire-demo"),
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            return Ok(());
        }
        "serve" => {}
        _ => anyhow::bail!(
            "Commands: serve, migrate, seed, openapi, build-catalogue, index-catalogue, scan-pokemon-sets, import-pokemon-sets, import-pokemon-cards"
        ),
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
