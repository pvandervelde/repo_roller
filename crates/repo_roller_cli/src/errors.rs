use std::io;

use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to load file.")]
    LoadFile(io::Error),

    #[error("Failed to parse TOML configuration file.")]
    ParseTomlFile(toml::de::Error),

    #[error("Failed to flush the std out buffer.")]
    StdOutFushFailed,
}
