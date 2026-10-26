//! Tests for template-load-time validation of `TemplateConfig.designation`
//! against `repository_type` (docs/spec/interfaces/fork-mirror-operations.md §1):
//!
//! > a template MUST NOT declare both `repository_type` policy `Fixed`-for-a-type
//! > AND a `designation` — rejected at template-load validation time with
//! > `ConfigurationError::InvalidConfiguration { field: "designation", .. }`
//! > only if the template also sets `repository_type`; declaring just
//! > `designation` alone is the common case and must not require
//! > `repository_type`.

use super::*;
use crate::errors::ConfigurationError;
use crate::template_config::{
    RepositoryTypePolicy, RepositoryTypeSpec, TemplateConfig, TemplateDesignation, TemplateMetadata,
};
use async_trait::async_trait;
use std::sync::Arc;

/// A `TemplateRepository` test double that returns a single, caller-supplied
/// `TemplateConfig` regardless of which org/template is requested — isolates
/// the loader's validation behaviour from real fetch/parse concerns.
struct FixedTemplateRepository {
    config: TemplateConfig,
}

#[async_trait]
impl TemplateRepository for FixedTemplateRepository {
    async fn load_template_config(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        Ok(self.config.clone())
    }

    async fn template_exists(&self, _org: &str, _template_name: &str) -> ConfigurationResult<bool> {
        Ok(true)
    }
}

fn metadata() -> TemplateMetadata {
    TemplateMetadata {
        name: "t".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        tags: vec![],
    }
}

fn config_with(
    designation: Option<TemplateDesignation>,
    repository_type: Option<RepositoryTypeSpec>,
) -> TemplateConfig {
    TemplateConfig {
        template: metadata(),
        repository_type,
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
        designation,
    }
}

fn loader_for(config: TemplateConfig) -> TemplateLoader {
    TemplateLoader::new(Arc::new(FixedTemplateRepository { config }))
}

// ============================================================================
// Tier 1: designation alone is valid
// ============================================================================

#[tokio::test]
async fn test_load_template_configuration_accepts_designation_without_repository_type() {
    let loader = loader_for(config_with(Some(TemplateDesignation::Fork), None));

    let result = loader
        .load_template_configuration("acme", "fork-settings")
        .await;

    assert!(
        result.is_ok(),
        "designation alone (no repository_type) must not require repository_type: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_load_template_configuration_accepts_no_designation_and_no_repository_type() {
    let loader = loader_for(config_with(None, None));

    let result = loader.load_template_configuration("acme", "plain").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_template_configuration_accepts_designation_with_preferable_repository_type() {
    let loader = loader_for(config_with(
        Some(TemplateDesignation::SourceHistoryMirror),
        Some(RepositoryTypeSpec {
            repository_type: "service".to_string(),
            policy: RepositoryTypePolicy::Preferable,
        }),
    ));

    let result = loader
        .load_template_configuration("acme", "mirror-settings")
        .await;

    assert!(
        result.is_ok(),
        "designation + Preferable repository_type policy is not the conflicting case: {:?}",
        result.err()
    );
}

// ============================================================================
// Tier 2: Adversarial — designation + Fixed repository_type is rejected
// ============================================================================

#[tokio::test]
async fn test_load_template_configuration_rejects_designation_with_fixed_repository_type() {
    let loader = loader_for(config_with(
        Some(TemplateDesignation::Fork),
        Some(RepositoryTypeSpec {
            repository_type: "service".to_string(),
            policy: RepositoryTypePolicy::Fixed,
        }),
    ));

    let result = loader
        .load_template_configuration("acme", "conflicting-template")
        .await;

    assert!(
        result.is_err(),
        "designation + Fixed-policy repository_type must be rejected at load time"
    );

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, .. } => {
            assert_eq!(field, "designation");
        }
        other => panic!("expected InvalidConfiguration{{field: \"designation\"}}, got {other:?}"),
    }
}

/// Stub-killing: an implementation that only checks for *any* `repository_type`
/// presence (rather than specifically the `Fixed` policy) would also reject
/// this case; this test alone can't disprove that, but combined with
/// `test_load_template_configuration_accepts_designation_with_preferable_repository_type`
/// above, an implementation must distinguish `Fixed` from `Preferable`.
#[tokio::test]
async fn test_load_template_configuration_rejects_each_designation_variant_with_fixed_type() {
    for designation in [
        TemplateDesignation::Fork,
        TemplateDesignation::SourceHistoryMirror,
        TemplateDesignation::SourceOnlyMirror,
    ] {
        let loader = loader_for(config_with(
            Some(designation),
            Some(RepositoryTypeSpec {
                repository_type: "library".to_string(),
                policy: RepositoryTypePolicy::Fixed,
            }),
        ));

        let result = loader.load_template_configuration("acme", "t").await;

        assert!(
            result.is_err(),
            "designation {designation:?} + Fixed repository_type must be rejected"
        );
    }
}
