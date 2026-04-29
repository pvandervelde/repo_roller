---
title: "`repo-roller validate` — validate configuration"
description: "Full reference for the repo-roller validate command."
audience: "all"
type: "reference"
---

# `repo-roller validate` — validate configuration

Validate the metadata repository configuration or a standalone template directory for schema errors, unknown keys, and broken references.

## Synopsis

```
repo-roller validate [--org <ORG>] [--path <DIR>] [--format <FMT>]
```

## Flags

| Flag | Type | Required | Description |
|---|---|---|---|
| `--org <ORG>` | string | Conditional | GitHub organisation name. When provided, validates the organisation's metadata repository (`.reporoller`) remotely. Required when `--path` is absent. |
| `--path <DIR>` | string | Conditional | Local path to a metadata repository or template directory. When provided, validates the directory without a GitHub API call. |
| `--format <FMT>` | string | No | Output format: `pretty` (default) or `json` |

## What is validated

- TOML file syntax in all configuration files
- Required fields present in each config file
- Unknown keys detected and reported as warnings
- Variable name format rules
- Repository type references (when `--org` is provided)
- Label colour and name constraints
- Ruleset structure and rule types

## Examples

```bash
# Validate the live metadata repository for an organisation
repo-roller validate --org myorg

# Validate a local checkout of the metadata repository
repo-roller validate --path ./my-reporoller-config

# Validate and output JSON (for CI parsing)
repo-roller validate --org myorg --format json
```

## Output — pretty format (success)

```
Validating configuration for: myorg

✓ global/defaults.toml — valid
✓ types/library/config.toml — valid
✓ types/service/config.toml — valid
✓ teams/backend-team/config.toml — valid

Validation PASSED (4 files checked, 0 errors, 1 warning)

Warnings:
  ⚠ teams/backend-team/config.toml: Unknown key 'reposit_type' — did you mean 'repository_type'?
```

## Output — JSON format

```json
{
  "valid": true,
  "files_checked": 4,
  "errors": [],
  "warnings": [
    {
      "file": "teams/backend-team/config.toml",
      "category": "unknown_key",
      "message": "Unknown key 'reposit_type' — did you mean 'repository_type'?"
    }
  ]
}
```

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Validation passed (warnings do not affect exit code) |
| `1` | Validation failed (one or more errors) |
| `2` | Invalid command-line arguments |
