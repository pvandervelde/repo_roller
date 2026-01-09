//! GitHub environment detection types and trait.
//!
//! GENERATED FROM: specs/interfaces/repository-visibility.md
//!
//! This module provides types and traits for detecting GitHub environment
//! capabilities, such as whether an organization is on GitHub Enterprise
//! and what plan limitations apply for repository visibility.
//!
//! See specs/interfaces/repository-visibility.md for complete specification.

use async_trait::async_trait;

use crate::Error;

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;

/// GitHub plan limitations affecting visibility.
///
/// Contains information about what visibility options are available
/// based on the organization's GitHub plan and environment.
///
/// # Examples
///
/// ```rust
/// use github_client::PlanLimitations;
///
/// let limitations = PlanLimitations {
///     supports_private_repos: true,
///     supports_internal_repos: true,
///     private_repo_limit: None,  // Unlimited
///     is_enterprise: true,
/// };
///
/// if limitations.supports_internal_repos {
///     println!("Internal repositories are available");
/// }
/// ```
///
/// See: specs/interfaces/repository-visibility.md#planlimitations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanLimitations {
    /// Whether private repositories are supported
    pub supports_private_repos: bool,

    /// Whether internal repositories are supported (GitHub Enterprise only)
    pub supports_internal_repos: bool,

    /// Maximum number of private repositories (None = unlimited)
    pub private_repo_limit: Option<u32>,

    /// Whether this is a GitHub Enterprise environment
    pub is_enterprise: bool,
}

impl PlanLimitations {
    /// Create plan limitations for a free plan.
    ///
    /// Free plans have no internal repositories.
    pub fn free_plan() -> Self {
        Self {
            supports_private_repos: true,
            supports_internal_repos: false,
            private_repo_limit: None, // Unlimited private repos
            is_enterprise: false,
        }
    }

    /// Create plan limitations for a paid (non-Enterprise) plan.
    ///
    /// Paid plans support private repositories but not internal repositories.
    pub fn paid_plan() -> Self {
        Self {
            supports_private_repos: true,
            supports_internal_repos: false,
            private_repo_limit: None, // Unlimited private repos
            is_enterprise: false,
        }
    }

    /// Create plan limitations for GitHub Enterprise.
    ///
    /// Enterprise environments support all visibility options including internal.
    pub fn enterprise() -> Self {
        Self {
            supports_private_repos: true,
            supports_internal_repos: true,
            private_repo_limit: None, // Unlimited
            is_enterprise: true,
        }
    }
}

/// Detects GitHub environment capabilities and limitations.
///
/// Implementations interact with GitHub APIs to determine what visibility
/// options are available based on the organization's plan and environment.
///
/// # Implementation Requirements
///
/// - Must cache results (max age: 1 hour)
/// - Must handle GitHub API rate limits gracefully
/// - Must detect environment from API responses
/// - Must fall back safely when detection fails
///
/// See: specs/interfaces/repository-visibility.md#githubenvironmentdetector
#[async_trait]
pub trait GitHubEnvironmentDetector: Send + Sync {
    /// Get plan limitations for an organization.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name (as &str)
    ///
    /// # Returns
    ///
    /// Plan limitations affecting visibility options
    ///
    /// # Errors
    ///
    /// * `Error::ApiError` - GitHub API request failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use github_client::GitHubEnvironmentDetector;
    /// # async fn example(detector: &dyn GitHubEnvironmentDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// let limitations = detector.get_plan_limitations("my-org").await?;
    ///
    /// if limitations.supports_internal_repos {
    ///     println!("Internal visibility is available");
    /// } else {
    ///     println!("Internal visibility requires GitHub Enterprise");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_plan_limitations(&self, organization: &str) -> Result<PlanLimitations, Error>;

    /// Check if organization is in GitHub Enterprise environment.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name (as &str)
    ///
    /// # Returns
    ///
    /// `true` if organization is in GitHub Enterprise
    ///
    /// # Errors
    ///
    /// * `Error::ApiError` - GitHub API request failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use github_client::GitHubEnvironmentDetector;
    /// # async fn example(detector: &dyn GitHubEnvironmentDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// if detector.is_enterprise("my-org").await? {
    ///     println!("Organization is on GitHub Enterprise");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn is_enterprise(&self, organization: &str) -> Result<bool, Error>;
}
