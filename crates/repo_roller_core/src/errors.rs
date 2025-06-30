use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Errors that can occur in the repo roller core functionality.
///
/// This enum represents all the different types of errors that can happen
/// during repository creation, template processing, and related operations.
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
