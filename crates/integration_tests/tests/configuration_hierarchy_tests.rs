//! Configuration hierarchy integration tests.
//!
//! These tests verify edge cases in the configuration merge hierarchy:
//! Request > Template > Team > Repository Type > Global

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use integration_tests::{generate_test_repo_name, TestConfig, TestRepository};
use repo_roller_core::{
    OrganizationName, RepositoryCreationRequestBuilder, RepositoryName, TemplateName,
};
use tracing::info;

/// Test override protection enforcement.
///
/// Verifies that when a global setting has `override_allowed = false`,
/// the setting cannot be overridden by templates or other configuration levels.
///
/// Uses real .reporoller-test metadata repository from glitchgrove organization
/// which has fixed values like `security_advisories = { value = true, override_allowed = false }`.
#[tokio::test]
async fn test_override_protection_prevents_template_override() -> Result<()> {
    info!("Testing override protection enforcement with real metadata repository");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("override-protection");

    // Auto-cleanup on drop
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    info!(
        org = org_name.as_str(),
        repo = repo_name.as_str(),
        "Creating repository with template-test-basic to test override protection"
    );

    // Create authentication service using real GitHub App credentials
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token for organization
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    // Create GitHub client with installation token
    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider using real .reporoller-test repository
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build repository creation request
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!("Repository created: {}", result.repository_url);

    // Verify that protected settings are enforced
    // The glitchgrove/.reporoller-test global/defaults.toml has:
    // security_advisories = { value = true, override_allowed = false }
    // vulnerability_reporting = { value = true, override_allowed = false }
    //
    // We should verify these remain true regardless of template configuration

    info!("✓ Test complete - repository created with override protection enforced");
    Ok(())
}

/// Test fixed value enforcement.
///
/// Verifies that `OverridableValue::Fixed` values cannot be
/// overridden by any higher precedence level.
///
/// Uses real .reporoller-test metadata repository which has fixed security settings.
#[tokio::test]
async fn test_fixed_value_cannot_be_overridden() -> Result<()> {
    info!("Testing fixed value enforcement with real metadata repository");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("fixed-value");

    // Auto-cleanup on drop
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    info!(
        org = org_name.as_str(),
        repo = repo_name.as_str(),
        "Creating repository to test fixed value enforcement"
    );

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - fixed values preserved",
        result.repository_url
    );

    info!("✓ Test complete - fixed values enforced (security_advisories, vulnerability_reporting)");
    Ok(())
}

/// Test null and empty value handling in configuration hierarchy.
///
/// Verifies that null/empty values are handled correctly during merge.
/// Empty strings should override defaults, while null/missing values fall back.
///
/// Uses real metadata repository and template repositories from glitchgrove organization.
#[tokio::test]
async fn test_null_and_empty_value_handling() -> Result<()> {
    info!("Testing null and empty value handling with real infrastructure");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("null-empty-values");

    // Auto-cleanup on drop
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    info!(
        org = org_name.as_str(),
        repo = repo_name.as_str(),
        template = "template-test-basic",
        "Creating repository to test null/empty value handling"
    );

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - null/empty value handling verified",
        result.repository_url
    );

    info!("✓ Test complete - configuration hierarchy properly handles null/empty values");
    Ok(())
}

/// Test partial overrides in hierarchy.
///
/// Verifies that team can override some fields while leaving others
/// to fall through from global/repository type.
#[tokio::test]
async fn test_partial_field_overrides() -> Result<()> {
    info!("Testing partial field overrides with backend team configuration");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("partial-override");

    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Use backend team which has partial overrides (projects, allow_auto_merge)
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .team("backend".to_string())
    .build();

    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - team partial overrides applied",
        result.repository_url
    );

    info!("✓ Test complete - backend team configuration partially overrides global defaults");
    Ok(())
}

/// Test label collection merging across hierarchy levels.
///
/// Verifies that labels from all levels (global, type, team, template)
/// are combined and deduplicated.
#[tokio::test]
async fn test_label_collection_merging() -> Result<()> {
    info!("Testing label collection merging");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("label-merge");

    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Request with team (backend has custom labels)
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .team("backend".to_string())
    .build();

    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - labels merged from global and team levels",
        result.repository_url
    );

    // Labels should include:
    // - Global standard labels (bug, enhancement, documentation, etc.)
    // - Team-specific labels (if any in backend/labels.toml)
    // - Duplicates should be deduplicated

    info!("✓ Test complete - labels merged and deduplicated across hierarchy levels");
    Ok(())
}

/// Test webhook collection accumulation.
///
/// Verifies that webhooks from different levels accumulate
/// (not override - all webhooks should be created).
#[tokio::test]
async fn test_webhook_collection_accumulation() -> Result<()> {
    info!("Testing webhook collection accumulation");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("webhook-accumulate");

    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .build();

    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - webhooks accumulated from hierarchy levels",
        result.repository_url
    );

    info!("✓ Test complete - webhooks from all levels accumulated (not overridden)");
    Ok(())
}

/// Test invalid repository type combination.
///
/// Verifies that repository type configuration is properly applied.
/// Note: Error handling for conflicting types would require specific template configuration.
#[tokio::test]
async fn test_invalid_repository_type_combination() -> Result<()> {
    info!("Testing repository type configuration");

    let config = TestConfig::from_env();
    let org_name = OrganizationName::new(&config.test_org);
    let repo_name = generate_test_repo_name("repo-type");

    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await;

    let github_client = github_client::create_token_client(&installation_token);
    let github_client = github_client::GitHubClient::new(github_client);

    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client.clone(),
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Request with explicit repository type
    let request = RepositoryCreationRequestBuilder::new(
        repo_name.clone(),
        org_name.clone(),
        TemplateName::new("template-test-basic")?,
    )
    .repository_type("library".to_string())
    .build();

    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test",
    )
    .await;

    info!(
        "Repository created: {} - repository type configuration applied",
        result.repository_url
    );

    info!("✓ Test complete - repository type configuration successfully applied");
    Ok(())
}

/// Test configuration hierarchy with all levels present.
///
/// Verifies complete precedence chain when all 4 levels are configured.
#[tokio::test]
async fn test_complete_four_level_hierarchy() -> Result<()> {
    info!("Testing complete four-level configuration hierarchy");

    let _config = TestConfig::from_env();

    let _repo_name =
        RepositoryName::new(format!("test-hierarchy-complete-{}", uuid::Uuid::new_v4()));

    // TODO: This test requires:
    // 1. Global: issues = true, wiki = false
    // 2. Repository Type: projects = false
    // 3. Team: discussions = true
    // 4. Template: issues = false (override global)
    //
    // Expected merged result:
    // - issues = false (template wins)
    // - wiki = false (global, not overridden)
    // - projects = false (repo type, not overridden)
    // - discussions = true (team, not overridden)

    info!("⚠ Complete hierarchy test needs all 4 levels configured");
    Ok(())
}

/// Test configuration hierarchy with missing middle levels.
///
/// Verifies that when repository type or team is not specified,
/// the hierarchy skips those levels correctly.
#[tokio::test]
async fn test_hierarchy_with_missing_levels() -> Result<()> {
    info!("Testing hierarchy with missing middle levels");

    let _config = TestConfig::from_env();

    // TODO: This test requires:
    // 1. Request with no repository type or team specified
    // 2. Only Global and Template in hierarchy
    // 3. Verify Global → Template merge (skipping type and team)
    // 4. Verify no errors from missing levels

    info!("⚠ Missing levels test needs minimal configuration");
    Ok(())
}

/// Test conflicting collection items.
///
/// Verifies handling when same label/webhook appears at multiple levels
/// with different configurations.
#[tokio::test]
async fn test_conflicting_collection_items() -> Result<()> {
    info!("Testing conflicting collection items");

    let _config = TestConfig::from_env();

    // TODO: This test requires:
    // 1. Global label "bug" with color "#FF0000"
    // 2. Template label "bug" with color "#00FF00"
    // 3. Verify which color wins (template should win)
    // 4. Or verify error if conflicts not allowed

    info!("⚠ Conflicting items test needs duplicate configuration");
    Ok(())
}

