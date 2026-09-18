# Repository guidance

This file applies to the entire TAP repository. Read any more specific guidance
in the directory you are changing. Keep this document and the relevant README
current when architecture, configuration, or development commands change.

## Project map

TAP has a Rust API and an Angular frontend with server-side rendering. Docker
Compose supplies Postgres, Redis, and OpenSearch.

Start with [README.md](README.md) for onboarding and [docs/README.md](docs/README.md)
for architecture, development, configuration, testing, API, and troubleshooting
guides. [CONTRIBUTING.md](CONTRIBUTING.md) describes change and review expectations.
Keep these documents consistent with the implementation; shared coding rules
remain in this file.

```text
backend/
  Cargo.toml, Cargo.lock   Rust dependencies; Rust 1.92+, edition 2024
  config/                 dev.yml, test.yml, prod.yml application defaults
  openapi.yaml            Checked-in OpenAPI 3.1.1 contract served by the API
  redocly.yaml            Specification and example validation rules
  src/
    main.rs               Thin executable entry point
    lib.rs                Library entry point for the server and tests
    app/                  Configuration, router assembly, state, server lifecycle
    infra/                Postgres, Redis, OpenSearch clients and logging setup
    middleware/           Shared HTTP layers: errors, CORS, and request tracing
    features/             Capability modules: health and OpenAPI documentation
  tests/                  Public API checks and live integration tests
    support/              Shared test helpers
  Dockerfile              Development, build, validation, production stages
  README.md               Detailed backend setup and configuration reference
frontend/
  src/app/                Angular components, routes, application providers
  src/server.ts           Angular SSR host
  src/styles.css          Global styles and Tailwind entry point
  public/                 Static assets
  angular.json            Angular build, development, and test configuration
  package.json            npm scripts and package manager version
  Dockerfile              Node 24 development/build and SSR runtime
compose.yml               Base stack; backend uses the prod profile
compose.dev.yml           Development override; backend uses the dev profile
compose.test.yml          Standalone isolated integration stack and test runner
.env.example              Optional Compose credentials and published port settings
.github/workflows/ci.yml   Frontend and backend formatting, build, quality, tests
```

## Backend boundaries

- Keep `main.rs` thin. Assemble routes, clients, configuration, and middleware in
  `app`; manage startup, graceful shutdown, and cleanup there.
- Put external client setup and shared connectivity operations in `infra`.
  Infrastructure must not depend on feature handlers or application assembly.
- Put shared HTTP policies in `middleware`; keep them independent of individual
  features and external clients.
- Organize capabilities under `features/<name>/`. A feature owns its routes,
  request/response models, services, and persistence queries as needed. Expose a
  route-registration function in its `mod.rs` and merge it in `app/router.rs`.
- Keep handlers focused on HTTP extraction, validation, responses, and status
  codes. Delegate application logic to services. Keep feature implementation
  modules private unless another module needs a deliberate public API.
- Reuse the SQLx pool, Redis connection manager, and OpenSearch client in
  `AppState`. Avoid opening new clients for every request.
- Use asynchronous I/O on Tokio. Move blocking work off the runtime threads.
  Bind SQL parameters rather than interpolating user input into queries.
- Return `app::HttpResult<T>` from fallible handlers and use `app::HttpError`
  for the shared `{ "error": { "code", "message", "details"? } }` contract.
  Map expected domain failures to appropriate statuses within the feature;
  propagate unexpected `anyhow` errors with `?` or `HttpError::internal`.
  Explicit messages and details are public: never put raw dependency failures or
  sensitive input in them. The middleware normalizes framework rejections and
  unformatted error responses; use typed errors to retain intentional messages.
  Attach context to internal errors.
  Use `tracing` for diagnostics; avoid exposing credentials or internal errors in
  HTTP responses or logs. Avoid `unwrap`/`expect` in recoverable request paths.
- Make database schema changes through versioned migrations and document their
  application steps. Do not assume a migration runner already exists.
- Keep `backend/openapi.yaml` aligned with routes, HTTP models, statuses, and
  public error codes. This is a manually maintained contract embedded in the
  binary and served at `GET /openapi.yaml`. Update operation IDs, schemas, and
  examples when changing API behavior; do not document unimplemented features.
  Validate it with the pinned Redocly command in `docs/testing.md` and preserve
  integration checks that compare response examples with actual API responses.

## Rust coding practices

Apply these practices to backend changes, together with the architecture
boundaries above. Favor straightforward, idiomatic Rust that fits the existing
code over introducing a new framework or abstraction.

### Types, ownership, and APIs

- Keep code and dependencies compatible with the declared Rust 1.92 minimum and
  edition 2024. Update the manifest, Dockerfile, and CI together when intentionally
  changing the supported toolchain.
- Use `snake_case` for functions and modules, `UpperCamelCase` for types and
  traits, and `SCREAMING_SNAKE_CASE` for constants. Let rustfmt handle formatting.
- Model meaningful states with structs, enums, and validated newtypes. Use
  `Option` for absence and `Result` for failure; avoid sentinel values or boolean
  flags that permit contradictory states.
- Prefer borrowed inputs such as `&str` and slices when ownership is not needed.
  Move values when ownership transfers. Clone deliberately; cheap clones of
  shared client handles are appropriate, but avoid copying payloads merely to
  work around ownership errors.
- Prefer the narrowest useful visibility: private, `pub(super)`, or `pub(crate)`
  before `pub`. Document public APIs, including important invariants and errors.
  Use `From`/`TryFrom` for meaningful conversions and checked conversions where
  casts could truncate or overflow.
- Use traits and generics when there is an actual shared contract. Avoid creating
  a trait for every service, unnecessary trait objects, or macros for simple code.
- Prefer safe Rust. If unsafe code is necessary, isolate it, explain each safety
  invariant in `SAFETY` comments, and test the affected behavior. Do not introduce
  unsafe code for speculative performance improvements.

### Errors and HTTP contracts

- Propagate recoverable failures with `?`; add actionable context with `anyhow`
  at infrastructure and application boundaries. Use typed errors when callers
  need to distinguish domain failures, and map them to HTTP responses centrally
  within the feature or a shared error module.
- Validate untrusted inputs at the boundary. Separate transport models from
  persistence or domain models when their contracts differ. Return appropriate
  status codes with a consistent error shape.
- Avoid panics in request handling. `unwrap`/`expect` are acceptable in tests or
  for an explicitly documented invariant, not for ordinary invalid input or
  unavailable dependencies.
- Handle results intentionally. Do not discard errors or replace failures with
  successful defaults unless that behavior is part of the contract. If failure
  is intentionally tolerated, make the reason and necessary diagnostics clear.

### Async work and persistence

- Keep blocking filesystem calls, synchronous network calls, and substantial
  CPU work out of async request paths. Use `spawn_blocking` or an appropriate
  worker for blocking work, and bound concurrency rather than spawning unlimited
  tasks.
- Keep lock scopes short and release guards before awaiting external I/O. Use
  Tokio synchronization when asynchronous waits are needed. Do not wrap already
  shareable database pools or clients in redundant locks.
- Give external operations configurable deadlines. Retry only suitable failures
  with bounded attempts and backoff; account for whether writes are idempotent.
  A timeout does not prove a remote write was cancelled.
- Keep spawned tasks owned by the application lifecycle. Observe task failures,
  respect shutdown and cancellation, and avoid accidental detached work.
- Use transactions for database writes that must succeed atomically. Keep SQL
  queries near the feature that owns them, bind input parameters, and explicitly
  choose cache expiration/invalidation and search consistency behavior.
- Bound request sizes, collection sizes, and pagination when adding endpoints
  that accept or return variable amounts of data. Avoid unbounded query results
  or allocations driven by user input.

### Tests and quality checks

- Test contracts and meaningful failure cases, including validation, dependency
  failures, and transaction behavior when relevant. Keep private unit tests near
  their implementation and public API tests under `backend/tests/`.
- Keep ordinary tests deterministic and independent of live services. Avoid
  timing-based sleeps or global environment mutation; use explicit inputs and
  controlled async coordination. Keep tests that require services opt-in.
- Run the Rust checks listed below for Rust changes. Fix compiler and Clippy
  warnings rather than weakening CI. A narrowly scoped lint allowance must
  explain a concrete reason; do not add broad warning suppressions.
- Do not upgrade dependencies, add new crates, or introduce performance
  optimizations incidentally. Measure a real bottleneck before trading clarity
  for a more complex implementation.

## Configuration

- Load and validate configuration once through `app::Config`. Pass settings to
  infrastructure and services; avoid scattered environment reads in handlers.
- Put application defaults in `backend/config/dev.yml`, `test.yml`, and
  `prod.yml`. `APP_ENV` selects the profile and defaults to `dev`.
- Environment variables override YAML. The backend does not load `.env` files;
  the root `.env` is optional Docker Compose input only.
- Bundled profiles are embedded in the binary. Rebuild after editing them.
  `CONFIG_DIR` loads a complete `<APP_ENV>.yml` file at runtime instead.
- Production requires an externally supplied `DATABASE_URL`. Supply actual
  credentials through deployment configuration, not committed defaults.
- When adding a setting, update the typed schema, applicable profiles,
  validation, override mapping, tests, and the backend README together. Unknown
  YAML fields and invalid configuration must fail startup clearly.
- Preserve the distinction between host URLs (`localhost` with published ports)
  and container URLs (`postgres`, `redis`, `opensearch` with internal ports).

## Frontend practices

- Follow the existing standalone Angular component and provider patterns.
  Group new UI capabilities by feature as the application grows; share code only
  when there is an actual shared use case.
- Keep components focused on presentation and interaction. Put API access and
  reusable application logic in injectable services with typed contracts.
- Use signals for component state and RxJS where streams are appropriate.
  Manage subscription lifetimes using Angular lifecycle utilities.
- Keep browser-only APIs out of server rendering paths. Access `window`,
  `document`, and browser storage only in a browser-safe context.
- Keep the application API in Axum. `frontend/src/server.ts` hosts Angular SSR.
- Follow the existing TypeScript and Angular compiler settings, the `@app/*`
  alias, `.prettierrc`, and Tailwind/global-style setup. Avoid weakening compiler
  checks to accommodate a change.
- Use semantic markup, keyboard-accessible controls, and labels for inputs.

## Development and checks

Run commands in the directory indicated. Run checks relevant to the changed
layer; documentation-only changes do not require rebuilding the application.

From the repository root, start the API and its dependencies:

```sh
docker compose up --build backend
```

For development, with source mounts and Cargo caches:

```sh
docker compose -f compose.yml -f compose.dev.yml up --build backend
docker compose -f compose.yml -f compose.dev.yml restart backend
```

Restart the development backend after Rust or bundled YAML changes to recompile.
Add `frontend` to the `up` command to run both applications. Default backend port
is 3000; frontend SSR port is 4000 and frontend development port is 4200.

From `backend`, check Rust changes:

```sh
cargo fmt --check
cargo build --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

When the local native toolchain is unavailable, from the repository root:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

Native TLS dependencies require C/C++ build tools, including Clang on Windows
ARM64. Use the Docker validation stage when these are unavailable.

Run live integration tests from the repository root:

```sh
docker compose -f compose.test.yml up --build --abort-on-container-exit --exit-code-from integration integration
docker compose -f compose.test.yml down --volumes --remove-orphans
```

This uses the separate `tap-tests` project with no published ports or persistent
named volumes. Cleanup removes only that test stack; run it after success or
failure. For concurrent runs, use a unique `-p` name on both commands.
Live tests live in `backend/tests/integration/`, registered by `integration.rs`.
From `backend`, with your own test services configured, use
`cargo test --locked --test integration -- --ignored`.
Helpers force test defaults and require `tap_test` and Redis database 1.
Use transactions/temporary tables, expiring unique keys, and unique search indexes;
clean up test resources on failure. Never flush databases or delete shared indexes.
The Compose stack creates the test database automatically. `/health` reports liveness;
`/ready` checks all three services and returns 503 when any is unavailable.

From `frontend`, use the package manager declared in `package.json`:

```sh
npm ci
npm run format:check
npm run build
npm test -- --watch=false
```

Use `npm run format` to fix frontend formatting. `.prettierignore` excludes
generated output and the npm lockfile; `.gitattributes` keeps text line endings
consistent across platforms. CI uses Node 24 and the npm version declared in
`frontend/package.json`; keep the workflow's npm version in sync with that field.

`.github/workflows/ci.yml` runs on pushes, pull requests, and manual dispatch.
Frontend and backend checks run independently. Backend quality checks include
Clippy with warnings treated as errors; ordinary backend tests skip live checks.
A separate integration job runs `compose.test.yml` with real services and always
cleans up its stack. The backend job validates all three Compose configurations.
Keep the CI Rust version aligned with the backend Dockerfile.

The OpenAPI job validates the spec, references, and response examples using
Redocly CLI 2.45.0 on Node 24. From the repository root:

```sh
npx --yes @redocly/cli@2.45.0 lint --config backend/redocly.yaml backend/openapi.yaml
```

Keep the validator version aligned between CI and documentation.

For Compose edits, from the repository root:

```sh
docker compose config --quiet
docker compose -f compose.yml -f compose.dev.yml config --quiet
docker compose -f compose.test.yml config --quiet
```

## Change discipline

- Inspect the working tree before editing and preserve unrelated user changes.
  Keep changes focused on the requested outcome; avoid incidental rewrites.
- Prefer existing dependencies and patterns. Add abstractions when they simplify
  real code, and keep lockfiles consistent with dependency changes.
- Test observable behavior and meaningful failure cases. Ordinary tests should
  not require external services or mutate process environment. Use explicit
  configuration lookup sources for deterministic configuration tests.
- Keep live tests opt-in and isolated from other applications' data. Do not
  delete Compose volumes to resolve routine development problems.
- Do not edit generated build output, dependency directories, or Cargo targets.
- Before finishing, inspect the diff and run `git diff --check`. Report what
  changed, which checks passed, and any checks blocked by the environment.
