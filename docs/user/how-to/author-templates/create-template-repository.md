---
title: "Create a template repository"
description: "Scaffold a RepoRoller template repository using the make-template command."
audience: "platform-engineer"
type: "how-to"
---

# Create a template repository

## Prerequisites

- The `repo-roller` CLI installed
- A Git repository (existing or newly created) with write access to the working directory
- The directory must contain a `.git` entry (directory for normal repos, or a file for worktree/submodule checkouts)

## Run make-template

```bash
repo-roller make-template ./my-template \
  --name "rust-service" \
  --description "Production-ready Rust microservice" \
  --author "Platform Team"
```

The command shows a preview of every file it will create or skip. Type `y` to confirm.

To skip the prompt in CI:

```bash
repo-roller make-template ./my-template \
  --name "rust-service" \
  --author "Platform Team" \
  --yes
```

To also generate a `renovate.json` dependency-update configuration:

```bash
repo-roller make-template ./my-template --renovate --yes
```

## Generated file structure

| File | Role | Included in created repos? |
|---|---|---|
| `.reporoller/template.toml` | Template configuration | No (`.reporoller/` is always excluded) |
| `README.md` | Template developer docs on GitHub | No (excluded via `exclude_patterns`) |
| `README.md.template` | README scaffold for created repos | Yes (`.template` suffix stripped → `README.md`) |
| `.gitignore` | Template developer gitignore | No (excluded via `exclude_patterns`) |
| `.gitignore.template` | Starter gitignore for created repos | Yes (`.template` suffix stripped → `.gitignore`) |
| `.github/workflows/test-template.yml` | CI that validates this template | No (excluded via `exclude_patterns`) |
| `.github/workflows/ci.yml.template` | CI scaffold for created repos | Yes (`.template` suffix stripped → `ci.yml`) |
| `renovate.json` | Dependency update config (opt-in) | Yes (copied as-is) |

## Handling existing files

By default, `make-template` **skips** files that already exist. Run again any time to add missing scaffold files without overwriting customisations.

To regenerate all scaffold files to their latest defaults:

```bash
repo-roller make-template ./my-template --force
```

> **Warning:** `--force` overwrites files without a backup. Commit any local changes first.

## Next steps

1. Edit `.reporoller/template.toml` — add tags, define variables, configure rulesets
2. Edit `README.md.template` — describe the structure developers will get
3. Edit `.gitignore.template` — add language-appropriate ignore patterns
4. Edit `.github/workflows/ci.yml.template` — add real build and test steps
5. Push to GitHub and [register the template](../configure/register-template.md)
6. Validate: `repo-roller template validate --org myorg --template rust-service`

See [make-template reference](../../reference/cli/make-template.md) for all flags.
