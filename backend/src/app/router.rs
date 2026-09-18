use axum::Router;

use super::{AppState, Config};
use crate::{features, middleware};

pub fn router(config: &Config, state: AppState) -> Router {
    let routes = Router::new()
        .merge(features::health::routes())
        .merge(features::openapi::routes());
    middleware::apply(routes, config.cors_origin.clone()).with_state(state)
}
