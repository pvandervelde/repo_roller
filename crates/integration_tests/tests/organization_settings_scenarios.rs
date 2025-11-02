//! Integration tests for organization settings features.
//!
//! These tests verify the organization settings system integration with repository creation,
//! including global defaults, team overrides, repository types, and configuration hierarchy.

use anyhow::Result;
use integration_tests::{test_runner::TestScenario, utils::TestConfig, IntegrationTestRunner};
use tracing::info;

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
#[ignore = "Requires GitHub App with custom properties permissions and metadata repository"]
async fn test_organization_settings_with_global_defaults() -> Result<()> {
    info!("Starting organization settings with global defaults test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the organization settings test scenario
    let results = runner
        .run_single_test_scenario(TestScenario::OrganizationSettings)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(
        results.success,
        "Organization settings test should succeed: {:?}",
        results.error
    );
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    info!("Organization settings with global defaults test completed successfully");
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
#[ignore = "Requires GitHub App with custom properties permissions and metadata repository"]
async fn test_team_configuration_overrides() -> Result<()> {
    info!("Starting team configuration overrides test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the team configuration test scenario
    let results = runner
        .run_single_test_scenario(TestScenario::TeamConfiguration)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(
        results.success,
        "Team configuration test should succeed: {:?}",
        results.error
    );
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    info!("Team configuration overrides test completed successfully");
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
#[ignore = "Requires GitHub App with custom properties permissions and metadata repository"]
async fn test_repository_type_configuration() -> Result<()> {
    info!("Starting repository type configuration test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the repository type test scenario
    let results = runner
        .run_single_test_scenario(TestScenario::RepositoryType)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(
        results.success,
        "Repository type test should succeed: {:?}",
        results.error
    );
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    info!("Repository type configuration test completed successfully");
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
#[ignore = "Requires GitHub App with custom properties permissions and metadata repository"]
async fn test_configuration_hierarchy_merging() -> Result<()> {
    info!("Starting configuration hierarchy merging test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run the configuration hierarchy test scenario
    let results = runner
        .run_single_test_scenario(TestScenario::ConfigurationHierarchy)
        .await;

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify results
    assert!(
        results.success,
        "Configuration hierarchy test should succeed: {:?}",
        results.error
    );
    assert!(
        results.details.repository_created,
        "Repository should be created"
    );
    assert!(
        results.details.validation_passed,
        "Repository validation should pass"
    );

    info!("Configuration hierarchy merging test completed successfully");
    Ok(())
}

/// Test complete integration workflow with organization settings.
///
/// This test runs all organization settings scenarios to verify
/// the complete integration of the configuration system.
#[tokio::test]
#[ignore = "Requires GitHub App with custom properties permissions and metadata repository"]
async fn test_complete_organization_settings_workflow() -> Result<()> {
    info!("Starting complete organization settings workflow test");

    let config = TestConfig::from_env()?;
    let mut runner = IntegrationTestRunner::new(config).await?;

    // Run all organization settings test scenarios
    let scenarios = vec![
        TestScenario::OrganizationSettings,
        TestScenario::TeamConfiguration,
        TestScenario::RepositoryType,
        TestScenario::ConfigurationHierarchy,
    ];

    let mut all_results = Vec::new();
    for scenario in scenarios {
        info!(?scenario, "Running organization settings scenario");
        let result = runner.run_single_test_scenario(scenario).await;
        all_results.push(result);
    }

    // Cleanup
    runner.cleanup_test_repositories().await?;

    // Verify all scenarios passed
    let all_passed = all_results.iter().all(|r| r.success);
    assert!(
        all_passed,
        "All organization settings scenarios should pass"
    );

    let total = all_results.len();
    let passed = all_results.iter().filter(|r| r.success).count();

    info!(
        total = total,
        passed = passed,
        "Complete organization settings workflow test completed"
    );

    Ok(())
}
