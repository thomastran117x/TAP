# Troubleshooting

[Documentation index](README.md)

Run Compose commands from the repository root, using the same configuration
files as the stack you started.

## API fails to start or readiness returns 503

```sh
docker compose ps
docker compose logs --tail 100 backend postgres redis opensearch
```

Startup requires connections to all three dependencies. Check service health,
database credentials, and client URLs. Container URLs must use service names;
host URLs must use localhost and published ports.

If the API is running, `/ready` reports failing dependencies under
`error.details`. Internal failure messages are intentionally generic; use
server logs for diagnostics. Do not delete data volumes to resolve a connectivity
failure.

## Configuration changes have no effect

Bundled YAML is embedded in the Rust binary. Rebuild built images, or restart
the mounted-source backend to recompile:

```sh
docker compose up -d --build backend
```

For mounted-source development instead:

```sh
docker compose -f compose.yml -f compose.dev.yml restart backend
```

Shell environment variables do not automatically reach containers. Set backend
overrides in the service environment. The Rust process never reads root `.env`.
An existing Postgres volume retains its initialized credentials and databases
even if Compose initialization inputs change.

## Browser rejects API responses

Check that `CORS_ORIGIN` matches the browser origin exactly, including the port,
without a path or trailing slash. Development uses `http://localhost:4200`;
the built SSR frontend normally uses `http://localhost:4000`.
`localhost` and `127.0.0.1` are different origins. CORS affects browser requests;
successful command-line requests alone do not verify browser access.

## A published port is already in use

Check for an existing host server or another stack. Stop the conflicting process,
or change the relevant port input listed in [.env.example](../.env.example).
For host Rust development, update client URLs when dependency ports change.
Changing a published port does not change the internal container port.
The mounted-source frontend runs on 4200; use that URL for the development UI.

## Native Rust build fails in a TLS dependency

Install the required C/C++ toolchain, including Clang on Windows ARM64, or use
the Docker validation stage:

```sh
docker build --target validation -t tap-backend-validation ./backend
```

This runs compilation, formatting, Clippy, and regular tests in the Linux image.

## Integration tests cannot connect

Use the isolated [test workflow](testing.md), which provisions its own services.
For native tests, create `tap_test`, use Redis database 1, and provide reachable
endpoints. The helper rejects other database names; the isolated stack exposes
no ports to host processes.

Inspect failed test services before cleanup:

```sh
docker compose -f compose.test.yml ps --all
docker compose -f compose.test.yml logs --tail 100
docker compose -f compose.test.yml down --volumes --remove-orphans
```

If OpenSearch logs identify insufficient `vm.max_map_count`, configure the Linux
host or Docker VM to at least 262144. CI sets this value on its Linux runner.
Make sure the Docker environment has enough memory for OpenSearch's configured
512 MB Java heap and the other services. Remove the disposable test stack after
success or failure.

## Frontend dependencies or formatting fail

Use Node 24 and the npm version declared in `frontend/package.json`. From
`frontend`, `npm ci` installs the lockfile's dependency versions, and
`npm run format` fixes Prettier differences. For mounted-source development,
install into its dependency volume as described in [development](development.md).
Do not edit generated output or installed packages.
