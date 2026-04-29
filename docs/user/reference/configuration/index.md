---
title: "Configuration Reference"
description: "Overview of the configuration file system: locations, loading order, merge semantics."
audience: "platform-engineer"
type: "reference"
---

# Configuration Reference

RepoRoller resolves configuration from a four-level hierarchy, reading TOML files from the `.reporoller` metadata repository and from individual template repositories.

## File locations

| Level | File path | Scope |
|---|---|---|
| Global | `global/defaults.toml` | All repositories in the organisation |
| Repository type | `types/{type-name}/config.toml` | All repositories of that type |
| Team | `teams/{team-name}/config.toml` | All repositories created by or for that team |
| Template | `.reporoller/template.toml` (inside the template repo) | Repositories created from that template |
| Notifications (any level) | `global/notifications.toml`, `teams/{name}/notifications.toml`, or `notifications.toml` inside template repo | Outbound webhooks for the corresponding scope |

## Loading order and precedence

```
Template  (highest — overrides all below)
   ↑
Team
   ↑
Repository type
   ↑
Global    (lowest — base defaults)
   ↑
System    (built-in fallback defaults)
```

Higher levels override lower levels for scalar settings.

## Merge semantics

**Scalar settings** (e.g. `has_wiki`, `required_approving_review_count`) are **replaced** by the higher level. The highest level that sets the value wins.

**Array sections** (e.g. `[[labels]]`, `[[rulesets]]`, `[[webhooks]]`) are **additive** — entries from all levels are combined. When the same `url`+`event_type` pair appears in multiple notification levels, it is deduplicated (one delivery per unique pair).

**Override control**: any scalar setting can be locked by setting `override_allowed = false`. Attempts to override a locked setting at a higher level cause a validation error and prevent the repository from being created.

```toml
# global/defaults.toml — lock this setting org-wide
[repository]
security_advisories = { value = true, override_allowed = false }
```

## Configuration schema pages

| Schema | Reference |
|---|---|
| Global defaults | [global-config.md](global-config.md) |
| Repository type | [type-config.md](type-config.md) |
| Team | [team-config.md](team-config.md) |
| Template | [template-config.md](template-config.md) |
