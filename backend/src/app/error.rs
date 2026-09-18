use std::{borrow::Cow, fmt};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

/// A client-safe HTTP failure. Return it from handlers using [`HttpResult`].
///
/// Messages and details supplied explicitly are public data. Internal error
/// sources are logged by the server and never serialized into client responses.
#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    code: &'static str,
    message: Cow<'static, str>,
    details: Option<Value>,
    source: Option<anyhow::Error>,
}

/// The result type for handlers that can return shared HTTP errors.
pub type HttpResult<T> = Result<T, HttpError>;

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

/// Identifies responses already serialized through the shared error contract.
#[derive(Clone)]
pub(crate) struct FormattedError;

impl HttpError {
    /// A malformed request (400). The supplied message is visible to clients.
    pub fn bad_request(message: impl Into<Cow<'static, str>>) -> Self {
        Self::from_status(StatusCode::BAD_REQUEST).with_message(message)
    }

    /// Authentication is required or invalid (401).
    pub fn unauthorized() -> Self {
        Self::from_status(StatusCode::UNAUTHORIZED)
    }

    /// The caller lacks permission (403).
    pub fn forbidden() -> Self {
        Self::from_status(StatusCode::FORBIDDEN)
    }

    /// A missing resource (404). The supplied message is visible to clients.
    pub fn not_found(message: impl Into<Cow<'static, str>>) -> Self {
        Self::from_status(StatusCode::NOT_FOUND).with_message(message)
    }

    /// The endpoint does not support the requested method (405).
    pub fn method_not_allowed() -> Self {
        Self::from_status(StatusCode::METHOD_NOT_ALLOWED)
    }

    /// The request was not received in time (408).
    pub fn request_timeout() -> Self {
        Self::from_status(StatusCode::REQUEST_TIMEOUT)
    }

    /// A resource conflict (409). The supplied message is visible to clients.
    pub fn conflict(message: impl Into<Cow<'static, str>>) -> Self {
        Self::from_status(StatusCode::CONFLICT).with_message(message)
    }

    /// The request body exceeds the permitted size (413).
    pub fn payload_too_large() -> Self {
        Self::from_status(StatusCode::PAYLOAD_TOO_LARGE)
    }

    /// The request's content type is unsupported (415).
    pub fn unsupported_media_type() -> Self {
        Self::from_status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
    }

    /// Invalid submitted values (422). The supplied message is visible to clients.
    pub fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Self::from_status(StatusCode::UNPROCESSABLE_ENTITY).with_message(message)
    }

    /// A rate limit has been exceeded (429).
    pub fn too_many_requests() -> Self {
        Self::from_status(StatusCode::TOO_MANY_REQUESTS)
    }

    /// An unexpected failure (500), with its source retained for server diagnostics.
    pub fn internal(source: impl Into<anyhow::Error>) -> Self {
        let mut error = Self::from_status(StatusCode::INTERNAL_SERVER_ERROR);
        error.source = Some(source.into());
        error
    }

    /// An upstream service returned an invalid response (502).
    pub fn bad_gateway() -> Self {
        Self::from_status(StatusCode::BAD_GATEWAY)
    }

    /// The service is temporarily unavailable (503).
    pub fn service_unavailable() -> Self {
        Self::from_status(StatusCode::SERVICE_UNAVAILABLE)
    }

    /// An upstream service did not respond in time (504).
    pub fn gateway_timeout() -> Self {
        Self::from_status(StatusCode::GATEWAY_TIMEOUT)
    }

    /// Attach explicitly client-safe metadata, such as field validation failures.
    /// Never include raw dependency errors, credentials, or sensitive input here.
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// The HTTP status associated with this failure.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    fn with_message(mut self, message: impl Into<Cow<'static, str>>) -> Self {
        self.message = message.into();
        self
    }

    pub(crate) fn from_status(status: StatusCode) -> Self {
        let (code, message) = match status {
            StatusCode::BAD_REQUEST => (
                "bad_request",
                "The request is invalid. Check its format and parameters.",
            ),
            StatusCode::UNAUTHORIZED => (
                "unauthorized",
                "Authentication is required to access this resource.",
            ),
            StatusCode::FORBIDDEN => (
                "forbidden",
                "You do not have permission to access this resource.",
            ),
            StatusCode::NOT_FOUND => ("not_found", "The requested resource was not found."),
            StatusCode::METHOD_NOT_ALLOWED => (
                "method_not_allowed",
                "This endpoint does not support the requested HTTP method.",
            ),
            StatusCode::REQUEST_TIMEOUT => (
                "request_timeout",
                "The request timed out. Please try again.",
            ),
            StatusCode::CONFLICT => (
                "conflict",
                "The request conflicts with the current state of the resource.",
            ),
            StatusCode::PAYLOAD_TOO_LARGE => (
                "payload_too_large",
                "The request body exceeds the allowed size.",
            ),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => (
                "unsupported_media_type",
                "The request's content type is not supported.",
            ),
            StatusCode::UNPROCESSABLE_ENTITY => (
                "validation_error",
                "Some submitted values are invalid. Check the request fields.",
            ),
            StatusCode::TOO_MANY_REQUESTS => (
                "too_many_requests",
                "Too many requests. Please try again later.",
            ),
            StatusCode::BAD_GATEWAY => (
                "bad_gateway",
                "An upstream service returned an invalid response.",
            ),
            StatusCode::SERVICE_UNAVAILABLE => (
                "service_unavailable",
                "The service is temporarily unavailable. Please try again later.",
            ),
            StatusCode::GATEWAY_TIMEOUT => (
                "gateway_timeout",
                "An upstream service timed out. Please try again later.",
            ),
            _ if status.is_client_error() => {
                ("request_error", "The request could not be completed.")
            }
            _ => (
                "internal_error",
                "An unexpected error occurred. Please try again later.",
            ),
        };
        Self {
            status,
            code,
            message: message.into(),
            details: None,
            source: None,
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|source| source.as_ref())
    }
}

impl From<anyhow::Error> for HttpError {
    fn from(source: anyhow::Error) -> Self {
        Self::internal(source)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        if let Some(source) = &self.source {
            tracing::error!(error = ?source, "Request failed with an internal error");
        }
        let mut response = (
            self.status,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                    details: self.details,
                },
            }),
        )
            .into_response();
        response.extensions_mut().insert(FormattedError);
        response
    }
}
