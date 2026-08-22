//! Unit and adversarial tests for `span_attributes`.
//!
//! Tier 1 (specification) tests map directly to the module-level contract in
//! `span_attributes.rs`. Tier 2 (adversarial) tests target boundary
//! conditions, side-effect-free purity, and stub-killing scenarios: every
//! test here fails against `todo!()` (the current, pre-implementation state)
//! and would also fail against a trivial hardcoded-return implementation
//! (e.g. one that always returns the input unchanged, or always returns the
//! marker, or always returns an empty string).

use super::*;

// ---------------------------------------------------------------------
// Tier 1: Specification tests
// ---------------------------------------------------------------------

#[test]
fn test_sanitize_span_attribute_value_empty_string_returns_empty() {
    assert_eq!(sanitize_span_attribute_value(""), "");
}

#[test]
fn test_sanitize_span_attribute_value_short_plain_org_name_passes_through_unchanged() {
    assert_eq!(sanitize_span_attribute_value("my-organization"), "my-organization");
}

#[test]
fn test_sanitize_span_attribute_value_short_plain_template_name_passes_through_unchanged() {
    assert_eq!(sanitize_span_attribute_value("rust-library"), "rust-library");
}

#[test]
fn test_sanitize_span_attribute_value_value_at_exactly_max_len_is_unchanged() {
    let value: String = std::iter::repeat('a').take(MAX_SPAN_ATTRIBUTE_LEN).collect();
    assert_eq!(sanitize_span_attribute_value(&value), value);
}

#[test]
fn test_sanitize_span_attribute_value_value_one_over_max_len_is_truncated_to_max_len() {
    let value: String = std::iter::repeat('a')
        .take(MAX_SPAN_ATTRIBUTE_LEN + 1)
        .collect();
    let result = sanitize_span_attribute_value(&value);
    assert_eq!(result.chars().count(), MAX_SPAN_ATTRIBUTE_LEN);
}

#[test]
fn test_sanitize_span_attribute_value_value_one_under_max_len_is_unchanged() {
    let value: String = std::iter::repeat('a')
        .take(MAX_SPAN_ATTRIBUTE_LEN - 1)
        .collect();
    assert_eq!(sanitize_span_attribute_value(&value), value);
}

#[test]
fn test_sanitize_span_attribute_value_gh_token_shape_is_fully_redacted() {
    let raw = format!("ghp_{}", "a".repeat(24));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_bearer_shape_is_fully_redacted() {
    let raw = "Bearer abcdefgh12345678";
    assert_eq!(sanitize_span_attribute_value(raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_shape_is_fully_redacted() {
    let raw = format!(
        "{}.{}.{}",
        "a".repeat(12),
        "b".repeat(12),
        "c".repeat(12)
    );
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_shape_is_fully_redacted() {
    // >= 32 chars from [A-Za-z0-9+/], mixed case + digit.
    let raw = "aB3aB3aB3aB3aB3aB3aB3aB3aB3aB3aB3aB3";
    assert!(raw.len() >= 32);
    assert_eq!(sanitize_span_attribute_value(raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

// ---------------------------------------------------------------------
// Tier 2: Adversarial / boundary tests
// ---------------------------------------------------------------------

#[test]
fn test_sanitize_span_attribute_value_secret_shape_embedded_inside_longer_org_name_is_redacted() {
    // A business-context value should never legitimately contain a secret
    // shape; if it does, the whole value is discarded, not just the match.
    let raw = format!("my-org-ghp_{}", "a".repeat(24));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_never_panics_on_pathologically_long_input() {
    let raw: String = std::iter::repeat('x').take(50_000).collect();
    let _ = sanitize_span_attribute_value(&raw);
}

#[test]
fn test_sanitize_span_attribute_value_never_panics_on_non_ascii_unicode() {
    let raw = "组织名称-🚀-émoji-テンプレート";
    let _ = sanitize_span_attribute_value(raw);
}

#[test]
fn test_sanitize_span_attribute_value_truncation_never_splits_a_multibyte_char() {
    // 200 repetitions of a 4-byte emoji: any naive byte-based truncation at
    // MAX_SPAN_ATTRIBUTE_LEN *bytes* would split a code point and panic (or
    // produce invalid UTF-8). Truncation must be char-based.
    let raw: String = std::iter::repeat('🚀').take(200).collect();
    let result = sanitize_span_attribute_value(&raw);
    // Must be valid UTF-8 by construction (String), and must not exceed the
    // char-count bound.
    assert!(result.chars().count() <= MAX_SPAN_ATTRIBUTE_LEN);
}

#[test]
fn test_sanitize_span_attribute_value_is_idempotent_for_plain_value() {
    let raw = "my-organization";
    let once = sanitize_span_attribute_value(raw);
    let twice = sanitize_span_attribute_value(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_sanitize_span_attribute_value_is_idempotent_for_secret_shaped_value() {
    let raw = format!("ghp_{}", "a".repeat(24));
    let once = sanitize_span_attribute_value(&raw);
    let twice = sanitize_span_attribute_value(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_sanitize_span_attribute_value_is_idempotent_for_truncated_value() {
    let raw: String = std::iter::repeat('a').take(500).collect();
    let once = sanitize_span_attribute_value(&raw);
    let twice = sanitize_span_attribute_value(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_sanitize_span_attribute_value_distinct_inputs_produce_distinct_outputs() {
    // Stub-killing: a hardcoded-return implementation (e.g. always "" or
    // always the marker) would collapse these two clearly-different,
    // non-secret-shaped, within-bound inputs to the same output.
    let a = sanitize_span_attribute_value("organization-alpha");
    let b = sanitize_span_attribute_value("organization-beta");
    assert_ne!(a, b);
}

#[test]
fn test_sanitize_span_attribute_value_does_not_always_return_the_marker() {
    // Stub-killing: guards against a lazy "always redact" implementation.
    let result = sanitize_span_attribute_value("perfectly-ordinary-template-name");
    assert_ne!(result, SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_does_not_always_pass_through_unchanged() {
    // Stub-killing: guards against a lazy "identity function" implementation.
    let raw = format!("ghp_{}", "a".repeat(24));
    let result = sanitize_span_attribute_value(&raw);
    assert_ne!(result, raw);
}

#[test]
fn test_sanitize_span_attribute_value_gh_token_boundary_19_chars_is_not_redacted() {
    // 19-char suffix is one below the 20-char threshold -- must NOT match.
    let raw = format!("ghp_{}", "a".repeat(19));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_gh_token_boundary_20_chars_is_redacted() {
    // Exactly 20-char suffix is the minimum that matches.
    let raw = format!("ghp_{}", "a".repeat(20));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_does_not_false_positive_on_uuid() {
    // UUIDs contain hyphens, which are excluded from the high-entropy
    // alphabet, so a UUID must never be misclassified as secret-shaped.
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    assert_ne!(sanitize_span_attribute_value(uuid), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_does_not_false_positive_on_iso8601_timestamp() {
    let ts = "2026-08-22T10:15:30Z";
    assert_ne!(sanitize_span_attribute_value(ts), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_does_not_false_positive_on_git_commit_sha() {
    // All-lowercase hex -- fails the mixed-case requirement for high-entropy.
    let sha = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
    assert_ne!(sanitize_span_attribute_value(sha), SPAN_ATTRIBUTE_REDACTED_MARKER);
}
