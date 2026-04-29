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
| `description` | string | Yes | Human-readable description shown in the web UI and CLI `template info` output. |
| `author` | string | Yes | Author or team name |
| `tags` | array of string | Yes | Tags for discoverability. Use an empty array (`[]`) when no tags are needed. |

```toml
[template]
name        = "rust-service"
description = "Production-ready Rust microservice with gRPC and observability"
author      = "Platform Team"
tags        = ["rust", "microservice", "backend"]
```

---

## `[repository]` — repository settings override

Overrides global repository settings for repositories created from this template.

Supports the same fields as [global-config.md](global-config.md#repository--repository-feature-settings).

---

## `[repository_type]` — repository type policy

A **top-level** section (not nested under `[repository]`) that forces or suggests a repository type.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `type` | string | Yes | Repository type slug. Must match a type defined in the metadata repository. |
| `policy` | string | Yes | See policy values below. |

Repository type policies:

| Policy | Description |
|---|---|
| `fixed` | Repositories created from this template always use this type. Users cannot override it. |
| `preferable` | This type is pre-selected but users can choose another type during creation. |

Omit the entire `[repository_type]` section to place no restriction on repository type.

```toml
[repository]
has_wiki = false

[repository_type]
type   = "service"
policy = "fixed"
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
| `options` | array of string | No | Restricts the value to one of the listed strings. The web UI presents these as a dropdown. |

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

Controls which files in the template repository are processed for variable substitution.

| Field | TOML type | Default | Description |
|---|---|---|---|
| `include_patterns` | array of string | `[]` (all files) | Glob patterns for files to include in variable substitution. When empty, all files are processed. When set, only matching files are processed. |
| `exclude_patterns` | array of string | `[]` | Glob patterns for files/directories to skip entirely. `.reporoller/` is always excluded regardless of this setting. |

```toml
[templating]
include_patterns = ["**/*.toml", "**/*.rs", "**/*.md", "**/*.yml", "**/*.json"]
exclude_patterns = ["README.md", ".github/workflows/test-template.yml"]
```

> **Note:** Both fields accept standard glob patterns (e.g. `**/*.rs` for all Rust files, `src/**` for everything under `src/`). They are not simple file-extension lists.
