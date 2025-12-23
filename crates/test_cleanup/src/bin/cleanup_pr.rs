//! Cleanup test repositories created by a specific PR.
//!
//! This binary cleans up test repositories created during a specific
//! pull request's CI runs. It's designed to be run from GitHub Actions when
//! a PR is closed or merged.
//!
//! Usage:
//!   cleanup-pr <pr_number>
//!
//! Environment variables required:
//! - GITHUB_APP_ID: GitHub App ID for authentication
//! - GITHUB_APP_PRIVATE_KEY: GitHub App private key
//! - TEST_ORG: Organization name (e.g., "glitchgrove")

use std::env;
use test_cleanup::{CleanupConfig, RepositoryCleanup};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    test_cleanup::init_logging();

    // Get PR number from command line args
    let pr_number: u32 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .expect("Usage: cleanup-pr <pr_number>");

    println!("🧹 RepoRoller PR-Based Test Repository Cleanup");
    println!("==============================================");
    println!();

    // Load configuration from environment
    let config = CleanupConfig::from_env()?;

    println!("📋 Configuration:");
    println!("   GitHub App ID: {}", config.github_app_id);
    println!("   Test Organization: {}", config.test_org);
    println!("   PR Number: #{}", pr_number);
    println!();

    // Create GitHub client with App authentication
    let app_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let github_client = github_client::GitHubClient::new(app_client);

    // Create cleanup instance
    let cleanup = RepositoryCleanup::new(github_client, config.test_org.clone());

    println!(
        "🔍 Searching for test repositories from PR #{}...",
        pr_number
    );
    let deleted = cleanup.cleanup_pr_repositories(pr_number).await?;

    println!();
    println!("✅ Cleanup completed!");
    println!("   Deleted {} repositories", deleted.len());

    if !deleted.is_empty() {
        println!();
        println!("📋 Deleted repositories:");
        for repo in &deleted {
            println!("   - {}", repo);
        }
    } else {
        println!("   No repositories found for PR #{}", pr_number);
    }

    Ok(())
}
