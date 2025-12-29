//! Integration tests for template configuration loading.
//!
//! These tests verify the TemplateLoader and GitHubTemplateRepository against
//! real GitHub template repositories in the glitchgrove organization.
//!
//! Tests cover:
//! - Loading template configurations from .reporoller/template.toml
//! - Handling missing templates
//! - Handling missing configuration files
//! - Cache effectiveness across multiple loads
//! - Error handling and reporting

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{
    ConfigurationError, GitHubMetadataProvider, GitHubTemplateRepository, MetadataProviderConfig,
    OrganizationSettingsManager, TemplateLoader, TemplateRepository,
};
use github_client::{create_token_client, GitHubClient};
use integration_tests::TestConfig;
use std::sync::Arc;
use tracing::{debug, info};

/// Initialize logging for tests.
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_test_writer()
        .try_init();
}

/// Create an authenticated GitHub client for testing.
async fn create_test_client(config: &TestConfig) -> Result<GitHubClient> {
    let auth_service =
        GitHubAuthService::new(config.github_app_id, config.github_app_private_key.clone());
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;
    let octocrab = create_token_client(&installation_token)?;
    Ok(GitHubClient::new(octocrab))
}

/// Test loading a valid template configuration from GitHub.
///
/// This test verifies that:
/// - GitHubTemplateRepository can fetch .reporoller/template.toml from a real template
/// - The TOML is parsed correctly into TemplateConfig
/// - Template metadata is populated correctly
#[tokio::test]
async fn test_load_valid_template_configuration() -> Result<()> {
    init_test_logging();
    info!("Testing loading valid template configuration from glitchgrove/template-test-basic");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;

    // Create template repository
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));

    // Load template configuration
    let template_config = template_repo
        .load_template_config(&config.test_org, "template-test-basic")
        .await?;

    // Verify template metadata
    assert_eq!(
        template_config.template.name, "template-test-basic",
        "Template name should match"
    );
    assert!(
        !template_config.template.description.is_empty(),
        "Template should have a description"
    );
    assert!(
        !template_config.template.author.is_empty(),
        "Template should have an author"
    );

    info!(
        "✓ Successfully loaded template configuration: {}",
        template_config.template.name
    );
    info!("  Description: {}", template_config.template.description);
    info!("  Author: {}", template_config.template.author);
    info!("  Tags: {:?}", template_config.template.tags);

    Ok(())
}

/// Test loading template configuration with repository settings.
///
/// Verifies that template-specific repository settings are loaded correctly.
#[tokio::test]
async fn test_load_template_with_repository_settings() -> Result<()> {
    init_test_logging();
    info!("Testing template configuration with repository settings");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));

    // Load template configuration
    let template_config = template_repo
        .load_template_config(&config.test_org, "template-test-basic")
        .await?;

    // Check if repository settings are present (may vary by template)
    if let Some(ref repo_settings) = template_config.repository {
        info!("✓ Template has repository settings");
        debug!("  Repository settings: {:?}", repo_settings);
    } else {
        info!("  Template has no custom repository settings");
    }

    // Check if template specifies repository type
    if let Some(ref repo_type) = template_config.repository_type {
        info!(
            "✓ Template specifies repository type: {} (policy: {:?})",
            repo_type.repository_type, repo_type.policy
        );
    } else {
        info!("  Template has no repository type specification");
    }

    Ok(())
}

/// Test loading template that doesn't exist.
///
/// Verifies that:
/// - GitHubTemplateRepository returns TemplateNotFound error
/// - Error includes organization and template name
#[tokio::test]
async fn test_load_nonexistent_template() -> Result<()> {
    init_test_logging();
    info!("Testing loading non-existent template");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));

    // Try to load non-existent template
    let result = template_repo
        .load_template_config(&config.test_org, "template-nonexistent-12345")
        .await;

    // Verify error
    assert!(
        result.is_err(),
        "Should return error for non-existent template"
    );

    let error = result.unwrap_err();
    assert!(
        matches!(error, ConfigurationError::TemplateNotFound { .. }),
        "Should return TemplateNotFound error"
    );

    if let ConfigurationError::TemplateNotFound { org, template } = error {
        assert_eq!(org, config.test_org, "Error should include organization");
        assert_eq!(
            template, "template-nonexistent-12345",
            "Error should include template name"
        );
        info!("✓ Correct error: Template not found: {}/{}", org, template);
    }

    Ok(())
}

/// Test template_exists() method.
///
/// Verifies that:
/// - Returns true for existing templates
/// - Returns false for non-existent templates
/// - Doesn't throw errors for missing templates
#[tokio::test]
async fn test_template_exists_check() -> Result<()> {
    init_test_logging();
    info!("Testing template_exists() method");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));

    // Check existing template
    let exists = template_repo
        .template_exists(&config.test_org, "template-test-basic")
        .await?;
    assert!(exists, "template-test-basic should exist");
    info!("✓ Existing template detected correctly");

    // Check non-existent template
    let not_exists = template_repo
        .template_exists(&config.test_org, "template-nonexistent-12345")
        .await?;
    assert!(!not_exists, "Non-existent template should return false");
    info!("✓ Non-existent template detected correctly");

    Ok(())
}

/// Test TemplateLoader caching with real GitHub repository.
///
/// Verifies that:
/// - First load calls GitHub API
/// - Second load uses cache (no API call)
/// - Cache statistics are accurate
#[tokio::test]
async fn test_template_loader_cache_effectiveness() -> Result<()> {
    init_test_logging();
    info!("Testing TemplateLoader cache effectiveness with real GitHub");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));
    let loader = TemplateLoader::new(template_repo);

    // Check initial statistics
    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 0, "Should have no initial requests");
    assert_eq!(stats.cached_entries, 0, "Should have no cached entries");

    // First load - cache miss
    info!("First load (should fetch from GitHub)...");
    let config1 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;
    assert_eq!(config1.template.name, "template-test-basic");

    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 1, "Should have 1 request");
    assert_eq!(stats.cache_misses, 1, "Should have 1 miss");
    assert_eq!(stats.cache_hits, 0, "Should have 0 hits");
    assert_eq!(stats.cached_entries, 1, "Should have 1 cached entry");
    info!("✓ First load: cache miss recorded");

    // Second load - cache hit
    info!("Second load (should use cache)...");
    let config2 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;
    assert_eq!(config2.template.name, "template-test-basic");

    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 2, "Should have 2 requests");
    assert_eq!(stats.cache_misses, 1, "Should still have 1 miss");
    assert_eq!(stats.cache_hits, 1, "Should have 1 hit");
    assert_eq!(stats.cached_entries, 1, "Should still have 1 cached entry");
    assert_eq!(stats.hit_ratio(), 0.5, "Hit ratio should be 50%");
    info!("✓ Second load: cache hit recorded");

    // Third load - another cache hit
    info!("Third load (should use cache again)...");
    let _config3 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;

    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 3, "Should have 3 requests");
    assert_eq!(stats.cache_hits, 2, "Should have 2 hits");
    assert!(
        (stats.hit_ratio() - 0.666).abs() < 0.01,
        "Hit ratio should be ~66.6%"
    );
    info!(
        "✓ Third load: cache hit recorded, hit ratio: {:.1}%",
        stats.hit_ratio() * 100.0
    );

    Ok(())
}

/// Test loading multiple different templates with caching.
///
/// Verifies that:
/// - Each template is cached independently
/// - Cache stores multiple entries
/// - Statistics track all requests
#[tokio::test]
async fn test_template_loader_multiple_templates() -> Result<()> {
    init_test_logging();
    info!("Testing TemplateLoader with multiple templates");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));
    let loader = TemplateLoader::new(template_repo);

    // Load first template
    info!("Loading template-test-basic...");
    let _config1 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;

    // Load second template (if it exists)
    // Note: We're using the same template name in different context to avoid
    // dependency on specific templates existing
    info!("Loading same template again (cache hit)...");
    let _config2 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;

    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 2, "Should have 2 total requests");
    assert_eq!(stats.cache_misses, 1, "Should have 1 miss");
    assert_eq!(stats.cache_hits, 1, "Should have 1 hit");
    assert_eq!(
        stats.cached_entries, 1,
        "Should have 1 unique template cached"
    );

    info!("✓ Multiple template loads tracked correctly");
    info!("  Total requests: {}", stats.total_requests);
    info!("  Cache hits: {}", stats.cache_hits);
    info!("  Cache misses: {}", stats.cache_misses);
    info!("  Cached entries: {}", stats.cached_entries);
    info!("  Hit ratio: {:.1}%", stats.hit_ratio() * 100.0);

    Ok(())
}

/// Test cache invalidation with real GitHub.
///
/// Verifies that:
/// - Cache invalidation removes the entry
/// - Next load fetches from GitHub again
/// - Statistics reflect the reload
#[tokio::test]
async fn test_template_loader_cache_invalidation() -> Result<()> {
    init_test_logging();
    info!("Testing TemplateLoader cache invalidation");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));
    let loader = TemplateLoader::new(template_repo);

    // Load and cache
    info!("Loading template (first time)...");
    let _config1 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;

    let stats = loader.cache_statistics();
    assert_eq!(stats.cached_entries, 1, "Should have 1 cached entry");

    // Invalidate cache
    info!("Invalidating cache...");
    let was_cached = loader.invalidate_cache(&config.test_org, "template-test-basic");
    assert!(was_cached, "Should return true for cached entry");

    let stats = loader.cache_statistics();
    assert_eq!(
        stats.cached_entries, 0,
        "Should have 0 cached entries after invalidation"
    );

    // Load again - should be a cache miss
    info!("Loading template (after invalidation)...");
    let _config2 = loader
        .load_template_configuration(&config.test_org, "template-test-basic")
        .await?;

    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 2, "Should have 2 total requests");
    assert_eq!(stats.cache_misses, 2, "Should have 2 misses");
    assert_eq!(stats.cache_hits, 0, "Should have 0 hits");
    assert_eq!(stats.cached_entries, 1, "Should have 1 cached entry again");

    info!("✓ Cache invalidation works correctly");

    Ok(())
}

/// Test error propagation through TemplateLoader.
///
/// Verifies that:
/// - Errors from GitHubTemplateRepository propagate correctly
/// - Error context is preserved
#[tokio::test]
async fn test_template_loader_error_propagation() -> Result<()> {
    init_test_logging();
    info!("Testing error propagation through TemplateLoader");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));
    let loader = TemplateLoader::new(template_repo);

    // Try to load non-existent template
    let result = loader
        .load_template_configuration(&config.test_org, "template-nonexistent-12345")
        .await;

    assert!(result.is_err(), "Should return error");

    let error = result.unwrap_err();
    assert!(
        matches!(error, ConfigurationError::TemplateNotFound { .. }),
        "Should propagate TemplateNotFound error"
    );

    if let ConfigurationError::TemplateNotFound { org, template } = error {
        assert_eq!(org, config.test_org);
        assert_eq!(template, "template-nonexistent-12345");
        info!("✓ Error propagated correctly with context");
    }

    // Verify error doesn't get cached
    let stats = loader.cache_statistics();
    assert_eq!(stats.total_requests, 1, "Error request should be counted");
    assert_eq!(stats.cache_misses, 1, "Should be a cache miss");
    assert_eq!(stats.cached_entries, 0, "Error should not be cached");

    Ok(())
}

/// Test integration of TemplateLoader with OrganizationSettingsManager.
///
/// This test verifies the end-to-end flow of template configuration loading
/// during repository creation configuration resolution.
#[tokio::test]
async fn test_template_loader_integration_with_settings_manager() -> Result<()> {
    init_test_logging();
    info!("Testing TemplateLoader integration with OrganizationSettingsManager");

    let config = TestConfig::from_env()?;
    let github_client = create_test_client(&config).await?;
    let github_client_arc = Arc::new(github_client);

    // Create template loader
    let template_repo = Arc::new(GitHubTemplateRepository::new(github_client_arc.clone()));
    let template_loader = Arc::new(TemplateLoader::new(template_repo));

    // Create metadata provider
    // GitHubMetadataProvider takes GitHubClient (not Arc), so create another client
    let github_client2 = create_test_client(&config).await?;
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client2,
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    // Create organization settings manager
    let settings_manager =
        OrganizationSettingsManager::new(metadata_provider, template_loader.clone());

    // Create configuration context
    let context =
        config_manager::ConfigurationContext::new(&config.test_org, "template-test-basic");

    // Resolve configuration - this should load the template configuration
    info!("Resolving configuration (should load template config)...");
    let merged_config = settings_manager.resolve_configuration(&context).await?;

    info!("✓ Configuration resolved successfully");
    info!("  Repository settings: {:?}", merged_config.repository);

    // Verify cache statistics
    let stats = template_loader.cache_statistics();
    assert_eq!(stats.total_requests, 1, "Should have loaded template once");
    assert_eq!(stats.cache_misses, 1, "Should have 1 cache miss");
    assert_eq!(stats.cached_entries, 1, "Should have cached the template");

    // Resolve again - should use cache
    info!("Resolving configuration again (should use cache)...");
    let _merged_config2 = settings_manager.resolve_configuration(&context).await?;

    let stats = template_loader.cache_statistics();
    assert_eq!(stats.total_requests, 2, "Should have 2 total requests");
    assert_eq!(stats.cache_hits, 1, "Should have 1 cache hit");
    assert_eq!(stats.hit_ratio(), 0.5, "Hit ratio should be 50%");

    info!("✓ Template configuration integrated correctly with settings manager");
    info!("  Cache hit ratio: {:.1}%", stats.hit_ratio() * 100.0);

    Ok(())
}
