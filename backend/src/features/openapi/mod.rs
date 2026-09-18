//! Serve the checked-in API contract independently of infrastructure clients.

use axum::{Router, http::header, routing::get};

const DOCUMENT: &str = include_str!("../../../openapi.yaml");

/// Register the embedded OpenAPI YAML endpoint for any application state.
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/openapi.yaml", get(document))
}

async fn document() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    ([(header::CONTENT_TYPE, "application/yaml")], DOCUMENT)
}
