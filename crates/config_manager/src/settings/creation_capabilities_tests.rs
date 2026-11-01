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
    assert!(
        settings.allowed_source_owners.is_none(),
        "security-critical default: unset allow-list must deny every source owner"
    );
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
// Tier 1: Specification tests — `allowed_source_owners`
// (docs/spec/interfaces/fork-mirror-operations.md §2, "Allow-list of source
// owners")
// ============================================================================

#[test]
fn test_creation_capabilities_settings_allowed_source_owners_roundtrips_through_json() {
    let settings = CreationCapabilitiesSettings {
        allowed_source_owners: Some(OverridableValue::allowed(vec![
            "upstream-org".to_string(),
            "another-org".to_string(),
        ])),
        ..Default::default()
    };
    let json = serde_json::to_string(&settings).expect("serialize");
    let roundtripped: CreationCapabilitiesSettings =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        roundtripped.allowed_source_owners.unwrap().value,
        vec!["upstream-org".to_string(), "another-org".to_string()]
    );
}

#[test]
fn test_creation_capabilities_settings_allowed_source_owners_parses_explicit_toml_table() {
    let toml = r#"
        allowed_source_owners = { value = ["acme-corp"], override_allowed = false }
    "#;
    let settings: CreationCapabilitiesSettings = toml::from_str(toml).expect("parse");
    let value = settings.allowed_source_owners.expect("set");
    assert_eq!(value.value, vec!["acme-corp".to_string()]);
    assert!(
        !value.override_allowed,
        "explicit table form must preserve override_allowed=false"
    );
}

/// Simple TOML array format (`allowed_source_owners = ["a", "b"]`) must
/// auto-wrap with `override_allowed = true`, mirroring the simple-scalar
/// convention already proven above for the `bool`-typed fields. This is new
/// ground for `OverridableValue<T>`'s custom `Deserialize` impl: every other
/// field on this struct is `OverridableValue<bool>`, so this is the first
/// case that exercises the simple-format path for a sequence-typed `T`.
#[test]
fn test_creation_capabilities_settings_allowed_source_owners_parses_simple_toml_array() {
    let toml = r#"
        allowed_source_owners = ["acme-corp", "widgets-inc"]
    "#;
    let settings: CreationCapabilitiesSettings = toml::from_str(toml).expect("parse");
    let value = settings
        .allowed_source_owners
        .expect("allowed_source_owners set");
    assert_eq!(
        value.value,
        vec!["acme-corp".to_string(), "widgets-inc".to_string()]
    );
    assert!(
        value.override_allowed,
        "simple array form implies override_allowed=true"
    );
}

/// Security-critical distinction: an explicitly-empty list and an unset
/// field both resolve to "deny all" at enforcement time (per spec), but the
/// settings struct itself must still distinguish "nothing configured yet"
/// from "administrator explicitly locked this down to nobody" — otherwise a
/// higher-precedence level could not tell whether a lower level deliberately
/// set an empty policy or never touched the field at all.
#[test]
fn test_creation_capabilities_settings_allowed_source_owners_empty_list_is_distinct_from_unset() {
    let unset = CreationCapabilitiesSettings::default();
    let explicit_empty = CreationCapabilitiesSettings {
        allowed_source_owners: Some(OverridableValue::allowed(Vec::<String>::new())),
        ..Default::default()
    };
    assert!(unset.allowed_source_owners.is_none());
    assert!(explicit_empty.allowed_source_owners.is_some());
    assert_ne!(unset, explicit_empty);
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
        allowed_source_owners: None,
    };

    assert_eq!(settings.fork_enabled.unwrap().value, true);
    assert!(settings.source_history_mirror_enabled.is_none());
    assert_eq!(settings.source_only_mirror_enabled.unwrap().value, false);
    assert!(settings.allowed_source_owners.is_none());
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
        allowed_source_owners: Some(OverridableValue::allowed(vec!["acme".to_string()])),
    };
    let cloned = settings.clone();
    assert_eq!(settings, cloned);
}

/// Stub-killing: setting `allowed_source_owners` must not perturb any of the
/// three `*_enabled` flags, and vice versa — rules out an implementation
/// that conflates "capability enabled" with "has an allow-list" behind the
/// scenes (e.g. deriving one from the other instead of storing both).
#[test]
fn test_creation_capabilities_allowed_source_owners_is_independent_of_enabled_flags() {
    let settings = CreationCapabilitiesSettings {
        fork_enabled: Some(OverridableValue::allowed(true)),
        source_history_mirror_enabled: None,
        source_only_mirror_enabled: None,
        allowed_source_owners: None,
    };
    assert_eq!(settings.fork_enabled.unwrap().value, true);
    assert!(
        settings.allowed_source_owners.is_none(),
        "enabling a capability must not implicitly populate the allow-list"
    );
}
