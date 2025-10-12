use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Errors that can occur in the repo roller core functionality.
///
/// This enum represents all the different types of errors that can happen
/// during repository creation, template processing, and related operations.
///
/// TODO: Expand this to full error hierarchy from specs/interfaces/error-types.md
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred during a git operation.
    ///
    /// This includes failures in git commands such as clone, commit, push, etc.
    #[error("Git operation failed: {0}")]
    GitOperation(String),

    /// An error occurred during a file system operation.
    ///
    /// This includes failures in reading, writing, creating, or deleting files and directories.
    #[error("File system operation failed: {0}")]
    FileSystem(String),

    /// An error occurred during template processing.
    ///
    /// This includes failures in variable substitution, file filtering, or other
    /// template-related operations.
    #[error("Template processing failed: {0}")]
    TemplateProcessing(String),
}

/// Validation errors for domain types
///
/// See specs/interfaces/error-types.md#validationerror for complete specification
///
/// TODO: Move to comprehensive error hierarchy when implementing full error system
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Field '{field}' cannot be empty")]
    EmptyField { field: String },

    #[error("Field '{field}' is too long: {actual} characters (max: {max})")]
    TooLong {
        field: String,
        actual: usize,
        max: usize,
    },

    #[error("Field '{field}' is too short: {actual} characters (min: {min})")]
    TooShort {
        field: String,
        actual: usize,
        min: usize,
    },

    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },
}

impl ValidationError {
    pub fn empty_field(field: impl Into<String>) -> Self {
        Self::EmptyField {
            field: field.into(),
        }
    }

    pub fn too_long(field: impl Into<String>, actual: usize, max: usize) -> Self {
        Self::TooLong {
            field: field.into(),
            actual,
            max,
        }
    }

    pub fn too_short(field: impl Into<String>, actual: usize, min: usize) -> Self {
        Self::TooShort {
            field: field.into(),
            actual,
            min,
        }
    }

    pub fn invalid_format(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            field: field.into(),
            reason: reason.into(),
        }
    }
}
