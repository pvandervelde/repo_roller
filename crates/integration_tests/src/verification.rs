//! Configuration verification helpers for integration tests.
//!
//! This module provides utilities to verify that repository configuration
//! was actually applied to GitHub repositories after creation. It addresses
//! the critical gap where tests only checked `results.success` but didn't
//! verify that settings were actually applied.

use anyhow::Result;
use github_client::{GitHubClient, RepositoryClient};
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
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    expected: &ExpectedRepositorySettings,
) -> Result<ConfigurationVerification> {
    let actual = client.get_repository_settings(owner, repo).await?;

    let mut result = ConfigurationVerification::success();
    result.settings_verified = true;

    // TODO: The Repository model needs to be extended to include has_issues, has_wiki,
    // has_discussions, and has_projects fields. These are available in the GitHub API
    // response but not currently exposed in our model.
    // For now, we return None indicating these fields aren't available.
    fn get_bool_from_repo(_repo: &github_client::models::Repository, _field: &str) -> Option<bool> {
        None
    }

    // Check has_issues if expected
    if let Some(expected_issues) = expected.has_issues {
        match get_bool_from_repo(&actual, "has_issues") {
            Some(actual_issues) if actual_issues == expected_issues => {
                // Match - continue
            }
            Some(actual_issues) => {
                result.add_failure(format!(
                    "has_issues: expected {}, got {}",
                    expected_issues, actual_issues
                ));
            }
            None => {
                result.add_failure("has_issues: field not available in API response".to_string());
            }
        }
    }

    // Check has_wiki if expected
    if let Some(expected_wiki) = expected.has_wiki {
        match get_bool_from_repo(&actual, "has_wiki") {
            Some(actual_wiki) if actual_wiki == expected_wiki => {
                // Match - continue
            }
            Some(actual_wiki) => {
                result.add_failure(format!(
                    "has_wiki: expected {}, got {}",
                    expected_wiki, actual_wiki
                ));
            }
            None => {
                result.add_failure("has_wiki: field not available in API response".to_string());
            }
        }
    }

    // Check has_discussions if expected
    if let Some(expected_discussions) = expected.has_discussions {
        match get_bool_from_repo(&actual, "has_discussions") {
            Some(actual_discussions) if actual_discussions == expected_discussions => {
                // Match - continue
            }
            Some(actual_discussions) => {
                result.add_failure(format!(
                    "has_discussions: expected {}, got {}",
                    expected_discussions, actual_discussions
                ));
            }
            None => {
                result
                    .add_failure("has_discussions: field not available in API response".to_string());
            }
        }
    }

    // Check has_projects if expected
    if let Some(expected_projects) = expected.has_projects {
        match get_bool_from_repo(&actual, "has_projects") {
            Some(actual_projects) if actual_projects == expected_projects => {
                // Match - continue
            }
            Some(actual_projects) => {
                result.add_failure(format!(
                    "has_projects: expected {}, got {}",
                    expected_projects, actual_projects
                ));
            }
            None => {
                result.add_failure("has_projects: field not available in API response".to_string());
            }
        }
    }

    Ok(result)
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
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    expected: &HashMap<String, String>,
) -> Result<ConfigurationVerification> {
    let actual = client.get_custom_properties(owner, repo).await?;

    let mut result = ConfigurationVerification::success();
    result.custom_properties_verified = true;

    // Check that all expected properties exist with correct values
    for (key, expected_value) in expected {
        match actual.get(key) {
            Some(actual_value) if actual_value == expected_value => {
                // Match - continue
            }
            Some(actual_value) => {
                result.add_failure(format!(
                    "Custom property '{}': expected '{}', got '{}'",
                    key, expected_value, actual_value
                ));
            }
            None => {
                result.add_failure(format!("Custom property '{}' not found", key));
            }
        }
    }

    // Note: We don't fail for extra properties that weren't expected
    // This allows for future additions without breaking tests

    Ok(result)
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
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    expected: &ExpectedBranchProtection,
) -> Result<ConfigurationVerification> {
    let actual = client
        .get_branch_protection(owner, repo, &expected.branch)
        .await?;

    let mut result = ConfigurationVerification::success();
    result.branch_protection_verified = true;

    match actual {
        None => {
            result.add_failure(format!(
                "Branch protection not configured for branch '{}'",
                expected.branch
            ));
        }
        Some(protection) => {
            // Check required_approving_review_count if expected
            if let Some(expected_count) = expected.required_approving_review_count {
                match protection.required_approving_review_count {
                    Some(actual_count) if actual_count == expected_count => {
                        // Match - continue
                    }
                    Some(actual_count) => {
                        result.add_failure(format!(
                            "required_approving_review_count: expected {}, got {}",
                            expected_count, actual_count
                        ));
                    }
                    None => {
                        result.add_failure(format!(
                            "required_approving_review_count: expected {}, but not configured",
                            expected_count
                        ));
                    }
                }
            }

            // Check require_code_owner_reviews if expected
            if let Some(expected_code_owner) = expected.require_code_owner_reviews {
                match protection.require_code_owner_reviews {
                    Some(actual_code_owner) if actual_code_owner == expected_code_owner => {
                        // Match - continue
                    }
                    Some(actual_code_owner) => {
                        result.add_failure(format!(
                            "require_code_owner_reviews: expected {}, got {}",
                            expected_code_owner, actual_code_owner
                        ));
                    }
                    None => {
                        result.add_failure(format!(
                            "require_code_owner_reviews: expected {}, but not configured",
                            expected_code_owner
                        ));
                    }
                }
            }

            // Check dismiss_stale_reviews if expected
            if let Some(expected_dismiss_stale) = expected.dismiss_stale_reviews {
                match protection.dismiss_stale_reviews {
                    Some(actual_dismiss_stale) if actual_dismiss_stale == expected_dismiss_stale => {
                        // Match - continue
                    }
                    Some(actual_dismiss_stale) => {
                        result.add_failure(format!(
                            "dismiss_stale_reviews: expected {}, got {}",
                            expected_dismiss_stale, actual_dismiss_stale
                        ));
                    }
                    None => {
                        result.add_failure(format!(
                            "dismiss_stale_reviews: expected {}, but not configured",
                            expected_dismiss_stale
                        ));
                    }
                }
            }
        }
    }

    Ok(result)
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
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    expected: &[String],
) -> Result<ConfigurationVerification> {
    let actual = client.list_repository_labels(owner, repo).await?;

    let mut result = ConfigurationVerification::success();
    result.labels_verified = true;

    // Check that all expected labels exist
    for expected_label in expected {
        if !actual.contains(expected_label) {
            result.add_failure(format!("Label '{}' not found", expected_label));
        }
    }

    // Note: We don't fail for extra labels that weren't expected
    // GitHub repositories often have default labels

    Ok(result)
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
