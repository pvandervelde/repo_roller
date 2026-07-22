---
title: "Repository type configuration schema"
description: "Complete field reference for types/{type-name}/config.toml in the metadata repository."
audience: "platform-engineer"
type: "reference"
---

# Repository type configuration schema

**File:** `types/{type-name}/config.toml` in the metadata repository.

Applies to every repository of a given type, regardless of which template or team is involved. Overrides values from global defaults (unless the global setting is locked).

The file name determines the type name (e.g. `types/library/config.toml` defines the `library` type).

---

## Available sections

Repository type configuration supports the same sections as global configuration with the following notes:

| Section | Available | Notes |
|---|---|---|
| `[repository]` | Yes | Overrides global defaults |
| `[pull_requests]` | Yes | Overrides global defaults |
| `[branch_protection]` | Yes | Overrides global defaults |
| `[actions]` | No | Not available at type level |
| `[push]` | No | Not available at type level |
| `[[labels]]` | Yes | **Additive** — combined with global labels |
| `[[default_teams]]` | No | Not available at type level |
| `[[rulesets]]` | Yes | **Additive** — combined with global rulesets |
| `[[webhooks]]` | Yes | **Additive** — combined with global webhooks |
| `[[environments]]` | Yes | **Additive** — combined with global environments |
| `[[github_apps]]` | Yes | **Additive** — combined with global GitHub Apps |
| `[[custom_properties]]` | Yes | **Additive** — combined with global custom properties |
| `[[naming_rules]]` | Yes | **Additive** — combined with global naming rules |
| `[[default_collaborators]]` | No | Not available at type level |
| `[permissions]` | Yes | Type-level permission policy (`required` and `restricted_types` fields) |
| `[notifications]` | Yes | Inline outbound webhook configuration |

For field-level documentation see [global-config.md](global-config.md). All field names, types, and `override_allowed` semantics are identical.

The type-level `[permissions]` section supports `required` (array of `PermissionGrantConfig`) and `restricted_types` (array of permission type strings). See [global-config.md](global-config.md#permissions--access-level-ceilings) for `PermissionGrantConfig` field details.

---

## Example

```toml
# types/library/config.toml

[repository]
wiki = false              # Libraries use README instead of wiki
security_advisories = true
vulnerability_reporting = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true

[[labels]]
name = "breaking-change"
color = "d73a4a"
description = "Breaking API changes"

[[labels]]
name = "semver-major"
color = "b60205"
description = "Requires a major version bump"

[[rulesets]]
name = "library-main-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
```
