# TAP

TAP is an Angular application with server-side rendering and a Rust HTTP API
built with Axum and Tokio. Docker Compose runs the API alongside Postgres,
Redis, and OpenSearch.

The repository currently provides the application foundation: infrastructure
clients, YAML configuration, shared HTTP errors, health/readiness endpoints,
and automated quality and integration checks.

## Get started

Install Docker with the Compose plugin and start Docker Desktop or your Docker
daemon. Run these commands from the repository root:

```sh
docker compose up --build
```

Open the frontend at **http://localhost:4000**. The API runs at
**http://localhost:3000**; visit `/health` for liveness and `/ready` for dependency
status. Download the OpenAPI contract at `/openapi.yaml`. The first build downloads
dependencies and can take several minutes.

No `.env` file is required. Defaults work for the local Compose stack. The
optional [.env.example](.env.example) lists credentials and published ports you
can customize through a root `.env` file.

Stop the attached stack with Ctrl+C, then remove its containers:

```sh
docker compose down
```

Normal shutdown preserves development data in named volumes. The Compose stack
uses local credentials and disables OpenSearch security; review deployment
configuration before exposing services outside your development environment.

## Development

For mounted source and the Angular development server:

```sh
docker compose -f compose.yml -f compose.dev.yml up --build
```

Open **http://localhost:4200**. Angular reloads frontend changes. After editing
Rust source or bundled YAML, recompile by restarting the backend:

```sh
docker compose -f compose.yml -f compose.dev.yml restart backend
```

Host development requires Rust 1.92 or newer, Node 24, npm 11.16.0, and native
C/C++ tools for the Rust TLS dependencies. See the [development guide](docs/development.md)
for host commands and daily workflows.

## Project layout

| Path                                                 | Purpose                                                     |
| ---------------------------------------------------- | ----------------------------------------------------------- |
| [backend/](backend/README.md)                        | Rust API, YAML defaults, infrastructure clients, tests      |
| [frontend/](frontend/README.md)                      | Angular application and SSR host                            |
| [docs/](docs/README.md)                              | Architecture, setup, configuration, testing, and API guides |
| [compose.yml](compose.yml)                           | Local stack using built application images                  |
| [compose.dev.yml](compose.dev.yml)                   | Development override with mounted source                    |
| [compose.test.yml](compose.test.yml)                 | Independent integration test stack                          |
| [.github/workflows/ci.yml](.github/workflows/ci.yml) | Formatting, builds, quality, and tests                      |
| [AGENTS.md](AGENTS.md)                               | Repository conventions and Rust coding practices            |

The backend separates application assembly, infrastructure, middleware, and
feature modules. Read the [architecture guide](docs/architecture.md) before
adding a feature.

## Checks and integration tests

Frontend checks run from `frontend`:

```sh
npm ci
npm run format:check
npm run build
npm test -- --watch=false
```

Backend checks run from `backend`:

```sh
cargo fmt --check
cargo build --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Run the isolated integration suite from the repository root, then clean up its
containers and disposable volumes after success or failure:

```sh
docker compose -f compose.test.yml up --build --abort-on-container-exit --exit-code-from integration integration
docker compose -f compose.test.yml down --volumes --remove-orphans
```

CI runs these checks automatically on pushes, pull requests, and manual
dispatch. See [testing](docs/testing.md) for Docker validation, test isolation,
and native integration commands.

## Documentation and contributing

Start at the [documentation index](docs/README.md). The
[configuration guide](docs/configuration.md) explains YAML profiles and
environment overrides; the [API reference](docs/api.md) documents endpoints
and the shared JSON error contract.
The machine-readable contract is [backend/openapi.yaml](backend/openapi.yaml),
also served by the API and validated in CI.

Follow [CONTRIBUTING.md](CONTRIBUTING.md) when making changes. AI coding agents
should also read [AGENTS.md](AGENTS.md); [CLAUDE.md](CLAUDE.md) points to the same
shared guidance.
