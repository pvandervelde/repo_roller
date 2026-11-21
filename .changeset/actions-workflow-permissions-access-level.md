---
"config_manager": minor
"github_client": minor
"repo_roller_core": minor
---

Add configuration support for GitHub Actions "Workflow permissions" and "Access" settings
(#276, #277).

RepoRoller can now manage two additional GitHub Actions repository settings that were
previously left at GitHub's defaults on every created repository:

- **Default workflow permissions** — the permissions granted to the automatic
  `GITHUB_TOKEN` for workflow runs (`"read"` or `"write"`), and whether workflows are
  allowed to approve pull request reviews. Maps to
  `PUT /repos/{owner}/{repo}/actions/permissions/workflow`.
- **Access** — which other repositories in the organisation or enterprise may use this
  repository's reusable Actions workflows (`"none"`, `"organization"`, or `"enterprise"`).
  Maps to `PUT /repos/{owner}/{repo}/actions/permissions/access`.

Both are configured through three new `[actions]` fields —
`default_workflow_permissions`, `can_approve_pull_request_reviews`, and `access_level` —
using the same `OverridableValue` override-policy pattern as every other `ActionSettings`
field, and are available at all four configuration levels (global, repository type, team,
and template). As part of this change, the `[actions]` section itself is now also
available at the repository-type and template configuration levels, where it previously
could only be set globally or per-team.

```toml
[actions]
default_workflow_permissions     = { value = "read", override_allowed = false }
can_approve_pull_request_reviews = { value = false, override_allowed = true }
access_level                     = { value = "organization", override_allowed = true }
```

`access_level = "organization"` or `"enterprise"` is most useful on a repository type or
template specifically meant for custom GitHub Actions, since that is what lets other
repositories consume the action defined in a repository created from it.

This is purely additive: existing `defaults.toml`, repository-type, team, and template
config files that don't set these fields are unaffected, and all three new fields are
optional.
