# HTTP API

[Documentation index](README.md)

The default local API base URL is `http://localhost:3000`. The implemented
endpoints are health and readiness. HTTP error constructors are shared building
blocks; authentication, authorization, and rate limiting have not yet been
implemented as application features.

## Liveness

`GET /health` returns status **200**:

```json
{ "status": "ok" }
```

This checks that the application can respond. It does not query dependencies.

## Readiness

`GET /ready` concurrently checks Postgres `SELECT 1`, Redis `PING`, and
OpenSearch `HEAD /`, each with the configured dependency deadline.

When all checks pass, status is **200**:

```json
{ "status": "ready", "postgres": true, "redis": true, "opensearch": true }
```

If a dependency fails or times out, status is **503**. For example:

```json
{
  "error": {
    "code": "service_unavailable",
    "message": "The service is temporarily unavailable. Please try again later.",
    "details": { "postgres": false, "redis": true, "opensearch": true }
  }
}
```

The base Compose backend healthcheck uses `/ready`. Readiness details contain
dependency availability flags, never raw dependency errors.

## Error contract

Error responses use `Content-Type: application/json` and retain their HTTP
status code. Their body contains an `error` object:

| Field     | Type                 | Meaning                                         |
| --------- | -------------------- | ----------------------------------------------- |
| `code`    | string               | Stable identifier clients can branch on         |
| `message` | string               | Readable public explanation                     |
| `details` | JSON value, optional | Explicitly public metadata; omitted when absent |

For example, an unknown route returns status **404**:

```json
{ "error": { "code": "not_found", "message": "The requested resource was not found." } }
```

| Status | Code                     |
| ------ | ------------------------ |
| 400    | `bad_request`            |
| 401    | `unauthorized`           |
| 403    | `forbidden`              |
| 404    | `not_found`              |
| 405    | `method_not_allowed`     |
| 408    | `request_timeout`        |
| 409    | `conflict`               |
| 413    | `payload_too_large`      |
| 415    | `unsupported_media_type` |
| 422    | `validation_error`       |
| 429    | `too_many_requests`      |
| 500    | `internal_error`         |
| 502    | `bad_gateway`            |
| 503    | `service_unavailable`    |
| 504    | `gateway_timeout`        |

Other unformatted 4xx responses use `request_error`; other unformatted 5xx
responses use `internal_error`. Middleware normalizes framework failures such
as missing routes, unsupported methods, and invalid request bodies to this shape.
It preserves protocol headers such as `Allow`, `WWW-Authenticate`, and
`Retry-After`. HEAD responses contain no body.

## Handler usage

Fallible handlers return `app::HttpResult<T>` and construct expected errors:

```rust
use tap_backend::app::{HttpError, HttpResult};

async fn handler() -> HttpResult<()> {
    Err(HttpError::not_found("Account was not found."))
}
```

Use `HttpError::validation(message).with_details(...)` for public field errors.
Map domain errors within the owning feature. `anyhow::Error` converts through
`?` to an internal error; other unexpected sources can be wrapped explicitly
with `HttpError::internal(source)`.

Internal failures return a generic message and log their source for diagnostics.
Explicit client messages and details must never contain credentials, sensitive
input, or internal driver errors. Clients should use `code` for logic rather
than matching message text.
