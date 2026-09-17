# Claude repository guidance

Read [AGENTS.md](AGENTS.md) before making changes. It is the shared source for
project structure, architecture boundaries, configuration, development commands,
testing, and repository practices. Follow any more specific instructions in the
directory you are changing.

Useful references:

- [Backend setup and configuration](backend/README.md)
- [Backend defaults](backend/config/)
- [Frontend scripts](frontend/package.json)
- [Frontend build configuration](frontend/angular.json)
- [Base Compose stack](compose.yml)
- [Development Compose override](compose.dev.yml)

Inspect the current code and working tree before choosing an approach. Complete
the requested change within its scope, preserve unrelated work, and verify the
affected behavior using the checks in `AGENTS.md`. Summarize the result and any
validation limitations clearly.

Update shared conventions in `AGENTS.md` instead of duplicating them here.
