//! Organization settings inspection commands for the RepoRoller CLI.
//!
//! This module provides commands for inspecting organization-specific configuration
//! settings, repository types, and merged configurations. It allows users to:
//! - List available repository types in an organization
//! - View repository type-specific configuration
//! - Preview merged configuration for a repository creation scenario
//! - View global defaults for an organization
//!
//! These commands help users understand the configuration hierarchy and validate
//! settings before creating repositories.
//!
//! # Examples
//!
//! ```bash
//! # List repository types
//! repo-roller org-settings list-types --org myorg
//!
//! # Show configuration for a specific repository type
//! repo-roller org-settings show-type --org myorg --type library
//!
//! # Preview merged configuration
//! repo-roller org-settings show-merged --org myorg --template rust-lib --team platform
//!
//! # Show global defaults
//! repo-roller org-settings show-global --org myorg
//! ```

use clap::Subcommand;
use config_manager::{
    ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
    MetadataRepositoryProvider, OrganizationSettingsManager, RepositoryTypeName,
};
use github_client::GitHubClient;
use keyring::Entry;
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::config::{get_config_path, AppConfig, DEFAULT_METADATA_REPOSITORY_NAME};
use crate::errors::Error;

// Keyring constants (shared with create_cmd and auth_cmd)
const KEY_RING_SERVICE_NAME: &str = "repo_roller";
const KEY_RING_APP_ID: &str = "github_app_id";
const KEY_RING_APP_PRIVATE_KEY_PATH: &str = "github_app_private_key_path";

/// Organization settings inspection subcommands.
///
/// These commands provide visibility into the organization configuration hierarchy,
/// allowing users to inspect settings before creating repositories.
#[derive(Subcommand, Debug, Clone)]
pub enum OrgSettingsCommands {
    /// List available repository types for an organization.
    ///
    /// Displays all repository types defined in the organization's metadata repository.
    /// Repository types are used to classify repositories and apply type-specific
    /// configuration settings.
    ListTypes {
        /// Organization name to query.
        ///
        /// Must be a valid GitHub organization that the authenticated user has access to.
        #[arg(long)]
        org: String,

        /// Output format (json or pretty).
        ///
        /// - json: Machine-readable JSON output
        /// - pretty: Human-readable formatted output (default)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Show configuration for a specific repository type.
    ///
    /// Displays the type-specific configuration settings that apply to repositories
    /// of the given type. This includes repository settings, branch protection rules,
    /// labels, webhooks, and other type-specific overrides.
    ShowType {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Repository type name to inspect.
        ///
        /// Must be a valid repository type defined in the organization configuration.
        #[arg(long)]
        type_name: String,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Show merged configuration for a repository creation scenario.
    ///
    /// Previews the final merged configuration that would be applied when creating
    /// a repository with the specified parameters. This merges settings from all
    /// hierarchy levels (global → repository type → team → template) and shows
    /// the final effective configuration.
    ShowMerged {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Template name to use for the preview.
        ///
        /// The template configuration will have the highest precedence in the merge.
        #[arg(long)]
        template: String,

        /// Team name (optional).
        ///
        /// If specified, team-specific configuration will be included in the merge.
        #[arg(long)]
        team: Option<String>,

        /// Repository type (optional).
        ///
        /// If specified, repository type configuration will be included in the merge.
        #[arg(long)]
        repo_type: Option<String>,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Show global defaults for an organization.
    ///
    /// Displays the organization-wide baseline configuration settings. These are
    /// the lowest precedence settings in the hierarchy and can be overridden by
    /// repository type, team, or template configurations.
    ShowGlobal {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

/// Executes the specified organization settings command.
///
/// This function serves as the main entry point for organization settings commands,
/// routing to the appropriate handler based on the command type.
///
/// # Arguments
///
/// * `cmd` - The organization settings command to execute
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
/// - The specified repository type or template is not found
/// - GitHub API operations fail
/// - Output formatting fails
#[instrument]
pub async fn execute(cmd: &OrgSettingsCommands) -> Result<(), Error> {
    match cmd {
        OrgSettingsCommands::ListTypes { org, format } => list_types(org, format).await,
        OrgSettingsCommands::ShowType {
            org,
            type_name,
            format,
        } => show_type(org, type_name, format).await,
        OrgSettingsCommands::ShowMerged {
            org,
            template,
            team,
            repo_type,
            format,
        } => show_merged(org, template, team.as_deref(), repo_type.as_deref(), format).await,
        OrgSettingsCommands::ShowGlobal { org, format } => show_global(org, format).await,
    }
}

/// Creates an authenticated metadata provider wrapped in Arc.
///
/// Loads GitHub App credentials from the system keyring and creates
/// an authenticated GitHubMetadataProvider instance. The metadata repository
/// name is loaded from the application configuration file.
///
/// # Returns
///
/// Returns the provider wrapped in Arc on success, or an Error if authentication fails.
///
/// # Errors
///
/// Returns an error if:
/// - GitHub App credentials are not found in keyring
/// - Private key file cannot be read
/// - GitHub client creation fails
/// - Application configuration cannot be loaded
async fn create_metadata_provider() -> Result<Arc<dyn MetadataRepositoryProvider>, Error> {
    // Load application config to get metadata repository name
    let config_path = get_config_path(None);
    let app_config = AppConfig::load(&config_path).unwrap_or_else(|_| {
        // If config doesn't exist, use default
        AppConfig::default()
    });

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

    // Create authenticated GitHub client using github_client helper
    let octocrab = github_client::create_app_client(app_id, &private_key)
        .await
        .map_err(|e| Error::Auth(format!("Failed to create GitHub App client: {}", e)))?;

    let github_client = GitHubClient::new(octocrab);

    // Create metadata provider config using the configured repository name
    // Falls back to DEFAULT_METADATA_REPOSITORY_NAME if not configured
    // Note: Empty string check handles case where config file explicitly sets empty value
    let metadata_repo_name = if app_config.organization.metadata_repository_name.is_empty() {
        DEFAULT_METADATA_REPOSITORY_NAME
    } else {
        &app_config.organization.metadata_repository_name
    };

    let config = MetadataProviderConfig::explicit(metadata_repo_name);

    let provider = GitHubMetadataProvider::new(github_client, config);

    Ok(Arc::new(provider) as Arc<dyn MetadataRepositoryProvider>)
}

/// Formats output as JSON or pretty-printed text.
///
/// # Arguments
///
/// * `value` - The value to format (must be Serialize)
/// * `format` - Output format ("json" or "pretty")
fn format_output<T: serde::Serialize>(value: &T, format: &str) -> Result<String, Error> {
    match format {
        "json" => serde_json::to_string_pretty(value)
            .map_err(|e| Error::Config(format!("Failed to serialize to JSON: {}", e))),
        "pretty" => {
            // For now, use JSON formatting for pretty print too
            // Can be enhanced later with custom formatting
            serde_json::to_string_pretty(value)
                .map_err(|e| Error::Config(format!("Failed to serialize for display: {}", e)))
        }
        _ => Err(Error::InvalidArguments(format!(
            "Invalid format '{}', must be 'json' or 'pretty'",
            format
        ))),
    }
}

/// Lists available repository types for an organization.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `format` - Output format ("json" or "pretty")
///
/// # Returns
///
/// Returns `Ok(())` after displaying the repository types, or an `Error` if retrieval fails.
///
/// # Errors
///
/// Returns an error if:
/// - Authentication fails
/// - Metadata repository is not found
/// - GitHub API operation fails
#[instrument]
async fn list_types(org: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Listing repository types",
        org = org,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(org)
        .await
        .map_err(|e| {
            Error::Config(format!(
                "Failed to discover metadata repository for '{}': {}",
                org, e
            ))
        })?;

    // List available repository types
    let types = provider
        .list_available_repository_types(&metadata_repo)
        .await
        .map_err(|e| Error::Config(format!("Failed to list repository types: {}", e)))?;

    // Format and display output
    let output = format_output(&types, format)?;
    println!("{}", output);

    Ok(())
}

/// Shows configuration for a specific repository type.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `type_name` - Repository type name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn show_type(org: &str, type_name: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Showing repository type configuration",
        org = org,
        type_name = type_name,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(org)
        .await
        .map_err(|e| {
            Error::Config(format!(
                "Failed to discover metadata repository for '{}': {}",
                org, e
            ))
        })?;

    // Validate repository type name
    let repo_type = RepositoryTypeName::try_new(type_name).map_err(|e| {
        Error::InvalidArguments(format!(
            "Invalid repository type name '{}': {}",
            type_name, e
        ))
    })?;

    // Load repository type configuration
    let type_config = provider
        .load_repository_type_configuration(&metadata_repo, repo_type.as_str())
        .await
        .map_err(|e| {
            Error::Config(format!(
                "Failed to load repository type configuration for '{}': {}",
                type_name, e
            ))
        })?;

    match type_config {
        Some(config) => {
            // Format and display output
            let output = format_output(&config, format)?;
            println!("{}", output);
            Ok(())
        }
        None => Err(Error::Config(format!(
            "Repository type '{}' not found in organization '{}'",
            type_name, org
        ))),
    }
}

/// Shows merged configuration for a repository creation scenario.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template` - Template name
/// * `team` - Optional team name
/// * `repo_type` - Optional repository type
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn show_merged(
    org: &str,
    template: &str,
    team: Option<&str>,
    repo_type: Option<&str>,
    format: &str,
) -> Result<(), Error> {
    debug!(
        message = "Showing merged configuration",
        org = org,
        template = template,
        team = ?team,
        repo_type = ?repo_type,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Create organization settings manager
    let manager = OrganizationSettingsManager::new(provider);

    // Create configuration context
    let mut context = ConfigurationContext::new(org, template);

    if let Some(t) = team {
        context = context.with_team(t);
    }

    if let Some(rt) = repo_type {
        context = context.with_repository_type(rt);
    }

    // Resolve merged configuration
    let merged_config = manager
        .resolve_configuration(&context)
        .await
        .map_err(|e| Error::Config(format!("Failed to resolve merged configuration: {}", e)))?;

    // Format and display output
    let output = format_output(&merged_config, format)?;
    println!("{}", output);

    Ok(())
}

/// Shows global defaults for an organization.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn show_global(org: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Showing global defaults",
        org = org,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(org)
        .await
        .map_err(|e| {
            Error::Config(format!(
                "Failed to discover metadata repository for '{}': {}",
                org, e
            ))
        })?;

    // Load global defaults
    let global_defaults = provider
        .load_global_defaults(&metadata_repo)
        .await
        .map_err(|e| Error::Config(format!("Failed to load global defaults: {}", e)))?;

    // Format and display output
    let output = format_output(&global_defaults, format)?;
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
#[path = "org_settings_cmd_tests.rs"]
mod tests;
