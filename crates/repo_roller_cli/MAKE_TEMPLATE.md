# RepoRoller CLI - Creating Template Repositories

This guide covers the `make-template` command, which converts an existing local Git repository into a RepoRoller template by scaffolding all the necessary configuration and starter files.

## Table of Contents

- [Overview](#overview)
- [When to Use This Command](#when-to-use-this-command)
- [Command Syntax](#command-syntax)
- [Arguments](#arguments)
- [Examples](#examples)
- [Command Output](#command-output)
- [Generated File Structure](#generated-file-structure)
- [Understanding Each Scaffold File](#understanding-each-scaffold-file)
- [Template Variables](#template-variables)
- [Handling Existing Files](#handling-existing-files)
- [Next Steps After Running](#next-steps-after-running)
- [Full Workflow: Create a Template Then Use It](#full-workflow-create-a-template-then-use-it)

## Overview

The `make-template` command sets up a Git repository so that RepoRoller can use it as a template when creating new repositories for your organization. Running the command once produces:

- A `.reporoller/template.toml` configuration file with **all available settings** documented and commented out (so you know exactly what you can configure)
- A `README.md` for template developers (visible on GitHub, excluded from repos created from this template)
- A `README.md.template` scaffold that becomes the `README.md` in every repo created from this template
- Template-aware `.gitignore` files using the same pattern
- GitHub Actions workflow files for both validating the template itself and scaffolding CI in created repos
- Optionally a `renovate.json` dependency-update configuration

The command shows a **preview** of every file it will create (or skip) before making any changes. You can confirm interactively or use `--yes` to skip the prompt for CI pipelines.

## When to Use This Command

Use `make-template` when you want to:

- Turn an existing repository into a reusable organizational template
- Start a new template repository from scratch with a proper structure
- Add RepoRoller template metadata to a repository that was previously managed manually
- Get a fully documented `template.toml` showing all supported configuration options

**Prerequisites:**

- The target path must be a Git repository (must contain a `.git` entry — either a directory for normal repos or a file for worktree / submodule checkouts)
- You must have write access to the repository's working directory

## Command Syntax

```bash
repo-roller make-template <PATH> [OPTIONS]
```

## Arguments

| Argument | Type | Description |
|---|---|---|
| `<PATH>` | Required positional | Path to the Git repository to convert into a template |
| `--name <NAME>` | Optional | Template name written into `template.toml`; defaults to the repository directory name |
| `--description <DESC>` | Optional | Human-readable description of this template; defaults to `"A new repository template"` |
| `--author <AUTHOR>` | Optional | Author name or team owning this template; defaults to `"Your Name / Team"` |
| `--force` | Flag | Overwrite existing scaffold files instead of skipping them |
| `--renovate` | Flag | Also create a `renovate.json` dependency-update configuration |
| `--yes` / `-y` | Flag | Skip the confirmation prompt (suitable for CI / non-interactive use) |

## Examples

### Basic usage — let the command infer the name from the directory

```bash
repo-roller make-template ./rust-service-template
```

### Provide all metadata up front

```bash
repo-roller make-template ./rust-service-template \
  --name "rust-service" \
  --description "Production-ready Rust microservice with gRPC and observability" \
  --author "Platform Team"
```

### Add Renovate dependency-update configuration

```bash
repo-roller make-template ./rust-service-template --renovate
```

### Skip confirmation (for CI pipelines or automated setup)

```bash
repo-roller make-template ./rust-service-template --yes
```

### Re-run to update scaffold files after `template.toml` changes

```bash
# Skip files that already exist (default behaviour — safe to re-run)
repo-roller make-template ./rust-service-template

# Or overwrite all scaffold files to get the latest generated versions
repo-roller make-template ./rust-service-template --force
```

### All options combined

```bash
repo-roller make-template ./rust-service-template \
  --name "rust-service" \
  --description "Production-ready Rust microservice with gRPC and observability" \
  --author "Platform Team" \
  --renovate \
  --yes
```

## Command Output

### Preview

Before writing any files, the command prints a table of planned operations:

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

Each line shows the action (`CREATE`, `OVERWRITE`, or `SKIP`) and an optional note explaining the file's role.

### Summary

After writing files, the command prints a brief summary and next-step guidance:

```
Template initialization complete!
  Written : 7 file(s)

Next steps:
  1. Edit .reporoller/template.toml to configure your template
  2. Customize README.md.template and .gitignore.template for repos created from this template
  3. Update .github/workflows/ci.yml.template for your project's CI needs
  4. Push this repository to GitHub and register it with RepoRoller
```

## Generated File Structure

The table below shows every file the command creates and its relationship to repositories created from this template.

| File | Role | Appears in output repos? |
|---|---|---|
| `.reporoller/template.toml` | Template configuration — all options documented | Never (`.reporoller/` is always excluded) |
| `README.md` | Template developer documentation shown on GitHub | No — excluded via `exclude_patterns` |
| `README.md.template` | README scaffold for new repositories | Yes — `.template` suffix stripped → `README.md` |
| `.gitignore` | Template developer gitignore | No — excluded via `exclude_patterns` |
| `.gitignore.template` | Starter gitignore for new repositories | Yes — `.template` suffix stripped → `.gitignore` |
| `.github/workflows/test-template.yml` | CI workflow that validates this template's structure | No — excluded via `exclude_patterns` |
| `.github/workflows/ci.yml.template` | CI scaffold for new repositories | Yes — `.template` suffix stripped → `.github/workflows/ci.yml` |
| `renovate.json` | Dependency-update config (opt-in via `--renovate`) | Yes — copied as-is |

### The `.template` suffix convention

The RepoRoller template engine automatically strips the `.template` suffix from any file when creating a new repository. This lets the template repository contain both a developer-facing version of a file (e.g. `README.md`) and an output scaffold (e.g. `README.md.template`) side by side without conflicts.

## Understanding Each Scaffold File

### `.reporoller/template.toml`

The central configuration file for this template. The generated version activates only the `[template]` (metadata) and `[templating]` (file-filtering) sections; every other option is commented out with inline documentation. Edit this file to:

- Add tags for discoverability
- Define template variables (substituted via `{{variable_name}}` syntax)
- Restrict the repository type or visibility
- Configure default branch-protection rulesets, labels, webhooks, teams, and collaborators
- Add repository naming rules

### `README.md`

Shown on the template repository's GitHub page. Use it to explain:

- What kind of projects this template is for
- What the template provides (CI, linting, etc.)
- How template variables work
- How to create a repository from this template

This file is listed in `exclude_patterns` inside `template.toml`, so it is **never** copied into repos created from the template.

### `README.md.template`

The README scaffold that every new repository gets as its `README.md`. The generated content includes `{{repo_name}}` and `{{template_name}}` placeholders that RepoRoller replaces with real values at creation time. Customize this to match the expected structure of projects built from your template.

### `.gitignore` and `.gitignore.template`

Same pattern as README:

- `.gitignore` contains rules specific to working on the template repository itself (e.g. ignoring local build artifacts related to template development)
- `.gitignore.template` becomes the `.gitignore` in every created repo and should contain rules appropriate for the kinds of projects built from this template

### `.github/workflows/test-template.yml`

A GitHub Actions workflow that validates the template repository's own structure — for example, checking that `template.toml` is valid TOML and that required scaffold files exist. This workflow runs inside the template repo, not in repos created from it, so it is excluded via `exclude_patterns`.

### `.github/workflows/ci.yml.template`

The CI scaffold that becomes `.github/workflows/ci.yml` in each created repository. The generated starter workflow includes `{{repo_name}}` and `{{template_name}}` placeholders and a minimal build-then-test structure. Update this to match the build system and test tooling used by projects built from your template.

### `renovate.json` (opt-in)

A standard Renovate configuration using `config:recommended`. Included in created repos because keeping dependencies up to date is generally desirable for all projects based on the template. Only generated when `--renovate` is passed.

## Template Variables

Variables let users supply values at repository-creation time (e.g. a service name, port number, or owner username). RepoRoller substitutes them using Handlebars syntax (`{{variable_name}}`) in:

- File **content** — any text file in the repository
- File and directory **names** — paths containing `{{variable_name}}`

Variables are declared in `.reporoller/template.toml`. The generated file includes a commented-out example:

```toml
# [variables.service_name]
# description = "Name of the service"
# required    = true
# example     = "user-service"
# pattern     = "^[a-z][a-z0-9-]*$"   # Regex validation (optional)
# min_length  = 3
# max_length  = 63
# default     = ""
```

The scaffold files generated by `make-template` already use `{{repo_name}}` and `{{template_name}}` — these are implicitly available for every template and do not need to be declared in `[variables]`.

## Handling Existing Files

By default the command **skips** files that already exist, printing `SKIP` in the preview. This makes it safe to re-run `make-template` after editing your `template.toml` without accidentally overwriting customizations.

To regenerate all scaffold files to their latest defaults, pass `--force`:

```bash
repo-roller make-template ./rust-service-template --force
```

**Warning:** `--force` overwrites existing files without a backup. Commit or stash local changes before using it.

## Next Steps After Running

1. **Open `.reporoller/template.toml`** and work through the commented-out sections. At minimum, add meaningful `tags` and uncomment any settings your template requires.

2. **Edit `README.md`** to describe the template for developers browsing your GitHub organization.

3. **Edit `README.md.template`** to provide a useful starting README for projects built from this template. Replace or expand the `{{repo_name}}` usage to match your project structure.

4. **Edit `.gitignore.template`** to list the ignore patterns appropriate for your target language or framework.

5. **Edit `.github/workflows/ci.yml.template`** to add the real build, lint, and test steps your projects need.

6. If you added variables to `template.toml`, **reference them** in your scaffold files using `{{variable_name}}`.

7. **Push the repository to GitHub** and register it as a template in your organization's RepoRoller configuration.

8. **Test the template** with `repo-roller template validate` to catch configuration issues before your team starts using it:

```bash
repo-roller template validate --org myorg --template rust-service
```

## Full Workflow: Create a Template Then Use It

```bash
# Step 1: Scaffold the template files into an existing repo
repo-roller make-template ./rust-service-template \
  --name "rust-service" \
  --description "Production-ready Rust microservice" \
  --author "Platform Team" \
  --renovate

# Step 2: Edit the generated files as needed, then commit
cd rust-service-template
git add .reporoller/ README.md README.md.template .gitignore .gitignore.template \
        .github/ renovate.json
git commit -m "chore: Initialize RepoRoller template scaffold"
git push

# Step 3: Validate the template via the CLI
repo-roller template validate --org myorg --template rust-service

# Step 4: Create a new repository from the template
repo-roller create \
  --org myorg \
  --repo my-new-service \
  --template rust-service \
  --description "My new service"
```
