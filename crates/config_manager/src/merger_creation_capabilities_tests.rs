//! Tests for `ConfigurationMerger::merge_creation_capabilities_settings` and its
//! wiring into `merge_configurations` across the four hierarchy levels.
//!
//! Mirrors the existing `merge_repository_settings` test structure
//! (see `merger_tests.rs`) per CRA-001 (precedence: template > team >
//! repository-type > global) and CRA-002 (override policy enforcement).
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::template_config::TemplateMetadata;

// ============================================================================
// Test Helpers
// ============================================================================

fn empty_template() -> NewTemplateConfig {
    NewTemplateConfig {
        template: TemplateMetadata {
            name: "t".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
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
        designation: None,
        creation_capabilities: None,
    }
}

fn template_with_capabilities(capabilities: CreationCapabilitiesSettings) -> NewTemplateConfig {
    NewTemplateConfig {
        creation_capabilities: Some(capabilities),
        ..empty_template()
    }
}

// ============================================================================
// Tier 1: Specification tests — field wiring and precedence
// ============================================================================

#[test]
fn test_merged_configuration_exposes_creation_capabilities_field() {
    let merged = MergedConfiguration::new();
    // All-absent at every level resolves to the all-None default.
    assert_eq!(
        merged.creation_capabilities,
        CreationCapabilitiesSettings::default()
    );
}

#[test]
fn test_global_only_fork_enabled_is_reflected_in_merged_configuration() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.creation_capabilities.fork_enabled.map(|v| v.value),
        Some(true)
    );
}

#[test]
fn test_repository_type_overrides_global_fork_enabled() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let repo_type = RepositoryTypeConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, Some(&repo_type), None, &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.creation_capabilities.fork_enabled.map(|v| v.value),
        Some(true),
        "repository-type level should win over global"
    );
}

#[test]
fn test_team_overrides_repository_type_source_history_mirror_enabled() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let repo_type = RepositoryTypeConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_history_mirror_enabled: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_history_mirror_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, Some(&repo_type), Some(&team), &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged
            .creation_capabilities
            .source_history_mirror_enabled
            .map(|v| v.value),
        Some(true),
        "team level should win over repository-type"
    );
}

#[test]
fn test_template_overrides_team_source_only_mirror_enabled() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_only_mirror_enabled: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_capabilities(CreationCapabilitiesSettings {
        source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    });

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &template)
        .expect("merge should succeed");

    assert_eq!(
        merged
            .creation_capabilities
            .source_only_mirror_enabled
            .map(|v| v.value),
        Some(true),
        "template level (highest precedence) should win over team"
    );
}

// ============================================================================
// Tier 2: Adversarial — override enforcement and field independence
// ============================================================================

/// Stub-killing / CRA-002: a fixed (`override_allowed = false`) capability at
/// global level is an organization policy; team must not be able to flip it.
#[test]
fn test_team_cannot_override_fixed_global_capability() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_only_mirror_enabled: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());

    assert!(
        result.is_err(),
        "team must not override a fixed global creation-capability policy"
    );
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(
                setting.contains("source_only_mirror_enabled"),
                "error should reference the offending field, got: {setting}"
            );
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

#[test]
fn test_template_cannot_override_fixed_global_fork_capability() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_capabilities(CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(setting.contains("fork_enabled"));
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

/// A fixed value re-asserted with the *same* value at a higher level is not
/// an override attempt and must succeed (mirrors `validate_override`'s
/// equality short-circuit for `RepositorySettings`).
#[test]
fn test_reasserting_identical_fixed_value_does_not_error() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::fixed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)), // same value
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());
    assert!(
        result.is_ok(),
        "identical re-assertion must not error: {result:?}"
    );
}

/// Stub-killing: setting one capability field at a higher level must not
/// perturb a different capability field inherited from a lower level —
/// rules out an implementation that merges the whole struct wholesale
/// (losing unset fields) instead of field-by-field.
#[test]
fn test_capability_fields_merge_independently_across_levels() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            source_history_mirror_enabled: Some(OverridableValue::allowed(true)),
            source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(false)), // only override this one
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.creation_capabilities.fork_enabled.map(|v| v.value),
        Some(false),
        "team override should apply to fork_enabled"
    );
    assert_eq!(
        merged
            .creation_capabilities
            .source_history_mirror_enabled
            .map(|v| v.value),
        Some(true),
        "source_history_mirror_enabled must be inherited from global, untouched by team's fork_enabled override"
    );
    assert_eq!(
        merged
            .creation_capabilities
            .source_only_mirror_enabled
            .map(|v| v.value),
        Some(true),
        "source_only_mirror_enabled must be inherited from global, untouched by team's fork_enabled override"
    );
}

#[test]
fn test_absent_creation_capabilities_at_every_level_resolves_to_all_none() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert!(merged.creation_capabilities.fork_enabled.is_none());
    assert!(merged
        .creation_capabilities
        .source_history_mirror_enabled
        .is_none());
    assert!(merged
        .creation_capabilities
        .source_only_mirror_enabled
        .is_none());
}
