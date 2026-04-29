---
title: "Label schema"
description: "Field constraints for [[labels]] entries in all configuration files."
audience: "platform-engineer"
type: "reference"
---

# Label schema

The `[[labels]]` section appears in global, type, team, and template configuration files. All levels share the same schema.

## Fields

| Field | TOML type | Required | Constraints | Description |
|---|---|---|---|---|
| `name` | string | Yes | 1–50 characters | Label name shown in GitHub. Must be unique within the repository. |
| `color` | string | Yes | Exactly 6 lowercase hexadecimal characters, no `#` prefix | Background colour of the label badge |
| `description` | string | No | 0–100 characters | Short description shown as a tooltip on GitHub |

## Valid examples

```toml
[[labels]]
name        = "bug"
color       = "d73a4a"
description = "Something isn't working"

[[labels]]
name  = "enhancement"
color = "a2eeef"

[[labels]]
name        = "dependencies"
color       = "0075ca"
description = "Pull requests that update a dependency file"
```

## Invalid examples

```toml
# Wrong: color has '#' prefix
[[labels]]
name  = "bug"
color = "#d73a4a"

# Wrong: color is uppercase
[[labels]]
name  = "bug"
color = "D73A4A"

# Wrong: color is 3 characters (not 6)
[[labels]]
name  = "bug"
color = "f00"

# Wrong: name exceeds 50 characters
[[labels]]
name  = "this-label-name-is-far-too-long-and-will-be-rejected-by-github"
color = "d73a4a"
```

## Additive behaviour

Labels are **additive** across configuration levels. If the `bug` label is defined at global level and also in the template, both definitions are processed. If they have the same `name`, the template-level definition takes precedence for that label (the template wins for `color` and `description`).

## Standard label set recommendation

A minimal starting set for any repository:

```toml
[[labels]]
name = "bug"
color = "d73a4a"
description = "Something isn't working"

[[labels]]
name = "enhancement"
color = "a2eeef"
description = "New feature or request"

[[labels]]
name = "documentation"
color = "0075ca"
description = "Improvements or additions to documentation"

[[labels]]
name = "good first issue"
color = "7057ff"
description = "Good for newcomers"

[[labels]]
name = "help wanted"
color = "008672"
description = "Extra attention is needed"
```
