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
}
