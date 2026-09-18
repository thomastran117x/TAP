//! HTTP layers shared by all features, applied in one place.

mod cors;
mod errors;

use axum::{Router, http::HeaderValue, middleware::from_fn};
use tower_http::trace::TraceLayer;

pub fn apply<S>(router: Router<S>, origin: HeaderValue) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .layer(from_fn(errors::translate))
        .layer(cors::layer(origin))
        .layer(TraceLayer::new_for_http())
}
