//! Integration tests for the configuration preview endpoint backend.
//!
//! These tests verify that `OrganizationSettingsManager::resolve_configuration()` —
//! the domain call that backs `POST /api/v1/orgs/:org/configuration/preview` —
//! returns correct merged configurations and populated source traces when run
//! against the glitchgrove test metadata repository.
//!
//! They also verify the two pre-conditions enforced by the handler itself:
//! - A non-existent template produces a `TemplateNotFound` error (→ HTTP 404)
//! - An unknown repository type produces `Ok(None)` from the metadata provider,
//!   which the handler maps to HTTP 400.

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{
    ConfigurationContext, ConfigurationError, ConfigurationSource, GitHubMetadataProvider,
    GitHubTemplateRepository, MetadataProviderConfig, MetadataRepositoryProvider,
    OrganizationSettingsManager, RepositoryTypeConfig, TemplateLoader,
};
use github_client::{create_token_client, GitHubClient};
use integration_tests::TestConfig;
use std::sync::Arc;
use tracing::info;

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

/// Build the settings manager wired to the glitchgrove `.reporoller-test` metadata.
async fn create_settings_manager(
    config: &TestConfig,
) -> Result<(OrganizationSettingsManager, Arc<GitHubMetadataProvider>)> {
    let client_for_templates = create_test_client(config).await?;
    let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(
        client_for_templates,
    )));
    let template_loader = Arc::new(TemplateLoader::new(template_repo));

    let client_for_metadata = create_test_client(config).await?;
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        client_for_metadata,
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    let manager = OrganizationSettingsManager::new(metadata_provider.clone(), template_loader);

    Ok((manager, metadata_provider))
}

/// Test that `resolve_configuration` returns a populated merged config and
/// a non-empty source trace for a known template.
///
/// This exercises the same code path as
/// `POST /api/v1/orgs/:org/configuration/preview`.
#[tokio::test]
async fn test_preview_merged_configuration_returns_populated_sources() -> Result<()> {
    init_test_logging();
    info!("Testing resolve_configuration returns populated source trace");

    let config = TestConfig::from_env()?;
    let (manager, _provider) = create_settings_manager(&config).await?;

    let context = ConfigurationContext::new(&config.test_org, "template-test-basic");
    let merged = manager.resolve_configuration(&context).await?;

    info!(
        "Source trace has {} configured fields",
        merged.source_trace.field_count()
    );

    assert!(
        merged.source_trace.field_count() > 0,
        "Source trace must have at least one entry when global defaults are loaded"
    );

    // Every source must be a recognised variant
    for field in merged.source_trace.configured_fields() {
        let src = merged
            .source_trace
            .get_source(field)
            .expect("configured_fields() should always have an entry");
        assert!(
            matches!(
                src,
                ConfigurationSource::Global
                    | ConfigurationSource::RepositoryType
                    | ConfigurationSource::Team
                    | ConfigurationSource::Template
            ),
            "field '{field}' has unexpected source {src:?}"
        );
        info!("  {field} → {src:?}");
    }

    Ok(())
}

/// Test that a field supplied only by global defaults is attributed to
/// `ConfigurationSource::Global`.
///
/// The `.reporoller-test` global/defaults.toml sets `issues = true`; since
/// `template-test-basic` does not override it the source must remain Global.
#[tokio::test]
async fn test_preview_global_only_field_attributed_to_global() -> Result<()> {
    init_test_logging();
    info!("Testing global-only field is attributed to ConfigurationSource::Global");

    let config = TestConfig::from_env()?;
    let (manager, _provider) = create_settings_manager(&config).await?;

    let context = ConfigurationContext::new(&config.test_org, "template-test-basic");
    let merged = manager.resolve_configuration(&context).await?;

    // At least one field in the trace must come from the Global source.
    let has_global = merged.source_trace.configured_fields().iter().any(|f| {
        matches!(
            merged.source_trace.get_source(f),
            Some(ConfigurationSource::Global)
        )
    });

    assert!(
        has_global,
        "Expected at least one field attributed to ConfigurationSource::Global; \
         check that global/defaults.toml is present in .reporoller-test"
    );

    Ok(())
}

/// Test that `resolve_configuration` with a non-existent template returns
/// `ConfigurationError::TemplateNotFound`.
///
/// This corresponds to the HTTP 404 response from the preview handler.
/// Previously, `OrganizationSettingsManager` silently swallowed this error;
/// it now propagates correctly.
#[tokio::test]
async fn test_preview_nonexistent_template_returns_template_not_found() -> Result<()> {
    init_test_logging();
    info!("Testing non-existent template produces TemplateNotFound");

    let config = TestConfig::from_env()?;
    let (manager, _provider) = create_settings_manager(&config).await?;

    let context = ConfigurationContext::new(&config.test_org, "template-does-not-exist-xyz-99999");
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_err(),
        "resolve_configuration must fail for a non-existent template"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConfigurationError::TemplateNotFound { .. }),
        "Expected TemplateNotFound, got: {:?}",
        err
    );

    if let ConfigurationError::TemplateNotFound { org, template } = err {
        assert_eq!(org, config.test_org, "Error must include the organisation");
        assert_eq!(
            template, "template-does-not-exist-xyz-99999",
            "Error must include the template name"
        );
        info!("✓ TemplateNotFound returned with correct org/template: {org}/{template}");
    }

    Ok(())
}

/// Test that `load_repository_type_configuration` returns `Ok(None)` for an
/// unknown repository type.
///
/// The preview handler calls this before resolving the merged configuration
/// and returns HTTP 400 when the result is `None`.  This test verifies the
/// underlying metadata provider behaves correctly so the handler pre-check
/// works as expected.
#[tokio::test]
async fn test_preview_unknown_repository_type_returns_none_from_provider() -> Result<()> {
    init_test_logging();
    info!("Testing unknown repository type returns Ok(None) from metadata provider");

    let config = TestConfig::from_env()?;
    let (_manager, provider) = create_settings_manager(&config).await?;

    // Discover the metadata repository first (required by the provider API)
    let meta = provider
        .discover_metadata_repository(&config.test_org)
        .await?;

    let result: Option<RepositoryTypeConfig> = provider
        .load_repository_type_configuration(&meta, "type-that-does-not-exist-xyz-99999")
        .await?;

    assert!(
        result.is_none(),
        "Provider must return Ok(None) for an unknown repository type; \
         the preview handler maps this to HTTP 400"
    );

    info!("✓ Unknown repository type correctly returns Ok(None)");

    Ok(())
}
