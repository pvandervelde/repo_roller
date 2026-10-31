//! Tests for `TeamConfig.creation_capabilities`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::OverridableValue;

#[test]
fn test_team_config_creation_capabilities_defaults_to_none() {
    let config = TeamConfig::default();
    assert!(config.creation_capabilities.is_none());
}

#[test]
fn test_team_config_creation_capabilities_roundtrips_through_json() {
    let config = TeamConfig {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            source_history_mirror_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let json = serde_json::to_string(&config).expect("serialize");
    let roundtripped: TeamConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        roundtripped
            .creation_capabilities
            .unwrap()
            .source_history_mirror_enabled
            .unwrap()
            .value,
        true
    );
}

#[test]
fn test_team_config_creation_capabilities_parses_simple_toml() {
    let toml = r#"
        [creation_capabilities]
        source_only_mirror_enabled = false
    "#;

    let config: TeamConfig = toml::from_str(toml).expect("parse");
    let capabilities = config.creation_capabilities.expect("set");
    let value = capabilities
        .source_only_mirror_enabled
        .expect("source_only_mirror_enabled set");
    assert_eq!(value.value, false);
    assert!(value.override_allowed);
}
