//! Metadata repository provider system for organization configuration management.
//!
//! This module provides the foundational traits and structures for discovering, accessing,
//! and validating metadata repositories that contain organization-specific configuration data.
//! The system supports multiple discovery methods and provides comprehensive error handling
//! for repository access operations.
//!
//! # Architecture
//!
//! The metadata provider system uses a trait-based approach to abstract repository access:
//!
//! - `MetadataRepositoryProvider` - Core trait for repository operations
//! - `MetadataRepository` - Represents a discovered metadata repository
//! - `DiscoveryMethod` - Tracks how a repository was discovered
//!
//! # Discovery Methods
//!
//! The system supports two discovery methods:
//!
//! 1. **Configuration-based**: Repository name specified in application configuration
//! 2. **Topic-based**: Repository tagged with `template-metadata` topic
//!
//! # Examples
//!
//! ```rust
//! use config_manager::metadata::{MetadataRepositoryProvider, DiscoveryMethod};
//!
//! async fn discover_org_config(provider: &dyn MetadataRepositoryProvider) {
//!     match provider.discover_metadata_repository("acme-corp").await {
//!         Ok(repo) => println!("Found metadata repository: {}", repo.repository_name),
//!         Err(e) => eprintln!("Discovery failed: {}", e),
//!     }
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::organization::{RepositoryTypeConfig, TeamConfig};
use crate::settings::GlobalDefaults;

// Unit tests for metadata provider functionality
#[path = "metadata_tests.rs"]
#[cfg(test)]
mod tests;

/// Result type for metadata repository operations.
///
/// This type alias provides a consistent error handling pattern across all
/// metadata repository operations, using the configuration management error types.
pub type MetadataResult<T> = Result<T, crate::ConfigurationError>;

/// Core trait for metadata repository providers.
///
/// This trait abstracts access to organization configuration repositories, supporting
/// multiple discovery methods and providing comprehensive validation capabilities.
/// All implementations must be thread-safe to support concurrent access patterns.
///
/// # Discovery Process
///
/// 1. **Repository Discovery**: Locate the metadata repository using configured discovery method
/// 2. **Structure Validation**: Verify repository contains required directories and files
/// 3. **Access Validation**: Ensure the provider has necessary permissions
/// 4. **Configuration Loading**: Load and parse configuration files from repository
///
/// # Error Handling
///
/// All methods return `MetadataResult<T>` to provide consistent error handling:
///
/// - **Discovery Errors**: Repository not found, multiple candidates, access denied
/// - **Validation Errors**: Invalid structure, missing required files, parse errors
/// - **Network Errors**: API failures, timeouts, rate limiting
/// - **Permission Errors**: Insufficient access rights, authentication failures
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent access across async tasks.
/// Providers should implement appropriate internal synchronization for shared state.
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::MetadataRepositoryProvider;
/// use async_trait::async_trait;
///
/// struct MockProvider;
///
/// #[async_trait]
/// impl MetadataRepositoryProvider for MockProvider {
///     async fn discover_metadata_repository(&self, org: &str) -> MetadataResult<MetadataRepository> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     async fn validate_repository_structure(&self, repo: &MetadataRepository) -> MetadataResult<()> {
///         // Implementation details...
///         todo!("Not implemented")
///     }
///
///     // ... other required methods
/// }
/// ```
#[async_trait]
pub trait MetadataRepositoryProvider: Send + Sync {
    /// Discover the metadata repository for the specified organization.
    ///
    /// This method attempts to locate the organization's metadata repository using
    /// the configured discovery methods. It tries configuration-based discovery first,
    /// then falls back to topic-based discovery if configured.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name to search for metadata repository
    ///
    /// # Returns
    ///
    /// * `Ok(MetadataRepository)` - Successfully discovered repository with metadata
    /// * `Err(ConfigurationError::RepositoryNotFound)` - No metadata repository found
    /// * `Err(ConfigurationError::MultipleRepositoriesFound)` - Multiple candidates found
    /// * `Err(ConfigurationError::AccessDenied)` - Insufficient permissions to access repository
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::MetadataRepositoryProvider;
    /// # async fn example(provider: &dyn MetadataRepositoryProvider) {
    /// match provider.discover_metadata_repository("acme-corp").await {
    ///     Ok(repo) => {
    ///         println!("Found repository: {} via {:?}",
    ///                  repo.repository_name, repo.discovery_method);
    ///     },
    ///     Err(e) => eprintln!("Discovery failed: {}", e),
    /// }
    /// # }
    /// ```
    async fn discover_metadata_repository(&self, org: &str) -> MetadataResult<MetadataRepository>;

    /// Validate the structure of a metadata repository.
    ///
    /// This method checks that the repository contains all required directories and files
    /// for organization configuration management. It validates the presence of:
    ///
    /// - `global/` directory with `defaults.toml`
    /// - `teams/` directory (may be empty)
    /// - `types/` directory (may be empty)
    /// - Optional `schemas/` directory for validation schemas
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Repository structure is valid
    /// * `Err(ConfigurationError::InvalidRepositoryStructure)` - Missing required directories/files
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access repository contents
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// if let Err(e) = provider.validate_repository_structure(repo).await {
    ///     eprintln!("Repository structure validation failed: {}", e);
    ///     return;
    /// }
    /// println!("Repository structure is valid");
    /// # }
    /// ```
    async fn validate_repository_structure(&self, repo: &MetadataRepository) -> MetadataResult<()>;

    /// Load global default configuration from the metadata repository.
    ///
    /// This method loads and parses the `global/defaults.toml` file containing
    /// organization-wide baseline settings. The global defaults provide the foundation
    /// for the configuration hierarchy and define override policies.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(GlobalDefaults)` - Successfully loaded and parsed global configuration
    /// * `Err(ConfigurationError::FileNotFound)` - global/defaults.toml missing
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_global_defaults(repo).await {
    ///     Ok(defaults) => {
    ///         println!("Loaded global defaults with {} repository settings",
    ///                  defaults.repository.is_some() as usize);
    ///     },
    ///     Err(e) => eprintln!("Failed to load global defaults: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_global_defaults(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<GlobalDefaults>;

    /// Load team-specific configuration from the metadata repository.
    ///
    /// This method loads and parses team configuration from `teams/{team}/config.toml`.
    /// Team configurations provide team-specific overrides and additions to global settings.
    /// Returns `None` if the team configuration does not exist.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `team` - The team name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(TeamConfig))` - Successfully loaded team configuration
    /// * `Ok(None)` - Team configuration file does not exist (valid scenario)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_team_configuration(repo, "platform-team").await {
    ///     Ok(Some(team_config)) => {
    ///         println!("Loaded configuration for platform-team");
    ///     },
    ///     Ok(None) => {
    ///         println!("No configuration found for platform-team (using defaults)");
    ///     },
    ///     Err(e) => eprintln!("Failed to load team configuration: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_team_configuration(
        &self,
        repo: &MetadataRepository,
        team: &str,
    ) -> MetadataResult<Option<TeamConfig>>;

    /// Load repository type configuration from the metadata repository.
    ///
    /// This method loads and parses repository type configuration from
    /// `types/{repo_type}/config.toml`. Repository type configurations provide
    /// type-specific settings that override global defaults but are overridden by
    /// team and template settings. Returns `None` if the type configuration does not exist.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `repo_type` - The repository type name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(RepositoryTypeConfig))` - Successfully loaded repository type configuration
    /// * `Ok(None)` - Repository type configuration file does not exist (valid scenario)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::ValidationError)` - Configuration violates schema
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_repository_type_configuration(repo, "library").await {
    ///     Ok(Some(type_config)) => {
    ///         println!("Loaded configuration for library repositories");
    ///     },
    ///     Ok(None) => {
    ///         println!("No specific configuration for library repositories");
    ///     },
    ///     Err(e) => eprintln!("Failed to load repository type configuration: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_repository_type_configuration(
        &self,
        repo: &MetadataRepository,
        repo_type: &str,
    ) -> MetadataResult<Option<RepositoryTypeConfig>>;

    /// List all available repository types defined in the metadata repository.
    ///
    /// This method scans the `types/` directory to discover all available repository
    /// types that have configuration files. Repository types are determined by the
    /// presence of `config.toml` files in subdirectories of `types/`.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to scan
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of available repository type names
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access types directory
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.list_available_repository_types(repo).await {
    ///     Ok(types) => {
    ///         println!("Available repository types: {}", types.join(", "));
    ///     },
    ///     Err(e) => eprintln!("Failed to list repository types: {}", e),
    /// }
    /// # }
    /// ```
    async fn list_available_repository_types(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<Vec<String>>;

    /// Load standard label definitions from the metadata repository.
    ///
    /// This method loads standard label configurations that can be shared across
    /// multiple teams and repository types. Standard labels are typically defined
    /// in `global/labels.toml` or similar location within the metadata repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, LabelConfig>)` - Map of label names to configurations
    /// * `Err(ConfigurationError::FileNotFound)` - Labels file not found (may be optional)
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access labels file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepositoryProvider, MetadataRepository};
    /// # async fn example(provider: &dyn MetadataRepositoryProvider, repo: &MetadataRepository) {
    /// match provider.load_standard_labels(repo).await {
    ///     Ok(labels) => {
    ///         println!("Loaded {} standard labels", labels.len());
    ///     },
    ///     Err(e) => eprintln!("Failed to load standard labels: {}", e),
    /// }
    /// # }
    /// ```
    async fn load_standard_labels(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<HashMap<String, crate::LabelConfig>>;
}

/// Represents a discovered metadata repository containing organization configuration.
///
/// This structure contains information about a metadata repository that has been
/// discovered and validated for use in organization configuration management.
/// It tracks the discovery method used and maintains timestamps for cache invalidation.
///
/// # Fields
///
/// * `organization` - The GitHub organization name that owns this repository
/// * `repository_name` - The name of the metadata repository
/// * `discovery_method` - How this repository was discovered
/// * `last_updated` - Timestamp of last known update for cache invalidation
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
/// use chrono::Utc;
///
/// let repo = MetadataRepository {
///     organization: "acme-corp".to_string(),
///     repository_name: "acme-corp-config".to_string(),
///     discovery_method: DiscoveryMethod::ConfigurationBased {
///         repository_name: "acme-corp-config".to_string(),
///     },
///     last_updated: Utc::now(),
/// };
///
/// println!("Metadata repository: {}/{}", repo.organization, repo.repository_name);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataRepository {
    /// The GitHub organization that owns this metadata repository.
    ///
    /// This must match the organization for which configuration is being loaded
    /// to ensure proper access control and configuration isolation.
    pub organization: String,

    /// The name of the metadata repository within the organization.
    ///
    /// This is used to construct API calls and identify the repository
    /// in error messages and audit logs.
    pub repository_name: String,

    /// The method used to discover this metadata repository.
    ///
    /// This information is useful for troubleshooting discovery issues and
    /// understanding the configuration source in audit logs.
    pub discovery_method: DiscoveryMethod,

    /// Timestamp of the last known update to this repository.
    ///
    /// This is used for cache invalidation and determining when to refresh
    /// cached configuration data. The timestamp should reflect the last
    /// Git commit or similar repository modification time.
    pub last_updated: DateTime<Utc>,
}

impl MetadataRepository {
    /// Create a new metadata repository instance.
    ///
    /// # Arguments
    ///
    /// * `organization` - The GitHub organization name
    /// * `repository_name` - The metadata repository name
    /// * `discovery_method` - How the repository was discovered
    /// * `last_updated` - Timestamp of last repository update
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// use chrono::Utc;
    ///
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::TopicBased {
    ///         topic: "template-metadata".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    /// ```
    pub fn new(
        organization: String,
        repository_name: String,
        discovery_method: DiscoveryMethod,
        last_updated: DateTime<Utc>,
    ) -> Self {
        Self {
            organization,
            repository_name,
            discovery_method,
            last_updated,
        }
    }

    /// Get the full repository identifier in the format "org/repo".
    ///
    /// This is useful for API calls and logging that require the full repository path.
    ///
    /// # Returns
    ///
    /// A string in the format "{organization}/{repository_name}"
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use config_manager::metadata::{MetadataRepository, DiscoveryMethod};
    /// # use chrono::Utc;
    /// let repo = MetadataRepository::new(
    ///     "acme-corp".to_string(),
    ///     "acme-corp-config".to_string(),
    ///     DiscoveryMethod::ConfigurationBased {
    ///         repository_name: "acme-corp-config".to_string(),
    ///     },
    ///     Utc::now(),
    /// );
    ///
    /// assert_eq!(repo.full_name(), "acme-corp/acme-corp-config");
    /// ```
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.organization, self.repository_name)
    }
}

/// Enumeration of supported metadata repository discovery methods.
///
/// The system supports multiple methods for discovering organization metadata repositories,
/// allowing flexibility in how organizations structure their configuration management.
/// Discovery methods are tried in order of preference: configuration-based first,
/// then topic-based if enabled.
///
/// # Variants
///
/// * `ConfigurationBased` - Repository name specified in application configuration
/// * `TopicBased` - Repository discovered by GitHub topic search
///
/// # Examples
///
/// ```rust
/// use config_manager::metadata::DiscoveryMethod;
///
/// // Configuration-based discovery
/// let config_method = DiscoveryMethod::ConfigurationBased {
///     repository_name: "acme-corp-config".to_string(),
/// };
///
/// // Topic-based discovery
/// let topic_method = DiscoveryMethod::TopicBased {
///     topic: "template-metadata".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Repository name is explicitly specified in application configuration.
    ///
    /// This is the preferred discovery method as it provides deterministic behavior
    /// and doesn't require additional API calls to search for repositories.
    /// The repository name is typically configured during application deployment.
    ///
    /// # Fields
    ///
    /// * `repository_name` - The exact name of the metadata repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "org-repo-config".to_string(),
    /// };
    /// ```
    ConfigurationBased {
        /// The exact name of the metadata repository as configured.
        repository_name: String,
    },

    /// Repository is discovered by searching for repositories with a specific GitHub topic.
    ///
    /// This method provides flexibility for organizations that prefer to use GitHub's
    /// topic system for repository classification. It requires an additional API call
    /// to search for repositories with the specified topic.
    ///
    /// # Fields
    ///
    /// * `topic` - The GitHub topic to search for (typically "template-metadata")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// ```
    TopicBased {
        /// The GitHub topic used to identify metadata repositories.
        topic: String,
    },
}

impl DiscoveryMethod {
    /// Check if this discovery method requires additional API calls.
    ///
    /// Configuration-based discovery can directly access the repository,
    /// while topic-based discovery requires a search API call first.
    ///
    /// # Returns
    ///
    /// * `true` if the method requires a search API call
    /// * `false` if the method can directly access the repository
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let config_method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "config-repo".to_string(),
    /// };
    /// assert!(!config_method.requires_search());
    ///
    /// let topic_method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// assert!(topic_method.requires_search());
    /// ```
    pub fn requires_search(&self) -> bool {
        matches!(self, DiscoveryMethod::TopicBased { .. })
    }

    /// Get a human-readable description of this discovery method.
    ///
    /// This is useful for logging and error messages to help users understand
    /// how the metadata repository was discovered.
    ///
    /// # Returns
    ///
    /// A string describing the discovery method and its parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::metadata::DiscoveryMethod;
    ///
    /// let config_method = DiscoveryMethod::ConfigurationBased {
    ///     repository_name: "acme-config".to_string(),
    /// };
    /// assert_eq!(config_method.description(), "configuration-based (repository: acme-config)");
    ///
    /// let topic_method = DiscoveryMethod::TopicBased {
    ///     topic: "template-metadata".to_string(),
    /// };
    /// assert_eq!(topic_method.description(), "topic-based (topic: template-metadata)");
    /// ```
    pub fn description(&self) -> String {
        match self {
            DiscoveryMethod::ConfigurationBased { repository_name } => {
                format!("configuration-based (repository: {})", repository_name)
            }
            DiscoveryMethod::TopicBased { topic } => {
                format!("topic-based (topic: {})", topic)
            }
        }
    }
}
