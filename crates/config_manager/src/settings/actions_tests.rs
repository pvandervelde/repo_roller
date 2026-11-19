//! Tests for ActionSettings
//!
//! Covers the 5 pre-existing fields plus the 3 fields added for #276/#277
//! (`default_workflow_permissions`, `can_approve_pull_request_reviews`,
//! `access_level`). `ActionSettings` itself does no enum validation (matches
//! the existing unvalidated `allowed_actions: Option<OverridableValue<String>>`
//! field) -- these tests check field presence/shape/round-tripping, not value
//! validation.
use super::*;

// ============================================================================
// Tier 1: Specification tests -- field presence and defaults
// ============================================================================

#[test]
fn test_default_creates_empty() {
    let settings = ActionSettings::default();
    assert!(settings.enabled.is_none());
}

/// Stub-killing: a `Default` impl that only zeroes some fields (e.g. forgets
/// one of the 3 new fields) would still pass the single-field check above.
#[test]
fn test_default_all_eight_fields_are_none() {
    let settings = ActionSettings::default();

    assert!(settings.enabled.is_none());
    assert!(settings.allowed_actions.is_none());
    assert!(settings.github_owned_allowed.is_none());
    assert!(settings.verified_allowed.is_none());
    assert!(settings.patterns_allowed.is_none());
    assert!(
        settings.default_workflow_permissions.is_none(),
        "default_workflow_permissions must default to None"
    );
    assert!(
        settings.can_approve_pull_request_reviews.is_none(),
        "can_approve_pull_request_reviews must default to None"
    );
    assert!(
        settings.access_level.is_none(),
        "access_level must default to None"
    );
}

#[test]
fn test_default_workflow_permissions_holds_read_value() {
    let settings = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("read".to_string())),
        ..Default::default()
    };

    assert_eq!(settings.default_workflow_permissions.unwrap().value, "read");
}

#[test]
fn test_default_workflow_permissions_holds_write_value() {
    let settings = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    };

    assert_eq!(
        settings.default_workflow_permissions.unwrap().value,
        "write"
    );
}

#[test]
fn test_can_approve_pull_request_reviews_holds_true() {
    let settings = ActionSettings {
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    assert_eq!(
        settings.can_approve_pull_request_reviews.unwrap().value,
        true
    );
}

#[test]
fn test_can_approve_pull_request_reviews_holds_false() {
    let settings = ActionSettings {
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(false)),
        ..Default::default()
    };

    assert_eq!(
        settings.can_approve_pull_request_reviews.unwrap().value,
        false
    );
}

#[test]
fn test_access_level_holds_none_value() {
    let settings = ActionSettings {
        access_level: Some(OverridableValue::allowed("none".to_string())),
        ..Default::default()
    };

    assert_eq!(settings.access_level.unwrap().value, "none");
}

#[test]
fn test_access_level_holds_organization_value() {
    let settings = ActionSettings {
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    assert_eq!(settings.access_level.unwrap().value, "organization");
}

#[test]
fn test_access_level_holds_enterprise_value() {
    let settings = ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    assert_eq!(settings.access_level.unwrap().value, "enterprise");
}

// ============================================================================
// Tier 2: Adversarial -- field independence and serde round-tripping
// ============================================================================

/// Stub-killing: an implementation that aliases a new field onto an existing
/// one (e.g. reuses `allowed_actions`'s storage for `access_level`) would
/// fail this independence check.
#[test]
fn test_setting_new_fields_does_not_disturb_pre_existing_fields() {
    let settings = ActionSettings {
        enabled: Some(OverridableValue::allowed(true)),
        allowed_actions: Some(OverridableValue::allowed("selected".to_string())),
        github_owned_allowed: Some(OverridableValue::allowed(true)),
        verified_allowed: Some(OverridableValue::allowed(false)),
        patterns_allowed: Some(vec!["actions/checkout@*".to_string()]),
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        access_level: Some(OverridableValue::allowed("organization".to_string())),
    };

    assert_eq!(settings.enabled.as_ref().unwrap().value, true);
    assert_eq!(settings.allowed_actions.as_ref().unwrap().value, "selected");
    assert_eq!(settings.github_owned_allowed.as_ref().unwrap().value, true);
    assert_eq!(settings.verified_allowed.as_ref().unwrap().value, false);
    assert_eq!(
        settings.patterns_allowed.as_ref().unwrap(),
        &vec!["actions/checkout@*".to_string()]
    );
    assert_eq!(
        settings
            .default_workflow_permissions
            .as_ref()
            .unwrap()
            .value,
        "write"
    );
    assert_eq!(
        settings
            .can_approve_pull_request_reviews
            .as_ref()
            .unwrap()
            .value,
        true
    );
    assert_eq!(
        settings.access_level.as_ref().unwrap().value,
        "organization"
    );
}

/// Stub-killing: setting only a new field must leave all pre-existing fields
/// at their default (`None`) -- rules out an implementation that forces some
/// unrelated default onto the legacy fields when a new field is touched.
#[test]
fn test_setting_only_access_level_leaves_pre_existing_fields_none() {
    let settings = ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    assert!(settings.enabled.is_none());
    assert!(settings.allowed_actions.is_none());
    assert!(settings.github_owned_allowed.is_none());
    assert!(settings.verified_allowed.is_none());
    assert!(settings.patterns_allowed.is_none());
    assert!(settings.default_workflow_permissions.is_none());
    assert!(settings.can_approve_pull_request_reviews.is_none());
}

#[test]
fn test_serde_json_roundtrip_with_all_eight_fields_set() {
    let settings = ActionSettings {
        enabled: Some(OverridableValue::fixed(true)),
        allowed_actions: Some(OverridableValue::allowed("all".to_string())),
        github_owned_allowed: Some(OverridableValue::allowed(true)),
        verified_allowed: Some(OverridableValue::allowed(true)),
        patterns_allowed: Some(vec!["org/*".to_string()]),
        default_workflow_permissions: Some(OverridableValue::fixed("read".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(false)),
        access_level: Some(OverridableValue::allowed("none".to_string())),
    };

    let json = serde_json::to_string(&settings).expect("serialize");
    let roundtripped: ActionSettings = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(settings, roundtripped);
}

/// Stub-killing: ensures the 3 new fields are actually wired into
/// `Serialize`/`Deserialize` (not silently dropped) even when the 5
/// pre-existing fields are absent.
#[test]
fn test_serde_json_roundtrip_with_only_new_fields_set() {
    let settings = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::fixed(true)),
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    let json = serde_json::to_string(&settings).expect("serialize");
    assert!(
        json.contains("default_workflow_permissions"),
        "serialized JSON must contain default_workflow_permissions, got: {json}"
    );
    assert!(
        json.contains("can_approve_pull_request_reviews"),
        "serialized JSON must contain can_approve_pull_request_reviews, got: {json}"
    );
    assert!(
        json.contains("access_level"),
        "serialized JSON must contain access_level, got: {json}"
    );

    let roundtripped: ActionSettings = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(settings, roundtripped);
}

#[test]
fn test_toml_simple_value_auto_wraps_default_workflow_permissions() {
    let toml = r#"
        default_workflow_permissions = "read"
    "#;

    let settings: ActionSettings = toml::from_str(toml).expect("parse");
    let value = settings
        .default_workflow_permissions
        .expect("default_workflow_permissions set");

    assert_eq!(value.value, "read");
    assert!(
        value.override_allowed,
        "simple TOML format must auto-wrap with override_allowed = true"
    );
}

#[test]
fn test_toml_simple_value_auto_wraps_can_approve_pull_request_reviews() {
    let toml = r#"
        can_approve_pull_request_reviews = true
    "#;

    let settings: ActionSettings = toml::from_str(toml).expect("parse");
    let value = settings
        .can_approve_pull_request_reviews
        .expect("can_approve_pull_request_reviews set");

    assert_eq!(value.value, true);
    assert!(value.override_allowed);
}

#[test]
fn test_toml_simple_value_auto_wraps_access_level() {
    let toml = r#"
        access_level = "organization"
    "#;

    let settings: ActionSettings = toml::from_str(toml).expect("parse");
    let value = settings.access_level.expect("access_level set");

    assert_eq!(value.value, "organization");
    assert!(value.override_allowed);
}

#[test]
fn test_toml_explicit_format_for_access_level_preserves_override_allowed_false() {
    let toml = r#"
        access_level = { value = "enterprise", override_allowed = false }
    "#;

    let settings: ActionSettings = toml::from_str(toml).expect("parse");
    let value = settings.access_level.expect("access_level set");

    assert_eq!(value.value, "enterprise");
    assert!(
        !value.override_allowed,
        "explicit format must preserve override_allowed = false"
    );
}

#[test]
fn test_toml_explicit_format_for_can_approve_pull_request_reviews_preserves_override_allowed_false()
{
    let toml = r#"
        can_approve_pull_request_reviews = { value = true, override_allowed = false }
    "#;

    let settings: ActionSettings = toml::from_str(toml).expect("parse");
    let value = settings
        .can_approve_pull_request_reviews
        .expect("can_approve_pull_request_reviews set");

    assert_eq!(value.value, true);
    assert!(!value.override_allowed);
}

#[test]
fn test_clone_preserves_new_fields() {
    let settings = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    let cloned = settings.clone();
    assert_eq!(settings, cloned);
}

/// Stub-killing: `PartialEq` must actually distinguish settings that differ
/// only in one of the 3 new fields -- rules out a derive that somehow
/// excludes them (e.g. a hand-written impl that forgot a field).
#[test]
fn test_partial_eq_distinguishes_settings_differing_only_in_access_level() {
    let a = ActionSettings {
        access_level: Some(OverridableValue::allowed("none".to_string())),
        ..Default::default()
    };
    let b = ActionSettings {
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    assert_ne!(a, b);
}
