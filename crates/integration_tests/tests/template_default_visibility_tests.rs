//! Template default visibility integration tests.
//!
//! These tests verify that template default_visibility settings are correctly
//! applied when no explicit visibility is specified in the creation request.
//!
//! Tests cover:
//! - Template with default_visibility = "private"
//! - Template with default_visibility = "public"
//! - Template without default_visibility (uses system default)
//! - Template default overridden by user preference

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

/// Create test dependencies.
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

/// Test template with default_visibility = "private".
///
/// Uses `template-test-basic` which has `default_visibility = "private"`.
/// Verifies that when no visibility is specified, the template default is used.
#[tokio::test]
async fn test_template_default_private_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing template default private visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-tmpl-priv-{}", uuid::Uuid::new_v4()))?;

    // Create repository without specifying visibility - should use template default (private)
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?, // Has default_visibility = "private"
    )
    .build(); // No .with_visibility() call

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

    // Verify it's private (from template default)
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
        "Repository should be private from template default"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Template default private visibility correctly applied");
    Ok(())
}

/// Test template with default_visibility = "public".
///
/// Uses `template-test-variables` which has `default_visibility = "public"`.
/// Verifies that when no visibility is specified, the template default is used.
#[tokio::test]
async fn test_template_default_public_visibility() -> Result<()> {
    init_test_logging();
    info!("Testing template default public visibility");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-tmpl-pub-{}", uuid::Uuid::new_v4()))?;

    // Create repository without specifying visibility - should use template default (public)
    // Must provide all required variables for template-test-variables
    let mut variables = std::collections::HashMap::new();
    variables.insert("project_name".to_string(), repo_name.as_ref().to_string());
    variables.insert("version".to_string(), "0.1.0".to_string());
    variables.insert("author_name".to_string(), "Integration Test".to_string());
    variables.insert("author_email".to_string(), "test@example.com".to_string());
    variables.insert(
        "project_description".to_string(),
        "A test repository for template default visibility".to_string(),
    );
    variables.insert("license".to_string(), "MIT".to_string());
    variables.insert("license_type".to_string(), "MIT".to_string());
    variables.insert("environment".to_string(), "test".to_string());
    variables.insert("debug_mode".to_string(), "true".to_string());

    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-variables")?, // Has default_visibility = "public"
    )
    .variables(variables)
    .build(); // No .with_visibility() call

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

    // Verify it's public (from template default)
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
        "Repository should be public from template default"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ Template default public visibility correctly applied");
    Ok(())
}

/// Test user preference overrides template default.
///
/// Uses `template-test-basic` (default_visibility = "private").
/// Explicitly requests public visibility.
/// Verifies that user preference takes precedence over template default.
#[tokio::test]
async fn test_user_preference_overrides_template_default() -> Result<()> {
    init_test_logging();
    info!("Testing user preference overrides template default");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-override-{}", uuid::Uuid::new_v4()))?;

    // Template has default_visibility = "private", but we explicitly request public
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?, // Has default_visibility = "private"
    )
    .with_visibility(RepositoryVisibility::Public) // User prefers public
    .build();

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

    // Verify it's public (user preference overrode template default)
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
        "Repository should be public from user preference (overriding template default)"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ User preference correctly overrode template default");
    Ok(())
}

/// Test template without default_visibility uses system default.
///
/// Uses `template-test-filtering` which has no default_visibility setting.
/// Verifies that the system default (Private) is used.
#[tokio::test]
async fn test_no_template_default_uses_system_default() -> Result<()> {
    init_test_logging();
    info!("Testing no template default uses system default");

    let config = TestConfig::from_env()?;
    let (auth_service, metadata_provider, policy_provider, env_detector) =
        create_test_dependencies(&config).await?;

    let repo_name = RepositoryName::new(&format!("test-sysdef-{}", uuid::Uuid::new_v4()))?;

    // Create repository without specifying visibility, using template with no default
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-filtering")?, // No default_visibility
    )
    .build(); // No .with_visibility() call

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

    // Verify it's private (system default)
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
        "Repository should be private from system default"
    );

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup =
        RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());
    cleanup.delete_repository(repo_name.as_ref()).await.ok();

    info!("✓ System default correctly applied when template has no default");
    Ok(())
}
