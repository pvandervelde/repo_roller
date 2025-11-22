//! Comprehensive integration tests for configuration system (Task 2.7).
//!
//! These tests verify cross-module integration, complex scenarios, and
//! system-wide validation that complements the unit tests in each module.

use crate::*;

/// Test that all configuration types can be created and used together.
#[test]
fn test_configuration_types_integration() {
    // Create instances of all config types
    let global = GlobalDefaults::default();
    let team = TeamConfig::default();
    let repo_type = RepositoryTypeConfig::default();
    let merged = MergedConfiguration::new();

    // Verify they all exist and are usable
    assert_eq!(global, GlobalDefaults::default());
    assert_eq!(team, TeamConfig::default());
    assert_eq!(repo_type, RepositoryTypeConfig::default());
    assert_eq!(merged.source_trace.field_count(), 0);
}

/// Test MergedConfiguration simulates the full configuration hierarchy.
#[test]
fn test_configuration_hierarchy_integration() {
    let mut merged = MergedConfiguration::new();

    // Simulate merging from Global
    merged.repository.issues = Some(OverridableValue::allowed(true));
    merged.record_source("repository.issues", ConfigurationSource::Global);

    // Simulate overriding from RepositoryType
    merged.repository.wiki = Some(OverridableValue::allowed(false));
    merged.record_source("repository.wiki", ConfigurationSource::RepositoryType);

    // Simulate adding from Team
    merged.webhooks.push(settings::WebhookConfig {
        url: "https://team.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        active: true,
        events: vec!["push".to_string()],
    });
    merged.record_source("webhooks[0]", ConfigurationSource::Team);

    // Simulate Template taking precedence
    merged.repository.projects = Some(OverridableValue::fixed(true));
    merged.record_source("repository.projects", ConfigurationSource::Template);

    // Verify the hierarchy is tracked correctly
    assert_eq!(
        merged.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(
        merged.get_source("repository.wiki"),
        Some(ConfigurationSource::RepositoryType)
    );
    assert_eq!(
        merged.get_source("webhooks[0]"),
        Some(ConfigurationSource::Team)
    );
    assert_eq!(
        merged.get_source("repository.projects"),
        Some(ConfigurationSource::Template)
    );

    // Verify configuration count
    assert_eq!(merged.source_trace.field_count(), 4);
}

/// Test that OverridableValue policies work across different config levels.
#[test]
fn test_override_policies_integration() {
    // Fixed policy cannot be overridden
    let fixed_value = OverridableValue::fixed(true);
    assert!(!fixed_value.can_override());
    assert!(*fixed_value.get());

    // Allowed policy can be overridden
    let allowed_value = OverridableValue::allowed(false);
    assert!(allowed_value.can_override());
    assert!(!(*allowed_value.get()));

    // Test with different types
    let fixed_int = OverridableValue::fixed(42);
    assert!(!fixed_int.can_override());

    let allowed_string = OverridableValue::allowed("test".to_string());
    assert!(allowed_string.can_override());
}

/// Test label merging behavior (HashMap - last writer wins).
#[test]
fn test_label_merging_integration() {
    let mut merged = MergedConfiguration::new();

    // Global provides initial labels
    merged.labels.insert(
        "bug".to_string(),
        settings::LabelConfig {
            name: "bug".to_string(),
            color: "d73a4a".to_string(),
            description: "Bug from global".to_string(),
        },
    );
    merged.record_source("labels.bug", ConfigurationSource::Global);

    // Template overrides the bug label
    merged.labels.insert(
        "bug".to_string(),
        settings::LabelConfig {
            name: "bug".to_string(),
            color: "ff0000".to_string(),
            description: "Bug from template".to_string(),
        },
    );
    merged.record_source("labels.bug", ConfigurationSource::Template);

    // Team adds a new label
    merged.labels.insert(
        "enhancement".to_string(),
        settings::LabelConfig {
            name: "enhancement".to_string(),
            color: "a2eeef".to_string(),
            description: "Enhancement".to_string(),
        },
    );
    merged.record_source("labels.enhancement", ConfigurationSource::Team);

    // Verify: 2 labels total, bug from Template, enhancement from Team
    assert_eq!(merged.labels.len(), 2);
    assert_eq!(merged.labels["bug"].color, "ff0000");
    assert_eq!(merged.labels["bug"].description, "Bug from template");
    assert_eq!(
        merged.get_source("labels.bug"),
        Some(ConfigurationSource::Template)
    );
    assert_eq!(
        merged.get_source("labels.enhancement"),
        Some(ConfigurationSource::Team)
    );
}

/// Test additive collection merging (Vec - all items kept).
#[test]
fn test_additive_collection_merging_integration() {
    let mut merged = MergedConfiguration::new();

    // Global adds webhooks
    merged.webhooks.push(settings::WebhookConfig {
        url: "https://global.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        active: true,
        events: vec!["push".to_string()],
    });
    merged.record_source("webhooks[0]", ConfigurationSource::Global);

    // Team adds more webhooks
    merged.webhooks.push(settings::WebhookConfig {
        url: "https://team.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        active: true,
        events: vec!["pull_request".to_string()],
    });
    merged.record_source("webhooks[1]", ConfigurationSource::Team);

    // Template adds even more webhooks
    merged.webhooks.push(settings::WebhookConfig {
        url: "https://template.example.com/webhook".to_string(),
        content_type: "json".to_string(),
        secret: None,
        active: true,
        events: vec!["release".to_string()],
    });
    merged.record_source("webhooks[2]", ConfigurationSource::Template);

    // Verify: all 3 webhooks present (additive)
    assert_eq!(merged.webhooks.len(), 3);
    assert_eq!(merged.webhooks[0].url, "https://global.example.com/webhook");
    assert_eq!(merged.webhooks[1].url, "https://team.example.com/webhook");
    assert_eq!(
        merged.webhooks[2].url,
        "https://template.example.com/webhook"
    );
}

/// Test ConfigurationSource Display trait and equality.
#[test]
fn test_configuration_source_properties() {
    // Display trait
    assert_eq!(ConfigurationSource::Global.to_string(), "Global");
    assert_eq!(
        ConfigurationSource::RepositoryType.to_string(),
        "RepositoryType"
    );
    assert_eq!(ConfigurationSource::Team.to_string(), "Team");
    assert_eq!(ConfigurationSource::Template.to_string(), "Template");

    // Equality and Copy
    let source1 = ConfigurationSource::Global;
    let source2 = source1; // Copy trait
    assert_eq!(source1, source2);

    // Inequality
    assert_ne!(ConfigurationSource::Global, ConfigurationSource::Template);
}

/// Test that source trace can be cloned and compared.
#[test]
fn test_source_trace_clone_and_equality() {
    let mut trace1 = ConfigurationSourceTrace::new();
    trace1.add_source("field1", ConfigurationSource::Global);
    trace1.add_source("field2", ConfigurationSource::Template);

    let trace2 = trace1.clone();

    assert_eq!(trace1, trace2);
    assert_eq!(trace1.field_count(), 2);
    assert_eq!(trace2.field_count(), 2);
}

/// Test that MergedConfiguration can be cloned.
#[test]
fn test_merged_configuration_clone() {
    let mut config = MergedConfiguration::new();
    config.repository.issues = Some(OverridableValue::allowed(true));
    config.record_source("repository.issues", ConfigurationSource::Global);

    let cloned = config.clone();

    assert_eq!(config, cloned);
    assert_eq!(
        cloned.repository.issues,
        Some(OverridableValue::allowed(true))
    );
    assert_eq!(
        cloned.get_source("repository.issues"),
        Some(ConfigurationSource::Global)
    );
}

/// Test complex repository settings configuration.
#[test]
fn test_complex_repository_settings() {
    let mut merged = MergedConfiguration::new();

    // Configure multiple repository settings
    merged.repository.issues = Some(OverridableValue::allowed(true));
    merged.repository.wiki = Some(OverridableValue::fixed(false));
    merged.repository.projects = Some(OverridableValue::allowed(false));
    merged.repository.discussions = Some(OverridableValue::allowed(true));
    merged.repository.security_advisories = Some(OverridableValue::fixed(true));

    // Record sources
    merged.record_source("repository.issues", ConfigurationSource::Global);
    merged.record_source("repository.wiki", ConfigurationSource::Template);
    merged.record_source("repository.projects", ConfigurationSource::Team);
    merged.record_source("repository.discussions", ConfigurationSource::Global);
    merged.record_source(
        "repository.security_advisories",
        ConfigurationSource::Template,
    );

    // Verify all settings
    assert_eq!(
        merged.repository.issues,
        Some(OverridableValue::allowed(true))
    );
    assert_eq!(merged.repository.wiki, Some(OverridableValue::fixed(false)));
    assert_eq!(
        merged.repository.projects,
        Some(OverridableValue::allowed(false))
    );

    // Verify override policies
    assert!(merged.repository.issues.as_ref().unwrap().can_override());
    assert!(!merged.repository.wiki.as_ref().unwrap().can_override());
}

/// Test complex pull request settings configuration.
#[test]
fn test_complex_pull_request_settings() {
    let mut merged = MergedConfiguration::new();

    // Configure PR settings
    merged.pull_requests.allow_squash_merge = Some(OverridableValue::fixed(true));
    merged.pull_requests.allow_merge_commit = Some(OverridableValue::fixed(false));
    merged.pull_requests.allow_rebase_merge = Some(OverridableValue::allowed(true));
    merged.pull_requests.required_approving_review_count = Some(OverridableValue::fixed(2));
    merged.pull_requests.delete_branch_on_merge = Some(OverridableValue::allowed(true));

    // Record sources
    merged.record_source(
        "pull_requests.allow_squash_merge",
        ConfigurationSource::Template,
    );
    merged.record_source(
        "pull_requests.allow_merge_commit",
        ConfigurationSource::Template,
    );
    merged.record_source(
        "pull_requests.required_approving_review_count",
        ConfigurationSource::Global,
    );

    // Verify settings
    assert!(!merged
        .pull_requests
        .allow_squash_merge
        .as_ref()
        .unwrap()
        .can_override());
    assert!(merged
        .pull_requests
        .allow_rebase_merge
        .as_ref()
        .unwrap()
        .can_override());
}

/// Test RepositoryTypePolicy enum.
#[test]
fn test_repository_type_policy() {
    use template_config::RepositoryTypePolicy;

    let fixed = RepositoryTypePolicy::Fixed;
    let preferable = RepositoryTypePolicy::Preferable;

    assert_eq!(fixed, RepositoryTypePolicy::Fixed);
    assert_eq!(preferable, RepositoryTypePolicy::Preferable);
    assert_ne!(fixed, preferable);
}

/// Test that empty/default configurations are valid.
#[test]
fn test_default_configurations_are_valid() {
    // All defaults should be valid and equal to themselves
    let global = GlobalDefaults::default();
    let team = TeamConfig::default();
    let repo_type = RepositoryTypeConfig::default();
    let merged = MergedConfiguration::new();

    assert_eq!(global, GlobalDefaults::default());
    assert_eq!(team, TeamConfig::default());
    assert_eq!(repo_type, RepositoryTypeConfig::default());
    assert_eq!(merged, MergedConfiguration::new());
}

/// Test that all settings types implement Debug.
#[test]
fn test_debug_implementations() {
    let config = MergedConfiguration::new();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("MergedConfiguration"));

    let source = ConfigurationSource::Template;
    let debug_str = format!("{:?}", source);
    assert!(debug_str.contains("Template"));

    let global = GlobalDefaults::default();
    let debug_str = format!("{:?}", global);
    assert!(debug_str.contains("GlobalDefaults"));
}

/// Test ConfigurationSourceTrace behavior.
#[test]
fn test_configuration_source_trace_operations() {
    let mut trace = ConfigurationSourceTrace::new();

    // Initially empty
    assert_eq!(trace.field_count(), 0);
    assert_eq!(trace.configured_fields().len(), 0);

    // Add sources
    trace.add_source("field1", ConfigurationSource::Global);
    trace.add_source("field2", ConfigurationSource::Team);
    trace.add_source("field3", ConfigurationSource::Template);

    // Verify count
    assert_eq!(trace.field_count(), 3);

    // Verify configured fields
    let fields = trace.configured_fields();
    assert_eq!(fields.len(), 3);
    assert!(fields.contains(&"field1"));
    assert!(fields.contains(&"field2"));
    assert!(fields.contains(&"field3"));

    // Verify get_source
    assert_eq!(
        trace.get_source("field1"),
        Some(ConfigurationSource::Global)
    );
    assert_eq!(trace.get_source("field2"), Some(ConfigurationSource::Team));
    assert_eq!(
        trace.get_source("field3"),
        Some(ConfigurationSource::Template)
    );
    assert_eq!(trace.get_source("nonexistent"), None);

    // Overwrite a source
    trace.add_source("field1", ConfigurationSource::Template);
    assert_eq!(
        trace.get_source("field1"),
        Some(ConfigurationSource::Template)
    );
    assert_eq!(trace.field_count(), 3); // Count should stay the same
}

/// Test that all configuration types have proper serialization support.
#[test]
fn test_serialization_roundtrip_simple() {
    // GlobalDefaults
    let global = GlobalDefaults::default();
    let toml = toml::to_string(&global).expect("Failed to serialize GlobalDefaults");
    let back: GlobalDefaults = toml::from_str(&toml).expect("Failed to deserialize GlobalDefaults");
    assert_eq!(global, back);

    // TeamConfig
    let team = TeamConfig::default();
    let toml = toml::to_string(&team).expect("Failed to serialize TeamConfig");
    let back: TeamConfig = toml::from_str(&toml).expect("Failed to deserialize TeamConfig");
    assert_eq!(team, back);

    // RepositoryTypeConfig
    let repo_type = RepositoryTypeConfig::default();
    let toml = toml::to_string(&repo_type).expect("Failed to serialize RepositoryTypeConfig");
    let back: RepositoryTypeConfig =
        toml::from_str(&toml).expect("Failed to deserialize RepositoryTypeConfig");
    assert_eq!(repo_type, back);
}
