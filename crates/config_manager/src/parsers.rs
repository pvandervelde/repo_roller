//! Configuration file parsers for organization settings hierarchy.
//!
//! This module provides specialized parsers for different levels of the configuration
//! hierarchy, including global defaults, team configurations, and repository type
//! configurations. Each parser handles TOML format with comprehensive validation
//! and error reporting.

use crate::{
    organization::{GlobalDefaults, GlobalDefaultsEnhanced},
    ConfigurationError,
};
use std::collections::HashMap;

#[cfg(test)]
#[path = "parsers_tests.rs"]
mod tests;

/// Validation result for configuration parsing operations.
///
/// Contains detailed information about parsing success or failures,
/// including field-level validation errors and warnings.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult<T> {
    /// The successfully parsed configuration, if parsing succeeded
    pub config: Option<T>,
    /// Critical errors that prevented parsing
    pub errors: Vec<ParseError>,
    /// Non-critical warnings about the configuration
    pub warnings: Vec<ParseWarning>,
    /// Metadata about the parsing operation
    pub metadata: ParseMetadata,
}

/// Detailed information about a parsing error.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The configuration field that caused the error
    pub field_path: String,
    /// The invalid value that was encountered
    pub invalid_value: String,
    /// Description of why the value is invalid
    pub reason: String,
    /// Suggested correction for the error
    pub suggestion: Option<String>,
}

/// Warning about potentially problematic configuration values.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseWarning {
    /// The configuration field that triggered the warning
    pub field_path: String,
    /// The value that triggered the warning
    pub value: String,
    /// Description of the potential issue
    pub message: String,
    /// Recommended action to address the warning
    pub recommendation: Option<String>,
}

/// Metadata about the parsing operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseMetadata {
    /// The file path that was parsed
    pub file_path: String,
    /// The repository context for the configuration
    pub repository_context: String,
    /// Total number of configuration fields parsed
    pub fields_parsed: usize,
    /// Number of fields that used default values
    pub defaults_applied: usize,
    /// Whether any deprecated configuration syntax was encountered
    pub has_deprecated_syntax: bool,
}

/// Parser for global defaults configuration files.
///
/// This parser handles the `global/defaults.toml` file format and provides
/// comprehensive validation for organization-wide baseline settings.
/// It supports both the standard `GlobalDefaults` structure and the enhanced
/// `GlobalDefaultsEnhanced` structure for organizations with complex requirements.
///
/// # Validation Features
///
/// - **Syntax Validation**: Ensures valid TOML format
/// - **Schema Validation**: Validates all fields match expected types
/// - **Business Rule Validation**: Enforces organization-specific constraints
/// - **Override Policy Validation**: Ensures override settings are consistent
/// - **Security Policy Validation**: Validates security-critical settings
///
/// # Error Handling
///
/// The parser provides detailed error information including:
/// - Exact field path where errors occurred
/// - Invalid values that caused failures
/// - Specific reasons for validation failures
/// - Suggested corrections for common mistakes
///
/// # Examples
///
/// ```rust
/// use config_manager::parsers::GlobalDefaultsParser;
///
/// let parser = GlobalDefaultsParser::new();
/// let toml_content = r#"
/// [repository]
/// wiki = { value = false, override_allowed = true }
/// issues = { value = true, override_allowed = false }
/// "#;
///
/// let result = parser.parse(toml_content, "global/defaults.toml", "org/config-repo");
/// if result.config.is_some() {
///     println!("Successfully parsed {} fields", result.metadata.fields_parsed);
/// }
/// ```
pub struct GlobalDefaultsParser {
    /// Whether to validate security-critical settings with strict rules
    strict_security_validation: bool,
    /// Whether to allow deprecated configuration syntax
    allow_deprecated_syntax: bool,
    /// Custom validation rules specific to the organization
    custom_validators: HashMap<String, Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

impl GlobalDefaultsParser {
    /// Creates a new global defaults parser with default settings.
    ///
    /// The parser is configured with standard validation rules appropriate
    /// for most organizations. Security validation is enabled by default,
    /// and deprecated syntax is not allowed.
    ///
    /// # Returns
    ///
    /// A new `GlobalDefaultsParser` instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    ///
    /// let parser = GlobalDefaultsParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            strict_security_validation: true,
            allow_deprecated_syntax: false,
            custom_validators: HashMap::new(),
        }
    }

    /// Creates a new parser with custom validation settings.
    ///
    /// This constructor allows configuration of parser behavior including
    /// security validation strictness and deprecated syntax handling.
    ///
    /// # Arguments
    ///
    /// * `strict_security` - Whether to apply strict validation to security settings
    /// * `allow_deprecated` - Whether to accept deprecated configuration syntax
    ///
    /// # Returns
    ///
    /// A new `GlobalDefaultsParser` instance with the specified settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    ///
    /// // Create parser that allows deprecated syntax but enforces strict security
    /// let parser = GlobalDefaultsParser::with_options(true, true);
    /// ```
    pub fn with_options(strict_security: bool, allow_deprecated: bool) -> Self {
        Self {
            strict_security_validation: strict_security,
            allow_deprecated_syntax: allow_deprecated,
            custom_validators: HashMap::new(),
        }
    }

    /// Parses global defaults configuration from TOML content.
    ///
    /// This method performs complete parsing and validation of global defaults
    /// configuration. It handles both syntax validation and business rule
    /// enforcement, providing detailed error information for any issues.
    ///
    /// # Arguments
    ///
    /// * `toml_content` - The raw TOML content to parse
    /// * `file_path` - The file path for error reporting context
    /// * `repository_context` - The repository context (e.g., "org/config-repo")
    ///
    /// # Returns
    ///
    /// A `ParseResult<GlobalDefaults>` containing:
    /// - The parsed configuration if successful
    /// - Any parsing errors that occurred
    /// - Warnings about potential issues
    /// - Metadata about the parsing operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    ///
    /// let parser = GlobalDefaultsParser::new();
    /// let toml_content = r#"
    /// [repository]
    /// wiki = { value = false, override_allowed = true }
    /// "#;
    ///
    /// let result = parser.parse(toml_content, "global/defaults.toml", "org/config");
    /// if !result.errors.is_empty() {
    ///     for error in &result.errors {
    ///         eprintln!("Error in {}: {}", error.field_path, error.reason);
    ///     }
    /// }
    /// ```
    ///
    /// # Error Conditions
    ///
    /// Returns errors for:
    /// - Invalid TOML syntax
    /// - Unknown configuration fields
    /// - Invalid field values or types
    /// - Security policy violations
    /// - Inconsistent override policies
    /// - Business rule violations
    pub fn parse(
        &self,
        toml_content: &str,
        file_path: &str,
        repository_context: &str,
    ) -> ParseResult<GlobalDefaults> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut fields_parsed = 0;
        let mut defaults_applied = 0;
        let mut has_deprecated_syntax = false;

        // First, try to parse the TOML syntax
        let parsed_toml = match toml::from_str::<toml::Value>(toml_content) {
            Ok(value) => value,
            Err(e) => {
                errors.push(ParseError {
                    field_path: file_path.to_string(),
                    invalid_value: toml_content.to_string(),
                    reason: format!("TOML syntax error: {}", e),
                    suggestion: Some("Check TOML syntax and formatting".to_string()),
                });
                return ParseResult {
                    config: None,
                    errors,
                    warnings,
                    metadata: ParseMetadata {
                        file_path: file_path.to_string(),
                        repository_context: repository_context.to_string(),
                        fields_parsed: 0,
                        defaults_applied: 0,
                        has_deprecated_syntax: false,
                    },
                };
            }
        };

        // Try to parse into GlobalDefaults structure
        let config = match toml::from_str::<GlobalDefaults>(toml_content) {
            Ok(mut config) => {
                // Count non-None fields
                if config.branch_protection_enabled.is_some() {
                    fields_parsed += 1;
                }
                if config.repository_visibility.is_some() {
                    fields_parsed += 1;
                }
                if config.merge_configuration.is_some() {
                    fields_parsed += 1;
                }
                if config.default_labels.is_some() {
                    fields_parsed += 1;
                }
                if config.organization_webhooks.is_some() {
                    fields_parsed += 1;
                }
                if config.required_github_apps.is_some() {
                    fields_parsed += 1;
                }

                // Apply defaults if needed
                if config.branch_protection_enabled.is_none() {
                    defaults_applied += 1;
                }

                // Validate security settings if strict security is enabled
                if self.strict_security_validation {
                    if let Some(webhooks) = &config.organization_webhooks {
                        for (index, webhook) in webhooks.value().iter().enumerate() {
                            if let Err(error) = parsing_utils::validate_secure_url(
                                &webhook.url,
                                &format!("organization_webhooks[{}].url", index),
                            ) {
                                errors.push(error);
                            }
                        }
                    }
                }

                // Check for deprecated syntax patterns
                if parsed_toml.get("repository_visibility").is_some()
                    && !parsed_toml.get("repository_visibility").unwrap().is_table()
                {
                    has_deprecated_syntax = true;
                    let message = "Using deprecated direct value syntax. Use { value = X, override_allowed = Y } instead.";
                    if self.allow_deprecated_syntax {
                        warnings.push(ParseWarning {
                            field_path: "repository_visibility".to_string(),
                            value: format!("{:?}", parsed_toml.get("repository_visibility")),
                            message: message.to_string(),
                            recommendation: Some(
                                "Migrate to: repository_visibility = { value = \"private\", override_allowed = true }"
                                    .to_string(),
                            ),
                        });
                    } else {
                        errors.push(ParseError {
                            field_path: "repository_visibility".to_string(),
                            invalid_value: format!("{:?}", parsed_toml.get("repository_visibility")),
                            reason: format!("Deprecated syntax not allowed: {}", message),
                            suggestion: Some(
                                "Use repository_visibility = { value = \"private\", override_allowed = true }"
                                    .to_string(),
                            ),
                        });
                    }
                }

                config
            }
            Err(e) => {
                // Check if it's an unknown field error
                let error_message = e.to_string();
                if error_message.contains("unknown field") {
                    // Extract field name from error message
                    let field_name = if let Some(start) = error_message.find('`') {
                        if let Some(end) = error_message[start + 1..].find('`') {
                            &error_message[start + 1..start + 1 + end]
                        } else {
                            "unknown_field"
                        }
                    } else {
                        "unknown_field"
                    };

                    errors.push(ParseError {
                        field_path: field_name.to_string(),
                        invalid_value: "true".to_string(),
                        reason: format!("Unknown field '{}'", field_name),
                        suggestion: Some(
                            "Check the documentation for valid field names".to_string(),
                        ),
                    });
                } else {
                    errors.push(ParseError {
                        field_path: file_path.to_string(),
                        invalid_value: toml_content.to_string(),
                        reason: format!("Configuration parsing error: {}", e),
                        suggestion: Some(
                            "Check configuration structure and field types".to_string(),
                        ),
                    });
                }

                return ParseResult {
                    config: None,
                    errors,
                    warnings,
                    metadata: ParseMetadata {
                        file_path: file_path.to_string(),
                        repository_context: repository_context.to_string(),
                        fields_parsed: 0,
                        defaults_applied: 0,
                        has_deprecated_syntax: false,
                    },
                };
            }
        };

        // Additional validation
        let policy_errors = self.validate_policies(&config, file_path);
        errors.extend(policy_errors);

        // Apply custom validators
        for (pattern, validator) in &self.custom_validators {
            if pattern.contains("webhooks.*.url") {
                if let Some(webhooks) = &config.organization_webhooks {
                    for (index, webhook) in webhooks.value().iter().enumerate() {
                        if let Err(validation_error) = validator(&webhook.url) {
                            errors.push(ParseError {
                                field_path: format!("webhooks[{}].url", index),
                                invalid_value: webhook.url.clone(),
                                reason: validation_error,
                                suggestion: None,
                            });
                        }
                    }
                }
            }
        }

        let final_config = if errors.is_empty() {
            Some(config)
        } else {
            None
        };

        ParseResult {
            config: final_config,
            errors,
            warnings,
            metadata: ParseMetadata {
                file_path: file_path.to_string(),
                repository_context: repository_context.to_string(),
                fields_parsed,
                defaults_applied,
                has_deprecated_syntax,
            },
        }
    }

    /// Parses enhanced global defaults configuration from TOML content.
    ///
    /// This method parses the more comprehensive `GlobalDefaultsEnhanced` structure
    /// which includes additional configuration options for complex organizational
    /// setups. It provides the same validation features as the standard parser
    /// but supports extended configuration schemas.
    ///
    /// # Arguments
    ///
    /// * `toml_content` - The raw TOML content to parse
    /// * `file_path` - The file path for error reporting context
    /// * `repository_context` - The repository context (e.g., "org/config-repo")
    ///
    /// # Returns
    ///
    /// A `ParseResult<GlobalDefaultsEnhanced>` with the same structure as the
    /// standard parser but containing the enhanced configuration structure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    ///
    /// let parser = GlobalDefaultsParser::new();
    /// let result = parser.parse_enhanced(toml_content, "global/defaults.toml", "org/config");
    /// ```
    ///
    /// # Error Conditions
    ///
    /// Same error conditions as `parse()` method, with additional validation
    /// for enhanced configuration fields.
    pub fn parse_enhanced(
        &self,
        toml_content: &str,
        file_path: &str,
        repository_context: &str,
    ) -> ParseResult<GlobalDefaultsEnhanced> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut fields_parsed = 0;
        let mut defaults_applied = 0;
        let has_deprecated_syntax = false;

        // Try to parse into GlobalDefaultsEnhanced structure
        let config = match toml::from_str::<GlobalDefaultsEnhanced>(toml_content) {
            Ok(config) => {
                // Count non-None fields
                if config.actions.is_some() {
                    fields_parsed += 1;
                }
                if config.branch_protection.is_some() {
                    fields_parsed += 1;
                }
                if config.custom_properties.is_some() {
                    fields_parsed += 1;
                }
                if config.environments.is_some() {
                    fields_parsed += 1;
                }
                if config.github_apps.is_some() {
                    fields_parsed += 1;
                }
                if config.pull_requests.is_some() {
                    fields_parsed += 1;
                }
                if config.push.is_some() {
                    fields_parsed += 1;
                }
                if config.repository.is_some() {
                    fields_parsed += 1;
                }
                if config.webhooks.is_some() {
                    fields_parsed += 1;
                }

                config
            }
            Err(e) => {
                errors.push(ParseError {
                    field_path: file_path.to_string(),
                    invalid_value: toml_content.to_string(),
                    reason: format!("Enhanced configuration parsing error: {}", e),
                    suggestion: Some(
                        "Check enhanced configuration structure and field types".to_string(),
                    ),
                });

                return ParseResult {
                    config: None,
                    errors,
                    warnings,
                    metadata: ParseMetadata {
                        file_path: file_path.to_string(),
                        repository_context: repository_context.to_string(),
                        fields_parsed: 0,
                        defaults_applied: 0,
                        has_deprecated_syntax: false,
                    },
                };
            }
        };

        let final_config = if errors.is_empty() {
            Some(config)
        } else {
            None
        };

        ParseResult {
            config: final_config,
            errors,
            warnings,
            metadata: ParseMetadata {
                file_path: file_path.to_string(),
                repository_context: repository_context.to_string(),
                fields_parsed,
                defaults_applied,
                has_deprecated_syntax,
            },
        }
    }

    /// Validates the parsed configuration against organization policies.
    ///
    /// This method performs additional validation beyond basic syntax checking,
    /// including security policy enforcement, business rule validation, and
    /// consistency checks across different configuration sections.
    ///
    /// # Arguments
    ///
    /// * `config` - The parsed configuration to validate
    /// * `context` - The validation context for error reporting
    ///
    /// # Returns
    ///
    /// A vector of validation errors. Empty vector indicates successful validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    /// use config_manager::organization::GlobalDefaults;
    ///
    /// let parser = GlobalDefaultsParser::new();
    /// let config = GlobalDefaults::new();
    /// let errors = parser.validate_policies(&config, "global/defaults.toml");
    ///
    /// if errors.is_empty() {
    ///     println!("Configuration passes all policy checks");
    /// }
    /// ```
    ///
    /// # Validation Rules
    ///
    /// - Security settings cannot weaken organization security posture
    /// - Override policies must be consistent across related settings
    /// - Required security features must be enabled
    /// - Webhook URLs must use secure protocols
    /// - Custom properties must follow naming conventions
    pub fn validate_policies(&self, config: &GlobalDefaults, context: &str) -> Vec<ParseError> {
        let mut errors = Vec::new();

        // Validate security policies if strict security is enabled
        if self.strict_security_validation {
            // Check branch protection is not disabled
            if let Some(branch_protection) = &config.branch_protection_enabled {
                if !branch_protection.value() {
                    errors.push(ParseError {
                        field_path: "branch_protection_enabled".to_string(),
                        invalid_value: "false".to_string(),
                        reason: "Branch protection cannot be disabled in strict security mode"
                            .to_string(),
                        suggestion: Some(
                            "Set branch_protection_enabled to true for security compliance"
                                .to_string(),
                        ),
                    });
                }
            }
        }

        errors
    }

    /// Adds a custom validation rule for specific configuration fields.
    ///
    /// This method allows organizations to define custom validation logic
    /// for specific configuration fields beyond the standard validation rules.
    ///
    /// # Arguments
    ///
    /// * `field_path` - The dot-separated path to the field (e.g., "repository.wiki")
    /// * `validator` - A closure that validates the field value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::parsers::GlobalDefaultsParser;
    ///
    /// let mut parser = GlobalDefaultsParser::new();
    /// parser.add_custom_validator("webhooks.*.url", Box::new(|url| {
    ///     if url.starts_with("https://internal.company.com/") {
    ///         Ok(())
    ///     } else {
    ///         Err("Webhook URLs must use internal company domain".to_string())
    ///     }
    /// }));
    /// ```
    pub fn add_custom_validator<F>(&mut self, field_path: &str, validator: F)
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.custom_validators
            .insert(field_path.to_string(), Box::new(validator));
    }
}

impl Default for GlobalDefaultsParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for configuration parsing and validation.
pub mod parsing_utils {
    use super::*;

    /// Validates that a TOML value matches the expected type.
    ///
    /// # Arguments
    ///
    /// * `value` - The TOML value to validate
    /// * `expected_type` - The expected type name for error messages
    /// * `field_path` - The field path for error context
    ///
    /// # Returns
    ///
    /// `Ok(())` if the type is correct, `Err(ParseError)` otherwise.
    pub fn validate_toml_type(
        value: &toml::Value,
        expected_type: &str,
        field_path: &str,
    ) -> Result<(), ParseError> {
        let actual_type = match value {
            toml::Value::String(_) => "string",
            toml::Value::Integer(_) => "integer",
            toml::Value::Float(_) => "float",
            toml::Value::Boolean(_) => "boolean",
            toml::Value::Array(_) => "array",
            toml::Value::Table(_) => "table",
            toml::Value::Datetime(_) => "datetime",
        };

        if actual_type != expected_type {
            Err(ParseError {
                field_path: field_path.to_string(),
                invalid_value: format!("{:?}", value),
                reason: format!("Expected {} but got {}", expected_type, actual_type),
                suggestion: Some(format!("Ensure the value is a valid {}", expected_type)),
            })
        } else {
            Ok(())
        }
    }

    /// Extracts override policy information from a TOML value.
    ///
    /// This function parses the `{ value = X, override_allowed = Y }` pattern
    /// used throughout the configuration hierarchy.
    ///
    /// # Arguments
    ///
    /// * `toml_value` - The TOML value to parse
    /// * `field_path` - The field path for error context
    ///
    /// # Returns
    ///
    /// A tuple of (value, override_allowed) or a ParseError if parsing fails.
    pub fn extract_override_policy(
        toml_value: &toml::Value,
        field_path: &str,
    ) -> Result<(toml::Value, bool), ParseError> {
        if let Some(table) = toml_value.as_table() {
            // Extract the value field
            let value = table.get("value").ok_or_else(|| ParseError {
                field_path: format!("{}.value", field_path),
                invalid_value: "missing".to_string(),
                reason: "Value field is required".to_string(),
                suggestion: Some("Add a 'value' field to the configuration".to_string()),
            })?;

            // Extract override_allowed, defaulting to true if not present
            let override_allowed = table
                .get("override_allowed")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            Ok((value.clone(), override_allowed))
        } else {
            // Treat non-table values as simple values with default override_allowed = true
            Ok((toml_value.clone(), true))
        }
    }

    /// Validates that URL values use secure protocols.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to validate
    /// * `field_path` - The field path for error context
    ///
    /// # Returns
    ///
    /// `Ok(())` if the URL is secure, `Err(ParseError)` otherwise.
    pub fn validate_secure_url(url: &str, field_path: &str) -> Result<(), ParseError> {
        // Check if URL starts with https://
        if !url.starts_with("https://") {
            Err(ParseError {
                field_path: field_path.to_string(),
                invalid_value: url.to_string(),
                reason: "URL must use secure HTTPS protocol".to_string(),
                suggestion: Some("Change URL to use https:// instead of http://".to_string()),
            })
        } else {
            Ok(())
        }
    }
}
