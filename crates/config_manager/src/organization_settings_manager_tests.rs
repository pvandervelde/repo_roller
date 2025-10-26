//! Tests for organization settings manager.

use super::*;
use crate::{
    errors::ConfigurationError, global_defaults::GlobalDefaults,
    metadata_provider::MetadataRepository, repository_type_config::RepositoryTypeConfig,
    team_config::TeamConfig,
};
use async_trait::async_trait;

// ============================================================================
// Mock MetadataRepositoryProvider for Testing
// ============================================================================

/// Mock metadata provider for testing.
#[derive(Debug, Clone)]
struct MockMetadataProvider {
    should_fail: bool,
}

impl MockMetadataProvider {
    fn new() -> Self {
        Self { should_fail: false }
    }

    #[allow(dead_code)]
    fn with_failure() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl MetadataRepositoryProvider for MockMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        _organization: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        if self.should_fail {
            return Err(ConfigurationError::MetadataRepositoryNotFound {
                org: "test-org".to_string(),
            });
        }

        Ok(MetadataRepository {
            organization: "test-org".to_string(),
            repository_name: "repo-config".to_string(),
            discovery_method: crate::metadata_provider::DiscoveryMethod::ConfigurationBased {
                repository_name: "repo-config".to_string(),
            },
            last_updated: chrono::Utc::now(),
        })
    }

    async fn load_global_defaults(
        &self,
        _repository: &MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults> {
        Ok(GlobalDefaults::default())
    }

    async fn load_team_configuration(
        &self,
        _repository: &MetadataRepository,
        _team_name: &str,
    ) -> ConfigurationResult<Option<TeamConfig>> {
        Ok(None)
    }

    async fn load_repository_type_configuration(
        &self,
        _repository: &MetadataRepository,
        _repository_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
        Ok(None)
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<std::collections::HashMap<String, crate::settings::LabelConfig>> {
        Ok(std::collections::HashMap::new())
    }

    async fn validate_repository_structure(
        &self,
        _repository: &MetadataRepository,
    ) -> ConfigurationResult<()> {
        Ok(())
    }

    async fn list_available_repository_types(
        &self,
        _repository: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }
}

// ============================================================================
// Constructor Tests (Task 5.1)
// ============================================================================

/// Verify OrganizationSettingsManager can be created with valid metadata provider.
#[test]
fn test_manager_creation_with_valid_provider() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider);

    // Manager should be created successfully
    assert!(
        format!("{:?}", manager).contains("OrganizationSettingsManager"),
        "Manager should be created with valid provider"
    );
}

/// Verify manager stores the metadata provider.
#[test]
fn test_manager_stores_metadata_provider() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider.clone());

    // Manager should store the provider reference
    assert!(
        Arc::ptr_eq(
            &manager.metadata_provider,
            &(provider as Arc<dyn MetadataRepositoryProvider>)
        ),
        "Manager should store the metadata provider"
    );
}

/// Verify manager creates internal configuration merger.
#[test]
fn test_manager_creates_internal_merger() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider);

    // Merger should be created
    assert!(
        format!("{:?}", manager.merger).contains("ConfigurationMerger"),
        "Manager should create internal ConfigurationMerger"
    );
}

/// Verify manager can be cloned.
#[test]
fn test_manager_is_cloneable() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider);

    // Clone should work
    let cloned = manager.clone();

    assert!(
        format!("{:?}", cloned).contains("OrganizationSettingsManager"),
        "Manager should be cloneable"
    );
}

/// Verify manager implements Debug trait.
#[test]
fn test_manager_implements_debug() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider);

    let debug_str = format!("{:?}", manager);

    assert!(
        debug_str.contains("OrganizationSettingsManager"),
        "Manager should implement Debug trait"
    );
    assert!(
        debug_str.contains("metadata_provider"),
        "Debug output should include metadata_provider"
    );
    assert!(
        debug_str.contains("merger"),
        "Debug output should include merger"
    );
}

/// Verify multiple managers can share the same metadata provider.
#[test]
fn test_multiple_managers_can_share_provider() {
    let provider = Arc::new(MockMetadataProvider::new());

    let manager1 = OrganizationSettingsManager::new(provider.clone());
    let manager2 = OrganizationSettingsManager::new(provider.clone());

    // Both managers should share the same provider
    assert!(
        Arc::ptr_eq(&manager1.metadata_provider, &manager2.metadata_provider),
        "Multiple managers should share the same metadata provider"
    );
}
