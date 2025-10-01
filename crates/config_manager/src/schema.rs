//! JSON Schema-based validation system for configuration files.
//!
//! This module provides comprehensive schema validation for all configuration file types
//! in the organization hierarchy. It uses JSON Schema to define structural and semantic
//! validation rules that go beyond basic TOML parsing.
//!
//! # Schema Validation Features
//!
//! - **Structural Validation**: Ensures all required fields are present and correctly typed
//! - **Semantic Validation**: Validates business rules and data relationships
//! - **Format Validation**: Validates field formats (URLs, email addresses, patterns)
//! - **Range Validation**: Validates numeric ranges and string length constraints
//! - **Enum Validation**: Validates that values match allowed options
//! - **Cross-field Validation**: Validates relationships between different fields
//!
//! # Schema Types
//!
//! The module provides validators for each configuration type:
//! - `GlobalDefaultsValidator` - Validates global defaults configuration
//! - `TeamConfigValidator` - Validates team-specific configuration
//! - `RepositoryTypeConfigValidator` - Validates repository type configuration
//! - `TemplateConfigValidator` - Validates template configuration
//!
//! # Examples
//!
//! ```rust
//! use config_manager::schema::{SchemaValidator, ValidationResult};
//! use config_manager::organization::GlobalDefaults;
//!
//! let validator = SchemaValidator::new();
//! let config = GlobalDefaults::new();
//! let result = validator.validate_global_defaults(&config);
//!
//! if result.is_valid() {
//!     println!("Configuration is valid");
//! } else {
//!     for error in result.errors() {
//!         eprintln!("Validation error: {}", error.message());
//!     }
//! }
//! ```

use crate::organization::{
    GlobalDefaults, GlobalDefaultsEnhanced, RepositoryTypeConfig, TeamConfig, TemplateConfig,
};
use jsonschema::{JSONSchema, ValidationError};
use schemars::{schema_for, JsonSchema};
use serde_json::Value;
use thiserror::Error;

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;

/// Errors that can occur during schema validation.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    /// Error generating JSON schema from Rust types.
    #[error("Schema generation failed: {message}")]
    SchemaGeneration { message: String },

    /// Error compiling the JSON schema for validation.
    #[error("Schema compilation failed: {message}")]
    SchemaCompilation { message: String },

    /// Error converting configuration to JSON for validation.
    #[error("JSON serialization failed: {message}")]
    JsonSerialization { message: String },

    /// Configuration validation failed against the schema.
    #[error("Configuration validation failed: {message}")]
    ValidationFailed { message: String },

    /// Custom validation rule failed.
    #[error("Custom validation failed for field '{field_path}': {message}")]
    CustomValidationFailed { field_path: String, message: String },
}

/// Severity level for validation issues.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationSeverity {
    /// Critical error that prevents configuration usage.
    Error,
    /// Warning about potentially problematic configuration.
    Warning,
    /// Informational message about configuration best practices.
    Info,
}

/// A single validation issue found during schema validation.
///
/// Validation issues represent problems found when validating configuration
/// against JSON schemas or custom validation rules. Each issue includes
/// detailed information about the problem location and suggested fixes.
///
/// # Examples
///
/// ```rust
/// use config_manager::schema::{ValidationIssue, ValidationSeverity};
///
/// let issue = ValidationIssue::new(
///     ValidationSeverity::Error,
///     "repository.wiki.value".to_string(),
///     "Field is required but not present".to_string(),
///     Some("Add a 'wiki' field to the repository section".to_string()),
/// );
///
/// println!("Error in {}: {}", issue.field_path(), issue.message());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    /// The severity level of this validation issue
    severity: ValidationSeverity,
    /// The JSON path to the field that caused the issue
    field_path: String,
    /// Human-readable description of the validation issue
    message: String,
    /// Optional suggestion for how to fix the issue
    suggestion: Option<String>,
    /// The invalid value that caused the issue, if applicable
    invalid_value: Option<String>,
    /// The expected value or format, if applicable
    expected_value: Option<String>,
}

impl ValidationIssue {
    /// Creates a new validation issue.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the issue (Error, Warning, Info)
    /// * `field_path` - The JSON path to the problematic field (e.g., "repository.wiki.value")
    /// * `message` - Human-readable description of the problem
    /// * `suggestion` - Optional suggestion for fixing the issue
    ///
    /// # Returns
    ///
    /// A new `ValidationIssue` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::new(
    ///     ValidationSeverity::Warning,
    ///     "webhooks[0].url".to_string(),
    ///     "URL uses insecure HTTP protocol".to_string(),
    ///     Some("Change to HTTPS for security".to_string()),
    /// );
    /// ```
    pub fn new(
        severity: ValidationSeverity,
        field_path: String,
        message: String,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            severity,
            field_path,
            message,
            suggestion,
            invalid_value: None,
            expected_value: None,
        }
    }

    /// Creates a validation issue with specific value information.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the issue
    /// * `field_path` - The JSON path to the problematic field
    /// * `message` - Human-readable description of the problem
    /// * `invalid_value` - The actual value that caused the problem
    /// * `expected_value` - The expected value or format
    /// * `suggestion` - Optional suggestion for fixing the issue
    ///
    /// # Returns
    ///
    /// A new `ValidationIssue` instance with value details.
    pub fn with_values(
        severity: ValidationSeverity,
        field_path: String,
        message: String,
        invalid_value: Option<String>,
        expected_value: Option<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            severity,
            field_path,
            message,
            suggestion,
            invalid_value,
            expected_value,
        }
    }

    /// Returns the severity level of this issue.
    pub fn severity(&self) -> &ValidationSeverity {
        &self.severity
    }

    /// Returns the field path where this issue was found.
    pub fn field_path(&self) -> &str {
        &self.field_path
    }

    /// Returns the human-readable message describing this issue.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the suggestion for fixing this issue, if available.
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    /// Returns the invalid value that caused this issue, if available.
    pub fn invalid_value(&self) -> Option<&str> {
        self.invalid_value.as_deref()
    }

    /// Returns the expected value or format, if available.
    pub fn expected_value(&self) -> Option<&str> {
        self.expected_value.as_deref()
    }

    /// Returns true if this is an error-level issue.
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Error)
    }

    /// Returns true if this is a warning-level issue.
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Warning)
    }

    /// Returns true if this is an info-level issue.
    pub fn is_info(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Info)
    }
}

/// Result of a schema validation operation.
///
/// Contains all validation issues found during validation, categorized by severity.
/// Provides convenience methods for checking validation status and retrieving
/// specific types of issues.
///
/// # Examples
///
/// ```rust
/// use config_manager::schema::{ValidationResult, ValidationIssue, ValidationSeverity};
///
/// let mut result = ValidationResult::new();
/// result.add_error("field", "Field is required", None);
/// result.add_warning("other_field", "Field is deprecated", Some("Use new_field instead"));
///
/// if !result.is_valid() {
///     println!("Found {} errors and {} warnings",
///         result.error_count(), result.warning_count());
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// All validation issues found, regardless of severity
    issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Creates a new empty validation result.
    ///
    /// # Returns
    ///
    /// A new `ValidationResult` with no issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::ValidationResult;
    ///
    /// let result = ValidationResult::new();
    /// assert!(result.is_valid());
    /// ```
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Adds an error-level validation issue.
    ///
    /// # Arguments
    ///
    /// * `field_path` - The JSON path to the problematic field
    /// * `message` - Human-readable description of the error
    /// * `suggestion` - Optional suggestion for fixing the error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::ValidationResult;
    ///
    /// let mut result = ValidationResult::new();
    /// result.add_error("repository.name", "Field is required",
    ///     Some("Add a 'name' field"));
    /// ```
    pub fn add_error(&mut self, field_path: &str, message: &str, suggestion: Option<&str>) {
        self.issues.push(ValidationIssue::new(
            ValidationSeverity::Error,
            field_path.to_string(),
            message.to_string(),
            suggestion.map(String::from),
        ));
    }

    /// Adds a warning-level validation issue.
    ///
    /// # Arguments
    ///
    /// * `field_path` - The JSON path to the problematic field
    /// * `message` - Human-readable description of the warning
    /// * `suggestion` - Optional suggestion for addressing the warning
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::ValidationResult;
    ///
    /// let mut result = ValidationResult::new();
    /// result.add_warning("webhooks[0].url", "Using HTTP instead of HTTPS",
    ///     Some("Consider using HTTPS for security"));
    /// ```
    pub fn add_warning(&mut self, field_path: &str, message: &str, suggestion: Option<&str>) {
        self.issues.push(ValidationIssue::new(
            ValidationSeverity::Warning,
            field_path.to_string(),
            message.to_string(),
            suggestion.map(String::from),
        ));
    }

    /// Adds an info-level validation issue.
    ///
    /// # Arguments
    ///
    /// * `field_path` - The JSON path to the field
    /// * `message` - Human-readable informational message
    /// * `suggestion` - Optional suggestion for improvement
    pub fn add_info(&mut self, field_path: &str, message: &str, suggestion: Option<&str>) {
        self.issues.push(ValidationIssue::new(
            ValidationSeverity::Info,
            field_path.to_string(),
            message.to_string(),
            suggestion.map(String::from),
        ));
    }

    /// Adds a validation issue to the result.
    ///
    /// # Arguments
    ///
    /// * `issue` - The validation issue to add
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Returns true if the validation was successful (no errors).
    ///
    /// Warnings and info messages do not prevent validation success.
    ///
    /// # Returns
    ///
    /// `true` if no error-level issues were found, `false` otherwise.
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    /// Returns true if any error-level issues were found.
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_error())
    }

    /// Returns true if any warning-level issues were found.
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_warning())
    }

    /// Returns true if any info-level issues were found.
    pub fn has_info(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_info())
    }

    /// Returns the number of error-level issues.
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_error()).count()
    }

    /// Returns the number of warning-level issues.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.is_warning())
            .count()
    }

    /// Returns the number of info-level issues.
    pub fn info_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_info()).count()
    }

    /// Returns the total number of issues.
    pub fn total_count(&self) -> usize {
        self.issues.len()
    }

    /// Returns all validation issues.
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Returns only the error-level issues.
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.is_error())
            .collect()
    }

    /// Returns only the warning-level issues.
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.is_warning())
            .collect()
    }

    /// Returns only the info-level issues.
    pub fn info_messages(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_info()).collect()
    }

    /// Merges another validation result into this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The validation result to merge in
    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom validation rule for configuration fields.
///
/// Custom validators allow organizations to define validation logic beyond
/// what can be expressed in JSON Schema. They are applied after schema
/// validation and can perform complex business rule validation.
///
/// # Examples
///
/// ```rust
/// use config_manager::schema::CustomValidator;
///
/// let validator = CustomValidator::new(
///     "webhooks.*.url".to_string(),
///     "Webhook URLs must use company domain".to_string(),
///     Box::new(|value| {
///         if value.contains("company.com") {
///             Ok(())
///         } else {
///             Err("URL must be within company domain".to_string())
///         }
///     })
/// );
/// ```
pub struct CustomValidator {
    /// Pattern matching field paths this validator applies to
    field_pattern: String,
    /// Human-readable description of what this validator checks
    description: String,
    /// The validation function
    validator: Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
}

impl CustomValidator {
    /// Creates a new custom validator.
    ///
    /// # Arguments
    ///
    /// * `field_pattern` - Pattern matching field paths (e.g., "webhooks.*.url")
    /// * `description` - Description of what this validator checks
    /// * `validator` - Validation function that returns `Ok(())` for valid values
    ///
    /// # Returns
    ///
    /// A new `CustomValidator` instance.
    pub fn new<F>(field_pattern: String, description: String, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            field_pattern,
            description,
            validator: Box::new(validator),
        }
    }

    /// Returns the field pattern this validator applies to.
    pub fn field_pattern(&self) -> &str {
        &self.field_pattern
    }

    /// Returns the description of what this validator checks.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Validates a value using this validator's logic.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the value is valid, `Err(message)` otherwise.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        (self.validator)(value)
    }
}

/// Main schema validator for configuration files.
///
/// This is the primary interface for performing schema-based validation
/// on configuration structures. It manages JSON schema compilation,
/// validation execution, and custom validation rules.
///
/// The validator supports both built-in JSON Schema validation and
/// custom validation rules for business logic that cannot be expressed
/// in schema form.
///
/// # Examples
///
/// ```rust
/// use config_manager::schema::SchemaValidator;
/// use config_manager::organization::GlobalDefaults;
///
/// let validator = SchemaValidator::new();
/// let config = GlobalDefaults::new();
/// let result = validator.validate_global_defaults(&config);
///
/// if !result.is_valid() {
///     for error in result.errors() {
///         println!("Error: {}", error.message());
///     }
/// }
/// ```
pub struct SchemaValidator {
    /// Custom validation rules registered with the validator
    custom_validators: Vec<CustomValidator>,
    /// Whether to enforce strict validation rules
    strict_mode: bool,
}

impl SchemaValidator {
    /// Creates a new schema validator with default settings.
    ///
    /// The validator is created in strict mode with no custom validators.
    /// Custom validators can be added using `add_custom_validator()`.
    ///
    /// # Returns
    ///
    /// A new `SchemaValidator` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::SchemaValidator;
    ///
    /// let validator = SchemaValidator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            custom_validators: Vec::new(),
            strict_mode: true,
        }
    }

    /// Creates a new schema validator with custom settings.
    ///
    /// # Arguments
    ///
    /// * `strict_mode` - Whether to enforce strict validation rules
    ///
    /// # Returns
    ///
    /// A new `SchemaValidator` instance with the specified settings.
    pub fn with_strict_mode(strict_mode: bool) -> Self {
        Self {
            custom_validators: Vec::new(),
            strict_mode,
        }
    }

    /// Adds a custom validation rule to the validator.
    ///
    /// Custom validators are applied after JSON schema validation and allow
    /// for complex business rule validation that cannot be expressed in schema form.
    ///
    /// # Arguments
    ///
    /// * `validator` - The custom validator to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::{SchemaValidator, CustomValidator};
    ///
    /// let mut validator = SchemaValidator::new();
    /// let custom = CustomValidator::new(
    ///     "webhooks.*.url".to_string(),
    ///     "Check webhook domain".to_string(),
    ///     Box::new(|url| {
    ///         if url.contains("company.com") {
    ///             Ok(())
    ///         } else {
    ///             Err("Must use company domain".to_string())
    ///         }
    ///     })
    /// );
    /// validator.add_custom_validator(custom);
    /// ```
    pub fn add_custom_validator(&mut self, validator: CustomValidator) {
        self.custom_validators.push(validator);
    }

    /// Validates global defaults configuration against its schema.
    ///
    /// This method performs comprehensive validation including:
    /// - JSON Schema structural validation
    /// - Field format validation (URLs, patterns)
    /// - Range and constraint validation
    /// - Custom business rule validation
    ///
    /// # Arguments
    ///
    /// * `config` - The global defaults configuration to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found during validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::SchemaValidator;
    /// use config_manager::organization::GlobalDefaults;
    ///
    /// let validator = SchemaValidator::new();
    /// let config = GlobalDefaults::new();
    /// let result = validator.validate_global_defaults(&config);
    ///
    /// assert!(result.is_valid());
    /// ```
    ///
    /// # Error Conditions
    ///
    /// Returns validation errors for:
    /// - Missing required fields
    /// - Invalid field types or formats
    /// - Values outside allowed ranges
    /// - Business rule violations
    /// - Security policy violations
    pub fn validate_global_defaults(&self, config: &GlobalDefaults) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Validates enhanced global defaults configuration against its schema.
    ///
    /// This method validates the more comprehensive `GlobalDefaultsEnhanced`
    /// structure with additional validation for advanced configuration options.
    ///
    /// # Arguments
    ///
    /// * `config` - The enhanced global defaults configuration to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found during validation.
    pub fn validate_enhanced_global_defaults(
        &self,
        config: &GlobalDefaultsEnhanced,
    ) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Validates team configuration against its schema.
    ///
    /// This method validates team-specific configuration including verification
    /// that any overrides are properly structured and that team-specific
    /// configurations follow security and organizational policies.
    ///
    /// # Arguments
    ///
    /// * `config` - The team configuration to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found during validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::schema::SchemaValidator;
    /// use config_manager::organization::TeamConfig;
    ///
    /// let validator = SchemaValidator::new();
    /// let config = TeamConfig::new();
    /// let result = validator.validate_team_config(&config);
    /// ```
    pub fn validate_team_config(&self, config: &TeamConfig) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Validates repository type configuration against its schema.
    ///
    /// This method validates repository type-specific configuration including
    /// labels, webhooks, GitHub Apps, and other type-specific settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The repository type configuration to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found during validation.
    pub fn validate_repository_type_config(
        &self,
        config: &RepositoryTypeConfig,
    ) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Validates template configuration against its schema.
    ///
    /// This method validates template-specific configuration including template
    /// metadata, variable definitions, repository type specifications, and
    /// template constraints.
    ///
    /// # Arguments
    ///
    /// * `config` - The template configuration to validate
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any issues found during validation.
    pub fn validate_template_config(&self, config: &TemplateConfig) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Generates a JSON Schema for a configuration type.
    ///
    /// This method generates JSON Schema definitions that can be used for
    /// external validation tools or documentation generation.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The configuration type that implements `JsonSchema`
    ///
    /// # Returns
    ///
    /// A JSON Schema value representing the structure of the configuration type.
    ///
    /// # Errors
    ///
    /// Returns a `SchemaValidationError` if schema generation fails.
    pub fn generate_schema<T: JsonSchema>(&self) -> Result<Value, SchemaValidationError> {
        let schema = schema_for!(T);
        serde_json::to_value(&schema).map_err(|e| SchemaValidationError::SchemaGeneration {
            message: format!("Failed to serialize schema: {}", e),
        })
    }

    /// Validates a JSON value against a compiled schema.
    ///
    /// This is a low-level method used by the type-specific validation methods.
    /// It performs the actual JSON Schema validation and converts validation
    /// errors into `ValidationIssue` objects.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value to validate
    /// * `schema` - The compiled JSON schema
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any schema validation issues found.
    fn validate_against_schema(&self, value: &Value, schema: &JSONSchema) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Applies custom validation rules to a configuration value.
    ///
    /// This method runs all registered custom validators against the provided
    /// JSON value, applying pattern matching to determine which validators
    /// are relevant for each field.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON representation of the configuration
    /// * `field_prefix` - The current field path prefix for nested validation
    ///
    /// # Returns
    ///
    /// A `ValidationResult` containing any custom validation issues found.
    fn apply_custom_validation(&self, value: &Value, field_prefix: &str) -> ValidationResult {
        // TODO: Implement validation logic
        ValidationResult::new()
    }

    /// Converts a JSON Schema validation error to a validation issue.
    ///
    /// # Arguments
    ///
    /// * `error` - The JSON Schema validation error
    ///
    /// # Returns
    ///
    /// A `ValidationIssue` representing the schema validation failure.
    fn convert_schema_error(&self, error: &ValidationError) -> ValidationIssue {
        let field_path = error.instance_path.to_string();
        let message = error.to_string();

        ValidationIssue::new(ValidationSeverity::Error, field_path, message, None)
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}
