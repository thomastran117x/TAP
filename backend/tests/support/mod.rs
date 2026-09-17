use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

pub async fn request(
    app: Router,
    path: &str,
    origin: &str,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(path)
                .header("origin", origin)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    (status, headers, String::from_utf8(body.to_vec()).unwrap())
}
