use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use tap_backend::{features::openapi, middleware};
use tower::ServiceExt;

fn app() -> Router {
    middleware::apply(openapi::routes(), "http://localhost:4200".parse().unwrap())
}

#[tokio::test]
async fn serves_the_checked_in_yaml_with_cors() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/openapi.yaml")
                .header("origin", "http://localhost:4200")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/yaml");
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "http://localhost:4200"
    );
    let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
    assert_eq!(bytes, include_str!("../openapi.yaml"));
    let document: serde_json::Value = serde_yaml_ng::from_slice(&bytes).unwrap();
    assert_eq!(document["openapi"], "3.1.1");
    assert_eq!(document["info"]["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn head_has_no_body_and_unsupported_methods_use_json_errors() {
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri("/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/yaml");
    assert!(
        to_bytes(response.into_body(), 65536)
            .await
            .unwrap()
            .is_empty()
    );
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/openapi.yaml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()["content-type"], "application/json");
    assert!(
        response.headers()["allow"]
            .to_str()
            .unwrap()
            .contains("GET")
    );
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "method_not_allowed");
}
