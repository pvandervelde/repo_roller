//! Repository visibility types and resolution.
//!
//! This module handles the hierarchical visibility decision process for repository
//! creation, implementing organization policies, user preferences, template defaults,
//! and GitHub platform constraints.
//!
//! # Architecture
//!
//! The visibility system uses a clear hierarchy:
//! 1. Organization Policy (required) - Mandatory visibility
//! 2. User Preference - Explicit user choice (if allowed by policy)
//! 3. Template Default - Template's configured visibility
//! 4. System Default - Fallback (Private)
//!
//! All decisions are validated against GitHub platform constraints (Enterprise, plan limits).
//!
//! # Examples
//!
//! ```rust,no_run
//! use repo_roller_core::{
//!     VisibilityResolver, VisibilityRequest, RepositoryVisibility, OrganizationName
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let resolver = VisibilityResolver::new(policy_provider, environment_detector);
//!
//! let request = VisibilityRequest {
//!     organization: OrganizationName::new("my-org")?,
//!     user_preference: Some(RepositoryVisibility::Private),
//!     template_default: None,
//! };
//!
//! let decision = resolver.resolve_visibility(request).await?;
//! println!("Visibility: {:?}, Source: {:?}", decision.visibility, decision.source);
//! # Ok(())
//! # }
//! ```
//!
//! See specs/interfaces/repository-visibility.md for complete interface specification.
//! GENERATED FROM: specs/interfaces/repository-visibility.md

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::OrganizationName;

#[cfg(test)]
#[path = "visibility_tests.rs"]
mod tests;

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

/// Source of the visibility decision.
///
/// Indicates which level of the hierarchy determined the final visibility.
///
/// See: specs/interfaces/repository-visibility.md#decisionsource
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
///
/// See: specs/interfaces/repository-visibility.md#visibilitydecision
#[derive(Debug, Clone)]
pub struct VisibilityDecision {
    /// The determined visibility
    pub visibility: RepositoryVisibility,

    /// Source of the decision in the hierarchy
    pub source: DecisionSource,

    /// Constraints that were applied during resolution
    pub constraints_applied: Vec<PolicyConstraint>,
}

/// Input to the visibility resolution process.
///
/// Contains all information needed to determine repository visibility.
///
/// See: specs/interfaces/repository-visibility.md#visibilityrequest
#[derive(Debug, Clone)]
pub struct VisibilityRequest {
    /// Organization where repository will be created
    pub organization: OrganizationName,

    /// User's explicit visibility preference (optional)
    pub user_preference: Option<RepositoryVisibility>,

    /// Template's default visibility (optional)
    pub template_default: Option<RepositoryVisibility>,
}

/// GitHub plan limitations affecting visibility.
///
/// Contains information about what visibility options are available
/// based on the organization's GitHub plan and environment.
///
/// See: specs/interfaces/repository-visibility.md#planlimitations
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

/// Provides organization visibility policies.
///
/// Implementations fetch and cache organization-level visibility policies
/// from the configuration system.
///
/// See: specs/interfaces/repository-visibility.md#visibilitypolicyprovider
#[async_trait]
pub trait VisibilityPolicyProvider: Send + Sync {
    /// Get the visibility policy for an organization.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name
    ///
    /// # Returns
    ///
    /// Organization's visibility policy
    ///
    /// # Errors
    ///
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
    ///
    /// * `organization` - Organization name
    async fn invalidate_cache(&self, organization: &OrganizationName);
}

/// Detects GitHub environment capabilities and limitations.
///
/// Implementations interact with GitHub APIs to determine what visibility
/// options are available based on the organization's plan and environment.
///
/// See: specs/interfaces/repository-visibility.md#githubenvironmentdetector
#[async_trait]
pub trait GitHubEnvironmentDetector: Send + Sync {
    /// Get plan limitations for an organization.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name
    ///
    /// # Returns
    ///
    /// Plan limitations affecting visibility options
    ///
    /// # Errors
    ///
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    async fn get_plan_limitations(
        &self,
        organization: &OrganizationName,
    ) -> Result<PlanLimitations, VisibilityError>;

    /// Check if organization is in GitHub Enterprise environment.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name
    ///
    /// # Returns
    ///
    /// `true` if organization is in GitHub Enterprise
    ///
    /// # Errors
    ///
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    async fn is_enterprise(&self, organization: &OrganizationName)
        -> Result<bool, VisibilityError>;
}

/// Resolves repository visibility based on policies and preferences.
///
/// Implements the hierarchical visibility decision process, validating
/// against organization policies and GitHub platform constraints.
///
/// # Examples
///
/// ```rust,no_run
/// use repo_roller_core::{VisibilityResolver, VisibilityRequest, RepositoryVisibility, OrganizationName};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
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
/// # Ok(())
/// # }
/// ```
///
/// See: specs/interfaces/repository-visibility.md#visibilityresolver
pub struct VisibilityResolver {
    policy_provider: Arc<dyn VisibilityPolicyProvider>,
    environment_detector: Arc<dyn GitHubEnvironmentDetector>,
}

impl VisibilityResolver {
    /// Create a new visibility resolver.
    ///
    /// # Arguments
    ///
    /// * `policy_provider` - Provider for organization policies
    /// * `environment_detector` - Detector for GitHub environment capabilities
    ///
    /// See: specs/interfaces/repository-visibility.md#visibilityresolver
    pub fn new(
        policy_provider: Arc<dyn VisibilityPolicyProvider>,
        environment_detector: Arc<dyn GitHubEnvironmentDetector>,
    ) -> Self {
        Self {
            policy_provider,
            environment_detector,
        }
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
    ///
    /// * `request` - Visibility resolution request with preferences
    ///
    /// # Returns
    ///
    /// Visibility decision with audit trail
    ///
    /// # Errors
    ///
    /// * `VisibilityError::PolicyViolation` - Requested visibility violates policy
    /// * `VisibilityError::GitHubConstraint` - Visibility not available on this plan
    /// * `VisibilityError::PolicyNotFound` - Organization has no policy configured
    /// * `VisibilityError::GitHubApiError` - GitHub API request failed
    ///
    /// # Performance
    ///
    /// Typical: <50ms (cached)
    /// Cache miss: <2s (requires API calls)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use repo_roller_core::{VisibilityResolver, VisibilityRequest, RepositoryVisibility, DecisionSource, OrganizationName};
    /// # async fn example(resolver: VisibilityResolver) -> Result<(), Box<dyn std::error::Error>> {
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
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: specs/interfaces/repository-visibility.md#visibilityresolver
    pub async fn resolve_visibility(
        &self,
        _request: VisibilityRequest,
    ) -> Result<VisibilityDecision, VisibilityError> {
        unimplemented!("See specs/interfaces/repository-visibility.md")
    }
}

/// Errors that can occur during visibility resolution.
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
    #[error("GitHub API error: {source}")]
    GitHubApiError {
        #[from]
        source: github_client::Error,
    },
}
