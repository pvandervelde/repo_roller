//! Configuration validation and merging errors.
//!
//! This module provides error types for configuration validation, merging, and processing.

// Test modules
#[cfg(test)]
#[path = "errors_tests.rs"]
mod errors_tests;

/// Configuration validation and merging errors.
///
/// This enum represents the various errors that can occur during configuration
/// validation, merging, and processing. Each variant provides specific context
/// about the nature of the error to help with troubleshooting and resolution.
///
/// # Error Categories
///
/// - **Override Violations**: Attempts to override fixed settings
/// - **Validation Failures**: Schema or business rule violations
/// - **Missing Dependencies**: Required configurations not found
/// - **Format Errors**: Invalid configuration file formats
///
/// # Examples
///
/// ```rust
/// use config_manager::errors::ConfigurationError;
///
/// let error = ConfigurationError::OverrideNotAllowed {
///     field: "branch_protection_enabled".to_string(),
///     attempted_value: "false".to_string(),
///     global_value: "true".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationError {
    /// Attempted to override a setting that is marked as fixed in global defaults.
    ///
    /// This error occurs when a team or template configuration tries to override
    /// a global setting that has `override_allowed = false`. The error includes
    /// details about which field was being overridden and what values were involved.
    OverrideNotAllowed {
        /// The configuration field that cannot be overridden.
        field: String,
        /// The value that was attempted to be set.
        attempted_value: String,
        /// The fixed value from global defaults.
        global_value: String,
    },

    /// Configuration value failed validation rules.
    ///
    /// This error occurs when a configuration value doesn't meet schema requirements
    /// or business rules, such as invalid webhook URLs, malformed color codes for labels,
    /// or unsupported merge strategies.
    InvalidValue {
        /// The configuration field that has an invalid value.
        field: String,
        /// The invalid value that was provided.
        value: String,
        /// Description of why the value is invalid.
        reason: String,
    },

    /// Required configuration field is missing.
    ///
    /// This error occurs when a mandatory configuration field is not provided
    /// in contexts where it's required, such as missing global defaults that
    /// are needed for security compliance.
    RequiredFieldMissing {
        /// The name of the missing required field.
        field: String,
        /// Context about where the field was expected.
        context: String,
    },

    /// Configuration file format is invalid or corrupted.
    ///
    /// This error occurs when configuration files cannot be parsed due to
    /// syntax errors, unsupported formats, or schema violations.
    FormatError {
        /// The file that contains the format error.
        file: String,
        /// Description of the format error.
        error: String,
    },

    /// Metadata repository could not be found for the organization.
    ///
    /// This error occurs when neither configuration-based nor topic-based discovery
    /// methods can locate a metadata repository for the specified organization.
    RepositoryNotFound {
        /// The organization name that was searched.
        organization: String,
        /// The search method that was attempted.
        search_method: String,
    },

    /// Multiple metadata repositories found when only one was expected.
    ///
    /// This error occurs when topic-based discovery finds multiple repositories
    /// with the metadata topic, creating ambiguity about which one to use.
    MultipleRepositoriesFound {
        /// The organization name that was searched.
        organization: String,
        /// The names of the repositories that were found.
        repositories: Vec<String>,
        /// The search method that found multiple results.
        search_method: String,
    },

    /// Access denied when trying to access metadata repository.
    ///
    /// This error occurs when the GitHub App doesn't have sufficient permissions
    /// to access the metadata repository or the repository is private.
    AccessDenied {
        /// The full repository name (org/repo).
        repository: String,
        /// Description of the operation that was denied.
        operation: String,
    },

    /// Network error occurred during repository access.
    ///
    /// This error occurs when API calls fail due to network issues, rate limiting,
    /// or other communication problems with GitHub.
    NetworkError {
        /// Description of the network error.
        error: String,
        /// The operation that failed due to network issues.
        operation: String,
    },

    /// Repository structure is invalid or missing required components.
    ///
    /// This error occurs when a metadata repository doesn't have the required
    /// directory structure or is missing essential configuration files.
    InvalidRepositoryStructure {
        /// The repository that has invalid structure.
        repository: String,
        /// List of missing or invalid items.
        missing_items: Vec<String>,
    },

    /// Configuration file not found in repository.
    ///
    /// This error occurs when a specific configuration file is expected but
    /// doesn't exist in the metadata repository.
    FileNotFound {
        /// The path of the file that was not found.
        file_path: String,
        /// The repository where the file was expected.
        repository: String,
    },

    /// Configuration file could not be parsed.
    ///
    /// This error occurs when a configuration file exists but contains invalid
    /// TOML syntax or structure that prevents parsing.
    ParseError {
        /// The path of the file that could not be parsed.
        file_path: String,
        /// The repository containing the file.
        repository: String,
        /// The parsing error details.
        error: String,
    },

    /// Configuration validation failed against schema or business rules.
    ///
    /// This error occurs when a configuration file is syntactically valid but
    /// violates schema requirements or business rules.
    ValidationError {
        /// The path of the file that failed validation.
        file_path: String,
        /// The repository containing the file.
        repository: String,
        /// List of validation errors.
        errors: Vec<String>,
    },
}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationError::OverrideNotAllowed {
                field,
                attempted_value,
                global_value,
            } => write!(
                f,
                "Cannot override field '{}': attempted to set '{}' but global default '{}' is fixed",
                field, attempted_value, global_value
            ),
            ConfigurationError::InvalidValue { field, value, reason } => {
                write!(f, "Invalid value '{}' for field '{}': {}", value, field, reason)
            }
            ConfigurationError::RequiredFieldMissing { field, context } => {
                write!(f, "Required field '{}' is missing in {}", field, context)
            }
            ConfigurationError::FormatError { file, error } => {
                write!(f, "Format error in file '{}': {}", file, error)
            }
            ConfigurationError::RepositoryNotFound {
                organization,
                search_method,
            } => write!(
                f,
                "Metadata repository not found for organization '{}' using {}",
                organization, search_method
            ),
            ConfigurationError::MultipleRepositoriesFound {
                organization,
                repositories,
                search_method,
            } => write!(
                f,
                "Multiple metadata repositories found for organization '{}' using {}: {}",
                organization,
                search_method,
                repositories.join(", ")
            ),
            ConfigurationError::AccessDenied { repository, operation } => {
                write!(f, "Access denied to repository '{}' for operation: {}", repository, operation)
            }
            ConfigurationError::NetworkError { error, operation } => {
                write!(f, "Network error during {}: {}", operation, error)
            }
            ConfigurationError::InvalidRepositoryStructure {
                repository,
                missing_items,
            } => write!(
                f,
                "Repository '{}' has invalid structure, missing: {}",
                repository,
                missing_items.join(", ")
            ),
            ConfigurationError::FileNotFound { file_path, repository } => {
                write!(f, "File '{}' not found in repository '{}'", file_path, repository)
            }
            ConfigurationError::ParseError {
                file_path,
                repository,
                error,
            } => write!(
                f,
                "Failed to parse file '{}' in repository '{}': {}",
                file_path, repository, error
            ),
            ConfigurationError::ValidationError {
                file_path,
                repository,
                errors,
            } => write!(
                f,
                "Validation failed for file '{}' in repository '{}': {}",
                file_path,
                repository,
                errors.join("; ")
            ),
        }
    }
}

impl std::error::Error for ConfigurationError {}

use thiserror::Error;

/// Configuration validation and merging errors with thiserror support.
///
/// This is an alternative error type that uses the thiserror crate for better
/// error handling and Display trait implementation. It provides the same error
/// variants as ConfigurationError but with thiserror's automatic implementations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Cannot override field '{field}': attempted to set '{attempted_value}' but global default '{global_value}' is fixed")]
    OverrideNotAllowed {
        field: String,
        attempted_value: String,
        global_value: String,
    },

    #[error("Invalid value '{value}' for field '{field}': {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Required field '{field}' is missing in {context}")]
    RequiredFieldMissing { field: String, context: String },

    #[error("Format error in file '{file}': {error}")]
    FormatError { file: String, error: String },

    #[error(
        "Metadata repository not found for organization '{organization}' using {search_method}"
    )]
    RepositoryNotFound {
        organization: String,
        search_method: String,
    },

    #[error("Multiple metadata repositories found for organization '{organization}' using {search_method}: {}", repositories.join(", "))]
    MultipleRepositoriesFound {
        organization: String,
        repositories: Vec<String>,
        search_method: String,
    },

    #[error("Access denied to repository '{repository}' for operation: {operation}")]
    AccessDenied {
        repository: String,
        operation: String,
    },

    #[error("Network error during {operation}: {error}")]
    NetworkError { error: String, operation: String },

    #[error("Repository '{repository}' has invalid structure, missing: {}", missing_items.join(", "))]
    InvalidRepositoryStructure {
        repository: String,
        missing_items: Vec<String>,
    },

    #[error("File '{file_path}' not found in repository '{repository}'")]
    FileNotFound {
        file_path: String,
        repository: String,
    },

    #[error("Failed to parse file '{file_path}' in repository '{repository}': {error}")]
    ParseError {
        file_path: String,
        repository: String,
        error: String,
    },

    #[error("Validation failed for file '{file_path}' in repository '{repository}': {}", errors.join("; "))]
    ValidationError {
        file_path: String,
        repository: String,
        errors: Vec<String>,
    },
}
