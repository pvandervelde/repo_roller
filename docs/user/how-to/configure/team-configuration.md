---
title: "Configure team-level settings"
description: "Create a team configuration file to override defaults for repositories created by or for a specific team."
audience: "platform-engineer"
type: "how-to"
---

# Configure team-level settings

Team configuration overrides global and type defaults for repositories created by or for a specific team.

## Create the team configuration file

Create `teams/{team-name}/config.toml` in the `.reporoller` metadata repository:

```bash
mkdir -p teams/backend-team
touch teams/backend-team/config.toml
```

The directory name must match the team slug exactly.

## Example: backend-team

```toml
# teams/backend-team/config.toml

[repository]
discussions = false
projects    = true

[pull_requests]
required_approving_review_count = 2
allow_auto_merge                = true

[[labels]]
name        = "performance"
color       = "f9d0c4"
description = "Performance improvement or regression"

[[rulesets]]
name        = "backend-ci-checks"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_status_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/lint" },
  { context = "security/cargo-audit" }
]
```

## Settings you can override

Team configuration supports the same sections as global configuration:

- `[repository]` — feature toggles, security settings
- `[pull_requests]` — merge settings, review requirements
- `[branch_protection]` — default branch, protection rules
- `[actions]` — GitHub Actions permissions
- `[push]` — push restriction settings
- `[[labels]]` — additive (merged with global and type labels)
- `[[rulesets]]` — additive (all rulesets from all levels are applied)
- `[[webhooks]]` — additive (all webhooks from all levels are applied)
- `[[environments]]` — additive (all environments from all levels are applied)
- `[[github_apps]]` — additive (all GitHub Apps from all levels are applied)
- `[[custom_properties]]` — additive (all custom properties from all levels are applied)
- `[[naming_rules]]` — additive (all naming rules from all levels are applied)
- `[notifications]` — inline outbound webhook configuration

> **Note:** `[[default_teams]]` and `[[default_collaborators]]` are **not** available at team level. Use `[[rulesets]]` for access governance instead.

## Naming rules

The directory name under `teams/` must exactly match the GitHub team slug. To find a team's slug: on GitHub go to **Organisation settings → Teams**, find the team, and read the URL: `github.com/orgs/myorg/teams/{slug}`.

## Apply changes

Commit and push the file. Changes take effect on the next repository creation — no restart required.

## Related reference

- [Team configuration schema](../../reference/configuration/team-config.md)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
