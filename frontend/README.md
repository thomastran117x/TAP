# TAP frontend

Angular 22 application with standalone components, Tailwind styling, and
server-side rendering through Express. Use Node 24 and npm 11.16.0 to match CI.
The npm version is declared in [package.json](package.json).

See the [repository README](../README.md) for the full stack and the
[documentation index](../docs/README.md) for architecture, configuration, and
testing guides.

## Host development

Run from `frontend`:

```sh
npm ci
npm start
```

Visit `http://localhost:4200`. Angular reloads source changes. Run the backend
separately using the [development guide](../docs/development.md); its default
host dev profile allows this frontend origin.

## Docker development

From the repository root:

```sh
docker compose -f compose.yml -f compose.dev.yml up --build
```

This starts the Angular development server at `http://localhost:4200` alongside
the Rust API and dependencies. Source is mounted; npm packages are stored in a
named volume. See the development guide for dependency updates and shutdown.

The base command `docker compose up --build` instead runs the built SSR frontend
at `http://localhost:4000`.

## Structure

| Path                 | Purpose                                                  |
| -------------------- | -------------------------------------------------------- |
| `src/app/`           | Standalone components, routes, and application providers |
| `src/main.ts`        | Browser bootstrap                                        |
| `src/main.server.ts` | Server bootstrap                                         |
| `src/server.ts`      | Express host for assets and Angular SSR                  |
| `src/styles.css`     | Global styles and Tailwind entry point                   |
| `public/`            | Static assets                                            |
| `angular.json`       | Build, development server, and test targets              |

Keep components focused on presentation, put API calls in typed services, and
guard browser-only APIs so SSR works. Application HTTP endpoints belong in Axum.

## Production build and SSR

From `frontend`:

```sh
npm run build
npm run serve:ssr:frontend
```

The build writes browser and server artifacts under `dist/frontend/`. The SSR
host serves assets and renders the application on port 4000 by default; `PORT`
overrides that value. Configure backend `CORS_ORIGIN` for the browser origin when
making API requests from the SSR frontend.

## Formatting and tests

```sh
npm run format:check
npm run build
npm test -- --watch=false
```

Use `npm run format` to apply Prettier fixes. `.prettierignore` excludes generated
output, installed dependencies, and the npm lockfile. Unit tests use the configured
Angular/Vitest runner. A browser end-to-end test target has not been configured.

For scaffolding, use the installed CLI through npm:

```sh
npm run ng -- generate component component-name
```

See [testing](../docs/testing.md) for CI checks and [contributing](../CONTRIBUTING.md)
for review expectations.
