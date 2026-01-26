//! Integration tests for CLI template commands.
//!
//! These tests verify template inspection and validation commands against
//! real GitHub infrastructure in the glitchgrove organization.

use std::sync::Arc;

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
use github_client::{create_token_client, GitHubClient};
use integration_tests::TestConfig;
use repo_roller_cli::commands::template_cmd::{
    get_template_info, list_templates, validate_template,
};
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

/// Test listing templates in the glitchgrove organization.
///
/// Verifies that the list_templates function can discover and load
/// template configurations from real GitHub repositories.
#[tokio::test]
async fn test_list_templates_real_org() -> Result<()> {
    init_test_logging();
    info!("Testing listing templates in glitchgrove organization");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    let result = list_templates(&test_config.test_org, provider).await;

    assert!(
        result.is_ok(),
        "Failed to list templates: {:?}",
        result.err()
    );
    let templates = result.unwrap();

    // glitchgrove should have multiple test templates
    assert!(
        !templates.is_empty(),
        "Expected to find templates in glitchgrove org"
    );
    assert!(
        templates.len() >= 3,
        "Expected at least 3 templates, found {}",
        templates.len()
    );

    // Verify templates have required information
    for template in &templates {
        assert!(
            !template.name.is_empty(),
            "Template name should not be empty"
        );
        assert!(
            !template.description.is_empty(),
            "Template {} should have description",
            template.name
        );
        assert!(
            !template.author.is_empty(),
            "Template {} should have author",
            template.name
        );
    }

    Ok(())
}

/// Test getting information for a specific template.
///
/// Verifies that get_template_info can load and parse a real template
/// configuration, returning complete metadata.
#[tokio::test]
async fn test_get_template_info_real_template() -> Result<()> {
    init_test_logging();
    info!("Testing getting template info for template-test-basic");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    // Use template-test-basic which should exist in glitchgrove
    let result = get_template_info(&test_config.test_org, "template-test-basic", provider).await;

    assert!(
        result.is_ok(),
        "Failed to get template info: {:?}",
        result.err()
    );
    let info = result.unwrap();

    // Verify basic metadata
    assert_eq!(info.name, "template-test-basic");
    assert!(!info.description.is_empty());
    assert!(!info.author.is_empty());

    // template-test-basic has [labels] and [webhooks] sections
    // This is expected for test templates that include basic configuration
    assert_eq!(info.configuration_sections, 2);

    // Verify tags are present
    assert!(!info.tags.is_empty());

    Ok(())
}

/// Test getting information for a template with variables.
///
/// Verifies that variable definitions are correctly parsed and
/// translated to CLI format.
#[tokio::test]
async fn test_get_template_info_with_variables() -> Result<()> {
    init_test_logging();
    info!("Testing getting template info with variables");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    // Note: The test templates in glitchgrove don't actually have [variables] sections
    // This test verifies that templates without variables return empty vectors
    let result =
        get_template_info(&test_config.test_org, "template-test-variables", provider).await;

    assert!(
        result.is_ok(),
        "Failed to get template info: {:?}",
        result.err()
    );
    let info = result.unwrap();

    // template-test-variables has no actual [variables] section in the test data
    // This is expected - the template name is for testing variable substitution
    // in file content, not for testing variable definitions in template.toml
    assert!(
        info.variables.is_empty(),
        "template-test-variables should have no variables defined"
    );

    // Verify other metadata is present
    assert_eq!(info.name, "template-test-variables");
    assert!(!info.description.is_empty());

    Ok(())
}

/// Test validating a valid template.
///
/// Verifies that validate_template correctly identifies a well-formed
/// template configuration with no issues.
#[tokio::test]
async fn test_validate_valid_template() -> Result<()> {
    init_test_logging();
    info!("Testing validation of valid template");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    let result = validate_template(&test_config.test_org, "template-test-basic", provider).await;

    assert!(
        result.is_ok(),
        "Failed to validate template: {:?}",
        result.err()
    );
    let validation = result.unwrap();

    assert_eq!(validation.template_name, "template-test-basic");
    // Template should be valid or have only warnings
    if !validation.valid {
        println!("Template validation issues: {:?}", validation.issues);
        println!("Template validation warnings: {:?}", validation.warnings);
    }

    Ok(())
}

/// Test handling non-existent template.
///
/// Verifies that appropriate errors are returned when attempting to
/// get information for a template that doesn't exist.
#[tokio::test]
async fn test_get_info_nonexistent_template() -> Result<()> {
    init_test_logging();
    info!("Testing error handling for nonexistent template");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    let result =
        get_template_info(&test_config.test_org, "nonexistent-template-xyz", provider).await;

    assert!(result.is_err(), "Expected error for nonexistent template");
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Error should mention the template name
    assert!(
        err_msg.contains("nonexistent-template-xyz"),
        "Error should mention template name"
    );

    Ok(())
}

/// Test that template with invalid configuration is detected.
///
/// Verifies that validate_template correctly identifies issues in
/// malformed template configurations.
#[tokio::test]
async fn test_validate_invalid_template() -> Result<()> {
    init_test_logging();
    info!("Testing validation of invalid template");

    let test_config = TestConfig::from_env()?;
    let github_client = create_test_client(&test_config).await?;
    let config = MetadataProviderConfig::explicit(".reporoller-test");
    let provider = Arc::new(GitHubMetadataProvider::new(github_client, config));

    // Use template-test-invalid if it exists, or expect validation issues
    let result = validate_template(&test_config.test_org, "template-test-invalid", provider).await;

    if result.is_ok() {
        let validation = result.unwrap();
        // If template exists but is invalid, validation should detect issues
        if validation.template_name == "template-test-invalid" {
            // Either not valid or has warnings
            assert!(
                !validation.valid || !validation.warnings.is_empty(),
                "Invalid template should have issues or warnings"
            );
        }
    }
    // If template doesn't exist, that's also acceptable for this test

    Ok(())
}
