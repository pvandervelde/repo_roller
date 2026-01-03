//! Repository visibility types and policy provider trait.
//!
//! GENERATED FROM: specs/interfaces/repository-visibility.md
//!
//! This module defines visibility policy types in config_manager to avoid
//! circular dependencies. These types are re-exported by repo_roller_core.
//!
//! See specs/interfaces/repository-visibility.md#circular-dependency-resolution
//!
//! # Architectural Decision
//!
//! Policy types are defined here (in config_manager) rather than in repo_roller_core
//! to avoid circular dependency. This follows the existing pattern where repo_roller_core
//! re-exports ConfigurationError from config_manager.
//!
//! The split is:
//! - Policy definitions (this module) → config_manager
//! - Resolution logic (VisibilityResolver) → repo_roller_core
//! - Environment detection → github_client

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Note: OrganizationName will be imported from repo_roller_core by implementations
// We don't import it here to avoid circular dependency during type definition

#[cfg(test)]
#[path = "visibility_tests.rs"]
mod tests;

/// Repository visibility level (GitHub platform concept).
///
/// Represents the three visibility options available in GitHub.
/// Internal visibility is only available in GitHub Enterprise environments.
///
/// # Examples
///
/// ```rust
/// use config_manager::RepositoryVisibility;
///
/// let public = RepositoryVisibility::Public;
/// let private = RepositoryVisibility::Private;
/// let internal = RepositoryVisibility::Internal;
/// ```
///
/// # Serialization
///
/// Serializes to/from lowercase strings: "public", "private", "internal"
///
/// See: specs/interfaces/repository-visibility.md#repositoryvisibility
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

impl RepositoryVisibility {
    /// Convert visibility to string representation.
    ///
    /// # Returns
    ///
    /// "public", "private", or "internal"
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Internal => "internal",
        }
    }

    /// Convert visibility to boolean for GitHub API.
    ///
    /// Used when creating repositories via GitHub API which uses
    /// the `private` boolean field.
    ///
    /// # Returns
    ///
    /// `true` for Private/Internal, `false` for Public
    pub fn is_private(&self) -> bool {
        match self {
            Self::Public => false,
            Self::Private | Self::Internal => true,
        }
    }
}

/// Organization-level visibility policy.
///
/// Defines how repository visibility is controlled at the organization level.
/// Policies can require specific visibility, restrict certain options, or allow
/// unrestricted choice.
///
/// # Examples
///
/// ```rust
/// use config_manager::{VisibilityPolicy, RepositoryVisibility};
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
///
/// See: specs/interfaces/repository-visibility.md#visibilitypolicy
#[derive(Debug, Clone, PartialEq)]
pub enum VisibilityPolicy {
    /// Forces a specific visibility for all repositories
    Required(RepositoryVisibility),

    /// Prohibits specific visibility options (others are allowed)
    Restricted(Vec<RepositoryVisibility>),

    /// Allows any visibility choice
    Unrestricted,
}

impl VisibilityPolicy {
    /// Check if a visibility is allowed by this policy.
    ///
    /// # Arguments
    ///
    /// * `visibility` - Visibility to check
    ///
    /// # Returns
    ///
    /// `true` if the visibility is allowed, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::{VisibilityPolicy, RepositoryVisibility};
    ///
    /// let required = VisibilityPolicy::Required(RepositoryVisibility::Private);
    /// assert!(required.allows(RepositoryVisibility::Private));
    /// assert!(!required.allows(RepositoryVisibility::Public));
    ///
    /// let restricted = VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public]);
    /// assert!(restricted.allows(RepositoryVisibility::Private));
    /// assert!(!restricted.allows(RepositoryVisibility::Public));
    ///
    /// let unrestricted = VisibilityPolicy::Unrestricted;
    /// assert!(unrestricted.allows(RepositoryVisibility::Public));
    /// assert!(unrestricted.allows(RepositoryVisibility::Private));
    /// ```
    pub fn allows(&self, visibility: RepositoryVisibility) -> bool {
        match self {
            Self::Required(required) => *required == visibility,
            Self::Restricted(prohibited) => !prohibited.contains(&visibility),
            Self::Unrestricted => true,
        }
    }

    /// Get the required visibility if this is a Required policy.
    ///
    /// # Returns
    ///
    /// `Some(visibility)` if Required policy, `None` otherwise
    pub fn required_visibility(&self) -> Option<RepositoryVisibility> {
        match self {
            Self::Required(visibility) => Some(*visibility),
            _ => None,
        }
    }
}

/// Constraint that was applied during visibility resolution.
///
/// Documents which constraints influenced the visibility decision
/// for audit and troubleshooting purposes.
///
/// See: specs/interfaces/repository-visibility.md#policyconstraint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Visibility-related errors.
///
/// Provides detailed context for visibility-related failures.
///
/// See: specs/interfaces/repository-visibility.md#error-types
#[derive(Debug, thiserror::Error)]
pub enum VisibilityError {
    /// Organization policy not found
    #[error("No visibility policy configured for organization: {organization}")]
    PolicyNotFound { organization: String },

    /// Requested visibility violates organization policy
    #[error("Visibility {requested:?} violates organization policy: {policy}")]
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
    ConfigurationError { message: String },

    /// GitHub API error during visibility resolution
    #[error("GitHub API error: {0}")]
    GitHubApiError(String),
}

/// Provider for organization visibility policies.
///
/// Implementations fetch and cache organization-level visibility policies
/// from the configuration system.
///
/// # Implementation Requirements
///
/// - Must implement caching (max age: 5 minutes)
/// - Must handle concurrent access safely
/// - Must refresh stale data automatically
/// - Must provide clear error messages
///
/// See: specs/interfaces/repository-visibility.md#visibilitypolicyprovider
#[async_trait]
pub trait VisibilityPolicyProvider: Send + Sync {
    /// Get the visibility policy for an organization.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name (as &str)
    ///
    /// # Returns
    ///
    /// Organization's visibility policy
    ///
    /// # Errors
    ///
    /// * `VisibilityError::PolicyNotFound` - Organization has no policy configured
    /// * `VisibilityError::ConfigurationError` - Policy configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use config_manager::{VisibilityPolicyProvider, VisibilityPolicy};
    /// # async fn example(provider: &dyn VisibilityPolicyProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// let policy = provider.get_policy("my-org").await?;
    ///
    /// match policy {
    ///     VisibilityPolicy::Required(vis) => println!("Required: {:?}", vis),
    ///     VisibilityPolicy::Restricted(prohibited) => println!("Restricted: {:?}", prohibited),
    ///     VisibilityPolicy::Unrestricted => println!("Unrestricted"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_policy(&self, organization: &str) -> Result<VisibilityPolicy, VisibilityError>;

    /// Invalidate cached policy for an organization.
    ///
    /// Forces the next `get_policy` call to fetch fresh policy data.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name (as &str)
    async fn invalidate_cache(&self, organization: &str);
}
