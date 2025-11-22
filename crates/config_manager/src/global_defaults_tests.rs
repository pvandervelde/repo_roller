//! Tests for GlobalDefaults

use super::*;
use crate::OverridableValue;

#[test]
fn test_default_creates_empty_config() {
    let defaults = GlobalDefaults::default();
    assert!(defaults.repository.is_none());
    assert!(defaults.pull_requests.is_none());
    assert!(defaults.webhooks.is_none());
}

#[test]
fn test_deserialize_from_toml() {
    let toml = r#"
        [repository]
        issues = { value = true, override_allowed = true }
        wiki = { value = false, override_allowed = false }

        [pull_requests]
        allow_merge_commit = { value = false, override_allowed = true }
        required_approving_review_count = { value = 1, override_allowed = true }
    "#;

    let defaults: GlobalDefaults = toml::from_str(toml).expect("Failed to deserialize");

    assert!(defaults.repository.is_some());
    let repo = defaults.repository.unwrap();
    assert!(repo.issues.as_ref().unwrap().value);
    assert!(!repo.wiki.as_ref().unwrap().value);
    assert!(!repo.wiki.as_ref().unwrap().override_allowed);

    assert!(defaults.pull_requests.is_some());
    let pr = defaults.pull_requests.unwrap();
    assert!(!pr.allow_merge_commit.as_ref().unwrap().value);
    assert_eq!(
        pr.required_approving_review_count.as_ref().unwrap().value,
        1
    );
}

#[test]
fn test_deserialize_with_webhooks() {
    let toml = r#"
        [[webhooks]]
        url = "https://example.com/webhook"
        content_type = "json"
        active = true
        events = ["push", "pull_request"]

        [[webhooks]]
        url = "https://example.com/webhook2"
        content_type = "form"
        active = false
        events = ["issues"]
    "#;

    let defaults: GlobalDefaults = toml::from_str(toml).expect("Failed to deserialize");

    assert!(defaults.webhooks.is_some());
    let webhooks = defaults.webhooks.unwrap();
    assert_eq!(webhooks.len(), 2);
    assert_eq!(webhooks[0].url, "https://example.com/webhook");
    assert_eq!(webhooks[0].events.len(), 2);
    assert!(!webhooks[1].active);
}

#[test]
fn test_deserialize_with_github_apps() {
    let toml = r#"
        [[github_apps]]
        app_id = 12345

        [github_apps.permissions]
        issues = "write"
        pull_requests = "read"
    "#;

    let defaults: GlobalDefaults = toml::from_str(toml).expect("Failed to deserialize");

    assert!(defaults.github_apps.is_some());
    let apps = defaults.github_apps.unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].app_id, 12345);
    assert_eq!(
        apps[0].permissions.get("issues"),
        Some(&"write".to_string())
    );
}

#[test]
fn test_serialize_round_trip() {
    let defaults = GlobalDefaults {
        repository: Some(RepositorySettings {
            issues: Some(OverridableValue::allowed(true)),
            wiki: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            allow_merge_commit: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let toml = toml::to_string(&defaults).expect("Failed to serialize");
    let deserialized: GlobalDefaults = toml::from_str(&toml).expect("Failed to deserialize");
    assert_eq!(defaults, deserialized);
}

#[test]
fn test_all_sections_optional() {
    let toml = r#"
        [repository]
        issues = { value = true, override_allowed = true }
    "#;

    let defaults: GlobalDefaults = toml::from_str(toml).expect("Failed to deserialize");
    assert!(defaults.repository.is_some());
    assert!(defaults.pull_requests.is_none());
    assert!(defaults.webhooks.is_none());
}

#[test]
fn test_empty_config_deserializes() {
    let toml = "";
    let defaults: GlobalDefaults = toml::from_str(toml).expect("Failed to deserialize");
    assert!(defaults.repository.is_none());
}
