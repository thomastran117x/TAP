# Configuration

[Documentation index](README.md)

Backend application configuration and Docker Compose inputs are separate.
Application defaults live in YAML; the optional root `.env` supplies Compose
interpolation values and is not read by the Rust process.

## Backend profiles

| Profile | Default use                                       | Postgres                         | Redis                        | OpenSearch               |
| ------- | ------------------------------------------------- | -------------------------------- | ---------------------------- | ------------------------ |
| `dev`   | Host development; default when `APP_ENV` is unset | `localhost:5432/tap`             | `localhost:6379`, database 0 | `http://localhost:9200`  |
| `test`  | Test defaults                                     | `localhost:5432/tap_test`        | `localhost:6379`, database 1 | `http://localhost:9200`  |
| `prod`  | Built Compose backend                             | Required `DATABASE_URL` override | `redis:6379`, database 0     | `http://opensearch:9200` |

The profile files are in [backend/config](../backend/config/). `APP_ENV` accepts
only `dev`, `test`, and `prod`. Configuration loads once at startup and validates
URLs, port numbers, pool sizes, timeouts, logging filters, and CORS origin.
Invalid settings or unknown YAML fields fail startup.

Values load in this order:

1. Select the profile with `APP_ENV`, defaulting to `dev`.
2. Load its embedded YAML, or a complete `<APP_ENV>.yml` from `CONFIG_DIR`.
3. Apply explicitly supplied environment overrides.
4. Validate and pass the typed configuration to the application.

Bundled YAML is embedded into the binary. Rebuild after editing it. `CONFIG_DIR`
replaces the embedded file rather than merging a partial file; the directory
must be readable by the running process. In Docker, mount the directory and set
`CONFIG_DIR` to its path inside the container.

The [backend configuration reference](../backend/README.md#configuration)
contains the complete YAML-to-environment override table. Common overrides are
`DATABASE_URL`, `REDIS_URL`, `OPENSEARCH_NODE`, `CORS_ORIGIN`,
`DEPENDENCY_TIMEOUT_MS`, and `RUST_LOG`.

For a host process, set overrides in the shell that runs `cargo run`. For Docker,
pass them to the backend service through Compose environment configuration.
Shell values do not automatically become container environment variables.

## Compose inputs and addresses

The root [.env.example](../.env.example) lists optional base-stack inputs:

| Input                                 | Default       | Purpose                    |
| ------------------------------------- | ------------- | -------------------------- |
| `POSTGRES_DB`                         | `tap`         | Development database name  |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` | `tap` / `tap` | Local database credentials |
| `FRONTEND_PORT`                       | `4000`        | Published SSR port         |
| `BACKEND_PORT`                        | `3000`        | Published API port         |
| `POSTGRES_PORT`                       | `5432`        | Published Postgres port    |
| `REDIS_PORT`                          | `6379`        | Published Redis port       |
| `OPENSEARCH_PORT`                     | `9200`        | Published OpenSearch port  |

Create a root `.env` only when customizing these inputs, and keep actual secrets
out of version control. Compose provides a database URL matching its configured
credentials and database name to the backend.

Containers use service names and internal ports: `postgres:5432`, `redis:6379`,
and `opensearch:9200`. Host processes use localhost and published ports. Changing
`BACKEND_PORT` changes the host mapping; it does not change the server's internal
port. Changing Postgres initialization variables does not rewrite credentials or
databases already stored in an existing volume.

The development override selects `dev`, changes client URLs to container service
names, and allows `http://localhost:4200`. Its Angular server publishes port 4200.
The standalone test stack selects `test`, provisions `tap_test`, sets service
URLs, and raises the dependency timeout to ten seconds. It does not use the base
stack's credential or port interpolation.

## CORS and deployment settings

`CORS_ORIGIN` must be one HTTP(S) origin, such as `http://localhost:4200`, without
a path or trailing slash. Use the origin the browser actually visits. The host
dev profile defaults to 4200; the base Compose stack allows its published SSR
origin, normally `http://localhost:4000`.

The `prod` profile requires externally supplied `DATABASE_URL`. Supply real
credentials through deployment configuration. For an authenticated OpenSearch
cluster, set both `OPENSEARCH_USERNAME` and `OPENSEARCH_PASSWORD`.
The checked-in Compose services use local defaults and disable OpenSearch
security; the `prod` profile name describes application defaults and does not
make that local service stack ready for a public deployment.
