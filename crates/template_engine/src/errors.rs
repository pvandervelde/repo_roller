use thiserror::Error;

/// Error types for template processing
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid UTF-8 content in file: {0}")]
    InvalidUtf8(String),

    #[error("Template variable validation failed: {variable} - {reason}")]
    VariableValidation { variable: String, reason: String },

    #[error("Required variable missing: {0}")]
    RequiredVariableMissing(String),

    #[error("Variable pattern validation failed: {variable} does not match pattern {pattern}")]
    PatternValidationFailed { variable: String, pattern: String },
}
