# Security model

The gateway is read-only by construction: its registered route list contains only health, readiness, bars, calendar, provider metadata, OpenAPI, and an intentionally unimplemented quote contract. Trading support is feature-gated and is not instantiated by `market_gateway`.

Secrets use secret-aware strings; configuration debug output is redacted. Request signing and authorization happen only inside the SDK transport. Logs and public errors omit signed bodies, upstream bodies, headers, identifiers, keys, and tokens. Credential directories are explicit, are never returned by the API, and Unix files with group/world permission bits are rejected.

Loopback is the default and CORS is disabled. Remote binding requires an explicit opt-in plus a bearer token read from a file and compared without early exit. Put that token file and Tiger credential directory outside the repository with restrictive OS permissions.

`.env`, `.gitignore`, and prompt instructions are not security boundaries. Real broker credentials must be inaccessible to the Codex process using OS permissions, a separate OS user, a container boundary, or a separate machine. Environment variables are supported only by the legacy downloader for compatibility and are not a strong isolation boundary.

The service is not a hardened internet-facing broker proxy. Prefer loopback or an authenticated private tunnel, rotate exposed credentials, and keep filesystem backups from capturing secret directories.
