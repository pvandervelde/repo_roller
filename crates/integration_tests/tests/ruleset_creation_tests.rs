//! Repository ruleset creation integration tests.
//!
//! These tests verify that repository rulesets are correctly created and configured
//! during repository creation, testing against real GitHub infrastructure (glitchgrove).
//!
//! Tests cover:
//! - Ruleset creation from configuration
//! - Idempotency (running configuration multiple times)
//! - Error handling and partial failures
//! - Listing and verification of rulesets
//!
//! Note: Tests use real GitHub API and verify actual ruleset state.

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{
    settings::{RefNameConditionConfig, RuleConfig, RulesetConditionsConfig, RulesetConfig},
    ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig,
};
use github_client::{
    create_token_client, GitHubApiEnvironmentDetector, GitHubClient, RepositoryClient,
    RepositoryCreatePayload,
};
use integration_tests::{RepositoryCleanup, TestConfig};
use repo_roller_core::{
    OrganizationName, RepositoryCreationRequestBuilder, RepositoryName, RulesetManager,
    TemplateName,
};
use std::{collections::HashMap, sync::Arc};
use tracing::info;

/// Initialize logging for tests.
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_test_writer()
        .try_init();
}

/// Create test dependencies (auth service, GitHub client, metadata provider).
async fn create_test_dependencies(
    config: &TestConfig,
) -> Result<(
    GitHubAuthService,
    GitHubClient,
    Arc<GitHubMetadataProvider>,
    Arc<ConfigBasedPolicyProvider>,
    Arc<GitHubApiEnvironmentDetector>,
)> {
    let auth_service =
        GitHubAuthService::new(config.github_app_id, config.github_app_private_key.clone());

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab = Arc::new(create_token_client(&installation_token)?);
    let github_client = GitHubClient::new((*octocrab).clone());

    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client.clone(),
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    Ok((
        auth_service,
        github_client,
        metadata_provider,
        visibility_policy_provider,
        environment_detector,
    ))
}

/// Helper to create a minimal test ruleset configuration.
fn create_test_ruleset_config(name: &str, target: &str) -> RulesetConfig {
    RulesetConfig {
        name: name.to_string(),
        target: target.to_string(),
        enforcement: "active".to_string(),
        bypass_actors: vec![],
        conditions: Some(RulesetConditionsConfig {
            ref_name: RefNameConditionConfig {
                include: vec!["refs/heads/main".to_string()],
                exclude: vec![],
            },
        }),
        rules: vec![RuleConfig::Deletion],
    }
}

/// Test creating rulesets directly via RulesetManager.
///
/// Verifies that rulesets can be created programmatically on repositories.
#[tokio::test]
async fn test_ruleset_creation_via_manager() -> Result<()> {
    init_test_logging();
    info!("Testing ruleset creation via RulesetManager");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(format!(
        "integration-test-rulesets-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository first
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for ruleset creation".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Wait for GitHub API to sync
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Apply rulesets using RulesetManager
    let ruleset_manager = RulesetManager::new(github_client.clone());
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "main-protection".to_string(),
        create_test_ruleset_config("main-protection", "branch"),
    );

    let result = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &rulesets)
        .await?;

    info!(
        "✓ Ruleset application complete: created={}, updated={}, failed={}",
        result.created, result.updated, result.failed
    );

    assert_eq!(result.created, 1, "Should have created 1 ruleset");
    assert_eq!(result.failed, 0, "Should have no failures");
    assert!(result.is_success(), "Result should indicate success");

    // Verify ruleset was created
    let list_result = ruleset_manager
        .list_rulesets(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(list_result.len(), 1, "Repository should have 1 ruleset");
    assert_eq!(
        list_result[0].name, "main-protection",
        "Ruleset name should match"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Ruleset creation via manager test passed");
    Ok(())
}

/// Test ruleset idempotency - applying same rulesets multiple times.
///
/// Verifies that calling ruleset application multiple times does not create duplicates.
#[tokio::test]
async fn test_rulesets_are_idempotent() -> Result<()> {
    init_test_logging();
    info!("Testing ruleset idempotency");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(format!(
        "integration-test-rulesets-idempotent-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for ruleset idempotency".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created");

    // Wait for GitHub API to sync
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Apply rulesets first time
    let ruleset_manager = RulesetManager::new(github_client.clone());
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "main-protection".to_string(),
        create_test_ruleset_config("main-protection", "branch"),
    );
    rulesets.insert(
        "tag-protection".to_string(),
        create_test_ruleset_config("tag-protection", "tag"),
    );

    let result1 = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &rulesets)
        .await?;

    info!(
        "✓ First application: created={}, updated={}",
        result1.created, result1.updated
    );
    assert_eq!(result1.created, 2, "First run should create 2 rulesets");
    assert_eq!(result1.updated, 0, "First run should not update any");

    // Apply same rulesets again
    let result2 = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &rulesets)
        .await?;

    info!(
        "✓ Second application: created={}, updated={}",
        result2.created, result2.updated
    );
    assert_eq!(result2.created, 0, "Second run should not create any new");
    assert_eq!(result2.updated, 2, "Second run should update 2 existing");

    // Verify still only 2 rulesets exist
    let list_result = ruleset_manager
        .list_rulesets(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(
        list_result.len(),
        2,
        "Repository should still have exactly 2 rulesets"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Ruleset idempotency test passed");
    Ok(())
}

/// Test listing rulesets from an existing repository.
///
/// Verifies that we can retrieve rulesets that have been applied.
#[tokio::test]
async fn test_list_existing_rulesets() -> Result<()> {
    init_test_logging();
    info!("Testing listing existing rulesets");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(format!(
        "integration-test-rulesets-list-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for listing rulesets".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created");

    // Wait for GitHub API to sync
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create known rulesets
    let ruleset_manager = RulesetManager::new(github_client.clone());
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "branch-protection".to_string(),
        create_test_ruleset_config("branch-protection", "branch"),
    );
    rulesets.insert(
        "tag-protection".to_string(),
        create_test_ruleset_config("tag-protection", "tag"),
    );
    rulesets.insert(
        "deletion-protection".to_string(),
        create_test_ruleset_config("deletion-protection", "branch"),
    );

    ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &rulesets)
        .await?;

    info!("✓ Applied 3 rulesets");

    // List rulesets
    let list_result = ruleset_manager
        .list_rulesets(&config.test_org, repo_name.as_ref())
        .await?;

    info!("✓ Listed {} rulesets", list_result.len());
    assert_eq!(list_result.len(), 3, "Should list all 3 rulesets");

    // Verify all expected rulesets are present
    let ruleset_names: Vec<String> = list_result.iter().map(|r| r.name.clone()).collect();
    assert!(
        ruleset_names.contains(&"branch-protection".to_string()),
        "Should contain branch-protection"
    );
    assert!(
        ruleset_names.contains(&"tag-protection".to_string()),
        "Should contain tag-protection"
    );
    assert!(
        ruleset_names.contains(&"deletion-protection".to_string()),
        "Should contain deletion-protection"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ List existing rulesets test passed");
    Ok(())
}

/// Test updating existing rulesets.
///
/// Verifies that existing rulesets are updated rather than duplicated.
#[tokio::test]
async fn test_update_existing_rulesets() -> Result<()> {
    init_test_logging();
    info!("Testing updating existing rulesets");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(format!(
        "integration-test-rulesets-update-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for updating rulesets".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created");

    // Wait for GitHub API to sync
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create initial rulesets
    let ruleset_manager = RulesetManager::new(github_client.clone());
    let mut rulesets = HashMap::new();
    rulesets.insert(
        "main-protection".to_string(),
        create_test_ruleset_config("main-protection", "branch"),
    );

    let result1 = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &rulesets)
        .await?;

    info!("✓ Initial application: created={}", result1.created);
    assert_eq!(result1.created, 1);

    // Modify the ruleset configuration (change enforcement)
    let mut updated_rulesets = HashMap::new();
    let mut modified_config = create_test_ruleset_config("main-protection", "branch");
    modified_config.enforcement = "evaluate".to_string(); // Changed from "active"
    updated_rulesets.insert("main-protection".to_string(), modified_config);

    // Apply the updated configuration
    let result2 = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &updated_rulesets)
        .await?;

    info!("✓ Update application: updated={}", result2.updated);
    assert_eq!(result2.updated, 1, "Should update the existing ruleset");
    assert_eq!(result2.created, 0, "Should not create a new ruleset");

    // Verify still only 1 ruleset exists
    let list_result = ruleset_manager
        .list_rulesets(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(
        list_result.len(),
        1,
        "Repository should still have exactly 1 ruleset"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Update existing rulesets test passed");
    Ok(())
}

/// Test empty ruleset configuration.
///
/// Verifies that applying an empty ruleset configuration succeeds without errors.
#[tokio::test]
async fn test_empty_ruleset_configuration() -> Result<()> {
    init_test_logging();
    info!("Testing empty ruleset configuration");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(format!(
        "integration-test-rulesets-empty-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for empty ruleset config".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created");

    // Wait for GitHub API to sync
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Apply empty rulesets
    let ruleset_manager = RulesetManager::new(github_client.clone());
    let empty_rulesets = HashMap::new();

    let result = ruleset_manager
        .apply_rulesets(&config.test_org, repo_name.as_ref(), &empty_rulesets)
        .await?;

    info!("✓ Empty ruleset application complete");
    assert_eq!(result.created, 0, "Should create no rulesets");
    assert_eq!(result.updated, 0, "Should update no rulesets");
    assert_eq!(result.failed, 0, "Should have no failures");
    assert!(result.is_success(), "Should indicate success");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Empty ruleset configuration test passed");
    Ok(())
}
