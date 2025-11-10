//! Configuration validation types and trait.
//!
//! This module provides the core validation infrastructure for configuration
//! validation including the `ConfigurationValidator` trait, validation result
//! types, and error structures.
//!
//! # Examples
//!
//! ```rust
//! use config_manager::{ValidationResult, ValidationError, ValidationErrorType};
//!
//! let mut result = ValidationResult::new();
//!
//! // Add an error
//! result.add_error(ValidationError {
//!     error_type: ValidationErrorType::RequiredFieldMissing,
//!     field_path: "repository.issues".to_string(),
//!     message: "Issues setting is required".to_string(),
//!     suggestion: Some("Set repository.issues = true".to_string()),
//! });
//!
//! assert!(!result.is_valid());
//! assert_eq!(result.errors.len(), 1);
//! ```

use crate::{
    global_defaults::GlobalDefaults, merged_config::MergedConfiguration,
    repository_type_config::RepositoryTypeConfig, team_config::TeamConfig,
    template_config::TemplateConfig as NewTemplateConfig, ConfigurationResult,
};
use async_trait::async_trait;

/// Result of configuration validation.
///
/// Contains all validation errors and warnings found during validation.
/// Validation is considered successful only if no errors are present.
///
/// # Examples
///
/// ```rust
/// use config_manager::{ValidationResult, ValidationError, ValidationErrorType};
///
/// let mut result = ValidationResult::new();
/// assert!(result.is_valid());
///
/// result.add_error(ValidationError {
///     error_type: ValidationErrorType::InvalidValue,
///     field_path: "pull_requests.required_approving_review_count".to_string(),
///     message: "Review count cannot be negative".to_string(),
///     suggestion: Some("Use a non-negative integer".to_string()),
/// });
///
/// assert!(!result.is_valid());
/// ```
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// List of validation errors (blocking issues).
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (non-blocking suggestions).
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new empty validation result.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add a validation error.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a validation warning.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Add multiple errors at once.
    pub fn add_errors(&mut self, errors: Vec<ValidationError>) {
        self.errors.extend(errors);
    }

    /// Add multiple warnings at once.
    pub fn add_warnings(&mut self, warnings: Vec<ValidationWarning>) {
        self.warnings.extend(warnings);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual validation error with context.
///
/// Provides detailed information about what went wrong and how to fix it.
///
/// # Examples
///
/// ```rust
/// use config_manager::{ValidationError, ValidationErrorType};
///
/// let error = ValidationError {
///     error_type: ValidationErrorType::SchemaViolation,
///     field_path: "branch_protection.required_status_checks".to_string(),
///     message: "Status checks list cannot be empty when branch protection is enabled".to_string(),
///     suggestion: Some("Add at least one required status check or disable branch protection".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// The category of validation error.
    pub error_type: ValidationErrorType,
    /// Dot-separated path to the field that failed validation.
    pub field_path: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional suggestion for how to fix the error.
    pub suggestion: Option<String>,
}

/// Validation error categories.
///
/// Used to classify validation errors for appropriate handling and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationErrorType {
    /// Configuration structure doesn't match expected schema.
    SchemaViolation,
    /// A required field is missing or None.
    RequiredFieldMissing,
    /// A field value is invalid (wrong type, out of range, etc.).
    InvalidValue,
    /// A business rule constraint was violated.
    BusinessRuleViolation,
    /// Attempted to override a non-overridable setting.
    OverrideNotAllowed,
}

impl std::fmt::Display for ValidationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaViolation => write!(f, "SchemaViolation"),
            Self::RequiredFieldMissing => write!(f, "RequiredFieldMissing"),
            Self::InvalidValue => write!(f, "InvalidValue"),
            Self::BusinessRuleViolation => write!(f, "BusinessRuleViolation"),
            Self::OverrideNotAllowed => write!(f, "OverrideNotAllowed"),
        }
    }
}

/// Non-blocking validation warning.
///
/// Warnings indicate potential issues or best practice violations that
/// don't prevent repository creation but should be addressed.
///
/// # Examples
///
/// ```rust
/// use config_manager::ValidationWarning;
///
/// let warning = ValidationWarning {
///     field_path: "webhooks[0].url".to_string(),
///     message: "Webhook URL uses HTTP instead of HTTPS".to_string(),
///     recommendation: Some("Use HTTPS for secure webhook delivery".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    /// Dot-separated path to the field that triggered the warning.
    pub field_path: String,
    /// Human-readable warning message.
    pub message: String,
    /// Optional recommendation for best practice.
    pub recommendation: Option<String>,
}

/// Configuration validation service.
///
/// Validates configuration files for schema correctness, business rules,
/// and security policies. Implementations should validate all aspects of
/// the configuration and collect all errors/warnings in a single pass.
///
/// # Examples
///
/// ```rust,no_run
/// use config_manager::{ConfigurationValidator, GlobalDefaults, ValidationResult};
/// use async_trait::async_trait;
///
/// struct MyValidator;
///
/// #[async_trait]
/// impl ConfigurationValidator for MyValidator {
///     async fn validate_global_defaults(
///         &self,
///         defaults: &GlobalDefaults,
///     ) -> config_manager::ConfigurationResult<ValidationResult> {
///         let mut result = ValidationResult::new();
///         // Perform validation...
///         Ok(result)
///     }
///
///     async fn validate_team_config(
///         &self,
///         config: &config_manager::TeamConfig,
///         global: &GlobalDefaults,
///     ) -> config_manager::ConfigurationResult<ValidationResult> {
///         Ok(ValidationResult::new())
///     }
///
///     async fn validate_repository_type_config(
///         &self,
///         config: &config_manager::RepositoryTypeConfig,
///         global: &GlobalDefaults,
///     ) -> config_manager::ConfigurationResult<ValidationResult> {
///         Ok(ValidationResult::new())
///     }
///
///     async fn validate_template_config(
///         &self,
///         config: &config_manager::NewTemplateConfig,
///     ) -> config_manager::ConfigurationResult<ValidationResult> {
///         Ok(ValidationResult::new())
///     }
///
///     async fn validate_merged_config(
///         &self,
///         merged: &config_manager::MergedConfiguration,
///     ) -> config_manager::ConfigurationResult<ValidationResult> {
///         Ok(ValidationResult::new())
///     }
/// }
/// ```
#[async_trait]
pub trait ConfigurationValidator: Send + Sync {
    /// Validate global defaults configuration.
    ///
    /// Checks that the organization-wide defaults are valid and complete.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if validation logic fails (not validation failures).
    async fn validate_global_defaults(
        &self,
        defaults: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult>;

    /// Validate team configuration against global defaults.
    ///
    /// Checks that team overrides are valid and respect global policies.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if validation logic fails (not validation failures).
    async fn validate_team_config(
        &self,
        config: &TeamConfig,
        global: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult>;

    /// Validate repository type configuration against global defaults.
    ///
    /// Checks that repository type settings are valid and respect global policies.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if validation logic fails (not validation failures).
    async fn validate_repository_type_config(
        &self,
        config: &RepositoryTypeConfig,
        global: &GlobalDefaults,
    ) -> ConfigurationResult<ValidationResult>;

    /// Validate template configuration.
    ///
    /// Checks that template settings and variables are valid.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if validation logic fails (not validation failures).
    async fn validate_template_config(
        &self,
        config: &NewTemplateConfig,
    ) -> ConfigurationResult<ValidationResult>;

    /// Validate merged configuration before repository creation.
    ///
    /// Performs comprehensive validation of the final merged configuration
    /// including schema validation, business rules, and security policies.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if validation logic fails (not validation failures).
    async fn validate_merged_config(
        &self,
        merged: &MergedConfiguration,
    ) -> ConfigurationResult<ValidationResult>;
}

#[cfg(test)]
#[path = "validator_tests.rs"]
mod tests;
