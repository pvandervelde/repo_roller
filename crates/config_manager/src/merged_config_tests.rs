//! Tests for merged configuration.

use super::*;

#[test]
fn test_merged_configuration_new() {
    let config = MergedConfiguration::new();

    // All settings should start empty/default
    assert_eq!(config.labels.len(), 0);
    assert_eq!(config.webhooks.len(), 0);
    assert_eq!(config.custom_properties.len(), 0);
    assert_eq!(config.environments.len(), 0);
    assert_eq!(config.github_apps.len(), 0);
    assert_eq!(config.source_trace.field_count(), 0);
}

#[test]
fn test_merged_configuration_default() {
    let config = MergedConfiguration::default();

    assert_eq!(config.labels.len(), 0);
    assert_eq!(config.source_trace.field_count(), 0);
}

#[test]
fn test_record_source() {
    let mut config = MergedConfiguration::new();

    config.record_source("repository.issues", ConfigurationSource::Global);
    config.record_source(
        "pull_requests.required_approving_review_count",
        ConfigurationSource::Template,
    );

    assert_eq!(config.source_trace.field_count(), 2);
}

#[test]
fn test_get_source() {
    let mut config = MergedConfiguration::new();

    config.record_source("repository.issues", ConfigurationSource::Global);
    config.record_source("repository.wiki", ConfigurationSource::Team);

    assert_eq!(
        config.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        config.get_source("repository.wiki"),
        Some(ConfigurationSource::Team)
    );
    assert_eq!(config.get_source("repository.projects"), None);
}

#[test]
fn test_configuration_source_trace_new() {
    let trace = ConfigurationSourceTrace::new();

    assert_eq!(trace.field_count(), 0);
    assert_eq!(trace.configured_fields().len(), 0);
}

#[test]
fn test_configuration_source_trace_add_source() {
    let mut trace = ConfigurationSourceTrace::new();

    trace.add_source("field1", ConfigurationSource::Global);
    trace.add_source("field2", ConfigurationSource::Template);

    assert_eq!(trace.field_count(), 2);
    assert_eq!(
        trace.get_source("field1"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        trace.get_source("field2"),
        Some(ConfigurationSource::Template)
    );
}

#[test]
fn test_configuration_source_trace_overwrite() {
    let mut trace = ConfigurationSourceTrace::new();

    // First set from Global
    trace.add_source("repository.issues", ConfigurationSource::Global);
    assert_eq!(
        trace.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );

    // Then override from Template
    trace.add_source("repository.issues", ConfigurationSource::Template);
    assert_eq!(
        trace.get_source("repository.issues"),
        Some(ConfigurationSource::Template)
    );

    // Field count should still be 1 (not 2)
    assert_eq!(trace.field_count(), 1);
}

#[test]
fn test_configuration_source_trace_configured_fields() {
    let mut trace = ConfigurationSourceTrace::new();

    trace.add_source("field1", ConfigurationSource::Global);
    trace.add_source("field2", ConfigurationSource::Team);
    trace.add_source("field3", ConfigurationSource::Template);

    let fields = trace.configured_fields();
    assert_eq!(fields.len(), 3);
    assert!(fields.contains(&"field1"));
    assert!(fields.contains(&"field2"));
    assert!(fields.contains(&"field3"));
}

#[test]
fn test_configuration_source_display() {
    assert_eq!(ConfigurationSource::Global.to_string(), "Global");
    assert_eq!(
        ConfigurationSource::RepositoryType.to_string(),
        "RepositoryType"
    );
    assert_eq!(ConfigurationSource::Team.to_string(), "Team");
    assert_eq!(ConfigurationSource::Template.to_string(), "Template");
}

#[test]
fn test_configuration_source_equality() {
    assert_eq!(ConfigurationSource::Global, ConfigurationSource::Global);
    assert_ne!(ConfigurationSource::Global, ConfigurationSource::Template);
}

#[test]
fn test_configuration_source_copy() {
    let source1 = ConfigurationSource::Team;
    let source2 = source1; // Copy trait
    assert_eq!(source1, source2);
}

#[test]
fn test_merged_configuration_with_labels() {
    let mut config = MergedConfiguration::new();

    // Add labels from different sources
    config.labels.insert(
        "bug".to_string(),
        LabelConfig {
            name: "bug".to_string(),
            color: "d73a4a".to_string(),
            description: "Something isn't working".to_string(),
        },
    );
    config.record_source("labels.bug", ConfigurationSource::Global);

    config.labels.insert(
        "enhancement".to_string(),
        LabelConfig {
            name: "enhancement".to_string(),
            color: "a2eeef".to_string(),
            description: "New feature".to_string(),
        },
    );
    config.record_source("labels.enhancement", ConfigurationSource::Template);

    assert_eq!(config.labels.len(), 2);
    assert!(config.labels.contains_key("bug"));
    assert!(config.labels.contains_key("enhancement"));
    assert_eq!(
        config.get_source("labels.bug"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        config.get_source("labels.enhancement"),
        Some(ConfigurationSource::Template)
    );
}

#[test]
fn test_merged_configuration_with_webhooks() {
    let mut config = MergedConfiguration::new();

    config.webhooks.push(WebhookConfig {
        url: "https://example.com/webhook1".to_string(),
        content_type: "json".to_string(),
        secret: None,
        events: vec!["push".to_string()],
        active: true,
    });
    config.record_source("webhooks[0]", ConfigurationSource::Global);

    config.webhooks.push(WebhookConfig {
        url: "https://example.com/webhook2".to_string(),
        content_type: "json".to_string(),
        secret: None,
        events: vec!["pull_request".to_string()],
        active: true,
    });
    config.record_source("webhooks[1]", ConfigurationSource::Team);

    assert_eq!(config.webhooks.len(), 2);
}

#[test]
fn test_merged_configuration_with_custom_properties() {
    let mut config = MergedConfiguration::new();

    config.custom_properties.push(CustomProperty {
        property_name: "type".to_string(),
        value: crate::settings::custom_property::CustomPropertyValue::String("service".to_string()),
    });

    assert_eq!(config.custom_properties.len(), 1);
}

#[test]
fn test_merged_configuration_with_environments() {
    let mut config = MergedConfiguration::new();

    config.environments.push(EnvironmentConfig {
        name: "production".to_string(),
        protection_rules: None,
        deployment_branch_policy: None,
    });
    config.environments.push(EnvironmentConfig {
        name: "staging".to_string(),
        protection_rules: None,
        deployment_branch_policy: None,
    });

    assert_eq!(config.environments.len(), 2);
}

#[test]
fn test_merged_configuration_with_github_apps() {
    let mut config = MergedConfiguration::new();

    let mut permissions = HashMap::new();
    permissions.insert("contents".to_string(), "read".to_string());

    config.github_apps.push(GitHubAppConfig {
        app_id: 12345,
        permissions,
    });

    assert_eq!(config.github_apps.len(), 1);
}

#[test]
fn test_merged_configuration_clone() {
    let mut config = MergedConfiguration::new();
    config.record_source("test.field", ConfigurationSource::Global);

    let cloned = config.clone();
    assert_eq!(config, cloned);
    assert_eq!(cloned.source_trace.field_count(), 1);
}

#[test]
fn test_configuration_source_trace_clone() {
    let mut trace = ConfigurationSourceTrace::new();
    trace.add_source("field1", ConfigurationSource::Global);

    let cloned = trace.clone();
    assert_eq!(trace, cloned);
    assert_eq!(cloned.field_count(), 1);
}

#[test]
fn test_merged_configuration_debug_format() {
    let config = MergedConfiguration::new();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("MergedConfiguration"));
}

#[test]
fn test_configuration_source_debug_format() {
    let source = ConfigurationSource::Template;
    let debug_str = format!("{:?}", source);
    assert!(debug_str.contains("Template"));
}

#[test]
fn test_configuration_precedence_tracking() {
    let mut config = MergedConfiguration::new();

    // Simulate configuration merge with precedence
    // Global sets default
    config.record_source("repository.issues", ConfigurationSource::Global);

    // Team overrides
    config.record_source("repository.wiki", ConfigurationSource::Team);

    // Template has highest precedence
    config.record_source("repository.projects", ConfigurationSource::Template);

    // Verify sources
    assert_eq!(
        config.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        config.get_source("repository.wiki"),
        Some(ConfigurationSource::Team)
    );
    assert_eq!(
        config.get_source("repository.projects"),
        Some(ConfigurationSource::Template)
    );
}

#[test]
fn test_additive_collections_from_multiple_sources() {
    let mut config = MergedConfiguration::new();

    // Global provides some webhooks
    config.webhooks.push(WebhookConfig {
        url: "https://global.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        events: vec!["push".to_string()],
        active: true,
    });
    config.record_source("webhooks[0]", ConfigurationSource::Global);

    // Team adds more webhooks
    config.webhooks.push(WebhookConfig {
        url: "https://team.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        events: vec!["pull_request".to_string()],
        active: true,
    });
    config.record_source("webhooks[1]", ConfigurationSource::Team);

    // Template adds more webhooks
    config.webhooks.push(WebhookConfig {
        url: "https://template.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        events: vec!["release".to_string()],
        active: true,
    });
    config.record_source("webhooks[2]", ConfigurationSource::Template);

    // All webhooks should be present (additive)
    assert_eq!(config.webhooks.len(), 3);
}
