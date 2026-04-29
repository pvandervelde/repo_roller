---
title: "Template configuration schema"
description: "Complete field reference for .reporoller/template.toml in a template repository."
audience: "platform-engineer"
type: "reference"
---

# Template configuration schema

**File:** `.reporoller/template.toml` inside the template repository (not inside the metadata repository).

Defines the template's metadata, the variables users fill in at creation time, and configuration to apply to repositories created from this template.

---

## `[template]` — template metadata

| Field | TOML type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Template identifier used in CLI and API calls. Must be unique within the organisation. |
| `description` | string | No | Human-readable description shown in the web UI and CLI `template info` output. |
| `author` | string | No | Author or team name |
| `tags` | array of string | No | Tags for discoverability (e.g. `["rust", "microservice"]`) |

```toml
[template]
name        = "rust-service"
description = "Production-ready Rust microservice with gRPC and observability"
author      = "Platform Team"
tags        = ["rust", "microservice", "backend"]
```

---

## `[repository]` — repository settings override

Overrides type and global repository settings for repositories created from this template.

Supports the same fields as [global-config.md](global-config.md#repository--repository-feature-settings).

Additionally supports a `repository_type` field:

| Field | TOML type | Required | Description |
|---|---|---|---|
| `repository_type` | inline table | No | Forces or suggests a repository type for repositories created from this template |

Repository type policies:

| Policy | Description |
|---|---|
| `fixed` | Repositories created from this template always use this type. Users cannot override it. |
| `default` | This type is pre-selected but users can choose another. |
| `allowed` | No restriction on type; this is the default when `repository_type` is not set. |

```toml
[repository]
has_wiki = false

[repository.repository_type]
type_name = "service"
policy    = "fixed"
```

---

## `[pull_requests]` — pull request settings override

Identical fields to the global schema. See [global-config.md](global-config.md#pull_requests--pull-request-and-merge-settings).

---

## `[variables]` — user-supplied variables

Variables declared here are shown to users in the web UI and CLI at repository creation time.

Each variable is a key under `[variables]`:

```toml
[variables.service_name]
description = "Name of the service (used in file names and URLs)"
required    = true
example     = "payment-service"
pattern     = "^[a-z][a-z0-9-]*$"

[variables.service_port]
description = "Port the service listens on"
required    = false
default     = "8080"
example     = "3000"
min_length  = 2
max_length  = 5
```

| Field | TOML type | Required | Description |
|---|---|---|---|
| `description` | string | Yes | Shown to users in the web UI and CLI |
| `required` | bool | No (`false`) | When `true`, users must provide a value. When `false`, `default` is used if no value is given. |
| `default` | string | No | Value used when the user provides nothing. Only meaningful when `required = false`. |
| `example` | string | No | Example value shown as a placeholder in the web UI |
| `pattern` | string | No | Regular expression the value must match |
| `min_length` | integer | No | Minimum number of characters |
| `max_length` | integer | No | Maximum number of characters |

---

## `[[labels]]` — template-specific labels

Labels to create on repositories made from this template. Additive with global and type labels.

Same schema as [global-config.md](global-config.md#labels--default-repository-labels).

---

## `[[teams]]` — team access

Teams to assign to repositories created from this template.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `slug` | string | Yes | GitHub team slug |
| `access_level` | string | Yes | `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, or `"admin"` |

May upgrade (but not demote) team levels set at global/type config levels.

---

## `[[collaborators]]` — individual access

Individual collaborators to assign.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `username` | string | Yes | GitHub username |
| `access_level` | string | Yes | Same values as for teams |

---

## `[[webhooks]]` — repository webhooks

Webhooks applied to repositories. Additive with global and type webhooks. Same schema as [global-config.md](global-config.md#webhooks--repository-webhooks).

---

## `[[rulesets]]` — branch/tag protection

Rulesets applied to repositories. Additive. Same schema as global configuration.

---

## `[templating]` — file processing rules

Controls which files are copied to created repositories and which are processed for variable substitution.

| Field | TOML type | Default | Description |
|---|---|---|---|
| `exclude_patterns` | array of string | `[]` | Glob patterns for files/directories to exclude from output repos. `.reporoller/` is always excluded. |
| `process_extensions` | array of string | All text files | File extensions to process for variable substitution. When set, only files with these extensions are processed. |

```toml
[templating]
exclude_patterns   = ["README.md", ".github/workflows/test-template.yml"]
process_extensions = [".toml", ".rs", ".md", ".yml", ".json"]
```
