# ADR-004: Repository Visibility Policy System

Status: Accepted
Date: 2026-01-31
Owners: RepoRoller team

## Context

Repository visibility (Public, Private, Internal) has security and organizational implications. Organizations need to enforce policies about what visibility levels are permitted, while still allowing flexibility for appropriate use cases. Users may have preferences, templates may have defaults, but organization security policies must take precedence.

Security requirements:

- Organizations must be able to require all repositories be Private
- Organizations must be able to prohibit Public repositories
- Users should be able to specify preferences when policy allows
- GitHub Enterprise features (Internal visibility) must be detected
- GitHub plan limitations must be respected
- Clear audit trail of visibility decisions

Challenge: Multiple inputs to the decision (org policy, user preference, template default, system default) with complex precedence rules and external constraints (GitHub plan limitations).

## Decision

Implement a **hierarchical visibility resolution system** with policy enforcement:

**Decision Hierarchy:**

1. **Organization Policy** (if Required → enforced immediately)
2. **User Preference** (validated against policy restrictions)
3. **Template Default** (validated against policy restrictions)
4. **System Default** (Private - most secure)
5. **GitHub Platform Validation** (ensure plan supports chosen visibility)

**Policy Levels:**

- **Required**: Organization mandates specific visibility (e.g., "all repos must be Private")
- **Restricted**: Organization prohibits certain visibilities (e.g., "no Public repos allowed")
- **Unrestricted**: Organization allows all visibility levels

**Resolution Output:**

- Determined visibility
- Decision source (why this visibility was chosen)
- Applied constraints (what policies were enforced)

## Consequences

**Enables:**

- Organization security policies are always enforced
- Clear audit trail of why each visibility was chosen
- User flexibility where policy allows
- Graceful handling of GitHub plan limitations
- Prevention of accidental public repository creation

**Forbids:**

- Users overriding organization Required policy
- Creating repositories with prohibited visibility
- Using Internal visibility on GitHub.com (Enterprise only)
- Bypassing GitHub plan limitations

**Trade-offs:**

- More complex visibility resolution logic
- Need to detect GitHub environment capabilities
- Policy configuration required for each organization
- Potential user confusion when preferences are overridden

## Alternatives considered

### Option A: Simple user preference only

**Why not**: No security controls. Users could accidentally create public repos with sensitive data. Organizations cannot enforce security policies.

### Option B: Organization policy only (no user input)

**Why not**: Too rigid. Legitimate use cases for different visibilities (e.g., open source libraries should be Public). Reduces user autonomy unnecessarily.

### Option C: Template defaults only

**Why not**: Templates may come from external sources. No security enforcement. Cannot adapt to different organizational security postures.

### Option D: Flat precedence (first-specified-wins)

**Why not**: Unclear which input takes precedence. No security guarantees. Cannot express "organization requires Private" consistently.

## Implementation notes

**Key components:**

1. **VisibilityPolicyProvider** - Loads organization policies
2. **GitHubEnvironmentDetector** - Detects plan limitations (Enterprise vs Free)
3. **VisibilityResolver** - Orchestrates decision process

**Caching strategy:**

- Organization policies: 5-minute TTL (security-sensitive)
- GitHub environment: 1-hour TTL (rarely changes)
- Thread-safe cache with `Arc<RwLock<>>`

**Error handling:**

- `PolicyViolation` - User request violates organization policy
- `GitHubConstraint` - Requested visibility not supported by GitHub plan
- `VisibilityError` - Wraps all visibility-related errors

**Circular dependency resolution:**
`RepositoryVisibility` enum defined in `config_manager` (where policy config lives) but re-exported through `repo_roller_core` to avoid circular dependency. See [constraints.md](../spec/constraints.md#pragmatic-exception-for-infrastructure-defined-types).

## Examples

**Organization requires Private (level 1 - highest precedence):**

```rust
// Organization policy
let policy = VisibilityPolicy::Required(RepositoryVisibility::Private);

// User wants Public
let request = VisibilityRequest {
    organization: "acme-corp",
    user_preference: Some(RepositoryVisibility::Public),
    template_default: None,
};

// Result: Private (org policy overrides)
let decision = resolver.resolve_visibility(request).await?;
assert_eq!(decision.visibility, RepositoryVisibility::Private);
assert_eq!(decision.source, DecisionSource::OrganizationPolicy);
assert!(decision.constraints_applied.contains(&PolicyConstraint::OrganizationRequired));
```

**Organization restricts Public (level 2 - validation):**

```rust
// Organization prohibits Public
let policy = VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public]);

// User wants Public
let request = VisibilityRequest {
    organization: "acme-corp",
    user_preference: Some(RepositoryVisibility::Public),
    template_default: None,
};

// Result: Error - violates policy
let result = resolver.resolve_visibility(request).await;
assert!(matches!(result, Err(VisibilityError::PolicyViolation { .. })));
```

**User preference allowed by policy (level 2 - accepted):**

```rust
// Organization allows all visibilities
let policy = VisibilityPolicy::Unrestricted;

// User wants Private
let request = VisibilityRequest {
    organization: "open-source-org",
    user_preference: Some(RepositoryVisibility::Private),
    template_default: Some(RepositoryVisibility::Public),
};

// Result: Private (user preference honored)
let decision = resolver.resolve_visibility(request).await?;
assert_eq!(decision.visibility, RepositoryVisibility::Private);
assert_eq!(decision.source, DecisionSource::UserPreference);
```

**Template default (level 3):**

```rust
let request = VisibilityRequest {
    organization: "open-source-org",
    user_preference: None,  // User didn't specify
    template_default: Some(RepositoryVisibility::Public),
};

// Result: Public (from template)
let decision = resolver.resolve_visibility(request).await?;
assert_eq!(decision.visibility, RepositoryVisibility::Public);
assert_eq!(decision.source, DecisionSource::TemplateDefault);
```

**System default (level 4 - safest fallback):**

```rust
let request = VisibilityRequest {
    organization: "new-org",
    user_preference: None,
    template_default: None,
};

// Result: Private (system default - most secure)
let decision = resolver.resolve_visibility(request).await?;
assert_eq!(decision.visibility, RepositoryVisibility::Private);
assert_eq!(decision.source, DecisionSource::SystemDefault);
```

**GitHub platform validation (level 5):**

```rust
// User wants Internal, but not on Enterprise
let request = VisibilityRequest {
    organization: "github-com-org",  // Not Enterprise
    user_preference: Some(RepositoryVisibility::Internal),
    template_default: None,
};

// Result: Error - Internal only available on Enterprise
let result = resolver.resolve_visibility(request).await;
assert!(matches!(result, Err(VisibilityError::GitHubConstraint { .. })));
```

## References

- [Repository Visibility Requirements](../spec/requirements/repository-visibility.md)
- [Repository Visibility Design](../spec/design/repository-visibility.md)
- [Repository Visibility Interface](../spec/interfaces/repository-visibility.md)
- [Architectural Tradeoffs](../spec/tradeoffs.md)
- GitHub Visibility Docs: <https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility>
