//! Repository visibility integration tests.
//!
//! These tests verify that repository visibility is correctly resolved and applied
//! during repository creation, testing against real GitHub infrastructure (glitchgrove).
//!
//! Tests cover:
//! - Explicit visibility preferences (Private, Public)
//! - Visibility resolution with different policies
//! - Verification of actual GitHub repository visibility after creation
//!
//! Note: Internal visibility requires GitHub Enterprise and is not tested here.

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{
    ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig, RepositoryVisibility,
};
use github_client::{create_token_client, GitHubApiEnvironmentDetector, GitHubClient};
use integration_tests::{RepositoryCleanup, TestConfig};
use repo_roller_core::{
    create_repository, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
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

/// Create test dependencies (auth service, metadata provider, visibility providers).
async fn create_test_dependencies(
    config: &TestConfig,
) -> Result<(
    GitHubAuthService,
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
        github_client,
        MetadataProviderConfig::explicit(".reporoller"),
    ));

    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    Ok((
        auth_service,
        metadata_provider,
        visibility_policy_provider,
        environment_detector,
    ))
}

/// Test creating a private repository explicitly.
///
/// Verifies that when visibility is set to Private,
/// the created repository is actually private on GitHub.
#[tokio::test]
async fn test_create_private_repository_explicit() -> Result<()> {
    init_test_logging();
    info!("Testing explicit private repository creation");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-private-{}", uuid::Uuid::new_v4()))?;

    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Private)
    .build();

    // Execute repository creation
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify repository was created with correct visibility
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let github_client = GitHubClient::new(octocrab);

    let repository = github_client
        .get_repository(&config.test_org, repo_name.as_ref())
        .await?;

    assert!(
        repository.is_private(),
        "Repository should be private but is public"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Private repository creation test passed");
    Ok(())
}

/// Test creating a public repository explicitly.
///
/// Verifies that when visibility is set to Public,
/// the created repository is actually public on GitHub.
#[tokio::test]
async fn test_create_public_repository_explicit() -> Result<()> {
    init_test_logging();
    info!("Testing explicit public repository creation");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-public-{}", uuid::Uuid::new_v4()))?;

    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Public)
    .build();

    // Execute repository creation
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify repository was created with correct visibility
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let github_client = GitHubClient::new(octocrab);

    let repository = github_client
        .get_repository(&config.test_org, repo_name.as_ref())
        .await?;

    assert!(
        !repository.is_private(),
        "Repository should be public (private=false)"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Public repository creation test passed");
    Ok(())
}

/// Test creating repository without explicit visibility (uses system default).
///
/// Verifies that when no visibility is specified, the system default (Private)
/// is applied.
#[tokio::test]
async fn test_create_repository_default_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing repository creation with default visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-default-{}", uuid::Uuid::new_v4()))?;

    // Don't specify visibility - should use system default (Private)
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .build();

    // Execute repository creation
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify repository defaults to private
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let github_client = GitHubClient::new(octocrab);

    let repository = github_client
        .get_repository(&config.test_org, repo_name.as_ref())
        .await?;

    assert!(
        repository.is_private(),
        "Repository should default to private when no visibility specified"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Default visibility test passed");
    Ok(())
}

/// Test concurrent repository creation with different visibilities.
///
/// Verifies that visibility resolution is thread-safe and handles
/// concurrent operations correctly.
#[tokio::test]
async fn test_concurrent_visibility_resolution() -> Result<()> {
    init_test_logging();
    info!("Testing concurrent visibility resolution");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    // Create 3 repositories concurrently with different visibilities
    let repo_name_1 = RepositoryName::new(&format!("test-concurrent-1-{}", uuid::Uuid::new_v4()))?;
    let repo_name_2 = RepositoryName::new(&format!("test-concurrent-2-{}", uuid::Uuid::new_v4()))?;
    let repo_name_3 = RepositoryName::new(&format!("test-concurrent-3-{}", uuid::Uuid::new_v4()))?;

    let request_1 = RepositoryCreationRequestBuilder::new(
        repo_name_1.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Private)
    .build();

    let request_2 = RepositoryCreationRequestBuilder::new(
        repo_name_2.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Public)
    .build();

    let request_3 = RepositoryCreationRequestBuilder::new(
        repo_name_3.clone(),
        OrganizationName::new(&config.test_org)?,
        )
    ).template(TemplateName::new("template-test-basic")?)
    .build(); // No visibility (defaults to Private)

    // Execute all three concurrently
    let (_result_1, _result_2, _result_3) = tokio::try_join!(
        create_repository(
            request_1,
            metadata_provider.as_ref(),
            &auth_service,
            ".reporoller",
            policy_provider.clone(),
            env_detector.clone(),
        ),
        create_repository(
            request_2,
            metadata_provider.as_ref(),
            &auth_service,
            ".reporoller",
            policy_provider.clone(),
            env_detector.clone(),
        ),
        create_repository(
            request_3,
            metadata_provider.as_ref(),
            &auth_service,
            ".reporoller",
            policy_provider,
            env_detector,
        ),
    )?;

    info!("✓ All three repositories created concurrently");

    // Verify visibilities
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let github_client = GitHubClient::new(octocrab);

    let (repo_1, repo_2, repo_3) = tokio::try_join!(
        github_client.get_repository(&config.test_org, repo_name_1.as_ref()),
        github_client.get_repository(&config.test_org, repo_name_2.as_ref()),
        github_client.get_repository(&config.test_org, repo_name_3.as_ref()),
    )?;

    assert!(repo_1.is_private(), "Repository 1 should be private");
    assert!(!repo_2.is_private(), "Repository 2 should be public");
    assert!(
        repo_3.is_private(),
        "Repository 3 should be private (default)"
    );

    // Cleanup all three
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    tokio::try_join!(
        async { cleanup.delete_repository(repo_name_1.as_ref()).await },
        async { cleanup.delete_repository(repo_name_2.as_ref()).await },
        async { cleanup.delete_repository(repo_name_3.as_ref()).await },
    )
    .ok();

    info!("✓ Concurrent visibility resolution test passed");
    Ok(())
}
