// Property-based tests for RepositoryNamingValidator.
//
// These tests use proptest to verify invariants that hold for ALL inputs
// within a given domain, not just hand-picked examples.

use super::*;
use config_manager::RepositoryNamingRulesConfig;
use proptest::prelude::*;

proptest! {
    /// Any name whose byte-length is strictly less than `min_length` must be
    /// rejected, regardless of its content.
    #[test]
    fn prop_name_shorter_than_min_length_always_rejected(
        // Generate a min_length in [2, 64] and a name strictly shorter than it.
        min in 2usize..=64usize,
        name in "[a-z]{1}",  // length 1 — always < min (min ≥ 2)
    ) {
        let v = RepositoryNamingValidator::new();
        let rules = vec![RepositoryNamingRulesConfig {
            min_length: Some(min),
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&name, &rules).is_err(),
            "name '{}' (len {}) should be rejected by min_length={}", name, name.len(), min
        );
    }

    /// Any name whose byte-length is strictly greater than `max_length` must be
    /// rejected, regardless of its content.
    #[test]
    fn prop_name_longer_than_max_length_always_rejected(
        // max_length in [1, 30]; name is always "aaaa…" of length max+1
        max in 1usize..=30usize,
    ) {
        let v = RepositoryNamingValidator::new();
        let name = "a".repeat(max + 1);
        let rules = vec![RepositoryNamingRulesConfig {
            max_length: Some(max),
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&name, &rules).is_err(),
            "name of length {} should be rejected by max_length={}", name.len(), max
        );
    }

    /// Any name whose byte-length is exactly `min_length` passes the length
    /// constraint (assuming no other constraints).
    #[test]
    fn prop_name_at_exact_min_length_passes_length_check(
        len in 1usize..=50usize,
    ) {
        let v = RepositoryNamingValidator::new();
        let name = "a".repeat(len);
        let rules = vec![RepositoryNamingRulesConfig {
            min_length: Some(len),
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&name, &rules).is_ok(),
            "name of length {} should pass min_length={}", len, len
        );
    }

    /// Any name whose byte-length is exactly `max_length` passes the length
    /// constraint (assuming no other constraints).
    #[test]
    fn prop_name_at_exact_max_length_passes_length_check(
        len in 1usize..=50usize,
    ) {
        let v = RepositoryNamingValidator::new();
        let name = "a".repeat(len);
        let rules = vec![RepositoryNamingRulesConfig {
            max_length: Some(len),
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&name, &rules).is_ok(),
            "name of length {} should pass max_length={}", len, len
        );
    }

    /// Any name that does NOT start with the required prefix must be rejected.
    #[test]
    fn prop_name_without_required_prefix_always_rejected(
        // Prefix is "svc-"; any lower-alpha name that starts with anything
        // other than 's' can never match the prefix.
        name in "[a-rt-z][a-z0-9-]{2,10}",
    ) {
        let v = RepositoryNamingValidator::new();
        let rules = vec![RepositoryNamingRulesConfig {
            required_prefix: Some("svc-".to_string()),
            ..Default::default()
        }];
        // Names starting with 's' could match; exclude that ambiguity via the regex above.
        prop_assert!(
            v.validate(&name, &rules).is_err(),
            "name '{}' missing prefix 'svc-' should be rejected", name
        );
    }

    /// Any name that does NOT end with the required suffix must be rejected.
    #[test]
    fn prop_name_without_required_suffix_always_rejected(
        // Suffix is "-api"; names generated here end with letters a-z only.
        name in "[a-z]{3,10}",
    ) {
        let v = RepositoryNamingValidator::new();
        let rules = vec![RepositoryNamingRulesConfig {
            required_suffix: Some("-api".to_string()),
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&name, &rules).is_err(),
            "name '{}' missing suffix '-api' should be rejected", name
        );
    }

    /// A name that contains a reserved word must be rejected regardless of the
    /// surrounding characters.
    ///
    /// The validator checks for a case-insensitive exact match against the
    /// whole name, so a name equal to the reserved word must always fail.
    #[test]
    fn prop_name_equal_to_reserved_word_always_rejected(
        // Vary the reserved word itself.
        word in "[a-z]{3,12}",
    ) {
        let v = RepositoryNamingValidator::new();
        let rules = vec![RepositoryNamingRulesConfig {
            reserved_words: vec![word.clone()],
            ..Default::default()
        }];
        prop_assert!(
            v.validate(&word, &rules).is_err(),
            "name '{}' equals reserved word and must be rejected", word
        );
    }

    /// Empty rule slice never rejects any name (no constraints to violate).
    #[test]
    fn prop_empty_rules_never_reject_any_name(
        name in ".*",
    ) {
        let v = RepositoryNamingValidator::new();
        prop_assert!(
            v.validate(&name, &[]).is_ok(),
            "empty rules must accept every name, including '{}'", name
        );
    }
}
