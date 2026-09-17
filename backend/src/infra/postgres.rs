use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

use std::time::Duration;

pub async fn connect(
    database_url: &str,
    max_connections: u32,
    timeout: Duration,
) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(timeout)
        .connect(database_url)
        .await
        .context("Unable to connect to Postgres")
}

pub async fn ping(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
