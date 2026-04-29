---
title: "Repositories API"
description: "API reference for creating and validating repositories via the REST API."
audience: "all"
type: "reference"
---

# Repositories API

## `POST /api/v1/repositories`

Creates a new GitHub repository.

### Request body

```json
{
  "name": "payment-service",
  "organization": "myorg",
  "contentStrategy": "template",
  "template": "rust-service",
  "description": "Payment processing microservice",
  "visibility": "private",
  "repositoryType": "service",
  "team": "payments-team",
  "variables": {
    "service_name": "payment-service",
    "service_port": "8080"
  }
}
```

### Request fields

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | string | Yes | — | Repository name. 1–100 characters; lowercase letters, numbers, hyphens, underscores, periods; cannot start with `.` or `-`. |
| `organization` | string | Yes | — | GitHub organisation slug |
| `contentStrategy` | string | No | `"template"` | How to populate the repository: `"template"`, `"empty"`, or `"custom_init"` |
| `template` | string | Conditional | — | Template repository name. Required when `contentStrategy` is `"template"`. |
| `initializeReadme` | boolean | No | `false` | Generate a `README.md`. Only valid with `contentStrategy: "custom_init"`. |
| `initializeGitignore` | boolean | No | `false` | Generate a `.gitignore`. Only valid with `contentStrategy: "custom_init"`. |
| `description` | string | No | — | Repository description shown on GitHub |
| `visibility` | string | No | `"private"` | `"private"` or `"public"`. Subject to organisation policy. |
| `repositoryType` | string | No | — | Repository type slug for type-level configuration |
| `team` | string | No | — | Team slug for team-level configuration |
| `variables` | object | No | — | Key-value pairs of template variable values. Only valid with `contentStrategy: "template"`. |

### Content strategy examples

**Template:**

```json
{
  "name": "my-service",
  "organization": "myorg",
  "template": "rust-service"
}
```

**Empty:**

```json
{
  "name": "legacy-import",
  "organization": "myorg",
  "contentStrategy": "empty"
}
```

**Custom init:**

```json
{
  "name": "scripts",
  "organization": "myorg",
  "contentStrategy": "custom_init",
  "initializeReadme": true,
  "initializeGitignore": true
}
```

### Response — 201 Created

```json
{
  "repository": {
    "name": "payment-service",
    "fullName": "myorg/payment-service",
    "url": "https://github.com/myorg/payment-service",
    "visibility": "private"
  },
  "appliedConfiguration": {
    "features": {},
    "branchProtection": {},
    "labels": []
  }
}
```

### Error responses

| HTTP status | Code | Condition |
|---|---|---|
| 400 | `VALIDATION_ERROR` | Name format invalid, missing required field |
| 401 | `UNAUTHORIZED` | Token invalid or expired |
| 404 | `TEMPLATE_NOT_FOUND` | Template repository does not exist or is not accessible |
| 409 | `REPOSITORY_ALREADY_EXISTS` | Repository with that name already exists in the org |
| 502 | `GITHUB_API_ERROR` | GitHub API returned an unexpected error |

---

## `POST /api/v1/repositories/validate-name`

Checks whether a repository name satisfies GitHub naming rules.

### Request body

```json
{
  "organization": "myorg",
  "name": "payment-service"
}
```

### Validation rules

- Length: 1–100 characters
- Allowed characters: lowercase letters, numbers, hyphens, underscores, periods
- Cannot start with `.` or `-`
- Cannot be `"."` or `".."`

### Response — 200 OK

```json
{
  "valid": true,
  "available": true,
  "messages": null
}
```

On invalid name:

```json
{
  "valid": false,
  "available": null,
  "messages": ["Name cannot start with a hyphen"]
}
```

---

## `POST /api/v1/repositories/validate-request`

Validates a complete repository creation request without creating anything.

### Request body

Same shape as `POST /api/v1/repositories`.

### Response — 200 OK

```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

On invalid request:

```json
{
  "valid": false,
  "errors": [
    "template 'nonexistent-template' not found in organisation myorg"
  ],
  "warnings": []
}
```
