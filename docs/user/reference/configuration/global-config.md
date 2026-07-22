---
title: "Global configuration schema"
description: "Complete field reference for global/defaults.toml in the metadata repository."
audience: "platform-engineer"
type: "reference"
---

# Global configuration schema

**File:** `global/defaults.toml` in the metadata repository.

Applies to every repository created in the organisation. Provides baseline defaults that all higher levels can override (unless `override_allowed = false` is set).

---

## `[repository]` — repository feature settings

| Field | TOML type | Default | override_allowed default | Description |
|---|---|---|---|---|
| `issues` | bool | `true` | `true` | Enable GitHub Issues |
| `projects` | bool | `false` | `true` | Enable GitHub Projects |
| `wiki` | bool | `true` | `true` | Enable GitHub Wiki |
| `discussions` | bool | `false` | `true` | Enable GitHub Discussions |
| `pages` | bool | `false` | `true` | Enable GitHub Pages |
| `security_advisories` | bool | `false` | `true` | Enable private security advisories |
| `vulnerability_reporting` | bool | `false` | `true` | Enable vulnerability reporting for the repo |
| `auto_close_issues` | bool | `false` | `true` | Automatically close stale issues |

> **Note:** `default_branch` is configured under `[branch_protection]`, not `[repository]`. See the `[branch_protection]` section below.

To lock a setting, use the inline table form:

```toml
[repository]
security_advisories = { value = true, override_allowed = false }
```

---

## `[pull_requests]` — pull request and merge settings

| Field | TOML type | Default | Description |
|---|---|---|---|
| `allow_merge_commit` | bool | `true` | Allow merge commits |
| `allow_squash_merge` | bool | `true` | Allow squash merges |
| `allow_rebase_merge` | bool | `true` | Allow rebase merges |
| `delete_branch_on_merge` | bool | `false` | Delete head branch after merge |
| `required_approving_review_count` | integer | `0` | Minimum number of approving reviews before merging |
| `require_code_owner_reviews` | bool | `false` | Require review from a Code Owner |
| `require_conversation_resolution` | bool | `false` | Require all conversations to be resolved before merging |
| `allow_auto_merge` | bool | `false` | Allow auto-merge when all checks pass |
| `merge_commit_title` | string | `"MERGE_MESSAGE"` | Title format for merge commits: `"MERGE_MESSAGE"` or `"PR_TITLE"` |
| `merge_commit_message` | string | `"PR_BODY"` | Message format for merge commits: `"PR_BODY"`, `"COMMIT_MESSAGES"`, or `"BLANK"` |
| `squash_merge_commit_title` | string | `"COMMIT_OR_PR_TITLE"` | Title format for squash commits |
| `squash_merge_commit_message` | string | `"COMMIT_MESSAGES"` | Message format for squash commits: `"PR_BODY"`, `"COMMIT_MESSAGES"`, or `"BLANK"` |

> **Note:** `dismiss_stale_reviews_on_push` is not a `[pull_requests]` field. Use `dismiss_stale_reviews_on_push` inside a `[[rulesets]]` rule of type `pull_request`. See `[[rulesets]]` below.

---

## `[[labels]]` — default repository labels

Defines labels to create on every repository. Entries from all config levels are combined.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Label name (max 50 characters) |
| `color` | string | Yes | 6-character lowercase hex colour without `#` (e.g. `"d73a4a"`) |
| `description` | string | No | Short description (max 100 characters) |

```toml
[[labels]]
name = "bug"
color = "d73a4a"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "a2eeef"
description = "New feature or request"
```

---

## `[[default_teams]]` — default team access

Teams assigned to every repository by default.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `slug` | string | Yes | GitHub team slug |
| `access_level` | string | Yes | `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, or `"admin"` |
| `locked` | bool | No (`false`) | When `true`, no higher config level or request may change this entry. Hard error at config-resolution time; silent skip at request time. |

```toml
[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true

[[default_teams]]
slug         = "platform"
access_level = "write"
```

---

## `[[default_collaborators]]` — default individual access

Individual GitHub users assigned to every repository by default.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `username` | string | Yes | GitHub username |
| `access_level` | string | Yes | Same values as for teams |
| `locked` | bool | No (`false`) | Same semantics as for teams |

```toml
[[default_collaborators]]
username     = "ci-service-account"
access_level = "read"
locked       = true
```

---

## `[permissions]` — access level ceilings

Caps the maximum access level that a **request** may grant. Config-established entries (from global, type, or template config) are not subject to these ceilings.

| Field | TOML type | Default | Description |
|---|---|---|---|
| `max_team_access_level` | string | `"admin"` | Maximum team access level a request may specify |
| `max_collaborator_access_level` | string | `"admin"` | Maximum collaborator access level a request may specify |

The `baseline` and `restrictions` arrays allow fine-grained permission grant policies:

| Field | TOML type | Description |
|---|---|---|
| `baseline` | array of `PermissionGrantConfig` | Minimum permissions all repositories must have (floor) |
| `restrictions` | array of `PermissionGrantConfig` | Maximum permissions allowed; requests exceeding these are denied (ceiling) |

Each `PermissionGrantConfig` entry has:

| Field | TOML type | Description |
|---|---|---|
| `permission_type` | string | One of: `"pull"`, `"triage"`, `"push"`, `"maintain"`, `"admin"` |
| `level` | string | One of: `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, `"admin"` |
| `scope` | string | One of: `"repository"`, `"team"`, `"user"`, `"github_app"` |

```toml
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"

[[permissions.baseline]]
permission_type = "push"
level           = "write"
scope           = "team"

[[permissions.restrictions]]
permission_type = "admin"
level           = "admin"
scope           = "user"
```

---

## `[[rulesets]]` — branch and tag protection

Rulesets are additive — entries from all config levels are applied to the repository.

See [how-to/configure/branch-protection-rules.md](../../how-to/configure/branch-protection-rules.md) for practical examples and field details.

---

## `[[webhooks]]` — repository webhooks

Webhooks that GitHub fires for events inside the repository (not to be confused with outbound notification webhooks that RepoRoller fires).

| Field | TOML type | Required | Description |
|---|---|---|---|
| `url` | string | Yes | Endpoint URL |
| `content_type` | string | Yes | `"json"` or `"form"` |
| `secret` | string | No | Shared secret for request signing |
| `events` | array of string | Yes | GitHub event types (e.g. `["push", "pull_request"]`) |
| `active` | bool | No (`true`) | Whether the webhook is active |

---

## `[branch_protection]` — branch protection settings

Controls branch protection rules for the default branch and additional branches.

| Field | TOML type | Default | override_allowed default | Description |
|---|---|---|---|---|
| `default_branch` | string | `"main"` | `true` | Default branch name |
| `require_pull_request_reviews` | bool | `false` | `true` | Require pull request reviews before merging |
| `required_approving_review_count` | integer | `0` | `true` | Required number of approving reviews |
| `dismiss_stale_reviews` | bool | `false` | `true` | Dismiss stale reviews when new commits are pushed |
| `require_code_owner_reviews` | bool | `false` | `true` | Require review from code owners |
| `require_status_checks` | bool | `false` | `true` | Require status checks to pass before merging |
| `required_status_checks_list` | array of string | `[]` | — | Required status check context names |
| `strict_required_status_checks` | bool | `false` | `true` | Require branches to be up to date before merging |
| `restrict_pushes` | bool | `false` | `true` | Restrict who can push to matching branches |
| `allow_force_pushes` | bool | `false` | `true` | Allow force pushes |
| `allow_deletions` | bool | `false` | `true` | Allow branch deletions |
| `additional_protected_patterns` | array of string | `[]` | — | Additional branch name patterns to protect |

```toml
[branch_protection]
default_branch                 = "main"
require_pull_request_reviews   = true
required_approving_review_count = 2
dismiss_stale_reviews          = true
require_code_owner_reviews     = true
require_status_checks          = true
required_status_checks_list    = ["ci/build", "ci/test"]
strict_required_status_checks  = true
allow_force_pushes             = { value = false, override_allowed = false }
allow_deletions                = { value = false, override_allowed = false }
```

---

## `[actions]` — GitHub Actions settings

Controls GitHub Actions permissions for repositories.

| Field | TOML type | Default | override_allowed default | Description |
|---|---|---|---|---|
| `enabled` | bool | `true` | `true` | Enable GitHub Actions |
| `allowed_actions` | string | `"all"` | `true` | Actions permissions: `"all"`, `"local_only"`, or `"selected"` |
| `github_owned_allowed` | bool | `true` | `true` | Allow GitHub-owned actions when `allowed_actions = "selected"` |
| `verified_allowed` | bool | `false` | `true` | Allow verified creator actions when `allowed_actions = "selected"` |
| `patterns_allowed` | array of string | `[]` | — | List of allowed action patterns when `allowed_actions = "selected"` |

```toml
[actions]
enabled          = true
allowed_actions  = "selected"
github_owned_allowed = true
verified_allowed = false
patterns_allowed = ["actions/*", "myorg/*"]
```

---

## `[push]` — push restriction settings

Controls how many branches and tags can be pushed at once.

| Field | TOML type | Default | override_allowed default | Description |
|---|---|---|---|---|
| `max_branches_per_push` | integer | — | `true` | Maximum number of branches that can be pushed at once |
| `max_tags_per_push` | integer | — | `true` | Maximum number of tags that can be pushed at once |

```toml
[push]
max_branches_per_push = 5
max_tags_per_push     = 3
```

---

## `[repository_visibility]` — visibility policy

Controls the allowed visibility values for repositories in the organisation.

| Field | TOML type | Default | Description |
|---|---|---|---|
| `enforcement_level` | string | `"unrestricted"` | One of: `"unrestricted"`, `"required"`, `"restricted"` |
| `required_visibility` | string | — | When `enforcement_level = "required"`: `"private"`, `"public"`, or `"internal"` |
| `restricted_visibilities` | array of string | — | When `enforcement_level = "restricted"`: visibility values that are **not** allowed |

```toml
# All repositories must be private
[repository_visibility]
enforcement_level    = "required"
required_visibility  = "private"

# Or: public repositories are not allowed
[repository_visibility]
enforcement_level        = "restricted"
restricted_visibilities  = ["public"]
```

---

## `[[naming_rules]]` — repository naming rules

Naming rules are **additive** — all rules from all configuration levels are combined and every rule must be satisfied.

| Field | TOML type | Description |
|---|---|---|
| `description` | string | Human-readable explanation shown in error messages |
| `allowed_pattern` | string | Regex pattern the full repository name must match |
| `forbidden_patterns` | array of string | Regex patterns the name must **not** match (substring by default) |
| `reserved_words` | array of string | Exact strings that cannot be used as the full name (case-insensitive) |
| `required_prefix` | string | Required prefix (case-sensitive) |
| `required_suffix` | string | Required suffix (case-sensitive) |
| `min_length` | integer | Minimum name length |
| `max_length` | integer | Maximum name length |

```toml
[[naming_rules]]
description     = "All repositories must use the org prefix"
required_prefix = "acme-"
allowed_pattern = "^acme-[a-z][a-z0-9-]*$"

[[naming_rules]]
description    = "Reserved words must not be used"
reserved_words = ["test", "demo", "temp", "tmp"]

[[naming_rules]]
description = "Repository names must be between 5 and 40 characters"
min_length  = 5
max_length  = 40
```

---

## `[[environments]]` — deployment environments

Environments are **additive** — entries from all config levels are applied.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `name` | string | Yes | Environment name |
| `protection_rules` | table | No | Protection rules (see below) |
| `deployment_branch_policy` | table | No | Branch policy (see below) |

`protection_rules` fields:

| Field | TOML type | Description |
|---|---|---|
| `required_reviewers` | array of string | User or team names that must approve deployments |
| `wait_timer` | integer | Minutes to wait before deployment can proceed |

`deployment_branch_policy` fields:

| Field | TOML type | Description |
|---|---|---|
| `protected_branches` | bool | When `true`, only protected branches can deploy |
| `custom_branch_patterns` | array of string | Branch name patterns allowed to deploy (when `protected_branches = false`) |

```toml
[[environments]]
name = "production"

[environments.protection_rules]
required_reviewers = ["platform-team", "security-ops"]
wait_timer         = 5

[environments.deployment_branch_policy]
protected_branches = true
```

---

## `[[github_apps]]` — GitHub App installations

GitHub Apps to install on created repositories.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `app_id` | integer | Yes | GitHub App ID |
| `permissions` | table of string → string | Yes | Map of permission scope to access level (e.g. `{contents = "read"}`) |

```toml
[[github_apps]]
app_id = 12345

[github_apps.permissions]
contents      = "read"
pull_requests = "write"
```

---

## `[[custom_properties]]` — repository custom properties

Custom properties to set on created repositories.

| Field | TOML type | Required | Description |
|---|---|---|---|
| `property_name` | string | Yes | Custom property name |
| `value` | string, bool, or array of string | Yes | Property value |

```toml
[[custom_properties]]
property_name = "team"
value         = "platform"

[[custom_properties]]
property_name = "compliance-level"
value         = ["soc2", "iso27001"]

[[custom_properties]]
property_name = "public-facing"
value         = true
```

---

## `[notifications]` — inline notifications configuration

In addition to a separate `notifications.toml` file, outbound notification webhooks can be configured inline in any configuration file.

```toml
[[notifications.outbound_webhooks]]
url         = "https://audit.myorg.example/hooks/reporoller"
secret      = "REPOROLLER_AUDIT_SECRET"
events      = ["*"]
description = "Corporate audit log"
```

See [how-to/configure/outbound-notifications.md](../../how-to/configure/outbound-notifications.md) for full field documentation.
