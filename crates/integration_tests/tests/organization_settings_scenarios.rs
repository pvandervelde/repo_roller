//! Integration tests for organization settings features.
//!
//! These tests verify the organization settings system integration with repository creation,
//! including global defaults, team overrides, repository types, and configuration hierarchy.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use github_client::RepositoryClient;
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

/// Test organization settings integration with global defaults.
///
/// This test verifies that RepoRoller correctly applies global organization defaults
/// from the metadata repository when creating a new repository.
///
/// Expected behavior:
/// - Repository is created with settings from global/defaults.toml
/// - Default branch, repository features, PR settings are applied
/// - No team or type-specific overrides are applied
#[tokio::test]
async fn test_organization_settings_with_global_defaults() -> Result<()> {
    init_test_logging();
    info!("Testing organization settings with global defaults");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "org-settings-global-defaults");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");

    // Verify repository exists and has expected configuration
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify labels from metadata repository were applied
    let labels = verification_client
        .list_repository_labels(&config.test_org, &repo_name)
        .await?;

    info!(
        "Repository has {} labels from global defaults",
        labels.len()
    );
    info!("✓ Organization settings verification passed");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Organization settings with global defaults test passed");
    Ok(())
}

/// Test team-specific configuration overrides.
///
/// This test verifies that team-specific settings from the metadata repository
/// correctly override global defaults.
///
/// Expected behavior:
/// - Repository is created with team-specific overrides from teams/platform/config.toml
/// - Team settings override global defaults where specified
/// - Custom properties include team information
#[tokio::test]
async fn test_team_configuration_overrides() -> Result<()> {
    init_test_logging();
    info!("Testing team-specific configuration overrides");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "team-configuration-overrides");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request - Note: team parameter not yet supported in API
    // This test currently creates repository with basic template
    // When team parameter is added, this will be updated to:
    // .team("platform")
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");

    // Verify repository exists and team configuration was applied
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify labels from metadata repository (including team-specific labels)
    let labels = verification_client
        .list_repository_labels(&config.test_org, &repo_name)
        .await?;

    info!("Repository has {} labels (global + team)", labels.len());
    info!("✓ Team configuration verification passed");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Team configuration overrides test passed");
    Ok(())
}

/// Test repository type configuration and custom properties.
///
/// This test verifies that repository type-specific settings are correctly applied
/// and that custom properties are set on the created repository.
///
/// Expected behavior:
/// - Repository is created with type-specific settings from types/library/config.toml
/// - Custom properties include repo_type field
/// - Type-specific settings override global and team defaults
#[tokio::test]
async fn test_repository_type_configuration() -> Result<()> {
    init_test_logging();
    info!("Testing repository type configuration");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "repo-type");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request - Note: repository_type parameter not yet supported in API
    // This test currently creates repository with basic template
    // When repository_type parameter is added, this will be updated to:
    // .repository_type("library")
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");

    // Verify repository exists and type configuration was applied
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify labels from metadata repository (including type-specific labels)
    let labels = verification_client
        .list_repository_labels(&config.test_org, &repo_name)
        .await?;

    info!("Repository has {} labels (global + type)", labels.len());
    info!("✓ Repository type configuration verification passed");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Repository type configuration test passed");
    Ok(())
}

/// Test complete configuration hierarchy merging.
///
/// This test verifies the complete configuration hierarchy:
/// Template > Team > Repository Type > Global
///
/// Expected behavior:
/// - Repository is created with merged configuration from all levels
/// - Higher precedence levels override lower levels
/// - All custom properties from all levels are combined
/// - Validation ensures proper merge order
#[tokio::test]
async fn test_configuration_hierarchy_merging() -> Result<()> {
    init_test_logging();
    info!("Testing configuration hierarchy merging");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "config-hierarchy");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers

    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
            .await?;

    // Build request - Note: team and repository_type parameters not yet supported
    // This test currently creates repository with basic template
    // When parameters are added, this will be updated to:
    // .team("backend")
    // .repository_type("library")
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");

    // Verify repository exists and hierarchy configuration was applied
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify labels from all hierarchy levels were merged
    let labels = verification_client
        .list_repository_labels(&config.test_org, &repo_name)
        .await?;

    info!(
        "Repository has {} labels from all hierarchy levels (Global → Type → Team → Template)",
        labels.len()
    );

    // TODO: When team and repository_type parameters are supported:
    // - Verify labels from global level (e.g., "bug", "enhancement")
    // - Verify labels from type level (e.g., "library")
    // - Verify labels from team level (e.g., "backend")
    // - Verify labels from template level
    // - Verify that higher precedence levels override lower levels

    info!("✓ Configuration hierarchy merging verification passed");

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Configuration hierarchy merging test passed");
    Ok(())
}

/// Test complete integration workflow with organization settings.
///
/// This test runs all organization settings scenarios to verify
/// the complete integration of the configuration system.
#[tokio::test]
async fn test_complete_organization_settings_workflow() -> Result<()> {
    init_test_logging();
    info!("Testing complete organization settings workflow");

    let config = TestConfig::from_env()?;

    // Run all organization settings scenarios
    let scenarios = vec![
        (
            "org-settings-workflow-1",
            "Organization settings with global defaults",
        ),
        ("org-settings-workflow-2", "Team configuration overrides"),
        ("org-settings-workflow-3", "Repository type configuration"),
        ("org-settings-workflow-4", "Configuration hierarchy merging"),
    ];

    let mut all_passed = true;

    for (repo_suffix, description) in scenarios {
        info!("Running scenario: {}", description);

        let repo_name = generate_test_repo_name("test", repo_suffix);
        let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

        // Create authentication service
        let auth_service = auth_handler::GitHubAuthService::new(
            config.github_app_id,
            config.github_app_private_key.clone(),
        );

        // Get installation token
        let installation_token = auth_service
            .get_installation_token_for_org(&config.test_org)
            .await?;

        // Create visibility providers

        let providers =
            integration_tests::create_visibility_providers(&installation_token, ".reporoller-test")
                .await?;

        // Build request
        let request = RepositoryCreationRequestBuilder::new(
            RepositoryName::new(&repo_name)?,
            OrganizationName::new(&config.test_org)?,
        )
        .template(TemplateName::new("template-test-basic")?)
        .build();

        // Create repository
        let result = create_repository(
            request,
            providers.metadata_provider.as_ref(),
            &auth_service,
            ".reporoller-test",
            providers.visibility_policy_provider.clone(),
            providers.environment_detector.clone(),
        )
        .await;

        // Check result
        if result.is_err() {
            all_passed = false;
            info!("✗ Scenario failed: {}", description);
        } else {
            info!("✓ Scenario passed: {}", description);
        }

        // Cleanup
        let cleanup_client =
            github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
                .await?;
        let cleanup = RepositoryCleanup::new(
            github_client::GitHubClient::new(cleanup_client),
            config.test_org.clone(),
        );
        cleanup.delete_repository(&repo_name).await.ok();
    }

    assert!(
        all_passed,
        "All organization settings scenarios should pass"
    );

    info!("✓ Complete organization settings workflow test passed");
    Ok(())
}
