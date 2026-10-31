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
| `contentStrategy` | string | No | `"template"` | How to populate the repository: `"template"`, `"empty"`, or `"custom_init"`. Must not be set to a non-default value together with a non-`"standard"` `creationMode` — see below. |
| `template` | string | Conditional | — | Template repository name. Required when `contentStrategy` is `"template"`. Optional (settings-only) with a fork/mirror `creationMode`. |
| `initializeReadme` | boolean | No | `false` | Generate a `README.md`. Only valid with `contentStrategy: "custom_init"`. |
| `initializeGitignore` | boolean | No | `false` | Generate a `.gitignore`. Only valid with `contentStrategy: "custom_init"`. |
| `creationMode` | object | No | `{"type": "standard"}` | Governs how the repository itself comes into existence: `"standard"` (default, create then run `contentStrategy`), `"fork"`, or `"mirror"`. See [Creation modes](#creation-modes-fork-and-mirror) below. |
| `description` | string | No | — | Repository description shown on GitHub |
| `visibility` | string | No | `"private"` | `"private"` or `"public"`. Subject to organisation policy. |
| `repositoryType` | string | No | — | Repository type slug for type-level configuration |
| `team` | string | No | — | Team slug for team-level configuration |
| `variables` | object | No | — | Key-value pairs of template variable values. Only valid with `contentStrategy: "template"`. |

### Creation modes (fork and mirror)

`creationMode` is an optional field, separate from and orthogonal to `contentStrategy`,
that bypasses ordinary content-strategy-based creation entirely — GitHub (for a fork) or
RepoRoller's own git mirroring (for a mirror) supplies the repository's contents instead.
`template` is optional with either mode and, if given, only ever contributes its
*settings* (branch protection, labels, etc.), never files. Both modes require the target
capability (`fork_enabled`/`source_history_mirror_enabled`/`source_only_mirror_enabled`)
to be enabled and the source owner to be allow-listed — see
[Configure fork and mirror repository creation](../../how-to/configure/fork-and-mirror-creation.md).

| Mode | `creationMode.type` | Fields | Description |
|---|---|---|---|
| Fork | `"fork"` | `source: { owner, name }` | Create a standard GitHub fork of the source repository. Joins GitHub's fork network; full history. |
| Mirror (source + history) | `"mirror"` | `source: { owner, name }`, `preserveHistory: true` | Create an independent copy of the source repository with full commit history, `HEAD` set to the source's default branch. Pushed directly. |
| Mirror (source only) | `"mirror"` | `source: { owner, name }`, `preserveHistory: false` (or omitted — defaults to `false`) | Create an independent repository seeded from the source's current files only (no history), delivered via a pull request for review. |

```json
{ "creationMode": { "type": "fork", "source": { "owner": "upstream-org", "name": "some-repo" } } }
{ "creationMode": { "type": "mirror", "source": { "owner": "upstream-org", "name": "some-repo" }, "preserveHistory": true } }
```

Sending an explicit, non-default `contentStrategy` together with a `"fork"`/`"mirror"`
`creationMode` is rejected with `400 VALIDATION_ERROR` on the `content_strategy` field.
`source.owner`/`source.name` are validated the same way as the top-level `organization`/
`name` fields; a malformed value is also rejected with `400 VALIDATION_ERROR`.

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

**Fork:**

```json
{
  "name": "payment-service",
  "organization": "myorg",
  "creationMode": {
    "type": "fork",
    "source": { "owner": "upstream-org", "name": "payment-service" }
  }
}
```

**Mirror (source + history):**

```json
{
  "name": "payment-service-mirror",
  "organization": "myorg",
  "creationMode": {
    "type": "mirror",
    "source": { "owner": "upstream-org", "name": "payment-service" },
    "preserveHistory": true
  }
}
```

**Mirror (source only, via pull request):**

```json
{
  "name": "payment-service-snapshot",
  "organization": "myorg",
  "creationMode": {
    "type": "mirror",
    "source": { "owner": "upstream-org", "name": "payment-service" },
    "preserveHistory": false
  }
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
| 400 | `VALIDATION_ERROR` | Name format invalid, missing required field, non-default `contentStrategy` combined with a fork/mirror `creationMode`, or malformed `creationMode.source.owner`/`.name` |
| 401 | `UNAUTHORIZED` | Token invalid or expired |
| 403 | `CapabilityDisabled` | The requested `creationMode` (`fork_enabled`/`source_history_mirror_enabled`/`source_only_mirror_enabled`) is disabled by configuration |
| 403 | `SourceOwnerNotAllowed` | `creationMode.source.owner` is not on the resolved `allowed_source_owners` allow-list |
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
