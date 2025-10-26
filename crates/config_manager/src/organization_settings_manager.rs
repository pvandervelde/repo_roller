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
//! ```rust,no_run
//! use config_manager::{
//!     OrganizationSettingsManager, ConfigurationContext,
//!     GitHubMetadataProvider, MetadataProviderConfig
//! };
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create metadata provider
//! let provider_config = MetadataProviderConfig {
//!     organization: "my-org".to_string(),
//!     metadata_repository_name: Some("repo-config".to_string()),
//! };
//! let metadata_provider = GitHubMetadataProvider::new(
//!     Arc::new(github_client), // Your GitHub client
//!     provider_config,
//! )?;
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
/// ```rust,no_run
/// use config_manager::{
///     OrganizationSettingsManager, GitHubMetadataProvider, MetadataProviderConfig
/// };
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create metadata provider
/// let provider_config = MetadataProviderConfig {
///     organization: "my-org".to_string(),
///     metadata_repository_name: Some("repo-config".to_string()),
/// };
/// let metadata_provider = GitHubMetadataProvider::new(
///     Arc::new(github_client), // Your GitHub client
///     provider_config,
/// )?;
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
    /// ```rust,no_run
    /// use config_manager::{
    ///     OrganizationSettingsManager, GitHubMetadataProvider, MetadataProviderConfig
    /// };
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider_config = MetadataProviderConfig {
    ///     organization: "my-org".to_string(),
    ///     metadata_repository_name: Some("repo-config".to_string()),
    /// };
    /// let metadata_provider = GitHubMetadataProvider::new(
    ///     Arc::new(github_client), // Your GitHub client
    ///     provider_config,
    /// )?;
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
