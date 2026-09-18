use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::DefaultBodyLimit,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tap_backend::{
    app::{HttpError, HttpResult},
    middleware,
};
use tower::ServiceExt;

fn app(router: Router) -> Router {
    middleware::apply(router, "http://localhost:4200".parse().unwrap())
}

async fn send(router: Router, method: Method, path: &str, body: &str) -> Response {
    router
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("origin", "http://localhost:4200")
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    assert_eq!(response.headers()["content-type"], "application/json");
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn typed_errors_keep_public_messages_and_details() {
    let router = app(Router::new().route(
        "/test",
        get(|| async {
            Err::<(), _>(
                HttpError::validation("Name is required.")
                    .with_details(json!({"fields": {"name": "required"}})),
            )
        }),
    ));
    let response = send(router, Method::GET, "/test", "").await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "http://localhost:4200"
    );
    assert_eq!(
        json_body(response).await,
        json!({"error": {
            "code": "validation_error", "message": "Name is required.",
            "details": {"fields": {"name": "required"}}
        }})
    );
}

#[tokio::test]
async fn internal_sources_are_hidden_when_propagated_with_question_mark() {
    async fn handler() -> HttpResult<()> {
        Err::<(), _>(anyhow::anyhow!("database password=private-value"))?;
        Ok(())
    }
    let response = send(
        app(Router::new().route("/test", get(handler))),
        Method::GET,
        "/test",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        json_body(response).await,
        json!({"error": {
            "code": "internal_error",
            "message": "An unexpected error occurred. Please try again later."
        }})
    );
}

#[tokio::test]
async fn routing_errors_are_json_and_keep_allow_headers() {
    let router = app(Router::new().route("/test", get(|| async { "ok" })));
    let missing = send(router.clone(), Method::GET, "/missing", "").await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(missing).await["error"]["code"], "not_found");
    let wrong_method = send(router, Method::POST, "/test", "").await;
    assert_eq!(wrong_method.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        wrong_method.headers()["allow"]
            .to_str()
            .unwrap()
            .contains("GET")
    );
    assert_eq!(
        json_body(wrong_method).await["error"]["code"],
        "method_not_allowed"
    );
}

#[derive(Deserialize)]
struct Input {
    count: u32,
}

#[tokio::test]
async fn extractor_rejections_use_the_shared_contract() {
    let router = app(Router::new()
        .route(
            "/test",
            post(|Json(input): Json<Input>| async move { input.count.to_string() }),
        )
        .layer(DefaultBodyLimit::max(64)));
    for (body, status, code) in [
        ("{", StatusCode::BAD_REQUEST, "bad_request"),
        (
            r#"{"count":"private-value"}"#,
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
        ),
        (
            &"x".repeat(65),
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
        ),
    ] {
        let response = send(router.clone(), Method::POST, "/test", body).await;
        assert_eq!(response.status(), status);
        let value = json_body(response).await;
        assert_eq!(value["error"]["code"], code);
        assert!(!value.to_string().contains("private-value"));
    }
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/test")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        json_body(response).await["error"]["code"],
        "unsupported_media_type"
    );
}

#[tokio::test]
async fn unformatted_errors_hide_the_body_and_preserve_protocol_headers() {
    let router = app(Router::new()
        .route(
            "/internal",
            get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "private-value") }),
        )
        .route(
            "/limited",
            get(|| async {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", "30")],
                    "private-value",
                )
            }),
        )
        .route(
            "/auth",
            get(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    [("www-authenticate", "Bearer")],
                    "private-value",
                )
            }),
        ));
    for (path, header, code) in [
        ("/internal", None, "internal_error"),
        ("/limited", Some(("retry-after", "30")), "too_many_requests"),
        (
            "/auth",
            Some(("www-authenticate", "Bearer")),
            "unauthorized",
        ),
    ] {
        let response = send(router.clone(), Method::GET, path, "").await;
        if let Some((name, value)) = header {
            assert_eq!(response.headers()[name], value);
        }
        let value = json_body(response).await;
        assert_eq!(value["error"]["code"], code);
        assert!(!value.to_string().contains("private-value"));
    }
}

#[tokio::test]
async fn head_errors_have_no_body_and_successes_are_unchanged() {
    let router = app(Router::new().route("/test", get(|| async { "ok" })));
    let response = send(router.clone(), Method::HEAD, "/missing", "").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(
        to_bytes(response.into_body(), 4096)
            .await
            .unwrap()
            .is_empty()
    );
    let response = send(router, Method::GET, "/test", "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(to_bytes(response.into_body(), 4096).await.unwrap(), "ok");
}

#[tokio::test]
async fn typed_errors_also_work_without_the_translation_layer() {
    let response = HttpError::not_found("Account was not found.").into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json_body(response).await,
        json!({"error": {
            "code": "not_found", "message": "Account was not found."
        }})
    );
}
