//! Test repository cleanup utilities.
//!
//! This crate provides utilities for cleaning up test repositories created during
//! RepoRoller integration and E2E testing. It can be used both programmatically
//! (from test code) and via CLI binaries.

use anyhow::{Context, Result};
use chrono::Utc;
use github_client::GitHubClient;
use std::env;
use tracing::{debug, error, info, warn};

/// Configuration for cleanup operations loaded from environment variables.
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// GitHub App ID for authentication
    pub github_app_id: u64,
    /// GitHub App private key for authentication
    pub github_app_private_key: String,
    /// Organization where test repositories exist
    pub test_org: String,
}

impl CleanupConfig {
    /// Load cleanup configuration from environment variables.
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

/// Repository cleanup operations for test repositories.
///
/// This struct provides methods to identify and delete test repositories
/// based on naming conventions and age criteria.
pub struct RepositoryCleanup {
    client: GitHubClient,
    test_org: String,
}

impl RepositoryCleanup {
    /// Create a new cleanup instance.
    ///
    /// # Arguments
    ///
    /// * `client` - Authenticated GitHub client
    /// * `test_org` - Organization name where test repositories exist
    pub fn new(client: GitHubClient, test_org: String) -> Self {
        Self { client, test_org }
    }

    /// Check if a repository name matches test repository naming patterns.
    ///
    /// Returns true if the name starts with "test-repo-roller-" or "e2e-repo-roller-".
    pub fn is_test_repository(repo_name: &str) -> bool {
        repo_name.starts_with("test-repo-roller-") || repo_name.starts_with("e2e-repo-roller-")
    }

    /// Find and delete orphaned test repositories.
    ///
    /// This method searches for repositories matching test naming patterns
    /// (test-repo-roller-* and e2e-repo-roller-*) that are older than
    /// the specified age and deletes them.
    pub async fn cleanup_orphaned_repositories(&self, max_age_hours: u64) -> Result<Vec<String>> {
        self.cleanup_repositories_internal(max_age_hours, None)
            .await
    }

    /// Find and delete test repositories created by a specific PR.
    ///
    /// This method searches for repositories matching PR-specific naming patterns
    /// (test-repo-roller-pr{number}-* and e2e-repo-roller-pr{number}-*)
    /// and deletes them regardless of age.
    ///
    /// # Arguments
    ///
    /// * `pr_number` - The PR number to clean up repositories for
    pub async fn cleanup_pr_repositories(&self, pr_number: u32) -> Result<Vec<String>> {
        self.cleanup_repositories_internal(0, Some(pr_number)).await
    }

    /// Check if a repository name matches misnamed test repository patterns.
    ///
    /// Returns true if the name looks like a test repository but doesn't follow
    /// the correct naming convention. Patterns include:
    /// - Starts with test patterns: "template-", "test-", "integration-", "e2e-test-", "e2e-", "temp-", "demo-"
    /// - Does NOT match correct patterns: "test-repo-roller-", "e2e-repo-roller-"
    /// - Is not a template repository
    pub fn is_misnamed_repository(repo_name: &str, is_template: bool) -> bool {
        // Skip if it's a template repository
        if is_template {
            return false;
        }

        // Check if it matches correct naming patterns
        if Self::is_test_repository(repo_name) {
            return false;
        }

        // Check if it matches misnamed patterns
        repo_name.starts_with("template-")
            || repo_name.starts_with("test-")
            || repo_name.starts_with("integration-")
            || repo_name.starts_with("e2e-test-")
            || repo_name.starts_with("e2e-")
            || repo_name.starts_with("temp-")
            || repo_name.starts_with("demo-")
    }

    /// Find and delete misnamed test repositories.
    ///
    /// This method searches for repositories that look like test repositories
    /// but don't follow the correct naming convention (e.g., "e2e-test-global-*"
    /// instead of "e2e-repo-roller-*"). It deletes repositories older than
    /// the specified age.
    ///
    /// # Arguments
    ///
    /// * `max_age_hours` - Minimum age in hours for repositories to be deleted
    pub async fn cleanup_misnamed_repositories(&self, max_age_hours: u64) -> Result<Vec<String>> {
        info!(
            org = self.test_org,
            max_age_hours = max_age_hours,
            "Searching for misnamed test repositories"
        );

        let mut deleted_repos = Vec::new();
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        // Get installation token for the organization
        let installation_token = self
            .client
            .get_installation_token_for_org(&self.test_org)
            .await
            .context("Failed to get installation token for cleanup")?;

        // Create client with installation token
        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client for cleanup")?;

        // List repositories in the organization with pagination
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
                        let is_template = repo.is_template.unwrap_or(false);

                        // Check if this is a misnamed test repository
                        if !Self::is_misnamed_repository(&repo_name, is_template) {
                            continue;
                        }

                        // Age-based cleanup
                        let created_at = repo
                            .created_at
                            .unwrap_or_else(|| cutoff_time + chrono::Duration::hours(1));

                        if created_at < cutoff_time {
                            info!(
                                repo_name = repo_name,
                                created_at = %created_at,
                                cutoff_time = %cutoff_time,
                                "Found misnamed test repository, attempting deletion"
                            );

                            if self.delete_repository(&repo_name).await.is_ok() {
                                deleted_repos.push(repo_name);
                            }
                        } else {
                            debug!(
                                repo_name = repo_name,
                                created_at = %created_at,
                                age_hours = (Utc::now() - created_at).num_hours(),
                                "Repository is too new, skipping"
                            );
                        }
                    }

                    page += 1;
                }
                Err(err) => {
                    error!(
                        org = self.test_org,
                        page = page,
                        error = %err,
                        "Failed to list repositories"
                    );
                    return Err(err).context("Failed to list organization repositories");
                }
            }
        }

        info!(
            org = self.test_org,
            deleted_count = deleted_repos.len(),
            "Cleanup completed"
        );

        Ok(deleted_repos)
    }

    /// Internal method to clean up repositories with optional PR filtering.
    ///
    /// # Arguments
    ///
    /// * `max_age_hours` - Maximum age for repositories (ignored if pr_number is Some)
    /// * `pr_number` - Optional PR number to filter by
    async fn cleanup_repositories_internal(
        &self,
        max_age_hours: u64,
        pr_number: Option<u32>,
    ) -> Result<Vec<String>> {
        if let Some(pr) = pr_number {
            info!(
                org = self.test_org,
                pr_number = pr,
                "Searching for test repositories from PR {}",
                pr
            );
        } else {
            info!(
                org = self.test_org,
                max_age_hours = max_age_hours,
                "Searching for orphaned test repositories"
            );
        }

        let mut deleted_repos = Vec::new();
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        // Get installation token for the organization
        let installation_token = self
            .client
            .get_installation_token_for_org(&self.test_org)
            .await
            .context("Failed to get installation token for cleanup")?;

        // Create client with installation token
        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client for cleanup")?;

        // List repositories in the organization with pagination
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
                        if !Self::is_test_repository(&repo_name) {
                            continue;
                        }

                        // If filtering by PR, check if this repo matches the PR pattern
                        if let Some(pr) = pr_number {
                            let pr_pattern = format!("-pr{}-", pr);
                            if !repo_name.contains(&pr_pattern) {
                                debug!(
                                    repo_name = repo_name,
                                    pr_number = pr,
                                    "Skipping repository - not from PR {}",
                                    pr
                                );
                                continue;
                            }

                            info!(
                                repo_name = repo_name,
                                pr_number = pr,
                                "Found PR {} repository, attempting deletion",
                                pr
                            );

                            if self.delete_repository(&repo_name).await.is_ok() {
                                deleted_repos.push(repo_name);
                            }
                        } else {
                            // Age-based cleanup
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
                                    age_hours = (Utc::now() - created_at).num_hours(),
                                    "Repository is too new, skipping"
                                );
                            }
                        }
                    }

                    page += 1;
                }
                Err(err) => {
                    error!(
                        org = self.test_org,
                        page = page,
                        error = %err,
                        "Failed to list repositories"
                    );
                    return Err(err).context("Failed to list organization repositories");
                }
            }
        }

        info!(
            org = self.test_org,
            deleted_count = deleted_repos.len(),
            "Cleanup completed"
        );

        Ok(deleted_repos)
    }

    /// Delete a repository by name.
    ///
    /// This is a best-effort operation that logs errors but doesn't fail
    /// the entire cleanup process if one repository can't be deleted.
    pub async fn delete_repository(&self, repo_name: &str) -> Result<()> {
        info!(
            org = self.test_org,
            repo_name = repo_name,
            "Deleting repository"
        );

        // Get installation token for the organization
        let installation_token = self
            .client
            .get_installation_token_for_org(&self.test_org)
            .await
            .context("Failed to get installation token for deletion")?;

        // Create client with installation token
        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client for deletion")?;

        // Delete the repository
        match installation_client
            .repos(&self.test_org, repo_name)
            .delete()
            .await
        {
            Ok(_) => {
                info!(
                    org = self.test_org,
                    repo_name = repo_name,
                    "Successfully deleted repository"
                );
                Ok(())
            }
            Err(err) => {
                warn!(
                    org = self.test_org,
                    repo_name = repo_name,
                    error = %err,
                    "Failed to delete repository (may not exist or lack permissions)"
                );
                Err(err).context(format!("Failed to delete repository {}", repo_name))
            }
        }
    }
}

/// Initialize logging for cleanup operations.
///
/// Sets up tracing with appropriate formatting for CLI use.
pub fn init_logging() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
