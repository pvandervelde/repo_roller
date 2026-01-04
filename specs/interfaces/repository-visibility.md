# Repository Visibility Interface

**Architectural Layers**: 
- **Configuration Management** (`config_manager`): Policy definitions and provider interface
- **Core Domain** (`repo_roller_core`): Resolution orchestration logic
- **GitHub Integration** (`github_client`): Environment detection

**Module Paths**: 
- `crates/config_manager/src/visibility.rs` (policy types and provider trait)
- `crates/repo_roller_core/src/visibility.rs` (resolution logic)
- `crates/github_client/src/environment.rs` (environment detection)

**Responsibilities** (from RDD):

- **config_manager** Knows: Visibility policy definitions, organization-specific rules
- **repo_roller_core** Does: Resolves visibility using hierarchical policy system
- **github_client** Knows: GitHub environment capabilities (Enterprise vs standard)

## Overview

The repository visibility interface defines the types, traits, and contracts for managing repository visibility decisions during repository creation. It implements a hierarchical policy system that balances organizational security requirements, user preferences, template defaults, and GitHub platform constraints.

**Architectural Split**: The visibility domain is split across three crates to avoid circular dependencies:

1. **`config_manager`** (Infrastructure - Configuration)
   - Policy type definitions (`VisibilityPolicy`, `RepositoryVisibility`)
   - Policy provider trait (`VisibilityPolicyProvider`)
   - Configuration-based policy implementation
   - Location: `crates/config_manager/src/visibility.rs`

2. **`repo_roller_core`** (Core Domain - Business Logic)
   - Resolution orchestration (`VisibilityResolver`)
   - Request/decision types (`VisibilityRequest`, `VisibilityDecision`)
   - Resolution hierarchy and validation logic
   - Re-exports policy types from `config_manager` for convenience
   - Location: `crates/repo_roller_core/src/visibility.rs`

3. **`github_client`** (Infrastructure - GitHub Integration)
   - Environment detection trait (`GitHubEnvironmentDetector`)
   - Plan limitations detection
   - Enterprise environment detection
   - Location: `crates/github_client/src/environment.rs`

This separation mirrors the existing pattern where `repo_roller_core` re-exports `ConfigurationError` from `config_manager` (see `crates/repo_roller_core/src/errors.rs:134`).

## Dependencies

- Policy Types: `RepositoryVisibility`, `VisibilityPolicy`, `PolicyConstraint` (defined in `config_manager/src/visibility.rs`)
- Resolution Types: `VisibilityDecision`, `VisibilityRequest`, `DecisionSource` (defined in `repo_roller_core/src/visibility.rs`)
- Domain Types: `OrganizationName` from `repository-domain.md`
- Traits: `VisibilityPolicyProvider` (in `config_manager`), `GitHubEnvironmentDetector` (in `github_client`)
- Shared: Standard Rust types (`String`, `Vec`, etc.)

## Core Types (config_manager)

These types are defined in `crates/config_manager/src/visibility.rs` and re-exported by `repo_roller_core` for convenience.

### RepositoryVisibility

**Module**: `config_manager/src/visibility.rs`
**Re-exported**: `repo_roller_core::RepositoryVisibility`

```rust
/// Repository visibility level.
///
/// Represents the three visibility options available in GitHub.
/// Internal visibility is only available in GitHub Enterprise environments.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::RepositoryVisibility;
///
/// let public = RepositoryVisibility::Public;
/// let private = RepositoryVisibility::Private;
/// let internal = RepositoryVisibility::Internal;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryVisibility {
    /// Visible to all GitHub users
    Public,

    /// Visible only to repository collaborators
    Private,

    /// Visible to all organization/enterprise members (GitHub Enterprise only)
    Internal,
}
```

**Validation Rules**:

- Must be one of: Public, Private, Internal
- Internal only valid in GitHub Enterprise environments
- Serialization uses lowercase strings ("public", "private", "internal")

**Behavior**:

- Implements Copy for efficiency
- Implements Eq for policy comparisons
- Implements Hash for use in collections
- JSON serialization to/from lowercase strings

**String Representation**:

- Public → "public"
- Private → "private"
- Internal → "internal"

---

### VisibilityPolicy

**Module**: `config_manager/src/visibility.rs`
**Re-exported**: `repo_roller_core::VisibilityPolicy`

```rust
/// Organization-level visibility policy.
///
/// Defines how repository visibility is controlled at the organization level.
/// Policies can require specific visibility, restrict certain options, or allow
/// unrestricted choice.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{VisibilityPolicy, RepositoryVisibility};
///
/// // Require all repositories to be private
/// let required = VisibilityPolicy::Required(RepositoryVisibility::Private);
///
/// // Prohibit public repositories
/// let restricted = VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public]);
///
/// // Allow any visibility
/// let unrestricted = VisibilityPolicy::Unrestricted;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum VisibilityPolicy {
    /// Forces a specific visibility for all repositories
    Required(RepositoryVisibility),

    /// Prohibits specific visibility options (others are allowed)
    Restricted(Vec<RepositoryVisibility>),

    /// Allows any visibility choice
    Unrestricted,
}
```

**Validation Rules**:

- Required: Must specify exactly one visibility
- Restricted: Must specify at least one prohibited visibility
- Unrestricted: No additional validation

**Behavior**:

- Policies are enforced before any GitHub API calls
- Required policy takes precedence over all preferences
- Restricted policy validates user/template preferences
- Unrestricted policy allows any valid visibility

---

### PolicyConstraint

**Module**: `config_manager/src/visibility.rs`
**Re-exported**: `repo_roller_core::PolicyConstraint`

```rust
/// Constraint that was applied during visibility resolution.
///
/// Documents which constraints influenced the visibility decision
/// for audit and troubleshooting purposes.
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyConstraint {
    /// Organization requires specific visibility
    OrganizationRequired,

    /// Organization restricts certain visibilities
    OrganizationRestricted,

    /// Requires GitHub Enterprise
    RequiresEnterprise,

    /// Requires paid GitHub plan
    RequiresPaidPlan,

    /// User lacks permission for requested visibility
    InsufficientPermissions,
}
```

**Usage**:

- Included in `VisibilityDecision` for audit trail
- Multiple constraints can apply to a single decision
- Used in error messages to explain policy violations

---

## Resolution Types (repo_roller_core)

These types are defined in `crates/repo_roller_core/src/visibility.rs` for business logic operations.

### DecisionSource

**Module**: `repo_roller_core/src/visibility.rs`

```rust
/// Source of the visibility decision.
///
/// Indicates which level of the hierarchy determined the final visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSource {
    /// Organization policy mandated the visibility
    OrganizationPolicy,

    /// User explicitly specified the visibility
    UserPreference,

    /// Template default was used
    TemplateDefault,

    /// System default was applied
    SystemDefault,
}
```

**Decision Hierarchy** (highest to lowest precedence):

1. OrganizationPolicy (required policy)
2. UserPreference (if allowed by policy)
3. TemplateDefault (if allowed by policy)
4. SystemDefault (fallback)

---

### VisibilityDecision

**Module**: `repo_roller_core/src/visibility.rs`

```rust
/// Result of visibility resolution.
///
/// Contains the determined visibility and metadata about how the
/// decision was made.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{VisibilityDecision, RepositoryVisibility, DecisionSource, PolicyConstraint};
///
/// let decision = VisibilityDecision {
///     visibility: RepositoryVisibility::Private,
///     source: DecisionSource::OrganizationPolicy,
///     constraints_applied: vec![PolicyConstraint::OrganizationRequired],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct VisibilityDecision {
    /// The determined visibility
    pub visibility: RepositoryVisibility,

    /// Source of the decision in the hierarchy
    pub source: DecisionSource,

    /// Constraints that were applied during resolution
    pub constraints_applied: Vec<PolicyConstraint>,
}
```

**Usage**:

- Returned by `VisibilityResolver::resolve_visibility()`
- Provides audit trail for visibility decisions
- Used to document why specific visibility was chosen

---

### VisibilityRequest

**Module**: `repo_roller_core/src/visibility.rs`

```rust
/// Input to the visibility resolution process.
///
/// Contains all information needed to determine repository visibility.
#[derive(Debug, Clone)]
pub struct VisibilityRequest {
    /// Organization where repository will be created
    pub organization: OrganizationName,

    /// User's explicit visibility preference (optional)
    pub user_preference: Option<RepositoryVisibility>,

    /// Template's default visibility (optional)
    pub template_default: Option<RepositoryVisibility>,
}
```

**Validation Rules**:

- organization: Must be valid OrganizationName
- user_preference: If specified, must be valid RepositoryVisibility
- template_default: If specified, must be valid RepositoryVisibility

**Construction**:

```rust
let request = VisibilityRequest {
    organization: OrganizationName::new("my-org")?,
    user_preference: Some(RepositoryVisibility::Private),
    template_default: None,
};
```

---

### PlanLimitations

**Module**: `github_client/src/environment.rs`

```rust
/// GitHub plan limitations affecting visibility.
///
/// Contains information about what visibility options are available
/// based on the organization's GitHub plan and environment.
#[derive(Debug, Clone)]
pub struct PlanLimitations {
    /// Whether private repositories are supported
    pub supports_private_repos: bool,

    /// Whether internal repositories are supported (Enterprise only)
    pub supports_internal_repos: bool,

    /// Maximum number of private repositories (None = unlimited)
    pub private_repo_limit: Option<u32>,

    /// Whether this is a GitHub Enterprise environment
    pub is_enterprise: bool,
}
```

**Usage**:

- Retrieved by `GitHubEnvironmentDetector`
- Cached to reduce API calls
- Used to validate visibility choices against platform constraints

---

## Trait Definitions

### VisibilityPolicyProvider

**Module**: `config_manager/src/visibility.rs`
**Re-exported**: `repo_roller_core::VisibilityPolicyProvider`
**Implementation**: `config_manager/src/visibility_policy_provider.rs`

```rust
/// Provides organization visibility policies.
///
/// Implementations fetch and cache organization-level visibility policies
/// from the configuration system.
///
/// See: specs/interfaces/repository-visibility.md
#[async_trait]
pub trait VisibilityPolicyProvider: Send + Sync {
    /// Get the visibility policy for an organization.
    ///
    /// # Arguments
    /// * `organization` - Organization name
    ///
    /// # Returns
    /// Organization's visibility policy
    ///
    /// # Errors
    /// * `VisibilityError::PolicyNotFound` - Organization has no policy configured
    /// * `VisibilityError::ConfigurationError` - Policy configuration is invalid
    async fn get_policy(
        &self,
        organization: &OrganizationName,
    ) -> Result<VisibilityPolicy, VisibilityError>;

    /// Invalidate cached policy for an organization.
    ///
    /// Forces the next `get_policy` call to fetch fresh policy data.
    ///
    /// # Arguments
    /// * `organization` - Organization name
    async fn invalidate_cache(&self, organization: &OrganizationName);
}
```

**Implementation Requirements**:

- Must implement caching (max age: 5 minutes)
- Must handle concurrent access safely
- Must refresh stale data automatically
- Must provide clear error messages

**Example Usage**:

```rust
let provider = ConfigBasedPolicyProvider::new(config_manager);
let policy = provider.get_policy(&org_name).await?;

match policy {
    VisibilityPolicy::Required(vis) => {
        // Must use this visibility
    }
    VisibilityPolicy::Restricted(prohibited) => {
        // Cannot use these visibilities
    }
    VisibilityPolicy::Unrestricted => {
        // Any visibility allowed
    }
}
```

---

### GitHubEnvironmentDetector

**Module**: `github_client/src/environment.rs`
**Implementation**: `github_client/src/environment_detector.rs`

```rust
/// Detects GitHub environment capabilities and limitations.
///
/// Implementations interact with GitHub APIs to determine what visibility
/// options are available based on the organization's plan and environment.
///
/// See: specs/interfaces/repository-visibility.md
#[async_trait]
pub trait GitHubEnvironmentDetector: Send + Sync {
    /// Get plan limitations for an organization.
    ///
    /// # Arguments
    /// * `organization` - Organization name
    ///
    /// # Returns
    /// Plan limitations affecting visibility options
    ///
    /// # Errors
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    async fn get_plan_limitations(
        &self,
        organization: &OrganizationName,
    ) -> Result<PlanLimitations, VisibilityError>;

    /// Check if organization is in GitHub Enterprise environment.
    ///
    /// # Arguments
    /// * `organization` - Organization name
    ///
    /// # Returns
    /// `true` if organization is in GitHub Enterprise
    ///
    /// # Errors
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    async fn is_enterprise(
        &self,
        organization: &OrganizationName,
    ) -> Result<bool, VisibilityError>;
}
```

**Implementation Requirements**:

- Must cache results (max age: 1 hour)
- Must handle GitHub API rate limits gracefully
- Must detect environment from API responses
- Must fall back safely when detection fails

**Example Usage**:

```rust
let detector = GitHubApiEnvironmentDetector::new(github_client);
let limitations = detector.get_plan_limitations(&org_name).await?;

if limitations.supports_internal_repos {
    // Internal visibility is available
} else {
    // Internal visibility not available
}
```

---

## VisibilityResolver Component

**Module**: `repo_roller_core/src/visibility.rs`

### Interface

```rust
/// Resolves repository visibility based on policies and preferences.
///
/// Implements the hierarchical visibility decision process, validating
/// against organization policies and GitHub platform constraints.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{VisibilityResolver, VisibilityRequest, OrganizationName};
///
/// let resolver = VisibilityResolver::new(policy_provider, environment_detector);
///
/// let request = VisibilityRequest {
///     organization: OrganizationName::new("my-org")?,
///     user_preference: Some(RepositoryVisibility::Private),
///     template_default: None,
/// };
///
/// let decision = resolver.resolve_visibility(request).await?;
/// println!("Using visibility: {:?}", decision.visibility);
/// ```
///
/// See: specs/interfaces/repository-visibility.md
pub struct VisibilityResolver {
    policy_provider: Arc<dyn VisibilityPolicyProvider>,
    environment_detector: Arc<dyn GitHubEnvironmentDetector>,
}
```

### Methods

```rust
impl VisibilityResolver {
    /// Create a new visibility resolver.
    ///
    /// # Arguments
    /// * `policy_provider` - Provider for organization policies
    /// * `environment_detector` - Detector for GitHub environment capabilities
    pub fn new(
        policy_provider: Arc<dyn VisibilityPolicyProvider>,
        environment_detector: Arc<dyn GitHubEnvironmentDetector>,
    ) -> Self {
        unimplemented!("See specs/interfaces/repository-visibility.md")
    }

    /// Resolve repository visibility.
    ///
    /// Implements the hierarchical decision process:
    /// 1. Check organization policy (required → enforced immediately)
    /// 2. Validate user preference against policy
    /// 3. Fall back to template default (if allowed by policy)
    /// 4. Use system default (Private)
    /// 5. Validate against GitHub platform constraints
    ///
    /// # Arguments
    /// * `request` - Visibility resolution request with preferences
    ///
    /// # Returns
    /// Visibility decision with audit trail
    ///
    /// # Errors
    /// * `VisibilityError::PolicyViolation` - Requested visibility violates policy
    /// * `VisibilityError::GitHubConstraint` - Visibility not available on this plan
    /// * `VisibilityError::PolicyNotFound` - Organization has no policy configured
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    ///
    /// # Performance
    /// Typical: <50ms (cached)
    /// Cache miss: <2s (requires API calls)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let request = VisibilityRequest {
    ///     organization: OrganizationName::new("my-org")?,
    ///     user_preference: Some(RepositoryVisibility::Public),
    ///     template_default: Some(RepositoryVisibility::Private),
    /// };
    ///
    /// let decision = resolver.resolve_visibility(request).await?;
    ///
    /// match decision.source {
    ///     DecisionSource::UserPreference => println!("Used user preference"),
    ///     DecisionSource::OrganizationPolicy => println!("Enforced by policy"),
    ///     DecisionSource::TemplateDefault => println!("Used template default"),
    ///     DecisionSource::SystemDefault => println!("Used system default"),
    /// }
    /// ```
    pub async fn resolve_visibility(
        &self,
        request: VisibilityRequest,
    ) -> Result<VisibilityDecision, VisibilityError> {
        unimplemented!("See specs/interfaces/repository-visibility.md")
    }
}
```

---

## Error Types

**Module**: `config_manager/src/visibility.rs` (policy-related errors)
**Module**: `repo_roller_core/src/visibility.rs` (resolution-related errors)

```rust
/// Errors that can occur during visibility resolution.
///
/// Provides detailed context for visibility-related failures.
/// Defined in config_manager and re-exported by repo_roller_core.
#[derive(Debug, thiserror::Error)]
pub enum VisibilityError {
    /// Organization policy not found
    #[error("No visibility policy configured for organization: {organization}")]
    PolicyNotFound {
        organization: String,
    },

    /// Requested visibility violates organization policy
    #[error("Visibility {requested:?} violates organization policy: {policy:?}")]
    PolicyViolation {
        requested: RepositoryVisibility,
        policy: String,
    },

    /// Requested visibility not available on GitHub plan
    #[error("Visibility {requested:?} not available: {reason}")]
    GitHubConstraint {
        requested: RepositoryVisibility,
        reason: String,
    },

    /// Configuration error
    #[error("Visibility configuration error: {message}")]
    ConfigurationError {
        message: String,
    },

    /// GitHub API error during visibility resolution
    #[error("GitHub API error: {source}")]
    GitHubApiError {
        #[from]
        source: github_client::Error,
    },
}
```

**Error Behavior**:

- PolicyNotFound: Organization has no visibility policy configured
- PolicyViolation: User/template preference violates organization policy
- GitHubConstraint: Requested visibility not available on plan
- ConfigurationError: Invalid policy configuration
- GitHubApiError: Wrapped GitHub API errors

**User Messages**:

- PolicyNotFound: "Contact organization admin to configure visibility policy"
- PolicyViolation: "Organization requires {required} repositories" or "Organization prohibits {prohibited} repositories"
- GitHubConstraint: "Internal repositories require GitHub Enterprise" or "Private repositories require paid plan"
- ConfigurationError: "Invalid policy configuration: {details}"
- GitHubApiError: "GitHub API error: {details}"

---

## Integration with Repository Creation

The visibility resolution integrates with the repository creation flow:

```rust
// In RepositoryCreationRequest
pub struct RepositoryCreationRequest {
    pub name: RepositoryName,
    pub owner: OrganizationName,
    pub template: TemplateName,
    pub variables: HashMap<String, String>,

    /// Explicit visibility preference (optional)
    pub visibility: Option<RepositoryVisibility>,
}

// In create_repository orchestration
pub async fn create_repository(
    request: RepositoryCreationRequest,
    // ... other parameters
) -> Result<RepositoryCreationResult, RepoRollerError> {
    // 1. Load template configuration (includes template default visibility)
    let template_config = load_template_config(&request.template).await?;

    // 2. Resolve visibility
    let visibility_request = VisibilityRequest {
        organization: request.owner.clone(),
        user_preference: request.visibility,
        template_default: template_config.default_visibility,
    };

    let visibility_decision = visibility_resolver
        .resolve_visibility(visibility_request)
        .await?;

    // 3. Create repository with resolved visibility
    let repo_payload = RepositoryCreatePayload {
        name: request.name.as_str().to_string(),
        private: match visibility_decision.visibility {
            RepositoryVisibility::Public => false,
            RepositoryVisibility::Private => true,
            RepositoryVisibility::Internal => true, // Handled by GitHub API
        },
        // ... other fields
    };

    // ... continue with repository creation
}
```

---

## Configuration Format

Organization visibility policies are defined in the metadata repository:

```toml
# global/defaults.toml
[repository_visibility]
# Policy enforcement level: "required", "restricted", "unrestricted"
enforcement_level = "restricted"

# Required visibility (only used when enforcement_level = "required")
# Values: "public", "private", "internal"
required_visibility = "private"

# Prohibited visibilities (only used when enforcement_level = "restricted")
# Values: array of ["public", "private", "internal"]
restricted_visibilities = ["public"]

# System default when no preference specified
# Values: "public", "private", "internal"
default_visibility = "private"
```

**Configuration Validation**:

- enforcement_level: Must be "required", "restricted", or "unrestricted"
- required_visibility: Only valid when enforcement_level = "required"
- restricted_visibilities: Only valid when enforcement_level = "restricted", must not be empty
- default_visibility: Must be valid visibility value

---

## Testing Requirements

### Unit Tests

Test files: `crates/repo_roller_core/src/visibility_tests.rs`

**Coverage Requirements**:

1. RepositoryVisibility enum
   - Serialization/deserialization
   - String conversions
   - Equality comparisons

2. VisibilityPolicy enum
   - Policy matching logic
   - Validation behavior
   - Edge cases (empty restrictions, etc.)

3. VisibilityResolver
   - Each decision source path
   - Policy enforcement (required, restricted, unrestricted)
   - GitHub constraint validation
   - Error conditions

**Test Patterns**:

```rust
#[test]
fn test_required_policy_overrides_user_preference() {
    // Mock: Policy = Required(Private)
    // Input: user_preference = Some(Public)
    // Expected: Decision = Private, source = OrganizationPolicy
}

#[test]
fn test_restricted_policy_blocks_prohibited_visibility() {
    // Mock: Policy = Restricted([Public])
    // Input: user_preference = Some(Public)
    // Expected: Error = PolicyViolation
}

#[test]
fn test_internal_requires_enterprise() {
    // Mock: is_enterprise = false
    // Input: user_preference = Some(Internal)
    // Expected: Error = GitHubConstraint
}
```

### Integration Tests

Test file: `crates/integration_tests/tests/repository_visibility_tests.rs`

**Test Scenarios** (already exist):

1. Create private repository explicitly
2. Create public repository explicitly
3. Create internal repository explicitly (Enterprise only)
4. Repository visibility from template configuration
5. Organization default when no preferences specified
6. Explicit visibility overrides template
7. Visibility hierarchy precedence
8. Update repository visibility
9. Visibility with empty repository

**Additional Scenarios Needed**:

1. Organization required policy enforcement
2. Organization restricted policy enforcement
3. GitHub plan limitations handling
4. Policy cache invalidation
5. Concurrent visibility resolution

---

## Performance Expectations

- Policy lookup (cached): <10ms
- Policy lookup (cache miss): <500ms
- Environment detection (cached): <10ms
- Environment detection (cache miss): <1s
- Complete visibility resolution (cached): <50ms
- Complete visibility resolution (cache miss): <2s

---

## Security Considerations

1. **Policy Enforcement**: Organization policies must be enforced at all times, no bypass allowed
2. **Audit Trail**: All visibility decisions must be logged with source and constraints
3. **Cache Invalidation**: Stale policies must not cause security violations
4. **Error Messages**: Must not expose sensitive organization configuration details
5. **Permission Validation**: User permissions must be checked before allowing visibility changes

---

## Implementation Notes

### Circular Dependency Resolution

**Architectural Decision**: Policy types are defined in `config_manager` to avoid circular dependencies.

**Context**: The original design placed all visibility types in `repo_roller_core`, but this created a circular dependency:
- `repo_roller_core` depends on `config_manager` (re-exports `ConfigurationError`)
- `config_manager` would need to depend on `repo_roller_core` (to implement `VisibilityPolicyProvider`)

**Solution**: Split the visibility domain across crates following the existing `ConfigurationError` pattern:
- **Policy definitions** (`RepositoryVisibility`, `VisibilityPolicy`, `VisibilityPolicyProvider`) → `config_manager`
- **Resolution logic** (`VisibilityResolver`, `VisibilityDecision`, `VisibilityRequest`) → `repo_roller_core`
- **Environment detection** (`GitHubEnvironmentDetector`, `PlanLimitations`) → `github_client`

**Precedent**: This follows the existing pattern where `repo_roller_core` re-exports types from infrastructure crates:
```rust
// crates/repo_roller_core/src/errors.rs:134
pub use config_manager::{ConfigurationError, ConfigurationResult};
```

**Trade-off**: Policy types are not in the "pure" core domain, but this pragmatic approach:
- ✅ Avoids circular dependencies
- ✅ Maintains consistency with existing patterns
- ✅ Keeps configuration concerns in the configuration crate
- ✅ Enables rapid implementation without architectural refactoring

See: `.llm/task-5.1-circular-dependency-blocker.md` for complete analysis.

### General Implementation Notes

1. **System Default**: Private is the safest default visibility
2. **Cache Duration**: Policies cached for 5 minutes, environment for 1 hour
3. **Error Recovery**: Cache invalidation on policy errors, retry with fresh data
4. **Concurrency**: All caches must use Arc<RwLock<>> for thread safety
5. **Logging**: Log all visibility decisions with full context (org, source, constraints)

---

## Dependencies on Other Tasks

- **Configuration Loading** (Task 3.0): Template default visibility from template.toml
- **Organization Settings** (Existing): Organization policy configuration
- **GitHub Client** (Existing): Environment detection via GitHub API

---

## Success Criteria

✅ RepositoryVisibility enum implemented with serialization
✅ VisibilityPolicy enum implemented with validation
✅ VisibilityResolver implements hierarchical decision process
✅ VisibilityPolicyProvider trait with caching
✅ GitHubEnvironmentDetector trait with caching
✅ All error types with clear messages
✅ VisibilityDecision includes audit trail
✅ Integration with RepositoryCreationRequest
✅ Configuration format documented
✅ Unit tests achieve >90% coverage
✅ Integration tests pass against real infrastructure
✅ Performance meets documented expectations
✅ Documentation complete with examples
