//! Manages loading and accessing application configuration for RepoRoller.
//!
//! This crate is responsible for reading configuration files (e.g., TOML)
//! and providing access to settings like template definitions, standard labels, etc.

use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

// Unit tests will be added in a separate file: lib_tests.rs
#[path = "lib_tests.rs"]
#[cfg(test)]
mod tests;

/// Errors that can occur while loading or accessing configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Error reading the configuration file.
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] io::Error),

    /// Error parsing the TOML configuration content.
    #[error("Failed to parse TOML configuration: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Represents the configuration for a single repository template.
/// (Placeholder - details to be added based on requirements)
#[derive(Deserialize, Debug, Clone)]
pub struct TemplateConfig {
    pub name: String,
    pub source_repo: String,
    // Add other template-specific fields later (e.g., description, variables)
}

/// Represents the overall application configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// List of available repository templates.
    pub templates: Vec<TemplateConfig>,
    // Add other global settings later (e.g., default labels, settings profiles)
}

/// Loads the application configuration from the specified TOML file.
///
/// # Arguments
///
/// * `path` - The path to the configuration file.
///
/// # Errors
///
/// Returns `ConfigError` if the file cannot be read or parsed.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
