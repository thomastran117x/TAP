use anyhow::{Context, Result, ensure};
use redis::aio::{ConnectionManager, ConnectionManagerConfig};

use std::time::Duration;

pub async fn connect(redis_url: &str, timeout: Duration) -> Result<ConnectionManager> {
    let config = ConnectionManagerConfig::new()
        .set_connection_timeout(Some(timeout))
        .set_response_timeout(Some(timeout));
    tokio::time::timeout(
        timeout,
        ConnectionManager::new_with_config(
            redis::Client::open(redis_url).context("Invalid REDIS_URL")?,
            config,
        ),
    )
    .await
    .context("Redis connection timed out")?
    .context("Unable to connect to Redis")
}

pub async fn ping(manager: &ConnectionManager) -> Result<()> {
    let mut connection = manager.clone();
    let pong: String = redis::cmd("PING").query_async(&mut connection).await?;
    ensure!(pong == "PONG", "Unexpected Redis PING response");
    Ok(())
}
