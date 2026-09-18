mod support;

use axum::http::StatusCode;
use tap_backend::app::{AppState, Config, router};

#[tokio::test]
#[ignore = "requires running services matching the selected configuration"]
async fn live_services_and_routes() {
    let config = Config::from_env().unwrap();
    let state = AppState::connect(&config).await.unwrap();
    let app = router(&config, state.clone());
    let origin = config.cors_origin.to_str().unwrap();
    for (path, expected) in [
        ("/health", r#"{"status":"ok"}"#),
        (
            "/ready",
            r#"{"status":"ready","postgres":true,"redis":true,"opensearch":true}"#,
        ),
    ] {
        let (status, headers, body) = support::request(app.clone(), path, origin).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers["access-control-allow-origin"], config.cors_origin);
        assert_eq!(body, expected);
    }
    // Closing this test's pool does not affect the running Compose backend.
    state.postgres.close().await;
    let (status, _, body) = support::request(app.clone(), "/ready", origin).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let error: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(error["error"]["code"], "service_unavailable");
    assert_eq!(
        error["error"]["details"],
        serde_json::json!({
            "postgres": false, "redis": true, "opensearch": true,
        })
    );
    let (status, _, body) = support::request(app, "/health", origin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok"}"#);
}
