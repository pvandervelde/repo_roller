---
title: "Configure labels for a template"
description: "Add GitHub issue and PR labels that will be created on every repository built from this template."
audience: "platform-engineer"
type: "how-to"
---

# Configure labels for a template

Add `[[labels]]` sections to `.reporoller/template.toml` to define labels that will be applied to every repository created from this template.

## Add labels to template.toml

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
name        = "breaking-change"
color       = "e11d48"
description = "API or interface change that is not backwards compatible"

[[labels]]
name        = "dependencies"
color       = "0075ca"
description = "Dependency version updates"

[[labels]]
name        = "documentation"
color       = "0075ca"
description = "Improvements or additions to documentation"
```

## Field reference

| Field | Type | Required | Constraints |
|---|---|---|---|
| `name` | string | Yes | Maximum 50 characters |
| `color` | string | Yes | Exactly 6 lowercase hexadecimal characters, no `#` prefix |
| `description` | string | No | Maximum 100 characters |

## Colour format rules

The `color` field must be exactly 6 lowercase hexadecimal characters **without** the leading `#`:

```toml
color = "d73a4a"   # correct
color = "#d73a4a"  # incorrect — do not include the hash
color = "D73A4A"   # incorrect — must be lowercase
color = "d73"      # incorrect — must be 6 characters
```

## Label accumulation

Labels from the template are merged with labels defined at the global, type, and team levels. All unique labels are applied. If two levels define a label with the same name, the last definition wins (template has highest precedence).

## Related guides

- [Configure repository labels](../configure/labels.md) — labels at global, type, and team level
- [Label schema](../../reference/template-authoring/label-schema.md) — full field reference
