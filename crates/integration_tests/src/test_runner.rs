//! Integration test runner for RepoRoller.
//!
//! This module implements comprehensive end-to-end testing of the RepoRoller
//! functionality, including repository creation, template processing, variable
//! substitution, and error handling scenarios.

use anyhow::{Context, Result};
use config_manager::{Config, TemplateConfig};
use github_client::{create_app_client, GitHubClient};
use repo_roller_core::{
    create_repository, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
};
use std::collections::HashMap;
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
            TestScenario::BasicCreation => "test-basic",
            TestScenario::VariableSubstitution => "test-variables",
            TestScenario::FileFiltering => "test-filtering",
            TestScenario::ErrorHandling => "test-nonexistent",
            // Organization settings scenarios use basic template
            TestScenario::OrganizationSettings => "test-basic",
            TestScenario::TeamConfiguration => "test-basic",
            TestScenario::RepositoryType => "test-basic",
            TestScenario::ConfigurationHierarchy => "test-basic",
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

    /// Create a mock template configuration for testing
    pub fn create_mock_template(&self, org: &str) -> TemplateConfig {
        let description = match self {
            TestScenario::ErrorHandling => {
                Some("Non-existent template for testing error handling".to_string())
            }
            _ => Some(format!("Test template for {}", self.test_name())),
        };

        // Create variable configurations with defaults for VariableSubstitution scenario
        let variable_configs = if matches!(self, TestScenario::VariableSubstitution) {
            let mut configs = HashMap::new();

            configs.insert(
                "project_name".to_string(),
                config_manager::VariableConfig {
                    description: "Name of the project".to_string(),
                    example: Some("my-awesome-project".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("test-project".to_string()),
                },
            );

            configs.insert(
                "project_description".to_string(),
                config_manager::VariableConfig {
                    description: "Description of the project".to_string(),
                    example: Some("A simple test project".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("Integration test project for RepoRoller".to_string()),
                },
            );

            configs.insert(
                "author_name".to_string(),
                config_manager::VariableConfig {
                    description: "Author's name".to_string(),
                    example: Some("John Doe".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("RepoRoller Test".to_string()),
                },
            );

            configs.insert(
                "author_email".to_string(),
                config_manager::VariableConfig {
                    description: "Author's email".to_string(),
                    example: Some("john@example.com".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("test@example.com".to_string()),
                },
            );

            configs.insert(
                "version".to_string(),
                config_manager::VariableConfig {
                    description: "Project version".to_string(),
                    example: Some("1.0.0".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("0.1.0".to_string()),
                },
            );

            configs.insert(
                "license".to_string(),
                config_manager::VariableConfig {
                    description: "License type".to_string(),
                    example: Some("MIT".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("MIT".to_string()),
                },
            );

            configs.insert(
                "custom_ignore".to_string(),
                config_manager::VariableConfig {
                    description: "Additional gitignore patterns".to_string(),
                    example: Some("*.tmp".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("*.test".to_string()),
                },
            );

            configs.insert(
                "debug_mode".to_string(),
                config_manager::VariableConfig {
                    description: "Boolean for debug mode".to_string(),
                    example: Some("true".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("true".to_string()),
                },
            );

            configs.insert(
                "environment".to_string(),
                config_manager::VariableConfig {
                    description: "Environment name".to_string(),
                    example: Some("production".to_string()),
                    required: Some(false),
                    pattern: None,
                    min_length: None,
                    max_length: None,
                    options: None,
                    default: Some("development".to_string()),
                },
            );

            Some(configs)
        } else {
            Some(HashMap::new())
        };

        TemplateConfig {
            name: self.template_name().to_string(),
            source_repo: format!(
                "https://github.com/{}/template-{}",
                org,
                self.template_name()
            ),
            description,
            topics: Some(vec!["test".to_string(), "repo-roller".to_string()]),
            features: None,
            pr_settings: None,
            labels: None,
            branch_protection_rules: None,
            action_permissions: None,
            required_variables: None,
            variable_configs,
        }
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
    /// Whether configuration verification was performed
    pub configuration_verified: bool,
    /// Whether repository settings matched expected configuration
    pub settings_match: bool,
    /// Whether custom properties matched expected configuration
    pub custom_properties_match: bool,
    /// Whether branch protection matched expected configuration
    pub branch_protection_match: bool,
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

        let scenarios = vec![
            TestScenario::BasicCreation,
            TestScenario::VariableSubstitution,
            TestScenario::FileFiltering,
            TestScenario::ErrorHandling,
        ];

        let mut results = Vec::new();

        for scenario in scenarios {
            info!(scenario = ?scenario, "Running test scenario");
            let result = self.run_single_test(scenario).await;
            results.push(result);
        }

        // Log summary
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - passed_tests;

        info!(
            total = total_tests,
            passed = passed_tests,
            failed = failed_tests,
            "Integration test suite completed"
        );

        Ok(results)
    }

    /// Run a single test scenario (for individual test methods)
    #[allow(dead_code)]
    pub async fn run_single_test_scenario(&mut self, scenario: TestScenario) -> TestResult {
        info!(scenario = ?scenario, "Running single test scenario");
        self.run_single_test(scenario).await
    }

    /// Run a single test scenario
    async fn run_single_test(&mut self, scenario: TestScenario) -> TestResult {
        let start_time = std::time::Instant::now();
        let mut details = TestDetails::default();
        let mut error_message = None;

        info!(scenario = ?scenario, "Starting test scenario");

        // Generate unique repository name
        let repo_name = generate_test_repo_name(scenario.test_name());
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

    /// Execute the actual test scenario logic
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

        let request = RepositoryCreationRequestBuilder::new(name, owner, template).build();
        details.request_created = true;

        // Step 2: Create mock configuration with test template
        info!(scenario = ?scenario, "Creating test configuration");

        let template_config = scenario.create_mock_template(&self.config.test_org);
        let config = Config {
            templates: vec![template_config],
        };
        details.config_loaded = true;

        // Step 3: Call the repository creation function
        info!(scenario = ?scenario, repo_name = test_repo.name, "Creating repository via RepoRoller");

        // Get metadata repository name from scenario
        let metadata_repo_name = scenario.metadata_repository().unwrap_or(".reporoller");

        // Create authentication service
        let auth_service = auth_handler::GitHubAuthService::new(
            self.config.github_app_id,
            self.config.github_app_private_key.clone(),
        );

        let result = create_repository(request, &config, &auth_service, metadata_repo_name).await;

        // Step 4: Evaluate the result
        match result {
            Ok(creation_result) => {
                info!(
                    scenario = ?scenario,
                    repo_name = test_repo.name,
                    repo_url = creation_result.repository_url,
                    "Repository creation succeeded"
                );
                details.repository_created = true;

                // Step 5: Validate the created repository
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
        assert_eq!(basic.template_name(), "test-basic");

        let error = TestScenario::ErrorHandling;
        assert_eq!(error.test_name(), "error-handling");
        assert!(!error.should_succeed());
        assert_eq!(error.template_name(), "test-nonexistent");
    }

    #[test]
    fn test_mock_template_creation() {
        let scenario = TestScenario::VariableSubstitution;
        let template = scenario.create_mock_template("glitchgrove");
        assert_eq!(template.name, "test-variables");
        assert!(template.source_repo.contains("template-test-variables"));
    }
}
