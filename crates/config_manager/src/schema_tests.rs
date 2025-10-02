//! Unit tests for JSON Schema-based configuration validation.
//!
//! This module contains comprehensive tests for the schema validation system,
//! including validation result handling, custom validators, and schema compilation.

use crate::organization::{
    GlobalDefaults, RepositoryTypeConfig, TeamConfig, TemplateConfig, TemplateMetadata,
};
use crate::schema::{
    CustomValidator, SchemaValidator, ValidationIssue, ValidationResult, ValidationSeverity,
};

/// Tests for ValidationIssue creation and accessors.
mod validation_issue_tests {
    use crate::schema::{ValidationIssue, ValidationSeverity};

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Error,
            "repository.wiki.value".to_string(),
            "Field is required".to_string(),
            Some("Add a wiki field".to_string()),
        );

        assert_eq!(issue.severity(), &ValidationSeverity::Error);
        assert_eq!(issue.field_path(), "repository.wiki.value");
        assert_eq!(issue.message(), "Field is required");
        assert_eq!(issue.suggestion(), Some("Add a wiki field"));
        assert_eq!(issue.invalid_value(), None);
        assert_eq!(issue.expected_value(), None);
        assert!(issue.is_error());
        assert!(!issue.is_warning());
        assert!(!issue.is_info());
    }

    #[test]
    fn test_validation_issue_with_values() {
        let issue = ValidationIssue::with_values(
            ValidationSeverity::Warning,
            "webhooks[0].url".to_string(),
            "Invalid URL format".to_string(),
            Some("http://example.com".to_string()),
            Some("https://example.com".to_string()),
            Some("Use HTTPS protocol".to_string()),
        );

        assert_eq!(issue.severity(), &ValidationSeverity::Warning);
        assert_eq!(issue.invalid_value(), Some("http://example.com"));
        assert_eq!(issue.expected_value(), Some("https://example.com"));
        assert!(!issue.is_error());
        assert!(issue.is_warning());
        assert!(!issue.is_info());
    }

    #[test]
    fn test_validation_issue_info_level() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Info,
            "labels.count".to_string(),
            "Consider adding more descriptive labels".to_string(),
            None,
        );

        assert_eq!(issue.severity(), &ValidationSeverity::Info);
        assert!(!issue.is_error());
        assert!(!issue.is_warning());
        assert!(issue.is_info());
    }
}

/// Tests for ValidationResult functionality.
mod validation_result_tests {
    use crate::schema::{ValidationIssue, ValidationResult, ValidationSeverity};

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::new();

        assert!(result.is_valid());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert!(!result.has_info());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
        assert_eq!(result.info_count(), 0);
        assert_eq!(result.total_count(), 0);
        assert!(result.issues().is_empty());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        result.add_error("field.path", "Error message", Some("Fix suggestion"));

        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert!(!result.has_warnings());
        assert!(!result.has_info());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 0);
        assert_eq!(result.info_count(), 0);
        assert_eq!(result.total_count(), 1);

        let errors = result.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field_path(), "field.path");
        assert_eq!(errors[0].message(), "Error message");
        assert_eq!(errors[0].suggestion(), Some("Fix suggestion"));
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();
        result.add_warning("field.path", "Warning message", None);

        assert!(result.is_valid()); // Warnings don't make result invalid
        assert!(!result.has_errors());
        assert!(result.has_warnings());
        assert!(!result.has_info());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(result.info_count(), 0);
        assert_eq!(result.total_count(), 1);

        let warnings = result.warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].field_path(), "field.path");
        assert_eq!(warnings[0].message(), "Warning message");
        assert_eq!(warnings[0].suggestion(), None);
    }

    #[test]
    fn test_validation_result_add_info() {
        let mut result = ValidationResult::new();
        result.add_info("field.path", "Info message", Some("Consider this"));

        assert!(result.is_valid()); // Info messages don't make result invalid
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert!(result.has_info());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
        assert_eq!(result.info_count(), 1);
        assert_eq!(result.total_count(), 1);

        let info_messages = result.info_messages();
        assert_eq!(info_messages.len(), 1);
        assert_eq!(info_messages[0].field_path(), "field.path");
        assert_eq!(info_messages[0].message(), "Info message");
        assert_eq!(info_messages[0].suggestion(), Some("Consider this"));
    }

    #[test]
    fn test_validation_result_mixed_issues() {
        let mut result = ValidationResult::new();
        result.add_error("error.field", "Error", None);
        result.add_warning("warning.field", "Warning", None);
        result.add_info("info.field", "Info", None);

        assert!(!result.is_valid()); // Errors make result invalid
        assert!(result.has_errors());
        assert!(result.has_warnings());
        assert!(result.has_info());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(result.info_count(), 1);
        assert_eq!(result.total_count(), 3);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error("field1", "Error 1", None);

        let mut result2 = ValidationResult::new();
        result2.add_warning("field2", "Warning 2", None);

        result1.merge(result2);

        assert_eq!(result1.error_count(), 1);
        assert_eq!(result1.warning_count(), 1);
        assert_eq!(result1.total_count(), 2);
    }

    #[test]
    fn test_validation_result_add_issue() {
        let mut result = ValidationResult::new();
        let issue = ValidationIssue::new(
            ValidationSeverity::Error,
            "custom.field".to_string(),
            "Custom error".to_string(),
            None,
        );

        result.add_issue(issue);

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.errors()[0].field_path(), "custom.field");
    }
}

/// Tests for CustomValidator functionality.
mod custom_validator_tests {
    use crate::schema::{CustomValidator, ValidationSeverity};

    #[test]
    fn test_custom_validator_creation() {
        let validator = CustomValidator::new(
            "webhooks.*.url".to_string(),
            "URL must use HTTPS".to_string(),
            |url| {
                if url.starts_with("https://") {
                    Ok(())
                } else {
                    Err("URL must use HTTPS protocol".to_string())
                }
            },
        );

        assert_eq!(validator.field_pattern(), "webhooks.*.url");
        assert_eq!(validator.description(), "URL must use HTTPS");

        // Test validation logic
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://example.com").is_err());

        let error = validator.validate("http://example.com").unwrap_err();
        assert_eq!(error, "URL must use HTTPS protocol");
    }

    #[test]
    fn test_custom_validator_complex_pattern() {
        let validator = CustomValidator::new(
            "github_apps.*".to_string(),
            "GitHub App names must follow naming convention".to_string(),
            |app_name| {
                if app_name.starts_with("org-") && app_name.len() > 4 {
                    Ok(())
                } else {
                    Err(
                        "GitHub App name must start with 'org-' and be longer than 4 characters"
                            .to_string(),
                    )
                }
            },
        );

        assert!(validator.validate("org-security-scanner").is_ok());
        assert!(validator.validate("org-ci").is_ok());
        assert!(validator.validate("security-scanner").is_err());
        assert!(validator.validate("org-").is_err());
        assert!(validator.validate("").is_err());
    }
}

/// Tests for SchemaValidator basic functionality.
mod schema_validator_tests {
    use crate::schema::{CustomValidator, SchemaValidator};

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new();

        // Test that default validator is created successfully
        assert_eq!(validator.custom_validator_count(), 0);
        assert!(validator.is_strict_mode());
    }

    #[test]
    fn test_schema_validator_with_strict_mode() {
        let strict_validator = SchemaValidator::with_strict_mode(true);
        let lenient_validator = SchemaValidator::with_strict_mode(false);

        assert!(strict_validator.is_strict_mode());
        assert!(!lenient_validator.is_strict_mode());
    }

    #[test]
    fn test_schema_validator_add_custom_validator() {
        let mut validator = SchemaValidator::new();

        let custom_validator = CustomValidator::new(
            "test.field".to_string(),
            "Test validation".to_string(),
            |_| Ok(()),
        );

        validator.add_custom_validator(custom_validator);

        assert_eq!(validator.custom_validator_count(), 1);
        assert_eq!(
            validator.custom_validators()[0].field_pattern(),
            "test.field"
        );
    }

    #[test]
    fn test_schema_validator_default() {
        let validator = SchemaValidator::default();

        assert_eq!(validator.custom_validator_count(), 0);
        assert!(validator.is_strict_mode());
    }
}

/// Tests for global defaults validation.
mod global_defaults_validation_tests {
    use crate::organization::GlobalDefaults;
    use crate::schema::{CustomValidator, SchemaValidator};

    #[test]
    fn test_validate_empty_global_defaults() {
        let validator = SchemaValidator::new();
        let config = GlobalDefaults::new();

        let result = validator.validate_global_defaults(&config);

        // Empty configuration should be valid (all fields are optional)
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validate_global_defaults_with_custom_validator() {
        let mut validator = SchemaValidator::new();

        // Add a custom validator that always fails for testing
        let custom_validator = CustomValidator::new(
            "organization_webhooks.*.url".to_string(),
            "Test validator".to_string(),
            |_| Err("Custom validation failed".to_string()),
        );
        validator.add_custom_validator(custom_validator);

        let config = GlobalDefaults::new();
        let result = validator.validate_global_defaults(&config);

        // Should still be valid since we have no webhooks to validate
        assert!(result.is_valid());
    }
}

/// Tests for team configuration validation.
mod team_config_validation_tests {
    use crate::organization::TeamConfig;
    use crate::schema::SchemaValidator;

    #[test]
    fn test_validate_empty_team_config() {
        let validator = SchemaValidator::new();
        let config = TeamConfig::new();

        let result = validator.validate_team_config(&config);

        // Empty team configuration should be valid
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }
}

/// Tests for repository type configuration validation.
mod repository_type_config_validation_tests {
    use crate::organization::RepositoryTypeConfig;
    use crate::schema::SchemaValidator;

    #[test]
    fn test_validate_empty_repository_type_config() {
        let validator = SchemaValidator::new();
        let config = RepositoryTypeConfig::new();

        let result = validator.validate_repository_type_config(&config);

        // Empty repository type configuration should be valid
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }
}

/// Tests for template configuration validation.
mod template_config_validation_tests {
    use crate::organization::{TemplateConfig, TemplateMetadata};
    use crate::schema::SchemaValidator;

    #[test]
    fn test_validate_empty_template_config() {
        let validator = SchemaValidator::new();
        let template_metadata = TemplateMetadata::new(
            "test".to_string(),
            "Test template".to_string(),
            "Test Author".to_string(),
            vec!["test".to_string()],
        );
        let config = TemplateConfig::new(template_metadata);

        let result = validator.validate_template_config(&config);

        // Template configuration with basic metadata should be valid
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }
}

/// Tests for schema generation functionality.
mod schema_generation_tests {
    use crate::organization::{GlobalDefaults, RepositoryTypeConfig, TeamConfig, TemplateConfig};
    use crate::schema::SchemaValidator;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct TestConfig {
        name: String,
        #[serde(default)]
        optional_field: Option<String>,
    }

    #[test]
    fn test_generate_schema_success() {
        let validator = SchemaValidator::new();

        let schema_result = validator.generate_schema::<TestConfig>();

        assert!(schema_result.is_ok());
        let schema = schema_result.unwrap();
        assert!(schema.is_object());

        // Verify schema contains expected structure
        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("$schema"));
        assert!(schema_obj.contains_key("title"));
        assert!(schema_obj.contains_key("type"));
        assert_eq!(schema_obj["type"], "object");
    }
}

/// Tests for schema validation error handling.
mod schema_error_tests {
    use crate::schema::{SchemaValidationError, ValidationSeverity};

    #[test]
    fn test_schema_validation_error_display() {
        let error = SchemaValidationError::SchemaGeneration {
            message: "Test error".to_string(),
        };

        assert_eq!(error.to_string(), "Schema generation failed: Test error");
    }

    #[test]
    fn test_schema_validation_error_compilation() {
        let error = SchemaValidationError::SchemaCompilation {
            message: "Invalid schema".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Schema compilation failed: Invalid schema"
        );
    }

    #[test]
    fn test_schema_validation_error_serialization() {
        let error = SchemaValidationError::JsonSerialization {
            message: "Serialization failed".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "JSON serialization failed: Serialization failed"
        );
    }

    #[test]
    fn test_schema_validation_error_validation_failed() {
        let error = SchemaValidationError::ValidationFailed {
            message: "Validation error".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Configuration validation failed: Validation error"
        );
    }

    #[test]
    fn test_schema_validation_error_custom_validation() {
        let error = SchemaValidationError::CustomValidationFailed {
            field_path: "test.field".to_string(),
            message: "Custom error".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Custom validation failed for field 'test.field': Custom error"
        );
    }

    #[test]
    fn test_schema_validation_error_equality() {
        let error1 = SchemaValidationError::SchemaGeneration {
            message: "Test".to_string(),
        };
        let error2 = SchemaValidationError::SchemaGeneration {
            message: "Test".to_string(),
        };
        let error3 = SchemaValidationError::SchemaGeneration {
            message: "Different".to_string(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}

/// Integration tests combining multiple validation features.
mod integration_tests {
    use crate::organization::{GlobalDefaults, RepositoryTypeConfig, TeamConfig, TemplateConfig};
    use crate::schema::{CustomValidator, SchemaValidator, ValidationResult};

    #[test]
    fn test_schema_validator_with_multiple_custom_validators() {
        let mut validator = SchemaValidator::new();

        // Add multiple custom validators
        let url_validator = CustomValidator::new(
            "*.url".to_string(),
            "URLs must be secure".to_string(),
            |url| {
                if url.starts_with("https://") {
                    Ok(())
                } else {
                    Err("URL must use HTTPS".to_string())
                }
            },
        );

        let name_validator = CustomValidator::new(
            "*.name".to_string(),
            "Names must not be empty".to_string(),
            |name| {
                if !name.trim().is_empty() {
                    Ok(())
                } else {
                    Err("Name cannot be empty".to_string())
                }
            },
        );

        validator.add_custom_validator(url_validator);
        validator.add_custom_validator(name_validator);

        assert_eq!(validator.custom_validator_count(), 2);
    }

    #[test]
    fn test_validation_result_comprehensive_workflow() {
        let mut result = ValidationResult::new();

        // Add various types of issues
        result.add_error(
            "config.required_field",
            "Missing required field",
            Some("Add the field"),
        );
        result.add_warning(
            "config.deprecated_field",
            "Field is deprecated",
            Some("Use new_field instead"),
        );
        result.add_info(
            "config.optimization",
            "Consider enabling this feature",
            None,
        );

        // Verify comprehensive state
        assert!(!result.is_valid());
        assert!(result.has_errors());
        assert!(result.has_warnings());
        assert!(result.has_info());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(result.info_count(), 1);
        assert_eq!(result.total_count(), 3);

        // Verify we can access different issue types
        let errors = result.errors();
        let warnings = result.warnings();
        let info_messages = result.info_messages();

        assert_eq!(errors.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(info_messages.len(), 1);

        assert_eq!(errors[0].field_path(), "config.required_field");
        assert_eq!(warnings[0].field_path(), "config.deprecated_field");
        assert_eq!(info_messages[0].field_path(), "config.optimization");
    }
}
