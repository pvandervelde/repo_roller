//! Integration test runner for RepoRoller.
//!
//! This module implements comprehensive end-to-end testing of the RepoRoller
//! functionality, including repository creation, template processing, variable
//! substitution, and error handling scenarios.
//!
//! This test runner uses the MetadataRepositoryProvider system and runs against
//! actual GitHub template repositories. For containerized E2E tests that test
//! the REST API layer, see e2e_containerized.rs.

use anyhow::{Context, Result};
use auth_handler::UserAuthenticationService;
use github_client::{create_app_client, GitHubClient};
use repo_roller_core::{
    create_repository, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
};
use tracing::{debug, error, info, warn};

use crate::utils::{generate_test_repo_name, RepositoryCleanup, TestConfig, TestRepository};

/// Test scenario definitions matching the integration testing strategy.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants only used in organization_settings_scenarios tests
pub enum TestScenario {
    /// Basic repository creation with minimal template
    BasicCreation,
    /// Variable substitution in templates
    VariableSubstitution,
    /// File filtering based on patterns
    FileFiltering,
    /// Error handling for invalid inputs
    ErrorHandling,
    /// Organization settings integration with metadata repository
    OrganizationSettings,
    /// Team-specific configuration overrides
    TeamConfiguration,
    /// Repository type configuration and custom properties
    RepositoryType,
    /// Configuration hierarchy merging (Template > Team > Type > Global)
    ConfigurationHierarchy,
}

impl TestScenario {
    /// Get the test name for repository naming
    #[allow(dead_code)] // Used in organization_settings_scenarios tests
    pub fn test_name(&self) -> &'static str {
        match self {
            TestScenario::BasicCreation => "basic",
            TestScenario::VariableSubstitution => "variables",
            TestScenario::FileFiltering => "filtering",
            TestScenario::ErrorHandling => "error-handling",
            TestScenario::OrganizationSettings => "org-settings",
            TestScenario::TeamConfiguration => "team-config",
            TestScenario::RepositoryType => "repo-type",
            TestScenario::ConfigurationHierarchy => "config-hierarchy",
        }
    }

    /// Get the template name for this test scenario
    pub fn template_name(&self) -> &'static str {
        match self {
            TestScenario::BasicCreation => "template-test-basic",
            TestScenario::VariableSubstitution => "template-test-variables",
            TestScenario::FileFiltering => "template-test-filtering",
            TestScenario::ErrorHandling => "test-nonexistent",
            // Organization settings scenarios use basic template
            TestScenario::OrganizationSettings => "template-test-basic",
            TestScenario::TeamConfiguration => "template-test-basic",
            TestScenario::RepositoryType => "template-test-basic",
            TestScenario::ConfigurationHierarchy => "template-test-basic",
        }
    }

    /// Check if this scenario is expected to succeed
    pub fn should_succeed(&self) -> bool {
        !matches!(self, TestScenario::ErrorHandling)
    }

    /// Get the metadata repository name for this test scenario
    pub fn metadata_repository(&self) -> Option<&'static str> {
        match self {
            // Original scenarios don't use metadata repository
            TestScenario::BasicCreation
            | TestScenario::VariableSubstitution
            | TestScenario::FileFiltering
            | TestScenario::ErrorHandling => None,
            // Organization settings scenarios use test metadata repository
            TestScenario::OrganizationSettings
            | TestScenario::TeamConfiguration
            | TestScenario::RepositoryType
            | TestScenario::ConfigurationHierarchy => Some(".reporoller-test"),
        }
    }

    /// Get the team name for team-specific scenarios
    #[allow(dead_code)] // Used in organization_settings_scenarios tests
    pub fn team_name(&self) -> Option<&'static str> {
        match self {
            TestScenario::TeamConfiguration => Some("platform"),
            TestScenario::ConfigurationHierarchy => Some("backend"),
            _ => None,
        }
    }

    /// Get the repository type for type-specific scenarios
    #[allow(dead_code)] // Used in organization_settings_scenarios tests
    pub fn repository_type(&self) -> Option<&'static str> {
        match self {
            TestScenario::RepositoryType => Some("library"),
            TestScenario::ConfigurationHierarchy => Some("service"),
            _ => None,
        }
    }

    /// Get the expected template repository name for this scenario.
    ///
    /// Returns the repository name that should contain the template for this test scenario.
    /// Templates are loaded from actual GitHub repositories via MetadataRepositoryProvider.
    ///
    /// # Returns
    /// A string in the format "template-{template_name}"
    #[allow(dead_code)]
    pub fn template_repository(&self) -> String {
        format!("template-{}", self.template_name())
    }
}

/// Result of running a single test scenario
#[derive(Debug)]
pub struct TestResult {
    pub scenario: TestScenario,
    pub success: bool,
    pub repository: Option<TestRepository>,
    pub error: Option<String>,
    pub duration: std::time::Duration,
    pub details: TestDetails,
}

/// Detailed test execution information
#[derive(Debug, Default)]
pub struct TestDetails {
    pub request_created: bool,
    pub config_loaded: bool,
    pub repository_created: bool,
    pub validation_passed: bool,
}

/// Integration test runner that orchestrates all test scenarios
pub struct IntegrationTestRunner {
    config: TestConfig,
    github_client: GitHubClient,
    cleanup: RepositoryCleanup,
    created_repositories: Vec<TestRepository>,
}

impl IntegrationTestRunner {
    /// Create a new test runner with the provided configuration
    pub async fn new(config: TestConfig) -> Result<Self> {
        info!("Initializing integration test runner");

        // Create GitHub client with App authentication
        let octocrab_client =
            create_app_client(config.github_app_id, &config.github_app_private_key)
                .await
                .context("Failed to create GitHub App client")?;

        let github_client = GitHubClient::new(octocrab_client);

        // Create cleanup utility - we'll create a new client for cleanup
        let cleanup_client =
            create_app_client(config.github_app_id, &config.github_app_private_key)
                .await
                .context("Failed to create GitHub App client for cleanup")?;
        let cleanup =
            RepositoryCleanup::new(GitHubClient::new(cleanup_client), config.test_org.clone());

        Ok(Self {
            config,
            github_client,
            cleanup,
            created_repositories: Vec::new(),
        })
    }

    /// Run all integration test scenarios
    pub async fn run_all_tests(&mut self) -> Result<Vec<TestResult>> {
        info!("Starting integration test suite");

        // Run all integration test scenarios
        let scenarios = vec![
            TestScenario::BasicCreation,
            TestScenario::VariableSubstitution,
            TestScenario::FileFiltering,
            TestScenario::ErrorHandling,
            TestScenario::OrganizationSettings,
            TestScenario::TeamConfiguration,
            TestScenario::RepositoryType,
            TestScenario::ConfigurationHierarchy,
        ];

        let mut results = Vec::new();

        for scenario in scenarios {
            info!(scenario = ?scenario, "Running test scenario");
            let result = self.run_single_test(scenario).await;
            results.push(result);
        }

        // Log detailed summary with pass/fail lists
        let total_tests = results.len();
        let passed_tests: Vec<_> = results.iter().filter(|r| r.success).collect();
        let failed_tests: Vec<_> = results.iter().filter(|r| !r.success).collect();

        info!(
            total = total_tests,
            passed = passed_tests.len(),
            failed = failed_tests.len(),
            "Integration test suite completed"
        );

        // Log passed tests
        if !passed_tests.is_empty() {
            info!("Passed tests:");
            for result in &passed_tests {
                info!(
                    scenario = ?result.scenario,
                    duration_ms = result.duration.as_millis(),
                    "  ✓"
                );
            }
        }

        // Log failed tests
        if !failed_tests.is_empty() {
            error!("Failed tests:");
            for result in &failed_tests {
                error!(
                    scenario = ?result.scenario,
                    duration_ms = result.duration.as_millis(),
                    error = result.error.as_deref().unwrap_or("Unknown error"),
                    "  ✗"
                );
            }
        }

        Ok(results)
    }

    /// Run a single test scenario (for individual test methods)
    #[allow(dead_code)]
    pub async fn run_single_test_scenario(&mut self, scenario: TestScenario) -> TestResult {
        info!(scenario = ?scenario, "Running single test scenario");
        self.run_single_test(scenario).await
    }

    /// Run a single test scenario using the new MetadataRepositoryProvider system
    async fn run_single_test(&mut self, scenario: TestScenario) -> TestResult {
        let start_time = std::time::Instant::now();
        let mut details = TestDetails::default();
        let mut error_message = None;

        info!(scenario = ?scenario, "Starting test scenario");

        // Generate unique repository name
        let repo_name = generate_test_repo_name("test", scenario.test_name());
        let test_repo = TestRepository::new(repo_name.clone(), self.config.test_org.clone());

        let success = match self
            .execute_test_scenario(&scenario, &test_repo, &mut details)
            .await
        {
            Ok(_) => {
                if scenario.should_succeed() {
                    info!(scenario = ?scenario, repo_name = repo_name, "Test scenario completed successfully");
                    true
                } else {
                    warn!(scenario = ?scenario, repo_name = repo_name, "Test scenario succeeded but was expected to fail");
                    false
                }
            }
            Err(e) => {
                if scenario.should_succeed() {
                    error!(scenario = ?scenario, repo_name = repo_name, error = %e, "Test scenario failed");
                    error_message = Some(e.to_string());
                    false
                } else {
                    info!(scenario = ?scenario, repo_name = repo_name, "Test scenario failed as expected");
                    true
                }
            }
        };

        let duration = start_time.elapsed();

        // Track created repository for cleanup
        if details.repository_created {
            self.created_repositories.push(test_repo.clone());
        }

        TestResult {
            scenario,
            success,
            repository: if details.repository_created {
                Some(test_repo)
            } else {
                None
            },
            error: error_message,
            duration,
            details,
        }
    }

    /// Execute the actual test scenario logic using real GitHub system
    async fn execute_test_scenario(
        &self,
        scenario: &TestScenario,
        test_repo: &TestRepository,
        details: &mut TestDetails,
    ) -> Result<()> {
        // Step 1: Create repository creation request
        info!(scenario = ?scenario, "Creating repository request");

        let name = RepositoryName::new(&test_repo.name)
            .map_err(|e| anyhow::anyhow!("Invalid repository name: {}", e))?;
        let owner = OrganizationName::new(&test_repo.owner)
            .map_err(|e| anyhow::anyhow!("Invalid organization name: {}", e))?;
        let template = TemplateName::new(scenario.template_name())
            .map_err(|e| anyhow::anyhow!("Invalid template name: {}", e))?;

        // Build request with scenario-specific variables
        let mut builder = RepositoryCreationRequestBuilder::new(name, owner).template(template);

        // Add test variables based on scenario
        match scenario {
            TestScenario::VariableSubstitution => {
                // Variables matching template-test-variables template files
                // (README.md, Cargo.toml, config.yml, src/main.rs)
                builder = builder
                    .variable("project_name", "test-project")
                    .variable("version", "0.1.0")
                    .variable("author_name", "Integration Test")
                    .variable("author_email", "test@example.com")
                    .variable(
                        "project_description",
                        "A test project for integration testing",
                    )
                    .variable("license", "MIT")
                    .variable("license_type", "MIT")
                    .variable("environment", "test")
                    .variable("debug_mode", "true");
            }
            TestScenario::FileFiltering => {
                builder = builder
                    .variable("include_docs", "true")
                    .variable("include_config", "true");
            }
            _ => {
                // Other scenarios don't require specific variables
            }
        }

        let request = builder.build();
        details.request_created = true;

        // Step 2: Create authentication service
        info!(scenario = ?scenario, "Setting up authentication");
        let auth_service = auth_handler::GitHubAuthService::new(
            self.config.github_app_id,
            self.config.github_app_private_key.clone(),
        );

        // Step 3: Get installation token and create GitHub client
        let installation_token = auth_service
            .get_installation_token_for_org(&self.config.test_org)
            .await
            .context("Failed to get installation token")?;

        let octocrab = std::sync::Arc::new(
            github_client::create_token_client(&installation_token)
                .context("Failed to create GitHub client")?,
        );
        let github_client = github_client::GitHubClient::new(octocrab.as_ref().clone());

        // Step 4: Create metadata provider
        info!(scenario = ?scenario, "Creating metadata provider");
        let metadata_repo_name = scenario.metadata_repository().unwrap_or(".reporoller");
        let metadata_provider = std::sync::Arc::new(config_manager::GitHubMetadataProvider::new(
            github_client,
            config_manager::MetadataProviderConfig::explicit(metadata_repo_name),
        ));
        details.config_loaded = true;

        // Create visibility providers
        let visibility_policy_provider = std::sync::Arc::new(
            config_manager::ConfigBasedPolicyProvider::new(metadata_provider.clone()),
        );
        let environment_detector =
            std::sync::Arc::new(github_client::GitHubApiEnvironmentDetector::new(octocrab));

        // Create event notification providers
        let event_providers = crate::create_event_notification_providers();
        let event_context = repo_roller_core::EventNotificationContext::new(
            "test-runner",
            event_providers.secret_resolver,
            event_providers.metrics,
        );

        // Step 5: Call the repository creation function
        info!(scenario = ?scenario, repo_name = test_repo.name, "Creating repository via RepoRoller");

        let result = create_repository(
            request,
            metadata_provider.as_ref(),
            &auth_service,
            metadata_repo_name,
            visibility_policy_provider,
            environment_detector,
            event_context,
        )
        .await;

        // Step 6: Evaluate the result
        match result {
            Ok(creation_result) => {
                info!(
                    scenario = ?scenario,
                    repo_name = test_repo.name,
                    repo_url = creation_result.repository_url,
                    "Repository creation succeeded"
                );
                details.repository_created = true;

                // Step 7: Validate the created repository
                self.validate_github_repository(test_repo)
                    .await
                    .context("GitHub repository validation failed")?;
                details.validation_passed = true;

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Repository creation failed: {}", e);
                error!(scenario = ?scenario, repo_name = test_repo.name, error = %e, "Repository creation failed");
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Validate that the repository was created correctly on GitHub
    async fn validate_github_repository(&self, test_repo: &TestRepository) -> Result<()> {
        debug!(repo_name = test_repo.name, "Validating GitHub repository");

        // Get installation token for validation
        let installation_token = self
            .github_client
            .get_installation_token_for_org(&test_repo.owner)
            .await
            .context("Failed to get installation token for validation")?;

        let installation_client = github_client::create_token_client(&installation_token)
            .context("Failed to create installation token client")?;

        // Check that repository exists
        let repo_result = installation_client
            .repos(&test_repo.owner, &test_repo.name)
            .get()
            .await;

        match repo_result {
            Ok(repo) => {
                debug!(
                    repo_name = test_repo.name,
                    private = repo.private,
                    "Repository validation successful"
                );

                if !repo.private.unwrap_or(false) {
                    warn!(
                        repo_name = test_repo.name,
                        "Repository should be private but is public"
                    );
                }

                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Repository validation failed: {}", e)),
        }
    }

    /// Clean up all created test repositories
    pub async fn cleanup_test_repositories(&self) -> Result<()> {
        info!(
            count = self.created_repositories.len(),
            "Starting cleanup of test repositories"
        );

        for repo in &self.created_repositories {
            if let Err(e) = self.cleanup.delete_repository(&repo.name).await {
                warn!(
                    repo_name = repo.name,
                    error = %e,
                    "Failed to cleanup test repository"
                );
            }
        }

        info!("Test repository cleanup completed");
        Ok(())
    }

    /// Clean up orphaned test repositories older than specified hours
    pub async fn cleanup_orphaned_repositories(&self, max_age_hours: u64) -> Result<Vec<String>> {
        self.cleanup
            .cleanup_orphaned_repositories(max_age_hours)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_properties() {
        let basic = TestScenario::BasicCreation;
        assert_eq!(basic.test_name(), "basic");
        assert!(basic.should_succeed());
        assert_eq!(basic.template_name(), "template-test-basic");

        let error = TestScenario::ErrorHandling;
        assert_eq!(error.test_name(), "error-handling");
        assert!(!error.should_succeed());
        assert_eq!(error.template_name(), "test-nonexistent");
    }

    #[test]
    #[cfg(any())] // Disabled until migration to new TemplateConfig
    fn test_mock_template_creation() {
        let scenario = TestScenario::VariableSubstitution;
        let template = scenario.create_mock_template("glitchgrove");
        //assert_eq!(template.name, "test-variables");
        //assert!(template.source_repo.contains("template-test-variables"));
    }
}
