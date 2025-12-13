//! Organization configuration resolution and application.
//!
//! This module handles the resolution of organization-wide settings and their
//! application to newly created repositories.
//!
//! ## Overview
//!
//! Configuration is managed through two main operations:
//! - **Resolution**: Fetching and merging organization-level configuration from metadata repositories
//! - **Application**: Applying resolved configuration (labels, webhooks, custom properties) to repositories
//!
//! ## Configuration Resolution
//!
//! The [`resolve_organization_configuration`] function:
//! 1. Creates a metadata provider to access the organization's metadata repository
//! 2. Uses `OrganizationSettingsManager` to resolve configuration hierarchy
//! 3. Merges organization, template, and global defaults
//! 4. Falls back to global defaults if metadata repository is unavailable
//!
//! ## Configuration Application
//!
//! The [`apply_repository_configuration`] function:
//! 1. Applies labels to the repository (future implementation)
//! 2. Creates webhooks (future implementation)
//! 3. Sets custom properties including repository type
//!
//! ## Error Handling
//!
//! - Configuration resolution failures fall back to global defaults with warnings
//! - Application failures return errors to prevent incomplete repository setup
//!
//! ## Examples
//!
//! ```rust,ignore
//! // Resolve organization configuration
//! let merged_config = resolve_organization_configuration(
//!     installation_token,
//!     "my-org",
//!     "rust-service",
//!     ".reporoller"
//! ).await?;
//!
//! // Apply configuration to repository
//! apply_repository_configuration(
//!     &client,
//!     "my-org",
//!     "my-repo",
//!     &merged_config
//! ).await?;
//! ```

use crate::errors::{GitHubError, RepoRollerError, RepoRollerResult, SystemError};
use github_client::{GitHubClient, RepositoryClient};
use tracing::{debug, error, info, warn};

#[cfg(test)]
#[path = "configuration_tests.rs"]
mod tests;

/// Resolve organization configuration from metadata repository.
///
/// This function fetches and merges configuration from the organization's metadata
/// repository, combining organization-level settings, template-specific overrides,
/// and global defaults into a single unified configuration.
///
/// ## Configuration Hierarchy
///
/// Configuration is resolved in the following priority order (highest to lowest):
/// 1. Template-specific overrides in metadata repository
/// 2. Organization-level settings in metadata repository
/// 3. Global defaults from config_manager
///
/// ## Metadata Repository
///
/// The metadata repository (typically named `.reporoller` or similar) contains:
/// - Organization-wide default settings
/// - Template-specific configuration overrides
/// - Label definitions
/// - Webhook configurations
/// - Custom property schemas
///
/// ## Fallback Behavior
///
/// If the metadata repository is not found or configuration resolution fails:
/// - Logs a warning
/// - Falls back to global defaults
/// - Allows repository creation to proceed with minimal configuration
///
/// ## Parameters
///
/// * `installation_token` - GitHub App installation token for authentication
/// * `organization` - Organization name where the repository will be created
/// * `template_name` - Name of the template being used
/// * `metadata_repository_name` - Name of the metadata repository (e.g., ".reporoller")
///
/// ## Returns
///
/// Returns `RepoRollerResult<MergedConfiguration>` with the resolved configuration.
///
/// ## Errors
///
/// Returns `RepoRollerError` if:
/// - GitHub client creation fails
/// - Configuration structure is invalid
/// - Internal errors occur during resolution
///
/// Note: Metadata repository access failures result in fallback, not errors.
///
/// ## Example
///
/// ```rust,ignore
/// let config = resolve_organization_configuration(
///     installation_token,
///     "acme-corp",
///     "rust-service",
///     ".reporoller"
/// ).await?;
///
/// println!("Resolved {} labels", config.labels.len());
/// println!("Resolved {} webhooks", config.webhooks.len());
/// ```
pub(crate) async fn resolve_organization_configuration(
    installation_token: &str,
    organization: &str,
    template_name: &str,
    metadata_repository_name: &str,
) -> RepoRollerResult<config_manager::MergedConfiguration> {
    use config_manager::{
        ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
        OrganizationSettingsManager,
    };
    use std::sync::Arc;

    info!("Resolving organization configuration");

    // Create a separate client for the metadata provider
    let metadata_client = github_client::create_token_client(installation_token).map_err(|e| {
        error!("Failed to create metadata provider client: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create metadata provider client: {}", e),
        })
    })?;
    let metadata_repo_client = GitHubClient::new(metadata_client);

    let metadata_provider_config = MetadataProviderConfig::explicit(metadata_repository_name);
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        metadata_repo_client,
        metadata_provider_config,
    ));

    let settings_manager = OrganizationSettingsManager::new(metadata_provider);

    let config_context = ConfigurationContext::new(organization, template_name);

    let merged_config = settings_manager
        .resolve_configuration(&config_context)
        .await
        .or_else(|e: config_manager::ConfigurationError| -> config_manager::ConfigurationResult<config_manager::MergedConfiguration> {
            warn!(
                "Failed to resolve organization configuration: {}. Using global defaults.",
                e
            );
            // If configuration resolution fails (e.g., metadata repository not found),
            // fall back to using global defaults with empty overrides
            Ok(config_manager::MergedConfiguration::default())
        })?;

    info!("Organization configuration resolved successfully");
    Ok(merged_config)
}

/// Apply merged configuration to a newly created repository.
///
/// This function applies the resolved organization configuration to a repository,
/// setting up labels, webhooks, and custom properties according to organization
/// standards and template specifications.
///
/// ## Applied Configuration
///
/// The function applies the following configuration elements:
///
/// ### Labels (Future Implementation)
/// - Creates repository labels with specified colors and descriptions
/// - Currently logged but not yet implemented (requires GitHubClient enhancement)
///
/// ### Webhooks (Future Implementation)
/// - Configures repository webhooks for events
/// - Currently logged but not yet implemented (requires GitHubClient enhancement)
///
/// ### Custom Properties
/// - Sets custom repository properties including repository type
/// - Fully implemented and functional
/// - Uses GitHub's custom properties API
///
/// ## Parameters
///
/// * `installation_repo_client` - Authenticated GitHub client for repository operations
/// * `owner` - Repository owner (organization or user)
/// * `repo_name` - Name of the repository
/// * `merged_config` - Resolved configuration from `resolve_organization_configuration`
///
/// ## Returns
///
/// Returns `RepoRollerResult<()>` on success.
///
/// ## Errors
///
/// Returns `RepoRollerError` if:
/// - Custom properties API call fails
/// - Network errors occur
/// - Authentication is insufficient
///
/// ## Example
///
/// ```rust,ignore
/// let merged_config = resolve_organization_configuration(...).await?;
///
/// apply_repository_configuration(
///     &client,
///     "acme-corp",
///     "new-service",
///     &merged_config
/// ).await?;
///
/// println!("Configuration applied successfully");
/// ```
///
/// ## Future Enhancements
///
/// The following features are planned:
/// - Label creation via GitHubClient::create_label()
/// - Webhook creation via GitHubClient::create_webhook()
/// - Branch protection rules application
pub(crate) async fn apply_repository_configuration(
    installation_repo_client: &GitHubClient,
    owner: &str,
    repo_name: &str,
    merged_config: &config_manager::MergedConfiguration,
) -> RepoRollerResult<()> {
    info!(
        "Applying merged configuration to repository {}/{}",
        owner, repo_name
    );

    // Apply labels
    if !merged_config.labels.is_empty() {
        debug!("Creating {} labels", merged_config.labels.len());
        for (label_name, label_config) in &merged_config.labels {
            info!("Creating label: {} -> {:?}", label_name, label_config);
            installation_repo_client
                .create_label(
                    owner,
                    repo_name,
                    &label_config.name,
                    &label_config.color,
                    &label_config.description,
                )
                .await
                .map_err(|e| {
                    error!("Failed to create label '{}': {}", label_name, e);
                    RepoRollerError::System(SystemError::Internal {
                        reason: format!("Failed to create label '{}': {}", label_name, e),
                    })
                })?;
        }
        info!("Successfully created {} labels", merged_config.labels.len());
    }

    // Apply webhooks
    if !merged_config.webhooks.is_empty() {
        debug!("Creating {} webhooks", merged_config.webhooks.len());
        // TODO: Implement webhook creation via GitHub API
        // This requires adding create_webhook() method to GitHubClient
        // Tracked in separate task/issue
        for webhook in &merged_config.webhooks {
            info!("Webhook to create: {:?}", webhook);
        }
        warn!(
            "Webhook creation not yet implemented - requires GitHubClient::create_webhook() method"
        );
    }

    // Apply custom properties (including repository type)
    if !merged_config.custom_properties.is_empty() {
        debug!(
            "Setting {} custom properties",
            merged_config.custom_properties.len()
        );

        // Convert custom properties to GitHub API format
        let properties: Vec<serde_json::Value> = merged_config
            .custom_properties
            .iter()
            .map(|prop| {
                use config_manager::settings::custom_property::CustomPropertyValue;
                let value = match &prop.value {
                    CustomPropertyValue::String(s) => serde_json::Value::String(s.clone()),
                    CustomPropertyValue::SingleSelect(s) => serde_json::Value::String(s.clone()),
                    CustomPropertyValue::MultiSelect(vec) => serde_json::Value::Array(
                        vec.iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                    CustomPropertyValue::Boolean(b) => serde_json::Value::Bool(*b),
                };

                serde_json::json!({
                    "property_name": prop.property_name,
                    "value": value
                })
            })
            .collect();

        let payload = github_client::CustomPropertiesPayload::new(properties);

        installation_repo_client
            .set_repository_custom_properties(owner, repo_name, &payload)
            .await
            .map_err(|e| {
                error!("Failed to set custom properties on repository: {}", e);
                RepoRollerError::GitHub(GitHubError::NetworkError {
                    reason: format!(
                        "Failed to set custom properties on {}/{}: {}",
                        owner, repo_name, e
                    ),
                })
            })?;

        info!(
            "Successfully set {} custom properties",
            merged_config.custom_properties.len()
        );
    }

    Ok(())
}
