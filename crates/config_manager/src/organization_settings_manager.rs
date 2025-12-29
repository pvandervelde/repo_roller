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
    basic_validator::BasicConfigurationValidator, errors::ConfigurationResult,
    merger::ConfigurationMerger, metadata_provider::MetadataRepositoryProvider,
    validator::ConfigurationValidator,
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

    /// Configuration validator for validating merged configurations.
    ///
    /// Created internally as BasicConfigurationValidator is stateless.
    validator: Arc<BasicConfigurationValidator>,

    /// Template loader for loading template configurations.
    template_loader: Arc<crate::TemplateLoader>,
}

impl OrganizationSettingsManager {
    /// Creates a new organization settings manager.
    ///
    /// # Arguments
    ///
    /// * `metadata_provider` - Provider for configuration discovery and loading
    /// * `template_loader` - Loader for template configurations
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use config_manager::{
    ///     OrganizationSettingsManager, GitHubMetadataProvider, MetadataProviderConfig,
    ///     TemplateLoader, GitHubTemplateRepository
    /// };
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider_config = MetadataProviderConfig::explicit("repo-config");
    /// let metadata_provider = GitHubMetadataProvider::new(
    ///     "my-org",
    ///     Arc::new(github_client.clone()), // Your GitHub client
    ///     provider_config,
    /// );
    ///
    /// let template_repo = Arc::new(GitHubTemplateRepository::new(Arc::new(github_client)));
    /// let template_loader = Arc::new(TemplateLoader::new(template_repo));
    ///
    /// let manager = OrganizationSettingsManager::new(
    ///     Arc::new(metadata_provider),
    ///     template_loader,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        metadata_provider: Arc<dyn MetadataRepositoryProvider>,
        template_loader: Arc<crate::TemplateLoader>,
    ) -> Self {
        Self {
            metadata_provider,
            merger: Arc::new(ConfigurationMerger::new()),
            validator: Arc::new(BasicConfigurationValidator::new()),
            template_loader,
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

        // Step 2.5: Load standard labels from global configuration
        debug!("Loading standard labels");
        let standard_labels = self
            .metadata_provider
            .load_standard_labels(&metadata_repo)
            .await
            .map_err(|e| {
                warn!("Failed to load standard labels: {}", e);
                e
            })?;

        if !standard_labels.is_empty() {
            info!("Loaded {} standard labels", standard_labels.len());
        } else {
            debug!("No standard labels found");
        }

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

        // Step 5: Load template configuration from template repository
        debug!("Loading template configuration: {}", context.template());
        let template_config = self
            .template_loader
            .load_template_configuration(context.organization(), context.template())
            .await
            .map_err(|e| {
                warn!("Failed to load template configuration: {}", e);
                e
            })?;

        if let Some(ref repo_type) = template_config.repository_type {
            info!(
                "Template specifies repository type: {} (policy: {:?})",
                repo_type.repository_type, repo_type.policy
            );
        }

        if let Some(ref variables) = template_config.variables {
            info!("Template defines {} variables", variables.len());
        }

        // Step 6: Merge all configurations using ConfigurationMerger
        debug!("Merging configurations");
        let mut merged = self
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

        // Step 6.5: Merge standard labels into configuration
        // Standard labels act as the baseline, and labels from other sources
        // (repository type, team, template) override them by name
        debug!("Merging standard labels into configuration");
        for (label_name, label_config) in standard_labels {
            // Only add if not already present (higher precedence sources override)
            merged.labels.entry(label_name).or_insert(label_config);
        }

        if !merged.labels.is_empty() {
            info!(
                "Configuration has {} labels after merging",
                merged.labels.len()
            );
        }

        // Step 7: Validate merged configuration
        debug!("Validating merged configuration");
        let validation_result = self
            .validator
            .validate_merged_config(&merged)
            .await
            .map_err(|e| {
                warn!("Configuration validation check failed: {}", e);
                e
            })?;

        if !validation_result.is_valid() {
            warn!(
                "Configuration validation failed with {} errors",
                validation_result.errors.len()
            );
            for error in &validation_result.errors {
                warn!(
                    "  - [{}] {}: {}",
                    error.error_type, error.field_path, error.message
                );
            }
            return Err(crate::errors::ConfigurationError::ValidationFailed {
                error_count: validation_result.errors.len(),
                errors: validation_result.errors,
            });
        }

        if !validation_result.warnings.is_empty() {
            info!(
                "Configuration has {} warnings",
                validation_result.warnings.len()
            );
            for warning in &validation_result.warnings {
                info!("  - {}: {}", warning.field_path, warning.message);
            }
        }

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
            .field("validator", &self.validator)
            .field("template_loader", &self.template_loader)
            .finish()
    }
}

#[cfg(test)]
#[path = "organization_settings_manager_tests.rs"]
mod tests;
