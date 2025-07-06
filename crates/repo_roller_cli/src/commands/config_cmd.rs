//! Configuration management commands for the RepoRoller CLI.
//!
//! This module provides commands for managing configuration files, including:
//! - Initializing new configuration files
//! - Validating existing configuration syntax
//! - Getting and setting configuration values
//!
//! Configuration files are stored in TOML format and contain settings for
//! templates, authentication methods, and other application preferences.

use clap::Subcommand;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, AppConfig};
use crate::errors::Error;

/// Configuration management subcommands.
///
/// This enum defines the available configuration commands for managing
/// application settings, including initialization, validation, and
/// getting/setting configuration values.
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Create a new configuration file with default values.
    ///
    /// Initializes a new TOML configuration file at the specified path
    /// or the default location if no path is provided.
    Init {
        /// Path where the configuration file should be created.
        /// If not specified, uses the default location (./config.toml).
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Validate the syntax and structure of a configuration file.
    ///
    /// Checks that the configuration file can be parsed and contains
    /// valid settings according to the application schema.
    Validate {
        /// Path to the configuration file to validate.
        /// If not specified, uses the default location (./config.toml).
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Display current configuration values.
    ///
    /// Shows either the entire configuration or a specific value
    /// identified by a dot-separated key path.
    Get {
        /// Path to the configuration file to read.
        /// If not specified, uses the default location (./config.toml).
        #[arg(short, long)]
        path: Option<String>,

        /// Specific configuration key to retrieve (e.g., "authentication.auth_method").
        /// If not specified, displays the entire configuration.
        key: Option<String>,
    },

    /// Update a configuration value.
    ///
    /// Sets a specific configuration value identified by a dot-separated
    /// key path to the provided value.
    Set {
        /// Path to the configuration file to modify.
        /// If not specified, uses the default location (./config.toml).
        #[arg(short, long)]
        path: Option<String>,

        /// Configuration key to set using dot notation (e.g., "authentication.auth_method").
        key: String,

        /// New value to assign to the specified key.
        value: String,
    },
}

/// Executes the specified configuration command.
///
/// This function serves as the main entry point for configuration commands,
/// routing to the appropriate handler based on the command type.
///
/// # Arguments
///
/// * `cmd` - The configuration command to execute
///
/// # Returns
///
/// Returns `Ok(())` on successful command execution, or an `Error` if
/// the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Configuration file operations fail (read, write, parse)
/// - Invalid configuration keys are specified
/// - File system operations fail
#[instrument]
pub async fn execute(cmd: &ConfigCommands) -> Result<(), Error> {
    match cmd {
        ConfigCommands::Init { path } => init_config(path.as_deref()),
        ConfigCommands::Validate { path } => validate_config(path.as_deref()),
        ConfigCommands::Get { path, key } => get_config(path.as_deref(), key.as_deref()),
        ConfigCommands::Set { path, key, value } => set_config(path.as_deref(), key, value),
    }
}

/// Creates a new configuration file with default values.
///
/// This function initializes a new TOML configuration file at the specified
/// path with default application settings. It will fail if a configuration
/// file already exists at the target location.
///
/// # Arguments
///
/// * `path` - Optional path for the configuration file. If None, uses default location.
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an `Error` if creation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - A configuration file already exists at the target path
/// - The file cannot be written due to permissions or disk space
/// - Parent directories cannot be created
#[instrument]
fn init_config(path: Option<&str>) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(message = "Initializing configuration", path = ?config_path);

    if config_path.exists() {
        let err = Error::Config(format!(
            "Configuration file already exists at {:?}",
            config_path
        ));
        error!(
            message = "Configuration file already exists",
            path = ?config_path,
            error = ?err
        );
        return Err(err);
    }

    let config = AppConfig::default();
    if let Err(e) = config.save(&config_path) {
        error!(message = "Failed to save configuration", path = ?config_path, error = ?e);
        return Err(Error::Config("Failed to save configuration".to_string()));
    }

    info!(message = "Configuration initialized", path = ?config_path);
    println!("Configuration initialized at {:?}", config_path);
    Ok(())
}

/// Validates the syntax and structure of a configuration file.
///
/// This function attempts to load and parse the configuration file to
/// verify that it contains valid TOML syntax and matches the expected
/// application schema.
///
/// # Arguments
///
/// * `path` - Optional path to the configuration file. If None, uses default location.
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid, or an `Error` if validation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be found
/// - The file contains invalid TOML syntax
/// - The configuration structure doesn't match the expected schema
#[instrument]
fn validate_config(path: Option<&str>) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(message = "Validating configuration", path = ?config_path);

    match AppConfig::load(&config_path) {
        Ok(_) => {
            info!(message = "Configuration is valid", path = ?config_path);
            println!("Configuration is valid");
            Ok(())
        }
        Err(e) => {
            error!(
                message = "Configuration is invalid",
                path = ?config_path,
                error = ?e
            );
            Err(Error::Config("The configuration is invalid".to_string()))
        }
    }
}

/// Retrieves and displays configuration values.
///
/// This function loads the configuration file and either displays the entire
/// configuration or a specific value identified by a dot-separated key path.
///
/// # Arguments
///
/// * `path` - Optional path to the configuration file. If None, uses default location.
/// * `key` - Optional dot-separated key path. If None, displays entire configuration.
///
/// # Returns
///
/// Returns `Ok(())` after displaying the requested configuration, or an `Error` if retrieval fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be loaded
/// - The specified key path is invalid or doesn't exist
/// - Configuration serialization fails
#[instrument]
fn get_config(path: Option<&str>, key: Option<&str>) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(message = "Getting configuration", path = ?config_path, key = ?key);

    let config = match AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(Error::Config(
                "Failed to load the configuration".to_string(),
            ));
        }
    };

    if let Some(key) = key {
        // Get specific key
        let value = get_config_value(&config, key)?;
        println!("{}: {}", key, value);
    } else {
        // Print entire config
        let config_str = toml::to_string_pretty(&config)
            .map_err(|e| Error::Config(format!("Failed to serialize configuration: {}", e)))?;
        println!("{}", config_str);
    }

    Ok(())
}

/// Updates a configuration value.
///
/// This function loads the configuration file, updates the specified key
/// with the new value, and saves the modified configuration back to disk.
/// If the configuration file doesn't exist, it creates a new one with default values.
///
/// # Arguments
///
/// * `path` - Optional path to the configuration file. If None, uses default location.
/// * `key` - Dot-separated key path identifying the configuration value to update.
/// * `value` - New value to assign to the specified key.
///
/// # Returns
///
/// Returns `Ok(())` after successfully updating the configuration, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be loaded or saved
/// - The specified key path is invalid
/// - File system operations fail
#[instrument]
fn set_config(path: Option<&str>, key: &str, value: &str) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(
        message = "Setting configuration",
        path = ?config_path,
        key = key,
        value = value
    );

    // Load existing config or create a new one
    let mut config = match if config_path.exists() {
        AppConfig::load(&config_path)
    } else {
        Ok(AppConfig::default())
    } {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(Error::Config(
                "Failed to load the configuration".to_string(),
            ));
        }
    };

    // Update the config
    if let Err(e) = set_config_value(&mut config, key, value) {
        error!(message = "Failed to set configuration value", key = key, value = value, error = ?e);
        return Err(e);
    }

    // Save the updated config
    if let Err(e) = config.save(&config_path) {
        error!(message = "Failed to save configuration", path = ?config_path, error = ?e);
        return Err(Error::Config("Failed to save configuration".to_string()));
    }

    info!(message = "Configuration updated", key = key, value = value);
    println!("Configuration updated: {} = {}", key, value);
    Ok(())
}

/// Retrieves a configuration value using a dot-separated key path.
///
/// This helper function navigates the configuration structure using the
/// provided key path and returns the corresponding value as a string.
/// Currently supports keys under the "authentication" section.
fn get_config_value(config: &AppConfig, key: &str) -> Result<String, Error> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(Error::InvalidArguments(
            "Invalid configuration key".to_string(),
        ));
    }

    match parts[0] {
        "authentication" => match parts.get(1) {
            Some(&"auth_method") => Ok(config.authentication.auth_method.clone()),
            _ => Err(Error::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        _ => Err(Error::InvalidArguments(format!(
            "Invalid configuration key: {}",
            key
        ))),
    }
}

/// Updates a configuration value using a dot-separated key path.
///
/// This helper function navigates the configuration structure using the
/// provided key path and updates the corresponding value. Currently
/// supports keys under the "authentication" section.
fn set_config_value(config: &mut AppConfig, key: &str, value: &str) -> Result<(), Error> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err(Error::InvalidArguments(
            "Invalid configuration key".to_string(),
        ));
    }

    match parts[0] {
        "authentication" => match parts.get(1) {
            Some(&"auth_method") => {
                if value.is_empty() {
                    config.authentication.auth_method = String::new();
                } else {
                    config.authentication.auth_method = value.to_string();
                }
                Ok(())
            }
            _ => Err(Error::InvalidArguments(format!(
                "Invalid configuration key: {}",
                key
            ))),
        },
        _ => Err(Error::InvalidArguments(format!(
            "Invalid configuration key: {}",
            key
        ))),
    }
}
