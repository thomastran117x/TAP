use axum::{Json, extract::State, http::StatusCode};

use super::{
    models::{Health, Readiness},
    service,
};
use crate::app::AppState;

pub(super) async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

pub(super) async fn ready(State(state): State<AppState>) -> (StatusCode, Json<Readiness>) {
    let readiness = service::readiness(&state).await;
    let status = if readiness.postgres && readiness.redis && readiness.opensearch {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(readiness))
}
