//! Metadata repository provider interface and types.
//!
//! This module defines the abstract interface for accessing organization
//! configuration from metadata repositories stored in GitHub.
//!
//! # Repository Structure
//!
//! The metadata repository is expected to have the following structure:
//!
//! ```text
//! org-metadata/
//! ├── global-defaults.toml      # Organization-wide baseline
//! ├── labels.toml               # Standard labels
//! ├── teams/
//! │   ├── backend-team/
//! │   │   └── config.toml       # Team overrides
//! │   └── frontend-team/
//! │       └── config.toml
//! └── types/
//!     ├── library/
//!     │   └── config.toml       # Repository type config
//!     └── service/
//!         └── config.toml
//! ```
//!
//! See specs/design/organization-repository-settings.md for complete specification.

use crate::{ConfigurationResult, GlobalDefaults, LabelConfig, RepositoryTypeConfig, TeamConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Reference the tests module in the separate file
#[cfg(test)]
#[path = "metadata_provider_tests.rs"]
mod tests;

/// Records how a metadata repository was actually discovered.
///
/// **This is a result/output type** that documents the discovery method used
/// to find a repository. It's part of `MetadataRepository` and tells consumers
/// how the repository was located (after the fact).
///
/// For **configuring** how to search (input), see `MetadataProviderConfig` in
/// the `github_metadata_provider` module.
///
/// Organizations can discover their metadata repository through:
/// - Configuration-based: Explicitly named in application configuration
/// - Topic-based: Discovered by searching for a specific GitHub topic
///
/// # Examples
///
/// ```
/// use config_manager::DiscoveryMethod;
///
/// // Explicit configuration
/// let config_based = DiscoveryMethod::ConfigurationBased {
///     repository_name: "org-metadata".to_string(),
/// };
///
/// // Topic-based discovery
/// let topic_based = DiscoveryMethod::TopicBased {
///     topic: "reporoller-metadata".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Repository name explicitly configured in application settings.
    ConfigurationBased { repository_name: String },

    /// Repository discovered by searching for a specific GitHub topic.
    TopicBased { topic: String },
}

/// Metadata about the discovered organization configuration repository.
///
/// Contains information about how the repository was found and when
/// it was last accessed for caching and validation purposes.
///
/// # Examples
///
/// ```
/// use config_manager::{MetadataRepository, DiscoveryMethod};
/// use chrono::Utc;
///
/// let metadata_repo = MetadataRepository {
///     organization: "my-org".to_string(),
///     repository_name: "org-metadata".to_string(),
///     discovery_method: DiscoveryMethod::ConfigurationBased {
///         repository_name: "org-metadata".to_string(),
///     },
///     last_updated: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataRepository {
    /// The GitHub organization name.
    pub organization: String,

    /// The repository name containing configuration.
    pub repository_name: String,

    /// How this repository was discovered.
    pub discovery_method: DiscoveryMethod,

    /// When this metadata was last refreshed.
    pub last_updated: DateTime<Utc>,
}

/// Abstract interface for accessing organization metadata repositories.
///
/// This trait defines the contract for discovering and loading configuration
/// from organization metadata repositories. Implementations handle the actual
/// interaction with GitHub APIs or other storage systems.
///
/// # Responsibilities
///
/// - Discover the metadata repository for an organization
/// - Load configuration files from the repository structure
/// - Validate repository structure and file formats
/// - List available repository types and team configurations
///
/// # Error Handling
///
/// All methods return `ConfigurationResult<T>` which wraps `ConfigurationError`.
/// Implementations should map external system errors to appropriate configuration
/// errors with sufficient context for debugging.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent access across
/// async tasks and threads.
///
/// # Examples
///
/// ```no_run
/// use config_manager::MetadataRepositoryProvider;
/// use async_trait::async_trait;
///
/// async fn load_config(provider: &dyn MetadataRepositoryProvider) {
///     let metadata_repo = provider
///         .discover_metadata_repository("my-org")
///         .await
///         .expect("Failed to discover metadata repository");
///
///     let global_defaults = provider
///         .load_global_defaults(&metadata_repo)
///         .await
///         .expect("Failed to load global defaults");
///
///     println!("Loaded configuration for: {}", metadata_repo.organization);
/// }
/// ```
#[async_trait]
pub trait MetadataRepositoryProvider: Send + Sync {
    /// Discover the metadata repository for an organization.
    ///
    /// This method attempts to find the configuration repository using
    /// configured discovery methods (explicit naming or topic search).
    ///
    /// # Arguments
    ///
    /// * `org` - The GitHub organization name
    ///
    /// # Returns
    ///
    /// Returns metadata about the discovered repository, including discovery
    /// method and last update timestamp.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::MetadataRepositoryNotFound` if:
    /// - No repository with the configured name exists
    /// - No repository with the configured topic is found
    /// - Multiple repositories match and disambiguation is needed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider) {
    /// let metadata_repo = provider
    ///     .discover_metadata_repository("my-org")
    ///     .await
    ///     .expect("Repository not found");
    ///
    /// assert_eq!(metadata_repo.organization, "my-org");
    /// # }
    /// ```
    async fn discover_metadata_repository(
        &self,
        org: &str,
    ) -> ConfigurationResult<MetadataRepository>;

    /// Load the global defaults configuration.
    ///
    /// Reads and parses `global-defaults.toml` from the root of the
    /// metadata repository. This file defines organization-wide baseline
    /// settings with override controls.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    ///
    /// # Returns
    ///
    /// Returns the parsed global defaults configuration structure.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::FileNotFound` if global-defaults.toml doesn't exist.
    /// Returns `ConfigurationError::ParseError` if the TOML is invalid.
    /// Returns `ConfigurationError::InvalidConfiguration` if required fields are missing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &config_manager::MetadataRepository) {
    /// let global_defaults = provider
    ///     .load_global_defaults(repo)
    ///     .await
    ///     .expect("Failed to load global defaults");
    /// # }
    /// ```
    async fn load_global_defaults(
        &self,
        repo: &MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults>;

    /// Load team-specific configuration overrides.
    ///
    /// Reads and parses `teams/{team_name}/config.toml` from the metadata
    /// repository. Returns `None` if the team has no specific configuration.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    /// * `team` - The team name (e.g., "backend-team")
    ///
    /// # Returns
    ///
    /// Returns `Some(TeamConfig)` if the team has configuration, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::ParseError` if the TOML is invalid.
    /// Returns `ConfigurationError::InvalidConfiguration` if the structure is malformed.
    ///
    /// Note: Missing team configuration is not an error; it returns `Ok(None)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &config_manager::MetadataRepository) {
    /// match provider.load_team_configuration(repo, "backend-team").await {
    ///     Ok(Some(config)) => println!("Team has custom config"),
    ///     Ok(None) => println!("Team uses global defaults"),
    ///     Err(e) => eprintln!("Error loading config: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_team_configuration(
        &self,
        repo: &MetadataRepository,
        team: &str,
    ) -> ConfigurationResult<Option<TeamConfig>>;

    /// Load repository type-specific configuration.
    ///
    /// Reads and parses `types/{repo_type}/config.toml` from the metadata
    /// repository. Returns `None` if the repository type has no configuration.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    /// * `repo_type` - The repository type name (e.g., "library", "service")
    ///
    /// # Returns
    ///
    /// Returns `Some(RepositoryTypeConfig)` if the type has configuration, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::ParseError` if the TOML is invalid.
    /// Returns `ConfigurationError::InvalidConfiguration` if the structure is malformed.
    ///
    /// Note: Missing repository type configuration is not an error; it returns `Ok(None)`.
    async fn load_repository_type_configuration(
        &self,
        repo: &MetadataRepository,
        repo_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>>;

    /// Load standard label definitions.
    ///
    /// Reads and parses `labels.toml` from the root of the metadata repository.
    /// Labels define issue/PR categorization standards for the organization.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    ///
    /// # Returns
    ///
    /// Returns a map of label names to their configurations. Returns an empty
    /// map if `labels.toml` doesn't exist (labels are optional).
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::ParseError` if the TOML is invalid.
    /// Returns `ConfigurationError::InvalidConfiguration` if label structure is malformed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &config_manager::MetadataRepository) {
    /// let labels = provider
    ///     .load_standard_labels(repo)
    ///     .await
    ///     .expect("Failed to load labels");
    ///
    /// for (name, config) in labels {
    ///     println!("Label: {} ({})", name, config.color);
    /// }
    /// # }
    /// ```
    async fn load_standard_labels(
        &self,
        repo: &MetadataRepository,
    ) -> ConfigurationResult<HashMap<String, LabelConfig>>;

    /// List all available repository types.
    ///
    /// Scans the `types/` directory to find all repository type configurations
    /// available in the organization.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    ///
    /// # Returns
    ///
    /// Returns a vector of repository type names (directory names under `types/`).
    /// Returns an empty vector if the `types/` directory doesn't exist.
    ///
    /// # Errors
    ///
    /// May return errors if the repository structure cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &config_manager::MetadataRepository) {
    /// let types = provider
    ///     .list_available_repository_types(repo)
    ///     .await
    ///     .expect("Failed to list types");
    ///
    /// for repo_type in types {
    ///     println!("Available type: {}", repo_type);
    /// }
    /// # }
    /// ```
    async fn list_available_repository_types(
        &self,
        repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>>;

    /// Validate the metadata repository structure.
    ///
    /// Checks that the repository contains required files and follows the
    /// expected directory structure. This is useful for diagnostics and
    /// validation during setup.
    ///
    /// # Arguments
    ///
    /// * `repo` - Metadata repository information
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the structure is valid.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::InvalidConfiguration` if:
    /// - `global-defaults.toml` is missing
    /// - Required directories have invalid structures
    /// - Path security violations are detected
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &config_manager::MetadataRepository) {
    /// provider
    ///     .validate_repository_structure(repo)
    ///     .await
    ///     .expect("Invalid repository structure");
    ///
    /// println!("Repository structure is valid");
    /// # }
    /// ```
    async fn validate_repository_structure(
        &self,
        repo: &MetadataRepository,
    ) -> ConfigurationResult<()>;

    /// List all available template repositories for an organization.
    ///
    /// Discovers template repositories by searching for repositories with the
    /// `reporoller-template` topic. Each template repository should contain a
    /// `.reporoller/template.toml` configuration file.
    ///
    /// # Arguments
    ///
    /// * `org` - The GitHub organization name
    ///
    /// # Returns
    ///
    /// Returns a vector of template names (repository names).
    /// Returns an empty vector if no template repositories are found.
    ///
    /// # Errors
    ///
    /// May return errors if the GitHub API cannot be accessed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider) {
    /// let templates = provider
    ///     .list_templates("my-org")
    ///     .await
    ///     .expect("Failed to list templates");
    ///
    /// for template_name in templates {
    ///     println!("Available template: {}", template_name);
    /// }
    /// # }
    /// ```
    async fn list_templates(&self, org: &str) -> ConfigurationResult<Vec<String>>;

    /// Load template configuration from a template repository.
    ///
    /// Fetches and parses `.reporoller/template.toml` from the specified
    /// template repository.
    ///
    /// # Arguments
    ///
    /// * `org` - The GitHub organization name
    /// * `template_name` - The template repository name
    ///
    /// # Returns
    ///
    /// Returns the parsed template configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::FileNotFound` if template repository or template.toml doesn't exist.
    /// Returns `ConfigurationError::ParseError` if the TOML is invalid.
    /// Returns `ConfigurationError::InvalidConfiguration` if required fields are missing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider) {
    /// let config = provider
    ///     .load_template_configuration("my-org", "rust-library")
    ///     .await
    ///     .expect("Failed to load template config");
    ///
    /// println!("Template: {}", config.template.name);
    /// # }
    /// ```
    async fn load_template_configuration(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<crate::template_config::TemplateConfig>;
}
