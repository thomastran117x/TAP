# TAP backend

Rust 1.92+ server using Axum and Tokio. Shared application state contains a SQLx
Postgres pool, a reconnecting Redis connection manager, and the official OpenSearch
client. Startup checks all three services and fails with a contextual error if a
connection cannot be established. Requests are traced, CORS allows the configured
frontend origin, and Ctrl+C/SIGTERM shuts down the server gracefully.

See the [repository README](../README.md) for the full stack and
[documentation index](../docs/README.md) for development, architecture, testing,
and the [HTTP API reference](../docs/api.md).

## Structure

```text
backend/
  config/
    dev.yml                 # Local development defaults
    test.yml                # Test defaults
    prod.yml                # Production defaults; credentials supplied externally
  src/
    main.rs                 # Thin binary entry point
    lib.rs                  # Library modules used by the server and tests
    app/
      mod.rs
      config.rs             # YAML configuration, environment overrides, validation
      error.rs              # Shared HTTP errors and JSON response contract
      router.rs             # Combine feature routes and apply middleware
      server.rs             # Startup, listener, shutdown, and cleanup
      state.rs              # Assemble shared infrastructure clients
    infra/
      mod.rs
      postgres.rs           # SQLx pool construction and connectivity checks
      redis.rs              # Redis connection manager and connectivity checks
      opensearch.rs         # OpenSearch client and connectivity checks
      telemetry.rs          # Logging initialization
    middleware/
      mod.rs                # Shared HTTP layer stack, including request tracing
      cors.rs               # Frontend origin policy
      errors.rs             # Normalize framework and unformatted HTTP errors
    features/
      mod.rs
      health/
        mod.rs              # Feature route registration
        handlers.rs         # HTTP inputs, outputs, and status codes
        models.rs           # Response contracts
        service.rs          # Readiness logic and dependency orchestration
  tests/
    config.rs               # Configuration checks without process env mutation
    errors.rs               # Error contracts, extractor rejections, and headers
    middleware.rs           # HTTP middleware behavior without external services
    integration.rs          # Opt-in live integration test target
    integration/
      api.rs                # Assembled routes, CORS, health, readiness failures
      infrastructure.rs     # Postgres, Redis, and OpenSearch round trips
      support.rs            # Test configuration and unique resource names
    support/mod.rs          # Shared request helpers
```

`app` wires infrastructure, features, and middleware and owns the shared HTTP
error contract.
`infra` owns external client setup and operations and does not depend on the app
or HTTP handlers. `middleware` applies shared HTTP policies without depending on
any feature or client. Features own their routes, contracts, and application
logic; handlers delegate to services, which use infrastructure through the clients
in `AppState`.

Add a capability under `src/features/<name>/`, expose its routes from `mod.rs`,
and merge them in `app/router.rs`. Keep handlers, models, services, and database
queries within the feature as needed; add shared client setup to `infra` and
shared HTTP layers to `middleware`. Feature implementation modules stay private
unless another module needs an explicit public API.

Tests under `tests/` exercise the library's public API. Small unit tests for
private logic live beside that implementation. Live integration tests are ignored
by default; ordinary tests need no external services.

## Docker Compose

From the repository root:

```sh
docker compose up --build backend
```

This starts the backend and its Postgres, Redis, and OpenSearch dependencies.
Add `frontend` to the command to start the full application. The API is available
at `http://localhost:3000` by default. Containers connect through service names
(`postgres`, `redis`, `opensearch`); published ports are for host access.

For development:

```sh
docker compose -f compose.yml -f compose.dev.yml up --build backend
```

Source is mounted and Cargo downloads/builds are cached in named volumes.
After editing Rust source, restart the backend to recompile:

```sh
docker compose -f compose.yml -f compose.dev.yml restart backend
```

Compose selects `prod.yml` normally and `dev.yml` with the development override.
It supplies the database URL to match the Postgres container credentials, service
addresses for local development containers, and the frontend's published origin.

The existing OpenSearch Compose service has security disabled, so it does not
require credentials. For an external authenticated cluster, set both
`OPENSEARCH_USERNAME` and `OPENSEARCH_PASSWORD` in the backend environment.

## Run on the host

Local builds also require the platform's C/C++ build tools for TLS dependencies
(including Clang on Windows ARM64). The Docker build provides the Linux toolchain.

Start dependencies from the repository root:

```sh
docker compose up -d postgres redis opensearch
```

Then, from `backend`, run:

```sh
cargo run --locked
```

No `.env` file is needed. Local startup uses `config/dev.yml`. If you customize
Compose's published service ports or database credentials, override the matching
backend settings through environment variables or a custom configuration directory.

## Configuration

`APP_ENV` selects `dev`, `test`, or `prod` and defaults to `dev`. Other values fail
startup. The selected YAML file supplies application defaults; explicitly set
environment variables take precedence. The backend does not read any `.env` files.
The root `.env` remains optional input to Docker Compose for container credentials
and published ports.

The three YAML files are embedded in the binary, so defaults work independently of
the working directory and are included in the production image. Editing bundled
YAML requires recompilation, just like editing Rust source. To load files at
runtime, set `CONFIG_DIR` to a directory containing a complete `<APP_ENV>.yml` file.
This replaces the embedded profile; missing files, malformed YAML, unknown fields,
and invalid settings fail startup. Environment variables still override that file.

| YAML setting | Environment override |
| --- | --- |
| `server.host` | `HOST` |
| `server.port` | `PORT` |
| `server.cors_origin` | `CORS_ORIGIN` |
| `database.url` | `DATABASE_URL` |
| `database.max_connections` | `DATABASE_MAX_CONNECTIONS` |
| `redis.url` | `REDIS_URL` |
| `opensearch.node` | `OPENSEARCH_NODE` |
| `opensearch.username` | `OPENSEARCH_USERNAME` |
| `opensearch.password` | `OPENSEARCH_PASSWORD` |
| `health.timeout_ms` | `DEPENDENCY_TIMEOUT_MS` |
| `logging.filter` | `RUST_LOG` |

`prod.yml` requires `DATABASE_URL` from the deployment environment. Supply real
credentials through your deployment's secret configuration. The example
development and test database credentials match the local Compose defaults.
`test.yml` uses a separate `tap_test` database and Redis database 1; create the test
database before running host tests against your own services. The dedicated test
stack provisions it automatically. Integration tests use unique Redis keys and
OpenSearch indexes and remove them after use.

PowerShell examples from `backend`:

```powershell
$env:APP_ENV = 'test'
cargo test --locked --test integration -- --ignored

$env:APP_ENV = 'prod'
$env:DATABASE_URL = 'postgresql://user:password@postgres:5432/tap'
cargo run --locked
```

## Health and validation

- `GET /health`: liveness, returns `200 {"status":"ok"}`.
- `GET /ready`: runs Postgres `SELECT 1`, Redis `PING`, and OpenSearch `HEAD /`
  concurrently with the configured timeout (three seconds by default). Returns 200 when all succeed, or 503
  when any fail. Successful responses contain a boolean for each service;
  failures use the shared error envelope with those booleans in `error.details`.
  Compose uses this endpoint.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# With dependencies running and a matching profile/environment:
cargo test --locked -- --ignored
```

The live suite checks service round trips, health/readiness, CORS, HTTP error
contracts, and readiness failure when the test's Postgres pool is closed.

To run formatting, Clippy, and unit tests with the Docker toolchain:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

Run this command from the repository root.

## Integration tests

From the repository root, run the complete isolated suite:

```sh
docker compose -f compose.test.yml up --build --abort-on-container-exit --exit-code-from integration integration
docker compose -f compose.test.yml down --volumes --remove-orphans
```

Run the cleanup command after success or failure. This standalone Compose file
uses the `tap-tests` project, provisions `tap_test`, and waits for Postgres,
Redis, and OpenSearch to become healthy before starting the Rust test runner.
It has no published ports or persistent named volumes and runs independently of
the development stack. The runner's exit code reflects the test result.
Use a different `-p` project name on both commands for concurrent suite runs.

The suite exercises the assembled Axum router in process with real dependency
clients. Postgres tests use a temporary table and roll back their transaction;
Redis tests set expiring keys; OpenSearch tests create, index, search, and delete
a unique index. Resource names are unique across parallel tests. Cleanup is
attempted on failed Redis/OpenSearch assertions too. No tests flush Redis or
delete shared indexes or database tables.

For a native toolchain with your own test services, from `backend`:

```sh
cargo test --locked --test integration -- --ignored
```

Test helpers always load `config/test.yml` and accept the usual environment
overrides without mutating process environment. They require database name
`tap_test` and Redis database 1 to avoid accidental development/production use.
Supply `DATABASE_URL`, `REDIS_URL`, and `OPENSEARCH_NODE` for the actual endpoints;
the isolated stack itself does not expose host ports. Ordinary `cargo test`
compiles the suite and skips live tests; Clippy checks it with `--all-targets`.

CI runs the same Compose command in a separate backend integration job, prints
service logs on failure, and always removes the test containers and volumes.

## HTTP errors

Fallible handlers return `app::HttpResult<T>` and use `app::HttpError` for
expected failures. Rust propagates errors with `Result` and `?`. For example:

```rust
use tap_backend::app::{HttpError, HttpResult};

async fn handler() -> HttpResult<()> {
    Err(HttpError::not_found("Account was not found."))
}
```

The response has status 404 and this JSON body:

```json
{"error":{"code":"not_found","message":"Account was not found."}}
```

Constructors cover bad requests (400), authentication (401), permission (403),
missing resources (404), unsupported methods (405), request timeouts (408),
conflicts (409), oversized bodies (413), unsupported content types (415),
validation (422), rate limits (429), internal failures (500), and upstream
failures (502/503/504). Codes are stable snake_case identifiers; validation uses
`validation_error` and unexpected failures use `internal_error`.

Use `.with_details(serde_json::json!({...}))` for optional public metadata such
as field validation failures. Explicit messages and details are visible to
clients and must contain only safe information. Map known domain failures to
appropriate errors in the feature. Convert unexpected dependency failures with
`HttpError::internal(source)`; `anyhow::Error` also converts through `?`. These
sources are logged for diagnostics and excluded from responses. Internal errors
always return a generic readable message.

The error middleware normalizes unformatted 4xx/5xx responses, including Axum
route, method, and extractor rejections, to the same JSON envelope. It preserves
HTTP status and protocol headers such as `Allow`, `WWW-Authenticate`, and
`Retry-After`, keeps HEAD bodies empty, and leaves successful responses intact.
Typed errors preserve their public messages and details. CORS and tracing wrap
this layer so error responses receive the same policies as successful requests.
