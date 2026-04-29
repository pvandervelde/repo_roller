---
title: "Templates and organisation settings API"
description: "API reference for listing and inspecting templates and repository types."
audience: "all"
type: "reference"
---

# Templates and organisation settings API

## `GET /api/v1/orgs/{org}/templates`

Lists all templates available in the organisation.

Templates are discovered by searching for repositories with the `reporoller-template` GitHub topic inside the organisation.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |

### Response — 200 OK

```json
{
  "templates": [
    {
      "name": "rust-library",
      "description": "Rust library template with CI/CD",
      "category": "rust"
    },
    {
      "name": "rust-service",
      "description": "Production-ready Rust microservice",
      "category": "rust"
    }
  ]
}
```

---

## `GET /api/v1/orgs/{org}/templates/{template}`

Loads the full configuration for a specific template.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |
| `template` | Template repository name |

### Response — 200 OK

```json
{
  "name": "rust-library",
  "description": "Rust library template with CI/CD",
  "variables": {
    "project_name": {
      "description": "Human-readable project name",
      "required": true,
      "default": null,
      "example": "my-library"
    },
    "service_port": {
      "description": "Port number",
      "required": false,
      "default": "8080",
      "example": "3000"
    }
  },
  "configuration": {
    "repositoryType": {
      "policy": "fixed",
      "value": "library"
    }
  }
}
```

### Error responses

| HTTP status | Code | Condition |
|---|---|---|
| 404 | `TEMPLATE_NOT_FOUND` | Template repository does not exist or is not accessible |

---

## `POST /api/v1/orgs/{org}/templates/{template}/validate`

Validates a template's `.reporoller/template.toml` for structural correctness.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |
| `template` | Template repository name |

### Response — 200 OK

```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "category": "best_practice",
      "message": "Consider adding tags for discoverability"
    }
  ]
}
```

---

## `GET /api/v1/orgs/{org}/repository-types`

Lists all repository types defined in the organisation's metadata repository.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |

### Response — 200 OK

```json
{
  "types": [
    {
      "name": "library",
      "description": "Reusable library packages"
    },
    {
      "name": "service",
      "description": "Deployable microservices"
    }
  ]
}
```

---

## `GET /api/v1/orgs/{org}/repository-types/{type}`

Retrieves the resolved configuration for a specific repository type.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |
| `type` | Repository type slug |

### Response — 200 OK

```json
{
  "name": "library",
  "configuration": {
    "features": {
      "hasIssues": { "value": true, "overrideAllowed": false },
      "hasWiki": { "value": false, "overrideAllowed": true }
    }
  }
}
```

### Error responses

| HTTP status | Code | Condition |
|---|---|---|
| 404 | `TYPE_NOT_FOUND` | Repository type does not exist in the metadata repository |
