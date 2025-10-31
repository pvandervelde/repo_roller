//! Configuration management for the RepoRoller CLI.
//!
//! This module provides functionality for loading, saving, and managing
//! configuration files for the RepoRoller CLI application. It handles both
//! core configuration (templates, repository settings) and CLI-specific
//! settings like authentication configuration.
//!
//! The configuration is stored in TOML format and can be loaded from a
//! specified file path or from the default location in the current directory.

use std::{
    fs,
    path::{Path, PathBuf},
};

use config_manager::Config;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::errors::Error;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = "config.toml";

/// Default metadata repository name
pub const DEFAULT_METADATA_REPOSITORY_NAME: &str = ".reporoller";

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

/// Main configuration structure for the RepoRoller CLI application.
///
/// This structure combines core repository and template configuration with
/// CLI-specific settings. It serves as the primary configuration interface
/// for the application and handles serialization to/from TOML format.
///
/// # Structure
///
/// The configuration is divided into two main sections:
/// - Core configuration: Templates, repository settings, and variable definitions
/// - Authentication configuration: CLI-specific authentication settings
///
/// # Example TOML Configuration
///
/// ```toml
/// [authentication]
/// auth_method = "token"
///
/// [[templates]]
/// name = "rust-library"
/// source = "https://github.com/example/rust-template"
/// description = "A Rust library template"
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Core configuration (templates, repository settings, etc.)
    #[serde(flatten)]
    pub core: Config,

/// - CLI-specific configuration
    #[serde(default)]
    pub authentication: AuthenticationConfig,

    /// Organization settings configuration
    #[serde(default)]
    pub organization: OrganizationConfig,
}

impl AppConfig {
    /// Loads configuration from a TOML file at the specified path.
    ///
    /// This method reads and parses a TOML configuration file, deserializing it
    /// into an `AppConfig` instance. The configuration file should contain both
    /// core repository/template settings and CLI-specific authentication settings.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path to the configuration file to load
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(AppConfig)` - Successfully loaded and parsed configuration
    /// - `Err(Error::Config)` - If the file doesn't exist, can't be read, or contains invalid TOML
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The specified file does not exist
    /// - The file cannot be read due to permissions or I/O issues
    /// - The file contains invalid TOML syntax
    /// - The TOML structure doesn't match the expected configuration schema
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use repo_roller_cli::config::AppConfig;
    ///
    /// let config_path = Path::new("./config.toml");
    /// match AppConfig::load(&config_path) {
    ///     Ok(config) => println!("Loaded {} templates", config.core.templates.len()),
    ///     Err(e) => eprintln!("Failed to load config: {}", e),
    /// }
    /// ```
    pub fn load(path: &Path) -> Result<Self, Error> {
        debug!("Loading configuration from {:?}", path);

        if !path.exists() {
            return Err(Error::Config(format!(
                "Configuration file not found: {:?}",
                path
            )));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read configuration file: {}", e)))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse configuration file: {}", e)))?;

        Ok(config)
    }

    /// Saves the configuration to a TOML file at the specified path.
    ///
    /// This method serializes the current `AppConfig` instance to TOML format
    /// and writes it to the specified file. The output will be pretty-formatted
    /// for human readability. If the target directory doesn't exist, it will be
    /// created automatically.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the configuration file should be saved
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(())` - Configuration was successfully saved
    /// - `Err(Error::Config)` - If serialization fails or the file cannot be written
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The configuration cannot be serialized to TOML (should be rare)
    /// - Parent directories cannot be created due to permissions
    /// - The file cannot be written due to permissions or disk space issues
    ///
    /// # Behaviour
    ///
    /// - Creates parent directories automatically if they don't exist
    /// - Overwrites existing files at the target path
    /// - Uses pretty-formatted TOML output for readability
    /// - Logs the save operation for debugging purposes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use repo_roller_cli::config::AppConfig;
    ///
    /// let config = AppConfig::default();
    /// let config_path = Path::new("./my-config.toml");
    ///
    /// match config.save(&config_path) {
    ///     Ok(()) => println!("Configuration saved successfully"),
    ///     Err(e) => eprintln!("Failed to save config: {}", e),
    /// }
    /// ```
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        debug!("Saving configuration to {:?}", path);

        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize configuration: {}", e)))?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Config(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| Error::Config(format!("Failed to write configuration file: {}", e)))?;

        info!("Configuration saved to {:?}", path);
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: Config { templates: vec![] },
            authentication: AuthenticationConfig::new(),
            organization: OrganizationConfig::new(),
        }
    }
}

/// Configuration for CLI authentication settings.
///
/// This structure holds authentication-related configuration for the CLI,
/// including the preferred authentication method for GitHub operations.
/// Currently supports token-based authentication.
///
/// # Fields
///
/// * `auth_method` - The authentication method to use (defaults to "token")
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(default = "AuthenticationConfig::default_auth_method")]
    pub auth_method: String,
}

impl AuthenticationConfig {
    /// Returns the default authentication method.
    ///
    /// This is used as the default value for the auth_method field
    /// when deserializing from TOML if the field is not present.
    fn default_auth_method() -> String {
        "token".to_string()
    }

    /// Creates a new AuthenticationConfig with default values.
    ///
    /// # Returns
    ///
    /// Returns a new `AuthenticationConfig` instance with the default
    /// authentication method set to "token".
    pub fn new() -> Self {
        AuthenticationConfig {
            auth_method: Self::default_auth_method(),
        }
    }
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            auth_method: AuthenticationConfig::default_auth_method(),
        }
    }
}

/// Configuration for organization settings.
///
/// This structure holds organization-specific configuration for the CLI,
/// including the metadata repository name used for organization settings discovery.
///
/// # Fields
///
/// * `metadata_repository_name` - Name of the repository containing organization configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// Name of the metadata repository for organization configuration.
    ///
    /// This repository should contain the organization's configuration files
    /// including global defaults, repository type configurations, and team settings.
    #[serde(default = "OrganizationConfig::default_metadata_repository_name")]
    pub metadata_repository_name: String,
}

impl OrganizationConfig {
    /// Returns the default metadata repository name.
    ///
    /// This is used as the default value for the metadata_repository_name field
    /// when deserializing from TOML if the field is not present.
    fn default_metadata_repository_name() -> String {
        DEFAULT_METADATA_REPOSITORY_NAME.to_string()
    }

    /// Creates a new OrganizationConfig with default values.
    ///
    /// # Returns
    ///
    /// Returns a new `OrganizationConfig` instance with the default
    /// metadata repository name set to ".reporoller".
    pub fn new() -> Self {
        OrganizationConfig {
            metadata_repository_name: Self::default_metadata_repository_name(),
        }
    }
}

impl Default for OrganizationConfig {
    fn default() -> Self {
        Self {
            metadata_repository_name: OrganizationConfig::default_metadata_repository_name(),
        }
    }
}

/// Resolves the path to the configuration file.
///
/// This function determines the configuration file path based on the provided
/// argument. If a specific path is provided, it uses that path. Otherwise,
/// it defaults to looking for the configuration file in the current directory.
///
/// # Arguments
///
/// * `config_path` - Optional path to a specific configuration file
///
/// # Returns
///
/// Returns a `PathBuf` pointing to the configuration file location.
///
/// # Behaviour
///
/// - If `config_path` is `Some(path)`, returns that path as a `PathBuf`
/// - If `config_path` is `None`, returns `./config.toml` in the current directory
/// - Falls back to the current directory if unable to determine the working directory
pub fn get_config_path(config_path: Option<&str>) -> PathBuf {
    if let Some(path) = config_path {
        PathBuf::from(path)
    } else {
        // Look for config in current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join(DEFAULT_CONFIG_FILENAME)
    }
}
