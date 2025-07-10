//! Integration test scenarios for RepoRoller.
//!
//! This module contains specific test implementations for each integration test scenario.
//! These tests are designed to run against actual GitHub repositories using the RepoRoller
//! core functionality.

use anyhow::Result;
use integration_tests::{test_runner::IntegrationTestRunner, utils::TestConfig};
use std::collections::HashMap;
use tokio;
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
/// invalid templates using the test-invalid template.
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

    // Verify results - this test should fail as expected
    assert!(
        !results.success,
        "Error handling test should fail as expected"
    );
    assert!(results.error.is_some(), "Should have error message");

    // Verify that we got a meaningful error message
    let error_msg = results.error.unwrap();
    assert!(!error_msg.is_empty(), "Error message should not be empty");

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

    // Verify results
    assert_eq!(results.len(), 4, "Should have run 4 test scenarios");

    let success_count = results.iter().filter(|r| r.success).count();
    let expected_successes = 3; // All except ErrorHandling should succeed

    assert_eq!(
        success_count, expected_successes,
        "Should have {} successful tests out of 4",
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
