//! Configuration verification helpers for integration tests.
//!
//! This module provides utilities to verify that repository configuration
//! was actually applied to GitHub repositories after creation. It addresses
//! the critical gap where tests only checked `results.success` but didn't
//! verify that settings were actually applied.

use anyhow::Result;
use github_client::GitHubClient;
use std::collections::HashMap;

/// Results of configuration verification against actual GitHub repository state.
#[derive(Debug, Clone)]
pub struct ConfigurationVerification {
    /// Overall verification passed
    pub passed: bool,
    /// Repository settings verification results
    pub settings_verified: bool,
    /// Custom properties verification results
    pub custom_properties_verified: bool,
    /// Branch protection verification results
    pub branch_protection_verified: bool,
    /// Labels verification results
    pub labels_verified: bool,
    /// Detailed verification failures
    pub failures: Vec<String>,
}

impl ConfigurationVerification {
    /// Create a successful verification result
    pub fn success() -> Self {
        Self {
            passed: true,
            settings_verified: true,
            custom_properties_verified: true,
            branch_protection_verified: true,
            labels_verified: true,
            failures: Vec::new(),
        }
    }

    /// Create a failed verification result with reason
    pub fn failure(reason: String) -> Self {
        Self {
            passed: false,
            settings_verified: false,
            custom_properties_verified: false,
            branch_protection_verified: false,
            labels_verified: false,
            failures: vec![reason],
        }
    }

    /// Add a failure reason to this verification
    pub fn add_failure(&mut self, reason: String) {
        self.passed = false;
        self.failures.push(reason);
    }
}

/// Expected configuration to verify against actual repository state.
#[derive(Debug, Clone)]
pub struct ExpectedConfiguration {
    /// Expected repository feature settings
    pub repository_settings: Option<ExpectedRepositorySettings>,
    /// Expected custom properties
    pub custom_properties: Option<HashMap<String, String>>,
    /// Expected branch protection rules
    pub branch_protection: Option<ExpectedBranchProtection>,
    /// Expected labels
    pub labels: Option<Vec<String>>,
}

/// Expected repository feature settings.
#[derive(Debug, Clone)]
pub struct ExpectedRepositorySettings {
    /// Whether issues should be enabled
    pub has_issues: Option<bool>,
    /// Whether wiki should be enabled
    pub has_wiki: Option<bool>,
    /// Whether discussions should be enabled
    pub has_discussions: Option<bool>,
    /// Whether projects should be enabled
    pub has_projects: Option<bool>,
}

/// Expected branch protection rules.
#[derive(Debug, Clone)]
pub struct ExpectedBranchProtection {
    /// Branch name to protect (e.g., "main", "master")
    pub branch: String,
    /// Required number of approving reviews
    pub required_approving_review_count: Option<u32>,
    /// Require code owner reviews
    pub require_code_owner_reviews: Option<bool>,
    /// Dismiss stale reviews on push
    pub dismiss_stale_reviews: Option<bool>,
}

/// Verify repository settings match expected configuration.
///
/// # Arguments
///
/// * `client` - GitHub client for API calls
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `expected` - Expected repository settings
///
/// # Returns
///
/// Verification result indicating whether settings match
pub async fn verify_repository_settings(
    _client: &GitHubClient,
    _owner: &str,
    _repo: &str,
    _expected: &ExpectedRepositorySettings,
) -> Result<ConfigurationVerification> {
    // TODO: Implement repository settings verification
    // Will require adding get_repository_settings() to GitHubClient
    Ok(ConfigurationVerification::failure(
        "Repository settings verification not yet implemented".to_string(),
    ))
}

/// Verify custom properties match expected configuration.
///
/// # Arguments
///
/// * `client` - GitHub client for API calls
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `expected` - Expected custom properties map
///
/// # Returns
///
/// Verification result indicating whether custom properties match
pub async fn verify_custom_properties(
    _client: &GitHubClient,
    _owner: &str,
    _repo: &str,
    _expected: &HashMap<String, String>,
) -> Result<ConfigurationVerification> {
    // TODO: Implement custom properties verification
    // Will require adding get_custom_properties() to GitHubClient
    Ok(ConfigurationVerification::failure(
        "Custom properties verification not yet implemented".to_string(),
    ))
}

/// Verify branch protection rules match expected configuration.
///
/// # Arguments
///
/// * `client` - GitHub client for API calls
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `expected` - Expected branch protection rules
///
/// # Returns
///
/// Verification result indicating whether branch protection matches
pub async fn verify_branch_protection(
    _client: &GitHubClient,
    _owner: &str,
    _repo: &str,
    _expected: &ExpectedBranchProtection,
) -> Result<ConfigurationVerification> {
    // TODO: Implement branch protection verification
    // Will require adding get_branch_protection() to GitHubClient
    Ok(ConfigurationVerification::failure(
        "Branch protection verification not yet implemented".to_string(),
    ))
}

/// Verify labels match expected configuration.
///
/// # Arguments
///
/// * `client` - GitHub client for API calls
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `expected` - Expected label names
///
/// # Returns
///
/// Verification result indicating whether labels match
pub async fn verify_labels(
    _client: &GitHubClient,
    _owner: &str,
    _repo: &str,
    _expected: &[String],
) -> Result<ConfigurationVerification> {
    // TODO: Implement labels verification
    // Will require adding list_repository_labels() to GitHubClient
    Ok(ConfigurationVerification::failure(
        "Labels verification not yet implemented".to_string(),
    ))
}

/// Load expected configuration from metadata repository for comparison.
///
/// # Arguments
///
/// * `client` - GitHub client for API calls
/// * `org` - Organization name
/// * `metadata_repo` - Metadata repository name
/// * `scenario` - Test scenario to load configuration for
///
/// # Returns
///
/// Expected configuration for the given scenario
pub async fn load_expected_configuration(
    _client: &GitHubClient,
    _org: &str,
    _metadata_repo: &str,
    _scenario: &crate::TestScenario,
) -> Result<ExpectedConfiguration> {
    // TODO: Implement loading expected configuration from metadata repository
    // This will parse TOML files and construct ExpectedConfiguration
    Ok(ExpectedConfiguration {
        repository_settings: None,
        custom_properties: None,
        branch_protection: None,
        labels: None,
    })
}

#[cfg(test)]
#[path = "verification_tests.rs"]
mod tests;
