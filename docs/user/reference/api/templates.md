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
      "displayName": "Rust Library",
      "description": "Rust library template with CI/CD",
      "tags": ["rust"],
      "variables": ["project_name"]
    },
    {
      "name": "rust-service",
      "displayName": "Rust Service",
      "description": "Production-ready Rust microservice",
      "tags": ["rust", "microservice"],
      "variables": ["project_name", "service_port"]
    }
  ]
}
```

- `name` is the template repository's slug. `displayName` is the human-readable name
  configured in the template's `.reporoller/template.toml` (`name` field); if the template
  author left it unset, `displayName` is the same as `name`.
- `tags` lists every tag declared by the template. There is no `category` field.

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
  "displayName": "Rust Library",
  "description": "Rust library template with CI/CD",
  "tags": ["rust"],
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

---

## `GET /api/v1/orgs/{org}/creation-capabilities`

Discovers whether fork creation, mirror (source + history) creation, and
mirror (source-only) creation are enabled for the organisation, the resolved
source-owner allow-list, and the designated settings-only template (if any)
for each of the three modes — all resolved through the standard
Global → RepositoryType → Team configuration precedence.

This endpoint exists because no other endpoint can answer these questions
before a client composes a fork/mirror creation request:
`GET /api/v1/orgs/{org}/defaults` only reflects the Global level, and
`POST /api/v1/orgs/{org}/configuration/preview` requires a `template` name up
front, which fork/mirror creation intentionally makes optional. This endpoint
is purely informational — it performs zero mutations and issues no GitHub
write calls — and it always reports a disabled capability as `false` with a
`200` response, never as an error.

### Path parameters

| Parameter | Description |
|---|---|
| `org` | GitHub organisation slug |

### Query parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `team` | string | No | Team slug. Applies Team-level overrides on top of RepositoryType/Global. |
| `repositoryType` | string | No | Repository type slug. Applies RepositoryType-level overrides on top of Global. |

Both parameters are optional. With neither supplied, capabilities resolve at
the Global level only.

### Response — 200 OK

```json
{
  "forkEnabled": true,
  "sourceHistoryMirrorEnabled": true,
  "sourceOnlyMirrorEnabled": false,
  "allowedSourceOwners": ["upstream-org", "some-vendor-org"],
  "forkTemplate": "fork-settings",
  "sourceHistoryMirrorTemplate": null,
  "sourceOnlyMirrorTemplate": null
}
```

- `forkEnabled` / `sourceHistoryMirrorEnabled` / `sourceOnlyMirrorEnabled` —
  whether each creation mode is enabled, resolved from the
  `[creation_capabilities]` configuration at whichever level wins. All three
  default to `false` (fail-closed) when unset at every level.
- `allowedSourceOwners` — the resolved allow-list of GitHub owners permitted
  as a fork/mirror source. Always an array — `[]` when unset at every level,
  never `null` or omitted.
- `forkTemplate` / `sourceHistoryMirrorTemplate` / `sourceOnlyMirrorTemplate`
  — the name of the template designated (via `designation` in its
  `.reporoller/template.toml`) for that mode, or `null` if none is
  designated.

See [Configure fork and mirror repository creation](../../how-to/configure/fork-and-mirror-creation.md)
for how these fields are configured.

### Error responses

| HTTP status | Code | Condition |
|---|---|---|
| 400 | `InvalidConfiguration` | `repositoryType` query parameter names a repository type that doesn't exist in the organisation |
| 409 | `AmbiguousTemplateDesignation` | Two or more templates in the organisation declare the same `designation` value |
