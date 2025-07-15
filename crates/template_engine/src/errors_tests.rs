use super::*;
use std::io;

#[test]
fn test_error_debug_format() {
    let error = Error::RequiredVariableMissing("test_var".to_string());
    let debug_output = format!("{error:?}");
    assert!(debug_output.contains("RequiredVariableMissing"));
    assert!(debug_output.contains("test_var"));
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Error>();
}

#[test]
fn test_invalid_utf8_error_display() {
    let error = Error::InvalidUtf8("test.txt".to_string());
    assert_eq!(error.to_string(), "Invalid UTF-8 content in file: test.txt");
}

#[test]
fn test_io_error_display() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = Error::Io(io_error);
    assert_eq!(error.to_string(), "IO error: File not found");
}

#[test]
fn test_pattern_validation_failed_error_display() {
    let error = Error::PatternValidationFailed {
        variable: "project_name".to_string(),
        pattern: r"^[a-z]+$".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "Variable pattern validation failed: project_name does not match pattern ^[a-z]+$"
    );
}

#[test]
fn test_required_variable_missing_error_display() {
    let error = Error::RequiredVariableMissing("author".to_string());
    assert_eq!(error.to_string(), "Required variable missing: author");
}

#[test]
fn test_variable_validation_error_display() {
    let error = Error::VariableValidation {
        variable: "email".to_string(),
        reason: "Invalid email format".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "Template variable validation failed: email - Invalid email format"
    );
}
