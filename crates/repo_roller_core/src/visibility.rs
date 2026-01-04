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
//! # Type Organization
//!
//! **Policy types** (defined in config_manager, re-exported here):
//! - `RepositoryVisibility` - Public/Private/Internal enum
//! - `VisibilityPolicy` - Required/Restricted/Unrestricted policies
//! - `PolicyConstraint` - Constraint tracking
//! - `VisibilityError` - Error types
//! - `VisibilityPolicyProvider` - Policy provider trait
//!
//! **Resolution types** (defined in this module):
//! - `DecisionSource` - Hierarchy level that made decision
//! - `VisibilityDecision` - Resolution result with audit trail
//! - `VisibilityRequest` - Input to resolution
//! - `PlanLimitations` - GitHub plan constraints
//! - `GitHubEnvironmentDetector` - Environment detection trait
//! - `VisibilityResolver` - Orchestrator implementation
//!
//! This split avoids circular dependencies. See specs/interfaces/repository-visibility.md
//! for complete architectural rationale.
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
use std::sync::Arc;

use crate::OrganizationName;

// Re-export policy types from config_manager to avoid circular dependency
// This follows the existing pattern with ConfigurationError
pub use config_manager::{
    PolicyConstraint, RepositoryVisibility, VisibilityError, VisibilityPolicy,
    VisibilityPolicyProvider,
};

// Re-export environment detection from github_client
pub use github_client::{GitHubEnvironmentDetector, PlanLimitations};

#[cfg(test)]
#[path = "visibility_tests.rs"]
mod tests;

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
