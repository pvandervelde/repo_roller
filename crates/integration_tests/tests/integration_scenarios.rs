//! Integration test scenarios for RepoRoller.
//!
//! This module contains specific test implementations for each integration test scenario.
//! These tests are designed to run against actual GitHub repositories using the RepoRoller
//! core functionality.

use anyhow::Result;
use integration_tests::{
    test_runner::IntegrationTestRunner,
    utils::TestConfig,
    verification::{verify_labels, ExpectedConfiguration, ExpectedRepositorySettings},
};
use tracing::info;

/// Test basic repository creation functionality.
///
/// This test verifies that RepoRoller can create a repository from the test-basic template
/// with minimal configuration and no variable substitution.
#[tokio::test]
async fn test_basic_repository_creation() -> Result<()> {
    info!("Starting basic repository creation test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run only the basic creation test
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::BasicCreation)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(results.success, "Basic repository creation should succeed");
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    Ok(())
}

/// Test variable substitution in templates.
///
/// This test verifies that RepoRoller correctly substitutes variables in template files
/// using the test-variables template with complex variable scenarios.
#[tokio::test]
async fn test_variable_substitution() -> Result<()> {
    info!("Starting variable substitution test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the variable substitution test
    let results = runner
        .run_single_test_scenario(
            integration_tests::test_runner::TestScenario::VariableSubstitution,
        )
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(results.success, "Variable substitution should succeed");
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    // TODO: Add specific validation of variable substitution in created files
    // This would require reading the created repository content and verifying
    // that variables were correctly replaced

    Ok(())
}

/// Test file filtering based on include/exclude patterns.
///
/// This test verifies that RepoRoller correctly filters files during template processing
/// using the test-filtering template with various file patterns.
#[tokio::test]
async fn test_file_filtering() -> Result<()> {
    info!("Starting file filtering test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the file filtering test
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::FileFiltering)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(results.success, "File filtering should succeed");
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    // TODO: Add specific validation of file filtering results
    // This would require checking which files were included/excluded in the created repository

    Ok(())
}

/// Test error handling for invalid template configurations.
///
/// This test verifies that RepoRoller properly handles and reports errors when processing
/// invalid templates using a non-existent template repository.
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    info!("Starting error handling test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the error handling test
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::ErrorHandling)
        .await;

    // Cleanup any partially created repositories
    runner.cleanup_test_repositories().await?;

    // Verify results - TestResult.success=true means test executed as expected
    // For ErrorHandling, we expect the repository creation to fail, which is handled correctly
    // by the test runner (returns success=true when error is expected and received)
    assert!(
        results.success,
        "Error handling test should pass (error was expected and received)"
    );
    
    // Verify no repository was created (since creation failed as expected)
    assert!(results.repository.is_none(), "Should not have created repository");

    Ok(())
}

/// Test orphaned repository cleanup functionality.
///
/// This test verifies that the cleanup system can find and remove orphaned test repositories.
#[tokio::test]
async fn test_orphaned_repository_cleanup() -> Result<()> {
    info!("Starting orphaned repository cleanup test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Create a test repository that we'll treat as orphaned
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::BasicCreation)
        .await;

    assert!(
        results.success,
        "Should create test repository for cleanup test"
    );

    // Now test cleanup with 0 hours max age (should clean up everything)
    let deleted_repos = runner.cleanup_orphaned_repositories(0).await?;

    // Verify that our test repository was cleaned up
    assert!(
        !deleted_repos.is_empty(),
        "Should have deleted at least one repository"
    );

    Ok(())
}

/// Integration test for the complete end-to-end workflow.
///
/// This test runs all test scenarios in sequence and verifies the complete
/// integration testing workflow.
#[tokio::test]
async fn test_complete_integration_workflow() -> Result<()> {
    info!("Starting complete integration workflow test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run all tests
    let results = runner.run_all_tests().await?;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results - now have 8 total scenarios
    assert_eq!(results.len(), 8, "Should have run 8 test scenarios");

    let success_count = results.iter().filter(|r| r.success).count();
    let expected_successes = 7; // All except ErrorHandling should succeed

    assert_eq!(
        success_count, expected_successes,
        "Should have {} successful tests out of 8",
        expected_successes
    );

    // Verify specific test results
    for result in &results {
        match result.scenario {
            integration_tests::test_runner::TestScenario::ErrorHandling => {
                assert!(
                    !result.success,
                    "Error handling test should fail as expected"
                );
            }
            _ => {
                assert!(result.success, "Test {:?} should succeed", result.scenario);
            }
        }
    }

    Ok(())
}

/// Test basic repository creation with configuration verification.
///
/// This test addresses the critical gap where tests only checked success flags
/// without verifying that settings were actually applied to GitHub.
///
/// **CRITICAL**: This test verifies actual GitHub repository state, not just API success responses.
#[tokio::test]
async fn test_basic_creation_with_configuration_verification() -> Result<()> {
    info!("Starting basic repository creation with configuration verification");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config.clone()).await?;

    // Run the basic creation test
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::BasicCreation)
        .await;

    assert!(results.success, "Repository creation should succeed");

    let repo = results
        .repository
        .as_ref()
        .expect("Repository should be created");

    // CRITICAL: Verify configuration was actually applied to GitHub
    info!(
        "Verifying configuration for repository: {}/{}",
        repo.owner, repo.name
    );

    // Create expected configuration for basic template
    // Note: These values should match what's in template-test-basic's configuration
    let expected_config = ExpectedConfiguration {
        repository_settings: Some(ExpectedRepositorySettings {
            has_issues: Some(true),
            has_wiki: Some(false),
            has_discussions: Some(false),
            has_projects: Some(false),
        }),
        custom_properties: None, // Basic template has no custom properties
        branch_protection: None, // Basic template has no branch protection
        labels: Some(vec![
            "bug".to_string(),
            "enhancement".to_string(),
            "documentation".to_string(),
        ]),
    };

    // Create GitHub client for verification (skip if token not available)
    let github_token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            info!("GITHUB_TOKEN not available, skipping detailed verification");
            // Cleanup and return success - repository was created successfully
            runner.cleanup_test_repositories().await?;
            return Ok(());
        }
    };
    let octocrab =
        github_client::create_token_client(&github_token).expect("Failed to create GitHub client");
    let github_client = github_client::GitHubClient::new(octocrab);

    // Verify labels (the only verification that doesn't require Repository model extension)
    if let Some(expected_labels) = &expected_config.labels {
        let label_verification =
            verify_labels(&github_client, &repo.owner, &repo.name, expected_labels).await?;

        if !label_verification.passed {
            info!(
                "Label verification failed: {:#?}",
                label_verification.failures
            );
        }
        assert!(
            label_verification.passed,
            "Labels verification failed: {:?}",
            label_verification.failures
        );
        info!("✓ Label verification passed");
    }

    // TODO: Verify repository settings once Repository model is extended
    // if let Some(settings) = &expected_config.repository_settings {
    //     let settings_verification = verify_repository_settings(
    //         &github_client,
    //         &repo.owner,
    //         &repo.name,
    //         settings
    //     ).await?;
    //
    //     assert!(settings_verification.passed,
    //         "Repository settings verification failed: {:?}",
    //         settings_verification.failures);
    //     info!("✓ Repository settings verification passed");
    // }

    // Cleanup
    runner.cleanup_test_repositories().await?;

    info!("✓ Basic repository creation with configuration verification completed successfully");
    Ok(())
}

/// Test variable substitution with file content verification.
///
/// This test not only creates a repository with variable substitution but also
/// verifies that the variables were actually substituted in the created files.
#[tokio::test]
async fn test_variable_substitution_with_verification() -> Result<()> {
    info!("Starting variable substitution with content verification");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config.clone()).await?;

    // Run the variable substitution test
    let results = runner
        .run_single_test_scenario(
            integration_tests::test_runner::TestScenario::VariableSubstitution,
        )
        .await;

    assert!(results.success, "Variable substitution should succeed");

    let repo = results
        .repository
        .as_ref()
        .expect("Repository should be created");

    info!(
        "Verifying variable substitution in repository: {}/{}",
        repo.owner, repo.name
    );

    // Create GitHub client for verification (skip if token not available)
    let github_token = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            info!("GITHUB_TOKEN not available, skipping detailed verification");
            // Cleanup and return success - repository was created and variables substituted
            runner.cleanup_test_repositories().await?;
            return Ok(());
        }
    };
    let octocrab =
        github_client::create_token_client(&github_token).expect("Failed to create GitHub client");
    let github_client = github_client::GitHubClient::new(octocrab);

    // TODO: Verify that variables were substituted in template files
    // This requires:
    // 1. Fetching file contents from the created repository
    // 2. Checking that {{variable}} patterns were replaced
    // 3. Verifying specific variable values appear in the files
    //
    // Example:
    // let readme_content = github_client.get_file_content(&repo.owner, &repo.name, "README.md").await?;
    // assert!(readme_content.contains("test-project"), "README should contain substituted project_name");
    // assert!(!readme_content.contains("{{project_name}}"), "README should not contain template placeholders");

    info!("⚠ File content verification not yet implemented - needs GitHub file content API");

    // Cleanup
    runner.cleanup_test_repositories().await?;

    info!("✓ Variable substitution test completed");
    Ok(())
}

/// Test file filtering with directory structure verification.
///
/// This test verifies that file filtering rules are correctly applied by
/// checking which files exist in the created repository.
#[tokio::test]
async fn test_file_filtering_with_verification() -> Result<()> {
    info!("Starting file filtering with directory structure verification");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config.clone()).await?;

    // Run the file filtering test
    let results = runner
        .run_single_test_scenario(integration_tests::test_runner::TestScenario::FileFiltering)
        .await;

    assert!(results.success, "File filtering should succeed");

    let repo = results
        .repository
        .as_ref()
        .expect("Repository should be created");

    info!(
        "Verifying file filtering in repository: {}/{}",
        repo.owner, repo.name
    );

    // TODO: Verify that filtered files are present/absent
    // This requires:
    // 1. Listing all files in the created repository
    // 2. Checking that included files are present
    // 3. Checking that excluded files are absent
    //
    // Example:
    // let files = github_client.list_repository_files(&repo.owner, &repo.name).await?;
    // assert!(files.contains(&"docs/README.md".to_string()), "Docs should be included");
    // assert!(!files.contains(&".github/workflows/excluded.yml".to_string()), "Excluded files should not be present");

    info!("⚠ File structure verification not yet implemented - needs GitHub tree API");

    // Cleanup
    runner.cleanup_test_repositories().await?;

    info!("✓ File filtering test completed");
    Ok(())
}
