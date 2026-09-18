# Architecture

[Documentation index](README.md)

TAP has two application processes. Angular renders the frontend and serves its
browser assets through an Express SSR host. The Rust API owns application HTTP
behavior and shares clients for Postgres, Redis, and OpenSearch.

## Backend boundaries

| Directory                      | Responsibility                                                                      |
| ------------------------------ | ----------------------------------------------------------------------------------- |
| `backend/src/app/`             | Configuration, shared HTTP error contract, state, router assembly, server lifecycle |
| `backend/src/infra/`           | External client construction, connectivity operations, logging setup                |
| `backend/src/middleware/`      | Error translation, CORS, request tracing                                            |
| `backend/src/features/<name>/` | Feature routes, transport models, services, and persistence queries as needed       |
| `backend/tests/`               | Public API checks and opt-in live integration tests                                 |

`main.rs` delegates to the library's `app::run`. Startup loads and validates
configuration, initializes logging, connects the three infrastructure clients,
binds the listener, and serves the assembled router. Startup fails if required
connections cannot be established. Ctrl+C or SIGTERM begins graceful shutdown;
the Postgres pool closes after the server finishes.

`AppState` holds the Postgres pool, Redis connection manager, OpenSearch client,
and dependency timeout. Handlers reuse these clients rather than creating a
connection for each request. Infrastructure does not depend on features or
application assembly.

## Request flow

The middleware wraps the router in this order, from outermost to innermost:

```text
Request tracing -> CORS -> error translation -> feature handler/service
```

Handlers extract and validate HTTP input and delegate application logic to
services. Fallible handlers return `app::HttpResult<T>`; typed `HttpError`
values serialize through `IntoResponse`. The error middleware normalizes other
4xx/5xx responses, including Axum rejections. CORS and tracing apply to errors
as well as successes. See the [API reference](api.md) for the response contract.

The health feature is the first implemented capability. `/health` checks
liveness; `/ready` checks all three dependencies concurrently with deadlines.

## Add a feature

1. Create `backend/src/features/<name>/` and register its module in
   `features/mod.rs`.
2. Expose a `routes() -> Router<AppState>` function in the feature's `mod.rs`.
   Keep implementation modules private unless they need an explicit shared API.
3. Keep extraction and responses in handlers, request/response contracts in
   models, and application logic in services. Add persistence queries within
   the owning feature when needed.
4. Merge the feature routes in `app/router.rs`. Reuse the existing middleware
   stack and state.
5. Map expected domain failures to shared HTTP errors. Add context to unexpected
   errors, and keep internal sources out of client messages and details.
6. Add deterministic contract tests and live tests for meaningful persistence
   behavior. Update configuration and API documentation when contracts change.

Use versioned migrations for schema changes. A migration runner has not been
implemented yet; include application steps when introducing one.

## Frontend

`frontend/src/app/` contains standalone Angular components, routes, and
providers. SSR entry points include `src/main.server.ts` and `src/server.ts`.
Global styling uses Tailwind and `src/styles.css`; static assets live in
`public/`.

Keep API calls in typed Angular services and application endpoints in Axum.
Browser-only APIs must be accessed in a browser-safe context so server rendering
works. The Express process is the Angular rendering host.

## Compose environments

The base and development configurations use the `tap` project and persistent
named service volumes. The standalone test configuration uses `tap-tests`,
independent containers, and disposable storage. It exposes no host ports.
See [configuration](configuration.md) and [testing](testing.md) for the details.
