---
title: "How configuration is resolved"
description: "How the four configuration levels are merged and what override_allowed controls."
audience: "platform-engineer"
type: "explanation"
---

# How configuration is resolved

## The four levels

RepoRoller merges configuration from four levels, from lowest to highest precedence:

```
Template  ← highest (overrides all below)
Team
Repository type
Global    ← lowest
```

Higher levels win for scalar settings. If `global/defaults.toml` sets `has_wiki = true` and `types/library/config.toml` sets `has_wiki = false`, repositories of type `library` have wiki disabled.

## Scalar settings: last-wins override

Any scalar setting (a boolean, integer, or string) is replaced by the higher-level value when that level sets it. A level that does not set a value has no effect — it does not "reset" the value to a default.

This means a template can change the number of required reviewers without affecting anything else. It only needs to declare the settings it wants to differ from the layers below.

## Additive sections

Some sections are **additive** — entries from all levels are combined, not replaced:

- `[[labels]]` — labels from all levels are all applied
- `[[rulesets]]` — all rulesets from all levels are applied to the repository
- `[[webhooks]]` — all webhooks from all levels are applied
- `[[outbound_webhooks]]` in `notifications.toml` — all endpoints fire (with deduplication by URL + event type)

If the same ruleset name appears at multiple levels, two separate independent rulesets are created on the repository (they are not merged). A warning is logged when this happens.

## Override controls

Any scalar setting can be locked at a lower level to prevent higher levels from changing it:

```toml
# global/defaults.toml
[repository]
security_advisories = { value = true, override_allowed = false }
```

When a template or team config tries to set `security_advisories = false`, RepoRoller raises a validation error and refuses to create the repository. This is the primary mechanism for enforcing organisation security policy.

Override controls are one-directional: a lower level can lock a value against being overridden, but a higher level cannot unlock a value that a lower level locked.

## "Why is my setting being ignored?"

Work through this checklist:

1. **Is the setting locked at a lower level?** Check `global/defaults.toml` for `override_allowed = false` on that field. If so, your override is blocked.

2. **Is a higher-level config also setting it?** A template setting can be overridden by nothing — it is the highest level. A team setting can only be overridden by a template. Check whether the template you are using also sets the same field.

3. **Is the section additive?** Labels, rulesets, and webhooks accumulate — setting a label at type level does not remove the global labels. If you see unexpected labels, they are probably from a different config level.

4. **Is the config file in the right location?** Type files must be at `types/{type-name}/config.toml`. Team files must be at `teams/{team-name}/config.toml`. A typo in the directory name means the file is never read.

5. **Was the right type or team passed in the creation request?** Type and team configuration only applies when the corresponding `--repository-type` or `--team` flag (or JSON field) is included in the creation request.
