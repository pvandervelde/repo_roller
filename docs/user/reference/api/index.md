---
title: "REST API Reference"
description: "Base URL, authentication, request format, response format, and error structure for the RepoRoller REST API."
audience: "all"
type: "reference"
---

# REST API Reference

The RepoRoller REST API provides programmatic access to repository creation and template discovery.

## Base URL

```
/api/v1
```

The full URL depends on your deployment, e.g. `https://reporoller.myorg.example/api/v1`.

## Authentication

All endpoints except `/api/v1/health` require an `Authorization` header:

```
Authorization: Bearer <token>
```

The token must be a GitHub App installation token for the target organisation. The API validates the token against GitHub's API on each request.

## Request format

- Method: as documented per endpoint
- `Content-Type: application/json` (for `POST` requests with a body)
- Field names: **camelCase**
- Encoding: UTF-8

## Response format

All responses return `Content-Type: application/json`.

- Success responses contain the documented payload
- Field names: **camelCase**
- HTTP status codes as documented per endpoint

## Error format

All error responses use a consistent JSON structure:

```json
{
  "error": {
    "code": "MachineReadableErrorCode",
    "message": "Human-readable error description",
    "details": "Additional context (optional, omitted when absent; type varies by error)"
  }
}
```

Common error codes:

| Code | HTTP status | Meaning |
|---|---|---|
| `INVALID_REQUEST` | 400 | Malformed JSON or missing required field |
| `VALIDATION_ERROR` | 400 | Field value fails validation rules |
| `UNAUTHORIZED` | 401 | Invalid, expired, or absent token |
| `RepositoryAlreadyExists` | 409 | A repository with that `organization`/`name` already exists. Enforced authoritatively, immediately before repository creation, for every `creationMode` — independent of what `POST /repositories/validate-name` reported earlier. |
| `ForkTargetNameMismatch` | 400 | `creationMode.type` is `"fork"` and the request's `name` does not match `creationMode.source.name` (GitHub cannot rename a repository on fork) |
| `TEMPLATE_NOT_FOUND` | 404 | Template repository not found or not accessible |
| `CONFIGURATION_ERROR` | 400/500 | Problem loading or parsing configuration |
| `GITHUB_API_ERROR` | 502 | GitHub API returned an unexpected error |
| `MissingPrincipal` | 401 | The token is validly signed but identifies no principal (neither a login nor a principal id). Obtain a new token via `POST /auth/token`. |
| `RateLimitExceeded` | 429 | A rate limit was exceeded: per user on authenticated endpoints, or per client IP on `POST /auth/token`. See [Rate limiting](#rate-limiting). |

<!-- TODO(docs-owner): the error codes in this table use a mix of conventions —
some (`INVALID_REQUEST`, `VALIDATION_ERROR`, `UNAUTHORIZED`, `TEMPLATE_NOT_FOUND`,
`CONFIGURATION_ERROR`, `GITHUB_API_ERROR`) are written in SCREAMING_SNAKE_CASE, but the
actual `code` field emitted by `repo_roller_api::errors` (see `crates/repo_roller_api/src/errors.rs`)
uses PascalCase throughout (e.g. `ValidationError`, `AuthenticationError`,
`RepositoryAlreadyExists`). `RepositoryAlreadyExists` and `ForkTargetNameMismatch` above
have been corrected to match the real wire value as part of task #282; the remaining
SCREAMING_SNAKE_CASE entries in this table predate that task and were not touched here —
flagging so the docs owner can decide whether to correct them for the whole table in a
follow-up, and so client code is not implemented against a code string that does not
actually appear on the wire. -->

## Rate limiting

Two independent fixed-window limiters apply to every request method except `GET`, `HEAD` and
`OPTIONS` (the token endpoint is limited on `POST`):

| Scope | Applies to | Keyed by | Default |
|---|---|---|---|
| `user` | Requests other than `GET`/`HEAD`/`OPTIONS` to authenticated endpoints | The authenticated principal: `login:<lowercased login>`, or `sub:<pid>` for tokens whose login could not be resolved. Tokens with neither receive `401` with code `MissingPrincipal`. | 10 requests per 60 seconds |
| `auth_ip` | The public `POST /api/v1/auth/token` | The client IP address of the connection; IPv6 clients are keyed by their /64 prefix (`X-Forwarded-*` headers are ignored) | 20 requests per 60 seconds |

Each principal has an independent budget; keys are namespaced so they cannot collide with client IP
keys. `GET`, `HEAD` and `OPTIONS` requests, `/health` and `/metrics` are not limited.

For loginless callers (for example GitHub App installation tokens, whose login cannot be
resolved) "per principal" means **per token**: the principal id is a digest of the GitHub token,
so it changes whenever the caller exchanges a different token and gets a fresh budget.

When the limit is exceeded the API returns `429 Too Many Requests` with a `Retry-After` header
(whole seconds until the window resets) and the standard error body:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 42
Content-Type: application/json

{"error":{"code":"RateLimitExceeded","message":"Too many requests. Retry after 42 seconds."}}
```

Clients should wait at least `Retry-After` seconds before retrying. Operators configure the limits
with `RATE_LIMIT_ENABLED` (governs both limiters), `RATE_LIMIT_REQUESTS`, `RATE_LIMIT_WINDOW_SECS`,
`RATE_LIMIT_AUTH_REQUESTS` and `RATE_LIMIT_AUTH_WINDOW_SECS` (see
[Environment variables](../environment-variables.md)). Rejections are logged at WARN
("Rate limit exceeded", without the user identity) and counted by the Prometheus counter
`rate_limit_rejections_total`, labelled `scope` = `user` or `auth_ip`; the gauge
`rate_limit_tracked_keys{scope}` reports how many keys each limiter currently tracks.

Each limiter tracks at most 10,000 keys. If that cap is reached and no expired key can be
reclaimed, the `user` limiter evicts the key with the oldest window (that user gets a fresh budget)
while the `auth_ip` limiter fails closed: requests from new client addresses receive `429` until a
window expires, and clients already tracked keep their budgets.

## Known limitations

- Every method except `GET`, `HEAD` and `OPTIONS` is limited (and counts against one shared per-principal budget); those three methods never consume budget.
- Loginless callers (installation tokens) are limited per token, not per underlying identity.
- The `/auth/token` limiter is shared by every frontend user: the frontend is the only caller, so the backend sees a single IP. Raise `RATE_LIMIT_AUTH_REQUESTS`.
- The window is fixed, not sliding: a client can send up to twice the limit in a short burst that
  straddles a window boundary (the end of one window and the start of the next).
- State is in memory per process, so effective limits scale with the number of replicas and reset
  on restart.
- The `POST /auth/token` limiter keys on the connection address only and ignores `X-Forwarded-For`
  and other `X-Forwarded-*` headers. Behind a reverse proxy or load balancer all clients share the
  proxy's bucket; raise `RATE_LIMIT_AUTH_REQUESTS` accordingly.
- IPv6 clients share one budget per /64 prefix, so hosts in the same /64 are limited together.
- Request headers (including `Authorization` and `Cookie`) are not written to server logs.

## Request tracing

Every request is automatically assigned a unique request ID (UUID v4). The API returns it as
an `X-Request-Id` response header on every response, and the same ID appears in every server
log line produced while that request was handled. When reporting an issue, capture the
`X-Request-Id` header value from the response (or the timestamp, if you don't have the
header) to help correlate the request with the server-side log entries.

```http
HTTP/1.1 201 Created
Content-Type: application/json
X-Request-Id: f47ac10b-58cc-4372-a567-0e02b2c3d479
```

## Logging

By default, the backend emits structured JSON logs to stdout (operators can switch to
human-readable output for local development via the `LOG_FORMAT` environment variable — see
[Environment variables](../environment-variables.md)). All log output — including free-text
log messages — is automatically scanned and redacted before it is written, so tokens,
passwords, and similarly-shaped secrets are never persisted in log storage even if they are
accidentally included in a log call upstream.

## Distributed tracing

When an operator enables it (`OTEL_TRACES_ENABLED=true` — off by default), each call to
`POST /api/v1/repositories` produces one OpenTelemetry trace exported over OTLP, with child
spans for authentication, configuration resolution, template processing, GitHub repository
creation, and every underlying GitHub API call.

This API accepts a standard W3C `traceparent` request header (see the
[Trace Context specification](https://www.w3.org/TR/trace-context/#traceparent-header)). If
present and well-formed, the request's top-level `http_request` span is recorded as a *child*
of the trace identified by that header, rather than starting a new, disconnected trace —
this is how the bundled RepoRoller frontend's browser → SvelteKit-server → backend calls all
appear as one continuous trace instead of three separate ones. The header is entirely
optional: if it is absent, empty, or malformed, the request starts a fresh root trace exactly
as before (fail-safe, not fail-closed) — no caller is required to send it, and no caller is
ever rejected for omitting or mis-forming it. The API does not currently return a trace ID in
the response body or headers — correlate a request with its trace using the `X-Request-Id`
header above (present in server-side span/log output) and a time-range search in your tracing
backend. See
[Deploy with Docker Compose: Distributed Tracing](../../how-to/deploy/deploy-with-docker.md#distributed-tracing-jaeger)
for how to view traces locally — including the full browser → SvelteKit-server → backend
chain — and the
[`repo_roller_api` crate README](../../../../crates/repo_roller_api/README.md#distributed-tracing)
for the full span hierarchy and the sanitization guarantee applied to business-context span
attributes.

## Metrics

`GET /metrics` returns Prometheus-format metrics (request rates, latencies, repository-creation
outcomes, GitHub API call counts, and `rate_limit_rejections_total{scope}` for requests rejected by rate limiting). It is mounted at the server root — **not** under `/api/v1` —
and is unauthenticated, intended for scraping by a Prometheus server on a private network. See
[Deploy with Docker Compose: Metrics (Prometheus)](../../how-to/deploy/deploy-with-docker.md#metrics-prometheus)
for scrape configuration and the operator security note.

## Endpoints

| Resource | Reference |
|---|---|
| Repositories | [repositories.md](repositories.md) |
| Templates and organisation settings | [templates.md](templates.md) |
