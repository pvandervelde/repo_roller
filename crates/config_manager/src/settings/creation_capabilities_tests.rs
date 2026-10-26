//! Tests for `CreationCapabilitiesSettings`.
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2 and RCF-005.

use super::*;
use crate::OverridableValue;

// ============================================================================
// Tier 1: Specification tests
// ============================================================================

#[test]
fn test_default_creation_capabilities_settings_has_no_fields_set() {
    let settings = CreationCapabilitiesSettings::default();
    assert!(settings.fork_enabled.is_none());
    assert!(settings.source_history_mirror_enabled.is_none());
    assert!(settings.source_only_mirror_enabled.is_none());
}

#[test]
fn test_creation_capabilities_settings_fork_enabled_roundtrips_through_json() {
    let settings = CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };
    let json = serde_json::to_string(&settings).expect("serialize");
    let roundtripped: CreationCapabilitiesSettings =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped.fork_enabled.unwrap().value, true);
}

#[test]
fn test_creation_capabilities_settings_source_history_mirror_enabled_roundtrips() {
    let settings = CreationCapabilitiesSettings {
        source_history_mirror_enabled: Some(OverridableValue::fixed(false)),
        ..Default::default()
    };
    let json = serde_json::to_string(&settings).expect("serialize");
    let roundtripped: CreationCapabilitiesSettings =
        serde_json::from_str(&json).expect("deserialize");
    let value = roundtripped.source_history_mirror_enabled.unwrap();
    assert_eq!(value.value, false);
    assert!(!value.override_allowed);
}

#[test]
fn test_creation_capabilities_settings_source_only_mirror_enabled_roundtrips() {
    let settings = CreationCapabilitiesSettings {
        source_only_mirror_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };
    let json = serde_json::to_string(&settings).expect("serialize");
    let roundtripped: CreationCapabilitiesSettings =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtripped.source_only_mirror_enabled.unwrap().value, true);
}

/// Simple TOML scalar format (`fork_enabled = true`) must auto-wrap with
/// `override_allowed = true`, mirroring every other `OverridableValue<T>`
/// field used in Team/RepositoryType/Template-level configuration (see
/// `OverridableValue`'s custom `Deserialize` impl).
#[test]
fn test_creation_capabilities_settings_parses_simple_toml_scalar() {
    let toml = r#"
        fork_enabled = true
        source_history_mirror_enabled = false
    "#;
    let settings: CreationCapabilitiesSettings = toml::from_str(toml).expect("parse");

    let fork = settings.fork_enabled.expect("fork_enabled set");
    assert_eq!(fork.value, true);
    assert!(
        fork.override_allowed,
        "simple scalar form implies override_allowed=true"
    );

    let history_mirror = settings
        .source_history_mirror_enabled
        .expect("source_history_mirror_enabled set");
    assert_eq!(history_mirror.value, false);
    assert!(history_mirror.override_allowed);
}

#[test]
fn test_creation_capabilities_settings_parses_explicit_toml_table() {
    let toml = r#"
        source_only_mirror_enabled = { value = true, override_allowed = false }
    "#;
    let settings: CreationCapabilitiesSettings = toml::from_str(toml).expect("parse");
    let value = settings.source_only_mirror_enabled.expect("set");
    assert_eq!(value.value, true);
    assert!(!value.override_allowed);
}

// ============================================================================
// Tier 2: Adversarial — field independence and equality
// ============================================================================

/// Stub-killing: a buggy implementation that shares one underlying flag for
/// all three capabilities (e.g. a single `enabled: bool` behind the scenes)
/// would fail this independence check.
#[test]
fn test_creation_capabilities_fields_are_independent() {
    let settings = CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        source_history_mirror_enabled: None,
        source_only_mirror_enabled: Some(OverridableValue::allowed(false)),
    };

    assert_eq!(settings.fork_enabled.unwrap().value, true);
    assert!(settings.source_history_mirror_enabled.is_none());
    assert_eq!(settings.source_only_mirror_enabled.unwrap().value, false);
}

#[test]
fn test_creation_capabilities_settings_equality_distinguishes_each_field() {
    let base = CreationCapabilitiesSettings::default();
    let fork_only = CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };
    let history_mirror_only = CreationCapabilitiesSettings {
        source_history_mirror_enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    assert_ne!(base, fork_only);
    assert_ne!(fork_only, history_mirror_only);
}

#[test]
fn test_creation_capabilities_settings_is_cloneable() {
    let settings = CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::fixed(true)),
        source_history_mirror_enabled: Some(OverridableValue::allowed(false)),
        source_only_mirror_enabled: None,
    };
    let cloned = settings.clone();
    assert_eq!(settings, cloned);
}
