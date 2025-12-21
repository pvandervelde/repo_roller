//! Utility functions for integration testing.
//!
//! This module provides helper functions for setting up, running, and cleaning up
//! integration tests. It handles repository cleanup operations and test
//! environment management.

use anyhow::{Context, Result};
use chrono::Utc;
use github_client::GitHubClient;
use std::env;
use tracing::{debug, error, info, warn};

// Re-export test_utils functions for backward compatibility
pub use test_utils::{cleanup_test_repository, generate_test_repo_name, get_workflow_context};

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

/// Repository cleanup utility for managing test artifacts.
pub struct RepositoryCleanup {
    client: GitHubClient,
    test_org: String,
}

impl RepositoryCleanup {
    /// Create a new cleanup utility with the provided GitHub client.
    pub fn new(client: GitHubClient, test_org: String) -> Self {
        Self { client, test_org }
    }

    /// Delete a specific test repository.
    ///
    /// This method attempts to delete the repository and logs the result.
    /// It does not fail if the repository doesn't exist or deletion fails,
    /// as cleanup should be best-effort.
    pub async fn delete_repository(&self, repo_name: &str) -> Result<()> {
        info!(
            repo_name = repo_name,
            org = self.test_org,
            "Attempting to delete test repository"
        );

        // Get installation token for the organization
        let installation_token = self
            .client
            .get_installation_token_for_org(&self.test_org)
            .await
            .context("Failed to get installation token for cleanup")?;

        // Create client with installation token
        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client for cleanup")?;

        // Use octocrab directly for repository deletion as it's not in our RepositoryClient trait
        let delete_result = installation_client
            .repos(&self.test_org, repo_name)
            .delete()
            .await;

        match delete_result {
            Ok(_) => {
                info!(
                    repo_name = repo_name,
                    org = self.test_org,
                    "Successfully deleted test repository"
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    repo_name = repo_name,
                    org = self.test_org,
                    error = %e,
                    "Failed to delete test repository - it may not exist or deletion failed"
                );
                // Don't return error for cleanup failures - log and continue
                Ok(())
            }
        }
    }

    /// Find and delete orphaned test repositories.
    ///
    /// This method searches for repositories matching the test naming pattern
    /// that are older than the specified age and deletes them.
    pub async fn cleanup_orphaned_repositories(&self, max_age_hours: u64) -> Result<Vec<String>> {
        info!(
            org = self.test_org,
            max_age_hours = max_age_hours,
            "Searching for orphaned test repositories"
        );

        let mut deleted_repos = Vec::new();
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        // Get installation token for the organization
        let installation_token = self
            .client
            .get_installation_token_for_org(&self.test_org)
            .await
            .context("Failed to get installation token for orphan cleanup")?;

        // Create client with installation token
        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client for orphan cleanup")?;

        // List repositories in the organization with pagination
        // GitHub API returns max 100 items per page by default (30 if not specified)
        let mut page = 1u32;
        let per_page = 100u8;

        info!(
            org = self.test_org,
            "Starting paginated repository listing (max {} repos per page)", per_page
        );

        loop {
            debug!(
                org = self.test_org,
                page = page,
                "Fetching page {} of repositories",
                page
            );

            let repos_result = installation_client
                .orgs(&self.test_org)
                .list_repos()
                .per_page(per_page)
                .page(page)
                .send()
                .await;

            match repos_result {
                Ok(repos) => {
                    let repo_count = repos.items.len();
                    debug!(
                        org = self.test_org,
                        page = page,
                        count = repo_count,
                        "Retrieved {} repositories on page {}",
                        repo_count,
                        page
                    );

                    if repo_count == 0 {
                        info!(
                            org = self.test_org,
                            total_pages = page - 1,
                            "No more repositories to process"
                        );
                        break;
                    }

                    for repo in repos.items {
                        let repo_name = repo.name;

                        // Check if this is a test repository
                        if repo_name.starts_with("test-repo-roller-") {
                            // Parse creation time from repository
                            let created_at = repo
                                .created_at
                                .unwrap_or_else(|| cutoff_time + chrono::Duration::hours(1));

                            if created_at < cutoff_time {
                                info!(
                                    repo_name = repo_name,
                                    created_at = %created_at,
                                    cutoff_time = %cutoff_time,
                                    "Found orphaned test repository, attempting deletion"
                                );

                                if self.delete_repository(&repo_name).await.is_ok() {
                                    deleted_repos.push(repo_name);
                                }
                            } else {
                                debug!(
                                    repo_name = repo_name,
                                    created_at = %created_at,
                                    "Test repository is recent, skipping cleanup"
                                );
                            }
                        }
                    }

                    // If we got fewer repos than per_page, we've reached the end
                    if repo_count < per_page as usize {
                        info!(
                            org = self.test_org,
                            total_pages = page,
                            "Reached last page of repositories"
                        );
                        break;
                    }

                    page += 1;
                }
                Err(e) => {
                    error!(
                        org = self.test_org,
                        page = page,
                        error = %e,
                        "Failed to list repositories for orphan cleanup"
                    );
                    return Err(anyhow::anyhow!(
                        "Failed to list repositories on page {}: {}",
                        page,
                        e
                    ));
                }
            }
        }

        info!(
            org = self.test_org,
            deleted_count = deleted_repos.len(),
            total_pages_processed = page,
            "Completed orphaned repository cleanup"
        );

        Ok(deleted_repos)
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
