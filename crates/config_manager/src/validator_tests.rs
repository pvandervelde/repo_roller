//! Tests for validator types and trait.

use super::*;

// ============================================================================
// ValidationResult Tests
// ============================================================================

/// Verify new ValidationResult is valid by default.
#[test]
fn test_validation_result_new_is_valid() {
    let result = ValidationResult::new();
    assert!(result.is_valid());
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

/// Verify Default implementation works.
#[test]
fn test_validation_result_default() {
    let result = ValidationResult::default();
    assert!(result.is_valid());
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

/// Verify validation result is invalid when errors exist.
#[test]
fn test_validation_result_invalid_with_errors() {
    let mut result = ValidationResult::new();
    result.add_error(ValidationError {
        error_type: ValidationErrorType::InvalidValue,
        field_path: "test.field".to_string(),
        message: "Test error".to_string(),
        suggestion: None,
    });

    assert!(!result.is_valid());
    assert_eq!(result.errors.len(), 1);
}

/// Verify validation result remains valid with only warnings.
#[test]
fn test_validation_result_valid_with_warnings() {
    let mut result = ValidationResult::new();
    result.add_warning(ValidationWarning {
        field_path: "test.field".to_string(),
        message: "Test warning".to_string(),
        recommendation: None,
    });

    assert!(result.is_valid());
    assert_eq!(result.warnings.len(), 1);
}

/// Verify multiple errors can be tracked.
#[test]
fn test_validation_result_tracks_multiple_errors() {
    let mut result = ValidationResult::new();

    result.add_error(ValidationError {
        error_type: ValidationErrorType::RequiredFieldMissing,
        field_path: "field1".to_string(),
        message: "Error 1".to_string(),
        suggestion: None,
    });

    result.add_error(ValidationError {
        error_type: ValidationErrorType::InvalidValue,
        field_path: "field2".to_string(),
        message: "Error 2".to_string(),
        suggestion: None,
    });

    assert!(!result.is_valid());
    assert_eq!(result.errors.len(), 2);
}

/// Verify add_errors batch operation works.
#[test]
fn test_validation_result_add_errors_batch() {
    let mut result = ValidationResult::new();

    let errors = vec![
        ValidationError {
            error_type: ValidationErrorType::SchemaViolation,
            field_path: "field1".to_string(),
            message: "Error 1".to_string(),
            suggestion: None,
        },
        ValidationError {
            error_type: ValidationErrorType::BusinessRuleViolation,
            field_path: "field2".to_string(),
            message: "Error 2".to_string(),
            suggestion: None,
        },
    ];

    result.add_errors(errors);
    assert_eq!(result.errors.len(), 2);
}

/// Verify add_warnings batch operation works.
#[test]
fn test_validation_result_add_warnings_batch() {
    let mut result = ValidationResult::new();

    let warnings = vec![
        ValidationWarning {
            field_path: "field1".to_string(),
            message: "Warning 1".to_string(),
            recommendation: None,
        },
        ValidationWarning {
            field_path: "field2".to_string(),
            message: "Warning 2".to_string(),
            recommendation: None,
        },
    ];

    result.add_warnings(warnings);
    assert_eq!(result.warnings.len(), 2);
}

// ============================================================================
// ValidationError Tests
// ============================================================================

/// Verify ValidationError can be created with all fields.
#[test]
fn test_validation_error_creation() {
    let error = ValidationError {
        error_type: ValidationErrorType::InvalidValue,
        field_path: "repository.issues".to_string(),
        message: "Issues setting is invalid".to_string(),
        suggestion: Some("Set to true or false".to_string()),
    };

    assert_eq!(error.error_type, ValidationErrorType::InvalidValue);
    assert_eq!(error.field_path, "repository.issues");
    assert_eq!(error.message, "Issues setting is invalid");
    assert_eq!(error.suggestion, Some("Set to true or false".to_string()));
}

/// Verify ValidationError with no suggestion.
#[test]
fn test_validation_error_without_suggestion() {
    let error = ValidationError {
        error_type: ValidationErrorType::SchemaViolation,
        field_path: "test".to_string(),
        message: "Schema error".to_string(),
        suggestion: None,
    };

    assert!(error.suggestion.is_none());
}

/// Verify ValidationError equality.
#[test]
fn test_validation_error_equality() {
    let error1 = ValidationError {
        error_type: ValidationErrorType::RequiredFieldMissing,
        field_path: "field".to_string(),
        message: "Missing".to_string(),
        suggestion: None,
    };

    let error2 = ValidationError {
        error_type: ValidationErrorType::RequiredFieldMissing,
        field_path: "field".to_string(),
        message: "Missing".to_string(),
        suggestion: None,
    };

    assert_eq!(error1, error2);
}

// ============================================================================
// ValidationErrorType Tests
// ============================================================================

/// Verify all ValidationErrorType variants can be created.
#[test]
fn test_validation_error_type_variants() {
    let types = [
        ValidationErrorType::SchemaViolation,
        ValidationErrorType::RequiredFieldMissing,
        ValidationErrorType::InvalidValue,
        ValidationErrorType::BusinessRuleViolation,
        ValidationErrorType::OverrideNotAllowed,
    ];

    assert_eq!(types.len(), 5);
}

/// Verify ValidationErrorType equality and hashing.
#[test]
fn test_validation_error_type_equality() {
    let type1 = ValidationErrorType::InvalidValue;
    let type2 = ValidationErrorType::InvalidValue;
    let type3 = ValidationErrorType::SchemaViolation;

    assert_eq!(type1, type2);
    assert_ne!(type1, type3);
}

// ============================================================================
// ValidationWarning Tests
// ============================================================================

/// Verify ValidationWarning can be created with all fields.
#[test]
fn test_validation_warning_creation() {
    let warning = ValidationWarning {
        field_path: "webhooks[0].url".to_string(),
        message: "Using HTTP instead of HTTPS".to_string(),
        recommendation: Some("Use HTTPS for secure delivery".to_string()),
    };

    assert_eq!(warning.field_path, "webhooks[0].url");
    assert_eq!(warning.message, "Using HTTP instead of HTTPS");
    assert_eq!(
        warning.recommendation,
        Some("Use HTTPS for secure delivery".to_string())
    );
}

/// Verify ValidationWarning without recommendation.
#[test]
fn test_validation_warning_without_recommendation() {
    let warning = ValidationWarning {
        field_path: "test".to_string(),
        message: "Test warning".to_string(),
        recommendation: None,
    };

    assert!(warning.recommendation.is_none());
}

/// Verify ValidationWarning equality.
#[test]
fn test_validation_warning_equality() {
    let warning1 = ValidationWarning {
        field_path: "field".to_string(),
        message: "Warning".to_string(),
        recommendation: None,
    };

    let warning2 = ValidationWarning {
        field_path: "field".to_string(),
        message: "Warning".to_string(),
        recommendation: None,
    };

    assert_eq!(warning1, warning2);
}
