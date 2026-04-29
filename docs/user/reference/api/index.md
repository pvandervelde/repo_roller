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

Every request is automatically assigned a unique request ID (UUID v4) that appears in server logs. When reporting an issue, include the timestamp of the request to help correlate the log entry.

## Endpoints

| Resource | Reference |
|---|---|
| Repositories | [repositories.md](repositories.md) |
| Templates and organisation settings | [templates.md](templates.md) |
