---
title: "Configure team and collaborator permissions"
description: "Control which GitHub teams and individual collaborators are added to created repositories, and what access level they receive."
audience: "platform-engineer"
type: "how-to"
---

# Configure team and collaborator permissions

RepoRoller can automatically assign GitHub teams and individual collaborators to every repository it creates. Entries are collected from the configuration hierarchy and merged before being applied to GitHub.

## Add default teams and collaborators at the global level

In `global/defaults.toml`:

```toml
# Every repository gets this team at admin level (locked — cannot be changed by templates or requests).
[[default_teams]]
slug         = "security-ops"
access_level = "admin"
locked       = true

# Platform team gets write access; templates may upgrade this.
[[default_teams]]
slug         = "platform"
access_level = "write"

# CI service account gets read-only access (locked).
[[default_collaborators]]
username     = "ci-service-account"
access_level = "read"
locked       = true

# Cap what creation requests can grant.
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
```

## Upgrade a team's access for a specific template

In `.reporoller/template.toml` inside the template repository:

```toml
# Upgrade platform to maintain level for repositories using this template.
[[teams]]
slug         = "platform"
access_level = "maintain"

# Add the owning team.
[[teams]]
slug         = "payments-team"
access_level = "write"
```

> **Note:** A template may only *upgrade* teams, never *downgrade* them. Attempting to lower an access level that was set at a higher (more-permissive) configuration level causes a validation error and blocks creation.

## Access levels

Valid values for `access_level`:

| Level | Description |
|---|---|
| `none` | Removes existing access |
| `read` | View code, open issues |
| `triage` | Manage issues and PRs (no write) |
| `write` | Push code, manage issues |
| `maintain` | Repository settings (no admin actions) |
| `admin` | Full repository administration |

## Locked entries

Setting `locked = true` on a `[[default_teams]]` or `[[default_collaborators]]` entry prevents any higher-level configuration or incoming creation request from changing that entry's access level.

If a template tries to change a locked entry, the creation request fails with a `PermissionLockedNotAllowed` error. If a human creation request tries to change a locked entry, the change is silently skipped and a warning is logged.

## No-demotion rule

An entry without `locked = true` may be *upgraded* by a higher level but never *downgraded*. A configuration-level demotion attempt fails with `PermissionDemotionNotAllowed`. A request-level demotion attempt is silently skipped with a warning.

## Access level ceiling

The `[permissions]` section in global configuration caps what a creation *request* (API or CLI) may grant. It does not cap what configuration files themselves can grant.

```toml
[permissions]
max_team_access_level         = "maintain"
max_collaborator_access_level = "write"
```

A request attempting to grant `admin` to a new team is capped at `maintain` with a warning.

## Merge algorithm summary

```
Level       Source                      Table key
─────────────────────────────────────────────────────────────────
1 Global    global/defaults.toml        [[default_teams]]
2 Type      types/{type}/config.toml    [[default_teams]]
3 Template  .reporoller/template.toml   [[teams]]
4 Request   API / CLI call              teams: {slug: level}
```

Each level refines the map produced by the previous levels, subject to the rules above.

## Related reference

- [Global configuration schema](../../reference/configuration/global-config.md)
- [Template configuration schema](../../reference/configuration/template-config.md)
- [The permissions model](../../explanation/permissions-model.md)
