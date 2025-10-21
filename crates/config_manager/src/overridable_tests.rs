//! Tests for OverridableValue<T>

use super::*;

#[test]
fn test_new_creates_overridable_value() {
    let value = OverridableValue::new(42, true);
    assert_eq!(value.value, 42);
    assert_eq!(value.override_allowed, true);
}

#[test]
fn test_new_creates_fixed_value() {
    let value = OverridableValue::new(100, false);
    assert_eq!(value.value, 100);
    assert_eq!(value.override_allowed, false);
}

#[test]
fn test_allowed_constructor() {
    let value = OverridableValue::allowed("test");
    assert_eq!(value.value, "test");
    assert!(value.can_override());
}

#[test]
fn test_fixed_constructor() {
    let value = OverridableValue::fixed("policy");
    assert_eq!(value.value, "policy");
    assert!(!value.can_override());
}

#[test]
fn test_can_override_returns_true_when_allowed() {
    let value = OverridableValue::new(true, true);
    assert!(value.can_override());
}

#[test]
fn test_can_override_returns_false_when_not_allowed() {
    let value = OverridableValue::new(true, false);
    assert!(!value.can_override());
}

#[test]
fn test_get_returns_reference_to_value() {
    let value = OverridableValue::new(42, true);
    assert_eq!(*value.get(), 42);
}

#[test]
fn test_get_mut_allows_modification() {
    let mut value = OverridableValue::new(42, true);
    *value.get_mut() = 100;
    assert_eq!(value.value, 100);
}

#[test]
fn test_into_inner_consumes_and_returns_value() {
    let value = OverridableValue::new(42, true);
    let inner = value.into_inner();
    assert_eq!(inner, 42);
}

#[test]
fn test_map_transforms_value_preserves_override_policy() {
    let value = OverridableValue::new(42, true);
    let doubled = value.map(|x| x * 2);
    assert_eq!(doubled.value, 84);
    assert!(doubled.override_allowed);
}

#[test]
fn test_map_preserves_fixed_policy() {
    let value = OverridableValue::new(42, false);
    let doubled = value.map(|x| x * 2);
    assert_eq!(doubled.value, 84);
    assert!(!doubled.override_allowed);
}

#[test]
fn test_map_changes_type() {
    let value = OverridableValue::new(42, true);
    let string_value = value.map(|x| x.to_string());
    assert_eq!(string_value.value, "42");
    assert!(string_value.override_allowed);
}

#[test]
fn test_clone_creates_independent_copy() {
    let value = OverridableValue::new(42, true);
    let cloned = value.clone();
    assert_eq!(cloned.value, value.value);
    assert_eq!(cloned.override_allowed, value.override_allowed);
}

#[test]
fn test_equality_compares_both_fields() {
    let v1 = OverridableValue::new(42, true);
    let v2 = OverridableValue::new(42, true);
    let v3 = OverridableValue::new(42, false);
    let v4 = OverridableValue::new(100, true);

    assert_eq!(v1, v2);
    assert_ne!(v1, v3); // Different override_allowed
    assert_ne!(v1, v4); // Different value
}

#[test]
fn test_default_creates_default_value_with_override_allowed() {
    let value: OverridableValue<i32> = OverridableValue::default();
    assert_eq!(value.value, 0); // i32::default()
    assert!(value.override_allowed);
}

#[test]
fn test_default_for_bool() {
    let value: OverridableValue<bool> = OverridableValue::default();
    assert_eq!(value.value, false); // bool::default()
    assert!(value.override_allowed);
}

#[test]
fn test_default_for_string() {
    let value: OverridableValue<String> = OverridableValue::default();
    assert_eq!(value.value, ""); // String::default()
    assert!(value.override_allowed);
}

// Serialization/Deserialization Tests

#[test]
fn test_serialize_to_toml_format() {
    let value = OverridableValue::new(true, false);
    let toml = toml::to_string(&value).expect("Failed to serialize");

    // TOML inline table format
    assert!(toml.contains("value"));
    assert!(toml.contains("true"));
    assert!(toml.contains("override_allowed"));
    assert!(toml.contains("false"));
}

#[test]
fn test_deserialize_from_toml_format() {
    let toml = r#"
        value = true
        override_allowed = false
    "#;

    let value: OverridableValue<bool> = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(value.value, true);
    assert_eq!(value.override_allowed, false);
}

#[test]
fn test_deserialize_inline_table_format() {
    let toml = r#"setting = { value = true, override_allowed = false }"#;

    #[derive(Deserialize)]
    struct Config {
        setting: OverridableValue<bool>,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.setting.value, true);
    assert_eq!(config.setting.override_allowed, false);
}

#[test]
fn test_round_trip_serialization_bool() {
    let original = OverridableValue::new(true, false);
    let toml = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: OverridableValue<bool> =
        toml::from_str(&toml).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn test_round_trip_serialization_i64() {
    let original = OverridableValue::new(42i64, true);
    let toml = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: OverridableValue<i64> = toml::from_str(&toml).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn test_round_trip_serialization_string() {
    let original = OverridableValue::new("test-value".to_string(), true);
    let toml = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: OverridableValue<String> =
        toml::from_str(&toml).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn test_deserialize_with_multiple_settings() {
    let toml = r#"
        [repository]
        issues = { value = true, override_allowed = true }
        wiki = { value = false, override_allowed = false }
    "#;

    #[derive(Deserialize)]
    struct RepositorySettings {
        issues: OverridableValue<bool>,
        wiki: OverridableValue<bool>,
    }

    #[derive(Deserialize)]
    struct Config {
        repository: RepositorySettings,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.repository.issues.value, true);
    assert!(config.repository.issues.override_allowed);
    assert_eq!(config.repository.wiki.value, false);
    assert!(!config.repository.wiki.override_allowed);
}

#[test]
fn test_generic_over_different_types() {
    let bool_val = OverridableValue::new(true, true);
    let int_val = OverridableValue::new(42, false);
    let string_val = OverridableValue::new("test".to_string(), true);

    assert!(bool_val.value);
    assert_eq!(int_val.value, 42);
    assert_eq!(string_val.value, "test");
}

#[test]
fn test_generic_over_vec() {
    let vec_val = OverridableValue::new(vec![1, 2, 3], true);
    assert_eq!(vec_val.value.len(), 3);
    assert!(vec_val.can_override());
}

#[test]
fn test_generic_over_option() {
    let some_val = OverridableValue::new(Some(42), true);
    let none_val = OverridableValue::new(None::<i32>, false);

    assert_eq!(some_val.value, Some(42));
    assert_eq!(none_val.value, None);
}

#[test]
fn test_debug_format() {
    let value = OverridableValue::new(42, true);
    let debug_str = format!("{:?}", value);
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("true"));
}

// Flexible deserialization tests

#[test]
fn test_deserialize_from_full_format() {
    // Test deserialization from explicit { value, override_allowed } format (GlobalDefaults style)
    let toml = r#"
        setting = { value = true, override_allowed = false }
    "#;

    #[derive(Deserialize)]
    struct Config {
        setting: OverridableValue<bool>,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.setting.value, true);
    assert!(!config.setting.override_allowed);
}

#[test]
fn test_deserialize_from_simple_format() {
    // Test deserialization from simple value format (Team/Template style)
    // Should auto-wrap with override_allowed = true
    let toml = r#"
        setting = true
    "#;

    #[derive(Deserialize)]
    struct Config {
        setting: OverridableValue<bool>,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.setting.value, true);
    assert!(
        config.setting.override_allowed,
        "Simple format should default to override_allowed = true"
    );
}

#[test]
fn test_deserialize_simple_integer() {
    // Test simple format with integer value
    let toml = r#"
        count = 42
    "#;

    #[derive(Deserialize)]
    struct Config {
        count: OverridableValue<i32>,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.count.value, 42);
    assert!(config.count.override_allowed);
}

#[test]
fn test_deserialize_simple_string() {
    // Test simple format with string value
    let toml = r#"
        name = "test-value"
    "#;

    #[derive(Deserialize)]
    struct Config {
        name: OverridableValue<String>,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");
    assert_eq!(config.name.value, "test-value");
    assert!(config.name.override_allowed);
}

#[test]
fn test_mixed_formats_in_same_config() {
    // Test that both formats can coexist in the same configuration
    // This is critical for the configuration hierarchy to work
    let toml = r#"
        [global]
        issues = { value = true, override_allowed = false }
        wiki = { value = false, override_allowed = true }

        [team]
        discussions = true
        projects = false
    "#;

    #[derive(Deserialize)]
    struct GlobalSettings {
        issues: OverridableValue<bool>,
        wiki: OverridableValue<bool>,
    }

    #[derive(Deserialize)]
    struct TeamSettings {
        discussions: OverridableValue<bool>,
        projects: OverridableValue<bool>,
    }

    #[derive(Deserialize)]
    struct Config {
        global: GlobalSettings,
        team: TeamSettings,
    }

    let config: Config = toml::from_str(toml).expect("Failed to deserialize");

    // Global (explicit format)
    assert_eq!(config.global.issues.value, true);
    assert!(!config.global.issues.override_allowed);
    assert_eq!(config.global.wiki.value, false);
    assert!(config.global.wiki.override_allowed);

    // Team (simple format)
    assert_eq!(config.team.discussions.value, true);
    assert!(config.team.discussions.override_allowed);
    assert_eq!(config.team.projects.value, false);
    assert!(config.team.projects.override_allowed);
}
