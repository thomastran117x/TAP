use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

pub(super) fn layer(origin: HeaderValue) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods(Any)
        .allow_headers(Any)
}
