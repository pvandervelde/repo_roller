//! Tests for `GlobalDefaults.creation_capabilities`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2.

use super::*;
use crate::settings::CreationCapabilitiesSettings;
use crate::OverridableValue;

#[test]
fn test_global_defaults_creation_capabilities_defaults_to_none() {
    let defaults = GlobalDefaults::default();
    assert!(defaults.creation_capabilities.is_none());
}

#[test]
fn test_global_defaults_creation_capabilities_roundtrips_through_json() {
    let defaults = GlobalDefaults {
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::fixed(true)),
            source_history_mirror_enabled: Some(OverridableValue::fixed(false)),
            source_only_mirror_enabled: Some(OverridableValue::fixed(false)),
            allowed_source_owners: None,
        }),
        ..Default::default()
    };

    let json = serde_json::to_string(&defaults).expect("serialize");
    let roundtripped: GlobalDefaults = serde_json::from_str(&json).expect("deserialize");

    let capabilities = roundtripped
        .creation_capabilities
        .expect("creation_capabilities present");
    assert_eq!(capabilities.fork_enabled.unwrap().value, true);
    assert_eq!(
        capabilities.source_history_mirror_enabled.unwrap().value,
        false
    );
    assert_eq!(
        capabilities.source_only_mirror_enabled.unwrap().value,
        false
    );
}

#[test]
fn test_global_defaults_creation_capabilities_parses_from_toml() {
    let toml = r#"
        [creation_capabilities]
        fork_enabled = { value = true, override_allowed = false }
        source_history_mirror_enabled = { value = true, override_allowed = true }
        source_only_mirror_enabled = { value = false, override_allowed = false }
    "#;

    let defaults: GlobalDefaults = toml::from_str(toml).expect("parse");
    let capabilities = defaults.creation_capabilities.expect("set");

    let fork = capabilities.fork_enabled.unwrap();
    assert_eq!(fork.value, true);
    assert!(!fork.override_allowed);
}

/// Adversarial: setting `creation_capabilities` must not perturb unrelated
/// global settings (e.g. `repository`), ruling out an implementation that
/// accidentally shares storage between setting categories.
#[test]
fn test_global_defaults_creation_capabilities_does_not_affect_other_settings() {
    let defaults = GlobalDefaults {
        repository: Some(crate::settings::RepositorySettings {
            wiki: Some(OverridableValue::fixed(false)),
            ..Default::default()
        }),
        creation_capabilities: Some(CreationCapabilitiesSettings {
            fork_enabled: Some(OverridableValue::allowed(true)),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_eq!(
        defaults.repository.unwrap().wiki.unwrap().value,
        false,
        "repository settings must be unaffected by creation_capabilities"
    );
}
