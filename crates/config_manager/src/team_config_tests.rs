//! Tests for TeamConfig

use super::*;

#[test]
fn test_default_creates_empty_config() {
    let config = TeamConfig::default();
    assert!(config.repository.is_none());
    assert!(config.pull_requests.is_none());
    assert!(config.branch_protection.is_none());
    assert!(config.actions.is_none());
    assert!(config.push.is_none());
    assert!(config.webhooks.is_none());
    assert!(config.custom_properties.is_none());
    assert!(config.environments.is_none());
    assert!(config.github_apps.is_none());
}

#[test]
fn test_all_fields_optional() {
    // Empty TOML should deserialize successfully
    let toml = "";
    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse empty config");
    assert!(config.repository.is_none());
    assert!(config.pull_requests.is_none());
}

#[test]
fn test_deserialize_repository_settings() {
    let toml = r#"
        [repository]
        discussions = false
        projects = true
        wiki = false
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository.is_some());

    let repo = config.repository.unwrap();
    assert!(repo.discussions.is_some());
    assert!(!repo.discussions.as_ref().unwrap().value);
    assert!(repo.discussions.as_ref().unwrap().override_allowed);

    assert!(repo.projects.is_some());
    assert!(repo.projects.as_ref().unwrap().value);
}

#[test]
fn test_deserialize_pull_request_settings() {
    let toml = r#"
        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true
        allow_auto_merge = true
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
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
        required_approving_review_count = 2
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.branch_protection.is_some());
}

#[test]
fn test_deserialize_actions_settings() {
    let toml = r#"
        [actions]
        enabled = true
        allowed_actions = "selected"
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.actions.is_some());
}

#[test]
fn test_deserialize_push_settings() {
    let toml = r#"
        [push]
        max_branches_per_push = 10
        max_tags_per_push = 5
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.push.is_some());
}

#[test]
fn test_deserialize_webhooks() {
    let toml = r#"
        [[webhooks]]
        url = "https://team.example.com/webhook"
        content_type = "json"
        events = ["push", "pull_request"]
        active = true

        [[webhooks]]
        url = "https://team.example.com/webhook2"
        content_type = "form"
        events = ["issues"]
        active = true
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.webhooks.is_some());

    let webhooks = config.webhooks.unwrap();
    assert_eq!(webhooks.len(), 2);
    assert_eq!(webhooks[0].url, "https://team.example.com/webhook");
    assert_eq!(webhooks[0].content_type, "json");
    assert_eq!(webhooks[1].url, "https://team.example.com/webhook2");
}

#[test]
fn test_deserialize_custom_properties() {
    let toml = r#"
        [[custom_properties]]
        property_name = "team-label"
        value = "backend"

        [[custom_properties]]
        property_name = "priority"
        value = "high"
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.custom_properties.is_some());

    let props = config.custom_properties.unwrap();
    assert_eq!(props.len(), 2);
    assert_eq!(props[0].property_name, "team-label");
}

#[test]
fn test_deserialize_environments() {
    let toml = r#"
        [[environments]]
        name = "staging"
        wait_timer = 300
        reviewers = ["@backend-team"]

        [[environments]]
        name = "production"
        wait_timer = 600
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.environments.is_some());

    let envs = config.environments.unwrap();
    assert_eq!(envs.len(), 2);
    assert_eq!(envs[0].name, "staging");
    assert_eq!(envs[1].name, "production");
}

#[test]
fn test_deserialize_github_apps() {
    let toml = r#"
        [[github_apps]]
        app_id = 11111
        permissions = { contents = "read", issues = "write" }

        [[github_apps]]
        app_id = 22222
        permissions = { pull_requests = "write" }
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.github_apps.is_some());

    let apps = config.github_apps.unwrap();
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0].app_id, 11111);
    assert_eq!(apps[1].app_id, 22222);
}

#[test]
fn test_deserialize_complete_team_config() {
    // Test with all sections present
    let toml = r#"
        [repository]
        discussions = false
        projects = true
        issues = true

        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true

        [branch_protection]
        require_pull_request_reviews = true

        [actions]
        enabled = true

        [push]
        max_branches_per_push = 5

        [[webhooks]]
        url = "https://team.example.com/webhook"
        content_type = "json"
        events = ["push"]
        active = true

        [[custom_properties]]
        property_name = "team"
        value = "backend"

        [[environments]]
        name = "staging"

        [[github_apps]]
        app_id = 12345
        permissions = { contents = "read" }
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");

    // Verify all sections are present
    assert!(config.repository.is_some());
    assert!(config.pull_requests.is_some());
    assert!(config.branch_protection.is_some());
    assert!(config.actions.is_some());
    assert!(config.push.is_some());
    assert!(config.webhooks.is_some());
    assert!(config.custom_properties.is_some());
    assert!(config.environments.is_some());
    assert!(config.github_apps.is_some());
}

#[test]
fn test_serialize_round_trip() {
    let config = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(crate::OverridableValue::new(false, true)),
            projects: Some(crate::OverridableValue::new(true, true)),
            ..Default::default()
        }),
        pull_requests: Some(PullRequestSettings {
            required_approving_review_count: Some(crate::OverridableValue::new(2, true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let toml = toml::to_string(&config).expect("Failed to serialize");
    let deserialized: TeamConfig = toml::from_str(&toml).expect("Failed to deserialize");

    assert_eq!(config, deserialized);
}

#[test]
fn test_clone_creates_independent_copy() {
    let config = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(crate::OverridableValue::new(false, true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_debug_format() {
    let config = TeamConfig {
        repository: Some(RepositorySettings {
            discussions: Some(crate::OverridableValue::new(false, true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("TeamConfig"));
    assert!(debug_str.contains("repository"));
}

#[test]
fn test_partial_config_with_only_webhooks() {
    // Test that teams can specify only additive collections
    let toml = r#"
        [[webhooks]]
        url = "https://team.example.com/webhook"
        content_type = "json"
        events = ["push"]
        active = true
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository.is_none());
    assert!(config.pull_requests.is_none());
    assert!(config.webhooks.is_some());
}

#[test]
fn test_values_auto_wrapped_with_override_allowed() {
    // Verify that simple values are automatically wrapped with override_allowed = true
    let toml = r#"
        [repository]
        discussions = false
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
    let repo = config.repository.unwrap();
    let discussions = repo.discussions.unwrap();

    assert!(!discussions.value);
    assert!(
        discussions.override_allowed,
        "Team config values should auto-wrap with override_allowed = true"
    );
}
