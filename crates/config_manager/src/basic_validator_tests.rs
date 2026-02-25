//! Tests for BasicConfigurationValidator.

use super::*;
use crate::{
    settings::{
        environment::EnvironmentProtectionRules, BranchProtectionSettings, EnvironmentConfig,
        GitHubAppConfig, PullRequestSettings, RepositorySettings, WebhookConfig,
    },
    OverridableValue,
};
use std::collections::HashMap;

// ============================================================================
// Validator Creation Tests
// ============================================================================

/// Verify validator can be created.
#[test]
fn test_validator_creation() {
    let validator = BasicConfigurationValidator::new();
    assert!(std::mem::size_of_val(&validator) == 0); // Zero-sized type
}

/// Verify default implementation works.
#[test]
fn test_validator_default() {
    let _validator = BasicConfigurationValidator;
}

// ============================================================================
// Repository Settings Validation Tests
// ============================================================================

/// Verify valid repository settings pass validation.
#[test]
fn test_validate_repository_settings_valid() {
    let validator = BasicConfigurationValidator::new();
    let settings = RepositorySettings {
        issues: Some(OverridableValue {
            value: true,
            override_allowed: true,
        }),
        wiki: Some(OverridableValue {
            value: false,
            override_allowed: true,
        }),
        projects: Some(OverridableValue {
            value: true,
            override_allowed: false,
        }),
        discussions: None,
        pages: None,
        security_advisories: None,
        vulnerability_reporting: None,
        auto_close_issues: None,
    };

    let errors = validator.validate_repository_settings(&settings);
    assert!(errors.is_empty());
}

// ============================================================================
// Pull Request Settings Validation Tests
// ============================================================================

/// Verify valid pull request settings pass validation.
#[test]
fn test_validate_pull_request_settings_valid() {
    let validator = BasicConfigurationValidator::new();
    let settings = PullRequestSettings {
        required_approving_review_count: Some(OverridableValue {
            value: 2,
            override_allowed: true,
        }),
        allow_merge_commit: Some(OverridableValue {
            value: true,
            override_allowed: true,
        }),
        allow_squash_merge: None,
        allow_rebase_merge: None,
        allow_auto_merge: None,
        delete_branch_on_merge: None,
        require_code_owner_reviews: None,
        require_conversation_resolution: None,
        merge_commit_title: None,
        merge_commit_message: None,
        squash_merge_commit_title: None,
        squash_merge_commit_message: None,
    };

    let errors = validator.validate_pull_request_settings(&settings);
    assert!(errors.is_empty());
}

/// Verify negative review count fails validation.
#[test]
fn test_validate_pull_request_settings_negative_review_count() {
    let validator = BasicConfigurationValidator::new();
    let settings = PullRequestSettings {
        required_approving_review_count: Some(OverridableValue {
            value: -1,
            override_allowed: true,
        }),
        allow_merge_commit: None,
        allow_squash_merge: None,
        allow_rebase_merge: None,
        allow_auto_merge: None,
        delete_branch_on_merge: None,
        require_code_owner_reviews: None,
        require_conversation_resolution: None,
        merge_commit_title: None,
        merge_commit_message: None,
        squash_merge_commit_title: None,
        squash_merge_commit_message: None,
    };

    let errors = validator.validate_pull_request_settings(&settings);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::InvalidValue);
    assert_eq!(
        errors[0].field_path,
        "pull_requests.required_approving_review_count"
    );
    assert!(errors[0].message.contains("cannot be negative"));
}

/// Verify zero review count is valid.
#[test]
fn test_validate_pull_request_settings_zero_review_count() {
    let validator = BasicConfigurationValidator::new();
    let settings = PullRequestSettings {
        required_approving_review_count: Some(OverridableValue {
            value: 0,
            override_allowed: true,
        }),
        allow_merge_commit: None,
        allow_squash_merge: None,
        allow_rebase_merge: None,
        allow_auto_merge: None,
        delete_branch_on_merge: None,
        require_code_owner_reviews: None,
        require_conversation_resolution: None,
        merge_commit_title: None,
        merge_commit_message: None,
        squash_merge_commit_title: None,
        squash_merge_commit_message: None,
    };

    let errors = validator.validate_pull_request_settings(&settings);
    assert!(errors.is_empty());
}

// ============================================================================
// Branch Protection Validation Tests
// ============================================================================

/// Verify valid branch protection passes validation.
#[test]
fn test_validate_branch_protection_valid() {
    let validator = BasicConfigurationValidator::new();
    let settings = BranchProtectionSettings {
        require_status_checks: Some(OverridableValue {
            value: true,
            override_allowed: true,
        }),
        required_status_checks_list: Some(vec!["test".to_string(), "lint".to_string()]),
        require_pull_request_reviews: None,
        required_approving_review_count: None,
        restrict_pushes: None,
        default_branch: None,
        allow_force_pushes: None,
        allow_deletions: None,
        dismiss_stale_reviews: None,
        require_code_owner_reviews: None,
        strict_required_status_checks: None,
        additional_protected_patterns: None,
    };

    let errors = validator.validate_branch_protection(&settings);
    assert!(errors.is_empty());
}

/// Verify empty required checks fails when status checks required.
#[test]
fn test_validate_branch_protection_empty_required_checks() {
    let validator = BasicConfigurationValidator::new();
    let settings = BranchProtectionSettings {
        require_status_checks: Some(OverridableValue {
            value: true,
            override_allowed: true,
        }),
        required_status_checks_list: Some(vec![]),
        require_pull_request_reviews: None,
        required_approving_review_count: None,
        restrict_pushes: None,
        default_branch: None,
        allow_force_pushes: None,
        allow_deletions: None,
        dismiss_stale_reviews: None,
        require_code_owner_reviews: None,
        strict_required_status_checks: None,
        additional_protected_patterns: None,
    };

    let errors = validator.validate_branch_protection(&settings);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        ValidationErrorType::BusinessRuleViolation
    );
    assert!(errors[0]
        .message
        .contains("Status checks list cannot be empty"));
}

/// Verify branch protection completeness validation.
#[test]
fn test_validate_branch_protection_completeness_missing_review_count() {
    let validator = BasicConfigurationValidator::new();
    let settings = BranchProtectionSettings {
        require_pull_request_reviews: Some(OverridableValue {
            value: true,
            override_allowed: true,
        }),
        required_approving_review_count: None,
        require_status_checks: None,
        restrict_pushes: None,
        default_branch: None,
        allow_force_pushes: None,
        allow_deletions: None,
        dismiss_stale_reviews: None,
        require_code_owner_reviews: None,
        required_status_checks_list: None,
        strict_required_status_checks: None,
        additional_protected_patterns: None,
    };

    let errors = validator.validate_branch_protection_completeness(&settings);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        ValidationErrorType::BusinessRuleViolation
    );
    assert!(errors[0]
        .message
        .contains("Required approving review count must be specified"));
}

// ============================================================================
// Webhook Validation Tests
// ============================================================================

/// Verify valid webhooks pass validation.
#[test]
fn test_validate_webhooks_valid() {
    let validator = BasicConfigurationValidator::new();
    let webhooks = vec![
        WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            content_type: "json".to_string(),
            events: vec!["push".to_string(), "pull_request".to_string()],
            active: true,
            secret: None,
        },
        WebhookConfig {
            url: "http://localhost:8080/hook".to_string(),
            content_type: "json".to_string(),
            events: vec!["issues".to_string()],
            active: true,
            secret: None,
        },
    ];

    let errors = validator.validate_webhooks(&webhooks);
    assert!(errors.is_empty());
}

/// Verify invalid webhook URL format fails validation.
#[test]
fn test_validate_webhooks_invalid_url_format() {
    let validator = BasicConfigurationValidator::new();
    let webhooks = vec![WebhookConfig {
        url: "ftp://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        events: vec!["push".to_string()],
        active: true,
        secret: None,
    }];

    let errors = validator.validate_webhooks(&webhooks);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::InvalidValue);
    assert_eq!(errors[0].field_path, "webhooks[0].url");
    assert!(errors[0].message.contains("Invalid webhook URL format"));
}

/// Verify empty events list fails validation.
#[test]
fn test_validate_webhooks_empty_events() {
    let validator = BasicConfigurationValidator::new();
    let webhooks = vec![WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        events: vec![],
        active: true,
        secret: None,
    }];

    let errors = validator.validate_webhooks(&webhooks);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::SchemaViolation);
    assert_eq!(errors[0].field_path, "webhooks[0].events");
    assert!(errors[0].message.contains("at least one event"));
}

/// Verify HTTP webhook generates warning.
#[test]
fn test_validate_webhook_urls_http_warning() {
    let validator = BasicConfigurationValidator::new();
    let webhooks = vec![WebhookConfig {
        url: "http://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        events: vec!["push".to_string()],
        active: true,
        secret: None,
    }];

    let warnings = validator.validate_webhook_urls(&webhooks);
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].field_path, "webhooks[0].url");
    assert!(warnings[0].message.contains("HTTP instead of HTTPS"));
}

// ============================================================================
// GitHub App Validation Tests
// ============================================================================

/// Verify valid GitHub apps pass validation.
#[test]
fn test_validate_github_apps_valid() {
    let validator = BasicConfigurationValidator::new();
    let mut permissions = HashMap::new();
    permissions.insert("issues".to_string(), "write".to_string());
    permissions.insert("pull_requests".to_string(), "read".to_string());

    let apps = vec![GitHubAppConfig {
        app_id: 12345,
        permissions,
    }];

    let errors = validator.validate_github_apps(&apps);
    assert!(errors.is_empty());
}

/// Verify zero app ID fails validation.
#[test]
fn test_validate_github_apps_zero_app_id() {
    let validator = BasicConfigurationValidator::new();
    let mut permissions = HashMap::new();
    permissions.insert("issues".to_string(), "write".to_string());

    let apps = vec![GitHubAppConfig {
        app_id: 0,
        permissions,
    }];

    let errors = validator.validate_github_apps(&apps);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::InvalidValue);
    assert_eq!(errors[0].field_path, "github_apps[0].app_id");
    assert!(errors[0].message.contains("must be a positive integer"));
}

/// Verify empty permissions fails validation.
#[test]
fn test_validate_github_apps_empty_permissions() {
    let validator = BasicConfigurationValidator::new();
    let apps = vec![GitHubAppConfig {
        app_id: 12345,
        permissions: HashMap::new(),
    }];

    let errors = validator.validate_github_apps(&apps);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::SchemaViolation);
    assert_eq!(errors[0].field_path, "github_apps[0].permissions");
    assert!(errors[0].message.contains("at least one permission"));
}

// ============================================================================
// Environment Validation Tests
// ============================================================================

/// Verify valid environments pass validation.
#[test]
fn test_validate_environments_valid() {
    let validator = BasicConfigurationValidator::new();
    let envs = vec![
        EnvironmentConfig {
            name: "production".to_string(),
            protection_rules: Some(EnvironmentProtectionRules {
                required_reviewers: Some(vec!["@maintainers".to_string()]),
                wait_timer: Some(300),
            }),
            deployment_branch_policy: None,
        },
        EnvironmentConfig {
            name: "staging".to_string(),
            protection_rules: None,
            deployment_branch_policy: None,
        },
    ];

    let errors = validator.validate_environments(&envs);
    assert!(errors.is_empty());
}

/// Verify empty environment name fails validation.
#[test]
fn test_validate_environments_empty_name() {
    let validator = BasicConfigurationValidator::new();
    let envs = vec![EnvironmentConfig {
        name: "".to_string(),
        protection_rules: None,
        deployment_branch_policy: None,
    }];

    let errors = validator.validate_environments(&envs);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        ValidationErrorType::RequiredFieldMissing
    );
    assert_eq!(errors[0].field_path, "environments[0].name");
    assert!(errors[0].message.contains("cannot be empty"));
}

/// Verify negative wait timer fails validation.
#[test]
fn test_validate_environments_negative_wait_timer() {
    let validator = BasicConfigurationValidator::new();
    let envs = vec![EnvironmentConfig {
        name: "production".to_string(),
        protection_rules: Some(EnvironmentProtectionRules {
            required_reviewers: Some(vec![]),
            wait_timer: Some(-60),
        }),
        deployment_branch_policy: None,
    }];

    let errors = validator.validate_environments(&envs);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].error_type, ValidationErrorType::InvalidValue);
    assert!(errors[0]
        .field_path
        .contains("environments[0].protection_rules.wait_timer"));
    assert!(errors[0].message.contains("cannot be negative"));
}

// ============================================================================
// Security Policy Validation Tests
// ============================================================================

/// Verify security advisories cannot be disabled.
#[test]
fn test_validate_security_policies_advisories_disabled() {
    let validator = BasicConfigurationValidator::new();
    let mut merged = MergedConfiguration::default();
    merged.repository.security_advisories = Some(OverridableValue {
        value: false,
        override_allowed: false,
    });

    let errors = validator.validate_security_policies(&merged);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        ValidationErrorType::BusinessRuleViolation
    );
    assert_eq!(errors[0].field_path, "repository.security_advisories");
    assert!(errors[0].message.contains("cannot be disabled"));
}

/// Verify vulnerability reporting cannot be disabled.
#[test]
fn test_validate_security_policies_vulnerability_reporting_disabled() {
    let validator = BasicConfigurationValidator::new();
    let mut merged = MergedConfiguration::default();
    merged.repository.vulnerability_reporting = Some(OverridableValue {
        value: false,
        override_allowed: false,
    });

    let errors = validator.validate_security_policies(&merged);
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].error_type,
        ValidationErrorType::BusinessRuleViolation
    );
    assert_eq!(errors[0].field_path, "repository.vulnerability_reporting");
    assert!(errors[0].message.contains("cannot be disabled"));
}

/// Verify security policies pass when enabled.
#[test]
fn test_validate_security_policies_valid() {
    let validator = BasicConfigurationValidator::new();
    let mut merged = MergedConfiguration::default();
    merged.repository.security_advisories = Some(OverridableValue {
        value: true,
        override_allowed: false,
    });
    merged.repository.vulnerability_reporting = Some(OverridableValue {
        value: true,
        override_allowed: false,
    });

    let errors = validator.validate_security_policies(&merged);
    assert!(errors.is_empty());
}

// ============================================================================
// Integration Tests (Trait Implementation)
// ============================================================================

/// Verify validate_global_defaults works end-to-end.
#[tokio::test]
async fn test_validate_global_defaults_integration() {
    let validator = BasicConfigurationValidator::new();
    let defaults = GlobalDefaults::default();

    let result = validator.validate_global_defaults(&defaults).await.unwrap();
    assert!(result.is_valid());
}

/// Verify validate_team_config works end-to-end.
#[tokio::test]
async fn test_validate_team_config_integration() {
    let validator = BasicConfigurationValidator::new();
    let team_config = TeamConfig::default();
    let global = GlobalDefaults::default();

    let result = validator
        .validate_team_config(&team_config, &global)
        .await
        .unwrap();
    assert!(result.is_valid());
}

/// Verify validate_repository_type_config works end-to-end.
#[tokio::test]
async fn test_validate_repository_type_config_integration() {
    let validator = BasicConfigurationValidator::new();
    let repo_type_config = RepositoryTypeConfig::default();
    let global = GlobalDefaults::default();

    let result = validator
        .validate_repository_type_config(&repo_type_config, &global)
        .await
        .unwrap();
    assert!(result.is_valid());
}

/// Verify validate_template_config works end-to-end.
#[tokio::test]
async fn test_validate_template_config_integration() {
    let validator = BasicConfigurationValidator::new();
    let template_config = NewTemplateConfig {
        template: crate::template_config::TemplateMetadata {
            name: "test".to_string(),
            description: "Test template".to_string(),
            author: "Test Author".to_string(),
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
    };

    let result = validator
        .validate_template_config(&template_config)
        .await
        .unwrap();
    assert!(result.is_valid());
}

/// Verify validate_merged_config catches multiple errors.
#[tokio::test]
async fn test_validate_merged_config_multiple_errors() {
    let validator = BasicConfigurationValidator::new();
    let mut merged = MergedConfiguration::default();

    // Add invalid pull request settings
    merged.pull_requests.required_approving_review_count = Some(OverridableValue {
        value: -1,
        override_allowed: true,
    });

    // Add disabled security advisory
    merged.repository.security_advisories = Some(OverridableValue {
        value: false,
        override_allowed: false,
    });

    // Add webhook with empty events
    merged.webhooks.push(WebhookConfig {
        url: "https://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        events: vec![],
        active: true,
        secret: None,
    });

    let result = validator.validate_merged_config(&merged).await.unwrap();
    assert!(!result.is_valid());
    assert!(result.errors.len() >= 3); // At least 3 errors
}

/// Verify validate_merged_config generates warnings.
#[tokio::test]
async fn test_validate_merged_config_with_warnings() {
    let validator = BasicConfigurationValidator::new();
    let mut merged = MergedConfiguration::default();

    // Add HTTP webhook (should generate warning)
    merged.webhooks.push(WebhookConfig {
        url: "http://example.com/webhook".to_string(),
        content_type: "json".to_string(),
        events: vec!["push".to_string()],
        active: true,
        secret: None,
    });

    let result = validator.validate_merged_config(&merged).await.unwrap();
    assert!(result.is_valid()); // Still valid, just warnings
    assert_eq!(result.warnings.len(), 1); // One warning for HTTP
}
