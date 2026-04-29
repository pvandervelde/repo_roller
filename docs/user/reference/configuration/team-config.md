---
title: "Team configuration schema"
description: "Complete field reference for teams/{team-name}/config.toml in the metadata repository."
audience: "platform-engineer"
type: "reference"
---

# Team configuration schema

**File:** `teams/{team-name}/config.toml` in the metadata repository.

Applies to repositories created by or for a specific team. Overrides global defaults and type configuration (unless locked). The directory name determines the team slug (e.g. `teams/backend-team/config.toml` applies to the `backend-team` team).

---

## Available sections

| Section | Available | Notes |
|---|---|---|
| `[repository]` | Yes | Overrides global and type |
| `[pull_requests]` | Yes | Overrides global and type |
| `[[labels]]` | Yes | **Additive** |
| `[[default_teams]]` | No | Use `[[rulesets]]` for access governance instead |
| `[[rulesets]]` | Yes | **Additive** |
| `[[webhooks]]` | Yes | **Additive** |
| `[permissions]` | No | Only at global level |

---

## Example

```toml
# teams/frontend-team/config.toml

[repository]
has_discussions = true    # Frontend team uses Discussions for design decisions

[pull_requests]
required_approving_review_count = 1   # Small team; 1 approval is sufficient
allow_auto_merge = true

[[labels]]
name = "ui"
color = "0e8a16"
description = "UI/UX changes"

[[labels]]
name = "accessibility"
color = "f9d0c4"
description = "Accessibility improvements"

[[rulesets]]
name = "frontend-ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/develop"]

[[rulesets.rules]]
type = "required_status_checks"
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/lint" },
  { context = "ci/visual-regression" }
]
```
