---
title: "Define template variables"
description: "Add user-supplied variables to a template's template.toml manifest."
audience: "platform-engineer"
type: "how-to"
---

# Define template variables

Variables let repository creators supply values at creation time. RepoRoller substitutes them into file content and file names when the repository is created.

## Add a variable to template.toml

Open `.reporoller/template.toml` and add a `[variables.<name>]` section for each variable:

```toml
[variables.service_name]
description = "Name of the microservice (lowercase, hyphens allowed)"
required    = true
example     = "payment-service"
pattern     = "^[a-z][a-z0-9-]*$"
min_length  = 3
max_length  = 63

[variables.service_port]
description = "Port the service listens on"
required    = false
default     = "8080"
example     = "3000"
```

## Supported fields

| Field | Type | Required | Description |
|---|---|---|---|
| `description` | string | No | Shown to the creator in the UI and CLI |
| `required` | bool | No | `true` means the creator must supply a value; default is `false` |
| `default` | string | No | Value used when the creator provides no value; only valid when `required = false` |
| `example` | string | No | Shown as a placeholder or hint to the creator |
| `pattern` | string | No | Regular-expression constraint; creation fails if the value does not match |
| `min_length` | integer | No | Minimum string length in characters |
| `max_length` | integer | No | Maximum string length in characters |

> **Note:** A variable cannot have both `required = true` and a `default` value — that is a configuration error. Make the variable optional or remove the default.

## Two-variable example

```toml
[variables.team_name]
description = "GitHub slug of the owning team"
required    = true
example     = "platform-engineering"
pattern     = "^[a-z][a-z0-9-]*$"
max_length  = 39

[variables.enable_monitoring]
description = "Include monitoring configuration"
required    = false
default     = "true"
example     = "false"
```

## Variable naming rules

- Names must be alphanumeric with underscores (`[a-z0-9_]`), starting with a letter.
- Names are case-sensitive. Use lowercase throughout.
- Do not use a name that conflicts with a built-in variable (`repo_name`, `org_name`, `template_name`, `creator_username`, `created_at`, `repository_type`, `visibility`).

## Related guides

- [Use variables in file content](use-variables-in-files.md)
- [Use variables in file and directory names](use-variables-in-filenames.md)
- [Built-in template variables](../../reference/template-authoring/built-in-variables.md)
