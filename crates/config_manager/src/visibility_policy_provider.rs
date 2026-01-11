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
    visibility::{
        RepositoryVisibility, VisibilityError, VisibilityPolicy, VisibilityPolicyProvider,
    },
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
/// ```rust,ignore
/// use config_manager::{ConfigBasedPolicyProvider, VisibilityPolicyProvider};
/// use std::sync::Arc;
///
/// # async fn example() -> anyhow::Result<()> {
/// # let metadata_provider = todo!();
/// let provider = ConfigBasedPolicyProvider::new(Arc::new(metadata_provider));
/// let policy = provider.get_policy("my-org").await?;
/// # Ok(())
/// # }
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
    /// Parse visibility policy from global defaults.
    ///
    /// Parsed visibility policy from the repository_visibility configuration
    ///
    /// # TOML Structure
    ///
    /// ```toml
    /// [repository_visibility]
    /// enforcement_level = "restricted"  # or "required" or "unrestricted"
    /// required_visibility = "private"   # only when enforcement_level = "required"
    /// restricted_visibilities = ["public"]  # only when enforcement_level = "restricted"
    /// ```
    fn parse_policy(&self, defaults: &GlobalDefaults) -> VisibilityPolicy {
        let Some(config) = &defaults.repository_visibility else {
            // No configuration specified - default to unrestricted
            return VisibilityPolicy::Unrestricted;
        };

        match config.enforcement_level.to_lowercase().as_str() {
            "required" => {
                // Parse required visibility
                if let Some(ref visibility_str) = config.required_visibility {
                    match visibility_str.to_lowercase().as_str() {
                        "private" => VisibilityPolicy::Required(RepositoryVisibility::Private),
                        "public" => VisibilityPolicy::Required(RepositoryVisibility::Public),
                        "internal" => VisibilityPolicy::Required(RepositoryVisibility::Internal),
                        _ => {
                            // Invalid visibility - default to unrestricted
                            VisibilityPolicy::Unrestricted
                        }
                    }
                } else {
                    // Required level but no visibility specified - default to unrestricted
                    VisibilityPolicy::Unrestricted
                }
            }
            "restricted" => {
                // Parse restricted visibilities
                if let Some(ref prohibited) = config.restricted_visibilities {
                    let parsed: Vec<RepositoryVisibility> = prohibited
                        .iter()
                        .filter_map(|s| match s.to_lowercase().as_str() {
                            "private" => Some(RepositoryVisibility::Private),
                            "public" => Some(RepositoryVisibility::Public),
                            "internal" => Some(RepositoryVisibility::Internal),
                            _ => None, // Skip invalid values
                        })
                        .collect();

                    if parsed.is_empty() {
                        // No valid prohibited visibilities - default to unrestricted
                        VisibilityPolicy::Unrestricted
                    } else {
                        VisibilityPolicy::Restricted(parsed)
                    }
                } else {
                    // Restricted level but no visibilities specified - default to unrestricted
                    VisibilityPolicy::Unrestricted
                }
            }
            "unrestricted" => {
                VisibilityPolicy::Unrestricted
            }
            _ => {
                // Unrecognized level - default to unrestricted
                VisibilityPolicy::Unrestricted
            }
        }
    }
}
