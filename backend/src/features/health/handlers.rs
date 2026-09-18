use axum::{Json, extract::State};

use super::{
    models::{Health, Readiness},
    service,
};
use crate::app::{AppState, HttpError, HttpResult};

pub(super) async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

pub(super) async fn ready(State(state): State<AppState>) -> HttpResult<Json<Readiness>> {
    let readiness = service::readiness(&state).await;
    if readiness.postgres && readiness.redis && readiness.opensearch {
        Ok(Json(readiness))
    } else {
        Err(
            HttpError::service_unavailable().with_details(serde_json::json!({
                "postgres": readiness.postgres,
                "redis": readiness.redis,
                "opensearch": readiness.opensearch,
            })),
        )
    }
}
