---
title: "Use variables in file content"
description: "Reference user-declared and built-in variables using Handlebars syntax in template files."
audience: "platform-engineer"
type: "how-to"
---

# Use variables in file content

RepoRoller uses [Handlebars](https://handlebarsjs.com/) to substitute variables into text file content when creating a repository.

## Basic substitution

Use `{{variable_name}}` to insert a variable value:

```markdown
# {{repo_name}}

{{description}}

Maintained by the **{{team_name}}** team.
```

When a repository named `payment-service` is created by the `payments-team` team, the output is:

```markdown
# payment-service

Payment processing microservice.

Maintained by the **payments-team** team.
```

## Unescaped substitution

`{{variable}}` HTML-escapes the value (e.g. `&` → `&amp;`). Use triple braces for raw output:

```
{{{raw_html_content}}}
```

For most variables this makes no difference. Use unescaped substitution only when the value must be injected verbatim into HTML or similar markup.

## Conditional blocks

```handlebars
{{#if enable_monitoring}}
monitoring:
  endpoint: "https://metrics.myorg.example"
{{else}}
# monitoring disabled
{{/if}}
```

The condition is truthy for any non-empty, non-`"false"` string value.

## Each / iteration

```handlebars
{{#each tags}}
- {{this}}
{{/each}}
```

Iteration is intended for variables declared as lists. Use `{{@index}}` for the zero-based iteration index.

## Built-in variables

These are available in every template without declaration:

| Variable | Example |
|---|---|
| `{{repo_name}}` | `payment-service` |
| `{{org_name}}` | `myorg` |
| `{{template_name}}` | `rust-service` |
| `{{creator_username}}` | `jane.doe` |
| `{{created_at}}` | `2026-04-27T14:30:00Z` |
| `{{repository_type}}` | `service` |
| `{{visibility}}` | `private` |

## Complete README.md.template example

```handlebars
# {{repo_name}}

{{description}}

## Service details

| Field | Value |
|---|---|
| Service name | `{{service_name}}` |
| Port | `{{service_port}}` |
| Team | `{{team_name}}` |
| Repository type | `{{repository_type}}` |
| Created | {{created_at}} |

## Getting started

```bash
git clone https://github.com/{{org_name}}/{{repo_name}}.git
cd {{repo_name}}
cargo run
```

{{#if enable_monitoring}}

## Monitoring

This service exports Prometheus metrics at `/metrics` on port `{{service_port}}`.
{{/if}}

```

## Files that are NOT processed

The template engine only processes text files. These are skipped:

- Binary files (images, compiled binaries, archives, fonts)
- Files listed in `exclude_patterns` in `template.toml`
- The `.reporoller/` directory

## Related guides

- [Define template variables](define-template-variables.md)
- [Use variables in file and directory names](use-variables-in-filenames.md)
- [Handlebars syntax reference](../../reference/template-authoring/handlebars-syntax.md)
- [Built-in template variables](../../reference/template-authoring/built-in-variables.md)
