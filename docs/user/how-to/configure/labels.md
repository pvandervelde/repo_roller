---
title: "Configure repository labels"
description: "Define GitHub issue and PR labels to be applied automatically to created repositories."
audience: "platform-engineer"
type: "how-to"
---

# Configure repository labels

Add `[[labels]]` entries to any configuration file — global, type, team, or template level. All labels from all levels are merged and applied to the created repository.

## Add labels to a configuration file

```toml
[[labels]]
name        = "bug"
color       = "d73a4a"
description = "Something isn't working"

[[labels]]
name        = "enhancement"
color       = "a2eeef"
description = "New feature or request"

[[labels]]
name        = "documentation"
color       = "0075ca"
description = "Improvements or additions to documentation"

[[labels]]
name        = "good first issue"
color       = "7057ff"
description = "Good for newcomers to the project"

[[labels]]
name        = "help wanted"
color       = "008672"
description = "Extra attention is needed"

[[labels]]
name        = "security"
color       = "e11d48"
description = "Security vulnerability or improvement"

[[labels]]
name        = "dependencies"
color       = "0366d6"
description = "Dependency version updates"
```

## Field reference

| Field | Type | Required | Constraints |
|---|---|---|---|
| `name` | string | Yes | Maximum 50 characters |
| `color` | string | Yes | Exactly 6 lowercase hex characters, no `#` prefix |
| `description` | string | No | Maximum 100 characters |

## Colour format

```toml
color = "d73a4a"   # correct — 6 lowercase hex chars, no hash
color = "#d73a4a"  # incorrect — do not include hash
color = "D73A4A"   # incorrect — must be lowercase
```

## Label merging

Labels are additive across all configuration levels. If two levels define a label with the same name, the definition from the more-specific level wins (Template > Team > Type > Global).

## Related reference

- [Label schema](../../reference/template-authoring/label-schema.md)
- [Template configuration schema](../../reference/configuration/template-config.md)
