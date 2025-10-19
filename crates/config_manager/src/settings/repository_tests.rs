//! Tests for RepositorySettings

use super::*;

#[test]
fn test_default_creates_empty_settings() {
    let settings = RepositorySettings::default();
    assert!(settings.issues.is_none());
    assert!(settings.wiki.is_none());
    assert!(settings.projects.is_none());
}

#[test]
fn test_serialize_deserialize_round_trip() {
    let settings = RepositorySettings {
        issues: Some(OverridableValue::allowed(true)),
        wiki: Some(OverridableValue::fixed(false)),
        security_advisories: Some(OverridableValue::fixed(true)),
        ..Default::default()
    };

    let toml = toml::to_string(&settings).expect("Failed to serialize");
    let deserialized: RepositorySettings = toml::from_str(&toml).expect("Failed to deserialize");
    assert_eq!(settings, deserialized);
}

#[test]
fn test_all_fields_optional() {
    let toml = r#"
        issues = { value = true, override_allowed = true }
    "#;

    let settings: RepositorySettings = toml::from_str(toml).expect("Failed to deserialize");
    assert!(settings.issues.is_some());
    assert!(settings.wiki.is_none());
}
