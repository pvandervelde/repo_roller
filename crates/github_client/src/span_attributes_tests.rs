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

// ---------------------------------------------------------------------
// Tier 3: Mutation-kill tests (QA audit pass, #2 tracing).
//
// Each test below targets a specific surviving mutant identified by
// `cargo mutants --file crates/github_client/src/span_attributes.rs`. See
// the doc comment on each test for the exact mutation it kills.
// ---------------------------------------------------------------------

#[test]
fn test_sanitize_span_attribute_value_jwt_shape_with_hyphens_in_segments_is_redacted() {
    // Kills: `173:43 replace || with && in is_jwt_segment_char` (the '-'
    // branch). '-' must be included in the JWT segment alphabet (base64url
    // includes '-') -- a scanner that drops it would fragment every segment
    // containing a hyphen below the minimum length threshold.
    let raw = format!("{}.{}.{}", "A-AAAAAAAA", "B-BBBBBBBB", "C-CCCCCCCC");
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_gh_token_suffix_with_underscore_is_redacted() {
    // Kills: `169:36 replace == with != in is_gh_token_suffix_char`. '_'
    // must be included in the token-suffix alphabet -- GitHub tokens can
    // (and often do) contain underscores.
    let raw = format!("ghp_{}_{}", "a".repeat(10), "a".repeat(13));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_gh_token_suffix_run_broken_by_space_is_not_redacted() {
    // Kills: `169:5 replace is_gh_token_suffix_char -> bool with true`. A
    // permissive "always true" predicate would treat a space as part of the
    // token suffix and falsely redact this otherwise-benign value (real
    // suffix run is only 2 chars, well under the 20-char minimum).
    let raw = format!("ghp_ab {}", "c".repeat(20));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_first_segment_below_minimum_is_not_redacted() {
    // Kills: `319:17 replace < with ==` and `319:17 replace < with >` in
    // `jwt_match_len`. Segment 1 is only 5 chars (below the 10-char
    // minimum) even though it is immediately followed by a valid dot --
    // both mutated comparisons fail to reject this too-short segment.
    let raw = format!("AAAAA.{}.{}", "B".repeat(10), "C".repeat(10));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_missing_dot_after_first_segment_is_not_redacted() {
    // Kills: `319:39 replace || with && in jwt_match_len`. Segment 1 is a
    // valid length but is NOT followed by a dot ('#' instead) -- the
    // mutated `&&` fails to reject this, silently skipping past the
    // missing separator.
    let raw = format!("{}#{}.{}", "A".repeat(10), "B".repeat(10), "C".repeat(10));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_second_segment_below_minimum_is_not_redacted() {
    // Kills: `324:17 replace < with == in jwt_match_len`. Segment 2 is only
    // 5 chars (below the 10-char minimum) even though it is immediately
    // followed by a valid dot.
    let raw = format!("{}.BBBBB.{}", "A".repeat(10), "C".repeat(10));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_missing_dot_after_second_segment_is_not_redacted() {
    // Kills: `324:39 replace || with && in jwt_match_len`. Segment 2 is a
    // valid length but is NOT followed by a dot ('#' instead).
    let raw = format!("{}.{}#{}", "A".repeat(10), "B".repeat(10), "C".repeat(10));
    assert_ne!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_exact_minimum_segment_lengths_is_redacted() {
    // Kills: `322:28 replace + with * in jwt_match_len` (the `i + seg1`
    // term of `seg2_start`). Using distinct characters per segment (rather
    // than a uniform repeated character) ensures a miscomputed boundary
    // lands on a run that is too short to still coincidentally match.
    let raw = format!("{}.{}.{}", "A".repeat(10), "B".repeat(10), "C".repeat(10));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_jwt_second_segment_longer_than_first_boundary_never_panics() {
    // Kills: `327:37 replace + with - in jwt_match_len` (the
    // `seg2_start + seg2` term of `seg3_start`). If this became a
    // subtraction, `seg2_start (11) - seg2 (15)` would underflow `usize`
    // and panic -- this is also a defense-in-depth regression test for the
    // module's documented "never panics on any input" contract.
    let raw = format!("{}.{}.{}", "A".repeat(10), "B".repeat(15), "C".repeat(10));
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_lowercase_only_late_in_run_is_still_redacted() {
    // Kills: `224:77 replace + with * in HighEntropyIndex::build` (the
    // `has_lower[i + 1]` propagation term). Lowercase only appears from
    // position 2 onward, not at the run's start -- broken propagation would
    // report `has_lower[0] == false` and miss the match.
    let raw = format!("3B{}", "a".repeat(30));
    assert_eq!(raw.chars().count(), 32);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_uppercase_only_late_in_run_is_still_redacted() {
    // Kills: `223:77 replace + with * in HighEntropyIndex::build` (the
    // `has_upper[i + 1]` propagation term).
    let raw = format!("3a{}", "B".repeat(30));
    assert_eq!(raw.chars().count(), 32);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_digit_only_late_in_run_is_still_redacted() {
    // Kills: `225:73 replace + with * in HighEntropyIndex::build` (the
    // `has_digit[i + 1]` propagation term).
    let raw = format!("aB{}", "3".repeat(30));
    assert_eq!(raw.chars().count(), 32);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_run_containing_slash_is_redacted() {
    // Kills: `177:43 replace || with && in is_high_entropy_char` and
    // `177:48 replace == with != in is_high_entropy_char` (both collapse
    // to excluding '/'). '/' is part of the standard base64 alphabet and
    // must not break an otherwise-qualifying run.
    let raw = format!("aB3/{}", "aB3".repeat(10));
    assert_eq!(raw.chars().count(), 34);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_run_containing_plus_is_redacted() {
    // Kills: `177:36 replace == with != in is_high_entropy_char` (excludes
    // '+'). '+' is part of the standard base64 alphabet and must not break
    // an otherwise-qualifying run.
    let raw = format!("aB3+{}", "aB3".repeat(10));
    assert_eq!(raw.chars().count(), 34);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_sentence_with_spaces_is_not_misclassified_as_high_entropy() {
    // Kills: `177:5 replace is_high_entropy_char -> bool with true` (and
    // incidentally the `177:36`/`177:48` over-permissive variants too). An
    // ordinary space-separated sentence containing mixed case and a digit
    // must not be treated as one giant high-entropy run just because an
    // "always true" (or near-always-true) predicate ignores whitespace.
    let raw = "Organization Display Name For Team 7";
    assert!(raw.chars().count() >= 32);
    assert_ne!(sanitize_span_attribute_value(raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}

#[test]
fn test_sanitize_span_attribute_value_high_entropy_length_exactly_at_threshold_with_diversity_at_start_is_redacted(
) {
    // Kills: `241:29 replace >= with < in HighEntropyIndex::match_len_at`.
    // Constructed so that only the run's start position (i=0) has both the
    // required length (exactly 32) AND the required upper/digit/lower mix;
    // every later position lacks either the length or a needed character
    // class (the tail is pure lowercase), so an inverted `<` comparison
    // collapses to "never matches" instead of "always matches except at 0".
    let raw = format!("A3{}", "a".repeat(30));
    assert_eq!(raw.chars().count(), 32);
    assert_eq!(sanitize_span_attribute_value(&raw), SPAN_ATTRIBUTE_REDACTED_MARKER);
}
