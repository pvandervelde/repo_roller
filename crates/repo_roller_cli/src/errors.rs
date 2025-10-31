use std::io;

use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Errors that can occur in the RepoRoller CLI application.
///
/// This enum represents all possible error conditions that can arise during
/// CLI operations, including authentication failures, configuration issues,
/// and I/O problems.
#[derive(Error, Debug)]
pub enum Error {
    /// Authentication error occurred during GitHub authentication process.
    ///
    /// This error is returned when authentication with GitHub fails, such as
    /// invalid tokens, expired credentials, or network issues during auth.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Configuration error occurred while loading or parsing configuration.
    ///
    /// This error is returned when there are issues with the configuration file,
    /// such as missing required fields, invalid values, or file access problems.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid command-line arguments were provided.
    ///
    /// This error is returned when the user provides invalid or incompatible
    /// command-line arguments that cannot be processed.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Failed to load a file from the filesystem.
    ///
    /// This error wraps underlying I/O errors that occur when reading files,
    /// such as permission issues or missing files.
    #[error("Failed to load file.")]
    LoadFile(io::Error),

    /// Failed to parse a TOML configuration file.
    ///
    /// This error is returned when the TOML configuration file contains
    /// invalid syntax or structure that cannot be parsed.
    #[error("Failed to parse TOML configuration file.")]
    ParseTomlFile(toml::de::Error),

    /// Failed to flush the standard output buffer.
    ///
    /// This error occurs when the CLI cannot write output to the terminal,
    /// typically due to broken pipes or terminal issues.
    #[error("Failed to flush the std out buffer.")]
    StdOutFlushFailed,

    /// Feature or command not yet implemented.
    ///
    /// This error is returned when a command or feature exists in the CLI
    /// but has not been fully implemented yet.
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
