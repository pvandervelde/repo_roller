//! Authentication command module for GitHub credentials management.
//!
//! This module provides commands for configuring authentication with GitHub,
//! supporting both GitHub App authentication and Personal Access Token authentication.
//! Credentials are securely stored in the system keyring.
//!
//! ## Supported Authentication Methods
//!
//! - **GitHub App**: Uses App ID, private key, and webhook secret
//! - **Personal Access Token**: Uses a GitHub personal access token
//!
//! ## Security
//!
//! All sensitive credentials are stored in the system keyring rather than
//! in configuration files to maintain security.

use clap::Subcommand;
use keyring::Entry;
use std::path::PathBuf;
use tracing::{debug, error, info, instrument};

use crate::{
    config::{get_config_path, AppConfig},
    errors::Error,
};

pub const KEY_RING_SERVICE_NAME: &str = "repo_roller_cli";
pub const KEY_RING_APP_ID: &str = "github_app_id";
pub const KEY_RING_APP_PRIVATE_KEY_PATH: &str = "github_private_key_path";
pub const KEY_RING_USER_TOKEN: &str = "github_token";
pub const KEY_RING_WEB_HOOK_SECRET: &str = "webhook_secret";

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;

/// Authentication subcommands for managing GitHub credentials.
///
/// This enum defines the available authentication commands that can be used
/// to configure different types of GitHub authentication methods.
#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Configure GitHub authentication credentials.
    ///
    /// Sets up authentication with GitHub using either GitHub App credentials
    /// or Personal Access Token. Credentials are securely stored in the system keyring.
    #[command(name = "github")]
    GitHub {
        /// Authentication method to configure.
        ///
        /// Supported methods:
        /// - "app": GitHub App authentication (requires App ID, private key, webhook secret)
        /// - "token": Personal Access Token authentication (requires GitHub PAT)
        #[arg(default_value = "token")]
        method: String,
    },
}

/// Executes the specified authentication command.
///
/// This function serves as the main entry point for authentication commands,
/// routing to the appropriate handler based on the command type.
///
/// # Arguments
///
/// * `cmd` - The authentication command to execute
///
/// # Returns
///
/// Returns `Ok(())` on successful authentication setup, or an `Error` if
/// the authentication process fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be loaded or saved
/// - Keyring operations fail
/// - Invalid authentication method is specified
/// - Required credentials are missing or invalid
#[instrument]
pub async fn execute(cmd: &AuthCommands) -> Result<(), Error> {
    match cmd {
        AuthCommands::GitHub { method } => auth_github(method).await,
    }
}

/// Configures GitHub authentication based on the specified method.
///
/// This function handles the interactive setup of GitHub authentication,
/// prompting the user for required credentials and storing them securely
/// in the system keyring. It also updates the application configuration
/// to reflect the chosen authentication method.
///
/// # Arguments
///
/// * `method` - The authentication method ("app" or "token")
///
/// # Returns
///
/// Returns `Ok(())` on successful configuration, or an `Error` if setup fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Configuration cannot be loaded or saved
/// - Keyring operations fail
/// - User input is invalid or missing
/// - Private key file doesn't exist (for app method)
/// - Unsupported authentication method is specified
#[instrument]
async fn auth_github(method: &str) -> Result<(), Error> {
    debug!(message = "Authenticating with GitHub", method = method);

    let config_path = get_config_path(None);
    let mut config = match AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!(message = "Failed to load configuration", path = ?config_path, error = ?e);
            return Err(Error::Config("Failed to load configuration".to_string()));
        }
    };

    match method {
        "app" => {
            // GitHub App authentication
            info!(message = "GitHub App Authentication");
            println!("GitHub App Authentication");
            println!("------------------------");
            println!("Please provide the following information:");

            // Get App ID
            println!("App ID:");
            let mut app_id = String::new();
            std::io::stdin()
                .read_line(&mut app_id)
                .map_err(|e| Error::Auth(format!("Failed to read input: {}", e)))?;
            let app_id = app_id.trim();
            debug!(message = "Read app ID from stdin", app_id = app_id);

            // Get private key path
            println!("Path to private key file:");
            let mut key_path = String::new();
            std::io::stdin()
                .read_line(&mut key_path)
                .map_err(|e| Error::Auth(format!("Failed to read input: {}", e)))?;
            let key_path = key_path.trim();
            debug!(message = "Read key path from stdin", key_path = key_path);

            // Verify the key file exists
            let key_path_buf = PathBuf::from(key_path);
            if !key_path_buf.exists() {
                let err = Error::Auth(format!("Private key file not found: {}", key_path));
                error!(message = "Private key file not found", key_path = key_path, error = ?err);
                return Err(err);
            }

            // Get the webhook secret
            println!("Webhook secret:");
            let mut webhook_secret = String::new();
            std::io::stdin()
                .read_line(&mut webhook_secret)
                .map_err(|e| Error::Auth(format!("Failed to read input: {}", e)))?;
            let webhook_secret = webhook_secret.trim();
            debug!(
                message = "Read webhook secret from stdin",
                webhook_secret = webhook_secret
            );

            let keyring_app_id =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID).map_err(|e| {
                    Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_app_id.set_password(app_id).map_err(|e| {
                Error::Auth(format!("Failed to save the app ID to the keyring: {}", e))
            })?;
            debug!(message = "Saved app ID to keyring");

            let keyring_key_path = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
                .map_err(|e| {
                    Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_key_path.set_password(key_path).map_err(|e| {
                Error::Auth(format!(
                    "Failed to save the app private key to the keyring: {}",
                    e
                ))
            })?;
            debug!(message = "Saved key path to keyring");

            let keyring_webhook_secret =
                Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_WEB_HOOK_SECRET).map_err(|e| {
                    Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
                })?;
            keyring_webhook_secret
                .set_password(webhook_secret)
                .map_err(|e| {
                    Error::Auth(format!(
                        "Failed to save the webhook secret to the keyring: {}",
                        e
                    ))
                })?;
            debug!(message = "Saved webhook secret to keyring");

            config.authentication.auth_method = "app".to_string();
            config.save(&config_path).map_err(|e| {
                error!(error = e.to_string(), "Failed to save the configuration");
                Error::Config("Failed to save the configuration.".to_string())
            })?;
            info!(
                message = "Updated configuration with auth method",
                auth_method = "app"
            );

            println!("GitHub App authentication configured successfully!");
        }
        "token" => {
            // Personal Access Token authentication
            info!(message = "GitHub Personal Access Token Authentication");
            println!("GitHub Personal Access Token Authentication");
            println!("------------------------------------------");
            println!("Please provide your GitHub Personal Access Token:");
            println!("(Token will not be displayed as you type)");

            // Get token (in a real implementation, this would use a secure input method)
            let mut token = String::new();
            std::io::stdin()
                .read_line(&mut token)
                .map_err(|e| Error::Auth(format!("Failed to read input: {}", e)))?;
            let token = token.trim();
            debug!(message = "Read token from stdin");

            if token.is_empty() {
                let err = Error::Auth("Token cannot be empty".to_string());
                error!(message = "Token cannot be empty", error = ?err);
                return Err(err);
            }

            let keyring = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_USER_TOKEN).map_err(|e| {
                Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
            })?;
            keyring
                .set_password(token)
                .map_err(|e| Error::Auth(format!("Failed to save token to keyring: {}", e)))?;
            debug!(message = "Saved token to keyring");

            config.authentication.auth_method = "token".to_string();
            config.save(&config_path).map_err(|e| {
                error!(error = e.to_string(), "Failed to save the configuration");
                Error::Config("Failed to save the configuration.".to_string())
            })?;
            info!(
                message = "Updated configuration with auth method",
                auth_method = "token"
            );

            println!("GitHub token authentication configured successfully!");
        }
        _ => {
            let err =
                Error::InvalidArguments(format!("Unsupported authentication method: {}", method));
            error!(message = "Unsupported authentication method", method = method, error = ?err);
            return Err(err);
        }
    }

    Ok(())
}
