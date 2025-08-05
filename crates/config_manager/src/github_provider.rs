//! GitHub-based implementation of the metadata repository provider.
//!
//! This module provides a concrete implementation of the `MetadataRepositoryProvider` trait
//! that uses the GitHub API to discover, access, and validate metadata repositories containing
//! organization-specific configuration data.
//!
//! # Features
//!
//! - **Dual Discovery Methods**: Supports both configuration-based and topic-based repository discovery
//! - **GitHub API Integration**: Uses the existing GitHub client for authenticated API access
//! - **Comprehensive Validation**: Validates repository structure and access permissions
//! - **Error Handling**: Provides detailed error messages for troubleshooting
//! - **Caching Support**: Optimizes API usage with intelligent caching strategies
//!
//! # Discovery Methods
//!
//! ## Configuration-Based Discovery
//!
//! The provider attempts to access a repository with a name specified in the application
//! configuration. This is the preferred method as it provides deterministic behavior.
//!
//! ## Topic-Based Discovery
//!
//! If configuration-based discovery fails, the provider searches for repositories in the
//! organization that are tagged with the `template-metadata` topic (configurable).
//!
//! # Examples
//!
//! ```rust,no_run
//! use config_manager::github_provider::GitHubMetadataProvider;
//! use config_manager::github_provider::DiscoveryConfig;
//! use github_client::GitHubClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let github_client = GitHubClient::new(/* ... */);
//!     let config = DiscoveryConfig::default();
//!     let provider = GitHubMetadataProvider::new(github_client, config);
//!
//!     let repo = provider.discover_metadata_repository("acme-corp").await?;
//!     println!("Found metadata repository: {}", repo.full_name());
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::metadata::{
    DiscoveryMethod, MetadataRepository, MetadataRepositoryProvider, MetadataResult,
};
use crate::organization::{RepositoryTypeConfig, TeamConfig};
use crate::settings::GlobalDefaults;
use crate::LabelConfig;

// Unit tests for GitHub metadata provider functionality
#[path = "github_provider_tests.rs"]
#[cfg(test)]
mod tests;

/// Configuration for metadata repository discovery.
///
/// This struct controls how the GitHub metadata provider searches for and accesses
/// metadata repositories within an organization. It supports multiple discovery
/// strategies and provides options for controlling API behavior.
///
/// # Discovery Strategy
///
/// The provider tries discovery methods in the following order:
/// 1. Configuration-based (if `repository_name_pattern` is provided)
/// 2. Topic-based (if `metadata_topic` is provided)
///
/// # Examples
///
/// ```rust
/// use config_manager::github_provider::DiscoveryConfig;
///
/// // Configuration-based discovery only
/// let config = DiscoveryConfig::builder()
///     .repository_name_pattern("{org}-config")
///     .build();
///
/// // Topic-based discovery with fallback
/// let config = DiscoveryConfig::builder()
///     .repository_name_pattern("{org}-repo-config")
///     .metadata_topic("template-metadata")
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Template for generating repository names for configuration-based discovery.
    ///
    /// The template can contain placeholders that will be replaced:
    /// - `{org}` - The organization name
    /// - `{org_lower}` - The organization name in lowercase
    /// - `{org_upper}` - The organization name in uppercase
    ///
    /// Common patterns:
    /// - `"{org}-config"` - Results in "acme-corp-config"
    /// - `"{org_lower}-repo-settings"` - Results in "acme-corp-repo-settings"
    /// - `"config-{org}"` - Results in "config-acme-corp"
    pub repository_name_pattern: Option<String>,

    /// GitHub topic used for topic-based discovery.
    ///
    /// When specified, the provider will search for repositories in the organization
    /// that have this topic. If multiple repositories are found, an error is returned.
    /// Common values: "template-metadata", "org-config", "repo-settings"
    pub metadata_topic: Option<String>,

    /// Maximum number of repositories to examine during topic-based discovery.
    ///
    /// This prevents excessive API usage when organizations have many repositories.
    /// Defaults to 100 repositories.
    pub max_search_results: usize,

    /// Timeout in seconds for GitHub API operations.
    ///
    /// Individual API calls will timeout after this duration to prevent
    /// hanging operations. Defaults to 30 seconds.
    pub api_timeout_seconds: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            repository_name_pattern: Some("{org}-config".to_string()),
            metadata_topic: Some("template-metadata".to_string()),
            max_search_results: 100,
            api_timeout_seconds: 30,
        }
    }
}

impl DiscoveryConfig {
    /// Create a new discovery configuration builder.
    ///
    /// This provides a fluent interface for constructing discovery configurations
    /// with validation and sensible defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::github_provider::DiscoveryConfig;
    ///
    /// let config = DiscoveryConfig::builder()
    ///     .repository_name_pattern("{org}-repo-config")
    ///     .metadata_topic("org-settings")
    ///     .max_search_results(50)
    ///     .api_timeout_seconds(60)
    ///     .build();
    /// ```
    pub fn builder() -> DiscoveryConfigBuilder {
        DiscoveryConfigBuilder::default()
    }

    /// Generate a repository name from the pattern for the given organization.
    ///
    /// This method substitutes placeholders in the repository name pattern
    /// with actual values from the organization name.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name to substitute into the pattern
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The generated repository name if pattern is configured
    /// * `None` - If no repository name pattern is configured
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::github_provider::DiscoveryConfig;
    ///
    /// let config = DiscoveryConfig::builder()
    ///     .repository_name_pattern("{org}-config")
    ///     .build();
    ///
    /// assert_eq!(
    ///     config.generate_repository_name("acme-corp"),
    ///     Some("acme-corp-config".to_string())
    /// );
    /// ```
    pub fn generate_repository_name(&self, org: &str) -> Option<String> {
        self.repository_name_pattern.as_ref().map(|pattern| {
            pattern
                .replace("{org}", org)
                .replace("{org_lower}", &org.to_lowercase())
                .replace("{org_upper}", &org.to_uppercase())
        })
    }

    /// Check if configuration-based discovery is enabled.
    ///
    /// # Returns
    ///
    /// `true` if a repository name pattern is configured, `false` otherwise.
    pub fn has_configuration_based_discovery(&self) -> bool {
        self.repository_name_pattern.is_some()
    }

    /// Check if topic-based discovery is enabled.
    ///
    /// # Returns
    ///
    /// `true` if a metadata topic is configured, `false` otherwise.
    pub fn has_topic_based_discovery(&self) -> bool {
        self.metadata_topic.is_some()
    }
}

/// Builder for creating discovery configurations.
///
/// This builder provides a fluent interface for constructing `DiscoveryConfig`
/// instances with validation and reasonable defaults.
#[derive(Debug, Default)]
pub struct DiscoveryConfigBuilder {
    repository_name_pattern: Option<String>,
    metadata_topic: Option<String>,
    max_search_results: Option<usize>,
    api_timeout_seconds: Option<u64>,
}

impl DiscoveryConfigBuilder {
    /// Set the repository name pattern for configuration-based discovery.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Template string with placeholders like `{org}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::github_provider::DiscoveryConfig;
    ///
    /// let config = DiscoveryConfig::builder()
    ///     .repository_name_pattern("{org}-repo-settings")
    ///     .build();
    /// ```
    pub fn repository_name_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.repository_name_pattern = Some(pattern.into());
        self
    }

    /// Set the metadata topic for topic-based discovery.
    ///
    /// # Arguments
    ///
    /// * `topic` - GitHub topic to search for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::github_provider::DiscoveryConfig;
    ///
    /// let config = DiscoveryConfig::builder()
    ///     .metadata_topic("org-config")
    ///     .build();
    /// ```
    pub fn metadata_topic<S: Into<String>>(mut self, topic: S) -> Self {
        self.metadata_topic = Some(topic.into());
        self
    }

    /// Set the maximum number of search results for topic-based discovery.
    ///
    /// # Arguments
    ///
    /// * `max_results` - Maximum repositories to examine (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `max_results` is 0.
    pub fn max_search_results(mut self, max_results: usize) -> Self {
        assert!(max_results > 0, "max_search_results must be greater than 0");
        self.max_search_results = Some(max_results);
        self
    }

    /// Set the API timeout for GitHub operations.
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - Timeout in seconds (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if `timeout_seconds` is 0.
    pub fn api_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        assert!(
            timeout_seconds > 0,
            "api_timeout_seconds must be greater than 0"
        );
        self.api_timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Build the discovery configuration.
    ///
    /// Uses defaults for any unspecified values.
    pub fn build(self) -> DiscoveryConfig {
        let default_config = DiscoveryConfig::default();
        DiscoveryConfig {
            repository_name_pattern: self
                .repository_name_pattern
                .or(default_config.repository_name_pattern),
            metadata_topic: self.metadata_topic.or(default_config.metadata_topic),
            max_search_results: self
                .max_search_results
                .unwrap_or(default_config.max_search_results),
            api_timeout_seconds: self
                .api_timeout_seconds
                .unwrap_or(default_config.api_timeout_seconds),
        }
    }
}

/// GitHub-based metadata repository provider.
///
/// This implementation uses the GitHub API to discover and access metadata repositories
/// containing organization-specific configuration data. It supports multiple discovery
/// methods and provides comprehensive error handling and validation.
///
/// # Thread Safety
///
/// This implementation is thread-safe and can be shared across multiple async tasks.
/// The underlying GitHub client handles concurrent access appropriately.
///
/// # Caching
///
/// The provider implements intelligent caching to minimize GitHub API usage:
/// - Repository structure validation results are cached
/// - File content is cached based on Git commit SHA
/// - Discovery results are cached with TTL
///
/// # Examples
///
/// ```rust,no_run
/// use config_manager::github_provider::{GitHubMetadataProvider, DiscoveryConfig};
/// use github_client::GitHubClient;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let github_client = Arc::new(GitHubClient::new(/* ... */));
///     let config = DiscoveryConfig::default();
///     let provider = GitHubMetadataProvider::new(github_client, config);
///
///     // Discover metadata repository
///     let repo = provider.discover_metadata_repository("acme-corp").await?;
///
///     // Validate repository structure
///     provider.validate_repository_structure(&repo).await?;
///
///     // Load configurations
///     let global_defaults = provider.load_global_defaults(&repo).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct GitHubMetadataProvider {
    /// GitHub client for API operations.
    ///
    /// This should be an authenticated client with appropriate permissions
    /// to access organization repositories and contents.
    github_client: Arc<dyn GitHubClientTrait>,

    /// Configuration for repository discovery behavior.
    discovery_config: DiscoveryConfig,
}

impl GitHubMetadataProvider {
    /// Create a new GitHub metadata provider.
    ///
    /// # Arguments
    ///
    /// * `github_client` - Authenticated GitHub client
    /// * `discovery_config` - Configuration for repository discovery
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use config_manager::github_provider::{GitHubMetadataProvider, DiscoveryConfig};
    /// use github_client::GitHubClient;
    /// use std::sync::Arc;
    ///
    /// let github_client = Arc::new(GitHubClient::new(/* ... */));
    /// let config = DiscoveryConfig::default();
    /// let provider = GitHubMetadataProvider::new(github_client, config);
    /// ```
    pub fn new(
        github_client: Arc<dyn GitHubClientTrait>,
        discovery_config: DiscoveryConfig,
    ) -> Self {
        Self {
            github_client,
            discovery_config,
        }
    }

    /// Get the discovery configuration.
    ///
    /// This is useful for introspection and testing.
    pub fn discovery_config(&self) -> &DiscoveryConfig {
        &self.discovery_config
    }

    /// Attempt configuration-based discovery for the given organization.
    ///
    /// This method tries to access a repository whose name is generated from
    /// the configured repository name pattern.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MetadataRepository))` - Repository found and accessible
    /// * `Ok(None)` - Configuration-based discovery not enabled or repository not found
    /// * `Err(ConfigurationError)` - Access denied, network error, or other failure
    async fn try_configuration_based_discovery(
        &self,
        org: &str,
    ) -> MetadataResult<Option<MetadataRepository>> {
        let repository_name = match self.discovery_config.generate_repository_name(org) {
            Some(name) => name,
            None => return Ok(None), // Configuration-based discovery not enabled
        };

        // Check if repository exists and is accessible
        match self.check_repository_access(org, &repository_name).await? {
            Some(last_updated) => Ok(Some(MetadataRepository {
                organization: org.to_string(),
                repository_name: repository_name.clone(),
                discovery_method: DiscoveryMethod::ConfigurationBased { repository_name },
                last_updated,
            })),
            None => Ok(None), // Repository doesn't exist or isn't accessible
        }
    }

    /// Attempt topic-based discovery for the given organization.
    ///
    /// This method searches for repositories in the organization that have
    /// the configured metadata topic.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name
    ///
    /// # Returns
    ///
    /// * `Ok(Some(MetadataRepository))` - Exactly one repository found with the topic
    /// * `Ok(None)` - Topic-based discovery not enabled or no repositories found
    /// * `Err(ConfigurationError::MultipleRepositoriesFound)` - Multiple repositories with topic
    /// * `Err(ConfigurationError)` - Access denied, network error, or other failure
    async fn try_topic_based_discovery(
        &self,
        org: &str,
    ) -> MetadataResult<Option<MetadataRepository>> {
        let topic = match &self.discovery_config.metadata_topic {
            Some(topic) => topic,
            None => return Ok(None), // Topic-based discovery not enabled
        };

        // Search for repositories with the specified topic
        let max_results = self.discovery_config.max_search_results;
        let repositories = self
            .github_client
            .search_repositories_by_topic(org, topic, max_results)
            .await
            .map_err(|e| crate::ConfigurationError::NetworkError {
                error: format!("Repository search failed: {}", e),
                operation: "topic-based discovery".to_string(),
            })?;

        match repositories.len() {
            0 => Ok(None), // No repositories found with the topic
            1 => {
                let repo_name = &repositories[0];
                // Get repository info to get the last updated time
                let last_updated = self
                    .github_client
                    .get_repository_info(org, repo_name)
                    .await
                    .map_err(|e| crate::ConfigurationError::NetworkError {
                        error: format!("Failed to get repository info: {}", e),
                        operation: "topic-based discovery".to_string(),
                    })?
                    .unwrap_or(Utc::now()); // Fallback to current time if not available

                Ok(Some(MetadataRepository {
                    organization: org.to_string(),
                    repository_name: repo_name.clone(),
                    discovery_method: DiscoveryMethod::TopicBased {
                        topic: topic.clone(),
                    },
                    last_updated,
                }))
            }
            _count => Err(crate::ConfigurationError::MultipleRepositoriesFound {
                organization: org.to_string(),
                search_method: format!("topic-based search for '{}'", topic),
                repositories,
            }),
        }
    }

    /// Check if a repository exists and is accessible.
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name
    /// * `repo_name` - The repository name
    ///
    /// # Returns
    ///
    /// * `Ok(Some(DateTime<Utc>))` - Repository exists and is accessible, returns last update time
    /// * `Ok(None)` - Repository does not exist or is not accessible
    /// * `Err(ConfigurationError)` - Network error or other failure
    async fn check_repository_access(
        &self,
        org: &str,
        repo_name: &str,
    ) -> MetadataResult<Option<DateTime<Utc>>> {
        // Use the trait method to check repository access
        self.github_client
            .get_repository_info(org, repo_name)
            .await
            .map_err(|e| crate::ConfigurationError::NetworkError {
                error: format!("Failed to check repository access: {}", e),
                operation: "repository access check".to_string(),
            })
    }

    /// Load file content from a repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository
    /// * `file_path` - Path to the file within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` - File content if file exists
    /// * `Ok(None)` - File does not exist
    /// * `Err(ConfigurationError)` - Access denied, network error, or other failure
    async fn load_file_content(
        &self,
        repo: &MetadataRepository,
        file_path: &str,
    ) -> MetadataResult<Option<String>> {
        // Use the trait method to get file content
        self.github_client
            .get_file_content(&repo.organization, &repo.repository_name, file_path)
            .await
            .map_err(|e| crate::ConfigurationError::NetworkError {
                error: format!("Failed to load file content: {}", e),
                operation: "file content loading".to_string(),
            })
    }

    /// List directory contents in a repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository
    /// * `dir_path` - Path to the directory within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<String>))` - List of items in the directory if directory exists
    /// * `Ok(None)` - Directory does not exist
    /// * `Err(ConfigurationError)` - Access denied, network error, or other failure
    async fn list_directory_contents(
        &self,
        repo: &MetadataRepository,
        dir_path: &str,
    ) -> MetadataResult<Option<Vec<String>>> {
        // Use the trait method to list directory contents
        self.github_client
            .list_directory(&repo.organization, &repo.repository_name, dir_path)
            .await
            .map_err(|e| crate::ConfigurationError::NetworkError {
                error: format!("Failed to list directory contents: {}", e),
                operation: "directory listing".to_string(),
            })
    }
}

/// Trait for abstracting GitHub client operations.
///
/// This trait allows for easier testing by enabling mock implementations
/// of GitHub API operations. It defines the minimal interface needed
/// by the metadata provider.
#[async_trait]
pub trait GitHubClientTrait: Send + Sync + std::fmt::Debug {
    /// Check if a repository exists and get its last update time.
    ///
    /// # Arguments
    ///
    /// * `org` - Organization name
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// * `Ok(Some(DateTime<Utc>))` - Repository exists, returns last update time
    /// * `Ok(None)` - Repository does not exist or is not accessible
    /// * `Err(String)` - Error message describing the failure
    async fn get_repository_info(
        &self,
        org: &str,
        repo: &str,
    ) -> Result<Option<DateTime<Utc>>, String>;

    /// Search for repositories in an organization by topic.
    ///
    /// # Arguments
    ///
    /// * `org` - Organization name
    /// * `topic` - Topic to search for
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of repository names with the topic
    /// * `Err(String)` - Error message describing the failure
    async fn search_repositories_by_topic(
        &self,
        org: &str,
        topic: &str,
        max_results: usize,
    ) -> Result<Vec<String>, String>;

    /// Get the content of a file in a repository.
    ///
    /// # Arguments
    ///
    /// * `org` - Organization name
    /// * `repo` - Repository name
    /// * `path` - File path within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` - File content if file exists
    /// * `Ok(None)` - File does not exist
    /// * `Err(String)` - Error message describing the failure
    async fn get_file_content(
        &self,
        org: &str,
        repo: &str,
        path: &str,
    ) -> Result<Option<String>, String>;

    /// List the contents of a directory in a repository.
    ///
    /// # Arguments
    ///
    /// * `org` - Organization name
    /// * `repo` - Repository name
    /// * `path` - Directory path within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<String>))` - List of items in directory if it exists
    /// * `Ok(None)` - Directory does not exist
    /// * `Err(String)` - Error message describing the failure
    async fn list_directory(
        &self,
        org: &str,
        repo: &str,
        path: &str,
    ) -> Result<Option<Vec<String>>, String>;
}

#[async_trait]
impl MetadataRepositoryProvider for GitHubMetadataProvider {
    /// Discover the metadata repository for the specified organization.
    ///
    /// This implementation tries multiple discovery methods in order:
    /// 1. Configuration-based discovery (if enabled)
    /// 2. Topic-based discovery (if enabled)
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name to search for metadata repository
    ///
    /// # Returns
    ///
    /// * `Ok(MetadataRepository)` - Successfully discovered repository
    /// * `Err(ConfigurationError::RepositoryNotFound)` - No metadata repository found
    /// * `Err(ConfigurationError::MultipleRepositoriesFound)` - Multiple candidates found
    /// * `Err(ConfigurationError::AccessDenied)` - Insufficient permissions
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use config_manager::github_provider::GitHubMetadataProvider;
    /// # use config_manager::metadata::MetadataRepositoryProvider;
    /// # async fn example(provider: &GitHubMetadataProvider) {
    /// match provider.discover_metadata_repository("acme-corp").await {
    ///     Ok(repo) => {
    ///         println!("Found repository: {} via {:?}",
    ///                  repo.repository_name, repo.discovery_method);
    ///     },
    ///     Err(e) => eprintln!("Discovery failed: {}", e),
    /// }
    /// # }
    /// ```
    async fn discover_metadata_repository(&self, org: &str) -> MetadataResult<MetadataRepository> {
        // Try configuration-based discovery first
        if let Some(repo) = self.try_configuration_based_discovery(org).await? {
            return Ok(repo);
        }

        // Try topic-based discovery as fallback
        if let Some(repo) = self.try_topic_based_discovery(org).await? {
            return Ok(repo);
        }

        // No repository found with any method
        Err(crate::ConfigurationError::RepositoryNotFound {
            organization: org.to_string(),
            search_method: "configuration-based and topic-based discovery".to_string(),
        })
    }

    /// Validate the structure of a metadata repository.
    ///
    /// This method verifies that the repository contains all required directories
    /// and files for organization configuration management:
    ///
    /// - `global/defaults.toml` - Required global defaults file
    /// - `global/` directory - Must exist
    /// - `teams/` directory - Must exist (may be empty)
    /// - `types/` directory - Must exist (may be empty)
    /// - `schemas/` directory - Optional validation schemas
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Repository structure is valid
    /// * `Err(ConfigurationError::InvalidRepositoryStructure)` - Missing required items
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access repository contents
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    async fn validate_repository_structure(&self, repo: &MetadataRepository) -> MetadataResult<()> {
        let mut missing_items = Vec::new();

        // Check for required global directory
        if self
            .list_directory_contents(repo, "global")
            .await?
            .is_none()
        {
            missing_items.push("global/ directory".to_string());
        }

        // Check for required global/defaults.toml file
        if self
            .load_file_content(repo, "global/defaults.toml")
            .await?
            .is_none()
        {
            missing_items.push("global/defaults.toml file".to_string());
        }

        // Check for required teams directory (may be empty)
        if self.list_directory_contents(repo, "teams").await?.is_none() {
            missing_items.push("teams/ directory".to_string());
        }

        // Check for required types directory (may be empty)
        if self.list_directory_contents(repo, "types").await?.is_none() {
            missing_items.push("types/ directory".to_string());
        }

        // schemas/ directory is optional, so we don't check for it

        if !missing_items.is_empty() {
            return Err(crate::ConfigurationError::InvalidRepositoryStructure {
                repository: format!("{}/{}", repo.organization, repo.repository_name),
                missing_items,
            });
        }

        Ok(())
    }

    /// Load global default configuration from the metadata repository.
    ///
    /// This method loads and parses the `global/defaults.toml` file containing
    /// organization-wide baseline settings.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(GlobalDefaults)` - Successfully loaded and parsed configuration
    /// * `Err(ConfigurationError::FileNotFound)` - global/defaults.toml missing
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    async fn load_global_defaults(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<GlobalDefaults> {
        // Load global/defaults.toml file content
        let file_content = self.load_file_content(repo, "global/defaults.toml").await?;

        match file_content {
            Some(content) => {
                // Parse TOML content
                toml::from_str(&content).map_err(|e| crate::ConfigurationError::ParseError {
                    file_path: "global/defaults.toml".to_string(),
                    repository: format!("{}/{}", repo.organization, repo.repository_name),
                    error: format!("TOML parsing error: {}", e),
                })
            }
            None => Err(crate::ConfigurationError::FileNotFound {
                file_path: "global/defaults.toml".to_string(),
                repository: format!("{}/{}", repo.organization, repo.repository_name),
            }),
        }
    }

    /// Load team-specific configuration from the metadata repository.
    ///
    /// This method loads and parses team configuration from `teams/{team}/config.toml`.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `team` - The team name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(TeamConfig))` - Successfully loaded team configuration
    /// * `Ok(None)` - Team configuration file does not exist
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    async fn load_team_configuration(
        &self,
        repo: &MetadataRepository,
        team: &str,
    ) -> MetadataResult<Option<TeamConfig>> {
        let file_path = format!("teams/{}/config.toml", team);

        // Load team configuration file if it exists
        let file_content = self.load_file_content(repo, &file_path).await?;

        match file_content {
            Some(content) => {
                // Parse TOML content
                let team_config = toml::from_str(&content).map_err(|e| {
                    crate::ConfigurationError::ParseError {
                        file_path: file_path.clone(),
                        repository: format!("{}/{}", repo.organization, repo.repository_name),
                        error: format!("TOML parsing error: {}", e),
                    }
                })?;
                Ok(Some(team_config))
            }
            None => Ok(None), // Team configuration file does not exist
        }
    }

    /// Load repository type configuration from the metadata repository.
    ///
    /// This method loads and parses repository type configuration from
    /// `types/{repo_type}/config.toml`.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    /// * `repo_type` - The repository type name to load configuration for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(RepositoryTypeConfig))` - Successfully loaded configuration
    /// * `Ok(None)` - Repository type configuration file does not exist
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access configuration file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    async fn load_repository_type_configuration(
        &self,
        repo: &MetadataRepository,
        repo_type: &str,
    ) -> MetadataResult<Option<RepositoryTypeConfig>> {
        let file_path = format!("types/{}/config.toml", repo_type);

        // Load repository type configuration file if it exists
        let file_content = self.load_file_content(repo, &file_path).await?;

        match file_content {
            Some(content) => {
                // Parse TOML content
                let repo_type_config = toml::from_str(&content).map_err(|e| {
                    crate::ConfigurationError::ParseError {
                        file_path: file_path.clone(),
                        repository: format!("{}/{}", repo.organization, repo.repository_name),
                        error: format!("TOML parsing error: {}", e),
                    }
                })?;
                Ok(Some(repo_type_config))
            }
            None => Ok(None), // Repository type configuration file does not exist
        }
    }

    /// List all available repository types defined in the metadata repository.
    ///
    /// This method scans the `types/` directory to discover all available repository
    /// types that have configuration files.
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
    async fn list_available_repository_types(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<Vec<String>> {
        // List directories in types/ that contain config.toml files
        let type_dirs = self.list_directory_contents(repo, "types").await?;

        match type_dirs {
            Some(dirs) => {
                let mut available_types = Vec::new();

                // Check each directory for config.toml file
                for dir in dirs {
                    let config_path = format!("types/{}/config.toml", dir);
                    if self.load_file_content(repo, &config_path).await?.is_some() {
                        available_types.push(dir);
                    }
                }

                Ok(available_types)
            }
            None => {
                // types/ directory doesn't exist, return empty list
                Ok(Vec::new())
            }
        }
    }

    /// Load standard label definitions from the metadata repository.
    ///
    /// This method loads standard label configurations from `global/labels.toml`
    /// or similar location within the metadata repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - The metadata repository to load from
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, LabelConfig>)` - Map of label names to configurations
    /// * `Err(ConfigurationError::FileNotFound)` - Labels file not found
    /// * `Err(ConfigurationError::ParseError)` - Invalid TOML syntax or structure
    /// * `Err(ConfigurationError::AccessDenied)` - Cannot access labels file
    /// * `Err(ConfigurationError::NetworkError)` - API communication failure
    async fn load_standard_labels(
        &self,
        repo: &MetadataRepository,
    ) -> MetadataResult<HashMap<String, LabelConfig>> {
        // Load global/labels.toml if it exists, or return empty map
        let file_content = self.load_file_content(repo, "global/labels.toml").await?;

        match file_content {
            Some(content) => {
                // Parse TOML content into label configuration map
                toml::from_str(&content).map_err(|e| crate::ConfigurationError::ParseError {
                    file_path: "global/labels.toml".to_string(),
                    repository: format!("{}/{}", repo.organization, repo.repository_name),
                    error: format!("TOML parsing error: {}", e),
                })
            }
            None => {
                // Labels file doesn't exist, return empty map
                Ok(HashMap::new())
            }
        }
    }
}
