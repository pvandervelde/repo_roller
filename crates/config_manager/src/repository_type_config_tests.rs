//! Tests for repository type configuration.

use super::*;

#[test]
fn test_default_creates_empty_config() {
    let config = RepositoryTypeConfig::default();
    assert!(config.repository.is_none());
    assert!(config.pull_requests.is_none());
    assert!(config.branch_protection.is_none());
    assert!(config.labels.is_none());
    assert!(config.webhooks.is_none());
    assert!(config.custom_properties.is_none());
    assert!(config.environments.is_none());
    assert!(config.github_apps.is_none());
}

#[test]
fn test_all_fields_optional() {
    // Empty config should parse successfully
    let toml = r#""#;
    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse empty config");
    assert!(config.repository.is_none());
}

#[test]
fn test_deserialize_repository_settings() {
    let toml = r#"
        [repository]
        discussions = false
        projects = true
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository.is_some());

    let repo = config.repository.unwrap();
    assert!(repo.discussions.is_some());
    assert!(!repo.discussions.as_ref().unwrap().value);
    assert!(repo.projects.is_some());
    assert!(repo.projects.as_ref().unwrap().value);
}

#[test]
fn test_deserialize_pull_request_settings() {
    let toml = r#"
        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.pull_requests.is_some());

    let pr = config.pull_requests.unwrap();
    assert!(pr.required_approving_review_count.is_some());
    assert_eq!(
        pr.required_approving_review_count.as_ref().unwrap().value,
        2
    );
}

#[test]
fn test_deserialize_branch_protection() {
    let toml = r#"
        [branch_protection]
        require_pull_request_reviews = true
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.branch_protection.is_some());

    let bp = config.branch_protection.unwrap();
    assert!(bp.require_pull_request_reviews.is_some());
    assert!(bp.require_pull_request_reviews.as_ref().unwrap().value);
}

#[test]
fn test_deserialize_labels() {
    let toml = r#"
        [[labels]]
        name = "bug"
        color = "d73a4a"
        description = "Something isn't working"

        [[labels]]
        name = "enhancement"
        color = "a2eeef"
        description = "New feature or request"
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.labels.is_some());

    let labels = config.labels.unwrap();
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0].name, "bug");
    assert_eq!(labels[0].color, "d73a4a");
    assert_eq!(labels[1].name, "enhancement");
    assert_eq!(labels[1].color, "a2eeef");
}

#[test]
fn test_deserialize_webhooks() {
    let toml = r#"
        [[webhooks]]
        url = "https://library.example.com/webhook"
        content_type = "json"
        events = ["push", "release"]
        active = true
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.webhooks.is_some());

    let webhooks = config.webhooks.unwrap();
    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0].url, "https://library.example.com/webhook");
}

#[test]
fn test_deserialize_custom_properties() {
    let toml = r#"
        [[custom_properties]]
        property_name = "type"
        value = "library"
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.custom_properties.is_some());

    let props = config.custom_properties.unwrap();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].property_name, "type");
    // Value is CustomPropertyValue::String
    match &props[0].value {
        crate::settings::custom_property::CustomPropertyValue::String(s) => {
            assert_eq!(s, "library");
        }
        _ => panic!("Expected String value"),
    }
}

#[test]
fn test_deserialize_environments() {
    let toml = r#"
        [[environments]]
        name = "production"
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.environments.is_some());

    let envs = config.environments.unwrap();
    assert_eq!(envs.len(), 1);
    assert_eq!(envs[0].name, "production");
}

#[test]
fn test_deserialize_github_apps() {
    let toml = r#"
        [[github_apps]]
        app_id = 54321
        permissions = { contents = "read", packages = "write" }
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.github_apps.is_some());

    let apps = config.github_apps.unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].app_id, 54321);
}

#[test]
fn test_deserialize_complete_repository_type_config() {
    let toml = r#"
        [repository]
        has_wiki = false
        allow_squash_merge = true

        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true

        [branch_protection]
        require_pull_request_reviews = true

        [[labels]]
        name = "breaking-change"
        color = "d73a4a"
        description = "Breaking API change"

        [[webhooks]]
        url = "https://library.example.com/webhook"
        content_type = "json"
        events = ["push", "release"]
        active = true

        [[custom_properties]]
        property_name = "type"
        value = "library"

        [[environments]]
        name = "production"

        [[github_apps]]
        app_id = 54321
        permissions = { contents = "read" }
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");

    // Verify all sections are present
    assert!(config.repository.is_some());
    assert!(config.pull_requests.is_some());
    assert!(config.branch_protection.is_some());
    assert!(config.labels.is_some());
    assert!(config.webhooks.is_some());
    assert!(config.custom_properties.is_some());
    assert!(config.environments.is_some());
    assert!(config.github_apps.is_some());

    // Verify counts for additive collections
    assert_eq!(config.labels.as_ref().unwrap().len(), 1);
    assert_eq!(config.webhooks.as_ref().unwrap().len(), 1);
    assert_eq!(config.custom_properties.as_ref().unwrap().len(), 1);
    assert_eq!(config.environments.as_ref().unwrap().len(), 1);
    assert_eq!(config.github_apps.as_ref().unwrap().len(), 1);
}

#[test]
fn test_partial_config_with_only_labels() {
    // Test that repository types can specify only additive collections
    let toml = r#"
        [[labels]]
        name = "documentation"
        color = "0052cc"
        description = "Documentation updates"
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository.is_none());
    assert!(config.pull_requests.is_none());
    assert!(config.labels.is_some());
}

#[test]
fn test_serialize_round_trip() {
    let config = RepositoryTypeConfig {
        labels: Some(vec![LabelConfig {
            name: "bug".to_string(),
            color: "d73a4a".to_string(),
            description: "Something isn't working".to_string(),
        }]),
        ..Default::default()
    };

    let toml = toml::to_string(&config).expect("Failed to serialize");
    let parsed: RepositoryTypeConfig = toml::from_str(&toml).expect("Failed to parse");

    assert_eq!(config, parsed);
}

#[test]
fn test_clone_creates_independent_copy() {
    let config = RepositoryTypeConfig {
        labels: Some(vec![LabelConfig {
            name: "test".to_string(),
            color: "ffffff".to_string(),
            description: "Test label".to_string(),
        }]),
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_debug_format() {
    let config = RepositoryTypeConfig {
        labels: Some(vec![LabelConfig {
            name: "test".to_string(),
            color: "ffffff".to_string(),
            description: "Test label".to_string(),
        }]),
        ..Default::default()
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("RepositoryTypeConfig"));
    assert!(debug_str.contains("labels"));
}

#[test]
fn test_values_auto_wrapped_with_override_allowed() {
    // Verify that simple values are automatically wrapped with override_allowed = true
    let toml = r#"
        [repository]
        discussions = false
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
    let repo = config.repository.as_ref().unwrap();
    let discussions = repo.discussions.as_ref().unwrap();

    assert!(!discussions.value);
    assert!(discussions.override_allowed); // Auto-wrapped
}
