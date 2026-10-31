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
//! let (merged_config, _template_name) = resolve_organization_configuration(
//!     installation_token,
//!     "my-org",
//!     "rust-service",
//!     ".reporoller",
//!     None,
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
use crate::{LabelManager, RulesetManager, WebhookManager};
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
/// * `template_name` - Name of the template being used; an empty string
///   means the caller gave no explicit template name
/// * `metadata_repository_name` - Name of the metadata repository (e.g., ".reporoller")
/// * `designation` - The `TemplateDesignation` implied by the request's
///   `CreationMode` (see `designation_for_creation_mode` in `lib.rs`), used
///   to auto-discover a designated template when `template_name` is empty
///   (#255 follow-up Gap 1)
///
/// ## Returns
///
/// Returns `RepoRollerResult<(MergedConfiguration, Option<String>)>`: the
/// resolved configuration, plus the effective template name used to build
/// it (see [`effective_template_name`]) -- `Some(explicit or auto-discovered
/// name)`, or `None` when neither an explicit name nor a designated
/// template was found.
///
/// ## Errors
///
/// Returns `RepoRollerError` if:
/// - GitHub client creation fails
/// - Configuration structure is invalid
/// - Internal errors occur during resolution
/// - Designated-template discovery is ambiguous
///   (`ConfigurationError::AmbiguousTemplateDesignation`), propagated rather
///   than swallowed by the metadata-repository-unavailable fallback below
///
/// Note: Metadata repository access failures result in fallback, not errors.
///
/// ## Example
///
/// ```rust,ignore
/// let (config, template_name) = resolve_organization_configuration(
///     installation_token,
///     "acme-corp",
///     "rust-service",
///     ".reporoller",
///     None,
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
    designation: Option<config_manager::TemplateDesignation>,
) -> RepoRollerResult<(config_manager::MergedConfiguration, Option<String>)> {
    use config_manager::{
        ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
        OrganizationSettingsManager,
    };
    use std::sync::Arc;

    info!("Resolving organization configuration");

    info!("Creating metadata provider for repository discovery");
    info!("Metadata repository name: {}", metadata_repository_name);

    // Create a separate client for the metadata provider
    let metadata_client = github_client::create_token_client(installation_token).map_err(|e| {
        error!("Failed to create metadata provider client: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create metadata provider client: {}", e),
        })
    })?;
    let metadata_repo_client = GitHubClient::new(metadata_client.clone());

    let metadata_provider_config = MetadataProviderConfig::explicit(metadata_repository_name);
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        metadata_repo_client,
        metadata_provider_config,
    ));

    info!("Metadata provider created successfully");

    // Create template loader for template configuration resolution
    // Template loader needs Arc<GitHubClient> so create a separate client instance
    let template_client = GitHubClient::new(metadata_client);
    let template_repo = Arc::new(config_manager::GitHubTemplateRepository::new(Arc::new(
        template_client,
    )));
    let template_loader = Arc::new(config_manager::TemplateLoader::new(template_repo));

    let settings_manager = OrganizationSettingsManager::new(metadata_provider, template_loader);

    // Auto-discover a designated template when the caller gave no explicit
    // template name but the request's CreationMode implies a designation
    // (Fork/SourceHistoryMirror/SourceOnlyMirror). Only attempt discovery in
    // that case: an explicit template name always wins without even
    // consulting discovery (see `effective_template_name`), and
    // `CreationMode::Standard` (designation = None) never has anything to
    // discover.
    let discovery_result: config_manager::ConfigurationResult<Option<String>> =
        if template_name.is_empty() {
            match designation {
                Some(d) => settings_manager.find_designated_template(organization, d).await,
                None => Ok(None),
            }
        } else {
            Ok(None)
        };

    let resolved_template_name = effective_template_name(template_name, designation, discovery_result)?;
    let effective_name = resolved_template_name.clone().unwrap_or_default();

    let config_context = ConfigurationContext::new(organization, &effective_name);

    info!("Calling settings_manager.resolve_configuration with context: org={}, template={}, team={:?}, repo_type={:?}",
           organization, effective_name, config_context.team(), config_context.repository_type());

    let merged_config = settings_manager
        .resolve_configuration(&config_context)
        .await
        .or_else(|e: config_manager::ConfigurationError| -> config_manager::ConfigurationResult<config_manager::MergedConfiguration> {
            error!(
                "Failed to resolve organization configuration: {}. Using global defaults.",
                e
            );
            // If configuration resolution fails (e.g., metadata repository not found),
            // fall back to using global defaults with empty overrides
            Ok(config_manager::MergedConfiguration::default())
        })?;

    info!(
        "Organization configuration resolved successfully: labels={}, webhooks={}",
        merged_config.labels.len(),
        merged_config.webhooks.len()
    );
    if !merged_config.labels.is_empty() {
        info!(
            "Resolved labels: {:?}",
            merged_config.labels.keys().collect::<Vec<_>>()
        );
    }
    Ok((merged_config, resolved_template_name))
}

/// Decides which template name [`resolve_organization_configuration`] must
/// use to build its `ConfigurationContext`, given:
///
/// - `template_name`: the caller's explicit template name (empty string when
///   the caller gave none),
/// - `designation`: the `TemplateDesignation` implied by the request's
///   `CreationMode` (see `designation_for_creation_mode` in `lib.rs`), and
/// - `discovery_result`: the result of calling
///   `settings_manager.find_designated_template(organization, designation)`
///   -- only meaningful (and only actually invoked by the caller) when
///   `template_name.is_empty() && designation.is_some()`.
///
/// ## Decision Table
///
/// - `template_name` non-empty: always returns
///   `Ok(Some(template_name.to_string()))`, regardless of
///   `designation`/`discovery_result` -- an explicit template always wins
///   and is never even inspected against discovery.
/// - `template_name` empty, `designation = None`: returns `Ok(None)` --
///   `CreationMode::Standard` has no designation and nothing to discover.
/// - `template_name` empty, `designation = Some(_)`: returns
///   `discovery_result` unchanged (`Ok(Some(name))`, `Ok(None)`, or the
///   propagated `Err`).
///
/// # Errors
///
/// Propagates `discovery_result`'s `Err` (notably
/// `ConfigurationError::AmbiguousTemplateDesignation`) when reached.
pub(crate) fn effective_template_name(
    template_name: &str,
    designation: Option<config_manager::TemplateDesignation>,
    discovery_result: config_manager::ConfigurationResult<Option<String>>,
) -> config_manager::ConfigurationResult<Option<String>> {
    if !template_name.is_empty() {
        return Ok(Some(template_name.to_string()));
    }

    match designation {
        Some(_) => discovery_result,
        None => Ok(None),
    }
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
/// ### Labels
/// - Creates repository labels with specified colors and descriptions
/// - Uses LabelManager for orchestration (idempotent, handles partial failures)
///
/// ### Webhooks
/// - Configures repository webhooks for events
/// - Uses WebhookManager for orchestration (validates, deduplicates, secure)
///
/// ### Custom Properties
/// - Sets custom repository properties including repository type
/// - Uses GitHub's custom properties API directly
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
/// - Label/webhook operations fail
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
/// ## Current Implementation
///
/// - Labels: Applied via LabelManager (idempotent, tracks results)
/// - Webhooks: Applied via WebhookManager (validates, deduplicates, secure)
/// - Rulesets: Applied via RulesetManager (idempotent, conflict detection)
/// - Custom Properties: Applied via GitHub API (including repository type)
///
/// ## Future Enhancements
///
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

    // Apply labels using LabelManager
    info!(
        "Checking labels to apply: total={}, empty={}",
        merged_config.labels.len(),
        merged_config.labels.is_empty()
    );

    if !merged_config.labels.is_empty() {
        info!(
            "Labels to apply: {:?}",
            merged_config.labels.keys().collect::<Vec<_>>()
        );
        let label_manager = LabelManager::new(installation_repo_client.clone());
        let label_result = label_manager
            .apply_labels(owner, repo_name, &merged_config.labels)
            .await?;

        info!(
            "Label application complete: created={}, updated={}, failed={}, skipped={}",
            label_result.created, label_result.updated, label_result.failed, label_result.skipped
        );

        if label_result.failed > 0 {
            warn!(
                "Failed to apply {} label(s): {:?}",
                label_result.failed, label_result.failed_labels
            );
        }
    }

    // Apply webhooks using WebhookManager
    if !merged_config.webhooks.is_empty() {
        let webhook_manager = WebhookManager::new(installation_repo_client.clone());
        let webhook_result = webhook_manager
            .apply_webhooks(owner, repo_name, &merged_config.webhooks)
            .await?;

        info!(
            "Webhook application complete: created={}, updated={}, failed={}, skipped={}",
            webhook_result.created,
            webhook_result.updated,
            webhook_result.failed,
            webhook_result.skipped
        );

        if webhook_result.failed > 0 {
            warn!(
                "Failed to apply {} webhook(s): {:?}",
                webhook_result.failed, webhook_result.failed_webhooks
            );
        }
    }

    // Apply rulesets using RulesetManager
    if !merged_config.rulesets.is_empty() {
        info!(
            "Applying {} rulesets to repository {}/{}",
            merged_config.rulesets.len(),
            owner,
            repo_name
        );

        // Convert Vec<RulesetConfig> to HashMap<String, RulesetConfig>
        let rulesets_map: std::collections::HashMap<
            String,
            config_manager::settings::RulesetConfig,
        > = merged_config
            .rulesets
            .iter()
            .map(|r| (r.name.clone(), r.clone()))
            .collect();

        let ruleset_manager = RulesetManager::new(installation_repo_client.clone());
        let ruleset_result = ruleset_manager
            .apply_rulesets(owner, repo_name, &rulesets_map)
            .await?;

        info!(
            "Ruleset application complete: created={}, updated={}, failed={}",
            ruleset_result.created, ruleset_result.updated, ruleset_result.failed
        );

        if ruleset_result.failed > 0 {
            warn!(
                "Failed to apply {} ruleset(s): {:?}",
                ruleset_result.failed, ruleset_result.failed_rulesets
            );
        }
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
