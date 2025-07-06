use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Error types that can occur during template processing operations.
///
/// This enum represents all possible error conditions that can arise during
/// template processing, including I/O failures, validation errors, and
/// content encoding issues.
///
/// # Examples
///
/// ```rust,ignore
/// use template_engine::Error;
///
/// // Handle different error types
/// match some_template_operation() {
///     Ok(result) => println!("Success: {:?}", result),
///     Err(Error::RequiredVariableMissing(var)) => {
///         eprintln!("Missing required variable: {}", var);
///     },
///     Err(Error::PatternValidationFailed { variable, pattern }) => {
///         eprintln!("Variable '{}' doesn't match pattern '{}'", variable, pattern);
///     },
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// # fn some_template_operation() -> Result<(), Error> { Ok(()) }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// I/O operation failed during file reading or writing.
    ///
    /// This error wraps underlying `std::io::Error` instances that occur
    /// when reading template files, writing output files, or performing
    /// other filesystem operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File content contains invalid UTF-8 sequences.
    ///
    /// This error occurs when a file that should contain text (and thus
    /// be processed for variable substitution) contains invalid UTF-8
    /// byte sequences that cannot be decoded to a string.
    #[error("Invalid UTF-8 content in file: {0}")]
    InvalidUtf8(String),

    /// Variable validation failed due to constraint violations.
    ///
    /// This error is returned when a template variable fails validation
    /// against its configured constraints (length, format, etc.).
    #[error("Template variable validation failed: {variable} - {reason}")]
    VariableValidation {
        /// The name of the variable that failed validation
        variable: String,
        /// Description of why the validation failed
        reason: String,
    },

    /// A required template variable was not provided.
    ///
    /// This error occurs when a template variable is marked as required
    /// but no value was provided and no default value is configured.
    #[error("Required variable missing: {0}")]
    RequiredVariableMissing(String),

    /// Variable value doesn't match the required pattern.
    ///
    /// This error is returned when a variable value fails to match
    /// a regex pattern specified in the variable configuration.
    #[error("Variable pattern validation failed: {variable} does not match pattern {pattern}")]
    PatternValidationFailed {
        /// The name of the variable that failed pattern matching
        variable: String,
        /// The regex pattern that the variable should have matched
        pattern: String,
    },
}
