---
title: "`repo-roller template` — inspect and validate templates"
description: "Full reference for the repo-roller template info and template validate subcommands."
audience: "all"
type: "reference"
---

# `repo-roller template` — inspect and validate templates

Two subcommands for discovering information about templates and checking them for errors.

---

## `repo-roller template info`

Display details about a specific template.

### Synopsis

```
repo-roller template info --org <ORG> --template <TMPL> [--format <FMT>]
```

### Flags

| Flag | Type | Required | Description |
|---|---|---|---|
| `--org <ORG>` | string | Yes | GitHub organisation name |
| `--template <TMPL>` | string | Yes | Template repository name |
| `--format <FMT>` | string | No | Output format: `pretty` (default) or `json` |

### Output — pretty format

```
Template: rust-library

Description: Production-ready Rust library template with CI/CD
Author: Platform Team
Tags: rust, library, ci-cd

Repository Type: library (policy: fixed)

Variables (2):
  ✓ project_name [required]
    Human-readable project name
    Example: my-awesome-library

  • service_port [optional]
    Service port number
    Default: 8080
    Example: 3000

Configuration: 3 sections defined
```

### Output — JSON format

```json
{
  "name": "rust-library",
  "description": "Production-ready Rust library template with CI/CD",
  "author": "Platform Team",
  "tags": ["rust", "library", "ci-cd"],
  "repository_type": {
    "type_name": "library",
    "policy": "fixed"
  },
  "variables": [
    {
      "name": "project_name",
      "description": "Human-readable project name",
      "required": true,
      "example": "my-awesome-library"
    },
    {
      "name": "service_port",
      "description": "Service port number",
      "required": false,
      "default_value": "8080",
      "example": "3000"
    }
  ],
  "configuration_sections": 3
}
```

---

## `repo-roller template validate`

Validate a template's structure and configuration. Three invocation modes are supported:

| Mode | Flags | When to use |
|---|---|---|
| Local only | `--path <DIR>` | Validate disk contents without GitHub credentials |
| Remote | `--org <ORG> --template <TMPL>` | Clone from GitHub and validate |
| Local + remote checks | `--path <DIR> --org <ORG>` | Validate disk contents and verify remote type references |

### Synopsis

```
repo-roller template validate [--path <DIR>] [--org <ORG>] [--template <TMPL>] [--format <FMT>]
```

At least one of `--path` or `--org` + `--template` must be provided.

### Flags

| Flag | Type | Required | Description |
|---|---|---|---|
| `--path <DIR>` | string | Conditional | Local directory containing the template repository. When provided, structural checks run without a GitHub API call. |
| `--org <ORG>` | string | Conditional | Organisation name. Required when `--path` is absent. When used with `--path`, also runs remote type-validity checks. |
| `--template <TMPL>` | string | Conditional | Template repository name. Required when `--path` is absent. |
| `--format <FMT>` | string | No | Output format: `pretty` (default) or `json` |

Remote type-validity checks require GitHub credentials (`GITHUB_TOKEN`). If credentials are absent, the remote check is skipped with a warning and structural validation still proceeds.

### Output — pretty format (valid template)

```
Validating template: rust-library

✓ Template repository accessible
✓ Configuration file found (.reporoller/template.toml)
✓ Template metadata complete
✓ Variables valid (2 defined)
✓ Repository type references valid

Template is VALID

Warnings (1):
  ⚠ best_practice: Consider adding more tags for better discoverability
```

### Output — pretty format (invalid template)

```
Validating template: broken-template

✗ Template has ISSUES

Issues (2):
  ✗ template.toml: Missing required field 'name'
  ✗ variables.service_name: Invalid variable name format

Template validation FAILED
```

### Output — JSON format

```json
{
  "template_name": "rust-library",
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

### Examples

```bash
# Validate a local template directory (no credentials needed)
repo-roller template validate --path ./my-template

# Validate a local directory and also check remote repository type
repo-roller template validate --path ./my-template --org myorg

# Validate a remote template by cloning it first
repo-roller template validate --org myorg --template rust-library

# Validate remote and output JSON
repo-roller template validate --org myorg --template rust-library --format json
```
