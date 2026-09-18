# Contributing

Read the [README](README.md), [architecture guide](docs/architecture.md), and
[AGENTS.md](AGENTS.md) before changing the project. Shared conventions apply to
both human contributors and coding agents.

## Make a focused change

Inspect the working tree and preserve unrelated work. Follow existing patterns
and dependencies. Keep feature logic within its feature, shared client setup in
infrastructure, and shared HTTP policies in middleware. Follow the Rust ownership,
error, async, and persistence practices in `AGENTS.md`.

When adding configuration, update the typed schema, relevant YAML profiles,
environment override mapping, validation, tests, and reference docs together.
When changing HTTP behavior, update [backend/openapi.yaml](backend/openapi.yaml),
the [API reference](docs/api.md), and relevant contract tests. Run
[OpenAPI validation](docs/testing.md#openapi-validation) and keep examples aligned
with actual responses. Include versioned migrations and application instructions when
introducing schema changes; a migration runner is not currently provided.

Keep browser-only code safe for SSR. Use typed Angular services for API access
and preserve existing compiler and build settings.

## Verify the affected behavior

Run the applicable [formatting, build, quality, and test checks](docs/testing.md).
Fix warnings rather than weakening CI. Add tests for meaningful contracts and
failure cases. Keep regular tests independent of live services and use the
isolated integration stack for persistence checks.

Check Compose configuration after changing any Compose file. Before submitting:

```sh
git diff --check
```

Documentation-only changes need link, command, and formatting review rather than
application rebuilds. Keep READMEs and guides current with the implementation.

## Describe the change for review

Explain the concrete problem and resulting behavior. Include relevant validation,
any changed response or configuration contracts, and remaining limitations.
Keep the description focused on the final implementation. Do not commit secrets,
dependency directories, or generated build output.
