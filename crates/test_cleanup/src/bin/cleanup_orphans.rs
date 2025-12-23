//! Cleanup orphaned test repositories.
//!
//! This binary cleans up test repositories older than a specified age.
//! It's designed to be run from GitHub Actions or manually for maintenance.
//!
//! Usage:
//!   cleanup-orphans <max_age_hours>
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

    // Get max age from command line args, default to 1 hour
    let max_age_hours: u64 = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    println!("🧹 RepoRoller Test Repository Cleanup");
    println!("=====================================");
    println!();

    // Load configuration from environment
    let config = CleanupConfig::from_env()?;

    println!("📋 Configuration:");
    println!("   GitHub App ID: {}", config.github_app_id);
    println!("   Test Organization: {}", config.test_org);
    println!("   Max age: {} hours", max_age_hours);
    println!();

    // Create GitHub client with App authentication
    let app_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let github_client = github_client::GitHubClient::new(app_client);

    // Create cleanup instance
    let cleanup = RepositoryCleanup::new(github_client, config.test_org.clone());

    println!("🔍 Searching for orphaned test repositories...");
    let deleted = cleanup.cleanup_orphaned_repositories(max_age_hours).await?;

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
        println!(
            "   No repositories found older than {} hours",
            max_age_hours
        );
    }

    Ok(())
}
