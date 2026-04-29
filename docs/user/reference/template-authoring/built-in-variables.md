---
title: "Built-in template variables"
description: "Variables automatically available in every template without declaration."
audience: "platform-engineer"
type: "reference"
---

# Built-in template variables

The following variables are injected by RepoRoller at repository creation time. They are available for use in file content and file/directory names without any declaration in `[variables]`.

## Variable table

| Variable | Type | Description | Example value |
|---|---|---|---|
| `repo_name` | string | Repository name as supplied in the creation request | `payment-service` |
| `org_name` | string | GitHub organisation slug | `myorg` |
| `template_name` | string | Name of the template being used (from `[template].name`) | `rust-service` |
| `creator_username` | string | GitHub username of the person who submitted the creation request | `jane.doe` |
| `created_at` | string | ISO 8601 UTC timestamp of when the repository was created | `2026-04-27T14:30:00Z` |
| `repository_type` | string | Repository type slug, if one was specified; empty string otherwise | `service` |
| `visibility` | string | Repository visibility as applied (may differ from request if policy restricted it) | `private` |

## Usage example

```markdown
<!-- README.md.template -->
# {{repo_name}}

Created from the `{{template_name}}` template.

Organisation: `{{org_name}}`
Created by: @{{creator_username}}
```

## Notes

- Built-in variables are always available and cannot be overridden by user input.
- User-declared variables (defined in `[variables]`) have separate names and do not conflict with built-in names. Avoid declaring user variables with the same names as built-in ones.
- `created_at` uses the UTC time zone and the format `YYYY-MM-DDTHH:MM:SSZ`.
