//! Property-based tests for `span_attributes::sanitize_span_attribute_value`.
//!
//! These exercise the invariants that must hold across the entire input
//! space, not just the hand-picked examples in `span_attributes_tests.rs`:
//! totality (never panics), the length bound, idempotency, and "a value
//! containing a deliberately-embedded secret shape is never returned
//! unredacted".

use super::*;
use proptest::prelude::*;

proptest! {
    /// Never panics for any `char` sequence, however pathological.
    #[test]
    fn prop_sanitize_span_attribute_value_never_panics(raw in ".{0,500}") {
        let _ = sanitize_span_attribute_value(&raw);
    }

    /// The result never exceeds `MAX_SPAN_ATTRIBUTE_LEN` characters, for any
    /// input length.
    #[test]
    fn prop_sanitize_span_attribute_value_never_exceeds_max_len(raw in ".{0,2000}") {
        let result = sanitize_span_attribute_value(&raw);
        prop_assert!(result.chars().count() <= MAX_SPAN_ATTRIBUTE_LEN);
    }

    /// Idempotency holds across the whole input space: sanitizing an
    /// already-sanitized value is a no-op.
    #[test]
    fn prop_sanitize_span_attribute_value_is_idempotent(raw in ".{0,500}") {
        let once = sanitize_span_attribute_value(&raw);
        let twice = sanitize_span_attribute_value(&once);
        prop_assert_eq!(once, twice);
    }

    /// A value containing a deliberately-embedded GitHub-token-shaped
    /// substring, wrapped in arbitrary surrounding text, is always fully
    /// redacted rather than passed through with the secret intact.
    #[test]
    fn prop_gh_token_embedded_in_arbitrary_text_is_always_redacted(
        prefix in "[a-zA-Z0-9_-]{0,20}",
        suffix in "[a-zA-Z0-9_-]{0,20}",
    ) {
        let token = format!("ghp_{}", "A".repeat(25));
        let raw = format!("{prefix}{token}{suffix}");
        let result = sanitize_span_attribute_value(&raw);
        prop_assert!(!result.contains(&token));
    }

    /// Plain, short, alphanumeric-plus-hyphen identifiers (the realistic
    /// shape of an organization login, repository name, or template name)
    /// are never redacted -- only secret-shaped values are.
    #[test]
    fn prop_plain_identifier_like_values_are_never_redacted(
        raw in "[a-z][a-z0-9-]{0,39}"
    ) {
        let result = sanitize_span_attribute_value(&raw);
        prop_assert_ne!(result, SPAN_ATTRIBUTE_REDACTED_MARKER.to_string());
    }
}
