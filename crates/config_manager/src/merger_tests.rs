//! Tests for configuration merging engine.

use super::*;
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
        variables: None,
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
        variables: None,
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
        variables: None,
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
        variables: None,
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
        variables: None,
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
