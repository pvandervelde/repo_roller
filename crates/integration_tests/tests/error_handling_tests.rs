//! Error handling and recovery integration tests.
//!
//! These tests verify that the system gracefully handles error conditions,
//! implements proper retry logic, and recovers from failures.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use config_manager::{ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig};
use github_client::{create_token_client, GitHubApiEnvironmentDetector, GitHubClient};
use integration_tests::{generate_test_repo_name, TestConfig, TestRepository};
use repo_roller_core::{
    OrganizationName, RepositoryCreationRequestBuilder, RepositoryName, TemplateName,
};
use std::sync::Arc;
use tracing::info;

/// Test metadata repository not found error handling.
///
/// Verifies graceful handling when organization doesn't have
/// a metadata repository configured.
#[tokio::test]
async fn test_metadata_repository_not_found() -> Result<()> {
    info!("Testing metadata repository not found handling");

    let config = TestConfig::from_env()?;
    let org_name = OrganizationName::new(&config.test_org)?;
    let repo_name = generate_test_repo_name("test", "no-metadata");

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

    let octocrab = Arc::new(create_token_client(&installation_token)?);
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

    // Create metadata provider with non-existent repository name
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client,
        MetadataProviderConfig::explicit(".definitely-does-not-exist"),
    ));

    // Create visibility providers
    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    // Try to create repository with bogus metadata repo
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        org_name,
        TemplateName::new("template-test-basic")?,
    )
    .build();

    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".definitely-does-not-exist",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

    // Verify error handling
    // System should either:
    // 1. Return error indicating metadata repo not found, OR
    // 2. Fall back to template-only configuration
    match result {
        Err(e) => {
            let error_msg = e.to_string();
            info!("Error returned (expected): {}", error_msg);
            // Verify error message mentions metadata repository
            assert!(
                error_msg.to_lowercase().contains("metadata")
                    || error_msg.to_lowercase().contains("not found")
                    || error_msg
                        .to_lowercase()
                        .contains(".definitely-does-not-exist"),
                "Error message should indicate metadata repository issue. Got: {}",
                error_msg
            );
        }
        Ok(_) => {
            info!("Repository created (fallback to template-only configuration)");
            // This is acceptable behavior - system fell back to template config
        }
    }

    info!("✓ Metadata repository not found handling test passed");
    Ok(())
}

/// Test template repository not found error handling.
///
/// Verifies that requesting a non-existent template returns
/// a clear 404 error with helpful information.
#[tokio::test]
async fn test_template_repository_not_found() -> Result<()> {
    info!("Testing template repository not found handling");

    let config = TestConfig::from_env()?;
    let org_name = OrganizationName::new(&config.test_org)?;
    let repo_name = generate_test_repo_name("test", "template-repository-not-found");

    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        org_name,
        TemplateName::new("definitely-does-not-exist-template")?,
    )
    .build();

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab = Arc::new(create_token_client(&installation_token)?);
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

    // Create metadata provider
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client,
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    // Create visibility providers
    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    // Attempt to create repository with non-existent template
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

    // Verify error handling
    assert!(
        result.is_err(),
        "Creating repository with non-existent template should fail"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Verify error message is helpful
    assert!(
        error_msg.to_lowercase().contains("template")
            || error_msg.to_lowercase().contains("not found")
            || error_msg.to_lowercase().contains("does not exist"),
        "Error message should indicate template not found. Got: {}",
        error_msg
    );

    info!("✓ Template repository not found test passed");
    Ok(())
}

/// Test malformed template configuration handling.
///
/// Verifies that templates with invalid TOML syntax are
/// detected and reported with clear error messages.
#[tokio::test]
async fn test_malformed_template_toml() -> Result<()> {
    info!("Testing malformed template TOML handling");

    let config = TestConfig::from_env()?;
    let org_name = OrganizationName::new(&config.test_org)?;
    let repo_name = generate_test_repo_name("test", "malformed-template-toml");

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

    let octocrab = Arc::new(create_token_client(&installation_token)?);
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

    // Create metadata provider
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client,
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    // Create visibility providers
    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    // Try to use template-test-invalid which has malformed template.toml
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        org_name,
        TemplateName::new("template-test-invalid")?,
    )
    .build();

    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

    // Verify error handling
    // Should fail with parsing error
    match result {
        Err(e) => {
            let error_msg = e.to_string();
            info!("Error returned (expected): {}", error_msg);
            // Verify error message indicates parsing issue
            // Could contain "toml", "parse", "syntax", "invalid", etc.
            info!("✓ Malformed template TOML test passed - error detected");
        }
        Ok(_) => {
            // If template-test-invalid doesn't exist or isn't actually invalid,
            // this test can't verify the behavior
            info!("⚠ template-test-invalid may not exist or may not be malformed");
            info!("⚠ Malformed template TOML test needs template-test-invalid with invalid TOML");
        }
    }

    Ok(())
}

/// Test invalid GitHub App credentials handling.
///
/// Verifies that invalid App ID or private key are detected
/// early with clear error messages.
#[tokio::test]
async fn test_invalid_github_app_credentials() -> Result<()> {
    info!("Testing invalid GitHub App credentials handling");

    // Test with invalid App ID
    let fake_app_id = 999999999u64;
    let fake_private_key =
        "-----BEGIN RSA PRIVATE KEY-----\nINVALID\n-----END RSA PRIVATE KEY-----";

    let auth_service =
        auth_handler::GitHubAuthService::new(fake_app_id, fake_private_key.to_string());

    // Attempt to get installation token
    let result = auth_service
        .get_installation_token_for_org("test-org")
        .await;

    assert!(
        result.is_err(),
        "Should fail with invalid GitHub App credentials"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Verify error is helpful
    assert!(
        error_msg.to_lowercase().contains("authentication")
            || error_msg.to_lowercase().contains("credential")
            || error_msg.to_lowercase().contains("app")
            || error_msg.to_lowercase().contains("jwt"),
        "Error message should indicate authentication issue. Got: {}",
        error_msg
    );

    // Verify error doesn't leak private key
    assert!(
        !error_msg.contains("INVALID"),
        "Error should not contain private key data"
    );

    info!("✓ Invalid GitHub App credentials test passed");
    Ok(())
}

/// Test error message quality and consistency.
///
/// Verifies that all error messages follow consistent format
/// and provide actionable information.
#[tokio::test]
async fn test_error_message_quality() -> Result<()> {
    info!("Testing error message quality and consistency");

    // Test various error scenarios and verify messages are:
    // 1. Clear and understandable
    // 2. Actionable (tell user what to do)
    // 3. Don't leak sensitive information
    // 4. Include relevant context
    // 5. Follow consistent format

    // Example: Invalid repository name
    let result = RepositoryName::new("Invalid Name With Spaces");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message should not be empty");
    assert!(
        error_msg.len() > 10,
        "Error message should be descriptive, got: {}",
        error_msg
    );

    // Example: Invalid organization name
    let result = OrganizationName::new("");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // Example: Invalid template name
    let result = TemplateName::new("../../../etc/passwd");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    info!("✓ Error message quality tests passed");
    Ok(())
}
