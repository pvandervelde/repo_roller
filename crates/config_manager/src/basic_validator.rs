//! Basic implementation of configuration validation.
//!
//! Provides MVP validation including:
//! - Schema validation for all configuration types
//! - Business rule enforcement
//! - Security policy validation
//!
//! # Examples
//!
//! ```rust
//! use config_manager::{BasicConfigurationValidator, MergedConfiguration};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let validator = BasicConfigurationValidator::new();
//!     let merged_config = MergedConfiguration::default();
//!
//!     let result = validator.validate_merged_config(&merged_config).await?;
//!
//!     if !result.is_valid() {
//!         for error in &result.errors {
//!             eprintln!("Validation error: {} - {}", error.field_path, error.message);
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use crate::{
    global_defaults::GlobalDefaults,
    merged_config::MergedConfiguration,
    repository_type_config::RepositoryTypeConfig,
    settings::{
        BranchProtectionSettings, EnvironmentConfig, GitHubAppConfig, PullRequestSettings,
        RepositorySettings, WebhookConfig,
    },
    team_config::TeamConfig,
    template_config::TemplateConfig as NewTemplateConfig,
    validator::{
        ConfigurationValidator, ValidationError, ValidationErrorType, ValidationResult,
        ValidationWarning,
    },
    ConfigurationResult,
};
use async_trait::async_trait;

/// Basic implementation of configuration validation.
///
/// Provides comprehensive validation for all configuration types including
/// schema validation, business rules, and security policy enforcement.
///
/// # Examples
///
/// ```rust
/// use config_manager::BasicConfigurationValidator;
///
/// let validator = BasicConfigurationValidator::new();
/// // Use validator to validate configurations...
/// ```
pub struct BasicConfigurationValidator;

impl BasicConfigurationValidator {
    /// Create a new basic configuration validator.
    pub fn new() -> Self {
        Self
    }

    // ========================================================================
    // Schema Validation Helpers
    // ========================================================================

    /// Validate repository settings.
    fn validate_repository_settings(&self, settings: &RepositorySettings) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // All repository settings are boolean options - no specific validation needed
        // for MVP beyond type checking (which serde handles)

        errors
    }

    /// Validate pull request settings.
    fn validate_pull_request_settings(
        &self,
        settings: &PullRequestSettings,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Validate required_approving_review_count is non-negative
        if let Some(count) = &settings.required_approving_review_count {
            if count.value < 0 {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidValue,
                    field_path: "pull_requests.required_approving_review_count".to_string(),
                    message: format!("Review count cannot be negative, got: {}", count.value),
                    suggestion: Some("Use a non-negative integer (0 or greater)".to_string()),
                });
            }
        }

        errors
    }

    /// Validate branch protection settings.
    fn validate_branch_protection(
        &self,
        settings: &BranchProtectionSettings,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Validate that if branch protection is enabled, it has required checks
        if let Some(require_checks) = &settings.require_status_checks {
            if require_checks.value {
                // If requiring status checks, ensure checks list is not empty
                if let Some(checks_list) = &settings.required_status_checks_list {
                    if checks_list.is_empty() {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::BusinessRuleViolation,
                            field_path: "branch_protection.required_status_checks_list"
                                .to_string(),
                            message:
                                "Status checks list cannot be empty when branch protection requires status checks"
                                    .to_string(),
                            suggestion: Some(
                                "Add at least one required status check or set require_status_checks to false"
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
        }

        errors
    }

    /// Validate webhook configurations.
    fn validate_webhooks(&self, webhooks: &[WebhookConfig]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (index, webhook) in webhooks.iter().enumerate() {
            // Basic URL format validation
            if !webhook.url.starts_with("http://") && !webhook.url.starts_with("https://") {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidValue,
                    field_path: format!("webhooks[{}].url", index),
                    message: format!("Invalid webhook URL format: {}", webhook.url),
                    suggestion: Some("URL must start with http:// or https://".to_string()),
                });
            }

            // Validate events list is not empty
            if webhook.events.is_empty() {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::SchemaViolation,
                    field_path: format!("webhooks[{}].events", index),
                    message: "Webhook must have at least one event".to_string(),
                    suggestion: Some(
                        "Add at least one event like 'push' or 'pull_request'".to_string(),
                    ),
                });
            }
        }

        errors
    }

    /// Validate GitHub App configurations.
    fn validate_github_apps(&self, apps: &[GitHubAppConfig]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (index, app) in apps.iter().enumerate() {
            // Validate app_id is positive
            if app.app_id == 0 {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidValue,
                    field_path: format!("github_apps[{}].app_id", index),
                    message: "GitHub App ID must be a positive integer".to_string(),
                    suggestion: Some("Use the actual app ID from GitHub App settings".to_string()),
                });
            }

            // Validate permissions are not empty
            if app.permissions.is_empty() {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::SchemaViolation,
                    field_path: format!("github_apps[{}].permissions", index),
                    message: "GitHub App must have at least one permission".to_string(),
                    suggestion: Some("Specify required permissions for the app".to_string()),
                });
            }
        }

        errors
    }

    /// Validate environment configurations.
    fn validate_environments(&self, envs: &[EnvironmentConfig]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (index, env) in envs.iter().enumerate() {
            // Validate environment name is not empty
            if env.name.is_empty() {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::RequiredFieldMissing,
                    field_path: format!("environments[{}].name", index),
                    message: "Environment name cannot be empty".to_string(),
                    suggestion: Some(
                        "Provide a meaningful environment name like 'production' or 'staging'"
                            .to_string(),
                    ),
                });
            }

            // If protection rules exist, validate wait_timer is non-negative
            if let Some(rules) = &env.protection_rules {
                if let Some(wait_timer) = rules.wait_timer {
                    if wait_timer < 0 {
                        errors.push(ValidationError {
                            error_type: ValidationErrorType::InvalidValue,
                            field_path: format!("environments[{}].protection_rules.wait_timer", index),
                            message: format!(
                                "Wait timer cannot be negative, got: {}",
                                wait_timer
                            ),
                            suggestion: Some(
                                "Use 0 for no wait or a positive number of minutes".to_string(),
                            ),
                        });
                    }
                }
            }
        }

        errors
    }

    // ========================================================================
    // Business Rule Validation Helpers
    // ========================================================================

    /// Validate security policies are enforced.
    ///
    /// Ensures that security-critical settings cannot be disabled:
    /// - security_advisories must be enabled
    /// - vulnerability_reporting must be enabled
    fn validate_security_policies(&self, merged: &MergedConfiguration) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check security_advisories
        if let Some(security_advisories) = &merged.repository.security_advisories {
            if !security_advisories.value {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::BusinessRuleViolation,
                    field_path: "repository.security_advisories".to_string(),
                    message: "Security advisories cannot be disabled".to_string(),
                    suggestion: Some(
                        "This is a security policy requirement - remove the override or set it to true"
                            .to_string(),
                    ),
                });
            }
        }

        // Check vulnerability_reporting
        if let Some(vulnerability_reporting) = &merged.repository.vulnerability_reporting {
            if !vulnerability_reporting.value {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::BusinessRuleViolation,
                    field_path: "repository.vulnerability_reporting".to_string(),
                    message: "Vulnerability reporting cannot be disabled".to_string(),
                    suggestion: Some(
                        "This is a security policy requirement - remove the override or set it to true"
                            .to_string(),
                    ),
                });
            }
        }

        errors
    }

    /// Generate warnings for webhook URLs using HTTP instead of HTTPS.
    fn validate_webhook_urls(&self, webhooks: &[WebhookConfig]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (index, webhook) in webhooks.iter().enumerate() {
            if webhook.url.starts_with("http://") {
                warnings.push(ValidationWarning {
                    field_path: format!("webhooks[{}].url", index),
                    message: "Webhook URL uses HTTP instead of HTTPS".to_string(),
                    recommendation: Some(
                        "Use HTTPS for secure webhook delivery to prevent data interception"
                            .to_string(),
                    ),
                });
            }
        }

        warnings
    }

    /// Validate branch protection completeness.
    ///
    /// If branch protection is enabled, ensure it's properly configured.
    fn validate_branch_protection_completeness(
        &self,
        settings: &BranchProtectionSettings,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check if requiring pull request reviews
        if let Some(require_pr) = &settings.require_pull_request_reviews {
            if require_pr.value {
                // Ensure required_approving_review_count is set
                if settings.required_approving_review_count.is_none() {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::BusinessRuleViolation,
                        field_path: "branch_protection.required_approving_review_count"
                            .to_string(),
                        message: "Required approving review count must be specified when pull request reviews are required"
                            .to_string(),
                        suggestion: Some(
                            "Set required_approving_review_count to at least 1".to_string(),
                        ),
                    });
                }
            }
        }

        errors
    }

    // ========================================================================
    // Cross-Field Validation
    // ========================================================================

    /// Validate conditional requirements across configuration.
    fn validate_conditional_requirements(
        &self,
        _merged: &MergedConfiguration,
    ) -> Vec<ValidationError> {
        let errors = Vec::new();

        // Placeholder for future cross-field validations
        // Examples: If feature X enabled, field Y must be set

        errors
    }
}

impl Default for BasicConfigurationValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigurationValidator for BasicConfigurationValidator {
    async fn validate_global_defaults(
        &self,
        defaults: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate repository settings
        if let Some(repo_settings) = &defaults.repository {
            result.add_errors(self.validate_repository_settings(repo_settings));
        }

        // Validate pull request settings
        if let Some(pr_settings) = &defaults.pull_requests {
            result.add_errors(self.validate_pull_request_settings(pr_settings));
        }

        // Validate branch protection
        if let Some(branch_protection) = &defaults.branch_protection {
            result.add_errors(self.validate_branch_protection(branch_protection));
        }

        // Validate webhooks
        if let Some(webhooks) = &defaults.webhooks {
            result.add_errors(self.validate_webhooks(webhooks));
            result.add_warnings(self.validate_webhook_urls(webhooks));
        }

        // Validate GitHub apps
        if let Some(apps) = &defaults.github_apps {
            result.add_errors(self.validate_github_apps(apps));
        }

        // Validate environments
        if let Some(envs) = &defaults.environments {
            result.add_errors(self.validate_environments(envs));
        }

        Ok(result)
    }

    async fn validate_team_config(
        &self,
        config: &TeamConfig,
        _global: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate team configuration using same validators as global
        // Override validation is handled by ConfigurationMerger

        if let Some(repo_settings) = &config.repository {
            result.add_errors(self.validate_repository_settings(repo_settings));
        }

        if let Some(pr_settings) = &config.pull_requests {
            result.add_errors(self.validate_pull_request_settings(pr_settings));
        }

        if let Some(branch_protection) = &config.branch_protection {
            result.add_errors(self.validate_branch_protection(branch_protection));
        }

        if let Some(webhooks) = &config.webhooks {
            result.add_errors(self.validate_webhooks(webhooks));
            result.add_warnings(self.validate_webhook_urls(webhooks));
        }

        if let Some(apps) = &config.github_apps {
            result.add_errors(self.validate_github_apps(apps));
        }

        if let Some(envs) = &config.environments {
            result.add_errors(self.validate_environments(envs));
        }

        Ok(result)
    }

    async fn validate_repository_type_config(
        &self,
        config: &RepositoryTypeConfig,
        _global: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate repository type configuration using same validators
        // Override validation is handled by ConfigurationMerger

        if let Some(repo_settings) = &config.repository {
            result.add_errors(self.validate_repository_settings(repo_settings));
        }

        if let Some(pr_settings) = &config.pull_requests {
            result.add_errors(self.validate_pull_request_settings(pr_settings));
        }

        if let Some(branch_protection) = &config.branch_protection {
            result.add_errors(self.validate_branch_protection(branch_protection));
        }

        if let Some(webhooks) = &config.webhooks {
            result.add_errors(self.validate_webhooks(webhooks));
            result.add_warnings(self.validate_webhook_urls(webhooks));
        }

        if let Some(apps) = &config.github_apps {
            result.add_errors(self.validate_github_apps(apps));
        }

        if let Some(envs) = &config.environments {
            result.add_errors(self.validate_environments(envs));
        }

        Ok(result)
    }

    async fn validate_template_config(
        &self,
        config: &NewTemplateConfig,
    ) -> ConfigurationResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate template configuration using same validators

        if let Some(repo_settings) = &config.repository {
            result.add_errors(self.validate_repository_settings(repo_settings));
        }

        if let Some(pr_settings) = &config.pull_requests {
            result.add_errors(self.validate_pull_request_settings(pr_settings));
        }

        if let Some(branch_protection) = &config.branch_protection {
            result.add_errors(self.validate_branch_protection(branch_protection));
        }

        if let Some(webhooks) = &config.webhooks {
            result.add_errors(self.validate_webhooks(webhooks));
            result.add_warnings(self.validate_webhook_urls(webhooks));
        }

        if let Some(apps) = &config.github_apps {
            result.add_errors(self.validate_github_apps(apps));
        }

        if let Some(envs) = &config.environments {
            result.add_errors(self.validate_environments(envs));
        }

        Ok(result)
    }

    async fn validate_merged_config(
        &self,
        merged: &MergedConfiguration,
    ) -> ConfigurationResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate all settings in merged configuration
        result.add_errors(self.validate_repository_settings(&merged.repository));
        result.add_errors(self.validate_pull_request_settings(&merged.pull_requests));
        result.add_errors(self.validate_branch_protection(&merged.branch_protection));
        result.add_errors(self.validate_webhooks(&merged.webhooks));
        result.add_errors(self.validate_github_apps(&merged.github_apps));
        result.add_errors(self.validate_environments(&merged.environments));

        // Business rule validation
        result.add_errors(self.validate_security_policies(merged));
        result.add_errors(self.validate_branch_protection_completeness(&merged.branch_protection));
        result.add_errors(self.validate_conditional_requirements(merged));

        // Warnings
        result.add_warnings(self.validate_webhook_urls(&merged.webhooks));

        Ok(result)
    }
}

#[cfg(test)]
#[path = "basic_validator_tests.rs"]
mod tests;
