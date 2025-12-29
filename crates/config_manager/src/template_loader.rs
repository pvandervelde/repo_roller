//! Template configuration loading interface and implementation.
//!
//! GENERATED FROM: specs/interfaces/template-loading.md
//!
//! This module provides interfaces for loading template configurations from
//! template repositories. Templates are GitHub repositories containing a
//! `.reporoller/template.toml` file that defines repository creation settings.
//!
//! # Architecture
//!
//! ```text
//! OrganizationSettingsManager
//!     ↓ uses
//! TemplateLoader
//!     ↓ depends on (abstraction)
//! TemplateRepository trait
//!     ↑ implemented by
//! GitHubTemplateRepository (infrastructure)
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use config_manager::{TemplateLoader, GitHubTemplateRepository};
//! use github_client::GitHubClient;
//! use std::sync::Arc;
//!
//! # async fn example(github_client: Arc<GitHubClient>) -> Result<(), Box<dyn std::error::Error>> {
//! // Create loader with GitHub backend
//! let github_repo = Arc::new(GitHubTemplateRepository::new(github_client));
//! let loader = TemplateLoader::new(github_repo);
//!
//! // Load template (first time: GitHub API call)
//! let config = loader
//!     .load_template_configuration("myorg", "rust-service")
//!     .await?;
//!
//! println!("Template: {}", config.template.name);
//!
//! // Check cache statistics
//! let stats = loader.cache_statistics();
//! println!("Cache hit ratio: {:.1}%", stats.hit_ratio() * 100.0);
//! # Ok(())
//! # }
//! ```
//!
//! See: specs/interfaces/template-loading.md

use crate::{ConfigurationResult, TemplateConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

// Reference the tests module in the separate file
#[cfg(test)]
#[path = "template_loader_tests.rs"]
mod tests;

/// Interface for accessing template repositories.
///
/// This trait abstracts the storage mechanism for templates, allowing
/// implementations backed by GitHub, local filesystem, or other sources.
///
/// Implementations must be thread-safe (`Send + Sync`).
///
/// # Examples
///
/// ```no_run
/// use async_trait::async_trait;
/// use config_manager::{TemplateRepository, TemplateConfig, ConfigurationResult};
///
/// # struct MyTemplateRepo;
/// #[async_trait]
/// impl TemplateRepository for MyTemplateRepo {
///     async fn load_template_config(
///         &self,
///         org: &str,
///         template_name: &str,
///     ) -> ConfigurationResult<TemplateConfig> {
///         // Implementation...
///         unimplemented!()
///     }
///
///     async fn template_exists(
///         &self,
///         org: &str,
///         template_name: &str,
///     ) -> ConfigurationResult<bool> {
///         // Implementation...
///         unimplemented!()
///     }
/// }
/// ```
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    /// Load template configuration from a template repository.
    ///
    /// Reads and parses the `.reporoller/template.toml` file from the
    /// specified template repository in the given organization.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns the parsed `TemplateConfig` structure.
    ///
    /// # Errors
    ///
    /// * `ConfigurationError::TemplateNotFound` - Repository doesn't exist or not accessible
    /// * `ConfigurationError::TemplateConfigurationMissing` - No `.reporoller/template.toml` file
    /// * `ConfigurationError::ParseError` - Invalid TOML syntax
    /// * `ConfigurationError::InvalidConfiguration` - Missing required fields or invalid values
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_trait::async_trait;
    /// # use config_manager::{TemplateRepository, TemplateConfig};
    /// # async fn example(repo: &dyn TemplateRepository) {
    /// let config = repo
    ///     .load_template_config("my-org", "rust-service-template")
    ///     .await
    ///     .expect("Failed to load template config");
    ///
    /// println!("Template: {}", config.template.name);
    /// # }
    /// ```
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig>;

    /// Check if a template repository exists and is accessible.
    ///
    /// Verifies that the template repository exists and the current
    /// authentication context has read access.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns `true` if the repository exists and is accessible, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns errors only for API failures, not for missing repositories.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateRepository;
    /// # async fn example(repo: &dyn TemplateRepository) {
    /// if repo.template_exists("myorg", "rust-service").await.unwrap() {
    ///     println!("Template is accessible");
    /// } else {
    ///     println!("Template not found or not accessible");
    /// }
    /// # }
    /// ```
    async fn template_exists(&self, org: &str, template_name: &str) -> ConfigurationResult<bool>;
}

/// Template configuration loader with intelligent caching.
///
/// Loads template configurations from template repositories and caches
/// them to minimize GitHub API calls. Thread-safe for concurrent access.
///
/// # Cache Behavior
///
/// - Cache key: `(organization, template_name)`
/// - No automatic expiration (manual invalidation via `invalidate_cache()`)
/// - Cache persists for application lifetime
/// - Thread-safe concurrent access via `RwLock`
///
/// # Examples
///
/// ```no_run
/// use config_manager::{TemplateLoader, GitHubTemplateRepository};
/// use github_client::GitHubClient;
/// use std::sync::Arc;
///
/// # async fn example(github_client: Arc<GitHubClient>) -> Result<(), Box<dyn std::error::Error>> {
/// let github_repo = Arc::new(GitHubTemplateRepository::new(github_client));
/// let loader = TemplateLoader::new(github_repo);
///
/// let config = loader
///     .load_template_configuration("my-org", "rust-service")
///     .await?;
///
/// println!("Loaded template: {}", config.template.name);
///
/// // Check cache performance
/// let stats = loader.cache_statistics();
/// println!("Cache hit ratio: {:.1}%", stats.hit_ratio() * 100.0);
/// # Ok(())
/// # }
/// ```
///
/// See: specs/interfaces/template-loading.md
pub struct TemplateLoader {
    /// Template repository implementation
    repository: Arc<dyn TemplateRepository>,

    /// Configuration cache: (org, template_name) -> TemplateConfig
    cache: Arc<RwLock<HashMap<TemplateCacheKey, TemplateConfig>>>,

    /// Cache statistics for monitoring
    stats: Arc<RwLock<CacheStatistics>>,
}

impl std::fmt::Debug for TemplateLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateLoader")
            .field("repository", &"Arc<dyn TemplateRepository>")
            .field("cache", &self.cache)
            .field("stats", &self.stats)
            .finish()
    }
}

impl TemplateLoader {
    /// Create a new template loader.
    ///
    /// # Arguments
    ///
    /// * `repository` - Template repository implementation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use config_manager::{TemplateLoader, GitHubTemplateRepository};
    /// use github_client::GitHubClient;
    /// use std::sync::Arc;
    ///
    /// # fn example(github_client: Arc<GitHubClient>) {
    /// let github_repo = Arc::new(GitHubTemplateRepository::new(github_client));
    /// let loader = TemplateLoader::new(github_repo);
    /// # }
    /// ```
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self {
            repository,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
        }
    }

    /// Load template configuration with caching.
    ///
    /// Checks cache first, loads from repository on cache miss.
    /// Updates cache and statistics on successful load.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns the loaded or cached `TemplateConfig`.
    ///
    /// # Errors
    ///
    /// * `ConfigurationError::TemplateNotFound` - Repository doesn't exist
    /// * `ConfigurationError::TemplateConfigurationMissing` - No config file
    /// * `ConfigurationError::ParseError` - Invalid TOML
    /// * `ConfigurationError::InvalidConfiguration` - Malformed structure
    ///
    /// # Thread Safety
    ///
    /// Safe for concurrent calls. Cache access is synchronized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateLoader;
    /// # async fn example(loader: &TemplateLoader) {
    /// // First call: loads from repository and caches
    /// let config1 = loader
    ///     .load_template_configuration("myorg", "rust-service")
    ///     .await
    ///     .expect("Failed to load");
    ///
    /// // Second call: served from cache (no API call)
    /// let config2 = loader
    ///     .load_template_configuration("myorg", "rust-service")
    ///     .await
    ///     .expect("Failed to load");
    /// # }
    /// ```
    pub async fn load_template_configuration(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        let cache_key = TemplateCacheKey::new(org, template_name);

        // Check cache first (read lock)
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached_config) = cache.get(&cache_key) {
                debug!(
                    "Template configuration cache hit: {}/{}",
                    org, template_name
                );

                // Update statistics
                let mut stats = self.stats.write().unwrap();
                stats.total_requests += 1;
                stats.cache_hits += 1;

                return Ok(cached_config.clone());
            }
        }

        // Cache miss - load from repository
        debug!(
            "Template configuration cache miss: {}/{}",
            org, template_name
        );

        // Update miss statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_requests += 1;
            stats.cache_misses += 1;
        }

        // Load from repository (outside any locks)
        let config = self
            .repository
            .load_template_config(org, template_name)
            .await?;

        // Store in cache (write lock)
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(cache_key, config.clone());

            // Update entry count
            let mut stats = self.stats.write().unwrap();
            stats.cached_entries = cache.len();
        }

        info!("Template configuration cached: {}/{}", org, template_name);
        Ok(config)
    }

    /// Invalidate cached template configuration.
    ///
    /// Removes a specific template from cache, forcing next load
    /// to fetch fresh configuration from repository.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns `true` if entry was cached and removed, `false` if not cached.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateLoader;
    /// # async fn example(loader: &TemplateLoader) {
    /// // Load and cache
    /// let _ = loader.load_template_configuration("myorg", "rust-service").await;
    ///
    /// // Template updated in GitHub...
    ///
    /// // Invalidate to force refresh
    /// let was_cached = loader.invalidate_cache("myorg", "rust-service");
    /// println!("Was cached: {}", was_cached);
    ///
    /// // Next load will fetch fresh config
    /// let fresh = loader.load_template_configuration("myorg", "rust-service").await;
    /// # }
    /// ```
    pub fn invalidate_cache(&self, org: &str, template_name: &str) -> bool {
        let cache_key = TemplateCacheKey::new(org, template_name);

        let mut cache = self.cache.write().unwrap();
        let was_present = cache.remove(&cache_key).is_some();

        if was_present {
            let mut stats = self.stats.write().unwrap();
            stats.cached_entries = cache.len();
            debug!("Invalidated cache entry: {}/{}", org, template_name);
        }

        was_present
    }

    /// Clear all cached template configurations.
    ///
    /// Removes all entries from cache. Useful for testing or
    /// when forcing a complete refresh.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateLoader;
    /// # fn example(loader: &TemplateLoader) {
    /// loader.clear_cache();
    /// println!("All cache entries cleared");
    /// # }
    /// ```
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        let mut stats = self.stats.write().unwrap();
        stats.cached_entries = 0;

        info!("All template cache entries cleared");
    }

    /// Get cache statistics for monitoring.
    ///
    /// Returns snapshot of cache performance metrics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateLoader;
    /// # fn example(loader: &TemplateLoader) {
    /// let stats = loader.cache_statistics();
    /// println!("Total requests: {}", stats.total_requests);
    /// println!("Cache hits: {}", stats.cache_hits);
    /// println!("Cache misses: {}", stats.cache_misses);
    /// println!("Hit ratio: {:.1}%", stats.hit_ratio() * 100.0);
    /// println!("Cached entries: {}", stats.cached_entries);
    /// # }
    /// ```
    pub fn cache_statistics(&self) -> CacheStatistics {
        let stats = self.stats.read().unwrap();
        *stats
    }

    /// Check if a template exists and is accessible.
    ///
    /// Does not use cache; always checks with repository.
    ///
    /// # Arguments
    ///
    /// * `org` - GitHub organization or user name
    /// * `template_name` - Name of the template repository
    ///
    /// # Returns
    ///
    /// Returns `true` if template exists and is accessible.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use config_manager::TemplateLoader;
    /// # async fn example(loader: &TemplateLoader) {
    /// if loader.template_exists("myorg", "new-template").await.unwrap() {
    ///     let config = loader
    ///         .load_template_configuration("myorg", "new-template")
    ///         .await
    ///         .unwrap();
    ///     println!("Template found: {}", config.template.name);
    /// } else {
    ///     println!("Template not accessible");
    /// }
    /// # }
    /// ```
    pub async fn template_exists(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<bool> {
        // Delegate to repository (no caching for existence checks)
        self.repository.template_exists(org, template_name).await
    }
}

/// Cache key for template configurations.
///
/// Uniquely identifies a template by organization and name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TemplateCacheKey {
    organization: String,
    template_name: String,
}

impl TemplateCacheKey {
    /// Create a new cache key.
    fn new(org: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            organization: org.into(),
            template_name: template.into(),
        }
    }
}

/// Cache performance statistics.
///
/// Tracks cache effectiveness for monitoring and optimization.
///
/// # Examples
///
/// ```
/// use config_manager::CacheStatistics;
///
/// let stats = CacheStatistics {
///     total_requests: 100,
///     cache_hits: 75,
///     cache_misses: 25,
///     cached_entries: 10,
/// };
///
/// assert_eq!(stats.hit_ratio(), 0.75);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CacheStatistics {
    /// Total number of load requests
    pub total_requests: u64,

    /// Number of requests served from cache
    pub cache_hits: u64,

    /// Number of requests that required repository load
    pub cache_misses: u64,

    /// Current number of cached templates
    pub cached_entries: usize,
}

impl CacheStatistics {
    /// Calculate cache hit ratio (0.0 to 1.0).
    ///
    /// Returns 0.0 if no requests have been made.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::CacheStatistics;
    ///
    /// let stats = CacheStatistics {
    ///     total_requests: 100,
    ///     cache_hits: 80,
    ///     cache_misses: 20,
    ///     cached_entries: 5,
    /// };
    ///
    /// assert_eq!(stats.hit_ratio(), 0.8);
    ///
    /// // No requests yet
    /// let empty_stats = CacheStatistics::default();
    /// assert_eq!(empty_stats.hit_ratio(), 0.0);
    /// ```
    pub fn hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }
}
