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
}
