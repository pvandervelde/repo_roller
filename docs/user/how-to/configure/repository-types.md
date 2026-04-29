---
title: "Configure a repository type"
description: "Create a repository type configuration file to apply shared defaults to a category of repositories."
audience: "platform-engineer"
type: "how-to"
---

# Configure a repository type

Repository types let you define named categories of repositories (for example `library`, `service`, `infrastructure`) and apply type-specific settings on top of the global defaults.

## Create the type configuration file

Create `types/{type-name}/config.toml` in the `.reporoller` metadata repository:

```bash
mkdir -p types/library
touch types/library/config.toml
```

## Example: library type

```toml
# types/library/config.toml

[repository]
has_wiki     = false  # Libraries use README, not wiki
has_projects = false
security_advisories     = true
vulnerability_reporting = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews      = true

[[labels]]
name        = "breaking-change"
color       = "d73a4a"
description = "Breaks the public API — requires a major version bump"

[[labels]]
name        = "semver-patch"
color       = "0075ca"
description = "Fix release — increment patch version"

[[default_teams]]
slug         = "library-reviewers"
access_level = "write"

[[rulesets]]
name        = "library-main-protection"
target      = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review       = true
```

## Naming rules

Directory names under `types/` become the repository type slug. Use lowercase with hyphens. The type slug is used in the `--repository-type` flag and in `repositoryType` API field.

## Apply changes

Commit and push `types/library/config.toml` to the metadata repository. The type becomes available immediately on the next request.

## Related reference

- [Repository type configuration schema](../../reference/configuration/type-config.md)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
