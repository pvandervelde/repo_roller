use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::errors::Error;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = "config.toml";

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

/// Configuration for Merge Warden CLI
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub authentication: AuthenticationConfig,
}

impl AppConfig {
    /// Load configuration from the specified file
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

    /// Save configuration to the specified file
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
            authentication: AuthenticationConfig::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    #[serde(default = "AuthenticationConfig::default_auth_method")]
    pub auth_method: String,
}

impl AuthenticationConfig {
    fn default_auth_method() -> String {
        "token".to_string()
    }

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

/// Get the path to the configuration file
pub fn get_config_path(config_path: Option<&str>) -> PathBuf {
    if let Some(path) = config_path {
        PathBuf::from(path)
    } else {
        // Look for config in current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join(DEFAULT_CONFIG_FILENAME)
    }
}
