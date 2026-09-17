mod handlers;
mod models;
mod service;

use axum::{Router, routing::get};

use crate::app::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
}
