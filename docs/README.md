# Documentation

Start with the [repository README](../README.md) to run TAP locally. Use these
guides for the next step:

| Guide                                 | Covers                                                      |
| ------------------------------------- | ----------------------------------------------------------- |
| [Architecture](architecture.md)       | Module boundaries, startup, request flow, adding features   |
| [Development](development.md)         | Docker and host workflows, rebuilds, logs, shutdown         |
| [Configuration](configuration.md)     | YAML profiles, overrides, Compose inputs, service addresses |
| [Testing](testing.md)                 | Local checks, integration isolation, CI jobs                |
| [HTTP API](api.md)                    | Health/readiness endpoints and JSON errors                  |
| [Troubleshooting](troubleshooting.md) | Startup, ports, CORS, native builds, test failures          |

Component references live in the [backend README](../backend/README.md) and
[frontend README](../frontend/README.md). The backend reference contains the
complete configuration override table and source layout.

See [contributing](../CONTRIBUTING.md) for change and review expectations, and
[AGENTS.md](../AGENTS.md) for shared coding conventions.

These documents describe the checked-in implementation. Update the relevant
guide when changing a command, setting, endpoint, or architectural boundary.

The [OpenAPI contract](../backend/openapi.yaml) is served at `/openapi.yaml` by
the backend. See the [HTTP API guide](api.md#openapi-contract) for usage and
maintenance.
