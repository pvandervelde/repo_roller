//! Test utilities for integration and E2E tests.
//!
//! This crate provides shared utilities for managing test repositories,
//! including naming conventions and cleanup functionality.

use anyhow::Result;
use chrono::Utc;
use std::env;
use tracing::{info, warn};
use uuid::Uuid;

/// Extract workflow context from GitHub Actions environment for repository naming.
///
/// Returns:
/// - `pr{number}` for pull request workflows (e.g., "pr123")
/// - `main` for pushes to main/master branch
/// - `local` for local development
///
/// Uses GITHUB_REF environment variable which contains:
/// - `refs/pull/{number}/merge` for pull requests
/// - `refs/heads/{branch}` for branch pushes
pub fn get_workflow_context() -> String {
    // Check if running in GitHub Actions PR context
    if let Ok(github_ref) = env::var("GITHUB_REF") {
        if github_ref.starts_with("refs/pull/") {
            // Extract PR number from refs/pull/{number}/merge
            if let Some(pr_num) = github_ref.split('/').nth(2) {
                return format!("pr{}", pr_num);
            }
        } else if github_ref.starts_with("refs/heads/") {
            // Extract branch name from refs/heads/{branch}
            // Skip "refs/heads/" prefix (11 characters)
            let branch = &github_ref[11..];
            // Use 'main' for main/master branches
            if branch == "main" || branch == "master" {
                return "main".to_string();
            }
            // Use sanitized branch name for other branches (replace / with -)
            return branch.replace('/', "-");
        }
    }

    // Fallback for local development
    "local".to_string()
}

/// Generate a unique test repository name following the naming convention.
///
/// Format: `{prefix}-repo-roller-{context}-{timestamp}-{test-name}-{random}`
///
/// Where context is:
/// - `pr{number}` for PR workflows (e.g., pr123)
/// - `main` for main branch workflows
/// - `local` for local development
///
/// # Arguments
///
/// * `prefix` - Repository prefix ("test" or "e2e")
/// * `test_name` - Test scenario name
///
/// # Examples
///
/// ```
/// use test_utils::generate_test_repo_name;
///
/// // For integration tests
/// let name = generate_test_repo_name("test", "basic");
/// // Result: test-repo-roller-pr123-20240108-120000-basic-a1b2c3 (in PR)
/// // Result: test-repo-roller-local-20240108-120000-basic-a1b2c3 (local)
///
/// // For E2E tests
/// let name = generate_test_repo_name("e2e", "api");
/// // Result: e2e-repo-roller-pr123-20240108-120000-api-a1b2c3 (in PR)
/// ```
pub fn generate_test_repo_name(prefix: &str, test_name: &str) -> String {
    let context = get_workflow_context();
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let random_suffix = Uuid::new_v4().simple().to_string()[..6].to_lowercase();
    format!(
        "{}-repo-roller-{}-{}-{}-{}",
        prefix, context, timestamp, test_name, random_suffix
    )
}

/// Delete a test repository (best effort cleanup).
///
/// This function attempts to delete a test repository but does not fail
/// if deletion is unsuccessful. Errors are logged but do not propagate.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `repo_name` - Repository name to delete
/// * `app_id` - GitHub App ID for authentication
/// * `private_key` - GitHub App private key
///
/// # Examples
///
/// ```no_run
/// use test_utils::cleanup_test_repository;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// cleanup_test_repository(
///     "my-org",
///     "test-repo-roller-pr123-20240108-120000-basic-a1b2c3",
///     12345,
///     "-----BEGIN RSA PRIVATE KEY-----\n...",
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn cleanup_test_repository(
    org: &str,
    repo_name: &str,
    app_id: u64,
    private_key: &str,
) -> Result<()> {
    info!(
        org = org,
        repo = repo_name,
        "Attempting best-effort cleanup of test repository"
    );

    // Create GitHub client with App authentication
    let app_client = match github_client::create_app_client(app_id, private_key).await {
        Ok(client) => client,
        Err(e) => {
            warn!(
                org = org,
                repo = repo_name,
                error = %e,
                "Failed to create GitHub client for cleanup"
            );
            return Ok(()); // Don't fail test on cleanup failure
        }
    };

    let github_client = github_client::GitHubClient::new(app_client);

    // Get installation token for the organization
    let installation_token = match github_client.get_installation_token_for_org(org).await {
        Ok(token) => token,
        Err(e) => {
            warn!(
                org = org,
                repo = repo_name,
                error = %e,
                "Failed to get installation token for cleanup"
            );
            return Ok(()); // Don't fail test on cleanup failure
        }
    };

    // Create client with installation token
    let installation_client = match github_client::create_token_client(&installation_token) {
        Ok(client) => client,
        Err(e) => {
            warn!(
                org = org,
                repo = repo_name,
                error = %e,
                "Failed to create installation token client for cleanup"
            );
            return Ok(()); // Don't fail test on cleanup failure
        }
    };

    // Delete repository using octocrab
    match installation_client.repos(org, repo_name).delete().await {
        Ok(_) => {
            info!(
                org = org,
                repo = repo_name,
                "✓ Successfully cleaned up test repository"
            );
        }
        Err(e) => {
            warn!(
                org = org,
                repo = repo_name,
                error = %e,
                "Failed to delete test repository - it may not exist or deletion failed"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

