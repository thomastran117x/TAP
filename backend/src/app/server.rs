use anyhow::{Context, Result};

use super::{AppState, Config, router};
use crate::infra::telemetry;

pub async fn run() -> Result<()> {
    let config = Config::from_env()?;
    telemetry::init(&config.log_filter);
    let state = AppState::connect(&config).await?;
    let listener = tokio::net::TcpListener::bind(config.address)
        .await
        .context("Unable to bind server")?;
    tracing::info!(address = %config.address, "Backend listening");
    axum::serve(listener, router(&config, state.clone()))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    state.postgres.close().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Unable to install Ctrl+C handler")
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Unable to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("Shutting down");
}
