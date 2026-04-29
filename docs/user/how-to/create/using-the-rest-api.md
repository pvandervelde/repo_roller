---
title: "Create a repository using the REST API"
description: "Authenticate with the RepoRoller API and create repositories with all three content strategies using curl."
audience: "repository-creator"
type: "how-to"
---

# Create a repository using the REST API

## Authentication

All API requests (except `/health`) require a `Bearer` token in the `Authorization` header:

```
Authorization: Bearer <token>
```

The token is a GitHub App installation token issued by the backend. To obtain one, exchange your GitHub OAuth token via:

```bash
curl -X POST https://reporoller.myorg.example/api/v1/auth/token \
  -H "Authorization: Bearer ${GITHUB_OAUTH_TOKEN}" \
  -H "Content-Type: application/json"
```

The response includes a short-lived JWT (8-hour TTL) which you use as the `Bearer` token for all subsequent requests.

## Create a repository from a template

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service",
    "organization": "myorg",
    "template": "rust-service",
    "description": "Payment processing microservice",
    "visibility": "private",
    "repositoryType": "service",
    "team": "platform",
    "variables": {
      "service_name": "payment-service",
      "service_port": "8080"
    }
  }'
```

**Response** (201 Created):

```json
{
  "repository": {
    "name": "payment-service",
    "fullName": "myorg/payment-service",
    "url": "https://github.com/myorg/payment-service",
    "visibility": "private"
  },
  "appliedConfiguration": {
    "features": { "hasIssues": true, "hasWiki": false },
    "branchProtection": {},
    "labels": []
  }
}
```

## Create an empty repository

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-empty-repo",
    "organization": "myorg",
    "contentStrategy": "empty",
    "description": "Code import repository",
    "visibility": "private"
  }'
```

## Create a repository with README and .gitignore

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-project",
    "organization": "myorg",
    "contentStrategy": "custom_init",
    "initializeReadme": true,
    "initializeGitignore": true,
    "description": "My project",
    "visibility": "private"
  }'
```

## Request fields reference

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Repository name (1–100 chars, lowercase, hyphens, underscores, periods) |
| `organization` | string | Yes | GitHub organisation slug |
| `contentStrategy` | string | No | `"template"` (default), `"empty"`, or `"custom_init"` |
| `template` | string | Conditional | Required when `contentStrategy` is `"template"` |
| `initializeReadme` | boolean | No | Generate a README.md (only with `"custom_init"`) |
| `initializeGitignore` | boolean | No | Generate a .gitignore (only with `"custom_init"`) |
| `description` | string | No | Repository description |
| `visibility` | string | No | `"private"` (default) or `"public"` |
| `repositoryType` | string | No | Repository type for configuration |
| `team` | string | No | Team slug for team-level configuration |
| `variables` | object | No | Template variable key→value pairs (only with `"template"`) |

## Validate the repository name first

Before creation you can check if a name is valid and available:

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories/validate-name \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"organization": "myorg", "name": "payment-service"}'
```

**Response:**

```json
{"valid": true, "available": true, "messages": null}
```

## Error responses

All errors follow the same format:

```json
{
  "error": "Repository already exists",
  "code": "REPOSITORY_EXISTS",
  "details": "A repository named payment-service already exists in myorg"
}
```

| HTTP status | Meaning |
|---|---|
| 400 | Validation error (bad name, missing required field) |
| 401 | Invalid or missing token |
| 404 | Template or organisation not found |
| 409 | Repository already exists |
| 429 | Rate limit from GitHub API |
| 502 | GitHub API error |

## Related reference

- [REST API Reference](../../reference/api/index.md)
- [Repositories API](../../reference/api/repositories.md)
- [Authenticate and call the REST API](../integrate/use-rest-api.md)
