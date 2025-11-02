//! Integration test runner for RepoRoller.
//!
//! This binary runs comprehensive end-to-end tests of the RepoRoller functionality,
//! including repository creation, template processing, variable substitution, and
//! error handling scenarios.
//!
//! ## Usage
//!
//! ```bash
//! # Run all integration tests
//! cargo run --bin integration_tests
//!
//! # Run with cleanup of orphaned repositories
//! cargo run --bin integration_tests -- --cleanup-orphans
//!
//! # Run with custom orphan cleanup age (default: 24 hours)
//! cargo run --bin integration_tests -- --cleanup-orphans --max-age-hours 48
//! ```
//!
//! ## Environment Variables
//!
//! Required environment variables (see .github/workflows/INTEGRATION_TESTS_SECRETS.md):
//! - `GITHUB_APP_ID`: GitHub App ID for authentication
//! - `GITHUB_APP_PRIVATE_KEY`: GitHub App private key in PEM format
//! - `TEST_ORG`: Organization name where test repositories will be created

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::process;
use tracing::{error, info, warn};

mod test_runner;
mod utils;

use test_runner::{IntegrationTestRunner, TestScenario};
use utils::{init_logging, validate_test_environment, TestConfig};

#[tokio::main]
async fn main() {
    // Initialize logging first
    init_logging();

    // Parse command line arguments
    let matches = Command::new("integration_tests")
        .version("1.0")
        .author("RepoRoller Team")
        .about("Integration tests for RepoRoller functionality")
        .arg(
            Arg::new("cleanup-orphans")
                .long("cleanup-orphans")
                .help("Clean up orphaned test repositories before running tests")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-age-hours")
                .long("max-age-hours")
                .help("Maximum age in hours for orphaned repositories (default: 24)")
                .value_name("HOURS")
                .default_value("24"),
        )
        .arg(
            Arg::new("cleanup-only")
                .long("cleanup-only")
                .help("Only perform cleanup, don't run tests")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Run the integration tests
    if let Err(e) = run_integration_tests(&matches).await {
        error!(error = %e, "Integration tests failed");
        process::exit(1);
    }
}

async fn run_integration_tests(matches: &clap::ArgMatches) -> Result<()> {
    info!("Starting RepoRoller integration tests");

    // Validate environment
    validate_test_environment().context("Environment validation failed")?;

    // Load configuration
    let config =
        TestConfig::from_env().context("Failed to load test configuration from environment")?;

    info!(
        github_app_id = config.github_app_id,
        test_org = config.test_org,
        "Loaded test configuration"
    );

    // Create test runner
    let mut test_runner = IntegrationTestRunner::new(config)
        .await
        .context("Failed to initialize test runner")?;

    // Handle cleanup of orphaned repositories
    let cleanup_orphans = matches.get_flag("cleanup-orphans");
    let cleanup_only = matches.get_flag("cleanup-only");
    let max_age_hours: u64 = matches
        .get_one::<String>("max-age-hours")
        .unwrap()
        .parse()
        .context("Invalid max-age-hours value")?;

    if cleanup_orphans || cleanup_only {
        info!(
            max_age_hours = max_age_hours,
            "Starting orphaned repository cleanup"
        );

        match test_runner
            .cleanup_orphaned_repositories(max_age_hours)
            .await
        {
            Ok(deleted_repos) => {
                if deleted_repos.is_empty() {
                    info!("No orphaned repositories found for cleanup");
                } else {
                    info!(
                        count = deleted_repos.len(),
                        repositories = ?deleted_repos,
                        "Successfully cleaned up orphaned repositories"
                    );
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to cleanup orphaned repositories");
                // Don't fail the entire test run for cleanup failures
            }
        }
    }

    // If cleanup-only flag is set, exit after cleanup
    if cleanup_only {
        info!("Cleanup completed, exiting as requested");
        return Ok(());
    }

    // Run the integration test suite
    info!("Starting integration test execution");

    let test_results = test_runner
        .run_all_tests()
        .await
        .context("Failed to run integration tests")?;

    // Process and report results
    let mut exit_code = 0;
    let total_tests = test_results.len();
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    info!("=== Integration Test Results ===");

    for result in &test_results {
        let status = if result.success { "PASS" } else { "FAIL" };
        let duration_ms = result.duration.as_millis();

        info!(
            scenario = ?result.scenario,
            status = status,
            duration_ms = duration_ms,
            repository = result.repository.as_ref().map(|r| &r.name),
            "Test result"
        );

        if result.success {
            passed_tests += 1;
        } else {
            failed_tests += 1;
            exit_code = 1;

            if let Some(error) = &result.error {
                error!(
                    scenario = ?result.scenario,
                    error = error,
                    "Test failure details"
                );
            }
        }

        // Log detailed execution information
        info!(
            scenario = ?result.scenario,
            request_created = result.details.request_created,
            config_loaded = result.details.config_loaded,
            repository_created = result.details.repository_created,
            validation_passed = result.details.validation_passed,
            "Test execution details"
        );
    }

    // Summary
    info!(
        total = total_tests,
        passed = passed_tests,
        failed = failed_tests,
        "=== Test Suite Summary ==="
    );

    // Cleanup test repositories
    info!("Starting cleanup of test repositories created during this run");
    if let Err(e) = test_runner.cleanup_test_repositories().await {
        warn!(error = %e, "Failed to cleanup some test repositories");
        // Don't fail the test run for cleanup failures
    }

    // Generate detailed report
    generate_test_report(&test_results)?;

    if exit_code != 0 {
        error!(
            "Integration test suite failed with {} failed tests",
            failed_tests
        );
        process::exit(exit_code);
    }

    info!("All integration tests passed successfully");
    Ok(())
}

/// Generate a detailed test report for CI/CD systems
fn generate_test_report(results: &[test_runner::TestResult]) -> Result<()> {
    use std::io::Write;

    // Create a simple text report
    let mut report = Vec::new();

    writeln!(report, "# RepoRoller Integration Test Report")?;
    writeln!(report)?;
    writeln!(
        report,
        "Generated: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(report)?;

    // Summary table
    writeln!(report, "## Summary")?;
    writeln!(report)?;
    writeln!(report, "| Metric | Value |")?;
    writeln!(report, "|--------|-------|")?;
    writeln!(report, "| Total Tests | {} |", results.len())?;
    writeln!(
        report,
        "| Passed | {} |",
        results.iter().filter(|r| r.success).count()
    )?;
    writeln!(
        report,
        "| Failed | {} |",
        results.iter().filter(|r| !r.success).count()
    )?;
    writeln!(
        report,
        "| Total Duration | {:.2}s |",
        results
            .iter()
            .map(|r| r.duration.as_secs_f64())
            .sum::<f64>()
    )?;
    writeln!(report)?;

    // Detailed results
    writeln!(report, "## Test Results")?;
    writeln!(report)?;

    for result in results {
        let status_emoji = if result.success { "✅" } else { "❌" };
        let scenario_name = match result.scenario {
            TestScenario::BasicCreation => "Basic Repository Creation",
            TestScenario::VariableSubstitution => "Variable Substitution",
            TestScenario::FileFiltering => "File Filtering",
            TestScenario::ErrorHandling => "Error Handling",
            TestScenario::OrganizationSettings => "Organization Settings Integration",
            TestScenario::TeamConfiguration => "Team Configuration Overrides",
            TestScenario::RepositoryType => "Repository Type Configuration",
            TestScenario::ConfigurationHierarchy => "Configuration Hierarchy Merging",
        };

        writeln!(report, "### {} {}", status_emoji, scenario_name)?;
        writeln!(report)?;
        writeln!(
            report,
            "- **Status**: {}",
            if result.success { "PASSED" } else { "FAILED" }
        )?;
        writeln!(
            report,
            "- **Duration**: {:.2}s",
            result.duration.as_secs_f64()
        )?;

        if let Some(repo) = &result.repository {
            writeln!(report, "- **Repository**: {}", repo.full_name)?;
        }

        if let Some(error) = &result.error {
            writeln!(report, "- **Error**: {}", error)?;
        }

        writeln!(
            report,
            "- **Request Created**: {}",
            if result.details.request_created {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(
            report,
            "- **Config Loaded**: {}",
            if result.details.config_loaded {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(
            report,
            "- **Repository Created**: {}",
            if result.details.repository_created {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(
            report,
            "- **Validation Passed**: {}",
            if result.details.validation_passed {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(report)?;
    }

    // Write report to file
    std::fs::write("integration-test-report.md", report).context("Failed to write test report")?;

    info!("Test report written to integration-test-report.md");
    Ok(())
}
