---
title: "Core concepts"
description: "The domain vocabulary you need to understand all other RepoRoller documentation."
audience: "all"
type: "explanation"
---

# Core concepts

RepoRoller uses a small set of domain terms consistently across the web UI, CLI, REST API, and documentation. Understanding these terms makes the rest of the system easy to reason about.

## Template repository

A GitHub repository that serves as the starting point for new repositories. It contains the scaffold files that will be copied into every new repository, plus a `.reporoller/template.toml` manifest that declares the template's metadata, variables, and configuration.

Template repositories live in the same GitHub organisation as the repositories they create. They are discovered automatically by a GitHub topic (`reporoller-template`).

## Metadata repository

A single GitHub repository — conventionally named `.reporoller` — that stores all organisation-level configuration. This is where you define defaults that apply to every repository, repository-type-specific settings, team overrides, and outbound notification webhooks.

The metadata repository is the source of truth for policy. Changing it affects all future repository creations immediately.

## Repository type

A named category of repository (e.g. `library`, `service`, `action`) with type-specific configuration stored in the metadata repository. Assigning a type to a created repository causes its type-level settings to be merged into the final configuration.

Repository types let you apply consistent policies across all repositories of the same kind without duplicating config in every template.

## Team configuration

A set of settings stored in the metadata repository under a team's directory (`teams/{team-name}/config.toml`). Applying a team configuration to a repository creation request causes the team's overrides to be layered on top of global and type defaults.

## Variable

A named value declared in a template's `[variables]` section that a user must (or may) supply at repository creation time. Variables are substituted into file content and file names using `{{variable_name}}` Handlebars syntax during template processing.

Variables have a description, an optional default, optional validation rules, and a flag indicating whether they are required.

## Content strategy

How a new repository is populated:

| Strategy | CLI flag | Description |
|---|---|---|
| `template` | `--template <name>` | Files copied from a template repository, variables substituted |
| `empty` | `--empty` | No files; only organisation default settings applied |
| `custom_init` | `--init-readme` / `--init-gitignore` | A minimal README.md and/or .gitignore seeded; org defaults applied |

Organisation security settings and branch protection rules are applied in all three cases.

## Configuration level

One of four layers in the configuration hierarchy:

1. **Global** — organisation-wide baseline (`global/defaults.toml`)
2. **Repository type** — per-type overrides (`types/{name}/config.toml`)
3. **Team** — per-team overrides (`teams/{name}/config.toml`)
4. **Template** — per-template overrides (`.reporoller/template.toml` in the template repo)

Higher levels override lower levels for scalar settings; array sections (labels, rulesets, webhooks) are additive.

## Override control

A setting may be marked `override_allowed = false` at any configuration level to prevent higher levels from changing it. Attempting to override a locked setting causes a hard error — the repository is not created.

This is the mechanism by which organisation policy is enforced: governance settings are locked at global or type level so that individual templates or requests cannot weaken them.
