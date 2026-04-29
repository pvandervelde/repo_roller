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
| `has_issues` | bool | `true` | `true` | Enable GitHub Issues |
| `has_projects` | bool | `false` | `true` | Enable GitHub Projects |
| `has_wiki` | bool | `true` | `true` | Enable GitHub Wiki |
| `has_discussions` | bool | `false` | `true` | Enable GitHub Discussions |
| `security_advisories` | bool | `false` | `true` | Enable private security advisories |
| `vulnerability_reporting` | bool | `false` | `true` | Enable vulnerability reporting for the repo |
| `default_branch` | string | `"main"` | `true` | Default branch name |
| `allow_forking` | bool | `true` | `true` | Allow forking for private repos |
| `is_template` | bool | `false` | `true` | Mark the repository as a GitHub template repository |

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
| `required_approving_review_count` | integer | `0` | Minimum number of approving reviews before merging |
| `require_code_owner_reviews` | bool | `false` | Require review from a Code Owner |
| `dismiss_stale_reviews_on_push` | bool | `false` | Dismiss approvals when new commits are pushed |
| `require_conversation_resolution` | bool | `false` | Require all conversations to be resolved before merging |
| `allow_auto_merge` | bool | `false` | Allow auto-merge when all checks pass |
| `merge_commit_title` | string | `"MERGE_MESSAGE"` | Title format for merge commits: `"MERGE_MESSAGE"` or `"PR_TITLE"` |
| `squash_merge_commit_title` | string | `"COMMIT_OR_PR_TITLE"` | Title format for squash commits |

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

```toml
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
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
| `name` | string | Yes | Identifier for the webhook |
| `url` | string | Yes | Endpoint URL |
| `content_type` | string | Yes | `"json"` or `"form"` |
| `secret` | string | No | Shared secret for request signing |
| `events` | array of string | Yes | GitHub event types (e.g. `["push", "pull_request"]`) |
| `active` | bool | No (`true`) | Whether the webhook is active |
| `insecure_ssl` | bool | No (`false`) | Disable SSL verification (not recommended in production) |
