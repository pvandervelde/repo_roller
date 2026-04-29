---
title: "Validate the metadata repository configuration"
description: "Use repo-roller template validate to check template structure and configuration before use."
audience: "platform-engineer"
type: "how-to"
---

# Validate the metadata repository configuration

## Run the validate command

Three invocation modes are supported:

**Validate a local template directory (no GitHub credentials needed):**

```bash
repo-roller template validate --path ./my-template
```

**Validate a local directory and check the repository type against the org:**

```bash
repo-roller template validate --path ./my-template --org myorg
```

**Clone and validate a remote template:**

```bash
repo-roller template validate --org myorg --template rust-service
```

## Output formats

**Pretty (default):**

```
Validating template: rust-service

✓ Template repository accessible
✓ Configuration file found (.reporoller/template.toml)
✓ Template metadata complete
✓ Variables valid (2 defined)
✓ Repository type references valid

Template is VALID

Warnings (1):
  ⚠ best_practice: Consider adding more tags for better discoverability
```

**JSON (for scripting):**

```bash
repo-roller template validate --org myorg --template rust-service --format json
```

```json
{
  "template_name": "rust-service",
  "valid": true,
  "issues": [],
  "warnings": [
    {
      "category": "best_practice",
      "message": "Consider adding more tags for better discoverability"
    }
  ]
}
```

## What is checked

- TOML syntax is valid (configuration files parse without errors)
- Required fields in `[template]` are present (`name` is mandatory)
- Variable definitions are logically consistent (e.g. a `required = true` variable must not have a `default`)
- Variable names are valid identifiers (alphanumeric and underscores, starting with a letter)
- Repository type name references a known type in the metadata repository (when `--org` is provided)

## Remote type-validity checks

Remote checks (repository type existence) run automatically when an organisation context is available via `--org` or a detected GitHub remote in `.git/config`. If GitHub credentials are not configured, the remote check is skipped with a warning and structural validation still proceeds.

## Warnings vs errors

- **Errors** (`✗`): The template has issues that prevent it from being used. Fix these before publishing.
- **Warnings** (`⚠`): Non-critical suggestions for best practices. The template will work, but addressing these improves discoverability and usability.

## Common errors and fixes

| Error | Fix |
|---|---|
| `Missing required field 'name'` | Add `name = "..."` to `[template]` |
| `Invalid variable name format` | Use only `[a-z0-9_]` characters, starting with a letter |
| `Required variable with default` | Remove `default` or set `required = false` |
| `Repository type not found` | Verify the type name or create the type configuration |

## Related reference

- [CLI: repo-roller template validate](../../reference/cli/template.md)
