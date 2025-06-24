use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An unknown error occurred.")]
    Unknown,
    
    #[error("Git operation failed: {0}")]
    GitOperation(String),
    
    #[error("File system operation failed: {0}")]
    FileSystem(String),
    
    #[error("Template processing failed: {0}")]
    TemplateProcessing(String),
}
