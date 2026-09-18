use std::{
    env,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, ensure};
use tap_backend::app::{AppState, Config};

pub(super) async fn connect() -> Result<(Config, AppState)> {
    // Always load test defaults, even if the parent shell selects dev or prod.
    let config = Config::from_lookup(|name| {
        if name == "APP_ENV" {
            Some("test".into())
        } else {
            env::var(name).ok()
        }
    })?;
    let database = opensearch::http::Url::parse(&config.database_url)?;
    ensure!(
        database.path() == "/tap_test",
        "Integration tests require the tap_test database"
    );
    let redis = opensearch::http::Url::parse(&config.redis_url)?;
    ensure!(
        redis.path() == "/1",
        "Integration tests require Redis database 1"
    );
    let state = AppState::connect(&config)
        .await
        .context("Unable to connect to integration services; run the stack in compose.test.yml")?;
    Ok((config, state))
}

pub(super) fn unique_name(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!(
        "tap-test-{prefix}-{}-{nanos}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}
