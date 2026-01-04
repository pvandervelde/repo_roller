//! Visibility policy provider implementation.
//!
//! Provides organization visibility policies by loading from the metadata
//! repository configuration with thread-safe caching.
//!
//! See specs/interfaces/repository-visibility.md for interface specification.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::{
    visibility::{VisibilityError, VisibilityPolicy, VisibilityPolicyProvider},
    GlobalDefaults, MetadataRepositoryProvider,
};

#[cfg(test)]
#[path = "visibility_policy_provider_tests.rs"]
mod tests;

/// Configuration-based visibility policy provider.
///
/// Loads organization visibility policies from the metadata repository's
/// global/defaults.toml file with thread-safe caching.
///
/// # Caching
///
/// Policies are cached for 5 minutes to reduce API calls. Cache is
/// thread-safe using Arc<RwLock<HashMap>>.
///
/// # Example
///
/// ```rust,no_run
/// use config_manager::ConfigBasedPolicyProvider;
/// use std::sync::Arc;
///
/// let provider = ConfigBasedPolicyProvider::new(Arc::new(metadata_provider));
/// let policy = provider.get_policy("my-org").await?;
/// ```
pub struct ConfigBasedPolicyProvider {
    /// Metadata repository provider for loading configuration
    metadata_provider: Arc<dyn MetadataRepositoryProvider>,

    /// Policy cache: (org_name -> (policy, loaded_at))
    cache: Arc<RwLock<HashMap<String, (VisibilityPolicy, Instant)>>>,

    /// Cache time-to-live
    cache_ttl: Duration,
}

impl ConfigBasedPolicyProvider {
    /// Create a new configuration-based policy provider.
    ///
    /// # Arguments
    ///
    /// * `metadata_provider` - Provider for loading configuration files
    pub fn new(metadata_provider: Arc<dyn MetadataRepositoryProvider>) -> Self {
        Self {
            metadata_provider,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5 * 60), // 5 minutes
        }
    }

    /// Get the visibility policy for an organization.
    ///
    /// Implements caching with 5-minute TTL. On cache miss, loads global defaults
    /// from the metadata repository and parses the visibility policy.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name
    ///
    /// # Returns
    ///
    /// Organization's visibility policy (Unrestricted if not configured)
    ///
    /// # Errors
    ///
    /// Returns VisibilityError if policy cannot be loaded
    async fn get_policy(&self, organization: &str) -> Result<VisibilityPolicy, VisibilityError> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some((policy, loaded_at)) = cache.get(organization) {
                if loaded_at.elapsed() < self.cache_ttl {
                    return Ok(policy.clone());
                }
            }
        }

        // Cache miss or expired - load from metadata repository
        let metadata_repo = self
            .metadata_provider
            .discover_metadata_repository(organization)
            .await
            .map_err(|e| VisibilityError::ConfigurationError {
                message: format!("Failed to discover metadata repository: {}", e),
            })?;

        // Load global defaults
        let defaults = self
            .metadata_provider
            .load_global_defaults(&metadata_repo)
            .await
            .map_err(|e| VisibilityError::ConfigurationError {
                message: format!("Failed to load global defaults: {}", e),
            })?;

        // Parse policy from defaults
        let policy = self.parse_policy(&defaults);

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(organization.to_string(), (policy.clone(), Instant::now()));
        }

        Ok(policy)
    }

    /// Invalidate cached policy for an organization.
    ///
    /// Forces the next `get_policy` call to fetch fresh policy data.
    ///
    /// # Arguments
    ///
    /// * `organization` - Organization name
    async fn invalidate_cache(&self, organization: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(organization);
    }
}

#[async_trait]
impl VisibilityPolicyProvider for ConfigBasedPolicyProvider {
    async fn get_policy(&self, organization: &str) -> Result<VisibilityPolicy, VisibilityError> {
        self.get_policy(organization).await
    }

    async fn invalidate_cache(&self, organization: &str) {
        self.invalidate_cache(organization).await
    }
}

impl ConfigBasedPolicyProvider {
    /// Parse visibility policy from global defaults configuration.
    ///
    /// Currently returns Unrestricted as the safe default since GlobalDefaults
    /// doesn't yet have a repository_visibility field. This will be enhanced
    /// in future work to parse the [repository_visibility] section.
    ///
    /// # Arguments
    ///
    /// * `_defaults` - Global defaults configuration
    ///
    /// # Returns
    ///
    /// Parsed visibility policy (currently always Unrestricted)
    ///
    /// # Future Implementation
    ///
    /// Will parse the following TOML structure:
    /// ```toml
    /// [repository_visibility]
    /// enforcement_level = "restricted"  # or "required" or "unrestricted"
    /// required_visibility = "private"   # only when enforcement_level = "required"
    /// restricted_visibilities = ["public"]  # only when enforcement_level = "restricted"
    /// ```
    fn parse_policy(&self, _defaults: &GlobalDefaults) -> VisibilityPolicy {
        // TODO: Parse [repository_visibility] section from defaults when field is added to GlobalDefaults
        // For now, return Unrestricted as safe default
        VisibilityPolicy::Unrestricted
    }
}
