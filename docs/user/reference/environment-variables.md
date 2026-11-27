---
title: "Environment variables"
description: "All environment variables consumed by the RepoRoller backend API and frontend."
audience: "operator"
type: "reference"
---

# Environment variables

## Backend (`repo_roller_api`)

> **Warning: the `/auth/token` rate-limit budget is shared by all users.** The frontend is the only
> caller of `POST /api/v1/auth/token`, so the backend sees a single client IP for every user and
> they all draw on one `RATE_LIMIT_AUTH_REQUESTS` budget (default 20 per 60 seconds). Raise
> `RATE_LIMIT_AUTH_REQUESTS` to match your expected sign-in rate, or sign-ins will start returning
> `429`. A reverse proxy has the same effect: its address becomes the single key.

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_APP_ID` | Yes | — | Numeric App ID from the GitHub App settings page |
| `GITHUB_APP_PRIVATE_KEY` | Yes | — | PEM private key with literal newlines replaced by `\n`. Equivalent to a password — store in a secrets manager in production. |
| `JWT_SECRET` | Yes | — | HS256 signing key for backend-issued JWTs. Minimum 32 characters. |
| `METADATA_REPOSITORY_NAME` | No | `.reporoller` | Name of the configuration repository inside the GitHub organisation |
| `API_HOST` | No | `0.0.0.0` | Network interface to bind to |
| `API_PORT` | No | `8080` | Port to listen on |
| `RUST_LOG` | No | `info` | Log level filter: `error`, `warn`, `info`, `debug`, `trace`. Supports per-module filters (e.g. `repo_roller_core=debug,info`). |
| `LOG_FORMAT` | No | `json` | Log output format. `json` (default) emits machine-readable structured JSON lines with sensitive fields (tokens, passwords, secrets, etc.) automatically redacted; set to `pretty` for human-readable output during local development. |
| `OTEL_TRACES_ENABLED` | No | `false` | Enables OpenTelemetry span export over OTLP. Fail-safe-off: only the case-insensitive literal values `true` or `1` enable export; any other value (unset, empty, `false`, `0`, garbage) leaves tracing disabled with no background exporter and no network I/O. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | No | `http://localhost:4317` | OTLP/gRPC collector endpoint spans are exported to when tracing is enabled. Use an `https://` URL to export over TLS (supported via the `tls-aws-lc` exporter feature). Ignored when `OTEL_TRACES_ENABLED` is not `true`/`1`. |
| `OTEL_SERVICE_NAME` | No | `repo-roller-api` | Service name attached to every exported span's resource attributes. Set a distinct value per environment (e.g. `repo-roller-api-staging`) when multiple deployments share one tracing backend. |
| `RATE_LIMIT_ENABLED` | No | `true` | Enables rate limiting of every method except `GET`/`HEAD`/`OPTIONS` on authenticated routes, plus the `POST /auth/token` per-IP limiter: both the per-user limiter and the `POST /auth/token` per-IP limiter. Set to `false` or `0` to disable both. |
| `RATE_LIMIT_REQUESTS` | No | `10` | Maximum requests (all methods except `GET`/`HEAD`/`OPTIONS`) each authenticated user may make per window. Invalid values fall back to the default. |
| `RATE_LIMIT_WINDOW_SECS` | No | `60` | Length of the per-user rate-limit window in seconds. Invalid values fall back to the default. |
| `RATE_LIMIT_AUTH_REQUESTS` | No | `20` | Shared by all frontend users (single caller IP): raise it. Maximum `POST /api/v1/auth/token` requests each client IP may make per window. Invalid values fall back to the default. |
| `RATE_LIMIT_AUTH_WINDOW_SECS` | No | `60` | Length of the token-endpoint rate-limit window in seconds. Invalid values fall back to the default. |

Rate limiting covers every method except `GET`, `HEAD` and `OPTIONS`. For loginless (installation
token) callers the limit is per token, because the principal id is a digest of the token.
Rate-limit state is held in memory per process. With multiple replicas each replica enforces the
limit independently (the effective limit is roughly `RATE_LIMIT_REQUESTS` times the replica count),
and counters reset when a replica restarts. Each limiter tracks at most 10,000 keys: the per-user
limiter evicts the oldest key at the cap, while the token-endpoint limiter fails closed (new client
addresses get `429` until a window expires). The token-endpoint limiter keys on the connection
address (IPv6 by /64 prefix) and ignores `X-Forwarded-*` headers, so behind a reverse proxy all clients share the proxy
IP's bucket: raise `RATE_LIMIT_AUTH_REQUESTS` accordingly. See
[Rate limiting](../reference/api/index.md#rate-limiting).

### Secret resolver variables

When using outbound notification webhooks, additional variables provide the signing secrets. The variable name is whatever you put in the `secret` field of `notifications.toml`:

```toml
# Example: secret = "REPOROLLER_WEBHOOK_SECRET"
# Then set:
REPOROLLER_WEBHOOK_SECRET=your-actual-secret-value
```

---

## Frontend (SvelteKit)

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_CLIENT_ID` | Yes | — | OAuth App client ID |
| `GITHUB_CLIENT_SECRET` | Yes | — | OAuth App client secret |
| `GITHUB_ORG` | Yes | — | GitHub organisation slug (e.g. `acme-corp`) |
| `ORIGIN` | Yes | — | Public URL of the frontend (e.g. `https://reporoller.acme.example`). Must exactly match the OAuth App callback URL base. Required by SvelteKit for CSRF protection. |
| `SESSION_SECRET` | Yes | — | HMAC-SHA256 key for signing session cookies. Minimum 32 characters. |
| `BACKEND_API_URL` | Yes | — | Base URL of the backend container, e.g. `http://backend:8080`. Must be reachable from the frontend container. Not exposed to browsers. |
| `NODE_ENV` | No | `production` | Set to `production` for container deployments |

### Observability variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `OTEL_TRACES_ENABLED` | No | `false` | Enables OpenTelemetry span export over OTLP from the SvelteKit **server** process (Node-side spans: page-load/form-submission handling, the outbound proxy call to the backend). Fail-safe-off — same matching rule as the backend variable of the same name: only the case-insensitive literal values `true` or `1` enable export. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | No | `http://localhost:4317` | OTLP/gRPC collector endpoint the SvelteKit **server** process exports spans to. Ignored when `OTEL_TRACES_ENABLED` is not `true`/`1`. Not reachable from browsers — use a Docker-internal address (e.g. `http://jaeger:4317`) when the collector runs as a sibling container. |
| `OTEL_SERVICE_NAME` | No | `repo-roller-frontend` | Service name attached to every span exported by the SvelteKit server process. |
| `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT` | No | *(unset)* | OTLP/**HTTP** collector endpoint the **browser** exports spans to directly (e.g. `http://localhost:4318/v1/traces`). Must be a publicly-routable URL reachable from the end user's browser, not the Docker-internal address used by `OTEL_EXPORTER_OTLP_ENDPOINT` above — gRPC is not usable from a browser, so this must point at an OTLP/HTTP receiver. Must be `PUBLIC_`-prefixed for SvelteKit to expose it to client bundles. Fail-safe-off: when unset or empty, the browser never attempts network export (spans go to the console only). There is no separate browser-side "enabled" flag — setting this variable is itself what turns browser export on. |

### Branding variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `BRAND_APP_NAME` | No | `RepoRoller` | Application name shown in the page title and header |
| `BRAND_LOGO_URL` | No | *(none)* | URL for the light-mode logo image |
| `BRAND_LOGO_URL_DARK` | No | *(none)* | URL for the dark-mode logo. Has no effect unless `BRAND_LOGO_URL` is also set. |
| `BRAND_LOGO_ALT` | No | `<name> logo` | Alt text for the logo image |
| `BRAND_PRIMARY_COLOR` | No | `#0969da` | CSS accent colour (any valid CSS colour value: hex, rgb, hsl) |
| `BRAND_PRIMARY_COLOR_DARK` | No | *(none)* | Accent colour override for dark mode |

---

## Generating secrets

Generate random secret values with:

```bash
# Node.js
node -e "console.log(require('crypto').randomBytes(48).toString('hex'))"

# openssl
openssl rand -hex 48
```

Run twice to produce two independent values for `SESSION_SECRET` and `JWT_SECRET`.

---

## Converting the GitHub App private key

The downloaded `.pem` file contains literal newlines. To store it as a single-line environment variable:

```bash
# macOS / Linux
awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' your-app.private-key.pem

# PowerShell (Windows)
(Get-Content your-app.private-key.pem -Raw) -replace "`r`n","`n" -replace "`n","\\n"
```

Use the resulting single-line string as the value of `GITHUB_APP_PRIVATE_KEY`.
