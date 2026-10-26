//! Tests for `TemplateConfig.creation_capabilities`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::OverridableValue;

fn minimal_metadata() -> TemplateMetadata {
    TemplateMetadata {
        name: "t".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        tags: vec![],
    }
}

fn template_with_capabilities(
    creation_capabilities: Option<CreationCapabilitiesSettings>,
) -> TemplateConfig {
    TemplateConfig {
        template: minimal_metadata(),
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities,
    }
}

#[test]
fn test_template_config_creation_capabilities_defaults_to_none_when_absent_in_toml() {
    let toml = r#"
        [template]
        name = "t"
        description = "d"
        author = "a"
        tags = []
    "#;
    let config: TemplateConfig = toml::from_str(toml).expect("parse");
    assert!(config.creation_capabilities.is_none());
}

#[test]
fn test_template_config_creation_capabilities_roundtrips_through_json() {
    let config = template_with_capabilities(Some(CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    }));

    let json = serde_json::to_string(&config).expect("serialize");
    let roundtripped: TemplateConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        roundtripped
            .creation_capabilities
            .unwrap()
            .fork_enabled
            .unwrap()
            .value,
        true
    );
}
