//! GitHub-based implementation of metadata repository provider.
//!
//! This module provides the concrete implementation of `MetadataRepositoryProvider`
//! that uses GitHub APIs to discover and access organization configuration repositories.

use crate::{
    ConfigurationError, ConfigurationResult, DiscoveryMethod, GlobalDefaults, LabelConfig,
    MetadataRepository, MetadataRepositoryProvider, RepositoryTypeConfig, TeamConfig,
};
use async_trait::async_trait;
use chrono::Utc;
use github_client::GitHubClient;
use std::collections::HashMap;

// Reference the tests module in the separate file
#[cfg(test)]
#[path = "github_metadata_provider_tests.rs"]
mod tests;

/// Configuration for metadata repository discovery.
///
/// **This is an input/configuration type** that specifies how the provider
/// should attempt to find an organization's metadata repository.
///
/// This is distinct from `DiscoveryMethod` which is an output type that
/// records how a repository was actually found.
///
/// # Discovery Strategies
///
/// - **Explicit naming**: Directly access a repository with a known name
/// - **Topic search**: Find repositories tagged with a specific GitHub topic
///
/// # Examples
///
/// ```
/// use config_manager::MetadataProviderConfig;
///
/// // Configuration-based discovery
/// let config = MetadataProviderConfig::explicit("org-metadata");
///
/// // Topic-based discovery
/// let config = MetadataProviderConfig::by_topic("reporoller-metadata");
/// ```
#[derive(Debug, Clone)]
pub struct MetadataProviderConfig {
    /// Discovery method configuration
    discovery: DiscoveryConfig,
}

/// Internal configuration enum for discovery strategy.
///
/// This is kept private and used internally by `GitHubMetadataProvider`.
/// External consumers use `MetadataProviderConfig::explicit()` or
/// `MetadataProviderConfig::by_topic()` to create configurations.
#[derive(Debug, Clone)]
enum DiscoveryConfig {
    /// Explicit repository name
    RepositoryName(String),
    /// Search by GitHub topic
    Topic(String),
}

impl MetadataProviderConfig {
    /// Create configuration for explicit repository name discovery.
    ///
    /// # Arguments
    ///
    /// * `repository_name` - The exact name of the metadata repository
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::MetadataProviderConfig;
    ///
    /// let config = MetadataProviderConfig::explicit("org-metadata");
    /// ```
    pub fn explicit(repository_name: impl Into<String>) -> Self {
        Self {
            discovery: DiscoveryConfig::RepositoryName(repository_name.into()),
        }
    }

    /// Create configuration for topic-based discovery.
    ///
    /// # Arguments
    ///
    /// * `topic` - The GitHub topic to search for
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::MetadataProviderConfig;
    ///
    /// let config = MetadataProviderConfig::by_topic("reporoller-metadata");
    /// ```
    pub fn by_topic(topic: impl Into<String>) -> Self {
        Self {
            discovery: DiscoveryConfig::Topic(topic.into()),
        }
    }
}

/// GitHub-based metadata repository provider.
///
/// This implementation uses the GitHub API to discover and load configuration
/// from organization metadata repositories.
///
/// # Discovery Methods
///
/// - **Configuration-based**: Directly access a named repository
/// - **Topic-based**: Search for repositories with a specific GitHub topic
///
/// # Examples
///
/// ```no_run
/// use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
/// use github_client::GitHubClient;
///
/// # async fn example(github_client: GitHubClient) {
/// let config = MetadataProviderConfig::explicit("org-metadata");
/// let provider = GitHubMetadataProvider::new(github_client, config);
///
/// let metadata_repo = provider
///     .discover_metadata_repository("my-org")
///     .await
///     .expect("Failed to discover repository");
/// # }
/// ```
pub struct GitHubMetadataProvider {
    /// GitHub API client
    client: GitHubClient,
    /// Discovery configuration
    config: MetadataProviderConfig,
}

impl GitHubMetadataProvider {
    /// Create a new GitHub metadata provider.
    ///
    /// # Arguments
    ///
    /// * `client` - Authenticated GitHub client
    /// * `config` - Discovery method configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
    /// use github_client::GitHubClient;
    ///
    /// # async fn example(github_client: GitHubClient) {
    /// let config = MetadataProviderConfig::explicit("org-metadata");
    /// let provider = GitHubMetadataProvider::new(github_client, config);
    /// # }
    /// ```
    pub fn new(client: GitHubClient, config: MetadataProviderConfig) -> Self {
        Self { client, config }
    }

    /// Discover repository using configuration-based method.
    ///
    /// Attempts to access the repository with the explicitly configured name.
    async fn discover_by_name(
        &self,
        org: &str,
        repository_name: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        // Attempt to get the repository
        self.client
            .get_repository(org, repository_name)
            .await
            .map_err(|_| ConfigurationError::MetadataRepositoryNotFound {
                org: org.to_string(),
            })?;

        // Repository exists, create metadata
        Ok(MetadataRepository {
            organization: org.to_string(),
            repository_name: repository_name.to_string(),
            discovery_method: DiscoveryMethod::ConfigurationBased {
                repository_name: repository_name.to_string(),
            },
            last_updated: Utc::now(),
        })
    }

    /// Discover repository using topic-based method.
    ///
    /// Searches for repositories in the organization with the specified topic.
    async fn discover_by_topic(
        &self,
        org: &str,
        _topic: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        // TODO: Implement topic-based search
        // For now, return an error indicating the feature is not yet implemented
        // This will be implemented once we add search capabilities to GitHubClient

        // Placeholder: In real implementation, we would:
        // 1. Search repositories in org with the specified topic
        // 2. Validate exactly one match (error if 0 or multiple)
        // 3. Return MetadataRepository with TopicBased discovery method

        Err(ConfigurationError::MetadataRepositoryNotFound {
            org: org.to_string(),
        })
    }
}

#[async_trait]
impl MetadataRepositoryProvider for GitHubMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        match &self.config.discovery {
            DiscoveryConfig::RepositoryName(repo_name) => {
                self.discover_by_name(org, repo_name).await
            }
            DiscoveryConfig::Topic(topic) => self.discover_by_topic(org, topic).await,
        }
    }

    async fn load_global_defaults(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults> {
        // TODO: Implement in Task 3.4
        todo!("Task 3.4: Implement TOML configuration file parsing")
    }

    async fn load_team_configuration(
        &self,
        _repo: &MetadataRepository,
        _team: &str,
    ) -> ConfigurationResult<Option<TeamConfig>> {
        // TODO: Implement in Task 3.4
        todo!("Task 3.4: Implement TOML configuration file parsing")
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &MetadataRepository,
        _repo_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
        // TODO: Implement in Task 3.4
        todo!("Task 3.4: Implement TOML configuration file parsing")
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<HashMap<String, LabelConfig>> {
        // TODO: Implement in Task 3.4
        todo!("Task 3.4: Implement TOML configuration file parsing")
    }

    async fn list_available_repository_types(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        // TODO: Implement in Task 3.4
        todo!("Task 3.4: Implement TOML configuration file parsing")
    }

    async fn validate_repository_structure(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<()> {
        // TODO: Implement in Task 3.3
        todo!("Task 3.3: Implement repository structure validation")
    }
}
