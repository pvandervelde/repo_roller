//! Utility functions for integration testing.
//!
//! This module provides helper functions for setting up, running, and cleaning up
//! integration tests. It re-exports utilities from test_utils and test_cleanup crates.

use anyhow::{Context, Result};
use chrono::Utc;
use std::env;
use tracing::info;

// Re-export test_utils functions for backward compatibility
pub use test_utils::{cleanup_test_repository, generate_test_repo_name, get_workflow_context};

// Re-export test_cleanup functionality
pub use test_cleanup::RepositoryCleanup;

/// Check if a repository name matches the test repository naming patterns.
///
/// Returns true if the name starts with "test-repo-roller-" or "e2e-repo-roller-".
///
/// # Examples
///
/// ```
/// // Use the full module path in doctests
/// assert!(integration_tests::is_test_repository("test-repo-roller-pr123-auth"));
/// assert!(integration_tests::is_test_repository("e2e-repo-roller-main-api"));
/// assert!(!integration_tests::is_test_repository("regular-repo"));
/// ```
pub fn is_test_repository(repo_name: &str) -> bool {
    test_cleanup::RepositoryCleanup::is_test_repository(repo_name)
}

/// Helper function to generate test repository names for integration tests.
///
/// This is a convenience wrapper around test_utils::generate_test_repo_name
/// with the "test" prefix pre-applied.
pub fn generate_integration_test_repo_name(test_name: &str) -> String {
    test_utils::generate_test_repo_name("test", test_name)
}

/// Configuration for integration tests loaded from environment variables.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// GitHub App ID for authentication
    pub github_app_id: u64,
    /// GitHub App private key for authentication
    pub github_app_private_key: String,
    /// Organization where test repositories will be created
    pub test_org: String,
}

impl TestConfig {
    /// Load test configuration from environment variables.
    ///
    /// Required environment variables:
    /// - `GITHUB_APP_ID`: GitHub App ID (numeric)
    /// - `GITHUB_APP_PRIVATE_KEY`: GitHub App private key (PEM format)
    /// - `TEST_ORG`: Organization name for test repositories
    pub fn from_env() -> Result<Self> {
        let github_app_id = env::var("GITHUB_APP_ID")
            .context("GITHUB_APP_ID environment variable not set")?
            .parse::<u64>()
            .context("GITHUB_APP_ID must be a valid number")?;

        let github_app_private_key = env::var("GITHUB_APP_PRIVATE_KEY")
            .context("GITHUB_APP_PRIVATE_KEY environment variable not set")?;

        let test_org = env::var("TEST_ORG").context("TEST_ORG environment variable not set")?;

        Ok(Self {
            github_app_id,
            github_app_private_key,
            test_org,
        })
    }
}

/// Test repository information for tracking and cleanup.
#[derive(Debug, Clone)]
pub struct TestRepository {
    pub name: String,
    pub owner: String,
    pub full_name: String,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<Utc>,
}

impl TestRepository {
    pub fn new(name: String, owner: String) -> Self {
        let full_name = format!("{}/{}", owner, name);
        Self {
            name,
            owner,
            full_name,
            created_at: Utc::now(),
        }
    }
}

/// Initialize logging for integration tests.
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

/// Validate that required environment variables are set for testing.
pub fn validate_test_environment() -> Result<()> {
    let required_vars = ["GITHUB_APP_ID", "GITHUB_APP_PRIVATE_KEY", "TEST_ORG"];

    for var in &required_vars {
        if env::var(var).is_err() {
            return Err(anyhow::anyhow!(
                "Required environment variable {} is not set. See .github/workflows/INTEGRATION_TESTS_SECRETS.md for setup instructions.",
                var
            ));
        }
    }

    info!("All required environment variables are set");
    Ok(())
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
