mod support;

use axum::{Router, http::StatusCode, routing::get};
use tap_backend::middleware;

#[tokio::test]
async fn cors_allows_the_configured_origin() {
    let app = middleware::apply(
        Router::new().route("/test", get(|| async { "ok" })),
        "http://localhost:4200".parse().unwrap(),
    );
    let (status, headers, body) =
        support::request(app.clone(), "/test", "http://localhost:4200").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers["access-control-allow-origin"],
        "http://localhost:4200"
    );
    assert_eq!(body, "ok");
    let (_, headers, _) = support::request(app, "/test", "http://example.com").await;
    // Browsers reject a response whose allowed origin does not match the request's origin.
    assert_ne!(
        headers
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://example.com")
    );
}
