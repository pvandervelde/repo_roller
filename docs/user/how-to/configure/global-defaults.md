---
title: "Set organisation-wide defaults"
description: "Edit global/defaults.toml in the metadata repository to apply baseline configuration to every repository in the organisation."
audience: "platform-engineer"
type: "how-to"
---

# Set organisation-wide defaults

Edit `global/defaults.toml` in the `.reporoller` metadata repository. These settings apply to every repository RepoRoller creates, unless overridden at a higher configuration level.

## Repository feature defaults

```toml
[repository]
has_issues      = true
has_projects    = false
has_wiki        = false
has_discussions = false
default_branch  = "main"
allow_forking   = false

security_advisories      = true
vulnerability_reporting  = true
```

## Pull request defaults

```toml
[pull_requests]
allow_merge_commit  = false
allow_squash_merge  = true
allow_rebase_merge  = false
allow_auto_merge    = false

required_approving_review_count   = 1
require_code_owner_reviews        = false
dismiss_stale_reviews_on_push     = true
require_conversation_resolution   = true
```

## Default labels

```toml
[[labels]]
name        = "bug"
color       = "d73a4a"
description = "Something isn't working"

[[labels]]
name        = "enhancement"
color       = "a2eeef"
description = "New feature or request"
```

## Default teams

```toml
[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true

[[default_teams]]
slug         = "platform"
access_level = "write"
```

## Default collaborators

```toml
[[default_collaborators]]
username     = "ci-service-account"
access_level = "read"
locked       = true
```

## Access level ceiling

```toml
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
```

## Override controls

To prevent type, team, and template configs from changing a setting, use the `override_allowed` form:

```toml
[repository]
security_advisories = { value = true, override_allowed = false }
```

Any attempt to override this at a higher level causes a configuration validation error.

## Apply changes

Changes to `global/defaults.toml` take effect immediately on the next repository creation. No restart is required. Commit and push the file to the metadata repository's default branch.

## Related reference

- [Global configuration schema](../../reference/configuration/global-config.md)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
