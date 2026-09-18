# Testing

[Documentation index](README.md)

Regular tests run without Postgres, Redis, or OpenSearch. Live integration tests
are opt-in locally and run automatically in their own CI job.

## Frontend checks

From `frontend`, using Node 24 and npm 11.16.0:

```sh
npm ci
npm run format:check
npm run build
npm test -- --watch=false
```

Prettier checks formatting; the Angular production build checks TypeScript,
templates, and build budgets. Unit tests use the configured Angular/Vitest
runner. Use `npm run format` to apply formatting fixes.
Browser end-to-end testing has not been configured.

## Backend checks

From `backend`, using Rust 1.92 or newer:

```sh
cargo fmt --check
cargo build --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Use `cargo fmt` to apply formatting fixes. `--locked` preserves the checked-in
dependency resolution. Clippy checks the integration target too, even though
its live tests are ignored during regular runs.

If the host native toolchain is unavailable, from the repository root:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

This stage compiles a release build, checks formatting, runs Clippy with warnings
treated as errors, and executes regular tests. It skips live integration tests.

## Isolated integration suite

From the repository root:

```sh
docker compose -f compose.test.yml up --build --abort-on-container-exit --exit-code-from integration integration
docker compose -f compose.test.yml down --volumes --remove-orphans
```

Run cleanup after success or failure. The runner's exit code reflects the test
result. Services must become healthy before tests start. The `tap-tests` project
has its own network and containers, no published ports, and no persistent named
volumes. The cleanup command removes its disposable volumes.

For concurrent runs, supply a unique project name on both commands:

```sh
docker compose -p tap-tests-review -f compose.test.yml up --build --abort-on-container-exit --exit-code-from integration integration
docker compose -p tap-tests-review -f compose.test.yml down --volumes --remove-orphans
```

The suite uses real clients with the assembled Axum router in process. It covers:

| Area       | Contract                                                                                               |
| ---------- | ------------------------------------------------------------------------------------------------------ |
| API        | Liveness, readiness, CORS, 404/405 JSON errors, readiness failure after closing the test Postgres pool |
| Postgres   | Bound values round-trip through a temporary table; rollback removes the test table                     |
| Redis      | Set/get round trip, bounded key expiration, explicit key cleanup                                       |
| OpenSearch | Create an isolated index, index a document, search it, delete the index                                |

These checks exercise router behavior and infrastructure clients. They do not
start the Angular application or drive a browser.

## Native live tests

From `backend`, with your own test services running:

```sh
cargo test --locked --test integration -- --ignored
```

The helper forces the test profile and accepts normal environment overrides.
It requires Postgres database `tap_test` and Redis database 1. Create that
database for host tests and set `DATABASE_URL`, `REDIS_URL`, and
`OPENSEARCH_NODE` when endpoints differ from test defaults. The isolated Compose
stack provisions the database itself but exposes no host ports.

Integration tests live under `backend/tests/integration/`; `integration.rs`
registers the modules. Use `support.rs` for configuration and unique resource
names. Keep new tests safe for parallel execution: use transactions or temporary
tables, expiring Redis keys, unique search indexes, and cleanup on failure.
Never flush shared databases or delete shared resources.

## CI

[The workflow](../.github/workflows/ci.yml) runs on pushes, pull requests, and
manual dispatch, with independent jobs:

| Job                 | Checks                                                                      |
| ------------------- | --------------------------------------------------------------------------- |
| Frontend            | Prettier, production build, non-watch unit tests                            |
| Backend             | rustfmt, build, Clippy, regular tests, all Compose configurations           |
| Backend integration | Same isolated Compose suite, logs on failure, cleanup on success or failure |

The integration job sets the Linux OpenSearch `vm.max_map_count` requirement.
Check job output and service logs when troubleshooting a failure.
