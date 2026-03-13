// Tests for RepositoryNamingRulesConfig

use super::*;

// ============================================================================
// RepositoryNamingRulesConfig struct tests
// ============================================================================

/// Verify Default produces an empty rule with all fields None/empty.
#[test]
fn test_naming_rules_config_default_is_empty() {
    let rule = RepositoryNamingRulesConfig::default();

    assert!(rule.description.is_none());
    assert!(rule.allowed_pattern.is_none());
    assert!(rule.forbidden_patterns.is_empty());
    assert!(rule.reserved_words.is_empty());
    assert!(rule.required_prefix.is_none());
    assert!(rule.required_suffix.is_none());
    assert!(rule.min_length.is_none());
    assert!(rule.max_length.is_none());
}

// ============================================================================
// TOML deserialization tests
// ============================================================================

/// Verify a fully-populated TOML entry deserializes correctly.
#[test]
fn test_naming_rules_config_deserialize_all_fields() {
    let toml = r#"
        description       = "Full constraint set"
        allowed_pattern   = "^[a-z][a-z0-9-]*$"
        forbidden_patterns = [".*--.*", ".*__.*"]
        reserved_words    = ["test", "demo", "temp"]
        required_prefix   = "acme-"
        required_suffix   = "-svc"
        min_length        = 5
        max_length        = 40
    "#;

    let rule: RepositoryNamingRulesConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(rule.description.as_deref(), Some("Full constraint set"));
    assert_eq!(rule.allowed_pattern.as_deref(), Some("^[a-z][a-z0-9-]*$"));
    assert_eq!(rule.forbidden_patterns, vec![".*--.*", ".*__.*"]);
    assert_eq!(rule.reserved_words, vec!["test", "demo", "temp"]);
    assert_eq!(rule.required_prefix.as_deref(), Some("acme-"));
    assert_eq!(rule.required_suffix.as_deref(), Some("-svc"));
    assert_eq!(rule.min_length, Some(5));
    assert_eq!(rule.max_length, Some(40));
}

/// Verify an empty TOML entry deserializes to the default (all-None) rule.
#[test]
fn test_naming_rules_config_deserialize_empty_is_default() {
    let toml = r#""#;

    let rule: RepositoryNamingRulesConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(rule, RepositoryNamingRulesConfig::default());
}

/// Verify partial TOML (only description and prefix) deserializes correctly.
#[test]
fn test_naming_rules_config_deserialize_partial_fields() {
    let toml = r#"
        description     = "Services must use the svc- prefix"
        required_prefix = "svc-"
    "#;

    let rule: RepositoryNamingRulesConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(
        rule.description.as_deref(),
        Some("Services must use the svc- prefix")
    );
    assert_eq!(rule.required_prefix.as_deref(), Some("svc-"));
    assert!(rule.required_suffix.is_none());
    assert!(rule.allowed_pattern.is_none());
    assert!(rule.forbidden_patterns.is_empty());
    assert!(rule.reserved_words.is_empty());
    assert!(rule.min_length.is_none());
    assert!(rule.max_length.is_none());
}

/// Verify reserved_words and forbidden_patterns default to empty vecs when absent.
#[test]
fn test_naming_rules_config_absent_vecs_default_to_empty() {
    let toml = r#"
        min_length = 3
    "#;

    let rule: RepositoryNamingRulesConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(rule.forbidden_patterns.is_empty());
    assert!(rule.reserved_words.is_empty());
    assert_eq!(rule.min_length, Some(3));
}

// ============================================================================
// TOML serialization round-trip tests
// ============================================================================

/// Verify a fully-populated rule serializes and deserializes correctly.
#[test]
fn test_naming_rules_config_serialize_round_trip() {
    let original = RepositoryNamingRulesConfig {
        description: Some("Round-trip test".to_string()),
        allowed_pattern: Some(r"^[a-z][a-z0-9-]*$".to_string()),
        forbidden_patterns: vec![".*--.*".to_string()],
        reserved_words: vec!["test".to_string()],
        required_prefix: Some("org-".to_string()),
        required_suffix: Some("-lib".to_string()),
        min_length: Some(6),
        max_length: Some(50),
    };

    let serialized = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: RepositoryNamingRulesConfig =
        toml::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(original, deserialized);
}

/// Verify that a default (all-None) rule serializes to an empty string (no keys).
#[test]
fn test_naming_rules_config_default_serializes_to_empty() {
    let rule = RepositoryNamingRulesConfig::default();
    let serialized = toml::to_string(&rule).expect("Failed to serialize");

    // All fields use skip_serializing_if = "Option::is_none" / Vec::is_empty,
    // so the empty rule should produce no key-value pairs.
    assert!(
        serialized.trim().is_empty(),
        "Default rule should serialize to empty TOML, got: {serialized:?}"
    );
}

// ============================================================================
// Clone and PartialEq tests
// ============================================================================

/// Verify Clone produces an equal value.
#[test]
fn test_naming_rules_config_clone() {
    let original = RepositoryNamingRulesConfig {
        description: Some("Clone test".to_string()),
        allowed_pattern: Some("^[a-z]+$".to_string()),
        forbidden_patterns: vec!["bad".to_string()],
        reserved_words: vec!["tmp".to_string()],
        required_prefix: Some("p-".to_string()),
        required_suffix: Some("-s".to_string()),
        min_length: Some(4),
        max_length: Some(20),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

/// Verify two different rules compare as not-equal.
#[test]
fn test_naming_rules_config_inequality() {
    let a = RepositoryNamingRulesConfig {
        required_prefix: Some("a-".to_string()),
        ..Default::default()
    };
    let b = RepositoryNamingRulesConfig {
        required_prefix: Some("b-".to_string()),
        ..Default::default()
    };

    assert_ne!(a, b);
}
