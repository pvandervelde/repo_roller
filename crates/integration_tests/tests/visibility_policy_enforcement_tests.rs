//! Visibility policy enforcement integration tests.
//!
//! These tests verify that organization visibility policies are correctly enforced
//! when creating repositories using the real test infrastructure in glitchgrove.
//!
//! Tests cover:
//! - Required policy (force specific visibility)
//! - Restricted policy (block specific visibilities)
//! - Policy violations and error handling
//! - Policy cache behavior

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

/// Create test dependencies with specific metadata repo.
async fn create_test_dependencies(
    config: &TestConfig,
    metadata_repo: &str,
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
        MetadataProviderConfig::explicit(metadata_repo),
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

/// Test required policy enforcement.
///
/// Uses `.reporoller-required` metadata repo which requires all repositories to be private.
/// Verifies that attempting to create a public repository is rejected.
#[tokio::test]
async fn test_required_policy_enforces_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing required visibility policy enforcement");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config, ".reporoller-required").await?;

    let repo_name = RepositoryName::new(format!("test-required-{}", uuid::Uuid::new_v4()))?;

    // Try to create a public repository when policy requires private
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Public)
    .build();

    // This should fail due to policy violation
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-required",
        policy_provider,
        env_detector,
    )
    .await;

    assert!(
        result.is_err(),
        "Creating public repository should fail when policy requires private"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("visibility")
            || error_msg.contains("policy")
            || error_msg.contains("Public"),
        "Error should mention visibility policy violation: {}",
        error_msg
    );

    info!("✓ Required policy correctly rejected public repository");
    Ok(())
}

/// Test required policy allows compliant visibility.
///
/// Uses `.reporoller-required` metadata repo which requires private.
/// Verifies that creating a private repository succeeds.
#[tokio::test]
async fn test_required_policy_allows_compliant_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing required policy allows compliant visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config, ".reporoller-required").await?;

    let repo_name = RepositoryName::new(format!("test-compliant-{}", uuid::Uuid::new_v4()))?;

    // Create a private repository as required by policy
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Private)
    .build();

    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-required",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify it's actually private
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
        "Repository should be private as required by policy"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Required policy correctly allowed compliant repository");
    Ok(())
}

/// Test restricted policy blocks prohibited visibility.
///
/// Uses `.reporoller-restricted` metadata repo which prohibits public repositories.
/// Verifies that attempting to create a public repository is rejected.
#[tokio::test]
async fn test_restricted_policy_blocks_prohibited_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing restricted policy blocks prohibited visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config, ".reporoller-restricted").await?;

    let repo_name = RepositoryName::new(format!("test-blocked-{}", uuid::Uuid::new_v4()))?;

    // Try to create a public repository when policy restricts it
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Public)
    .build();

    // This should fail due to policy restriction
    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-restricted",
        policy_provider,
        env_detector,
    )
    .await;

    assert!(
        result.is_err(),
        "Creating public repository should fail when policy restricts public"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("visibility")
            || error_msg.contains("prohibited")
            || error_msg.contains("Public"),
        "Error should mention visibility restriction: {}",
        error_msg
    );

    info!("✓ Restricted policy correctly blocked prohibited visibility");
    Ok(())
}

/// Test restricted policy allows non-prohibited visibility.
///
/// Uses `.reporoller-restricted` metadata repo which only prohibits public.
/// Verifies that creating a private repository succeeds.
#[tokio::test]
async fn test_restricted_policy_allows_permitted_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing restricted policy allows permitted visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config, ".reporoller-restricted").await?;

    let repo_name = RepositoryName::new(format!("test-permitted-{}", uuid::Uuid::new_v4()))?;

    // Create a private repository (not prohibited by policy)
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Private)
    .build();

    let result = create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-restricted",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Repository created: {} (ID: {})",
        result.repository_url, result.repository_id
    );

    // Verify it's private
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
        "Repository should be private as chosen"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Restricted policy correctly allowed permitted visibility");
    Ok(())
}

/// Test unrestricted policy allows all visibilities.
///
/// Uses `.reporoller` metadata repo with unrestricted policy.
/// Verifies that both public and private repositories can be created.
#[tokio::test]
async fn test_unrestricted_policy_allows_all_visibilities() -> Result<()> {
    init_test_logging();
    info!("Testing unrestricted policy allows all visibilities");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config, ".reporoller").await?;

    let repo_name_pub = RepositoryName::new(format!("test-unres-pub-{}", uuid::Uuid::new_v4()))?;
    let repo_name_priv = RepositoryName::new(format!("test-unres-priv-{}", uuid::Uuid::new_v4()))?;

    // Create public repository
    let request_pub = RepositoryCreationRequestBuilder::new(
        repo_name_pub.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Public)
    .build();

    let result_pub = create_repository(
        request_pub,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        policy_provider.clone(),
        env_detector.clone(),
    )
    .await?;

    info!("✓ Public repository created: {}", result_pub.repository_url);

    // Create private repository
    let request_priv = RepositoryCreationRequestBuilder::new(
        repo_name_priv.clone(),
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .with_visibility(RepositoryVisibility::Private)
    .build();

    let result_priv = create_repository(
        request_priv,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        policy_provider,
        env_detector,
    )
    .await?;

    info!(
        "✓ Private repository created: {}",
        result_priv.repository_url
    );

    // Verify visibilities
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    let github_client = GitHubClient::new(octocrab);

    let (repo_pub, repo_priv) = tokio::try_join!(
        github_client.get_repository(&config.test_org, repo_name_pub.as_ref()),
        github_client.get_repository(&config.test_org, repo_name_priv.as_ref()),
    )?;

    assert!(
        !repo_pub.is_private(),
        "Public repository should not be private"
    );
    assert!(
        repo_priv.is_private(),
        "Private repository should be private"
    );

    // Cleanup both
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    tokio::try_join!(
        async { cleanup.delete_repository(repo_name_pub.as_ref()).await },
        async { cleanup.delete_repository(repo_name_priv.as_ref()).await },
    )
    .ok();

    info!("✓ Unrestricted policy correctly allowed both visibilities");
    Ok(())
}
