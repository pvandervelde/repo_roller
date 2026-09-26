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
  "error": "Human-readable error description",
  "code": "MACHINE_READABLE_ERROR_CODE",
  "details": "Additional context (optional)"
}
```

Common error codes:

| Code | HTTP status | Meaning |
|---|---|---|
| `INVALID_REQUEST` | 400 | Malformed JSON or missing required field |
| `VALIDATION_ERROR` | 400 | Field value fails validation rules |
| `UNAUTHORIZED` | 401 | Invalid, expired, or absent token |
| `REPOSITORY_ALREADY_EXISTS` | 409 | A repository with that name already exists in the org |
| `TEMPLATE_NOT_FOUND` | 404 | Template repository not found or not accessible |
| `CONFIGURATION_ERROR` | 400/500 | Problem loading or parsing configuration |
| `GITHUB_API_ERROR` | 502 | GitHub API returned an unexpected error |

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
creation, and every underlying GitHub API call. This API does not currently accept or return
a trace ID in the request/response body or headers — correlate a request with its trace using
the `X-Request-Id` header above (present in server-side span/log output) and a time-range
search in your tracing backend. See
[Deploy with Docker Compose: Distributed Tracing](../../how-to/deploy/deploy-with-docker.md#distributed-tracing-jaeger)
for how to view traces locally, and the
[`repo_roller_api` crate README](../../../../crates/repo_roller_api/README.md#distributed-tracing)
for the full span hierarchy and the sanitization guarantee applied to business-context span
attributes.

## Metrics

`GET /metrics` returns Prometheus-format metrics (request rates, latencies, repository-creation
outcomes, GitHub API call counts). It is mounted at the server root — **not** under `/api/v1` —
and is unauthenticated, intended for scraping by a Prometheus server on a private network. See
[Deploy with Docker Compose: Metrics (Prometheus)](../../how-to/deploy/deploy-with-docker.md#metrics-prometheus)
for scrape configuration and the operator security note.

## Endpoints

| Resource | Reference |
|---|---|
| Repositories | [repositories.md](repositories.md) |
| Templates and organisation settings | [templates.md](templates.md) |
