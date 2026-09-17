//! HTTP layers shared by all features, applied in one place.

mod cors;

use axum::{Router, http::HeaderValue};
use tower_http::trace::TraceLayer;

pub fn apply<S>(router: Router<S>, origin: HeaderValue) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .layer(cors::layer(origin))
        .layer(TraceLayer::new_for_http())
}
