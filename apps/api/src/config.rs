use publishing::{CloudflareConfig, PublishingConfig};
use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub bind: String,
    pub contributor_token: String,
    pub moderator_token: String,
    pub metrics_token: String,
    pub publishing: PublishingConfig,
}
fn secret(name: &str) -> anyhow::Result<String> {
    let value = if let Ok(file) = env::var(format!("{name}_FILE")) {
        std::fs::read_to_string(file)?.trim().to_owned()
    } else {
        env::var(name)?
    };
    anyhow::ensure!(value.len() >= 32, "{name} must be at least 32 characters");
    Ok(value)
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let origin =
            env::var("PUBLIC_API_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".into());
        let parsed = url::Url::parse(&origin)?;
        anyhow::ensure!(
            ["http", "https"].contains(&parsed.scheme())
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && parsed.query().is_none()
                && parsed.path() == "/",
            "PUBLIC_API_ORIGIN must be an HTTP(S) origin"
        );
        let cloudflare = match (
            env::var("CLOUDFLARE_ZONE_ID"),
            env::var("CLOUDFLARE_API_TOKEN"),
        ) {
            (Ok(zone_id), Ok(api_token)) if !zone_id.is_empty() && !api_token.is_empty() => {
                anyhow::ensure!(
                    zone_id.len() == 32 && zone_id.bytes().all(|b| b.is_ascii_hexdigit()),
                    "Invalid Cloudflare zone ID"
                );
                anyhow::ensure!(
                    parsed.scheme() == "https",
                    "Cloudflare public origin must use HTTPS"
                );
                Some(CloudflareConfig { zone_id, api_token })
            }
            (Err(_), Err(_)) => None,
            _ => anyhow::bail!("Set both Cloudflare zone ID and token, or neither"),
        };
        let contributor_token = secret("CONTRIBUTOR_TOKEN")?;
        let moderator_token = secret("MODERATOR_TOKEN")?;
        let metrics_token = secret("METRICS_TOKEN")?;
        anyhow::ensure!(
            contributor_token != moderator_token
                && metrics_token != moderator_token
                && metrics_token != contributor_token,
            "Access roles must use distinct tokens"
        );
        Ok(Self {
            database_url: database_url()?,
            bind: env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            contributor_token,
            moderator_token,
            metrics_token,
            publishing: PublishingConfig {
                directory: PathBuf::from(
                    env::var("DATASET_DIRECTORY").unwrap_or_else(|_| ".local/datasets".into()),
                ),
                public_origin: origin,
                cloudflare,
            },
        })
    }
}

pub fn database_url() -> anyhow::Result<String> {
    if let Ok(file) = env::var("DATABASE_URL_FILE") {
        Ok(std::fs::read_to_string(file)?.trim().to_owned())
    } else {
        Ok(env::var("DATABASE_URL")?)
    }
}
