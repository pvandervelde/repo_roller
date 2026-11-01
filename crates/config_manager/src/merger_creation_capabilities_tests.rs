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
// Tier 1: Specification tests — `allowed_source_owners` precedence
// (docs/spec/interfaces/fork-mirror-operations.md §2, "Allow-list of source
// owners": merged through the *same* precedence as the `*_enabled` flags,
// NOT additive across levels — the highest-precedence level that sets the
// field wins outright, it does not get unioned with lower levels' lists.)
// ============================================================================

#[test]
fn test_global_only_allowed_source_owners_is_reflected_in_merged_configuration() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["acme".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged
            .creation_capabilities
            .allowed_source_owners
            .map(|v| v.value),
        Some(vec!["acme".to_string()])
    );
}

#[test]
fn test_team_overrides_repository_type_allowed_source_owners() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let repo_type = RepositoryTypeConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec![
                "repo-type-org".to_string()
            ])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["team-org".to_string()])),
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
            .allowed_source_owners
            .map(|v| v.value),
        Some(vec!["team-org".to_string()]),
        "team level should win over repository-type — allow-lists are replaced, not unioned"
    );
}

#[test]
fn test_template_overrides_team_allowed_source_owners() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["team-org".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_capabilities(CreationCapabilitiesSettings {
        allowed_source_owners: Some(OverridableValue::allowed(vec!["template-org".to_string()])),
        ..Default::default()
    });

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &template)
        .expect("merge should succeed");

    assert_eq!(
        merged
            .creation_capabilities
            .allowed_source_owners
            .map(|v| v.value),
        Some(vec!["template-org".to_string()]),
        "template level (highest precedence) should win over team"
    );
}

#[test]
fn test_absent_allowed_source_owners_at_every_level_resolves_to_none() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert!(
        merged.creation_capabilities.allowed_source_owners.is_none(),
        "fail-closed default: no source owner is permitted when the field is never set"
    );
}

// ============================================================================
// Tier 2: Adversarial — `allowed_source_owners` override enforcement
// ============================================================================

/// Stub-killing / CRA-002: a fixed (`override_allowed = false`) allow-list at
/// global level is an organization policy; team must not be able to replace
/// it with a different list.
#[test]
fn test_team_cannot_override_fixed_global_allowed_source_owners() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::fixed(vec!["locked-org".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec![
                "team-wants-this-org".to_string()
            ])),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());

    assert!(
        result.is_err(),
        "team must not override a fixed global source-owner allow-list policy"
    );
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(
                setting.contains("allowed_source_owners"),
                "error should reference the offending field, got: {setting}"
            );
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

/// A fixed allow-list re-asserted with the *exact same* list at a higher
/// level is not an override attempt and must succeed (mirrors the
/// `*_enabled` boolean fields' identical-value short-circuit).
#[test]
fn test_reasserting_identical_fixed_allowed_source_owners_does_not_error() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::fixed(vec!["acme".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["acme".to_string()])), // same list
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());
    assert!(
        result.is_ok(),
        "identical re-assertion of the allow-list must not error: {result:?}"
    );
}

/// Stub-killing: a fixed allow-list re-asserted with the same *set* of
/// owners but a different *order* is, by the plain `Vec<String>` equality
/// the rest of this struct's merge logic uses, a *different* value — an
/// implementation that silently treats reordering as a no-op (e.g. by
/// sorting before comparing) would diverge from the `*_enabled` fields'
/// documented exact-equality semantics and should be caught here.
#[test]
fn test_reasserting_fixed_allowed_source_owners_with_different_order_is_treated_as_a_change() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::fixed(vec![
                "acme".to_string(),
                "widgets".to_string(),
            ])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec![
                "widgets".to_string(),
                "acme".to_string(),
            ])),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());

    assert!(
        result.is_err(),
        "a reordered list is a different `Vec<String>` value under plain equality, \
         so it must be rejected the same way any other override of a fixed policy is"
    );
}

/// Stub-killing: setting `allowed_source_owners` at a higher level must not
/// perturb the independently-inherited `*_enabled` flags from a lower
/// level — rules out an implementation that merges the whole
/// `CreationCapabilitiesSettings` struct wholesale instead of field-by-field.
#[test]
fn test_allowed_source_owners_override_does_not_perturb_enabled_flags_from_lower_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            allowed_source_owners: Some(OverridableValue::allowed(vec!["global-org".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["team-org".to_string()])), // only this
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged
            .creation_capabilities
            .allowed_source_owners
            .map(|v| v.value),
        Some(vec!["team-org".to_string()]),
        "team override should apply to allowed_source_owners"
    );
    assert_eq!(
        merged.creation_capabilities.fork_enabled.map(|v| v.value),
        Some(true),
        "fork_enabled must be inherited from global, untouched by team's allow-list override"
    );
}

/// Symmetric to the test above: setting `fork_enabled` at a higher level
/// must not perturb an independently-inherited `allowed_source_owners` list.
#[test]
fn test_enabled_flag_override_does_not_perturb_allowed_source_owners_from_lower_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(false)),
            allowed_source_owners: Some(OverridableValue::allowed(vec!["global-org".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)), // only this
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.creation_capabilities.fork_enabled.map(|v| v.value),
        Some(true),
        "team override should apply to fork_enabled"
    );
    assert_eq!(
        merged
            .creation_capabilities
            .allowed_source_owners
            .map(|v| v.value),
        Some(vec!["global-org".to_string()]),
        "allowed_source_owners must be inherited from global, untouched by team's fork_enabled override"
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
            ..Default::default()
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

/// Mutation-kill: `track_creation_capabilities_sources` must actually record
/// a source-trace entry for every non-`None` capability field at Global
/// level — a no-op implementation would still pass every value-assertion
/// test above (the merged *value* is correct either way) but would silently
/// drop the audit trail used by `MergedConfiguration::get_source`.
#[test]
fn test_global_creation_capabilities_are_source_tracked() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            source_history_mirror_enabled: Some(OverridableValue::allowed(false)),
            source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.get_source("creation_capabilities.fork_enabled"),
        Some(ConfigurationSource::Global),
        "fork_enabled should be tracked as Global source"
    );
    assert_eq!(
        merged.get_source("creation_capabilities.source_history_mirror_enabled"),
        Some(ConfigurationSource::Global),
        "source_history_mirror_enabled should be tracked as Global source"
    );
    assert_eq!(
        merged.get_source("creation_capabilities.source_only_mirror_enabled"),
        Some(ConfigurationSource::Global),
        "source_only_mirror_enabled should be tracked as Global source"
    );
}

/// Mutation-kill: the allow-list field must be source-tracked the same way
/// the three `*_enabled` flags are — a no-op implementation would still
/// pass every value-assertion test above but would silently drop the audit
/// trail used by `MergedConfiguration::get_source`.
#[test]
fn test_global_allowed_source_owners_is_source_tracked() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(vec!["acme".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &empty_template())
        .expect("merge should succeed");

    assert_eq!(
        merged.get_source("creation_capabilities.allowed_source_owners"),
        Some(ConfigurationSource::Global),
        "allowed_source_owners should be tracked as Global source"
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
