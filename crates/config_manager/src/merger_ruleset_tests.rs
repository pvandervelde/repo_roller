//! Tests for ruleset merging in configuration merger.

use super::*;
use crate::settings::ruleset::{
    BypassActorConfig, RefNameConditionConfig, RuleConfig, RulesetConditionsConfig, RulesetConfig,
};
use crate::template_config::TemplateMetadata;

/// Creates a minimal template config for testing.
fn create_test_template() -> NewTemplateConfig {
    NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
    }
}

/// Test rulesets from global config are merged.
#[test]
fn test_merge_global_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: Some(vec![RulesetConfig {
            name: "main-protection".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: Some(RulesetConditionsConfig {
                ref_name: RefNameConditionConfig {
                    include: vec!["refs/heads/main".to_string()],
                    exclude: vec![],
                },
            }),
            rules: vec![RuleConfig::Deletion],
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, None, &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();
    assert_eq!(merged.rulesets.len(), 1);
    assert_eq!(merged.rulesets[0].name, "main-protection");
    assert_eq!(merged.rulesets[0].target, "branch");
    assert_eq!(merged.rulesets[0].enforcement, "active");

    // Verify source tracking
    assert_eq!(
        merged.get_source("rulesets"),
        Some(ConfigurationSource::Global)
    );
}

/// Test rulesets from repository type config are merged additively with global.
#[test]
fn test_merge_repository_type_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: Some(vec![RulesetConfig {
            name: "org-policy".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: None,
            rules: vec![RuleConfig::Deletion],
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        rulesets: Some(vec![RulesetConfig {
            name: "library-standards".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: Some(RulesetConditionsConfig {
                ref_name: RefNameConditionConfig {
                    include: vec!["refs/heads/main".to_string()],
                    exclude: vec![],
                },
            }),
            rules: vec![RuleConfig::PullRequest {
                required_approving_review_count: Some(2),
                dismiss_stale_reviews_on_push: None,
                require_code_owner_review: None,
                require_last_push_approval: None,
                required_review_thread_resolution: None,
                allowed_merge_methods: None,
            }],
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();

    // Both rulesets should be present (additive merge)
    assert_eq!(merged.rulesets.len(), 2);
    assert!(merged.rulesets.iter().any(|r| r.name == "org-policy"));
    assert!(merged
        .rulesets
        .iter()
        .any(|r| r.name == "library-standards"));
}

/// Test rulesets from team config are merged additively.
#[test]
fn test_merge_team_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: Some(vec![RulesetConfig {
            name: "global-rule".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: None,
            rules: vec![],
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        rulesets: Some(vec![RulesetConfig {
            name: "team-rule".to_string(),
            target: "tag".to_string(),
            enforcement: "evaluate".to_string(),
            bypass_actors: vec![BypassActorConfig {
                actor_id: 1,
                actor_type: "Team".to_string(),
                bypass_mode: "always".to_string(),
            }],
            conditions: None,
            rules: vec![],
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();

    // Both rulesets should be present
    assert_eq!(merged.rulesets.len(), 2);
    assert!(merged.rulesets.iter().any(|r| r.name == "global-rule"));
    assert!(merged.rulesets.iter().any(|r| r.name == "team-rule"));
}

/// Test rulesets from template config are merged additively.
#[test]
fn test_merge_template_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults::default();

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        rulesets: Some(vec![RulesetConfig {
            name: "template-rule".to_string(),
            target: "push".to_string(),
            enforcement: "disabled".to_string(),
            bypass_actors: vec![],
            conditions: None,
            rules: vec![RuleConfig::Creation],
        }]),
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
    };

    let result = merger.merge_configurations(&global, None, None, &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();
    assert_eq!(merged.rulesets.len(), 1);
    assert_eq!(merged.rulesets[0].name, "template-rule");
    assert_eq!(merged.rulesets[0].target, "push");

    // Verify source tracking
    assert_eq!(
        merged.get_source("rulesets"),
        Some(ConfigurationSource::Template)
    );
}

/// Test rulesets from all levels are merged additively.
#[test]
fn test_merge_all_levels_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: Some(vec![RulesetConfig {
            name: "global-main-protection".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: Some(RulesetConditionsConfig {
                ref_name: RefNameConditionConfig {
                    include: vec!["refs/heads/main".to_string()],
                    exclude: vec![],
                },
            }),
            rules: vec![RuleConfig::Deletion],
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        rulesets: Some(vec![RulesetConfig {
            name: "type-pr-rules".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: None,
            rules: vec![RuleConfig::PullRequest {
                required_approving_review_count: Some(1),
                dismiss_stale_reviews_on_push: None,
                require_code_owner_review: None,
                require_last_push_approval: None,
                required_review_thread_resolution: None,
                allowed_merge_methods: None,
            }],
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        rulesets: Some(vec![RulesetConfig {
            name: "team-tag-protection".to_string(),
            target: "tag".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: Some(RulesetConditionsConfig {
                ref_name: RefNameConditionConfig {
                    include: vec!["refs/tags/v*".to_string()],
                    exclude: vec![],
                },
            }),
            rules: vec![RuleConfig::Deletion],
        }]),
        ..Default::default()
    };

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        rulesets: Some(vec![RulesetConfig {
            name: "template-security".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![],
            conditions: None,
            rules: vec![RuleConfig::RequiredSignatures],
        }]),
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), Some(&team), &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();

    // All 4 rulesets from all levels should be present
    assert_eq!(merged.rulesets.len(), 4);
    assert!(merged
        .rulesets
        .iter()
        .any(|r| r.name == "global-main-protection"));
    assert!(merged.rulesets.iter().any(|r| r.name == "type-pr-rules"));
    assert!(merged
        .rulesets
        .iter()
        .any(|r| r.name == "team-tag-protection"));
    assert!(merged
        .rulesets
        .iter()
        .any(|r| r.name == "template-security"));

    // Verify different targets are preserved
    assert_eq!(
        merged
            .rulesets
            .iter()
            .filter(|r| r.target == "branch")
            .count(),
        3
    );
    assert_eq!(
        merged.rulesets.iter().filter(|r| r.target == "tag").count(),
        1
    );
}

/// Test empty rulesets don't cause issues.
#[test]
fn test_merge_empty_rulesets() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: None,
        ..Default::default()
    };

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        rulesets: None,
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
    };

    let result = merger.merge_configurations(&global, None, None, &template);
    assert!(result.is_ok(), "Merge should succeed with no rulesets");

    let merged = result.unwrap();
    assert_eq!(merged.rulesets.len(), 0, "Should have no rulesets");
}

/// Test bypass actors are preserved in merged rulesets.
#[test]
fn test_merge_rulesets_preserves_bypass_actors() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        rulesets: Some(vec![RulesetConfig {
            name: "with-bypass".to_string(),
            target: "branch".to_string(),
            enforcement: "active".to_string(),
            bypass_actors: vec![
                BypassActorConfig {
                    actor_id: 1,
                    actor_type: "OrganizationAdmin".to_string(),
                    bypass_mode: "always".to_string(),
                },
                BypassActorConfig {
                    actor_id: 123,
                    actor_type: "Team".to_string(),
                    bypass_mode: "pull_request".to_string(),
                },
            ],
            conditions: None,
            rules: vec![],
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, None, &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();
    assert_eq!(merged.rulesets.len(), 1);
    assert_eq!(merged.rulesets[0].bypass_actors.len(), 2);
    assert_eq!(
        merged.rulesets[0].bypass_actors[0].actor_type,
        "OrganizationAdmin"
    );
    assert_eq!(merged.rulesets[0].bypass_actors[1].actor_type, "Team");
}
