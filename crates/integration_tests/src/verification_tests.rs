//! Tests for configuration verification helpers.

use super::*;
use std::collections::HashMap;

/// Test ConfigurationVerification::success creates valid success result
#[test]
fn test_configuration_verification_success() {
    let result = ConfigurationVerification::success();

    assert!(result.passed);
    assert!(result.settings_verified);
    assert!(result.custom_properties_verified);
    assert!(result.branch_protection_verified);
    assert!(result.labels_verified);
    assert!(result.failures.is_empty());
}

/// Test ConfigurationVerification::failure creates valid failure result
#[test]
fn test_configuration_verification_failure() {
    let reason = "Test failure reason".to_string();
    let result = ConfigurationVerification::failure(reason.clone());

    assert!(!result.passed);
    assert!(!result.settings_verified);
    assert!(!result.custom_properties_verified);
    assert!(!result.branch_protection_verified);
    assert!(!result.labels_verified);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(result.failures[0], reason);
}

/// Test ConfigurationVerification::add_failure correctly updates state
#[test]
fn test_configuration_verification_add_failure() {
    let mut result = ConfigurationVerification::success();
    assert!(result.passed);
    assert!(result.failures.is_empty());

    let failure1 = "First failure".to_string();
    result.add_failure(failure1.clone());

    assert!(!result.passed);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(result.failures[0], failure1);

    let failure2 = "Second failure".to_string();
    result.add_failure(failure2.clone());

    assert!(!result.passed);
    assert_eq!(result.failures.len(), 2);
    assert_eq!(result.failures[1], failure2);
}

/// Test ExpectedConfiguration can be constructed with all None values
#[test]
fn test_expected_configuration_empty() {
    let config = ExpectedConfiguration {
        repository_settings: None,
        custom_properties: None,
        branch_protection: None,
        labels: None,
    };

    assert!(config.repository_settings.is_none());
    assert!(config.custom_properties.is_none());
    assert!(config.branch_protection.is_none());
    assert!(config.labels.is_none());
}

/// Test ExpectedConfiguration with repository settings
#[test]
fn test_expected_configuration_with_repository_settings() {
    let settings = ExpectedRepositorySettings {
        has_issues: Some(true),
        has_wiki: Some(false),
        has_discussions: Some(true),
        has_projects: Some(false),
    };

    let config = ExpectedConfiguration {
        repository_settings: Some(settings.clone()),
        custom_properties: None,
        branch_protection: None,
        labels: None,
    };

    assert!(config.repository_settings.is_some());
    let repo_settings = config.repository_settings.unwrap();
    assert_eq!(repo_settings.has_issues, Some(true));
    assert_eq!(repo_settings.has_wiki, Some(false));
    assert_eq!(repo_settings.has_discussions, Some(true));
    assert_eq!(repo_settings.has_projects, Some(false));
}

/// Test ExpectedConfiguration with custom properties
#[test]
fn test_expected_configuration_with_custom_properties() {
    let mut props = HashMap::new();
    props.insert("team".to_string(), "platform".to_string());
    props.insert("repo_type".to_string(), "library".to_string());

    let config = ExpectedConfiguration {
        repository_settings: None,
        custom_properties: Some(props.clone()),
        branch_protection: None,
        labels: None,
    };

    assert!(config.custom_properties.is_some());
    let custom_props = config.custom_properties.unwrap();
    assert_eq!(custom_props.len(), 2);
    assert_eq!(custom_props.get("team"), Some(&"platform".to_string()));
    assert_eq!(custom_props.get("repo_type"), Some(&"library".to_string()));
}

/// Test ExpectedConfiguration with branch protection
#[test]
fn test_expected_configuration_with_branch_protection() {
    let protection = ExpectedBranchProtection {
        branch: "main".to_string(),
        required_approving_review_count: Some(2),
        require_code_owner_reviews: Some(true),
        dismiss_stale_reviews: Some(false),
    };

    let config = ExpectedConfiguration {
        repository_settings: None,
        custom_properties: None,
        branch_protection: Some(protection.clone()),
        labels: None,
    };

    assert!(config.branch_protection.is_some());
    let branch_protection = config.branch_protection.unwrap();
    assert_eq!(branch_protection.branch, "main");
    assert_eq!(branch_protection.required_approving_review_count, Some(2));
    assert_eq!(branch_protection.require_code_owner_reviews, Some(true));
    assert_eq!(branch_protection.dismiss_stale_reviews, Some(false));
}

/// Test ExpectedConfiguration with labels
#[test]
fn test_expected_configuration_with_labels() {
    let labels = vec![
        "bug".to_string(),
        "enhancement".to_string(),
        "documentation".to_string(),
    ];

    let config = ExpectedConfiguration {
        repository_settings: None,
        custom_properties: None,
        branch_protection: None,
        labels: Some(labels.clone()),
    };

    assert!(config.labels.is_some());
    let label_list = config.labels.unwrap();
    assert_eq!(label_list.len(), 3);
    assert_eq!(label_list[0], "bug");
    assert_eq!(label_list[1], "enhancement");
    assert_eq!(label_list[2], "documentation");
}

/// Test ExpectedConfiguration with all fields populated
#[test]
fn test_expected_configuration_complete() {
    let settings = ExpectedRepositorySettings {
        has_issues: Some(true),
        has_wiki: Some(false),
        has_discussions: Some(true),
        has_projects: Some(false),
    };

    let mut props = HashMap::new();
    props.insert("team".to_string(), "backend".to_string());

    let protection = ExpectedBranchProtection {
        branch: "main".to_string(),
        required_approving_review_count: Some(1),
        require_code_owner_reviews: Some(false),
        dismiss_stale_reviews: Some(true),
    };

    let labels = vec!["critical".to_string(), "security".to_string()];

    let config = ExpectedConfiguration {
        repository_settings: Some(settings),
        custom_properties: Some(props),
        branch_protection: Some(protection),
        labels: Some(labels),
    };

    assert!(config.repository_settings.is_some());
    assert!(config.custom_properties.is_some());
    assert!(config.branch_protection.is_some());
    assert!(config.labels.is_some());
}

// Note: Integration tests for verify_repository_settings, verify_custom_properties,
// verify_branch_protection, verify_labels, and load_expected_configuration will be
// added once the GitHub API methods are implemented and available for testing.
