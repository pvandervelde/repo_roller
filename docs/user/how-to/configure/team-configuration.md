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
has_discussions = false
has_projects    = true

[pull_requests]
required_approving_review_count = 2
allow_auto_merge                = true
dismiss_stale_reviews_on_push   = true

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
required_checks = [
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
- `[[labels]]` — additive (merged with global and type labels)
- `[[default_teams]]` — additive (merged with global teams)
- `[[default_collaborators]]` — additive (merged with global collaborators)
- `[[rulesets]]` — additive (all rulesets from all levels are applied)
- `[[webhooks]]` — additive (all webhooks from all levels are applied)

## Naming rules

The directory name under `teams/` must exactly match the GitHub team slug. To find a team's slug: on GitHub go to **Organisation settings → Teams**, find the team, and read the URL: `github.com/orgs/myorg/teams/{slug}`.

## Apply changes

Commit and push the file. Changes take effect on the next repository creation — no restart required.

## Related reference

- [Team configuration schema](../../reference/configuration/team-config.md)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
