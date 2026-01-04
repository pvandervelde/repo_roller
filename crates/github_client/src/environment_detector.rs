//! GitHub environment detection implementation.
//!
//! This module provides concrete implementations of the GitHubEnvironmentDetector
//! trait that query the GitHub API to determine plan limitations and enterprise
//! status.

use super::environment::{GitHubEnvironmentDetector, PlanLimitations};
use async_trait::async_trait;
use octocrab::Octocrab;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::Error;

/// Cache entry for plan limitations with TTL.
#[derive(Debug, Clone)]
struct CachedLimitations {
    /// The cached plan limitations.
    limitations: PlanLimitations,
    /// When this cache entry was created.
    cached_at: Instant,
}

impl CachedLimitations {
    /// Create a new cache entry.
    fn new(limitations: PlanLimitations) -> Self {
        Self {
            limitations,
            cached_at: Instant::now(),
        }
    }

    /// Check if this cache entry is still valid.
    fn is_valid(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() < ttl
    }
}

/// GitHub API-based environment detector with caching.
///
/// This implementation queries the GitHub API to determine plan limitations
/// and caches results for 1 hour to minimize API calls.
///
/// # Examples
///
/// ```no_run
/// use github_client::environment_detector::GitHubApiEnvironmentDetector;
/// use github_client::environment::GitHubEnvironmentDetector;
/// use octocrab::Octocrab;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let github = Arc::new(Octocrab::builder().build()?);
/// let detector = GitHubApiEnvironmentDetector::new(github);
///
/// let limitations = detector.get_plan_limitations("my-org").await?;
/// println!("Supports private repos: {}", limitations.supports_private_repos);
/// # Ok(())
/// # }
/// ```
pub struct GitHubApiEnvironmentDetector {
    /// GitHub API client.
    client: Arc<Octocrab>,
    /// Cache of plan limitations by organization.
    cache: Arc<RwLock<HashMap<String, CachedLimitations>>>,
    /// Cache TTL (1 hour).
    cache_ttl: Duration,
}

impl GitHubApiEnvironmentDetector {
    /// Create a new GitHub API environment detector.
    ///
    /// # Arguments
    ///
    /// * `client` - GitHub API client
    ///
    /// # Returns
    ///
    /// A new environment detector with 1-hour caching enabled.
    pub fn new(client: Arc<Octocrab>) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Clear the cache for testing purposes.
    #[cfg(test)]
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Query the GitHub API for organization plan information.
    async fn query_organization_plan(&self, organization: &str) -> Result<PlanLimitations, Error> {
        // Query the GitHub API for organization details
        let org = self
            .client
            .orgs(organization)
            .get()
            .await
            .map_err(|_| Error::ApiError())?;

        // Determine plan limitations based on organization properties
        let is_enterprise = org
            .plan
            .as_ref()
            .map(|plan| plan.name.to_lowercase().contains("enterprise"))
            .unwrap_or(false);

        let supports_internal = is_enterprise;

        // All GitHub plans support private repositories (even free)
        // Free plans have collaborator limits (3 per repo) but can create private repos
        let supports_private = true;

        // No hard limit on number of private repos for any plan
        // (Free plans have collaborator limits per repo, not repo count limits)
        let private_repo_limit = None;

        Ok(PlanLimitations {
            supports_private_repos: supports_private,
            supports_internal_repos: supports_internal,
            private_repo_limit,
            is_enterprise,
        })
    }
}

#[async_trait]
impl GitHubEnvironmentDetector for GitHubApiEnvironmentDetector {
    async fn get_plan_limitations(&self, organization: &str) -> Result<PlanLimitations, Error> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(organization) {
                if cached.is_valid(self.cache_ttl) {
                    return Ok(cached.limitations.clone());
                }
            }
        }

        // Cache miss or expired, query API
        let limitations = self.query_organization_plan(organization).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                organization.to_string(),
                CachedLimitations::new(limitations.clone()),
            );
        }

        Ok(limitations)
    }

    async fn is_enterprise(&self, organization: &str) -> Result<bool, Error> {
        let limitations = self.get_plan_limitations(organization).await?;
        Ok(limitations.is_enterprise)
    }
}

#[cfg(test)]
#[path = "environment_detector_tests.rs"]
mod tests;
