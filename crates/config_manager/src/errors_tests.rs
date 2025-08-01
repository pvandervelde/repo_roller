//! Tests for configuration error types.

use crate::errors::*;

#[cfg(test)]
mod configuration_error_tests {
    use super::*;

    #[test]
    fn configuration_error_override_not_allowed() {
        let error = ConfigurationError::OverrideNotAllowed {
            field: "repository.name".to_string(),
            attempted_value: "new-name".to_string(),
            global_value: "fixed-name".to_string(),
        };

        match error {
            ConfigurationError::OverrideNotAllowed {
                field,
                attempted_value,
                global_value,
            } => {
                assert_eq!(field, "repository.name");
                assert_eq!(attempted_value, "new-name");
                assert_eq!(global_value, "fixed-name");
            }
            _ => panic!("Expected OverrideNotAllowed"),
        }
    }

    #[test]
    fn configuration_error_invalid_value() {
        let error = ConfigurationError::InvalidValue {
            field: "repository.visibility".to_string(),
            value: "invalid".to_string(),
            reason: "Must be one of: public, private, internal".to_string(),
        };

        match error {
            ConfigurationError::InvalidValue {
                field,
                value,
                reason,
            } => {
                assert_eq!(field, "repository.visibility");
                assert_eq!(value, "invalid");
                assert_eq!(reason, "Must be one of: public, private, internal");
            }
            _ => panic!("Expected InvalidValue"),
        }
    }

    #[test]
    fn configuration_error_required_field_missing() {
        let error = ConfigurationError::RequiredFieldMissing {
            field: "template.repository_type".to_string(),
            context: "Template configuration must specify repository type".to_string(),
        };

        match error {
            ConfigurationError::RequiredFieldMissing { field, context } => {
                assert_eq!(field, "template.repository_type");
                assert_eq!(
                    context,
                    "Template configuration must specify repository type"
                );
            }
            _ => panic!("Expected RequiredFieldMissing"),
        }
    }

    #[test]
    fn configuration_error_format_error() {
        let error = ConfigurationError::FormatError {
            file: "template.yaml".to_string(),
            error: "Invalid YAML syntax at line 5".to_string(),
        };

        match error {
            ConfigurationError::FormatError {
                file,
                error: err_msg,
            } => {
                assert_eq!(file, "template.yaml");
                assert_eq!(err_msg, "Invalid YAML syntax at line 5");
            }
            _ => panic!("Expected FormatError"),
        }
    }

    #[test]
    fn configuration_error_debug() {
        let error = ConfigurationError::InvalidValue {
            field: "test.field".to_string(),
            value: "invalid".to_string(),
            reason: "Test reason".to_string(),
        };

        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("InvalidValue"));
        assert!(debug_string.contains("test.field"));
        assert!(debug_string.contains("invalid"));
    }
}
