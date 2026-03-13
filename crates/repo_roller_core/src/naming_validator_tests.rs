// Tests for RepositoryNamingValidator

use super::*;
use config_manager::RepositoryNamingRulesConfig;

// ============================================================================
// Empty rule collection
// ============================================================================

/// An empty rule slice is always valid.
#[test]
fn test_empty_rules_always_passes() {
    let v = RepositoryNamingValidator::new();
    assert!(v.validate("anything", &[]).is_ok());
    assert!(v.validate("", &[]).is_ok());
}

// ============================================================================
// Min / max length
// ============================================================================

/// Name shorter than min_length is rejected.
#[test]
fn test_min_length_rejects_short_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        min_length: Some(6),
        ..Default::default()
    }];

    let err = v.validate("abc", &rules).unwrap_err();
    assert!(
        err.to_string().contains("too short"),
        "Error should mention 'too short': {err}"
    );
}

/// Name exactly at min_length is accepted.
#[test]
fn test_min_length_accepts_exact_length() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        min_length: Some(4),
        ..Default::default()
    }];
    assert!(v.validate("abcd", &rules).is_ok());
}

/// Name longer than max_length is rejected.
#[test]
fn test_max_length_rejects_long_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        max_length: Some(5),
        ..Default::default()
    }];

    let err = v.validate("toolongname", &rules).unwrap_err();
    assert!(
        err.to_string().contains("too long"),
        "Error should mention 'too long': {err}"
    );
}

/// Name exactly at max_length is accepted.
#[test]
fn test_max_length_accepts_exact_length() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        max_length: Some(5),
        ..Default::default()
    }];
    assert!(v.validate("hello", &rules).is_ok());
}

/// A naming rule where min_length > max_length is a misconfiguration.
/// The validator should return an error immediately rather than silently
/// rejecting every name with a misleading length message.
#[test]
fn test_min_length_greater_than_max_length_is_config_error() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        min_length: Some(10),
        max_length: Some(5),
        ..Default::default()
    }];

    let err = v.validate("any-name", &rules).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Misconfigured")
            || msg.contains("misconfigured")
            || msg.contains("min_length"),
        "Error should describe the misconfiguration; got: {msg}"
    );
}

// ============================================================================
// Required prefix / suffix
// ============================================================================

/// Name without the required prefix is rejected.
#[test]
fn test_required_prefix_rejects_missing_prefix() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        required_prefix: Some("acme-".to_string()),
        ..Default::default()
    }];

    let err = v.validate("payments", &rules).unwrap_err();
    assert!(
        err.to_string().contains("must start with 'acme-'"),
        "Error should mention prefix: {err}"
    );
}

/// Name with the required prefix is accepted.
#[test]
fn test_required_prefix_accepts_correct_prefix() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        required_prefix: Some("acme-".to_string()),
        ..Default::default()
    }];
    assert!(v.validate("acme-payments", &rules).is_ok());
}

/// Name without the required suffix is rejected.
#[test]
fn test_required_suffix_rejects_missing_suffix() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        required_suffix: Some("-svc".to_string()),
        ..Default::default()
    }];

    let err = v.validate("payments", &rules).unwrap_err();
    assert!(
        err.to_string().contains("must end with '-svc'"),
        "Error should mention suffix: {err}"
    );
}

/// Name with the required suffix is accepted.
#[test]
fn test_required_suffix_accepts_correct_suffix() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        required_suffix: Some("-svc".to_string()),
        ..Default::default()
    }];
    assert!(v.validate("payments-svc", &rules).is_ok());
}

// ============================================================================
// Reserved words
// ============================================================================

/// Exact reserved-word match (case-insensitive) is rejected.
#[test]
fn test_reserved_word_rejects_exact_match() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        reserved_words: vec!["test".to_string(), "demo".to_string()],
        ..Default::default()
    }];

    let err = v.validate("test", &rules).unwrap_err();
    assert!(
        err.to_string().contains("reserved word"),
        "Error should mention reserved word: {err}"
    );
}

/// Reserved word match is case-insensitive.
#[test]
fn test_reserved_word_case_insensitive() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        reserved_words: vec!["Test".to_string()],
        ..Default::default()
    }];

    assert!(v.validate("test", &rules).is_err());
    assert!(v.validate("TEST", &rules).is_err());
    assert!(v.validate("Test", &rules).is_err());
}

/// Name that contains a reserved word but is not equal is accepted.
#[test]
fn test_reserved_word_partial_match_is_accepted() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        reserved_words: vec!["test".to_string()],
        ..Default::default()
    }];
    // "test-service" is NOT "test" — should pass
    assert!(v.validate("test-service", &rules).is_ok());
}

// ============================================================================
// Allowed pattern
// ============================================================================

/// Name matching the allowed pattern is accepted.
#[test]
fn test_allowed_pattern_accepts_matching_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        allowed_pattern: Some(r"[a-z][a-z0-9-]*".to_string()),
        ..Default::default()
    }];
    assert!(v.validate("my-service", &rules).is_ok());
}

/// Name not matching the allowed pattern is rejected.
#[test]
fn test_allowed_pattern_rejects_non_matching_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        allowed_pattern: Some(r"[a-z][a-z0-9-]*".to_string()),
        ..Default::default()
    }];

    let err = v.validate("My_Service", &rules).unwrap_err();
    assert!(
        err.to_string().contains("does not match required pattern"),
        "Error should mention pattern: {err}"
    );
}

/// Allowed pattern is implicitly anchored to the full name.
#[test]
fn test_allowed_pattern_is_full_name_anchored() {
    let v = RepositoryNamingValidator::new();
    // Pattern matches only lowercase letters — trailing capital should fail
    let rules = vec![RepositoryNamingRulesConfig {
        allowed_pattern: Some(r"[a-z]+".to_string()),
        ..Default::default()
    }];
    // Full name "abcZ" contains uppercase — anchored pattern must reject it
    assert!(v.validate("abcZ", &rules).is_err());
    // "abc" matches the full-name anchored pattern
    assert!(v.validate("abc", &rules).is_ok());
}

// ============================================================================
// Forbidden patterns
// ============================================================================

/// Name matching a forbidden pattern is rejected.
#[test]
fn test_forbidden_pattern_rejects_matching_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        forbidden_patterns: vec![".*--.*".to_string()],
        ..Default::default()
    }];

    let err = v.validate("my--service", &rules).unwrap_err();
    assert!(
        err.to_string().contains("matches forbidden pattern"),
        "Error should mention forbidden: {err}"
    );
}

/// Name not matching any forbidden pattern is accepted.
#[test]
fn test_forbidden_pattern_accepts_non_matching_name() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        forbidden_patterns: vec![".*--.*".to_string()],
        ..Default::default()
    }];
    assert!(v.validate("my-service", &rules).is_ok());
}

// ============================================================================
// Multiple rules (additive — all must pass)
// ============================================================================

/// All rules must pass; first violation stops evaluation.
#[test]
fn test_multiple_rules_first_violation_stops() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![
        RepositoryNamingRulesConfig {
            description: Some("Prefix rule".to_string()),
            required_prefix: Some("acme-".to_string()),
            ..Default::default()
        },
        RepositoryNamingRulesConfig {
            description: Some("Suffix rule".to_string()),
            required_suffix: Some("-svc".to_string()),
            ..Default::default()
        },
    ];

    // Fails on prefix rule
    let err = v.validate("payments-svc", &rules).unwrap_err();
    assert!(
        err.to_string().contains("must start with 'acme-'"),
        "Should fail on prefix rule: {err}"
    );

    // Passes both rules
    assert!(v.validate("acme-payments-svc", &rules).is_ok());
}

/// A name that satisfies the first rule but fails the second is rejected.
#[test]
fn test_multiple_rules_second_violation_detected() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![
        RepositoryNamingRulesConfig {
            required_prefix: Some("acme-".to_string()),
            ..Default::default()
        },
        RepositoryNamingRulesConfig {
            required_suffix: Some("-lib".to_string()),
            ..Default::default()
        },
    ];

    // Passes prefix, fails suffix
    let err = v.validate("acme-payments", &rules).unwrap_err();
    assert!(
        err.to_string().contains("must end with '-lib'"),
        "Should fail on suffix rule: {err}"
    );
}

// ============================================================================
// Rule description in error messages
// ============================================================================

/// Error message includes the rule description when set.
#[test]
fn test_rule_description_included_in_error() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        description: Some("Services must use the svc- prefix".to_string()),
        required_prefix: Some("svc-".to_string()),
        ..Default::default()
    }];

    let err = v.validate("payments", &rules).unwrap_err();
    assert!(
        err.to_string()
            .contains("Services must use the svc- prefix"),
        "Error should include rule description: {err}"
    );
}

// ============================================================================
// Invalid regex patterns
// ============================================================================

/// An invalid allowed_pattern regex returns an error describing the issue.
#[test]
fn test_invalid_allowed_pattern_regex_returns_error() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        allowed_pattern: Some(r"[invalid((".to_string()),
        ..Default::default()
    }];

    assert!(v.validate("any-name", &rules).is_err());
}

/// An invalid forbidden_pattern regex returns an error describing the issue.
#[test]
fn test_invalid_forbidden_pattern_regex_returns_error() {
    let v = RepositoryNamingValidator::new();
    let rules = vec![RepositoryNamingRulesConfig {
        forbidden_patterns: vec![r"[bad((".to_string()],
        ..Default::default()
    }];

    assert!(v.validate("any-name", &rules).is_err());
}
