//! Tests for `RepositoryTypeConfig.creation_capabilities`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::OverridableValue;

#[test]
fn test_repository_type_config_creation_capabilities_defaults_to_none() {
    let config = RepositoryTypeConfig::default();
    assert!(config.creation_capabilities.is_none());
}

#[test]
fn test_repository_type_config_creation_capabilities_roundtrips_through_json() {
    let config = RepositoryTypeConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_string(&config).expect("serialize");
    let roundtripped: RepositoryTypeConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        roundtripped
            .creation_capabilities
            .unwrap()
            .source_only_mirror_enabled
            .unwrap()
            .value,
        true
    );
}

#[test]
fn test_repository_type_config_creation_capabilities_parses_simple_toml() {
    let toml = r#"
        [creation_capabilities]
        fork_enabled = true
    "#;

    let config: RepositoryTypeConfig = toml::from_str(toml).expect("parse");
    let capabilities = config.creation_capabilities.expect("set");
    let fork = capabilities.fork_enabled.expect("fork_enabled set");
    assert_eq!(fork.value, true);
    assert!(fork.override_allowed);
}
