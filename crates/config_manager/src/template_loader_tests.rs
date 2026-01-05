//! Tests for template configuration loading.
//!
//! Tests the TemplateLoader and TemplateRepository trait implementations.

use super::*;
use crate::ConfigurationError;
use std::sync::{Arc, Mutex};

/// Mock template repository for testing.
///
/// Tracks call counts to verify caching behavior.
struct MockTemplateRepository {
    templates: HashMap<(String, String), TemplateConfig>,
    call_count: Arc<Mutex<usize>>,
}

impl MockTemplateRepository {
    fn new() -> Self {
        Self {
            templates: HashMap::new(),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn with_template(mut self, org: &str, name: &str, config: TemplateConfig) -> Self {
        self.templates
            .insert((org.to_string(), name.to_string()), config);
        self
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl TemplateRepository for MockTemplateRepository {
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        drop(count);

        self.templates
            .get(&(org.to_string(), template_name.to_string()))
            .cloned()
            .ok_or_else(|| ConfigurationError::TemplateNotFound {
                org: org.to_string(),
                template: template_name.to_string(),
            })
    }

    async fn template_exists(&self, org: &str, template_name: &str) -> ConfigurationResult<bool> {
        Ok(self
            .templates
            .contains_key(&(org.to_string(), template_name.to_string())))
    }
}

/// Create a test template configuration.
fn create_test_template_config(name: &str) -> TemplateConfig {
    use crate::template_config::TemplateMetadata;

    TemplateConfig {
        template: TemplateMetadata {
            name: name.to_string(),
            description: format!("Test template: {}", name),
            author: "Test Author".to_string(),
            tags: vec!["test".to_string()],
        },
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        default_visibility: None,
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        // Create mock with one template
        let template_config = create_test_template_config("rust-service");
        let mock = Arc::new(MockTemplateRepository::new().with_template(
            "myorg",
            "rust-service",
            template_config.clone(),
        ));

        let loader = TemplateLoader::new(mock.clone());

        // First load - should call repository
        let result1 = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert!(result1.is_ok());
        assert_eq!(mock.call_count(), 1);

        // Second load - should use cache (no additional repository call)
        let result2 = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert!(result2.is_ok());
        assert_eq!(mock.call_count(), 1); // Still 1!

        // Verify statistics
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cached_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        // Create mock with one template
        let template_config = create_test_template_config("rust-service");
        let mock = Arc::new(MockTemplateRepository::new().with_template(
            "myorg",
            "rust-service",
            template_config.clone(),
        ));

        let loader = TemplateLoader::new(mock.clone());

        // First load - cache miss
        let result = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert!(result.is_ok());
        assert_eq!(mock.call_count(), 1);

        // Verify statistics
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cached_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        // Create mock with one template
        let template_config = create_test_template_config("rust-service");
        let mock = Arc::new(MockTemplateRepository::new().with_template(
            "myorg",
            "rust-service",
            template_config.clone(),
        ));

        let loader = TemplateLoader::new(mock.clone());

        // Load and cache
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert_eq!(mock.call_count(), 1);

        // Invalidate cache
        let was_cached = loader.invalidate_cache("myorg", "rust-service");
        assert!(was_cached);

        // Load again - should call repository again
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert_eq!(mock.call_count(), 2);

        // Verify statistics
        let stats = loader.cache_statistics();
        assert_eq!(stats.cache_misses, 2); // Both loads were misses
    }

    #[tokio::test]
    async fn test_clear_cache() {
        // Create mock with two templates
        let config1 = create_test_template_config("rust-service");
        let config2 = create_test_template_config("go-service");
        let mock = Arc::new(
            MockTemplateRepository::new()
                .with_template("myorg", "rust-service", config1)
                .with_template("myorg", "go-service", config2),
        );

        let loader = TemplateLoader::new(mock.clone());

        // Load both templates
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        let _ = loader
            .load_template_configuration("myorg", "go-service")
            .await;
        assert_eq!(mock.call_count(), 2);

        // Verify cache has 2 entries
        let stats = loader.cache_statistics();
        assert_eq!(stats.cached_entries, 2);

        // Clear cache
        loader.clear_cache();

        // Verify cache is empty
        let stats = loader.cache_statistics();
        assert_eq!(stats.cached_entries, 0);

        // Load again - should call repository
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        assert_eq!(mock.call_count(), 3);
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        // Create mock with one template
        let template_config = create_test_template_config("rust-service");
        let mock = Arc::new(MockTemplateRepository::new().with_template(
            "myorg",
            "rust-service",
            template_config.clone(),
        ));

        let loader = TemplateLoader::new(mock.clone());

        // Initial statistics
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);

        // First load (miss)
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.0);

        // Second load (hit)
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.hit_ratio(), 0.5);

        // Third load (hit)
        let _ = loader
            .load_template_configuration("myorg", "rust-service")
            .await;
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 2);
        assert!((stats.hit_ratio() - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use tokio::task;

        // Create mock with one template
        let template_config = create_test_template_config("rust-service");
        let mock = Arc::new(MockTemplateRepository::new().with_template(
            "myorg",
            "rust-service",
            template_config.clone(),
        ));

        let loader = Arc::new(TemplateLoader::new(mock.clone()));

        // Spawn multiple concurrent loads
        let mut handles = vec![];
        for _ in 0..10 {
            let loader_clone = loader.clone();
            let handle = task::spawn(async move {
                loader_clone
                    .load_template_configuration("myorg", "rust-service")
                    .await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Verify repository was called at least once
        // (May be called more than once due to race conditions before cache is populated)
        assert!(mock.call_count() >= 1);
        assert!(mock.call_count() <= 10);

        // Verify statistics are consistent
        let stats = loader.cache_statistics();
        assert_eq!(stats.total_requests, 10);
        assert_eq!(stats.cache_hits + stats.cache_misses, 10);
    }

    #[tokio::test]
    async fn test_error_propagation() {
        // Create mock with no templates
        let mock = Arc::new(MockTemplateRepository::new());
        let loader = TemplateLoader::new(mock.clone());

        // Try to load non-existent template
        let result = loader
            .load_template_configuration("myorg", "nonexistent")
            .await;

        // Verify error is propagated
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigurationError::TemplateNotFound { .. }));

        // Verify error details
        if let ConfigurationError::TemplateNotFound { org, template } = err {
            assert_eq!(org, "myorg");
            assert_eq!(template, "nonexistent");
        }
    }
}

#[cfg(test)]
mod statistics_tests {
    use super::*;

    #[test]
    fn test_hit_ratio_calculation() {
        let stats = CacheStatistics {
            total_requests: 100,
            cache_hits: 75,
            cache_misses: 25,
            cached_entries: 10,
        };

        assert_eq!(stats.hit_ratio(), 0.75);
    }

    #[test]
    fn test_hit_ratio_zero_requests() {
        let stats = CacheStatistics::default();
        assert_eq!(stats.hit_ratio(), 0.0);
    }
}
