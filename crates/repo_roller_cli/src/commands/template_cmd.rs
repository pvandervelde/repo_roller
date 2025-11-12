//! Template inspection and validation commands for the RepoRoller CLI.
//!
//! This module provides commands for inspecting and validating templates
//! in the organization's metadata repository. It allows users to:
//! - Get detailed information about a template
//! - Validate template configuration and structure
//!
//! # Examples
//!
//! ```bash
//! # Get template information
//! repo-roller template info --org myorg --template rust-library
//!
//! # Validate template
//! repo-roller template validate --org myorg --template rust-library
//! ```

use clap::Subcommand;
use config_manager::{
    GitHubMetadataProvider, MetadataProviderConfig, MetadataRepositoryProvider,
};
use github_client::GitHubClient;
use keyring::Entry;
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::config::{get_config_path, AppConfig, DEFAULT_METADATA_REPOSITORY_NAME};
use crate::errors::Error;

// Keyring constants (shared with create_cmd and auth_cmd)
// Currently unused but will be needed when template loading is implemented
#[allow(dead_code)]
const KEY_RING_SERVICE_NAME: &str = "repo_roller";
#[allow(dead_code)]
const KEY_RING_APP_ID: &str = "github_app_id";
#[allow(dead_code)]
const KEY_RING_APP_PRIVATE_KEY_PATH: &str = "github_app_private_key_path";

/// Template inspection and validation subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum TemplateCommands {
    /// Get detailed information about a template.
    ///
    /// Displays template metadata including name, description, author, tags,
    /// repository type specification, and defined variables.
    Info {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Template name.
        #[arg(long)]
        template: String,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Validate a template configuration.
    ///
    /// Validates the template configuration file (template.toml) including
    /// structure, variable definitions, and repository type specification.
    Validate {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Template name to validate.
        #[arg(long)]
        template: String,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

/// Executes the specified template command.
///
/// # Arguments
///
/// * `cmd` - The template command to execute
///
/// # Returns
///
/// Returns `Ok(())` on successful command execution, or an `Error` if
/// the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Authentication fails (GitHub credentials not available)
/// - The organization metadata repository is not found
/// - The specified template is not found
/// - Template configuration is invalid
/// - GitHub API operations fail
/// - Output formatting fails
#[instrument]
pub async fn execute(cmd: &TemplateCommands) -> Result<(), Error> {
    match cmd {
        TemplateCommands::Info {
            org,
            template,
            format,
        } => template_info(org, template, format).await,
        TemplateCommands::Validate {
            org,
            template,
            format,
        } => template_validate(org, template, format).await,
    }
}

/// Creates an authenticated metadata provider wrapped in Arc.
///
/// Loads GitHub App credentials from the system keyring and creates
/// an authenticated GitHubMetadataProvider instance.
///
/// TODO: This will be needed when template loading from template repositories is implemented
#[allow(dead_code)]
async fn create_metadata_provider() -> Result<Arc<dyn MetadataRepositoryProvider>, Error> {
    // Load application config to get metadata repository name
    let config_path = get_config_path(None);
    let app_config = AppConfig::load(&config_path).unwrap_or_else(|_| AppConfig::default());

    // Load GitHub App ID from keyring
    let app_id_entry = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
        .map_err(|e| Error::Auth(format!("Failed to access keyring for app ID: {}", e)))?;

    let app_id_str = app_id_entry
        .get_password()
        .map_err(|e| Error::Auth(format!("Failed to get app ID from keyring: {}", e)))?;

    let app_id: u64 = app_id_str
        .parse()
        .map_err(|e| Error::Auth(format!("Invalid app ID format: {}", e)))?;

    // Load private key path from keyring
    let key_path_entry = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
        .map_err(|e| Error::Auth(format!("Failed to access keyring for key path: {}", e)))?;

    let key_path = key_path_entry
        .get_password()
        .map_err(|e| Error::Auth(format!("Failed to get key path from keyring: {}", e)))?;

    // Read private key file
    let private_key = std::fs::read_to_string(&key_path).map_err(|e| {
        Error::Auth(format!(
            "Failed to read private key from {}: {}",
            key_path, e
        ))
    })?;

    // Create authenticated GitHub client
    let octocrab = github_client::create_app_client(app_id, &private_key)
        .await
        .map_err(|e| Error::Auth(format!("Failed to create GitHub App client: {}", e)))?;

    let github_client = GitHubClient::new(octocrab);

    // Create metadata provider config
    let metadata_repo_name = if app_config.organization.metadata_repository_name.is_empty() {
        DEFAULT_METADATA_REPOSITORY_NAME
    } else {
        &app_config.organization.metadata_repository_name
    };

    let config = MetadataProviderConfig::explicit(metadata_repo_name);
    let provider = GitHubMetadataProvider::new(github_client, config);

    Ok(Arc::new(provider))
}

/// Helper function to format output as JSON or pretty-printed debug format.
fn format_output<T: serde::Serialize + std::fmt::Debug>(
    data: &T,
    format: &str,
) -> Result<String, Error> {
    match format {
        "json" => serde_json::to_string_pretty(data)
            .map_err(|e| Error::Config(format!("Failed to serialize to JSON: {}", e))),
        "pretty" => Ok(format!("{:#?}", data)),
        _ => Err(Error::InvalidArguments(format!(
            "Invalid format: '{}'. Use 'json' or 'pretty'.",
            format
        ))),
    }
}

/// Gets detailed information about a template.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template` - Template name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn template_info(org: &str, template: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Getting template information",
        org = org,
        template = template,
        format = format
    );

    // TODO: Implement template loading from template repositories
    // For now, return a helpful message about the limitation

    #[derive(serde::Serialize, Debug)]
    struct TemplateInfoPlaceholder {
        message: String,
        template: String,
        organization: String,
        note: String,
    }

    let placeholder = TemplateInfoPlaceholder {
        message: "Template information loading not yet implemented".to_string(),
        template: template.to_string(),
        organization: org.to_string(),
        note: "Template metadata is loaded from template repositories, not the metadata repository. This feature will be implemented in a future version.".to_string(),
    };

    let output = format_output(&placeholder, format)?;
    println!("{}", output);

    Err(Error::Config(
        "Template information loading not yet implemented - see Task 9.3 template discovery endpoints".to_string(),
    ))
}

/// Validates a template configuration.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template` - Template name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn template_validate(org: &str, template: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Validating template",
        org = org,
        template = template,
        format = format
    );

    // TODO: Implement template validation from template repositories
    // For now, return a helpful message about the limitation

    #[derive(serde::Serialize, Debug)]
    struct TemplateValidationPlaceholder {
        message: String,
        template: String,
        organization: String,
        note: String,
    }

    let placeholder = TemplateValidationPlaceholder {
        message: "Template validation not yet implemented".to_string(),
        template: template.to_string(),
        organization: org.to_string(),
        note: "Template validation requires loading template.toml from template repositories. This feature will be implemented as part of Task 9.3 (REST API template endpoints).".to_string(),
    };

    let output = format_output(&placeholder, format)?;
    println!("{}", output);

    Err(Error::Config(
        "Template validation not yet implemented - see Task 9.3 template discovery endpoints".to_string(),
    ))
}

#[cfg(test)]
#[path = "template_cmd_tests.rs"]
mod tests;
