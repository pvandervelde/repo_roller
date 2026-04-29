---
title: "Configure branch protection rules"
description: "Add rulesets to the metadata repository or a template to enforce branch and tag protection policies."
audience: "platform-engineer"
type: "how-to"
---

# Configure branch protection rules

RepoRoller uses GitHub **Rulesets** (not the older branch-protection API) to protect branches and tags. Rulesets are defined in `[[rulesets]]` sections and are **additive** — every ruleset from every configuration level is applied to the created repository.

## Anatomy of a ruleset

```toml
[[rulesets]]
name        = "main-protection"   # Must be unique within the created repository
target      = "branch"            # "branch" or "tag"
enforcement = "active"            # "active", "evaluate", or "disabled"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]     # Branch patterns to protect
exclude = []                      # Patterns to exclude

[[rulesets.rules]]
type = "deletion"                 # Rule type
```

## Example 1: Prevent deletion of main

```toml
[[rulesets]]
name        = "prevent-main-deletion"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "non_fast_forward"
```

## Example 2: Require pull requests on main

```toml
[[rulesets]]
name        = "require-pr-on-main"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review       = true
dismiss_stale_reviews_on_push   = true
allowed_merge_methods           = ["squash"]
```

## Example 3: Require CI status checks

```toml
[[rulesets]]
name        = "require-ci"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "security/scan" }
]
```

## Ruleset rule types

| Type | Description |
|---|---|
| `deletion` | Prevents deletion of matching refs |
| `non_fast_forward` | Prevents force pushes |
| `required_linear_history` | Requires linear history (no merge commits) |
| `update` | Prevents updates to matching refs |
| `pull_request` | Requires a pull request before merging |
| `required_status_checks` | Requires specific checks to pass |
| `required_signatures` | Requires signed commits |
| `creation` | Prevents creation of new matching refs |

## Ruleset hierarchy

Rulesets from all levels are applied independently. A repository created with a global, type, team, and template ruleset will have four separate rulesets:

```
Global:   prevent-deletion        ─┐
Type:     library-protection       ├─ All four applied to the repository
Team:     backend-ci-checks        │
Template: microservice-pr-checks  ─┘
```

> **Note:** If two configuration levels define a `[[rulesets]]` with the same `name`, two separate rulesets are created on GitHub. A warning is logged. Avoid duplicate names across levels.

## Evaluate mode for testing

Set `enforcement = "evaluate"` to apply a ruleset in "dry-run" mode. GitHub reports what would have been blocked but does not block anything. Switch to `"active"` after validating the ruleset.

## Related reference

- [Repository rulesets in configuration guide](../../reference/configuration/global-config.md)
- [GitHub Rulesets documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)
