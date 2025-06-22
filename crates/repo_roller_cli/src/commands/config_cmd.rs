use clap::Subcommand;
use tracing::{debug, error, info, instrument};

use crate::config::{get_config_path, AppConfig};
use crate::errors::Error;

/// Subcommands for the config command
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Create initial configuration file
    Init {
        /// Path to save the configuration file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Check configuration syntax
    Validate {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Show current configuration
    Get {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Configuration key to get (e.g., "rules.require_work_items")
        key: Option<String>,
    },

    /// Update configuration values
    Set {
        /// Path to the configuration file
        #[arg(short, long)]
        path: Option<String>,

        /// Configuration key to set (e.g., "rules.require_work_items")
        key: String,

        /// Value to set
        value: String,
    },
}

/// Execute the config command
#[instrument]
pub async fn execute(cmd: ConfigCommands) -> Result<(), Error> {
    match cmd {
        ConfigCommands::Init { path } => init_config(path.as_deref()),
        ConfigCommands::Validate { path } => validate_config(path.as_deref()),
        ConfigCommands::Get { path, key } => get_config(path.as_deref(), key.as_deref()),
        ConfigCommands::Set { path, key, value } => set_config(path.as_deref(), &key, &value),
    }
}

/// Initialize a new configuration file
#[instrument]
fn init_config(path: Option<&str>) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(message = "Initializing configuration", path = ?config_path);

    if config_path.exists() {
        let err = Error::ConfigError(format!(
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
        return Err(Error::ConfigError(
            "Failed to save configuration".to_string(),
        ));
    }

    info!(message = "Configuration initialized", path = ?config_path);
    println!("Configuration initialized at {:?}", config_path);
    Ok(())
}

/// Validate a configuration file
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
            Err(Error::ConfigError(
                "The configuration is invalid".to_string(),
            ))
        }
    }
}

/// Get a configuration value
#[instrument]
fn get_config(path: Option<&str>, key: Option<&str>) -> Result<(), Error> {
    let config_path = get_config_path(path);
    debug!(message = "Getting configuration", path = ?config_path, key = ?key);

    let config = match AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(Error::ConfigError(
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
            .map_err(|e| Error::ConfigError(format!("Failed to serialize configuration: {}", e)))?;
        println!("{}", config_str);
    }

    Ok(())
}

/// Set a configuration value
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
            return Err(Error::ConfigError(
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
        return Err(Error::ConfigError(
            "Failed to save configuration".to_string(),
        ));
    }

    info!(message = "Configuration updated", key = key, value = value);
    println!("Configuration updated: {} = {}", key, value);
    Ok(())
}

/// Get a value from the configuration by key path
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

/// Set a value in the configuration by key path
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
