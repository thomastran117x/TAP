use axum::{
    body::Body,
    extract::Request,
    http::{
        Method,
        header::{CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::app::{HttpError, error::FormattedError};

/// Normalize framework rejections and untyped errors without reading their bodies.
pub(super) async fn translate(request: Request, next: Next) -> Response {
    let is_head = request.method() == Method::HEAD;
    let response = next.run(request).await;
    let status = response.status();
    let mut response = if (status.is_client_error() || status.is_server_error())
        && response.extensions().get::<FormattedError>().is_none()
    {
        let (mut parts, _) = response.into_parts();
        let (error_parts, body) = HttpError::from_status(status).into_response().into_parts();
        // Retain protocol headers such as Allow, WWW-Authenticate, and Retry-After.
        // Representation headers from the discarded body no longer describe the JSON.
        parts.headers.remove(CONTENT_LENGTH);
        parts.headers.remove(CONTENT_ENCODING);
        if let Some(content_type) = error_parts.headers.get(CONTENT_TYPE) {
            parts.headers.insert(CONTENT_TYPE, content_type.clone());
        }
        parts.extensions.extend(error_parts.extensions);
        Response::from_parts(parts, body)
    } else {
        response
    };
    if is_head {
        *response.body_mut() = Body::empty();
    }
    response
}
