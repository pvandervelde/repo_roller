//! Tests for configuration merging engine.

use super::*;
use crate::settings::custom_property::CustomPropertyValue;
use crate::settings::environment::EnvironmentProtectionRules;
use crate::settings::ActionSettings;
use crate::template_config::TemplateMetadata;

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a minimal GlobalDefaults for testing.
fn create_test_global_defaults() -> GlobalDefaults {
    GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            wiki: Some(OverridableValue::fixed(false)), // Policy: no wikis
            security_advisories: Some(OverridableValue::fixed(true)), // Security policy
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            required_approving_review_count: Some(OverridableValue::allowed(1)),
            require_conversation_resolution: Some(OverridableValue::fixed(true)), // Quality policy
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Creates a minimal template config for testing.
fn create_test_template() -> NewTemplateConfig {
    NewTemplateConfig {
        template: TemplateMetadata {
            name: "test-template".to_string(),
            description: "Test template".to_string(),
            author: "Test Author".to_string(),
            tags: vec!["test".to_string()],
        },
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            projects: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    }
}

/// Helper to create a template with custom repository settings.
fn create_template_with_repository(repo_settings: RepositorySettings) -> NewTemplateConfig {
    NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        repository: Some(repo_settings),
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    }
}

// ============================================================================
// ConfigurationMerger Creation Tests
// ============================================================================

/// Verify ConfigurationMerger can be created.
#[test]
fn test_merger_creation() {
    let merger = ConfigurationMerger::new();
    assert!(
        format!("{:?}", merger).contains("ConfigurationMerger"),
        "Merger should be created successfully"
    );
}

/// Verify ConfigurationMerger implements Default.
#[test]
fn test_merger_default() {
    let merger = ConfigurationMerger::default();
    assert!(
        format!("{:?}", merger).contains("ConfigurationMerger"),
        "Merger should support default construction"
    );
}

// ============================================================================
// Basic Merging Tests (Task 4.1 & 4.2)
// ============================================================================

/// Verify merging with only global defaults and template.
///
/// Template should override global defaults for matching fields.
#[test]
fn test_merge_global_and_template_only() {
    let merger = ConfigurationMerger::new();
    let global = create_test_global_defaults();
    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_ok(), "Merging global and template should succeed");

    let merged = result.unwrap();

    // Template should override issues setting
    assert_eq!(
        merged.repository.issues.as_ref().map(|v| v.value),
        Some(true),
        "Template should override issues setting"
    );

    // Template provides projects setting
    assert_eq!(
        merged.repository.projects.as_ref().map(|v| v.value),
        Some(false),
        "Template should provide projects setting"
    );

    // Global wiki policy should be preserved
    assert_eq!(
        merged.repository.wiki.as_ref().map(|v| v.value),
        Some(false),
        "Global wiki policy should be preserved"
    );
}

/// Verify four-level hierarchy: Global → Repository Type → Team → Template.
///
/// Each level should override the previous level according to precedence.
#[test]
fn test_merge_four_level_hierarchy() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            projects: Some(OverridableValue::allowed(true)),
            discussions: Some(OverridableValue::allowed(false)),
            wiki: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        repository: Some(RepositorySettings {
            projects: Some(OverridableValue::allowed(false)), // Override global
            discussions: Some(OverridableValue::allowed(true)), // Override global
            ..Default::default()
        }),
        ..Default::default()
    };

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(OverridableValue::allowed(false)), // Override repo type
            wiki: Some(OverridableValue::allowed(false)),        // Override global
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(false)), // Override global
            ..Default::default()
        }),
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), Some(&team), &template);

    assert!(
        result.is_ok(),
        "Four-level merge should succeed: {:?}",
        result.err()
    );

    let merged = result.unwrap();

    // Verify each level's contribution
    assert_eq!(
        merged.repository.issues.as_ref().map(|v| v.value),
        Some(false),
        "Template should override issues (Template > all)"
    );
    assert_eq!(
        merged.repository.projects.as_ref().map(|v| v.value),
        Some(false),
        "Repository type should override projects (RepositoryType > Global)"
    );
    assert_eq!(
        merged.repository.discussions.as_ref().map(|v| v.value),
        Some(false),
        "Team should override discussions (Team > RepositoryType > Global)"
    );
    assert_eq!(
        merged.repository.wiki.as_ref().map(|v| v.value),
        Some(false),
        "Team should override wiki (Team > Global)"
    );
}

// ============================================================================
// Override Policy Enforcement Tests (Task 4.3)
// ============================================================================

/// Verify that team cannot override a non-overridable global setting.
///
/// This tests security policy enforcement.
#[test]
fn test_team_cannot_override_fixed_global_setting() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            security_advisories: Some(OverridableValue::fixed(true)), // Security policy
            ..Default::default()
        }),
        ..Default::default()
    };

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            security_advisories: Some(OverridableValue::allowed(false)), // Attempt override
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(
        result.is_err(),
        "Team should not be able to override non-overridable global setting"
    );

    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, reason } => {
            assert!(
                setting.contains("security_advisories"),
                "Error should reference security_advisories field"
            );
            assert!(
                !reason.is_empty(),
                "Error should explain policy restriction"
            );
        }
        other => panic!("Expected OverrideNotPermitted error, got {:?}", other),
    }
}

/// Verify that template cannot override a non-overridable global setting.
#[test]
fn test_template_cannot_override_fixed_global_setting() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            wiki: Some(OverridableValue::fixed(false)), // Policy: no wikis
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_template_with_repository(RepositorySettings {
        wiki: Some(OverridableValue::allowed(true)), // Attempt override
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(
        result.is_err(),
        "Template should not be able to override non-overridable global setting"
    );

    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(
                setting.contains("wiki"),
                "Error should reference wiki field"
            );
        }
        other => panic!("Expected OverrideNotPermitted error, got {:?}", other),
    }
}

/// Verify that repository type cannot override a non-overridable global setting.
#[test]
fn test_repository_type_cannot_override_fixed_global_setting() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        pull_requests: Some(PullRequestSettings {
            require_conversation_resolution: Some(OverridableValue::fixed(true)), // Quality policy
            ..Default::default()
        }),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        pull_requests: Some(PullRequestSettings {
            require_conversation_resolution: Some(OverridableValue::allowed(false)), // Attempt override
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);

    assert!(
        result.is_err(),
        "Repository type should not be able to override non-overridable global setting"
    );
}

/// Verify that overriding with the same value as fixed policy is allowed.
///
/// If team/template sets the same value as a fixed global setting, it should succeed.
#[test]
fn test_override_with_same_value_as_fixed_policy_succeeds() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            wiki: Some(OverridableValue::fixed(false)), // Policy: no wikis
            ..Default::default()
        }),
        ..Default::default()
    };

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            wiki: Some(OverridableValue::allowed(false)), // Same value as policy
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(
        result.is_ok(),
        "Setting same value as fixed policy should succeed"
    );
}

// ============================================================================
// Additive Collection Merging Tests (Task 4.4)
// ============================================================================

/// Verify that webhooks are merged additively (all webhooks combined).
#[test]
fn test_webhooks_merge_additively() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        webhooks: Some(vec![WebhookConfig {
            url: "https://global.example.com/webhook".to_string(),
            content_type: "json".to_string(),
            events: vec!["push".to_string()],
            active: true,
            secret: None,
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        webhooks: Some(vec![WebhookConfig {
            url: "https://team.example.com/webhook".to_string(),
            content_type: "json".to_string(),
            events: vec!["pull_request".to_string()],
            active: true,
            secret: None,
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(result.is_ok(), "Webhook merging should succeed");

    let merged = result.unwrap();

    // Should have 2 webhooks (global + team)
    assert_eq!(
        merged.webhooks.len(),
        2,
        "Should have webhooks from both sources"
    );

    // Verify both webhooks are present
    let urls: Vec<&str> = merged.webhooks.iter().map(|w| w.url.as_str()).collect();
    assert!(
        urls.contains(&"https://global.example.com/webhook"),
        "Global webhook should be present"
    );
    assert!(
        urls.contains(&"https://team.example.com/webhook"),
        "Team webhook should be present"
    );
}

/// Verify that GitHub Apps are merged additively.
#[test]
fn test_github_apps_merge_additively() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        github_apps: Some(vec![GitHubAppConfig {
            app_id: 12345,
            permissions: std::collections::HashMap::new(),
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        github_apps: Some(vec![GitHubAppConfig {
            app_id: 67890,
            permissions: std::collections::HashMap::new(),
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(result.is_ok(), "GitHub Apps merging should succeed");

    let merged = result.unwrap();

    // Should have 2 GitHub Apps (global + team)
    assert_eq!(
        merged.github_apps.len(),
        2,
        "Should have GitHub Apps from both sources"
    );

    let app_ids: Vec<u64> = merged.github_apps.iter().map(|a| a.app_id).collect();
    assert!(app_ids.contains(&12345), "Global app should be present");
    assert!(app_ids.contains(&67890), "Team app should be present");
}

// ============================================================================
// Source Tracking Tests (Task 4.1)
// ============================================================================

/// Verify that source trace correctly tracks configuration sources.
///
/// Each setting should record which configuration level provided it.
#[test]
fn test_source_trace_records_configuration_sources() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            wiki: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            projects: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_template_with_repository(RepositorySettings {
        discussions: Some(OverridableValue::allowed(true)),
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();

    // Verify source tracking
    assert_eq!(
        merged.get_source("repository.issues"),
        Some(ConfigurationSource::Global),
        "Issues should be tracked as Global source"
    );

    assert_eq!(
        merged.get_source("repository.projects"),
        Some(ConfigurationSource::Team),
        "Projects should be tracked as Team source"
    );

    assert_eq!(
        merged.get_source("repository.discussions"),
        Some(ConfigurationSource::Template),
        "Discussions should be tracked as Template source"
    );
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

/// Verify merging with all None/empty configurations.
#[test]
fn test_merge_with_empty_configurations() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults::default();
    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "minimal".to_string(),
            description: "minimal".to_string(),
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(
        result.is_ok(),
        "Merging empty configurations should succeed"
    );
}

/// Verify merging with None optional configurations.
#[test]
fn test_merge_with_none_optional_configs() {
    let merger = ConfigurationMerger::new();

    let global = create_test_global_defaults();
    let template = create_test_template();

    // No repository type, no team
    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(
        result.is_ok(),
        "Merging without optional configs should succeed"
    );
}

/// Verify deterministic merging - same inputs produce identical outputs.
#[test]
fn test_merge_is_deterministic() {
    let merger = ConfigurationMerger::new();

    let global = create_test_global_defaults();
    let template = create_test_template();

    let result1 = merger
        .merge_configurations(&global, None, None, &template)
        .unwrap();
    let result2 = merger
        .merge_configurations(&global, None, None, &template)
        .unwrap();

    // Results should be identical
    assert_eq!(
        result1, result2,
        "Merging same inputs should produce identical results"
    );
}

// ============================================================================
// Complex Scenario Tests
// ============================================================================

/// Verify complex scenario with multiple override levels and collections.
///
/// This tests the full integration of all merge features.
#[test]
fn test_complex_merge_scenario() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            wiki: Some(OverridableValue::fixed(false)),
            security_advisories: Some(OverridableValue::fixed(true)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            required_approving_review_count: Some(OverridableValue::allowed(1)),
            ..Default::default()
        }),
        webhooks: Some(vec![WebhookConfig {
            url: "https://global.example.com/webhook".to_string(),
            content_type: "json".to_string(),
            events: vec!["push".to_string()],
            active: true,
            secret: None,
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        repository: Some(RepositorySettings {
            projects: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            required_approving_review_count: Some(OverridableValue::allowed(2)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "complex".to_string(),
            description: "Complex template".to_string(),
            author: "Test".to_string(),
            tags: vec!["complex".to_string()],
        },
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(false)), // Override global
            ..Default::default()
        }),
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: Some(vec![WebhookConfig {
            url: "https://template.example.com/webhook".to_string(),
            content_type: "json".to_string(),
            events: vec!["pull_request".to_string()],
            active: true,
            secret: None,
        }]),
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), Some(&team), &template);

    assert!(
        result.is_ok(),
        "Complex merge scenario should succeed: {:?}",
        result.err()
    );

    let merged = result.unwrap();

    // Verify repository settings
    assert_eq!(
        merged.repository.issues.as_ref().map(|v| v.value),
        Some(false),
        "Template should override issues"
    );
    assert_eq!(
        merged.repository.wiki.as_ref().map(|v| v.value),
        Some(false),
        "Global wiki policy should be enforced"
    );
    assert_eq!(
        merged
            .repository
            .security_advisories
            .as_ref()
            .map(|v| v.value),
        Some(true),
        "Global security policy should be enforced"
    );
    assert_eq!(
        merged.repository.projects.as_ref().map(|v| v.value),
        Some(false),
        "Repository type should set projects"
    );
    assert_eq!(
        merged.repository.discussions.as_ref().map(|v| v.value),
        Some(true),
        "Team should set discussions"
    );

    // Verify PR settings
    assert_eq!(
        merged
            .pull_requests
            .required_approving_review_count
            .as_ref()
            .map(|v| v.value),
        Some(2),
        "Repository type should override review count"
    );

    // Verify collections
    assert_eq!(
        merged.webhooks.len(),
        2,
        "Should have webhooks from global and template"
    );
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

/// Test environment merging from multiple sources.
#[test]
fn test_environments_merge_additively() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        environments: Some(vec![EnvironmentConfig {
            name: "production".to_string(),
            protection_rules: Some(EnvironmentProtectionRules {
                required_reviewers: Some(vec!["admin-team".to_string()]),
                wait_timer: Some(60),
            }),
            deployment_branch_policy: None,
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        environments: Some(vec![EnvironmentConfig {
            name: "staging".to_string(),
            protection_rules: Some(EnvironmentProtectionRules {
                required_reviewers: Some(vec!["dev-team".to_string()]),
                wait_timer: Some(30),
            }),
            deployment_branch_policy: None,
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    assert_eq!(
        merged.environments.len(),
        2,
        "Should have environments from both sources"
    );

    let env_names: Vec<&str> = merged
        .environments
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(
        env_names.contains(&"production"),
        "Global environment should be present"
    );
    assert!(
        env_names.contains(&"staging"),
        "Team environment should be present"
    );
}

/// Test custom properties merging from multiple sources.
#[test]
fn test_custom_properties_merge_additively() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        custom_properties: Some(vec![CustomProperty {
            property_name: "department".to_string(),
            value: CustomPropertyValue::String("engineering".to_string()),
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        custom_properties: Some(vec![CustomProperty {
            property_name: "tech_stack".to_string(),
            value: CustomPropertyValue::String("rust".to_string()),
        }]),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    assert_eq!(
        merged.custom_properties.len(),
        2,
        "Should have custom properties from both sources"
    );

    let prop_names: Vec<&str> = merged
        .custom_properties
        .iter()
        .map(|p| p.property_name.as_str())
        .collect();
    assert!(
        prop_names.contains(&"department"),
        "Global property should be present"
    );
    assert!(
        prop_names.contains(&"tech_stack"),
        "Repository type property should be present"
    );
}

/// Test merging when repository type provides settings but global doesn't.
#[test]
fn test_repository_type_settings_without_global_base() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults::default(); // No settings

    let repo_type = RepositoryTypeConfig {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            projects: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            allow_auto_merge: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        branch_protection: Some(BranchProtectionSettings {
            require_pull_request_reviews: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Use a template with NO settings so it doesn't override
    let template = NewTemplateConfig {
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    // Verify repository type settings are applied even without global baseline
    assert_eq!(
        merged.repository.issues.as_ref().map(|v| v.value),
        Some(true),
        "Repository type issues setting should be applied"
    );
    assert_eq!(
        merged
            .pull_requests
            .allow_auto_merge
            .as_ref()
            .map(|v| v.value),
        Some(true),
        "Repository type PR setting should be applied"
    );
    assert_eq!(
        merged
            .branch_protection
            .require_pull_request_reviews
            .as_ref()
            .map(|v| v.value),
        Some(true),
        "Repository type branch protection should be applied"
    );

    // Verify source tracking
    assert_eq!(
        merged.get_source("repository.issues"),
        Some(ConfigurationSource::RepositoryType),
        "Source should be RepositoryType"
    );
}

/// Test merging when team provides settings but no global or repository type.
#[test]
fn test_team_settings_without_base() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults::default();

    let team = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            delete_branch_on_merge: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, Some(&team), &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    assert_eq!(
        merged.repository.discussions.as_ref().map(|v| v.value),
        Some(true),
        "Team setting should be applied"
    );
    assert_eq!(
        merged.get_source("repository.discussions"),
        Some(ConfigurationSource::Team),
        "Source should be Team"
    );
}

/// Test merging when template provides settings but no other layers.
#[test]
fn test_template_settings_without_base() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults::default();

    let template = NewTemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "test".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        repository: Some(RepositorySettings {
            wiki: Some(OverridableValue::allowed(false)),
            pages: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            allow_squash_merge: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        branch_protection: Some(BranchProtectionSettings {
            restrict_pushes: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        repository_type: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    assert_eq!(
        merged.repository.wiki.as_ref().map(|v| v.value),
        Some(false),
        "Template setting should be applied"
    );
    assert_eq!(
        merged.repository.pages.as_ref().map(|v| v.value),
        Some(true),
        "Template setting should be applied"
    );
    assert_eq!(
        merged
            .pull_requests
            .allow_squash_merge
            .as_ref()
            .map(|v| v.value),
        Some(true),
        "Template PR setting should be applied"
    );
    assert_eq!(
        merged
            .branch_protection
            .restrict_pushes
            .as_ref()
            .map(|v| v.value),
        Some(true),
        "Template branch protection should be applied"
    );

    assert_eq!(
        merged.get_source("repository.wiki"),
        Some(ConfigurationSource::Template),
        "Source should be Template"
    );
}

/// Test all repository settings fields are merged correctly.
#[test]
fn test_all_repository_settings_fields() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            projects: Some(OverridableValue::allowed(false)),
            discussions: Some(OverridableValue::allowed(true)),
            wiki: Some(OverridableValue::allowed(false)),
            pages: Some(OverridableValue::allowed(true)),
            security_advisories: Some(OverridableValue::allowed(true)),
            vulnerability_reporting: Some(OverridableValue::allowed(true)),
            auto_close_issues: Some(OverridableValue::allowed(false)),
        }),
        ..Default::default()
    };

    // Template with no repository settings to preserve global source
    let template = NewTemplateConfig {
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    // Verify all fields are present
    assert!(
        merged.repository.issues.is_some(),
        "Issues field should be set"
    );
    assert!(
        merged.repository.projects.is_some(),
        "Projects field should be set"
    );
    assert!(
        merged.repository.discussions.is_some(),
        "Discussions field should be set"
    );
    assert!(merged.repository.wiki.is_some(), "Wiki field should be set");
    assert!(
        merged.repository.pages.is_some(),
        "Pages field should be set"
    );
    assert!(
        merged.repository.security_advisories.is_some(),
        "Security advisories field should be set"
    );
    assert!(
        merged.repository.vulnerability_reporting.is_some(),
        "Vulnerability reporting field should be set"
    );
    assert!(
        merged.repository.auto_close_issues.is_some(),
        "Auto close issues field should be set"
    );

    // Verify source tracking for all fields
    assert_eq!(
        merged.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        merged.get_source("repository.auto_close_issues"),
        Some(ConfigurationSource::Global)
    );
}

/// Test all pull request settings fields are merged correctly.
#[test]
fn test_all_pull_request_settings_fields() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        pull_requests: Some(PullRequestSettings {
            allow_auto_merge: Some(OverridableValue::allowed(true)),
            allow_merge_commit: Some(OverridableValue::allowed(true)),
            allow_rebase_merge: Some(OverridableValue::allowed(false)),
            allow_squash_merge: Some(OverridableValue::allowed(true)),
            delete_branch_on_merge: Some(OverridableValue::allowed(true)),
            required_approving_review_count: Some(OverridableValue::allowed(2)),
            require_code_owner_reviews: Some(OverridableValue::allowed(true)),
            require_conversation_resolution: Some(OverridableValue::allowed(true)),
            merge_commit_title: None,
            merge_commit_message: None,
            squash_merge_commit_title: None,
            squash_merge_commit_message: None,
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    // Verify all PR fields are present
    assert!(merged.pull_requests.allow_auto_merge.is_some());
    assert!(merged.pull_requests.allow_merge_commit.is_some());
    assert!(merged.pull_requests.allow_rebase_merge.is_some());
    assert!(merged.pull_requests.allow_squash_merge.is_some());
    assert!(merged.pull_requests.delete_branch_on_merge.is_some());
    assert!(merged
        .pull_requests
        .required_approving_review_count
        .is_some());
    assert!(merged.pull_requests.require_code_owner_reviews.is_some());
    assert!(merged
        .pull_requests
        .require_conversation_resolution
        .is_some());

    // Verify source tracking
    assert_eq!(
        merged.get_source("pull_requests.allow_auto_merge"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        merged.get_source("pull_requests.require_conversation_resolution"),
        Some(ConfigurationSource::Global)
    );
}

/// Test all branch protection settings fields are merged correctly.
#[test]
fn test_all_branch_protection_settings_fields() {
    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        branch_protection: Some(BranchProtectionSettings {
            default_branch: Some(OverridableValue::allowed("main".to_string())),
            require_pull_request_reviews: Some(OverridableValue::allowed(true)),
            require_status_checks: Some(OverridableValue::allowed(true)),
            restrict_pushes: Some(OverridableValue::allowed(true)),
            required_approving_review_count: None,
            dismiss_stale_reviews: None,
            require_code_owner_reviews: None,
            required_status_checks_list: None,
            strict_required_status_checks: None,
            allow_force_pushes: None,
            allow_deletions: None,
            additional_protected_patterns: None,
        }),
        ..Default::default()
    };

    let template = create_test_template();

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_ok(), "Merge should succeed");
    let merged = result.unwrap();

    // Verify all branch protection fields are present
    assert!(merged.branch_protection.default_branch.is_some());
    assert!(merged
        .branch_protection
        .require_pull_request_reviews
        .is_some());
    assert!(merged.branch_protection.require_status_checks.is_some());
    assert!(merged.branch_protection.restrict_pushes.is_some());

    // Verify source tracking
    assert_eq!(
        merged.get_source("branch_protection.default_branch"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        merged.get_source("branch_protection.restrict_pushes"),
        Some(ConfigurationSource::Global)
    );
}

// ============================================================================
// Naming Rule Merging Tests  (replaces earlier label-merger tests)
// ============================================================================
//
// Note: `ConfigurationMerger::merge_configurations` does NOT populate
// `MergedConfiguration::labels`; that field is filled by
// `OrganizationSettingsManager` (steps 6.5/6.6) after the merge returns.
// Label behaviour is therefore tested in `organization_settings_manager_tests`.
// The following tests verify the closest analogous additive-collection
// behaviour that IS handled by the merger: `naming_rules`.

/// Naming rules from repository type and template are merged additively.
///
/// Both layers define a rule; after merging both must be present — no rule
/// is silently dropped by a higher-precedence level.
#[test]
fn test_naming_rules_merge_additively_from_repository_type_and_template() {
    use crate::settings::naming::RepositoryNamingRulesConfig;

    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();

    let repo_type = RepositoryTypeConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Repository type rule".to_string()),
            allowed_pattern: Some("^[a-z].*$".to_string()),
            ..Default::default()
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
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Template rule: must end with -service".to_string()),
            allowed_pattern: Some(".*-service$".to_string()),
            ..Default::default()
        }]),
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        labels: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);
    assert!(
        result.is_ok(),
        "Naming rule merging should succeed: {:?}",
        result.err()
    );

    let merged = result.unwrap();

    assert_eq!(
        merged.naming_rules.len(),
        2,
        "Naming rules from both repository type and template must be present; got {} rules",
        merged.naming_rules.len()
    );
    let descriptions: Vec<&str> = merged
        .naming_rules
        .iter()
        .filter_map(|r| r.description.as_deref())
        .collect();
    assert!(
        descriptions.contains(&"Repository type rule"),
        "Repository type naming rule must be present"
    );
    assert!(
        descriptions.contains(&"Template rule: must end with -service"),
        "Template naming rule must be present"
    );
}

/// When naming rules are present at three levels (global, repository type,
/// template) all three sets are concatenated additively.
#[test]
fn test_naming_rules_accumulated_from_three_levels() {
    use crate::settings::naming::RepositoryNamingRulesConfig;

    let merger = ConfigurationMerger::new();

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Global: lowercase only".to_string()),
            allowed_pattern: Some("^[a-z0-9-]+$".to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Type: must start with 'svc-'".to_string()),
            allowed_pattern: Some("^svc-.*$".to_string()),
            ..Default::default()
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
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Template: must end with '-api'".to_string()),
            allowed_pattern: Some(".*-api$".to_string()),
            ..Default::default()
        }]),
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        labels: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    };

    let result = merger.merge_configurations(&global, Some(&repo_type), None, &template);
    assert!(result.is_ok(), "Three-level naming rule merge must succeed");

    let merged = result.unwrap();
    assert_eq!(
        merged.naming_rules.len(),
        3,
        "All three naming rules must be present; got {}",
        merged.naming_rules.len()
    );
}

/// Verify that when no level provides labels the merged map is empty.
///
/// Labels in `MergedConfiguration` are populated by `OrganizationSettingsManager`,
/// not by `ConfigurationMerger`. The merger always returns an empty label map.
#[test]
fn test_no_labels_across_all_levels_yields_empty_map() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let template = create_test_template(); // has labels: None

    let result = merger.merge_configurations(&global, None, None, &template);
    assert!(result.is_ok(), "Merge should succeed");

    let merged = result.unwrap();
    assert!(
        merged.labels.is_empty(),
        "No labels provided means empty merged map"
    );
}

// ============================================================================
// `actions` (ActionSettings) merge wiring (#276/#277)
//
// Prior to this change, `ActionSettings` was defined on `GlobalDefaults` and
// `TeamConfig` but never wired into `ConfigurationMerger` at all -- these
// tests pin down the first-ever wiring of all 8 `ActionSettings` fields
// (the 5 pre-existing fields plus the 3 new ones: `default_workflow_permissions`,
// `can_approve_pull_request_reviews`, `access_level`) across the full
// Global -> RepositoryType -> Team -> Template hierarchy, mirroring
// `merge_repository_settings`'s field-by-field precedence-and-override
// pattern (see `merger_creation_capabilities_tests.rs` for the analogous
// precedent on `CreationCapabilitiesSettings`).
// ============================================================================

/// Minimal template with no settings, so it never overrides anything.
fn minimal_template_for_actions() -> NewTemplateConfig {
    NewTemplateConfig {
        template: TemplateMetadata {
            name: "t".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
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
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
        actions: None,
    }
}

fn template_with_actions(actions: ActionSettings) -> NewTemplateConfig {
    NewTemplateConfig {
        actions: Some(actions),
        ..minimal_template_for_actions()
    }
}

// ----------------------------------------------------------------------------
// Tier 1: Specification tests -- defaults and precedence
// ----------------------------------------------------------------------------

#[test]
fn test_merged_configuration_exposes_actions_field_default_when_absent_everywhere() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(merged.actions, ActionSettings::default());
}

#[test]
fn test_global_only_enabled_is_reflected_in_merged_configuration() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(merged.actions.enabled.map(|v| v.value), Some(true));
}

#[test]
fn test_repository_type_overrides_global_allowed_actions() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            allowed_actions: Some(OverridableValue::allowed("all".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let repo_type = RepositoryTypeConfig {
        actions: Some(ActionSettings {
            allowed_actions: Some(OverridableValue::allowed("selected".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(
            &global,
            Some(&repo_type),
            None,
            &minimal_template_for_actions(),
        )
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.allowed_actions.map(|v| v.value),
        Some("selected".to_string()),
        "repository-type level should win over global"
    );
}

#[test]
fn test_team_overrides_repository_type_github_owned_allowed() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let repo_type = RepositoryTypeConfig {
        actions: Some(ActionSettings {
            github_owned_allowed: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            github_owned_allowed: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(
            &global,
            Some(&repo_type),
            Some(&team),
            &minimal_template_for_actions(),
        )
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.github_owned_allowed.map(|v| v.value),
        Some(true),
        "team level should win over repository-type"
    );
}

#[test]
fn test_template_overrides_team_verified_allowed() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let team = TeamConfig {
        actions: Some(ActionSettings {
            verified_allowed: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        verified_allowed: Some(OverridableValue::allowed(true)),
        ..Default::default()
    });

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &template)
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.verified_allowed.map(|v| v.value),
        Some(true),
        "template level (highest precedence) should win over team"
    );
}

#[test]
fn test_global_only_default_workflow_permissions_is_reflected_in_merged_configuration() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            default_workflow_permissions: Some(OverridableValue::allowed("read".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.default_workflow_permissions.map(|v| v.value),
        Some("read".to_string())
    );
}

#[test]
fn test_team_overrides_global_can_approve_pull_request_reviews() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            can_approve_pull_request_reviews: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged
            .actions
            .can_approve_pull_request_reviews
            .map(|v| v.value),
        Some(true),
        "team level should win over global"
    );
}

#[test]
fn test_template_overrides_team_access_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let team = TeamConfig {
        actions: Some(ActionSettings {
            access_level: Some(OverridableValue::allowed("organization".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    });

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &template)
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.access_level.map(|v| v.value),
        Some("enterprise".to_string()),
        "template level (highest precedence) should win over team"
    );
}

#[test]
fn test_global_only_patterns_allowed_is_reflected_in_merged_configuration() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["actions/checkout@*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.patterns_allowed,
        Some(vec!["actions/checkout@*".to_string()])
    );
}

#[test]
fn test_team_overrides_repository_type_patterns_allowed_wholesale_replace() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();
    let repo_type = RepositoryTypeConfig {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["repo-type/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["team/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(
            &global,
            Some(&repo_type),
            Some(&team),
            &minimal_template_for_actions(),
        )
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.patterns_allowed,
        Some(vec!["team/*".to_string()]),
        "team's list must replace repository-type's list wholesale, not union"
    );
}

// ----------------------------------------------------------------------------
// Tier 2: Adversarial -- override-policy enforcement
// ----------------------------------------------------------------------------

#[test]
fn test_team_cannot_override_fixed_global_enabled() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result =
        merger.merge_configurations(&global, None, Some(&team), &minimal_template_for_actions());

    assert!(
        result.is_err(),
        "team must not override a fixed enabled policy"
    );
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(
                setting.contains("enabled"),
                "error should reference the enabled field, got: {setting}"
            );
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

#[test]
fn test_template_cannot_override_fixed_global_allowed_actions() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            allowed_actions: Some(OverridableValue::fixed("local_only".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        allowed_actions: Some(OverridableValue::allowed("all".to_string())),
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(setting.contains("allowed_actions"));
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

#[test]
fn test_repository_type_cannot_override_fixed_global_github_owned_allowed() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            github_owned_allowed: Some(OverridableValue::fixed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let repo_type = RepositoryTypeConfig {
        actions: Some(ActionSettings {
            github_owned_allowed: Some(OverridableValue::allowed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = merger.merge_configurations(
        &global,
        Some(&repo_type),
        None,
        &minimal_template_for_actions(),
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(setting.contains("github_owned_allowed"));
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

#[test]
fn test_team_cannot_override_fixed_global_default_workflow_permissions() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            default_workflow_permissions: Some(OverridableValue::fixed("read".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result =
        merger.merge_configurations(&global, None, Some(&team), &minimal_template_for_actions());

    assert!(
        result.is_err(),
        "team must not override a fixed default_workflow_permissions policy"
    );
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(setting.contains("default_workflow_permissions"));
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

#[test]
fn test_template_cannot_override_fixed_global_access_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            access_level: Some(OverridableValue::fixed("none".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, None, &template);

    assert!(
        result.is_err(),
        "template must not override a fixed access_level policy"
    );
    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert!(setting.contains("access_level"));
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

/// A fixed value re-asserted with the *same* value at a higher level is not
/// an override attempt and must succeed (mirrors `validate_override`'s
/// equality short-circuit used throughout the rest of the merger).
#[test]
fn test_reasserting_identical_fixed_action_value_does_not_error() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            verified_allowed: Some(OverridableValue::fixed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            verified_allowed: Some(OverridableValue::allowed(true)), // same value
            ..Default::default()
        }),
        ..Default::default()
    };

    let result =
        merger.merge_configurations(&global, None, Some(&team), &minimal_template_for_actions());
    assert!(
        result.is_ok(),
        "identical re-assertion must not error: {result:?}"
    );
}

/// `patterns_allowed` is a plain `Option<Vec<String>>` with no
/// `override_allowed` flag (per the interface contract), so there is no
/// policy to violate -- a higher-precedence level must always be able to
/// replace it with a *different* list, even when the lower level's
/// `ActionSettings` is otherwise built from `OverridableValue::fixed(..)`
/// policies for its other fields. Stub-killing: an implementation that
/// mistakenly threads `patterns_allowed` through `validate_override` (which
/// requires a base `OverridableValue` to compare against) would either fail
/// to compile or spuriously reject this.
#[test]
fn test_patterns_allowed_can_always_be_replaced_with_a_different_list() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["global-org/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        patterns_allowed: Some(vec!["template-org/*".to_string()]),
        ..Default::default()
    });

    let result = merger.merge_configurations(&global, None, None, &template);

    let merged = result.expect("patterns_allowed has no override policy to violate");
    assert_eq!(
        merged.actions.patterns_allowed,
        Some(vec!["template-org/*".to_string()])
    );
}

// ----------------------------------------------------------------------------
// Tier 2: Adversarial -- field independence
// ----------------------------------------------------------------------------

/// Stub-killing: setting one `ActionSettings` field at a higher level must
/// not perturb other fields inherited from a lower level -- rules out an
/// implementation that merges the whole struct wholesale (losing unset
/// fields) instead of field-by-field.
#[test]
fn test_action_fields_merge_independently_across_levels() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            github_owned_allowed: Some(OverridableValue::allowed(true)),
            verified_allowed: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(false)), // only override this one
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.enabled.map(|v| v.value),
        Some(false),
        "team override should apply to enabled"
    );
    assert_eq!(
        merged.actions.github_owned_allowed.map(|v| v.value),
        Some(true),
        "github_owned_allowed must be inherited from global, untouched by team's enabled override"
    );
    assert_eq!(
        merged.actions.verified_allowed.map(|v| v.value),
        Some(true),
        "verified_allowed must be inherited from global, untouched by team's enabled override"
    );
}

#[test]
fn test_patterns_allowed_override_does_not_perturb_other_action_fields_from_lower_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            patterns_allowed: Some(vec!["global-org/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["team-org/*".to_string()]), // only this
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged.actions.patterns_allowed,
        Some(vec!["team-org/*".to_string()]),
        "team override should apply to patterns_allowed"
    );
    assert_eq!(
        merged.actions.enabled.map(|v| v.value),
        Some(true),
        "enabled must be inherited from global, untouched by team's patterns_allowed override"
    );
}

#[test]
fn test_other_action_field_override_does_not_perturb_patterns_allowed_from_lower_level() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(false)),
            patterns_allowed: Some(vec!["global-org/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)), // only this
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, Some(&team), &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(merged.actions.enabled.map(|v| v.value), Some(true));
    assert_eq!(
        merged.actions.patterns_allowed,
        Some(vec!["global-org/*".to_string()]),
        "patterns_allowed must be inherited from global, untouched by team's enabled override"
    );
}

// ----------------------------------------------------------------------------
// Tier 2: Adversarial -- source tracking
// ----------------------------------------------------------------------------

/// Mutation-kill: source tracking must actually record an entry for every
/// non-`None` `ActionSettings` field at Global level -- a no-op
/// implementation would still pass every value-assertion test above (the
/// merged *value* is correct either way) but would silently drop the audit
/// trail used by `MergedConfiguration::get_source`.
#[test]
fn test_global_action_settings_are_source_tracked_for_all_eight_fields() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            allowed_actions: Some(OverridableValue::allowed("all".to_string())),
            github_owned_allowed: Some(OverridableValue::allowed(true)),
            verified_allowed: Some(OverridableValue::allowed(true)),
            patterns_allowed: Some(vec!["org/*".to_string()]),
            default_workflow_permissions: Some(OverridableValue::allowed("read".to_string())),
            can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
            access_level: Some(OverridableValue::allowed("organization".to_string())),
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    for field in [
        "actions.enabled",
        "actions.allowed_actions",
        "actions.github_owned_allowed",
        "actions.verified_allowed",
        "actions.patterns_allowed",
        "actions.default_workflow_permissions",
        "actions.can_approve_pull_request_reviews",
        "actions.access_level",
    ] {
        assert_eq!(
            merged.get_source(field),
            Some(ConfigurationSource::Global),
            "{field} should be tracked as Global source"
        );
    }
}

#[test]
fn test_source_tracking_for_actions_fields_set_at_different_levels() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let repo_type = RepositoryTypeConfig {
        actions: Some(ActionSettings {
            allowed_actions: Some(OverridableValue::allowed("selected".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            access_level: Some(OverridableValue::allowed("organization".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    };
    let template = template_with_actions(ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    });

    let merged = merger
        .merge_configurations(&global, Some(&repo_type), Some(&team), &template)
        .expect("merge should succeed");

    assert_eq!(
        merged.get_source("actions.enabled"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        merged.get_source("actions.allowed_actions"),
        Some(ConfigurationSource::RepositoryType)
    );
    assert_eq!(
        merged.get_source("actions.access_level"),
        Some(ConfigurationSource::Team)
    );
    assert_eq!(
        merged.get_source("actions.default_workflow_permissions"),
        Some(ConfigurationSource::Template)
    );
}

#[test]
fn test_patterns_allowed_source_is_tracked() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            patterns_allowed: Some(vec!["org/*".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert_eq!(
        merged.get_source("actions.patterns_allowed"),
        Some(ConfigurationSource::Global)
    );
}

#[test]
fn test_absent_actions_at_every_level_resolves_to_all_none_and_no_source_tracked() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults::default();

    let merged = merger
        .merge_configurations(&global, None, None, &minimal_template_for_actions())
        .expect("merge should succeed");

    assert!(merged.actions.enabled.is_none());
    assert!(merged.actions.allowed_actions.is_none());
    assert!(merged.actions.github_owned_allowed.is_none());
    assert!(merged.actions.verified_allowed.is_none());
    assert!(merged.actions.patterns_allowed.is_none());
    assert!(merged.actions.default_workflow_permissions.is_none());
    assert!(merged.actions.can_approve_pull_request_reviews.is_none());
    assert!(merged.actions.access_level.is_none());
    assert_eq!(merged.get_source("actions.enabled"), None);
}

/// Stub-killing: the error path must name the *actual* offending field, not
/// a generic/wrong one -- rules out a copy-paste bug where every
/// `ActionSettings` field's validation reports the same hardcoded field name.
#[test]
fn test_override_error_references_the_specific_actions_field_path() {
    let merger = ConfigurationMerger::new();
    let global = GlobalDefaults {
        actions: Some(ActionSettings {
            can_approve_pull_request_reviews: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let team = TeamConfig {
        actions: Some(ActionSettings {
            can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result =
        merger.merge_configurations(&global, None, Some(&team), &minimal_template_for_actions());

    match result.unwrap_err() {
        ConfigurationError::OverrideNotPermitted { setting, .. } => {
            assert_eq!(
                setting, "actions.can_approve_pull_request_reviews",
                "error must reference the exact field path"
            );
        }
        other => panic!("expected OverrideNotPermitted, got {other:?}"),
    }
}

// ----------------------------------------------------------------------------
// Tier 3: Property-based tests -- precedence and override-policy invariants
// ----------------------------------------------------------------------------

mod actions_proptests {
    use super::*;
    use proptest::prelude::*;

    fn global_with_can_approve(value: bool, override_allowed: bool) -> GlobalDefaults {
        GlobalDefaults {
            actions: Some(ActionSettings {
                can_approve_pull_request_reviews: Some(OverridableValue::new(
                    value,
                    override_allowed,
                )),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn team_with_can_approve(value: bool) -> TeamConfig {
        TeamConfig {
            actions: Some(ActionSettings {
                can_approve_pull_request_reviews: Some(OverridableValue::allowed(value)),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn access_level_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("none".to_string()),
            Just("organization".to_string()),
            Just("enterprise".to_string()),
        ]
    }

    fn patterns_strategy() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[a-z]{3,6}/\\*", 0..4)
    }

    proptest! {
        /// For any combination of a global `can_approve_pull_request_reviews`
        /// policy and a team-level attempt to set it: the merge succeeds
        /// whenever the global value allows override OR the team's value
        /// matches the global value exactly, and fails with
        /// `OverrideNotPermitted` in every other case.
        #[test]
        fn prop_can_approve_pull_request_reviews_override_enforcement_matches_policy(
            global_value in any::<bool>(),
            global_override_allowed in any::<bool>(),
            team_value in any::<bool>(),
        ) {
            let merger = ConfigurationMerger::new();
            let global = global_with_can_approve(global_value, global_override_allowed);
            let team = team_with_can_approve(team_value);

            let result = merger.merge_configurations(
                &global,
                None,
                Some(&team),
                &minimal_template_for_actions(),
            );

            let override_is_a_noop = global_value == team_value;
            if global_override_allowed || override_is_a_noop {
                let merged = result.expect("override should be permitted or a no-op");
                prop_assert_eq!(
                    merged.actions.can_approve_pull_request_reviews.map(|v| v.value),
                    Some(team_value)
                );
            } else {
                prop_assert!(result.is_err(), "non-matching override of a fixed policy must be rejected");
            }
        }

        /// Precedence invariant: whichever of {template, team, repository_type,
        /// global} is the highest-precedence level that actually sets
        /// `access_level` (with `override_allowed = true` at every lower
        /// level so no rejection occurs), that level's value is the one that
        /// appears in the merged configuration, for any subset of levels
        /// providing a value and any `access_level` string chosen.
        #[test]
        fn prop_access_level_highest_precedence_level_wins_for_any_subset_of_levels_present(
            global_opt in proptest::option::of(access_level_strategy()),
            repo_type_opt in proptest::option::of(access_level_strategy()),
            team_opt in proptest::option::of(access_level_strategy()),
            template_opt in proptest::option::of(access_level_strategy()),
        ) {
            let merger = ConfigurationMerger::new();

            let global = GlobalDefaults {
                actions: global_opt.clone().map(|v| ActionSettings {
                    access_level: Some(OverridableValue::allowed(v)),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let repo_type = repo_type_opt.clone().map(|v| RepositoryTypeConfig {
                actions: Some(ActionSettings {
                    access_level: Some(OverridableValue::allowed(v)),
                    ..Default::default()
                }),
                ..Default::default()
            });
            let team = team_opt.clone().map(|v| TeamConfig {
                actions: Some(ActionSettings {
                    access_level: Some(OverridableValue::allowed(v)),
                    ..Default::default()
                }),
                ..Default::default()
            });
            let template = NewTemplateConfig {
                actions: template_opt.clone().map(|v| ActionSettings {
                    access_level: Some(OverridableValue::allowed(v)),
                    ..Default::default()
                }),
                ..minimal_template_for_actions()
            };

            let result = merger.merge_configurations(&global, repo_type.as_ref(), team.as_ref(), &template);
            let merged = result.expect("all-overridable levels must never be rejected");

            let expected = template_opt.or(team_opt).or(repo_type_opt).or(global_opt);

            prop_assert_eq!(
                merged.actions.access_level.map(|v| v.value),
                expected
            );
        }

        /// `patterns_allowed` has no `override_allowed` policy to violate, so
        /// for any subset of levels providing a list, the merge must always
        /// succeed and the highest-precedence level's *entire* list wins
        /// (never unioned with any lower level's list).
        #[test]
        fn prop_patterns_allowed_highest_precedence_level_wins_for_any_subset_of_levels_present(
            global_opt in proptest::option::of(patterns_strategy()),
            team_opt in proptest::option::of(patterns_strategy()),
            template_opt in proptest::option::of(patterns_strategy()),
        ) {
            let merger = ConfigurationMerger::new();

            let global = GlobalDefaults {
                actions: global_opt.clone().map(|v| ActionSettings {
                    patterns_allowed: Some(v),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let team = team_opt.clone().map(|v| TeamConfig {
                actions: Some(ActionSettings {
                    patterns_allowed: Some(v),
                    ..Default::default()
                }),
                ..Default::default()
            });
            let template = NewTemplateConfig {
                actions: template_opt.clone().map(|v| ActionSettings {
                    patterns_allowed: Some(v),
                    ..Default::default()
                }),
                ..minimal_template_for_actions()
            };

            let result = merger.merge_configurations(&global, None, team.as_ref(), &template);
            let merged = result.expect("patterns_allowed has no override policy to violate");

            let expected = template_opt.or(team_opt).or(global_opt);

            prop_assert_eq!(merged.actions.patterns_allowed, expected);
        }
    }
}
