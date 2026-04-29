---
title: "Configure"
description: "How-to guides for configuring organisation-wide defaults, repository types, team settings, and templates."
---

# Configure

All configuration lives in the metadata repository (`.reporoller`). Changes take effect immediately on the next repository creation — no restart required.

| Guide | What it covers |
|---|---|
| [Register a template](register-template.md) | Add the GitHub topic that makes RepoRoller discover a template |
| [Set organisation-wide defaults](global-defaults.md) | Edit `global/defaults.toml` |
| [Configure a repository type](repository-types.md) | Create `types/{type}/config.toml` |
| [Configure team-level settings](team-configuration.md) | Create `teams/{team}/config.toml` |
| [Add configuration to a template](template-configuration.md) | Edit `.reporoller/template.toml` in a template repo |
| [Configure branch protection rules](branch-protection-rules.md) | `[[rulesets]]` section |
| [Configure repository labels](labels.md) | `[[labels]]` section |
| [Configure repository webhooks](webhooks.md) | `[[webhooks]]` section |
| [Configure outbound notification webhooks](outbound-notifications.md) | `notifications.toml` files |
| [Configure team and collaborator permissions](team-permissions.md) | `[[default_teams]]`, `[[teams]]`, `[permissions]` |
| [Validate the configuration](validate-configuration.md) | `repo-roller template validate` |
