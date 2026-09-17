# Repository guidance

This file applies to the entire TAP repository. Read any more specific guidance
in the directory you are changing. Keep this document and the relevant README
current when architecture, configuration, or development commands change.

## Project map

TAP has a Rust API and an Angular frontend with server-side rendering. Docker
Compose supplies Postgres, Redis, and OpenSearch.

```text
backend/
  Cargo.toml, Cargo.lock   Rust dependencies; Rust 1.92+, edition 2024
  config/                 dev.yml, test.yml, prod.yml application defaults
  src/
    main.rs               Thin executable entry point
    lib.rs                Library entry point for the server and tests
    app/                  Configuration, router assembly, state, server lifecycle
    infra/                Postgres, Redis, OpenSearch clients and logging setup
    middleware/           Shared HTTP layers: CORS and request tracing
    features/             Capability-based modules; health is the first feature
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
- Return intentional error responses and attach context to internal errors.
  Use `tracing` for diagnostics; avoid exposing credentials or internal errors in
  HTTP responses or logs. Avoid `unwrap`/`expect` in recoverable request paths.
- Make database schema changes through versioned migrations and document their
  application steps. Do not assume a migration runner already exists.

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
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

When the local native toolchain is unavailable, from the repository root:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

Native TLS dependencies require C/C++ build tools, including Clang on Windows
ARM64. Use the Docker validation stage when these are unavailable.

Live service checks are opt-in, from `backend` with matching configuration:

```sh
cargo test --locked --test health -- --ignored
```

`APP_ENV=test` uses the `tap_test` database and Redis database 1. Create the test
database before running live tests with that profile. OpenSearch is shared; use
isolated indexes if future tests write search data. `/health` reports liveness;
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
Clippy with warnings treated as errors; CI tests do not start external services
or run the ignored live health test. The backend job also validates both Compose
configurations. Keep the CI Rust version aligned with the backend Dockerfile.

For Compose edits, from the repository root:

```sh
docker compose config --quiet
docker compose -f compose.yml -f compose.dev.yml config --quiet
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
