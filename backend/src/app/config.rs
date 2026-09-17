use std::{env, fs, net::SocketAddr, path::Path, time::Duration};

use anyhow::{Context, Result, ensure};
use axum::http::HeaderValue;
use opensearch::http::Url;
use serde::Deserialize;

pub struct Config {
    pub address: SocketAddr,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub opensearch_url: Url,
    pub opensearch_credentials: Option<(String, String)>,
    pub cors_origin: HeaderValue,
    pub dependency_timeout: Duration,
    pub log_filter: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// Select YAML defaults and apply explicit overrides without changing process environment.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let profile = get("APP_ENV").unwrap_or_else(|| "dev".into());
        let defaults = match profile.as_str() {
            "dev" => include_str!("../../config/dev.yml"),
            "test" => include_str!("../../config/test.yml"),
            "prod" => include_str!("../../config/prod.yml"),
            _ => anyhow::bail!("APP_ENV must be dev, test, or prod"),
        };
        match get("CONFIG_DIR") {
            Some(directory) => {
                let path = Path::new(&directory).join(format!("{profile}.yml"));
                let yaml = fs::read_to_string(&path)
                    .with_context(|| format!("Unable to read configuration {}", path.display()))?;
                Self::from_yaml(&yaml, get)
            }
            None => Self::from_yaml(defaults, get),
        }
    }

    /// Parse a complete YAML configuration and apply environment-style overrides.
    pub fn from_yaml(yaml: &str, get: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let settings: Settings =
            serde_yaml_ng::from_str(yaml).context("Invalid YAML configuration")?;
        let value = |name, default: String| get(name).unwrap_or(default);
        let required = |name, default: Option<String>| {
            get(name)
                .or(default)
                .filter(|value| !value.trim().is_empty())
                .with_context(|| format!("{name} must be set in YAML or the environment"))
        };
        let host = value("HOST", settings.server.host);
        let port: u16 = value("PORT", settings.server.port.to_string())
            .parse()
            .context("PORT must be a valid port number")?;
        ensure!(port > 0, "PORT must be greater than zero");
        let address = SocketAddr::new(host.parse().context("HOST must be an IP address")?, port);
        let database_url = required("DATABASE_URL", settings.database.url)?;
        database_url
            .parse::<sqlx::postgres::PgConnectOptions>()
            .context("Invalid DATABASE_URL")?;
        let database_max_connections: u32 = value(
            "DATABASE_MAX_CONNECTIONS",
            settings.database.max_connections.to_string(),
        )
        .parse()
        .context("DATABASE_MAX_CONNECTIONS must be a positive integer")?;
        ensure!(
            database_max_connections > 0,
            "DATABASE_MAX_CONNECTIONS must be greater than zero"
        );
        let redis_url = required("REDIS_URL", Some(settings.redis.url))?;
        redis::Client::open(redis_url.as_str()).context("Invalid REDIS_URL")?;
        let opensearch_url = Url::parse(&required(
            "OPENSEARCH_NODE",
            Some(settings.opensearch.node),
        )?)
        .context("OPENSEARCH_NODE must be a valid URL")?;
        ensure!(
            matches!(opensearch_url.scheme(), "http" | "https"),
            "OPENSEARCH_NODE must use HTTP or HTTPS"
        );
        let username = get("OPENSEARCH_USERNAME")
            .or(settings.opensearch.username)
            .filter(|value| !value.is_empty());
        let password = get("OPENSEARCH_PASSWORD")
            .or(settings.opensearch.password)
            .filter(|value| !value.is_empty());
        let opensearch_credentials = match (username, password) {
            (Some(username), Some(password)) => Some((username, password)),
            (None, None) => None,
            _ => anyhow::bail!("Set both OPENSEARCH_USERNAME and OPENSEARCH_PASSWORD, or neither"),
        };
        let origin = value("CORS_ORIGIN", settings.server.cors_origin);
        let origin_url = Url::parse(&origin).context("CORS_ORIGIN must be a valid URL")?;
        ensure!(
            matches!(origin_url.scheme(), "http" | "https")
                && origin_url.origin().ascii_serialization() == origin,
            "CORS_ORIGIN must be an HTTP(S) origin without a path or trailing slash"
        );
        let timeout_ms: u64 = value(
            "DEPENDENCY_TIMEOUT_MS",
            settings.health.timeout_ms.to_string(),
        )
        .parse()
        .context("DEPENDENCY_TIMEOUT_MS must be a positive integer")?;
        ensure!(
            timeout_ms > 0,
            "DEPENDENCY_TIMEOUT_MS must be greater than zero"
        );
        let log_filter = value("RUST_LOG", settings.logging.filter);
        tracing_subscriber::EnvFilter::try_new(&log_filter).context("Invalid logging filter")?;
        Ok(Self {
            address,
            database_url,
            database_max_connections,
            redis_url,
            opensearch_url,
            opensearch_credentials,
            cors_origin: origin.parse().context("Invalid CORS_ORIGIN header")?,
            dependency_timeout: Duration::from_millis(timeout_ms),
            log_filter,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    server: Server,
    database: Database,
    redis: Redis,
    opensearch: OpenSearch,
    health: Health,
    logging: Logging,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Server {
    host: String,
    port: u16,
    cors_origin: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Database {
    url: Option<String>,
    max_connections: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Redis {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenSearch {
    node: String,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Health {
    timeout_ms: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Logging {
    filter: String,
}
