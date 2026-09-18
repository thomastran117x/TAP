use super::support;
#[path = "../support/mod.rs"]
mod requests;

use axum::http::StatusCode;
use tap_backend::app::router;
use tower::ServiceExt;

#[tokio::test]
#[ignore = "requires running services matching the selected configuration"]
async fn live_services_and_routes() {
    let (config, state) = support::connect().await.unwrap();
    let app = router(&config, state.clone());
    let origin = config.cors_origin.to_str().unwrap();
    let spec: serde_json::Value =
        serde_yaml_ng::from_str(include_str!("../../openapi.yaml")).unwrap();
    for path in ["/health", "/ready"] {
        let (status, headers, body) = requests::request(app.clone(), path, origin).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers["access-control-allow-origin"], config.cors_origin);
        let body: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            body,
            spec["paths"][path]["get"]["responses"]["200"]["content"]["application/json"]["example"]
        );
    }
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/openapi.yaml")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/yaml");
    let document = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    assert_eq!(document, include_str!("../../openapi.yaml"));
    // Closing this test's pool does not affect the running Compose backend.
    state.postgres.close().await;
    let (status, _, body) = requests::request(app.clone(), "/ready", origin).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let error: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        error,
        spec["paths"]["/ready"]["get"]["responses"]["503"]["content"]["application/json"]["example"]
    );
    let (status, _, body) = requests::request(app, "/health", origin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok"}"#);
}

#[tokio::test]
#[ignore = "requires the integration test services"]
async fn assembled_router_returns_consistent_errors_and_cors_headers() {
    use axum::{body::Body, http::Request};

    let (config, state) = support::connect().await.unwrap();
    let app = router(&config, state.clone());
    for (method, path, status, code) in [
        ("GET", "/missing", StatusCode::NOT_FOUND, "not_found"),
        (
            "POST",
            "/health",
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("origin", &config.cors_origin)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(
            response.headers()["access-control-allow-origin"],
            config.cors_origin
        );
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["code"], code);
    }
    state.postgres.close().await;
}
