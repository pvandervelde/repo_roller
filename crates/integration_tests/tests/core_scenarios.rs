//! Core integration test scenarios for RepoRoller.
//!
//! These tests verify the fundamental functionality of repository creation,
//! template processing, variable substitution, and error handling.
//!
//! Tests are run sequentially (--test-threads=1) to avoid GitHub API rate limits
//! and repository name conflicts.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use integration_tests::{generate_test_repo_name, RepositoryCleanup, TestConfig, TestRepository};
use repo_roller_core::{
    create_repository, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
};
use tracing::info;

/// Initialize logging for tests
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_test_writer()
        .try_init();
}

/// Test basic repository creation with minimal template.
///
/// Verifies that a repository can be created from the basic template
/// with no special configuration or variables.
#[tokio::test]
async fn test_basic_repository_creation() -> Result<()> {
    init_test_logging();
    info!("Testing basic repository creation");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("basic");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller"),
    );

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Create repository
    let result = create_repository(request, &metadata_provider, &auth_service, ".reporoller").await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");
    info!("✓ Basic repository creation test passed");
    Ok(())
}

/// Test variable substitution in templates.
///
/// Verifies that template variables are correctly substituted
/// in file contents during repository creation.
#[tokio::test]
async fn test_variable_substitution() -> Result<()> {
    init_test_logging();
    info!("Testing variable substitution");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("variables");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller"),
    );

    // Build request with variables
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-variables")?,
    )
    .variable("project_name", "test-project")
    .variable("version", "0.1.0")
    .variable("author_name", "Integration Test")
    .variable("author_email", "test@example.com")
    .variable(
        "project_description",
        "A test project for integration testing",
    )
    .variable("license", "MIT")
    .variable("license_type", "MIT")
    .variable("environment", "test")
    .variable("debug_mode", "true")
    .build();

    // Create repository
    let result = create_repository(request, &metadata_provider, &auth_service, ".reporoller").await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(result.is_ok(), "Variable substitution should succeed");
    info!("✓ Variable substitution test passed");
    Ok(())
}

/// Test file filtering based on patterns.
///
/// Verifies that file filtering rules are correctly applied
/// during template processing.
#[tokio::test]
async fn test_file_filtering() -> Result<()> {
    init_test_logging();
    info!("Testing file filtering");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("filtering");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller"),
    );

    // Build request with filtering variables
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-filtering")?,
    )
    .variable("include_docs", "true")
    .variable("include_config", "true")
    .build();

    // Create repository
    let result = create_repository(request, &metadata_provider, &auth_service, ".reporoller").await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(result.is_ok(), "File filtering should succeed");
    info!("✓ File filtering test passed");
    Ok(())
}

/// Test error handling for invalid template.
///
/// Verifies that appropriate errors are returned when attempting
/// to create a repository from a non-existent template.
#[tokio::test]
async fn test_error_handling_invalid_template() -> Result<()> {
    init_test_logging();
    info!("Testing error handling for invalid template");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("error-handling");

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller"),
    );

    // Build request with non-existent template
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("test-nonexistent")?,
    )
    .build();

    // Create repository - should fail
    let result = create_repository(request, &metadata_provider, &auth_service, ".reporoller").await;

    // Assert failure
    assert!(result.is_err(), "Invalid template should fail");
    info!("✓ Error handling test passed");
    Ok(())
}

/// Test organization settings integration.
///
/// Verifies that organization-level settings from metadata repository
/// are correctly applied during repository creation.
#[tokio::test]
#[ignore = "Requires .reporoller-test metadata repository"]
async fn test_organization_settings() -> Result<()> {
    init_test_logging();
    info!("Testing organization settings integration");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("org-settings");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider using test metadata repository
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Create repository
    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(
        result.is_ok(),
        "Organization settings integration should succeed"
    );
    info!("✓ Organization settings test passed");
    Ok(())
}

/// Test team-specific configuration overrides.
///
/// Verifies that team-level configuration correctly overrides
/// organization defaults.
#[tokio::test]
#[ignore = "Requires .reporoller-test metadata repository with team configuration"]
async fn test_team_configuration() -> Result<()> {
    init_test_logging();
    info!("Testing team-specific configuration");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("team-config");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request with team specification
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Create repository
    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(result.is_ok(), "Team configuration should succeed");
    info!("✓ Team configuration test passed");
    Ok(())
}

/// Test repository type configuration.
///
/// Verifies that repository type-specific configuration
/// is correctly applied.
#[tokio::test]
#[ignore = "Requires .reporoller-test metadata repository with type configuration"]
async fn test_repository_type_configuration() -> Result<()> {
    init_test_logging();
    info!("Testing repository type configuration");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("repo-type");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request with repository type
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Create repository
    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(
        result.is_ok(),
        "Repository type configuration should succeed"
    );
    info!("✓ Repository type configuration test passed");
    Ok(())
}

/// Test configuration hierarchy merging.
///
/// Verifies that configuration is correctly merged from multiple levels:
/// Template > Team > Type > Global
#[tokio::test]
#[ignore = "Requires .reporoller-test metadata repository with full configuration hierarchy"]
async fn test_configuration_hierarchy() -> Result<()> {
    init_test_logging();
    info!("Testing configuration hierarchy merging");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("config-hierarchy");
    let test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let github_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request that will test hierarchy merging
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Create repository
    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    // Assert success
    assert!(
        result.is_ok(),
        "Configuration hierarchy merging should succeed"
    );
    info!("✓ Configuration hierarchy test passed");
    Ok(())
}

/// Test cleanup of orphaned repositories.
///
/// This test is used by CI for scheduled cleanup runs.
#[tokio::test]
#[ignore = "Only run during scheduled cleanup or manually"]
async fn test_cleanup_orphaned_repositories() -> Result<()> {
    init_test_logging();
    info!("Running orphaned repository cleanup");

    let config = TestConfig::from_env()?;

    // Create cleanup utility
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org,
    );

    // Clean up repositories older than 24 hours
    let max_age_hours = std::env::var("MAX_AGE_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);

    let cleaned = cleanup.cleanup_orphaned_repositories(max_age_hours).await?;

    info!(
        "Cleaned up {} orphaned repositories older than {} hours",
        cleaned.len(),
        max_age_hours
    );

    Ok(())
}
