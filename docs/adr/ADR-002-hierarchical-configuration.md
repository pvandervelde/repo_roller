# ADR-002: Hierarchical Configuration System

Status: Accepted
Date: 2026-01-31
Owners: RepoRoller team

## Context

RepoRoller needs to support configuration at multiple organizational levels while maintaining clear precedence rules. Different organizations have different standards, teams within organizations have specific requirements, and individual templates may have unique needs. The system must resolve these configurations in a predictable way while supporting overrides and policy enforcement.

Requirements:

- Global defaults applicable to all repositories
- Organization-wide policies and standards
- Team-specific customizations
- Repository type configurations (library, service, action)
- Template-specific requirements
- Clear precedence rules for conflict resolution

Previous approaches considered:

- Single configuration file per organization - too rigid, no customization
- Environment variables only - difficult to manage hierarchies and complex values
- Database-driven configuration - adds infrastructure complexity and deployment challenges

## Decision

Implement a **5-level hierarchical configuration system** with runtime resolution:

1. **System Defaults** - Hard-coded fallbacks in application
2. **Global Organization Defaults** - `.reporoller/defaults.toml` in metadata repo
3. **Team Configuration** - `.reporoller/teams/<team-name>.toml`
4. **Repository Type Configuration** - `.reporoller/types/<type-name>.toml`
5. **Template Configuration** - `.reporoller/template.toml` in template repo

**Resolution order**: Template → Type → Team → Organization → System (most specific wins)

**Storage**: All configuration stored in GitHub repositories as TOML files, version-controlled and auditable.

**Caching**: Configurations cached with 5-minute TTL to balance performance and freshness.

## Consequences

**Enables:**

- Organizations can define standards while allowing team flexibility
- Teams can customize without breaking organizational policies
- Templates can specify required variables and constraints
- Clear audit trail through version control
- Configuration changes don't require redeployment
- Policy enforcement at organization level

**Forbids:**

- Arbitrary override order (must follow hierarchy)
- Configuration outside metadata repositories
- Unlimited override freedom (organization policies can restrict)

**Trade-offs:**

- More complex configuration resolution logic
- Need for efficient caching to avoid performance issues
- Potential for configuration conflicts requiring careful design
- Learning curve for understanding hierarchy

## Alternatives considered

### Option A: Single configuration file per organization

**Why not**: No flexibility for different teams or project types. Every customization requires changing organization-wide file, causing contention and risk.

### Option B: Database-driven configuration

**Why not**: Requires additional infrastructure (database server, backup, monitoring). Adds complexity to deployment. Loses version control benefits. No clear advantage over file-based approach for this use case.

### Option C: Environment variables only

**Why not**: Poor structure for complex configurations. Difficult to manage hierarchies. No version control. Not suitable for per-template or per-team variations.

### Option D: Flat configuration with manual overrides

**Why not**: No clear precedence rules. Conflict resolution becomes ad-hoc. Difficult to understand what configuration applies. Cannot enforce policies consistently.

## Implementation notes

**Configuration file locations:**

```
<org>/.reporoller/
├── defaults.toml           # Global org defaults (level 2)
├── teams/
│   ├── backend.toml        # Team-specific (level 3)
│   └── frontend.toml
├── types/
│   ├── library.toml        # Type-specific (level 4)
│   ├── service.toml
│   └── github-action.toml
└── templates/              # Template metadata

<template-repo>/.reporoller/
└── template.toml           # Template config (level 5)
```

**Merge strategy:**

- Scalar values: Higher level overrides lower level
- Arrays: Higher level replaces lower level (no merging)
- Objects: Deep merge with higher level keys overriding

**Cache invalidation:**

- Webhook on metadata repository changes
- Manual invalidation API endpoint
- Automatic expiration after TTL

**Policy enforcement:**
Organization defaults can specify fields as "required" or "prohibited" to restrict overrides.

## Examples

**System defaults (level 1 - hard-coded):**

```rust
pub fn system_defaults() -> Configuration {
    Configuration {
        visibility: RepositoryVisibility::Private,
        auto_init: false,
        default_branch: "main".to_string(),
        // ...
    }
}
```

**Organization defaults (level 2 - `.reporoller/defaults.toml`):**

```toml
[repository]
visibility = "private"
default_branch = "main"
auto_init = true
license = "Apache-2.0"

[policies]
required_files = [".github/CODEOWNERS", "LICENSE"]
prohibited_visibility = ["public"]  # Organization policy

[github]
enable_issues = true
enable_projects = false
```

**Team configuration (level 3 - `.reporoller/teams/backend.toml`):**

```toml
[repository]
topics = ["backend", "rust"]
default_reviewers = ["@backend-leads"]

[ci]
required_checks = ["build", "test", "security-scan"]
```

**Type configuration (level 4 - `.reporoller/types/library.toml`):**

```toml
[repository]
topics = ["library"]
enable_wiki = false

[required_files]
files = ["README.md", "CHANGELOG.md", "examples/"]
```

**Template configuration (level 5 - `template/.reporoller/template.toml`):**

```toml
[template]
name = "rust-library"
description = "Rust library template"

[variables]
required = ["crate_name", "author_name"]
optional = ["description", "homepage"]

[repository]
topics = ["rust"]  # Added to team/type topics
```

**Merged result for backend team creating rust-library:**

```rust
Configuration {
    // From template (level 5)
    topics: ["rust", "backend", "library"],  // Merged from all levels

    // From type (level 4)
    enable_wiki: false,
    required_files: ["README.md", "CHANGELOG.md", "examples/"],

    // From team (level 3)
    default_reviewers: ["@backend-leads"],
    required_checks: ["build", "test", "security-scan"],

    // From org (level 2)
    visibility: Private,  // Enforced by policy
    default_branch: "main",
    license: "Apache-2.0",
    enable_issues: true,
    enable_projects: false,

    // System defaults used where nothing specified
}
```

## References

- [Organization Repository Settings Requirements](../spec/requirements/organization-repository-settings.md)
- [Configuration Interfaces Specification](../spec/interfaces/configuration-interfaces.md)
- [Organization Repository Settings Design](../spec/design/organization-repository-settings.md)
- [Architectural Tradeoffs](../spec/tradeoffs.md#configuration-architecture)
