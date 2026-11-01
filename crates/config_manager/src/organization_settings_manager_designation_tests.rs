//! Tests for `OrganizationSettingsManager::find_designated_template`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §1 ("Discovery") and
//! RCA-004/005/006 + RCF-006.
//!
//! # Assumed signature (spec gap — see Tester report)
//!
//! `docs/spec/interfaces/fork-mirror-operations.md` shows
//! `find_designated_template(&self, designation: TemplateDesignation)` with
//! no organization parameter, but every sibling discovery method on this
//! component (`list_templates(&self, org: &str)`) requires one, and
//! `OrganizationSettingsManager` holds no organization state of its own.
//! These tests assume the realistic signature
//! `find_designated_template(&self, org: &str, designation: TemplateDesignation)
//! -> ConfigurationResult<Option<TemplateName>>` and should be reconciled with
//! the architect/spec before GREEN.
//!
//! These tests also assume a `config_manager::TemplateName` branded type is
//! introduced (mirroring the existing `RepositoryTypeName`, which already
//! lives in `config_manager` for exactly this reason: `config_manager` is a
//! dependency of `repo_roller_core` and cannot depend back on
//! `repo_roller_core::TemplateName`).

use super::*;
use crate::errors::ConfigurationError;
use crate::global_defaults::GlobalDefaults;
use crate::repository_type_config::RepositoryTypeConfig;
use crate::team_config::TeamConfig;
use crate::template_config::{TemplateConfig, TemplateDesignation, TemplateMetadata};
use crate::TemplateRepository;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Test doubles
// ============================================================================

/// Lists a configurable, fixed set of template names for any organization.
#[derive(Clone)]
struct FixedTemplateListProvider {
    names: Vec<String>,
}

#[async_trait]
impl MetadataRepositoryProvider for FixedTemplateListProvider {
    async fn discover_metadata_repository(
        &self,
        _organization: &str,
    ) -> ConfigurationResult<crate::metadata_provider::MetadataRepository> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn load_global_defaults(
        &self,
        _repository: &crate::metadata_provider::MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(self.names.clone())
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        unimplemented!("designation discovery loads templates via the TemplateLoader, not directly")
    }

    async fn load_team_configuration(
        &self,
        _repository: &crate::metadata_provider::MetadataRepository,
        _team_name: &str,
    ) -> ConfigurationResult<Option<TeamConfig>> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn load_repository_type_configuration(
        &self,
        _repository: &crate::metadata_provider::MetadataRepository,
        _repository_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn load_standard_labels(
        &self,
        _repo: &crate::metadata_provider::MetadataRepository,
    ) -> ConfigurationResult<HashMap<String, crate::settings::LabelConfig>> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn validate_repository_structure(
        &self,
        _repository: &crate::metadata_provider::MetadataRepository,
    ) -> ConfigurationResult<()> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn list_available_repository_types(
        &self,
        _repository: &crate::metadata_provider::MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        unimplemented!("not exercised by designation discovery tests")
    }

    async fn load_global_webhooks(
        &self,
        _repo: &crate::metadata_provider::MetadataRepository,
    ) -> ConfigurationResult<Vec<crate::settings::WebhookConfig>> {
        unimplemented!("not exercised by designation discovery tests")
    }
}

/// Returns a `TemplateConfig` with a per-name designation, for the
/// `TemplateLoader` backing `find_designated_template`.
struct DesignationTemplateRepository {
    designations: HashMap<String, Option<TemplateDesignation>>,
}

#[async_trait]
impl TemplateRepository for DesignationTemplateRepository {
    async fn load_template_config(
        &self,
        _org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        let designation = self
            .designations
            .get(template_name)
            .copied()
            .unwrap_or(None);

        Ok(TemplateConfig {
            template: TemplateMetadata {
                name: template_name.to_string(),
                description: "test".to_string(),
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
            teams: None,
            collaborators: None,
            naming_rules: None,
            designation,
            creation_capabilities: None,
        })
    }

    async fn template_exists(&self, _org: &str, _template_name: &str) -> ConfigurationResult<bool> {
        Ok(true)
    }
}

fn manager_with(
    names: Vec<&str>,
    designations: HashMap<&str, Option<TemplateDesignation>>,
) -> OrganizationSettingsManager {
    let metadata_provider = Arc::new(FixedTemplateListProvider {
        names: names.into_iter().map(str::to_string).collect(),
    });
    let designations = designations
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    let template_loader = Arc::new(crate::TemplateLoader::new(Arc::new(
        DesignationTemplateRepository { designations },
    )));

    OrganizationSettingsManager::new(metadata_provider, template_loader)
}

// ============================================================================
// Tier 1: Specification tests — zero / one / many
// ============================================================================

#[tokio::test]
async fn test_find_designated_template_returns_none_when_zero_templates_match() {
    let manager = manager_with(
        vec!["plain-template"],
        HashMap::from([("plain-template", None)]),
    );

    let result = manager
        .find_designated_template("acme", TemplateDesignation::Fork)
        .await
        .expect("lookup should succeed");

    assert_eq!(result, None);
}

#[tokio::test]
async fn test_find_designated_template_returns_none_when_org_has_no_templates_at_all() {
    let manager = manager_with(vec![], HashMap::new());

    let result = manager
        .find_designated_template("acme", TemplateDesignation::SourceOnlyMirror)
        .await
        .expect("lookup should succeed");

    assert_eq!(result, None);
}

#[tokio::test]
async fn test_find_designated_template_returns_the_single_match() {
    let manager = manager_with(
        vec!["fork-settings", "plain-template"],
        HashMap::from([
            ("fork-settings", Some(TemplateDesignation::Fork)),
            ("plain-template", None),
        ]),
    );

    let result = manager
        .find_designated_template("acme", TemplateDesignation::Fork)
        .await
        .expect("lookup should succeed");

    assert_eq!(
        result.map(|n| n.as_str().to_string()),
        Some("fork-settings".to_string())
    );
}

/// Stub-killing: a template designated for a *different* role must not be
/// returned as a match.
#[tokio::test]
async fn test_find_designated_template_ignores_templates_with_a_different_designation() {
    let manager = manager_with(
        vec!["mirror-settings"],
        HashMap::from([(
            "mirror-settings",
            Some(TemplateDesignation::SourceHistoryMirror),
        )]),
    );

    let result = manager
        .find_designated_template("acme", TemplateDesignation::Fork)
        .await
        .expect("lookup should succeed");

    assert_eq!(result, None);
}

// ============================================================================
// Tier 2: Adversarial — ambiguity detection
// ============================================================================

#[tokio::test]
async fn test_find_designated_template_errors_when_two_templates_share_a_designation() {
    let manager = manager_with(
        vec!["fork-a", "fork-b"],
        HashMap::from([
            ("fork-a", Some(TemplateDesignation::Fork)),
            ("fork-b", Some(TemplateDesignation::Fork)),
        ]),
    );

    let result = manager
        .find_designated_template("acme", TemplateDesignation::Fork)
        .await;

    assert!(
        result.is_err(),
        "two templates with the same designation must be rejected as ambiguous"
    );
    match result.unwrap_err() {
        ConfigurationError::AmbiguousTemplateDesignation { org, templates, .. } => {
            assert_eq!(org, "acme");
            assert!(templates.contains(&"fork-a".to_string()));
            assert!(templates.contains(&"fork-b".to_string()));
        }
        other => panic!("expected AmbiguousTemplateDesignation, got {other:?}"),
    }
}

#[tokio::test]
async fn test_find_designated_template_errors_when_three_templates_share_a_designation() {
    let manager = manager_with(
        vec!["mirror-a", "mirror-b", "mirror-c"],
        HashMap::from([
            ("mirror-a", Some(TemplateDesignation::SourceOnlyMirror)),
            ("mirror-b", Some(TemplateDesignation::SourceOnlyMirror)),
            ("mirror-c", Some(TemplateDesignation::SourceOnlyMirror)),
        ]),
    );

    let result = manager
        .find_designated_template("acme", TemplateDesignation::SourceOnlyMirror)
        .await;

    assert!(result.is_err());
}

/// Two templates sharing a designation for a *different* role than the one
/// being queried must not trigger a false-positive ambiguity error.
#[tokio::test]
async fn test_find_designated_template_does_not_confuse_ambiguity_across_designations() {
    let manager = manager_with(
        vec!["fork-a", "fork-b", "mirror-only"],
        HashMap::from([
            ("fork-a", Some(TemplateDesignation::Fork)),
            ("fork-b", Some(TemplateDesignation::Fork)),
            ("mirror-only", Some(TemplateDesignation::SourceOnlyMirror)),
        ]),
    );

    // Querying for SourceOnlyMirror should find exactly one match (mirror-only)
    // and must not be affected by the Fork ambiguity.
    let result = manager
        .find_designated_template("acme", TemplateDesignation::SourceOnlyMirror)
        .await
        .expect("source-only-mirror lookup should succeed despite unrelated fork ambiguity");

    assert_eq!(
        result.map(|n| n.as_str().to_string()),
        Some("mirror-only".to_string())
    );
}
