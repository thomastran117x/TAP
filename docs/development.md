# Development

[Documentation index](README.md)

Run Compose commands from the repository root. Docker-only development needs
Docker and the Compose plugin; application toolchains are supplied by the images.

## Built application stack

```sh
docker compose up --build
```

The frontend SSR host is at `http://localhost:4000` and the API is at
`http://localhost:3000`. Dependencies start first and must become healthy before
the backend starts. The frontend waits for backend readiness.

For API-only work, start the backend and its dependencies:

```sh
docker compose up --build backend
```

## Mounted-source development

```sh
docker compose -f compose.yml -f compose.dev.yml up --build
```

Use `http://localhost:4200` for the Angular development server. The development
override mounts both source trees and caches npm dependencies and Cargo output
in named volumes. Angular watches source changes; polling supports Docker mounts.

Rust does not automatically watch source. After changing Rust or bundled YAML:

```sh
docker compose -f compose.yml -f compose.dev.yml restart backend
```

If you change a frontend dependency, update the lockfile and install into the
mounted dependency volume, then restart the frontend:

```sh
docker compose -f compose.yml -f compose.dev.yml exec frontend npm ci
docker compose -f compose.yml -f compose.dev.yml restart frontend
```

Rebuild after Dockerfile changes. When switching back to built images, recreate
the stack with the base command so development mounts and commands are replaced.

## Host toolchains

Use Rust 1.92 or newer with rustfmt and Clippy, Node 24, and npm 11.16.0.
Native Rust builds require C/C++ build tools for TLS dependencies, including
Clang on Windows ARM64. Docker validation is available if these are missing.

Start just the service containers:

```sh
docker compose up -d --wait postgres redis opensearch
```

From `backend`:

```sh
cargo run --locked
```

From `frontend`, in a separate terminal:

```sh
npm ci
npm start
```

The host API uses `dev.yml` with localhost service URLs. If you changed Compose
ports or credentials, supply matching backend overrides. Stop any Docker backend
already using port 3000 before running a host API on the same port.

To inspect the frontend SSR build locally, from `frontend`:

```sh
npm run build
npm run serve:ssr:frontend
```

The SSR server defaults to port 4000; its `PORT` environment variable overrides
that value. The API's CORS origin must match the browser origin when making API
requests. See [configuration](configuration.md).

## Logs and shutdown

```sh
docker compose ps
docker compose logs --tail 100 -f backend
```

For the mounted-source stack, use the same `-f compose.yml -f compose.dev.yml`
arguments when inspecting or changing its services.

Ctrl+C stops an attached stack. To remove development containers and networks:

```sh
docker compose -f compose.yml -f compose.dev.yml down
```

Named volumes preserve data and build caches. Avoid deleting them to fix routine
startup problems. The [test stack cleanup](testing.md) applies only to disposable
integration services.

Run the [quality checks](testing.md) before submitting changes. Common setup
problems are covered in [troubleshooting](troubleshooting.md).
