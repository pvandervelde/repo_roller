---
title: "`repo-roller make-template` — scaffold a template repository"
description: "Full reference for the repo-roller make-template command."
audience: "platform-engineer"
type: "reference"
---

# `repo-roller make-template` — scaffold a template repository

Converts an existing Git repository into a RepoRoller template by generating all required configuration and starter files.

## Synopsis

```
repo-roller make-template <PATH> [OPTIONS]
```

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<PATH>` | string | Yes | Path to the Git repository to convert. Must contain a `.git` entry (directory for normal repos, file for worktrees/submodules). |
| `--name <NAME>` | string | No | Template name written into `template.toml`. Defaults to the repository directory name. |
| `--description <DESC>` | string | No | Human-readable description. Defaults to `"A new repository template"`. |
| `--author <AUTHOR>` | string | No | Author name or team. Defaults to `"Your Name / Team"`. |
| `--force` | flag | No | Overwrite existing scaffold files (default: skip existing files). |
| `--renovate` | flag | No | Also create a `renovate.json` dependency-update configuration. |
| `--yes`, `-y` | flag | No | Skip the confirmation prompt. Suitable for CI and non-interactive use. |

## Generated files

| File | Role | Included in output repos? |
|---|---|---|
| `.reporoller/template.toml` | Template configuration — all options documented and commented | Never (`.reporoller/` is always excluded) |
| `README.md` | Template developer documentation shown on GitHub | No — excluded via `exclude_patterns` |
| `README.md.template` | README scaffold for new repositories | Yes — `.template` suffix stripped → `README.md` |
| `.gitignore` | Template developer gitignore | No — excluded via `exclude_patterns` |
| `.gitignore.template` | Starter gitignore for repos created from this template | Yes — `.template` suffix stripped → `.gitignore` |
| `.github/workflows/test-template.yml` | CI workflow that validates this template's structure | No — excluded via `exclude_patterns` |
| `.github/workflows/ci.yml.template` | CI scaffold for new repositories | Yes — `.template` suffix stripped → `.github/workflows/ci.yml` |
| `renovate.json` | Dependency-update config (opt-in via `--renovate`) | Yes — copied as-is |

### The `.template` suffix convention

The template engine automatically strips the `.template` suffix from any file name when creating a new repository. This lets the template repository contain both a developer-facing version of a file (e.g. `README.md`) and an output scaffold (e.g. `README.md.template`) side by side without naming conflicts.

## Behaviour with existing files

By default the command **skips** files that already exist, printing `SKIP` in the preview. This makes it safe to re-run after editing `template.toml`.

Pass `--force` to overwrite all scaffold files with the latest generated versions.

> **Warning:** `--force` overwrites files without a backup. Commit or stash local changes before using it.

## Examples

```bash
# Basic usage — infer name from directory
repo-roller make-template ./rust-service-template

# Provide all metadata up front
repo-roller make-template ./rust-service-template \
  --name "rust-service" \
  --description "Production-ready Rust microservice with gRPC and observability" \
  --author "Platform Team"

# Add Renovate configuration
repo-roller make-template ./rust-service-template --renovate

# Skip confirmation (for CI pipelines)
repo-roller make-template ./rust-service-template --yes

# Re-run to refresh scaffold files
repo-roller make-template ./rust-service-template --force
```

## Output

Before writing any files, the command prints a preview table:

```
Initializing template in: ./rust-service-template

  The following files will be affected:

  CREATE     .reporoller/template.toml
  CREATE     README.md  [template developer docs — excluded from output repos]
  CREATE     README.md.template  [scaffold for repos created from this template → renamed to README.md]
  CREATE     .gitignore  [template developer gitignore — excluded from output repos]
  CREATE     .gitignore.template  [starter gitignore for new repos → renamed to .gitignore]
  CREATE     .github/workflows/test-template.yml  [validates template structure in CI — excluded from output repos]
  CREATE     .github/workflows/ci.yml.template  [CI scaffold for new repos → renamed to ci.yml]

Proceed? [y/N]:
```

After writing files:

```
Template initialization complete!
  Written : 7 file(s)

Next steps:
  1. Edit .reporoller/template.toml to configure your template
  2. Customize README.md.template and .gitignore.template for repos created from this template
  3. Update .github/workflows/ci.yml.template for your project's CI needs
  4. Push this repository to GitHub and register it with RepoRoller
```
