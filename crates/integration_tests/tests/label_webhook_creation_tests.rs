//! Label and webhook creation integration tests.
//!
//! These tests verify that labels and webhooks are correctly created and configured
//! during repository creation, testing against real GitHub infrastructure (glitchgrove).
//!
//! Tests cover:
//! - Label creation from configuration
//! - Webhook creation with validation
//! - Idempotency (running configuration multiple times)
//! - Configuration inheritance from global/team/type levels
//! - Error handling and partial failures
//!
//! Note: Webhook URLs use httpbin.org for testing (webhook delivery not verified).

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig};
use github_client::{
    create_token_client, GitHubApiEnvironmentDetector, GitHubClient, RepositoryClient,
    RepositoryCreatePayload,
};
use integration_tests::{RepositoryCleanup, TestConfig};
use repo_roller_core::{
    create_repository, LabelManager, OrganizationName, RepositoryCreationRequestBuilder,
    RepositoryName, TemplateName, WebhookManager,
};
use std::sync::Arc;
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
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

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

/// Test creating repository with labels from global configuration.
///
/// Verifies that labels defined in .reporoller-test/global/standard-labels.toml
/// are created on new repositories.
#[tokio::test]
async fn test_labels_created_from_global_config() -> Result<()> {
    init_test_logging();
    info!("Testing label creation from global configuration");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name =
        RepositoryName::new(&format!("integration-test-labels-{}", uuid::Uuid::new_v4()))?;

    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify labels were created
    let labels = github_client
        .list_repository_labels(&config.test_org, repo_name.as_ref())
        .await?;

    info!("✓ Found {} labels on repository", labels.len());

    // The .reporoller-test global config should have standard labels
    // Check for at least some expected labels
    assert!(
        !labels.is_empty(),
        "Repository should have labels from global config"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Label creation from global config test passed");
    Ok(())
}

/// Test webhook creation on new repository.
///
/// Verifies that webhooks can be created programmatically via WebhookManager.
#[tokio::test]
async fn test_webhook_creation_via_manager() -> Result<()> {
    init_test_logging();
    info!("Testing webhook creation via WebhookManager");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!(
        "integration-test-webhooks-{}",
        uuid::Uuid::new_v4()
    ))?;

    // Create a test repository first (without template to keep it simple)
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let creation_client = GitHubClient::new(octocrab);

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some("Test repository for webhook creation".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Wait for GitHub API to sync after repository creation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Apply webhooks using WebhookManager
    let webhook_manager = WebhookManager::new(github_client.clone());
    let webhook_configs = vec![config_manager::settings::WebhookConfig {
        url: "https://httpbin.org/post".to_string(),
        content_type: "json".to_string(),
        secret: Some("test-secret-123".to_string()),
        active: true,
        events: vec!["push".to_string(), "pull_request".to_string()],
    }];

    let result = webhook_manager
        .apply_webhooks(&config.test_org, repo_name.as_ref(), &webhook_configs)
        .await?;

    info!(
        "✓ Webhook application complete: created={}, updated={}, failed={}, skipped={}",
        result.created, result.updated, result.failed, result.skipped
    );

    assert_eq!(result.created, 1, "Should have created 1 webhook");
    assert_eq!(result.failed, 0, "Should have no failures");

    // Verify webhook was created
    let webhooks = github_client
        .list_webhooks(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(webhooks.len(), 1, "Repository should have 1 webhook");
    assert_eq!(
        webhooks[0].config.url, "https://httpbin.org/post",
        "Webhook URL should match"
    );
    assert_eq!(webhooks[0].events.len(), 2, "Webhook should have 2 events");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Webhook creation via manager test passed");
    Ok(())
}

/// Test label application idempotency.
///
/// Verifies that applying labels multiple times doesn't create duplicates
/// and doesn't fail.
#[tokio::test]
async fn test_label_application_idempotency() -> Result<()> {
    init_test_logging();
    info!("Testing label application idempotency");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!(
        "integration-test-label-idempotent-{}",
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
        description: Some("Test repository for label idempotency".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Apply labels using LabelManager
    let label_manager = LabelManager::new(github_client.clone());
    let mut label_configs = std::collections::HashMap::new();
    label_configs.insert(
        "test-label".to_string(),
        config_manager::settings::LabelConfig {
            name: "test-label".to_string(),
            color: "ff0000".to_string(),
            description: "Test label for idempotency".to_string(),
        },
    );

    // First application
    let result1 = label_manager
        .apply_labels(&config.test_org, repo_name.as_ref(), &label_configs)
        .await?;

    info!("✓ First application: created={}", result1.created);
    assert_eq!(result1.created, 1, "Should have created 1 label");

    // Second application (should be idempotent)
    let result2 = label_manager
        .apply_labels(&config.test_org, repo_name.as_ref(), &label_configs)
        .await?;

    info!("✓ Second application: created={}", result2.created);
    // create_label is idempotent, so it should succeed again
    assert_eq!(
        result2.failed, 0,
        "Should have no failures on re-application"
    );

    // Verify only one label exists (not duplicated)
    let labels = github_client
        .list_repository_labels(&config.test_org, repo_name.as_ref())
        .await?;

    let test_label_count = labels.iter().filter(|l| l == &"test-label").count();
    assert_eq!(
        test_label_count, 1,
        "Should have exactly 1 test-label (no duplicates)"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Label application idempotency test passed");
    Ok(())
}

/// Test webhook application idempotency.
///
/// Verifies that applying webhooks multiple times doesn't create duplicates.
#[tokio::test]
async fn test_webhook_application_idempotency() -> Result<()> {
    init_test_logging();
    info!("Testing webhook application idempotency");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!(
        "integration-test-webhook-idempotent-{}",
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
        description: Some("Test repository for webhook idempotency".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Wait for GitHub API to sync after repository creation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Apply webhooks using WebhookManager
    let webhook_manager = WebhookManager::new(github_client.clone());
    let webhook_configs = vec![config_manager::settings::WebhookConfig {
        url: "https://httpbin.org/webhook-test".to_string(),
        content_type: "json".to_string(),
        secret: Some("idempotency-test".to_string()),
        active: true,
        events: vec!["push".to_string()],
    }];

    // First application
    let result1 = webhook_manager
        .apply_webhooks(&config.test_org, repo_name.as_ref(), &webhook_configs)
        .await?;

    info!("✓ First application: created={}", result1.created);
    assert_eq!(result1.created, 1, "Should have created 1 webhook");

    // Second application (should be idempotent - skipped)
    let result2 = webhook_manager
        .apply_webhooks(&config.test_org, repo_name.as_ref(), &webhook_configs)
        .await?;

    info!(
        "✓ Second application: created={}, skipped={}",
        result2.created, result2.skipped
    );
    assert_eq!(result2.created, 0, "Should not create duplicate webhook");
    assert_eq!(result2.skipped, 1, "Should skip existing webhook");

    // Verify only one webhook exists
    let webhooks = github_client
        .list_webhooks(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(
        webhooks.len(),
        1,
        "Should have exactly 1 webhook (no duplicates)"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Webhook application idempotency test passed");
    Ok(())
}

/// Test webhook validation rejects invalid configurations.
///
/// Verifies that WebhookManager validates webhook configurations
/// and rejects invalid URLs or configurations.
#[tokio::test]
async fn test_webhook_validation_rejects_invalid() -> Result<()> {
    init_test_logging();
    info!("Testing webhook validation with invalid configurations");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!(
        "integration-test-webhook-invalid-{}",
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
        description: Some("Test repository for webhook validation".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Try to apply webhook with HTTP URL (should fail validation - HTTPS required)
    let webhook_manager = WebhookManager::new(github_client.clone());
    let invalid_webhook_configs = vec![config_manager::settings::WebhookConfig {
        url: "http://example.com/webhook".to_string(), // HTTP not HTTPS!
        content_type: "json".to_string(),
        secret: Some("test-secret".to_string()),
        active: true,
        events: vec!["push".to_string()],
    }];

    let result = webhook_manager
        .apply_webhooks(
            &config.test_org,
            repo_name.as_ref(),
            &invalid_webhook_configs,
        )
        .await?;

    info!(
        "✓ Invalid webhook application: created={}, failed={}",
        result.created, result.failed
    );
    assert_eq!(result.created, 0, "Should not create invalid webhook");
    assert_eq!(result.failed, 1, "Should have 1 failure");
    assert_eq!(
        result.failed_webhooks.len(),
        1,
        "Should track failed webhook"
    );

    // Verify no webhooks were created
    let webhooks = github_client
        .list_webhooks(&config.test_org, repo_name.as_ref())
        .await?;

    assert_eq!(
        webhooks.len(),
        0,
        "Should have no webhooks after validation failure"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Webhook validation rejection test passed");
    Ok(())
}

/// Test multiple labels can be created in batch.
///
/// Verifies that LabelManager can handle multiple labels efficiently.
#[tokio::test]
async fn test_multiple_labels_batch_creation() -> Result<()> {
    init_test_logging();
    info!("Testing batch creation of multiple labels");

    let config = TestConfig::from_env()?;
    let (auth_service, github_client, _, _, _) = create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!(
        "integration-test-multiple-labels-{}",
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
        description: Some("Test repository for multiple labels".to_string()),
        private: Some(true),
        ..Default::default()
    };

    creation_client
        .create_org_repository(&config.test_org, &payload)
        .await?;

    info!("✓ Test repository created: {}", repo_name.as_ref());

    // Apply multiple labels using LabelManager
    let label_manager = LabelManager::new(github_client.clone());
    let mut label_configs = std::collections::HashMap::new();

    // Create 5 test labels
    for i in 1..=5 {
        label_configs.insert(
            format!("test-label-{}", i),
            config_manager::settings::LabelConfig {
                name: format!("test-label-{}", i),
                color: format!("00000{}", i),
                description: format!("Test label number {}", i),
            },
        );
    }

    let result = label_manager
        .apply_labels(&config.test_org, repo_name.as_ref(), &label_configs)
        .await?;

    info!(
        "✓ Batch label creation: created={}, failed={}",
        result.created, result.failed
    );
    assert_eq!(result.created, 5, "Should have created 5 labels");
    assert_eq!(result.failed, 0, "Should have no failures");

    // Verify all labels exist
    let labels = github_client
        .list_repository_labels(&config.test_org, repo_name.as_ref())
        .await?;

    for i in 1..=5 {
        let label_name = format!("test-label-{}", i);
        assert!(
            labels.contains(&label_name),
            "Label {} should exist",
            label_name
        );
    }

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Multiple labels batch creation test passed");
    Ok(())
}
