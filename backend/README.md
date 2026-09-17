# TAP backend

Rust 1.92+ server using Axum and Tokio. Shared application state contains a SQLx
Postgres pool, a reconnecting Redis connection manager, and the official OpenSearch
client. Startup checks all three services and fails with a contextual error if a
connection cannot be established. Requests are traced, CORS allows the configured
frontend origin, and Ctrl+C/SIGTERM shuts down the server gracefully.

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
    features/
      mod.rs
      health/
        mod.rs              # Feature route registration
        handlers.rs         # HTTP inputs, outputs, and status codes
        models.rs           # Response contracts
        service.rs          # Readiness logic and dependency orchestration
  tests/
    config.rs               # Configuration checks without process env mutation
    middleware.rs           # HTTP middleware behavior without external services
    health.rs               # Live service and route integration test
    support/mod.rs          # Shared request helpers
```

`app` is the composition root: it wires infrastructure, features, and middleware.
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
private logic live beside that implementation. The live health test is ignored
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
database before running live tests with `APP_ENV=test`. OpenSearch is shared, and
the current live test performs connectivity checks without writing search data.

PowerShell examples from `backend`:

```powershell
$env:APP_ENV = 'test'
cargo test --locked --test health -- --ignored

$env:APP_ENV = 'prod'
$env:DATABASE_URL = 'postgresql://user:password@postgres:5432/tap'
cargo run --locked
```

## Health and validation

- `GET /health`: liveness, returns `200 {"status":"ok"}`.
- `GET /ready`: runs Postgres `SELECT 1`, Redis `PING`, and OpenSearch `HEAD /`
  concurrently with the configured timeout (three seconds by default). Returns 200 when all succeed, or 503
  when any fail, with a boolean for each service. Compose uses this endpoint.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# With dependencies running and a matching profile/environment:
cargo test --locked -- --ignored
```

The live test checks service connectivity, health/readiness, CORS, and readiness
failure when the Postgres pool is closed.

To run formatting, Clippy, and unit tests with the Docker toolchain:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

Run this command from the repository root.
