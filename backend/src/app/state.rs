use anyhow::Result;
use opensearch::OpenSearch;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use super::Config;
use crate::infra;

/// Shared, reusable clients injected into feature handlers through Axum state.
#[derive(Clone)]
pub struct AppState {
    pub postgres: PgPool,
    pub redis: ConnectionManager,
    pub opensearch: OpenSearch,
    pub dependency_timeout: std::time::Duration,
}

impl AppState {
    pub async fn connect(config: &Config) -> Result<Self> {
        Ok(Self {
            postgres: infra::postgres::connect(
                &config.database_url,
                config.database_max_connections,
                config.dependency_timeout,
            )
            .await?,
            redis: infra::redis::connect(&config.redis_url, config.dependency_timeout).await?,
            opensearch: infra::opensearch::connect(
                &config.opensearch_url,
                config.opensearch_credentials.as_ref(),
                config.dependency_timeout,
            )
            .await?,
            dependency_timeout: config.dependency_timeout,
        })
    }
}
