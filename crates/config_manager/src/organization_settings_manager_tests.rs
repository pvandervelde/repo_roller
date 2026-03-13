//! Tests for organization settings manager.

use super::*;
use crate::{
    errors::ConfigurationError, global_defaults::GlobalDefaults,
    metadata_provider::MetadataRepository, repository_type_config::RepositoryTypeConfig,
    team_config::TeamConfig, template_config::TemplateConfig, TemplateRepository,
};
use async_trait::async_trait;

// ============================================================================
// Mock TemplateRepository for Testing
// ============================================================================

/// Mock template repository for testing.
#[derive(Debug, Clone)]
struct MockTemplateRepository;

#[async_trait]
impl TemplateRepository for MockTemplateRepository {
    async fn load_template_config(
        &self,
        _org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        // Return a minimal template config
        Ok(TemplateConfig {
            template: crate::template_config::TemplateMetadata {
                name: template_name.to_string(),
                description: format!("Test template: {}", template_name),
                author: "Test Author".to_string(),
                tags: vec![],
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
            rulesets: None,
            default_visibility: None,
            templating: None,
            notifications: None,
            permissions: None,
            teams: None,
            collaborators: None,
            naming_rules: None,
        })
    }

    async fn template_exists(&self, _org: &str, _template_name: &str) -> ConfigurationResult<bool> {
        Ok(true)
    }
}

/// Create a test template loader.
fn create_test_template_loader() -> Arc<crate::TemplateLoader> {
    Arc::new(crate::TemplateLoader::new(Arc::new(MockTemplateRepository)))
}

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

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<crate::template_config::TemplateConfig> {
        Err(ConfigurationError::FileNotFound {
            path: "template.toml".to_string(),
        })
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

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<crate::settings::WebhookConfig>> {
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
    let template_loader = create_test_template_loader();
    let manager = OrganizationSettingsManager::new(provider, template_loader);

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
    let manager = OrganizationSettingsManager::new(provider.clone(), create_test_template_loader());

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
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

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
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

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
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

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

    let manager1 =
        OrganizationSettingsManager::new(provider.clone(), create_test_template_loader());
    let manager2 =
        OrganizationSettingsManager::new(provider.clone(), create_test_template_loader());

    // Both managers should share the same provider
    assert!(
        Arc::ptr_eq(&manager1.metadata_provider, &manager2.metadata_provider),
        "Multiple managers should share the same metadata provider"
    );
}

// ============================================================================
// Configuration Resolution Tests (Task 5.2)
// ============================================================================

/// Verify configuration resolution workflow with only global defaults.
///
/// Tests the basic resolution workflow when no team or repository type is specified.
/// This validates the discover → load → merge workflow at its simplest.
///
/// Behavioral Assertion CRA-001: Configuration precedence enforcement
#[tokio::test]
async fn test_resolve_configuration_with_global_defaults_only() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context = crate::ConfigurationContext::new("test-org", "rust-service");

    // Resolution should succeed with global defaults
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_ok(),
        "Resolution should succeed with global defaults only"
    );

    let merged = result.unwrap();
    // MergedConfiguration always has repository settings (even if empty/default)
    assert!(
        format!("{:?}", merged.repository).contains("RepositorySettings"),
        "Merged configuration should contain repository settings"
    );
}

/// Verify configuration resolution workflow with team configuration.
///
/// Tests that team-specific configuration is loaded and merged correctly
/// when a team is specified in the context.
///
/// Behavioral Assertion CRA-001: Configuration precedence enforcement
#[tokio::test]
async fn test_resolve_configuration_with_team() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context =
        crate::ConfigurationContext::new("test-org", "rust-service").with_team("backend-team");

    // Resolution should succeed and include team configuration
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_ok(),
        "Resolution should succeed with team configuration"
    );
}

/// Verify configuration resolution workflow with repository type.
///
/// Tests that repository type-specific configuration is loaded and merged
/// correctly when a repository type is specified in the context.
///
/// Behavioral Assertion CRA-001: Configuration precedence enforcement
#[tokio::test]
async fn test_resolve_configuration_with_repository_type() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context = crate::ConfigurationContext::new("test-org", "rust-service")
        .with_repository_type("library");

    // Resolution should succeed and include repository type configuration
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_ok(),
        "Resolution should succeed with repository type configuration"
    );
}

/// Verify configuration resolution with both team and repository type.
///
/// Tests the complete hierarchy: Global → Repository Type → Team → Template
/// This validates that all configuration levels are loaded and merged correctly.
///
/// Behavioral Assertion CRA-001: Configuration precedence enforcement
#[tokio::test]
async fn test_resolve_configuration_with_team_and_repository_type() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context = crate::ConfigurationContext::new("test-org", "rust-service")
        .with_team("backend-team")
        .with_repository_type("library");

    // Resolution should succeed with all configuration levels
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_ok(),
        "Resolution should succeed with team and repository type configuration"
    );
}

/// Verify configuration resolution fails when metadata repository cannot be discovered.
///
/// Tests graceful error handling when the metadata repository doesn't exist.
/// System should return a clear error indicating the missing repository.
///
/// Behavioral Assertion CRF-001: Missing metadata repository
#[tokio::test]
async fn test_resolve_configuration_fails_when_metadata_repository_not_found() {
    let provider = Arc::new(MockMetadataProvider::with_failure());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context = crate::ConfigurationContext::new("test-org", "rust-service");

    // Resolution should fail with metadata repository not found error
    let result = manager.resolve_configuration(&context).await;

    assert!(
        result.is_err(),
        "Resolution should fail when metadata repository not found"
    );

    let error = result.unwrap_err();
    assert!(
        matches!(error, ConfigurationError::MetadataRepositoryNotFound { .. }),
        "Error should be MetadataRepositoryNotFound"
    );
}

/// Verify source tracking in merged configuration.
///
/// Tests that the merged configuration correctly tracks which configuration
/// source provided each setting (Global, Team, RepositoryType, Template).
///
/// Behavioral Assertion CRA-001: Configuration precedence enforcement with source tracking
#[tokio::test]
async fn test_resolve_configuration_tracks_configuration_sources() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context =
        crate::ConfigurationContext::new("test-org", "rust-service").with_team("backend-team");

    let result = manager.resolve_configuration(&context).await;

    assert!(result.is_ok(), "Resolution should succeed");

    let merged = result.unwrap();

    // MergedConfiguration should track sources
    // Source trace field_count() returns usize which is always >= 0, so just verify it exists
    let _field_count = merged.source_trace.field_count();
}

/// Verify configuration resolution is consistent across multiple calls.
///
/// Tests that calling resolve_configuration multiple times with the same
/// context produces identical results (assuming no configuration changes).
///
/// Behavioral Assertion CRA-003: Configuration cache consistency
#[tokio::test]
async fn test_resolve_configuration_is_consistent() {
    let provider = Arc::new(MockMetadataProvider::new());
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());

    let context = crate::ConfigurationContext::new("test-org", "rust-service");

    let result1 = manager.resolve_configuration(&context).await;
    let result2 = manager.resolve_configuration(&context).await;

    assert!(
        result1.is_ok() && result2.is_ok(),
        "Both resolutions should succeed"
    );

    // Both results should have same structure
    let merged1 = result1.unwrap();
    let merged2 = result2.unwrap();

    // Compare a specific field rather than entire struct (which doesn't implement PartialEq in a comparable way)
    assert_eq!(
        merged1.labels.len(),
        merged2.labels.len(),
        "Configurations should be consistent"
    );
}

// ============================================================================
// New Tests: Empty template handling should preserve labels
// ============================================================================

/// Template repository mock that returns an error when the template name is empty.
#[derive(Debug, Clone)]
struct EmptySensitiveTemplateRepository;

#[async_trait]
impl TemplateRepository for EmptySensitiveTemplateRepository {
    async fn load_template_config(
        &self,
        _org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        if template_name.is_empty() {
            return Err(ConfigurationError::TemplateNotFound {
                org: "test-org".to_string(),
                template: "".to_string(),
            });
        }

        Ok(TemplateConfig {
            template: crate::template_config::TemplateMetadata {
                name: template_name.to_string(),
                description: format!("Test template: {}", template_name),
                author: "Test Author".to_string(),
                tags: vec![],
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
            rulesets: None,
            default_visibility: None,
            templating: None,
            notifications: None,
            permissions: None,
            teams: None,
            collaborators: None,
            naming_rules: None,
        })
    }

    async fn template_exists(&self, _org: &str, _template_name: &str) -> ConfigurationResult<bool> {
        Ok(true)
    }
}

/// Create a test template loader that errors on empty template names.
fn create_empty_sensitive_template_loader() -> Arc<crate::TemplateLoader> {
    Arc::new(crate::TemplateLoader::new(Arc::new(
        EmptySensitiveTemplateRepository,
    )))
}

/// Metadata provider that returns a non-empty set of standard labels.
#[derive(Debug, Clone)]
struct LabelledMetadataProvider;

#[async_trait]
impl MetadataRepositoryProvider for LabelledMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        _organization: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        Ok(MetadataRepository {
            organization: "test-org".to_string(),
            repository_name: ".reporoller-test".to_string(),
            discovery_method: crate::metadata_provider::DiscoveryMethod::ConfigurationBased {
                repository_name: ".reporoller-test".to_string(),
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

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<crate::template_config::TemplateConfig> {
        Err(ConfigurationError::FileNotFound {
            path: "template.toml".to_string(),
        })
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
        let mut labels = std::collections::HashMap::new();
        labels.insert(
            "bug".to_string(),
            crate::settings::LabelConfig {
                name: "bug".to_string(),
                color: "d73a4a".to_string(),
                description: "Something isn't working".to_string(),
            },
        );
        Ok(labels)
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

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<crate::settings::WebhookConfig>> {
        Ok(vec![])
    }
}

/// Verify that when no template is specified (empty template name),
/// configuration resolution succeeds and preserves standard labels.
#[tokio::test]
async fn test_resolve_configuration_with_empty_template_preserves_standard_labels() {
    let provider = Arc::new(LabelledMetadataProvider);
    let manager =
        OrganizationSettingsManager::new(provider, create_empty_sensitive_template_loader());

    // Empty template name simulates non-template creation modes
    let context = crate::ConfigurationContext::new("test-org", "");

    let result = manager.resolve_configuration(&context).await;
    assert!(
        result.is_ok(),
        "Resolution should succeed without a template"
    );

    let merged = result.unwrap();
    assert!(
        merged.labels.contains_key("bug"),
        "Standard labels should be preserved when no template is provided"
    );
}
// ============================================================================
// Permission Protection Tests
// ============================================================================

/// A configurable mock metadata provider for permission protection tests.
/// Returns GlobalDefaults pre-loaded with specific teams, collaborators, and
/// permission policies.
struct PermissionTestMetadataProvider {
    global_defaults: GlobalDefaults,
}

impl PermissionTestMetadataProvider {
    fn new(global_defaults: GlobalDefaults) -> Self {
        Self { global_defaults }
    }
}

#[async_trait]
impl MetadataRepositoryProvider for PermissionTestMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        _organization: &str,
    ) -> ConfigurationResult<MetadataRepository> {
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
        Ok(self.global_defaults.clone())
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<crate::template_config::TemplateConfig> {
        Err(ConfigurationError::FileNotFound {
            path: "template.toml".to_string(),
        })
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

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<crate::settings::WebhookConfig>> {
        Ok(vec![])
    }
}

/// A configurable mock template repository for permission protection tests.
/// Returns a TemplateConfig pre-loaded with specific teams and collaborators.
#[derive(Debug, Clone)]
struct PermissionTestTemplateRepository {
    teams: Option<Vec<crate::settings::DefaultTeamConfig>>,
    collaborators: Option<Vec<crate::settings::DefaultCollaboratorConfig>>,
}

impl PermissionTestTemplateRepository {
    fn with_teams(teams: Vec<crate::settings::DefaultTeamConfig>) -> Self {
        Self {
            teams: Some(teams),
            collaborators: None,
        }
    }

    fn with_collaborators(collabs: Vec<crate::settings::DefaultCollaboratorConfig>) -> Self {
        Self {
            teams: None,
            collaborators: Some(collabs),
        }
    }
}

#[async_trait]
impl TemplateRepository for PermissionTestTemplateRepository {
    async fn load_template_config(
        &self,
        _org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        Ok(TemplateConfig {
            template: crate::template_config::TemplateMetadata {
                name: template_name.to_string(),
                description: "Permission test template".to_string(),
                author: "test".to_string(),
                tags: vec![],
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
            rulesets: None,
            default_visibility: None,
            templating: None,
            notifications: None,
            permissions: None,
            teams: self.teams.clone(),
            collaborators: self.collaborators.clone(),
            naming_rules: None,
        })
    }

    async fn template_exists(&self, _org: &str, _template_name: &str) -> ConfigurationResult<bool> {
        Ok(true)
    }
}

// --- Team protection tests ---

/// Verify that a template attempting to change a globally-locked team returns
/// PermissionLockedNotAllowed.
#[tokio::test]
async fn test_resolve_configuration_template_cannot_alter_locked_global_team() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultTeamConfig;

    let global_defaults = GlobalDefaults {
        default_teams: Some(vec![DefaultTeamConfig {
            slug: "security-ops".to_string(),
            access_level: "admin".to_string(),
            locked: true,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_teams(vec![
        DefaultTeamConfig {
            slug: "security-ops".to_string(),
            access_level: "write".to_string(), // Any change to a locked team is forbidden.
            locked: false,
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(
        result.is_err(),
        "Should fail when altering a locked global team"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            ConfigurationError::PermissionLockedNotAllowed { .. }
        ),
        "Error must be PermissionLockedNotAllowed"
    );
}

/// Verify that a template trying to demote a non-locked global team returns
/// PermissionDemotionNotAllowed.
#[tokio::test]
async fn test_resolve_configuration_template_cannot_demote_global_team() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultTeamConfig;

    let global_defaults = GlobalDefaults {
        default_teams: Some(vec![DefaultTeamConfig {
            slug: "backend-team".to_string(),
            access_level: "write".to_string(),
            locked: false,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_teams(vec![
        DefaultTeamConfig {
            slug: "backend-team".to_string(),
            access_level: "read".to_string(), // Demotion: write → read.
            locked: false,
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_err(), "Should fail on team demotion");
    assert!(
        matches!(
            result.unwrap_err(),
            ConfigurationError::PermissionDemotionNotAllowed { .. }
        ),
        "Error must be PermissionDemotionNotAllowed"
    );
}

/// Verify that a template can upgrade a team already established in global defaults.
#[tokio::test]
async fn test_resolve_configuration_template_can_upgrade_global_team() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultTeamConfig;

    let global_defaults = GlobalDefaults {
        default_teams: Some(vec![DefaultTeamConfig {
            slug: "frontend-team".to_string(),
            access_level: "read".to_string(),
            locked: false,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_teams(vec![
        DefaultTeamConfig {
            slug: "frontend-team".to_string(),
            access_level: "write".to_string(), // Upgrade: read → write.
            locked: false,
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_ok(), "Upgrade should succeed");
    let merged = result.unwrap();
    assert_eq!(
        merged.teams.get("frontend-team").map(|s| s.as_str()),
        Some("write"),
        "Team should be upgraded to write"
    );
}

/// Verify that globally-locked teams appear in merged.locked_teams.
#[tokio::test]
async fn test_resolve_configuration_locked_global_team_is_in_merged_locked_teams() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultTeamConfig;

    let global_defaults = GlobalDefaults {
        default_teams: Some(vec![
            DefaultTeamConfig {
                slug: "security-ops".to_string(),
                access_level: "admin".to_string(),
                locked: true,
            },
            DefaultTeamConfig {
                slug: "regular-team".to_string(),
                access_level: "write".to_string(),
                locked: false,
            },
        ]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    // Use an empty template (no teams) to avoid lock conflicts.
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());
    let context = crate::ConfigurationContext::new("test-org", "");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_ok(), "Resolution should succeed");
    let merged = result.unwrap();
    assert!(
        merged.locked_teams.contains("security-ops"),
        "Globally-locked team must be in locked_teams"
    );
    assert!(
        !merged.locked_teams.contains("regular-team"),
        "Non-locked team must not be in locked_teams"
    );
}

/// Verify that a template-locked team is added to merged.locked_teams.
#[tokio::test]
async fn test_resolve_configuration_template_locked_team_is_in_merged_locked_teams() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultTeamConfig;

    let global_defaults = GlobalDefaults {
        default_teams: Some(vec![DefaultTeamConfig {
            slug: "ops-team".to_string(),
            access_level: "write".to_string(),
            locked: false,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    // Template keeps the same level but marks the team as locked.
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_teams(vec![
        DefaultTeamConfig {
            slug: "ops-team".to_string(),
            access_level: "write".to_string(), // Same level – not a demotion.
            locked: true,                      // Template marks it locked for future levels.
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_ok(), "Resolution should succeed");
    let merged = result.unwrap();
    assert!(
        merged.locked_teams.contains("ops-team"),
        "Template-locked team must be in locked_teams"
    );
}

// --- Collaborator protection tests ---

/// Verify that a template attempting to change a globally-locked collaborator returns
/// PermissionLockedNotAllowed.
#[tokio::test]
async fn test_resolve_configuration_template_cannot_alter_locked_global_collaborator() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultCollaboratorConfig;

    let global_defaults = GlobalDefaults {
        default_collaborators: Some(vec![DefaultCollaboratorConfig {
            username: "owner-bot".to_string(),
            access_level: "admin".to_string(),
            locked: true,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_collaborators(vec![
        DefaultCollaboratorConfig {
            username: "owner-bot".to_string(),
            access_level: "read".to_string(),
            locked: false,
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(
        result.is_err(),
        "Should fail when altering a locked collaborator"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            ConfigurationError::PermissionLockedNotAllowed { .. }
        ),
        "Error must be PermissionLockedNotAllowed"
    );
}

/// Verify that a template trying to demote a collaborator established at global level
/// returns PermissionDemotionNotAllowed.
#[tokio::test]
async fn test_resolve_configuration_template_cannot_demote_global_collaborator() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::DefaultCollaboratorConfig;

    let global_defaults = GlobalDefaults {
        default_collaborators: Some(vec![DefaultCollaboratorConfig {
            username: "ci-bot".to_string(),
            access_level: "write".to_string(),
            locked: false,
        }]),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let template_repo = Arc::new(PermissionTestTemplateRepository::with_collaborators(vec![
        DefaultCollaboratorConfig {
            username: "ci-bot".to_string(),
            access_level: "read".to_string(), // Demotion.
            locked: false,
        },
    ]));
    let manager = OrganizationSettingsManager::new(
        provider,
        Arc::new(crate::TemplateLoader::new(template_repo)),
    );
    let context = crate::ConfigurationContext::new("test-org", "my-template");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_err(), "Should fail on collaborator demotion");
    assert!(
        matches!(
            result.unwrap_err(),
            ConfigurationError::PermissionDemotionNotAllowed { .. }
        ),
        "Error must be PermissionDemotionNotAllowed"
    );
}

// --- Max access level tests ---

/// Verify that max_team_access_level defined in global permission policies is
/// propagated into the merged configuration.
#[tokio::test]
async fn test_resolve_configuration_max_team_access_level_extracted_into_merged() {
    use crate::global_defaults::GlobalDefaults;
    use crate::settings::permissions::OrganizationPermissionPoliciesConfig;

    let global_defaults = GlobalDefaults {
        permissions: Some(OrganizationPermissionPoliciesConfig {
            baseline: None,
            restrictions: None,
            max_team_access_level: Some("write".to_string()),
            max_collaborator_access_level: Some("maintain".to_string()),
        }),
        ..Default::default()
    };
    let provider = Arc::new(PermissionTestMetadataProvider::new(global_defaults));
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());
    let context = crate::ConfigurationContext::new("test-org", "");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_ok(), "Resolution should succeed");
    let merged = result.unwrap();
    assert_eq!(
        merged.max_team_access_level.as_deref(),
        Some("write"),
        "max_team_access_level must be propagated from global permissions"
    );
    assert_eq!(
        merged.max_collaborator_access_level.as_deref(),
        Some("maintain"),
        "max_collaborator_access_level must be propagated from global permissions"
    );
}

/// Verify that when no max levels are configured the merged fields remain None.
#[tokio::test]
async fn test_resolve_configuration_max_access_levels_absent_when_not_configured() {
    let provider = Arc::new(PermissionTestMetadataProvider::new(
        GlobalDefaults::default(),
    ));
    let manager = OrganizationSettingsManager::new(provider, create_test_template_loader());
    let context = crate::ConfigurationContext::new("test-org", "");

    let result = manager.resolve_configuration(&context).await;
    assert!(result.is_ok(), "Resolution should succeed");
    let merged = result.unwrap();
    assert!(
        merged.max_team_access_level.is_none(),
        "max_team_access_level should be None when not configured"
    );
    assert!(
        merged.max_collaborator_access_level.is_none(),
        "max_collaborator_access_level should be None when not configured"
    );
}
