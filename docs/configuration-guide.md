# RepoRoller Configuration Guide

This guide explains how to configure RepoRoller using the hierarchical configuration system.

## Table of Contents

- [Overview](#overview)
- [Configuration Hierarchy](#configuration-hierarchy)
- [Configuration Files](#configuration-files)
  - [Global Configuration](#global-configuration)
  - [Repository Type Configuration](#repository-type-configuration)
  - [Team Configuration](#team-configuration)
  - [Template Configuration](#template-configuration)
- [Repository Settings](#repository-settings)
- [Pull Request Settings](#pull-request-settings)
- [Labels Configuration](#labels-configuration)
- [Webhooks Configuration](#webhooks-configuration)
- [Repository Rulesets](#repository-rulesets)
- [Override Controls](#override-controls)
- [Examples](#examples)

## Overview

RepoRoller uses a hierarchical configuration system that allows organizations to define baseline policies, teams to customize their workflows, and templates to specify their requirements. Configuration is stored in TOML files within a metadata repository (typically named `.reporoller-test` or `.reporoller`).

## Configuration Hierarchy

Configuration is resolved in the following order (highest precedence first):

1. **Template** - Defined in template repositories (`.reporoller/template.toml`)
2. **Team** - Team-specific overrides (`teams/{team-name}/config.toml`)
3. **Repository Type** - Type-specific settings (`types/{type-name}/config.toml`)
4. **Global** - Organization-wide defaults (`global/defaults.toml`)
5. **System** - Built-in fallback defaults

Higher levels override lower levels, but can be constrained by `override_allowed` controls.

## Configuration Files

### Global Configuration

Located at `global/defaults.toml` in the metadata repository.

```toml
# global/defaults.toml

[repository]
has_issues = true
has_projects = false
has_wiki = true
has_discussions = true

[pull_requests]
allow_merge_commit = false
allow_squash_merge = true
allow_rebase_merge = false
required_approving_review_count = 1
```

### Repository Type Configuration

Located at `types/{type-name}/config.toml` in the metadata repository.

```toml
# types/library/config.toml

[repository]
has_wiki = false  # Libraries typically don't need wikis
has_projects = false
security_advisories = true
vulnerability_reporting = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
```

### Team Configuration

Located at `teams/{team-name}/config.toml` in the metadata repository.

```toml
# teams/backend-team/config.toml

[repository]
has_discussions = false  # Override global default
has_projects = true

[pull_requests]
required_approving_review_count = 2
allow_auto_merge = true
```

### Template Configuration

Located at `.reporoller/template.toml` in template repositories.

```toml
# .reporoller/template.toml

[template]
name = "rust-microservice"
description = "Production-ready Rust microservice template"
author = "Platform Team"
tags = ["rust", "microservice", "backend"]

[repository]
has_wiki = false
security_advisories = true

[pull_requests]
required_approving_review_count = 2
require_code_owner_reviews = true
```

## Repository Settings

Configure repository features and settings:

```toml
[repository]
# Feature toggles
has_issues = true
has_projects = false
has_wiki = true
has_discussions = false

# Security settings
security_advisories = true
vulnerability_reporting = true

# Repository behavior
default_branch = "main"
allow_forking = true
is_template = false
```

## Pull Request Settings

Configure pull request and merge settings:

```toml
[pull_requests]
# Merge methods
allow_merge_commit = true
allow_squash_merge = true
allow_rebase_merge = false

# Review requirements
required_approving_review_count = 2
require_code_owner_reviews = true
dismiss_stale_reviews_on_push = true
require_conversation_resolution = true

# Auto-merge
allow_auto_merge = false

# Merge commit messages
merge_commit_title = "MERGE_MESSAGE"
squash_merge_commit_title = "PR_TITLE"
```

## Labels Configuration

Define repository labels:

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
```

## Webhooks Configuration

Configure repository webhooks:

```toml
[[webhooks]]
name = "ci-webhook"
url = "https://ci.example.com/webhook"
content_type = "json"
secret = "your-webhook-secret"
events = ["push", "pull_request"]
active = true
insecure_ssl = false
```

## Repository Rulesets

Repository rulesets provide governance rules for branches and tags.

### Basic Branch Protection

```toml
[[rulesets]]
name = "main-branch-protection"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "non_fast_forward"

[[rulesets.rules]]
type = "required_linear_history"
```

### Pull Request Requirements

```toml
[[rulesets]]
name = "pr-requirements"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/develop"]

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
dismiss_stale_reviews_on_push = true
allowed_merge_methods = ["squash"]
```

### Status Check Requirements

```toml
[[rulesets]]
name = "ci-checks"
target = "branch"
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

### Tag Protection

```toml
[[rulesets]]
name = "release-tags"
target = "tag"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/tags/v*"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "update"
```

### Advanced Ruleset Configuration

```toml
[[rulesets]]
name = "comprehensive-protection"
target = "branch"
enforcement = "active"

# Allow organization admins to bypass
[[rulesets.bypass_actors]]
actor_id = 1
actor_type = "OrganizationAdmin"
bypass_mode = "always"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/release/*"]
exclude = ["refs/heads/release/experimental-*"]

[[rulesets.rules]]
type = "deletion"

[[rulesets.rules]]
type = "non_fast_forward"

[[rulesets.rules]]
type = "pull_request"
required_approving_review_count = 2
require_code_owner_review = true
require_last_push_approval = true
dismiss_stale_reviews_on_push = true
required_review_thread_resolution = true
allowed_merge_methods = ["squash", "merge"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/integration", integration_id = 12345 },
  { context = "security/sast" },
  { context = "security/dependencies" }
]
```

### Ruleset Rule Types

Available rule types:

- **deletion**: Prevents deletion of matching references
- **non_fast_forward**: Prevents force pushes
- **required_linear_history**: Requires linear history (no merge commits)
- **update**: Prevents updates to matching references
- **pull_request**: Requires pull request before merging
- **required_status_checks**: Requires specific status checks to pass
- **required_signatures**: Requires signed commits
- **creation**: Prevents creation of matching references

### Ruleset Hierarchy

Rulesets are **additive** across the configuration hierarchy:

```
Global:      main-protection (prevents deletion)
Type:        library-protection (requires 2 approvals)
Team:        team-checks (requires specific status checks)
Template:    template-requirements (requires code owner review)

Result:      All four rulesets are applied to the repository
```

**Note:** If rulesets with the same name are defined at different levels, they will create
separate independent rulesets (not merged). A warning will be logged when duplicate names
are detected.

## Override Controls

Control which configuration values can be overridden at higher levels:

```toml
[repository]
# This value can be overridden by teams and templates
has_wiki = { value = true, override_allowed = true }

# This value CANNOT be overridden - it's an organization policy
security_advisories = { value = true, override_allowed = false }
```

When a setting has `override_allowed = false`, attempts to override it at team or template level will fail validation.

## Examples

### Example 1: Library Repository Configuration

```toml
# Global (global/defaults.toml)
[repository]
has_issues = true
has_wiki = true

# Repository Type (types/library/config.toml)
[repository]
has_wiki = false  # Libraries use README, not wiki
security_advisories = true

[pull_requests]
required_approving_review_count = 2

[[labels]]
name = "breaking-change"
color = "d73a4a"
description = "Breaking API changes"

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

**Result**: Library repositories have issues enabled, wiki disabled, security advisories enabled, require 2 approving reviews, have a "breaking-change" label, and protect the main branch with deletion prevention and pull request requirements.

### Example 2: Microservice Template Configuration

```toml
# Template (.reporoller/template.toml)
[template]
name = "rust-microservice"
description = "Production-ready Rust microservice"
author = "Platform Team"
tags = ["rust", "microservice", "backend"]

[repository]
has_wiki = false
has_discussions = false

[variables]
service_name = { description = "Name of the microservice", example = "user-service" }
service_port = { description = "Port the service runs on", default = "8080" }

[[webhooks]]
name = "deployment-webhook"
url = "https://deploy.example.com/webhook/{{service_name}}"
content_type = "json"
events = ["push", "release"]
active = true

[[rulesets]]
name = "microservice-ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main"]

[[rulesets.rules]]
type = "required_status_checks"
strict_required_status_checks_policy = true
required_checks = [
  { context = "rust/build" },
  { context = "rust/test" },
  { context = "rust/clippy" },
  { context = "docker/build" },
  { context = "security/cargo-audit" }
]
```

**Result**: Microservice repositories from this template have a deployment webhook configured with service-specific URL, and require all Rust CI checks to pass before merging to main.

### Example 3: Team-Specific Configuration

```toml
# Team (teams/frontend-team/config.toml)
[repository]
has_discussions = true  # Frontend team likes discussions for design decisions

[pull_requests]
required_approving_review_count = 1  # Smaller team, 1 approval is sufficient
allow_auto_merge = true

[[labels]]
name = "ui"
color = "0e8a16"
description = "UI/UX changes"

[[labels]]
name = "accessibility"
color = "f9d0c4"
description = "Accessibility improvements"

[[rulesets]]
name = "frontend-ci-checks"
target = "branch"
enforcement = "active"

[rulesets.conditions.ref_name]
include = ["refs/heads/main", "refs/heads/develop"]

[[rulesets.rules]]
type = "required_status_checks"
required_checks = [
  { context = "ci/build" },
  { context = "ci/test" },
  { context = "ci/lint" },
  { context = "ci/visual-regression" }
]
```

**Result**: Frontend team repositories have discussions enabled, require only 1 approval (overriding global default), allow auto-merge, have UI/accessibility labels, and require visual regression tests.

## Best Practices

1. **Start with sensible global defaults** that work for most repositories
2. **Use repository types** to group similar repositories with shared requirements
3. **Keep team configurations minimal** - only override what's necessary
4. **Document template requirements** clearly in template descriptions
5. **Use descriptive names** for labels, webhooks, and rulesets
6. **Test rulesets** in "evaluate" mode before switching to "active"
7. **Avoid overly restrictive rules** that might hinder productivity
8. **Use `override_allowed = false`** for critical security policies
9. **Leverage variables** in webhooks and other configuration for flexibility
10. **Keep configuration DRY** by using the hierarchy effectively

## Troubleshooting

### Configuration Not Applied

- Check the configuration hierarchy - higher levels override lower levels
- Verify `override_allowed` settings - some values cannot be overridden
- Check for TOML syntax errors in configuration files
- Ensure the metadata repository is accessible and correctly configured

### Rulesets Not Working

- Verify `enforcement` is set to "active" (not "disabled" or "evaluate")
- Check branch/tag patterns in `conditions.ref_name.include`
- Ensure rule types are spelled correctly
- Verify required status check contexts match your CI system

### Override Violations

- Check if the global configuration has `override_allowed = false` for that setting
- Verify repository type policies if using type-based configuration
- Review template configuration for conflicting requirements

## Additional Resources

- [GitHub Rulesets Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)
- [Organization Repository Settings Design](docs/spec/design/organization-repository-settings.md)
- [Configuration Interfaces](docs/spec/interfaces/configuration-interfaces.md)
