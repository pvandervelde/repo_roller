//! Property-based tests for `creation_capabilities` merge precedence and
//! override-policy enforcement (CRA-001/CRA-002 applied to
//! `CreationCapabilitiesSettings`).
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::template_config::TemplateMetadata;
use proptest::prelude::*;

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

fn global_with_fork(value: bool, override_allowed: bool) -> GlobalDefaults {
    GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::new(value, override_allowed)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn team_with_fork(value: bool) -> TeamConfig {
    TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(value)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn global_with_allowed_source_owners(value: Vec<String>, override_allowed: bool) -> GlobalDefaults {
    GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::new(value, override_allowed)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn team_with_allowed_source_owners(value: Vec<String>) -> TeamConfig {
    TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            allowed_source_owners: Some(OverridableValue::allowed(value)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Small, distinct alphabet strategy for generating owner-name lists:
/// short lowercase strings are enough to exercise equality/precedence
/// behavior without needing to satisfy `OrganizationName`'s full grammar
/// (the allow-list is plain `Vec<String>`, compared before any
/// `OrganizationName` validation runs).
fn owner_list_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z]{3,6}", 0..4)
}

proptest! {
    /// For any combination of a global `fork_enabled` policy and a team-level
    /// attempt to set it: the merge succeeds whenever the global value allows
    /// override OR the team's value matches the global value exactly, and
    /// fails with `OverrideNotPermitted` in every other case. This holds for
    /// all four `(global_value, global_override_allowed, team_value)` corners
    /// and must hold for every value in between (trivially just the two
    /// boolean values here, but the invariant generalizes).
    #[test]
    fn prop_override_enforcement_matches_policy_for_any_value_combination(
        global_value in any::<bool>(),
        global_override_allowed in any::<bool>(),
        team_value in any::<bool>(),
    ) {
        let merger = ConfigurationMerger::new();
        let global = global_with_fork(global_value, global_override_allowed);
        let team = team_with_fork(team_value);

        let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());

        let override_is_a_noop = global_value == team_value;
        if global_override_allowed || override_is_a_noop {
            let merged = result.expect("override should be permitted or a no-op");
            prop_assert_eq!(
                merged.creation_capabilities.fork_enabled.map(|v| v.value),
                Some(team_value)
            );
        } else {
            prop_assert!(result.is_err(), "non-matching override of a fixed policy must be rejected");
        }
    }

    /// Precedence invariant: whichever of {template, team, repository_type,
    /// global} is the highest-precedence level that actually sets
    /// `source_history_mirror_enabled` (with `override_allowed = true` at
    /// every lower level so no rejection occurs), that level's value is the
    /// one that appears in the merged configuration — for any subset of
    /// levels providing a value and any boolean values chosen.
    #[test]
    fn prop_highest_precedence_level_wins_for_any_subset_of_levels_present(
        global_opt in proptest::option::of(any::<bool>()),
        repo_type_opt in proptest::option::of(any::<bool>()),
        team_opt in proptest::option::of(any::<bool>()),
        template_opt in proptest::option::of(any::<bool>()),
    ) {
        let merger = ConfigurationMerger::new();

        let global = GlobalDefaults {
            creation_capabilities: global_opt.map(|v| CreationCapabilitiesSettings {
                source_history_mirror_enabled: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        };
        let repo_type = repo_type_opt.map(|v| RepositoryTypeConfig {
            creation_capabilities: Some(CreationCapabilitiesSettings {
                source_history_mirror_enabled: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        });
        let team = team_opt.map(|v| TeamConfig {
            creation_capabilities: Some(CreationCapabilitiesSettings {
                source_history_mirror_enabled: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        });
        let template = NewTemplateConfig {
            creation_capabilities: template_opt.map(|v| CreationCapabilitiesSettings {
                source_history_mirror_enabled: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..empty_template()
        };

        let result = merger.merge_configurations(&global, repo_type.as_ref(), team.as_ref(), &template);
        let merged = result.expect("all-overridable levels must never be rejected");

        // Highest precedence level that actually set a value wins.
        let expected = template_opt.or(team_opt).or(repo_type_opt).or(global_opt);

        prop_assert_eq!(
            merged.creation_capabilities.source_history_mirror_enabled.map(|v| v.value),
            expected
        );
    }

    /// Same override-enforcement invariant as
    /// `prop_override_enforcement_matches_policy_for_any_value_combination`,
    /// but for the `Vec<String>`-typed `allowed_source_owners` field: the
    /// merge succeeds whenever the global policy allows override OR the
    /// team's list is exactly equal to the global list, and fails with
    /// `OverrideNotPermitted` otherwise, for any generated pair of owner
    /// lists. This is the field that newly requires `OverridableValue<T>`'s
    /// override-enforcement path to work for a non-`bool`, non-`Display`
    /// `T` — a fix that only special-cases `bool` would fail to compile or
    /// fail this property.
    #[test]
    fn prop_allowed_source_owners_override_enforcement_matches_policy_for_any_value_combination(
        global_value in owner_list_strategy(),
        global_override_allowed in any::<bool>(),
        team_value in owner_list_strategy(),
    ) {
        let merger = ConfigurationMerger::new();
        let global = global_with_allowed_source_owners(global_value.clone(), global_override_allowed);
        let team = team_with_allowed_source_owners(team_value.clone());

        let result = merger.merge_configurations(&global, None, Some(&team), &empty_template());

        let override_is_a_noop = global_value == team_value;
        if global_override_allowed || override_is_a_noop {
            let merged = result.expect("override should be permitted or a no-op");
            prop_assert_eq!(
                merged.creation_capabilities.allowed_source_owners.map(|v| v.value),
                Some(team_value)
            );
        } else {
            prop_assert!(result.is_err(), "non-matching override of a fixed allow-list must be rejected");
        }
    }

    /// Precedence invariant for `allowed_source_owners`, mirroring
    /// `prop_highest_precedence_level_wins_for_any_subset_of_levels_present`:
    /// whichever of {template, team, repository_type, global} is the
    /// highest-precedence level that actually sets the field (all levels
    /// `override_allowed = true`, so no rejection occurs) is the one whose
    /// *entire list* appears in the merged configuration — never a union of
    /// multiple levels' lists, for any subset of levels providing a value.
    #[test]
    fn prop_allowed_source_owners_highest_precedence_level_wins_for_any_subset_of_levels_present(
        global_opt in proptest::option::of(owner_list_strategy()),
        repo_type_opt in proptest::option::of(owner_list_strategy()),
        team_opt in proptest::option::of(owner_list_strategy()),
        template_opt in proptest::option::of(owner_list_strategy()),
    ) {
        let merger = ConfigurationMerger::new();

        let global = GlobalDefaults {
            creation_capabilities: global_opt.clone().map(|v| CreationCapabilitiesSettings {
                allowed_source_owners: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        };
        let repo_type = repo_type_opt.clone().map(|v| RepositoryTypeConfig {
            creation_capabilities: Some(CreationCapabilitiesSettings {
                allowed_source_owners: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        });
        let team = team_opt.clone().map(|v| TeamConfig {
            creation_capabilities: Some(CreationCapabilitiesSettings {
                allowed_source_owners: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..Default::default()
        });
        let template = NewTemplateConfig {
            creation_capabilities: template_opt.clone().map(|v| CreationCapabilitiesSettings {
                allowed_source_owners: Some(OverridableValue::allowed(v)),
                ..Default::default()
            }),
            ..empty_template()
        };

        let result = merger.merge_configurations(&global, repo_type.as_ref(), team.as_ref(), &template);
        let merged = result.expect("all-overridable levels must never be rejected");

        // Highest precedence level that actually set a value wins, in full
        // (not unioned with any lower level's list).
        let expected = template_opt.or(team_opt).or(repo_type_opt).or(global_opt);

        prop_assert_eq!(
            merged.creation_capabilities.allowed_source_owners.map(|v| v.value),
            expected
        );
    }
}
