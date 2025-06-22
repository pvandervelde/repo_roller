use std::io;

use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Error, Debug)]
pub enum Error {
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Failed to load file.")]
    LoadFile(io::Error),

    #[error("Failed to parse TOML configuration file.")]
    ParseTomlFile(toml::de::Error),

    #[error("Failed to flush the std out buffer.")]
    StdOutFlushFailed,
}
