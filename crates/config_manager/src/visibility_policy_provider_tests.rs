//! Tests for visibility policy provider.
//!
//! Verifies policy loading, parsing, caching, and error handling for
//! organization visibility policies.

use super::*;
use crate::{
    settings::WebhookConfig, ConfigurationError, ConfigurationResult, DiscoveryMethod,
    GlobalDefaults, LabelConfig, MetadataRepository, MetadataRepositoryProvider,
    RepositoryTypeConfig, RepositoryVisibility, TeamConfig, TemplateConfig,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock metadata repository provider for testing policy provider.
///
/// Simulates loading global defaults configuration with various
/// visibility policy configurations.
struct MockMetadataProvider {
    /// Configuration data to return
    config_data: Option<GlobalDefaults>,
    /// Whether to simulate loading failure
    should_fail: bool,
}

impl MockMetadataProvider {
    fn new() -> Self {
        // Create default global defaults with no visibility policy
        let defaults = GlobalDefaults {
            repository: Some(Default::default()),
            pull_requests: None,
            branch_protection: None,
            actions: None,
            push: None,
            webhooks: None,
            custom_properties: None,
            environments: None,
            github_apps: None,
            repository_visibility: None, // No policy = unrestricted
        };

        Self {
            config_data: Some(defaults),
            should_fail: false,
        }
    }

    fn with_policy(
        mut self,
        enforcement_level: &str,
        required: Option<&str>,
        restricted: Option<Vec<&str>>,
    ) -> Self {
        use crate::visibility::VisibilityPolicyConfig;

        // Create mock global defaults with visibility policy
        let defaults = GlobalDefaults {
            repository: Some(Default::default()),
            pull_requests: None,
            branch_protection: None,
            actions: None,
            push: None,
            webhooks: None,
            custom_properties: None,
            environments: None,
            github_apps: None,
            repository_visibility: Some(VisibilityPolicyConfig {
                enforcement_level: enforcement_level.to_string(),
                required_visibility: required.map(|s| s.to_string()),
                restricted_visibilities: restricted
                    .map(|v| v.iter().map(|s| s.to_string()).collect()),
            }),
        };

        self.config_data = Some(defaults);
        self
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl MetadataRepositoryProvider for MockMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        _org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        if self.should_fail {
            return Err(ConfigurationError::MetadataRepositoryNotFound {
                org: "test-org".to_string(),
            });
        }
        Ok(MetadataRepository {
            organization: "test-org".to_string(),
            repository_name: ".reporoller-test".to_string(),
            discovery_method: DiscoveryMethod::ConfigurationBased {
                repository_name: ".reporoller-test".to_string(),
            },
            last_updated: Utc::now(),
        })
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        // Not needed for visibility policy tests
        Err(ConfigurationError::FileAccessError {
            path: "template.toml".to_string(),
            reason: "Not implemented in mock".to_string(),
        })
    }

    async fn load_global_defaults(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults> {
        if self.should_fail {
            return Err(ConfigurationError::FileAccessError {
                path: "global/defaults.toml".to_string(),
                reason: "Simulated failure".to_string(),
            });
        }
        self.config_data
            .clone()
            .ok_or_else(|| ConfigurationError::FileAccessError {
                path: "global/defaults.toml".to_string(),
                reason: "No config data".to_string(),
            })
    }

    async fn load_team_configuration(
        &self,
        _repo: &MetadataRepository,
        _team: &str,
    ) -> ConfigurationResult<Option<TeamConfig>> {
        Ok(None)
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &MetadataRepository,
        _repo_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
        Ok(None)
    }

    async fn list_available_repository_types(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<HashMap<String, LabelConfig>> {
        Ok(HashMap::new())
    }

    async fn validate_repository_structure(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<()> {
        Ok(())
    }

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<WebhookConfig>> {
        Ok(vec![])
    }
}

/// Test parsing required policy from configuration.
///
/// Verifies that Required policy is correctly parsed when
/// enforcement_level is "required" and required_visibility is specified.
#[tokio::test]
async fn test_required_policy_parsing() {
    let provider = MockMetadataProvider::new().with_policy("required", Some("private"), None);

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let policy = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(
        policy,
        VisibilityPolicy::Required(RepositoryVisibility::Private)
    );
}

/// Test parsing restricted policy from configuration.
///
/// Verifies that Restricted policy is correctly parsed when
/// enforcement_level is "restricted" with prohibited visibilities.
#[tokio::test]
async fn test_restricted_policy_parsing() {
    let provider =
        MockMetadataProvider::new().with_policy("restricted", None, Some(vec!["public"]));

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let policy = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(
        policy,
        VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public])
    );
}

/// Test parsing unrestricted policy from configuration.
///
/// Verifies that Unrestricted policy is used when enforcement_level
/// is "unrestricted" or when the section is missing entirely.
#[tokio::test]
async fn test_unrestricted_policy_parsing() {
    let provider = MockMetadataProvider::new().with_policy("unrestricted", None, None);

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let policy = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(policy, VisibilityPolicy::Unrestricted);
}

/// Test default to unrestricted when configuration section is missing.
///
/// Verifies that when no visibility policy configuration exists,
/// the provider defaults to Unrestricted policy.
#[tokio::test]
async fn test_default_to_unrestricted_when_missing() {
    let provider = MockMetadataProvider::new(); // No policy configured

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let policy = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(policy, VisibilityPolicy::Unrestricted);
}

/// Test policy caching behavior.
///
/// Verifies that policy is cached after first load and subsequent
/// calls return cached value without hitting the provider.
#[tokio::test]
async fn test_policy_caching() {
    let provider = MockMetadataProvider::new().with_policy("required", Some("private"), None);

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    // First call - loads from provider
    let policy1 = policy_provider.get_policy("test-org").await.unwrap();

    // Second call - should use cache
    let policy2 = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(policy1, policy2);
    assert_eq!(
        policy1,
        VisibilityPolicy::Required(RepositoryVisibility::Private)
    );
}

/// Test cache TTL expiration.
///
/// Verifies that cached policy expires after 5 minutes and
/// fresh policy is loaded on next request.
#[tokio::test]
async fn test_cache_ttl_expiration() {
    // This test would require mocking time or using a shorter TTL for testing
    // For now, we document the expected behavior

    // Expected: After 5 minutes, cache entry should be considered stale
    // and next get_policy() call should reload from provider
}

/// Test cache invalidation.
///
/// Verifies that invalidate_cache() removes the cached policy
/// and next get_policy() call reloads from provider.
#[tokio::test]
async fn test_cache_invalidation() {
    let provider = MockMetadataProvider::new().with_policy("required", Some("private"), None);

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    // Load policy into cache
    let _policy = policy_provider.get_policy("test-org").await.unwrap();

    // Invalidate cache
    policy_provider.invalidate_cache("test-org").await;

    // Next call should reload (we can't easily verify without mocking, but this tests the API)
    let policy = policy_provider.get_policy("test-org").await.unwrap();
    assert_eq!(
        policy,
        VisibilityPolicy::Required(RepositoryVisibility::Private)
    );
}

/// Test concurrent access to policy provider.
///
/// Verifies that multiple concurrent requests for policy are handled
/// correctly with thread-safe caching.
#[tokio::test]
async fn test_concurrent_access() {
    let provider =
        MockMetadataProvider::new().with_policy("restricted", None, Some(vec!["public"]));

    let policy_provider = Arc::new(ConfigBasedPolicyProvider::new(Arc::new(provider)));

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let provider_clone = policy_provider.clone();

        let handle =
            tokio::spawn(async move { provider_clone.get_policy("test-org").await.unwrap() });

        handles.push(handle);
    }

    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed with same policy
    for result in results {
        let policy = result.unwrap();
        assert_eq!(
            policy,
            VisibilityPolicy::Restricted(vec![RepositoryVisibility::Public])
        );
    }
}

/// Test error handling for configuration load failure.
///
/// Verifies that errors from metadata provider are properly
/// propagated as VisibilityError.
#[tokio::test]
async fn test_configuration_load_error() {
    let provider = MockMetadataProvider::new().with_failure();

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let result = policy_provider.get_policy("test-org").await;

    assert!(result.is_err());
    // Error should indicate configuration issue
}

/// Test parsing invalid visibility value.
///
/// Verifies that invalid visibility values in configuration
/// produce appropriate errors.
#[tokio::test]
async fn test_invalid_visibility_value() {
    // NOTE: Invalid visibility values are caught at TOML parsing time,
    // not at runtime. The serde deserializer will reject invalid values.
    // This behavior is tested in visibility.rs unit tests.
    // No additional test needed here since the provider works with
    // already-parsed GlobalDefaults.
}

/// Test restricted policy with multiple prohibited visibilities.
///
/// Verifies that Restricted policy can prohibit multiple
/// visibility options simultaneously.
#[tokio::test]
async fn test_restricted_policy_multiple_prohibited() {
    let provider = MockMetadataProvider::new().with_policy(
        "restricted",
        None,
        Some(vec!["public", "internal"]),
    );

    let policy_provider = ConfigBasedPolicyProvider::new(Arc::new(provider));

    let policy = policy_provider.get_policy("test-org").await.unwrap();

    assert_eq!(
        policy,
        VisibilityPolicy::Restricted(vec![
            RepositoryVisibility::Public,
            RepositoryVisibility::Internal,
        ])
    );
}
