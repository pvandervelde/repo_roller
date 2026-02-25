//! Permission types for the multi-level permissions system.
//!
//! This module provides the core types for managing repository access permissions
//! across organizational, repository type, template, and user levels.
//!
//! ## Permission Hierarchy
//!
//! Permissions flow through four layers, each narrowing what is allowed:
//!
//! ```text
//! Organization Baseline (floor — cannot go below)
//!   └─ Repository Type (maximum allowed for this type)
//!       └─ Template (required for template functionality)
//!           └─ User Request (requested by the creator)
//! ```
//!
//! ## Key Types
//!
//! - [`AccessLevel`] — the privilege level (None → Read → Triage → Write → Maintain → Admin)
//! - [`PermissionType`] — what kind of action is controlled (Pull, Push, Admin, …)
//! - [`PermissionScope`] — who/what the permission applies to (Team, User, …)
//! - [`PermissionGrant`] — a concrete grant of a level for a type+scope pair
//! - [`PermissionRequest`] — an incoming request to apply permissions to a repository
//! - [`PermissionHierarchy`] — the full four-layer hierarchy for evaluation
//!
//! See `docs/spec/design/multi-level-permissions.md` for the full specification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{OrganizationName, RepositoryName};

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod tests;

// ── Access levels ─────────────────────────────────────────────────────────────

/// The privilege level granted by a permission.
///
/// Variants are ordered from lowest (`None`) to highest (`Admin`) so that
/// comparison operators (`<`, `>`, etc.) work intuitively:
///
/// ```rust
/// use repo_roller_core::permissions::AccessLevel;
/// assert!(AccessLevel::Admin > AccessLevel::Write);
/// assert!(AccessLevel::None < AccessLevel::Read);
/// ```
///
/// Maps to GitHub's collaborator permission levels when converting via
/// [`GitHubPermissionLevel::from`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// No access — explicitly excludes access.
    None,
    /// Read-only (GitHub: "pull").
    Read,
    /// Read + triage issues and pull requests (GitHub: "triage").
    Triage,
    /// Read + write code, issues, pull requests (GitHub: "push").
    Write,
    /// Read + write + manage some repository settings (GitHub: "maintain").
    Maintain,
    /// Full repository administration (GitHub: "admin").
    Admin,
}

// ── Permission type ───────────────────────────────────────────────────────────

/// The GitHub repository permission type being controlled.
///
/// These mirror GitHub's collaborator permission strings as semantic labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    /// Read-only pull access.
    Pull,
    /// Issue and pull request triage access.
    Triage,
    /// Code push (write) access.
    Push,
    /// Repository maintenance access.
    Maintain,
    /// Full administrative access.
    Admin,
}

// ── Permission scope ──────────────────────────────────────────────────────────

/// What entity a permission grant applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    /// The permission applies to the repository itself.
    Repository,
    /// The permission applies to a GitHub team.
    Team,
    /// The permission applies to an individual GitHub user.
    User,
    /// The permission applies to a GitHub App installation.
    GitHubApp,
}

// ── GitHub API permission level ───────────────────────────────────────────────

/// GitHub REST API permission level string.
///
/// Used when calling GitHub APIs that accept permission levels as strings.
/// Convert from [`AccessLevel`] via the [`From`] implementation:
///
/// ```rust
/// use repo_roller_core::permissions::{AccessLevel, GitHubPermissionLevel};
/// let level = GitHubPermissionLevel::from(AccessLevel::Write);
/// assert_eq!(level.as_str(), "push");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitHubPermissionLevel {
    /// Read-only (GitHub API string: "pull").
    Pull,
    /// Triage (GitHub API string: "triage").
    Triage,
    /// Write (GitHub API string: "push").
    Push,
    /// Maintain (GitHub API string: "maintain").
    Maintain,
    /// Admin (GitHub API string: "admin").
    Admin,
}

impl GitHubPermissionLevel {
    /// Returns the GitHub REST API string for this permission level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permissions::GitHubPermissionLevel;
    /// assert_eq!(GitHubPermissionLevel::Push.as_str(), "push");
    /// assert_eq!(GitHubPermissionLevel::Admin.as_str(), "admin");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            GitHubPermissionLevel::Pull => "pull",
            GitHubPermissionLevel::Triage => "triage",
            GitHubPermissionLevel::Push => "push",
            GitHubPermissionLevel::Maintain => "maintain",
            GitHubPermissionLevel::Admin => "admin",
        }
    }
}

impl From<AccessLevel> for GitHubPermissionLevel {
    /// Converts an [`AccessLevel`] to the closest matching [`GitHubPermissionLevel`].
    ///
    /// `AccessLevel::None` maps to `Pull` (the minimum representable GitHub
    /// level) because GitHub has no "no access" API parameter for collaborators
    /// — removing a collaborator is a separate operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permissions::{AccessLevel, GitHubPermissionLevel};
    /// assert_eq!(GitHubPermissionLevel::from(AccessLevel::Write), GitHubPermissionLevel::Push);
    /// assert_eq!(GitHubPermissionLevel::from(AccessLevel::Admin), GitHubPermissionLevel::Admin);
    /// ```
    fn from(level: AccessLevel) -> Self {
        match level {
            // GitHub has no "no access" API string for collaborators;
            // removing a collaborator is a distinct operation. Map None → Pull
            // (the minimum representable level) so callers can detect this case
            // and issue a remove operation instead.
            AccessLevel::None => GitHubPermissionLevel::Pull,
            AccessLevel::Read => GitHubPermissionLevel::Pull,
            AccessLevel::Triage => GitHubPermissionLevel::Triage,
            AccessLevel::Write => GitHubPermissionLevel::Push,
            AccessLevel::Maintain => GitHubPermissionLevel::Maintain,
            AccessLevel::Admin => GitHubPermissionLevel::Admin,
        }
    }
}

// ── Condition ─────────────────────────────────────────────────────────────────

/// A condition that must be satisfied for a permission grant to apply.
///
/// Conditions allow permissions to be contextually applied based on
/// repository properties such as visibility, type, or other attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PermissionCondition {
    /// Human-readable description of the condition.
    pub description: String,
    /// The condition expression (reserved for future conditional logic).
    pub expression: Option<String>,
}

// ── Duration ──────────────────────────────────────────────────────────────────

/// Specifies a finite duration for a temporary permission grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionDuration {
    /// Duration in seconds.
    pub seconds: i64,
}

impl PermissionDuration {
    /// Creates a new `PermissionDuration` from a number of seconds.
    pub fn from_seconds(seconds: i64) -> Self {
        Self { seconds }
    }
}

// ── Core grant ────────────────────────────────────────────────────────────────

/// A concrete permission grant for a specific type, scope, and access level.
///
/// This is the atomic unit of the permission system. Grants can be time-limited
/// via `expiration` and conditionally applied via `conditions`.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permissions::{AccessLevel, PermissionGrant, PermissionScope, PermissionType};
///
/// let grant = PermissionGrant {
///     conditions: vec![],
///     expiration: None,
///     level: AccessLevel::Write,
///     permission_type: PermissionType::Push,
///     scope: PermissionScope::Team,
/// };
/// assert!(!grant.is_expired());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Conditions that must be met for this grant to apply.
    pub conditions: Vec<PermissionCondition>,
    /// Optional UTC timestamp after which this grant expires.
    pub expiration: Option<DateTime<Utc>>,
    /// The privilege level being granted.
    pub level: AccessLevel,
    /// The type of permission being granted.
    pub permission_type: PermissionType,
    /// The scope to which this grant applies.
    pub scope: PermissionScope,
}

impl PermissionGrant {
    /// Returns `true` if this grant has an expiration that is in the past.
    ///
    /// Grants without an expiration never expire.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permissions::{AccessLevel, PermissionGrant, PermissionScope, PermissionType};
    /// use chrono::Utc;
    ///
    /// let non_expiring = PermissionGrant {
    ///     conditions: vec![],
    ///     expiration: None,
    ///     level: AccessLevel::Read,
    ///     permission_type: PermissionType::Pull,
    ///     scope: PermissionScope::User,
    /// };
    /// assert!(!non_expiring.is_expired());
    ///
    /// let expired = PermissionGrant {
    ///     conditions: vec![],
    ///     expiration: Some(Utc::now() - chrono::Duration::hours(1)),
    ///     level: AccessLevel::Read,
    ///     permission_type: PermissionType::Pull,
    ///     scope: PermissionScope::User,
    /// };
    /// assert!(expired.is_expired());
    /// ```
    pub fn is_expired(&self) -> bool {
        match self.expiration {
            None => false,
            Some(expiry) => expiry <= Utc::now(),
        }
    }
}

// ── Repository context ────────────────────────────────────────────────────────

/// Identifies the repository for which permissions are being requested.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryContext {
    /// The GitHub organization that owns the repository.
    pub organization: OrganizationName,
    /// The repository name within the organization.
    pub repository: RepositoryName,
}

// ── User-requested permissions ────────────────────────────────────────────────

/// Permissions explicitly requested by the user creating the repository.
///
/// These sit at the lowest layer of the hierarchy and are subject to
/// validation against all upper layers before being applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserPermissionRequests {
    /// The list of permission grants requested by the user.
    pub permissions: Vec<PermissionGrant>,
}

// ── Hierarchy level: Organization ─────────────────────────────────────────────

/// Organization-wide permission policies — the top layer of the hierarchy.
///
/// Defines the minimum (baseline) permissions that all repositories must have
/// and any maximum restrictions that cannot be exceeded within the organization.
///
/// This is a skeleton type; it is expanded with full policy fields in task 12.6
/// when permission configuration support is added to `GlobalDefaults`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OrganizationPermissionPolicies {
    /// Minimum permissions that must be present on every repository.
    pub baseline_requirements: Vec<PermissionGrant>,
    /// Maximum permission grants; requests exceeding these are denied.
    pub restrictions: Vec<PermissionGrant>,
}

// ── Hierarchy level: Repository Type ─────────────────────────────────────────

/// Permission constraints specific to a repository type.
///
/// This is a skeleton type; it is expanded in task 12.6 when permission
/// configuration support is added to `RepositoryTypeConfig`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RepositoryTypePermissions {
    /// Permissions that all repositories of this type must have.
    pub required_permissions: Vec<PermissionGrant>,
    /// Permission types that are not allowed for this repository type.
    pub restricted_types: Vec<PermissionType>,
}

// ── Hierarchy level: Template ─────────────────────────────────────────────────

/// Permission requirements defined by a template.
///
/// This is a skeleton type; it is expanded in task 12.6 when permission
/// configuration support is added to `NewTemplateConfig`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TemplatePermissions {
    /// Permissions required for the template to function correctly.
    pub required_permissions: Vec<PermissionGrant>,
}

// ── Full hierarchy ────────────────────────────────────────────────────────────

/// The complete four-layer permission hierarchy for a repository.
///
/// Assembles all permission layers so that [`crate::policy_engine::PolicyEngine`]
/// can evaluate a [`PermissionRequest`] against all constraints in a single pass.
///
/// # Hierarchy Precedence (highest to lowest)
///
/// 1. `organization_policies` — absolute floor and ceiling
/// 2. `repository_type_permissions` — type-specific limits
/// 3. `template_permissions` — template requirements
/// 4. `user_requested_permissions` — user-supplied requests
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permissions::{
///     OrganizationPermissionPolicies, PermissionHierarchy, UserPermissionRequests,
/// };
///
/// let hierarchy = PermissionHierarchy {
///     organization_policies: OrganizationPermissionPolicies::default(),
///     repository_type_permissions: None,
///     template_permissions: None,
///     user_requested_permissions: UserPermissionRequests::default(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionHierarchy {
    /// Organization-wide policies — applied to every repository.
    pub organization_policies: OrganizationPermissionPolicies,
    /// Optional type-specific permissions; `None` when no type is configured.
    pub repository_type_permissions: Option<RepositoryTypePermissions>,
    /// Optional template permissions; `None` when creating an empty repository.
    pub template_permissions: Option<TemplatePermissions>,
    /// Permissions explicitly requested by the repository creator.
    pub user_requested_permissions: UserPermissionRequests,
}

// ── Permission request ────────────────────────────────────────────────────────

/// A request to apply permissions to a repository.
///
/// Submitted to the `PolicyEngine` for evaluation against the full hierarchy
/// before permissions are applied via the `PermissionManager`.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permissions::{PermissionRequest, RepositoryContext};
/// use repo_roller_core::{OrganizationName, RepositoryName};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = PermissionRequest {
///     duration: None,
///     emergency_access: false,
///     justification: "Standard team setup".to_string(),
///     repository_context: RepositoryContext {
///         organization: OrganizationName::new("my-org")?,
///         repository: RepositoryName::new("my-repo")?,
///     },
///     requested_permissions: vec![],
///     requestor: "jsmith".to_string(),
/// };
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    /// Optional time limit for the granted permissions.
    pub duration: Option<PermissionDuration>,
    /// If `true`, bypasses approval workflows for emergency access.
    pub emergency_access: bool,
    /// Human-readable justification for the permission request.
    pub justification: String,
    /// Identifies the target repository.
    pub repository_context: RepositoryContext,
    /// The specific permission grants being requested.
    pub requested_permissions: Vec<PermissionGrant>,
    /// GitHub username of the person requesting permissions.
    pub requestor: String,
}

// ── Permission error ──────────────────────────────────────────────────────────

/// Errors that can occur during permission evaluation and application.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PermissionError {
    /// The requested permissions are below the organization baseline.
    #[error(
        "Permission {permission_type:?}/{level:?} is below the organization baseline requirement"
    )]
    BelowBaseline {
        /// The permission type that failed the baseline check.
        permission_type: PermissionType,
        /// The access level that was requested.
        level: AccessLevel,
        /// The minimum level required by the organization baseline.
        minimum_required: AccessLevel,
    },

    /// The requested permissions exceed organization limits.
    #[error(
        "Permission {permission_type:?}/{level:?} exceeds the organization maximum of {maximum_allowed:?}"
    )]
    ExceedsOrganizationLimits {
        /// The permission type that exceeded limits.
        permission_type: PermissionType,
        /// The level that was requested.
        level: AccessLevel,
        /// The maximum level allowed by organization policy.
        maximum_allowed: AccessLevel,
    },

    /// A required template permission is not satisfiable within org policy.
    #[error(
        "Template requires permission {permission_type:?}/{level:?} but organization policy does not allow it"
    )]
    TemplateRequirementConflict {
        /// The permission type required by the template.
        permission_type: PermissionType,
        /// The level required by the template.
        level: AccessLevel,
    },
}
