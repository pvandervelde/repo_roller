//! Organization settings manager.
//!
//! This module implements the main orchestration component for configuration resolution.
//! The `OrganizationSettingsManager` coordinates metadata repository discovery, configuration
//! loading, and hierarchical merging to produce final repository settings.
//!
//! # Architecture
//!
//! The manager acts as the orchestration layer that ties together:
//! - `MetadataRepositoryProvider` for configuration discovery and loading
//! - `ConfigurationMerger` for hierarchical configuration merging
//!
//! # Usage
//!
//! ```rust,ignore
//! use config_manager::{
//!     OrganizationSettingsManager, ConfigurationContext,
//!     GitHubMetadataProvider, MetadataProviderConfig
//! };
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create metadata provider
//! let provider_config = MetadataProviderConfig::explicit("repo-config");
//! let metadata_provider = GitHubMetadataProvider::new(
//!     "my-org",
//!     Arc::new(github_client), // Your GitHub client
//!     provider_config,
//! );
//!
//! // Create organization settings manager
//! let manager = OrganizationSettingsManager::new(
//!     Arc::new(metadata_provider),
//! );
//!
//! // Create configuration context
//! let context = ConfigurationContext::new("my-org", "rust-service")
//!     .with_team("backend-team");
//!
//! // Resolve configuration
//! let merged_config = manager.resolve_configuration(&context).await?;
//! # Ok(())
//! # }
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::{
    errors::ConfigurationResult, merger::ConfigurationMerger,
    metadata_provider::MetadataRepositoryProvider,
};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Organization settings manager.
///
/// Orchestrates configuration resolution workflow:
/// 1. Discover metadata repository
/// 2. Load configuration from all hierarchy levels (global, team, repository type, template)
/// 3. Merge configurations according to precedence rules
/// 4. Return final merged configuration
///
/// This is a stateless component that can be shared across threads.
///
/// # Examples
///
/// ```rust,ignore
/// use config_manager::{
///     OrganizationSettingsManager, GitHubMetadataProvider, MetadataProviderConfig
/// };
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create metadata provider
/// let provider_config = MetadataProviderConfig::explicit("repo-config");
/// let metadata_provider = GitHubMetadataProvider::new(
///     "my-org",
///     Arc::new(github_client), // Your GitHub client
///     provider_config,
/// );
///
/// // Create manager
/// let manager = OrganizationSettingsManager::new(Arc::new(metadata_provider));
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct OrganizationSettingsManager {
    /// Metadata repository provider for configuration discovery and loading.
    metadata_provider: Arc<dyn MetadataRepositoryProvider>,

    /// Configuration merger for hierarchical merging.
    ///
    /// Created internally as ConfigurationMerger is stateless.
    merger: Arc<ConfigurationMerger>,
}

impl OrganizationSettingsManager {
    /// Creates a new organization settings manager.
    ///
    /// # Arguments
    ///
    /// * `metadata_provider` - Provider for configuration discovery and loading
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use config_manager::{
    ///     OrganizationSettingsManager, GitHubMetadataProvider, MetadataProviderConfig
    /// };
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider_config = MetadataProviderConfig::explicit("repo-config");
    /// let metadata_provider = GitHubMetadataProvider::new(
    ///     "my-org",
    ///     Arc::new(github_client), // Your GitHub client
    ///     provider_config,
    /// );
    ///
    /// let manager = OrganizationSettingsManager::new(Arc::new(metadata_provider));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(metadata_provider: Arc<dyn MetadataRepositoryProvider>) -> Self {
        Self {
            metadata_provider,
            merger: Arc::new(ConfigurationMerger::new()),
        }
    }

    /// Resolves configuration for a repository creation request.
    ///
    /// Implements the complete configuration resolution workflow:
    /// 1. Discover metadata repository
    /// 2. Load configuration from all applicable hierarchy levels
    /// 3. Merge configurations according to precedence rules
    /// 4. Return final merged configuration
    ///
    /// # Arguments
    ///
    /// * `context` - Configuration resolution context with organization, template, team, and repository type
    ///
    /// # Returns
    ///
    /// Returns the merged configuration combining all applicable configuration sources.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError` if:
    /// - Metadata repository cannot be discovered
    /// - Configuration files cannot be loaded or parsed
    /// - Override policies are violated during merging
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use config_manager::{
    ///     OrganizationSettingsManager, ConfigurationContext,
    ///     GitHubMetadataProvider, MetadataProviderConfig
    /// };
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider_config = MetadataProviderConfig::explicit("repo-config");
    /// # let metadata_provider = GitHubMetadataProvider::new(
    /// #     "my-org",
    /// #     Arc::new(github_client), // Your GitHub client
    /// #     provider_config,
    /// # );
    /// let manager = OrganizationSettingsManager::new(Arc::new(metadata_provider));
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service")
    ///     .with_team("backend-team");
    ///
    /// let merged_config = manager.resolve_configuration(&context).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        skip(self),
        fields(
            organization = %context.organization(),
            template = %context.template(),
            team = ?context.team(),
            repository_type = ?context.repository_type()
        )
    )]
    pub async fn resolve_configuration(
        &self,
        context: &crate::ConfigurationContext,
    ) -> ConfigurationResult<crate::merged_config::MergedConfiguration> {
        info!("Starting configuration resolution");

        // Step 1: Discover metadata repository
        debug!("Discovering metadata repository");
        let metadata_repo = self
            .metadata_provider
            .discover_metadata_repository(context.organization())
            .await
            .map_err(|e| {
                warn!("Failed to discover metadata repository: {}", e);
                e
            })?;

        info!(
            "Discovered metadata repository: {} (method: {:?})",
            metadata_repo.repository_name, metadata_repo.discovery_method
        );

        // Step 2: Load global defaults
        debug!("Loading global defaults");
        let global_defaults = self
            .metadata_provider
            .load_global_defaults(&metadata_repo)
            .await
            .map_err(|e| {
                warn!("Failed to load global defaults: {}", e);
                e
            })?;

        debug!("Global defaults loaded successfully");

        // Step 3: Load repository type configuration (if specified)
        let repository_type_config = if let Some(repo_type) = context.repository_type() {
            debug!("Loading repository type configuration: {}", repo_type);
            let config = self
                .metadata_provider
                .load_repository_type_configuration(&metadata_repo, repo_type)
                .await
                .map_err(|e| {
                    warn!("Failed to load repository type configuration: {}", e);
                    e
                })?;

            if config.is_some() {
                info!("Repository type configuration loaded: {}", repo_type);
            } else {
                debug!("No repository type configuration found for: {}", repo_type);
            }

            config
        } else {
            debug!("No repository type specified");
            None
        };

        // Step 4: Load team configuration (if specified)
        let team_config = if let Some(team) = context.team() {
            debug!("Loading team configuration: {}", team);
            let config = self
                .metadata_provider
                .load_team_configuration(&metadata_repo, team)
                .await
                .map_err(|e| {
                    warn!("Failed to load team configuration: {}", e);
                    e
                })?;

            if config.is_some() {
                info!("Team configuration loaded: {}", team);
            } else {
                debug!("No team configuration found for: {}", team);
            }

            config
        } else {
            debug!("No team specified");
            None
        };

        // Step 5: Create minimal template configuration
        // TODO: Load actual template configuration from template repository
        // For now, create a minimal template config to enable testing
        debug!("Creating minimal template configuration");
        let template_config = crate::template_config::TemplateConfig {
            template: crate::template_config::TemplateMetadata {
                name: context.template().to_string(),
                description: format!("Template: {}", context.template()),
                author: "System".to_string(),
                tags: vec![],
            },
            repository_type: None,
            variables: None,
            repository: None,
            pull_requests: None,
            branch_protection: None,
            labels: None,
            webhooks: None,
            environments: None,
            github_apps: None,
        };

        // Step 6: Merge all configurations using ConfigurationMerger
        debug!("Merging configurations");
        let merged = self
            .merger
            .merge_configurations(
                &global_defaults,
                repository_type_config.as_ref(),
                team_config.as_ref(),
                &template_config,
            )
            .map_err(|e| {
                warn!("Configuration merge failed: {}", e);
                e
            })?;

        info!(
            "Configuration resolution completed successfully (fields configured: {})",
            merged.source_trace.field_count()
        );

        Ok(merged)
    }
}

impl std::fmt::Debug for OrganizationSettingsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrganizationSettingsManager")
            .field("metadata_provider", &"Arc<dyn MetadataRepositoryProvider>")
            .field("merger", &self.merger)
            .finish()
    }
}

#[cfg(test)]
#[path = "organization_settings_manager_tests.rs"]
mod tests;
