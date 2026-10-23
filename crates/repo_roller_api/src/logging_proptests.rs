//! Property-based (Tier 3) tests for `crate::logging`'s pure functions.
//!
//! These verify invariants that must hold across arbitrary generated
//! inputs, not just hand-picked examples: no panics on adversarial garbage
//! input, no leaked sensitive values regardless of which sensitive key or
//! value shape is generated, idempotency, and preservation of non-sensitive
//! data.

use super::*;
use proptest::prelude::*;

/// One of the deny-listed name fragments, with randomised casing / compound
/// forms -- all of these MUST be classified as sensitive.
fn sensitive_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("token".to_string()),
        Just("TOKEN".to_string()),
        Just("password".to_string()),
        Just("Password".to_string()),
        Just("secret".to_string()),
        Just("authorization".to_string()),
        Just("Authorization".to_string()),
        Just("github_token".to_string()),
        Just("jwt".to_string()),
        Just("api_key".to_string()),
        Just("db_password".to_string()),
        Just("x-authorization-header".to_string()),
        Just("user_jwt_token".to_string()),
    ]
}

/// A field name known NOT to be on the deny-list -- these must always
/// survive redaction unchanged.
fn safe_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("message".to_string()),
        Just("method".to_string()),
        Just("status".to_string()),
        Just("request_id".to_string()),
        Just("uri".to_string()),
        Just("user_login".to_string()),
        Just("target".to_string()),
        Just("level".to_string()),
    ]
}

/// Arbitrary JSON primitive value, covering every value shape
/// `redact_sensitive_fields` must handle regardless of field name.
fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        any::<bool>().prop_map(serde_json::Value::from),
        any::<i64>().prop_map(serde_json::Value::from),
        "[a-zA-Z0-9 _-]{0,40}".prop_map(serde_json::Value::from),
        Just(serde_json::Value::Null),
    ]
}

/// Collapses `(key, value)` pairs that share a key down to a single entry
/// per key, keeping the *last* value for each -- mirroring the last-write-
/// wins semantics of inserting into a `serde_json::Map` in `build_line`.
/// Used to make per-pair "did this survive unchanged?" assertions
/// well-defined even when the generator produces the same key twice with
/// different values.
fn dedup_pairs_keep_last(
    pairs: &[(String, serde_json::Value)],
) -> Vec<(String, serde_json::Value)> {
    let mut deduped: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();
    for (k, v) in pairs {
        deduped.insert(k.clone(), v.clone());
    }
    deduped.into_iter().collect()
}

fn build_line(
    safe_pairs: &[(String, serde_json::Value)],
    sensitive_pairs: &[(String, serde_json::Value)],
) -> String {
    let mut fields = serde_json::Map::new();
    for (k, v) in safe_pairs {
        fields.insert(k.clone(), v.clone());
    }
    for (k, v) in sensitive_pairs {
        fields.insert(k.clone(), v.clone());
    }

    serde_json::json!({
        "timestamp": "2024-01-01T00:00:00.000000Z",
        "level": "INFO",
        "fields": fields,
        "target": "repo_roller_api::test"
    })
    .to_string()
}

proptest! {
    /// For any JSON object mixing sensitive and safe fields, every
    /// sensitive-named field must end up as exactly `REDACTED_MARKER`, and
    /// the output must remain valid JSON -- regardless of how many sensitive
    /// fields, which ones, or what value types they carry.
    #[test]
    fn prop_redact_replaces_every_sensitive_field_and_stays_valid_json(
        sensitive_pairs in prop::collection::vec((sensitive_key_strategy(), arb_json_value()), 1..6),
        safe_pairs in prop::collection::vec((safe_key_strategy(), arb_json_value()), 0..6),
    ) {
        let line = build_line(&safe_pairs, &sensitive_pairs);
        let redacted = redact_sensitive_fields(&line);

        let parsed: serde_json::Value = serde_json::from_str(&redacted)
            .expect("redacted output must remain valid JSON");
        let out_fields = parsed
            .get("fields")
            .and_then(|f| f.as_object())
            .expect("fields object must survive redaction");

        // Safe even when `sensitive_pairs` contains duplicate keys with
        // different values: every occurrence names a key that must map to
        // the same constant `REDACTED_MARKER`, so the assertion never
        // depends on which duplicate's original value "won".
        for (k, _) in &sensitive_pairs {
            prop_assert_eq!(
                out_fields.get(k).cloned(),
                Some(serde_json::Value::String(REDACTED_MARKER.to_string()))
            );
        }
    }

    /// Safe (non-sensitive) fields must be preserved exactly, deep-equal to
    /// their original value, regardless of what sensitive fields are also
    /// present in the same line.
    #[test]
    fn prop_redact_preserves_safe_fields_unchanged(
        sensitive_pairs in prop::collection::vec((sensitive_key_strategy(), arb_json_value()), 0..4),
        safe_pairs in prop::collection::vec((safe_key_strategy(), arb_json_value()), 1..6),
    ) {
        // `safe_key_strategy` draws with replacement, so `safe_pairs` may
        // contain the same key twice with different generated values.
        // `build_line` inserts pairs into a `serde_json::Map` (last write
        // wins), so only the deduplicated view is well-defined to assert
        // against below.
        let safe_pairs = dedup_pairs_keep_last(&safe_pairs);

        let line = build_line(&safe_pairs, &sensitive_pairs);
        let redacted = redact_sensitive_fields(&line);
        let parsed: serde_json::Value = serde_json::from_str(&redacted)
            .expect("redacted output must remain valid JSON");
        let out_fields = parsed
            .get("fields")
            .and_then(|f| f.as_object())
            .expect("fields object must survive redaction");

        for (k, v) in &safe_pairs {
            // A safe key can never collide with a sensitive key here because
            // the two strategies draw from disjoint fixed vocabularies.
            prop_assert_eq!(out_fields.get(k), Some(v));
        }
    }

    /// Redaction must be idempotent: applying it twice must be structurally
    /// identical to applying it once.
    #[test]
    fn prop_redact_is_idempotent(
        sensitive_pairs in prop::collection::vec((sensitive_key_strategy(), arb_json_value()), 0..4),
        safe_pairs in prop::collection::vec((safe_key_strategy(), arb_json_value()), 0..4),
    ) {
        let line = build_line(&safe_pairs, &sensitive_pairs);
        let once = redact_sensitive_fields(&line);
        let twice = redact_sensitive_fields(&once);

        // Safe even with duplicate keys in `safe_pairs`/`sensitive_pairs`:
        // this only compares the two fully-parsed outputs structurally, and
        // never re-derives an expectation from the original pairs, so
        // last-write-wins collapse (which happens identically both times)
        // cannot make the assertion unsatisfiable.
        let v1: serde_json::Value = serde_json::from_str(&once).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&twice).unwrap();
        prop_assert_eq!(v1, v2);
    }

    /// `redact_sensitive_fields` must never panic, no matter how garbled the
    /// input is -- a defensive, total function over arbitrary strings.
    #[test]
    fn prop_redact_sensitive_fields_never_panics_on_arbitrary_input(input in ".{0,300}") {
        let _ = redact_sensitive_fields(&input);
    }

    /// `is_sensitive_field_name` must never panic on arbitrary input,
    /// including empty strings, unicode, and very long strings.
    #[test]
    fn prop_is_sensitive_field_name_never_panics(name in ".{0,300}") {
        let _ = is_sensitive_field_name(&name);
    }

    /// `resolve_log_format` must never panic on arbitrary input and must
    /// always resolve to one of the two defined variants.
    #[test]
    fn prop_resolve_log_format_never_panics(raw in proptest::option::of(".{0,100}")) {
        let format = resolve_log_format(raw.as_deref());
        prop_assert!(matches!(format, LogFormat::Json | LogFormat::Pretty));
    }

    /// Any field name actually drawn from the deny-list vocabulary (in any
    /// casing/compound form used across this suite) must be classified
    /// sensitive -- restates the specification as a property so it holds
    /// even if the deny-list strategy above is extended later.
    #[test]
    fn prop_all_generated_sensitive_keys_are_classified_sensitive(key in sensitive_key_strategy()) {
        prop_assert!(is_sensitive_field_name(&key));
    }

    /// Any field name drawn from the safe vocabulary must never be
    /// classified sensitive.
    #[test]
    fn prop_all_generated_safe_keys_are_classified_not_sensitive(key in safe_key_strategy()) {
        prop_assert!(!is_sensitive_field_name(&key));
    }
}

// ─────────────────────────────────────────────────────────────────────────
// looks_like_secret_value / redact_secret_shaped_substrings — Tier 3
// property tests.
//
// Security remediation (HIGH-severity Security Reviewer finding): value-
// shaped redaction must catch secrets embedded in free text regardless of
// exact token content, while never false-positiving on UUIDs, ISO
// timestamps, or ordinary prose.
// ─────────────────────────────────────────────────────────────────────────

/// A random GitHub-token-shaped suffix, always long enough (20-60 chars) to
/// satisfy the documented threshold regardless of the random length drawn.
fn gh_token_suffix_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9]{20,60}"
}

/// A GitHub token prefix drawn from every recognised prefix in the
/// contract.
fn gh_token_prefix_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("ghp_"),
        Just("gho_"),
        Just("ghu_"),
        Just("ghs_"),
        Just("ghr_"),
        Just("github_pat_"),
    ]
}

/// Plain-word "prose" tokens drawn from a small vocabulary of ordinary
/// words -- used to build surrounding text that must never itself be
/// misclassified as secret-shaped.
fn prose_word_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("the"),
        Just("quick"),
        Just("brown"),
        Just("fox"),
        Just("completed"),
        Just("request"),
        Just("successfully"),
        Just("loaded"),
        Just("configuration"),
        Just("for"),
        Just("organization"),
    ]
}

fn prose_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(prose_word_strategy(), 0..8).prop_map(|words| words.join(" "))
}

/// A syntactically valid (lowercase) UUID-v4-shaped string. Must never be
/// classified as secret-shaped, regardless of which random hex digits are
/// drawn.
fn uuid_like_strategy() -> impl Strategy<Value = String> {
    (
        "[0-9a-f]{8}",
        "[0-9a-f]{4}",
        "[0-9a-f]{4}",
        "[0-9a-f]{4}",
        "[0-9a-f]{12}",
    )
        .prop_map(|(a, b, c, d, e)| format!("{a}-{b}-{c}-{d}-{e}"))
}

/// An ISO-8601-shaped timestamp string with random (valid-range) components.
/// Must never be classified as secret-shaped.
fn iso_timestamp_like_strategy() -> impl Strategy<Value = String> {
    (
        2000u32..2100,
        1u32..13,
        1u32..29,
        0u32..24,
        0u32..60,
        0u32..60,
    )
        .prop_map(|(year, month, day, hour, minute, second)| {
            format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
        })
}

proptest! {
    /// Every GitHub-token-prefixed string with a suffix at or above the
    /// documented 20-char threshold must be classified secret-shaped,
    /// regardless of which prefix or which specific random suffix is drawn.
    #[test]
    fn prop_looks_like_secret_value_gh_token_prefixed_strings_are_always_secret(
        prefix in gh_token_prefix_strategy(),
        suffix in gh_token_suffix_strategy(),
    ) {
        let value = format!("{prefix}{suffix}");
        prop_assert!(looks_like_secret_value(&value));
    }

    /// `looks_like_secret_value` must never panic, no matter how garbled or
    /// long the input is.
    #[test]
    fn prop_looks_like_secret_value_never_panics(input in ".{0,500}") {
        let _ = looks_like_secret_value(&input);
    }

    /// `redact_secret_shaped_substrings` must never panic on arbitrary
    /// input.
    #[test]
    fn prop_redact_secret_shaped_substrings_never_panics(input in ".{0,500}") {
        let _ = redact_secret_shaped_substrings(&input);
    }

    /// Redacting already-redacted text must be structurally a no-op
    /// (idempotency), for arbitrary garbled input.
    #[test]
    fn prop_redact_secret_shaped_substrings_is_idempotent_on_arbitrary_input(input in ".{0,300}") {
        let once = redact_secret_shaped_substrings(&input);
        let twice = redact_secret_shaped_substrings(&once);
        prop_assert_eq!(once, twice);
    }

    /// A GitHub-token-shaped secret embedded anywhere inside arbitrary
    /// surrounding prose must be removed from the output, regardless of
    /// where in the prose it is embedded or what the prose says.
    #[test]
    fn prop_redact_secret_shaped_substrings_removes_gh_token_embedded_in_prose(
        prefix in gh_token_prefix_strategy(),
        suffix in gh_token_suffix_strategy(),
        before in prose_strategy(),
        after in prose_strategy(),
    ) {
        let token = format!("{prefix}{suffix}");
        let text = format!("{before} {token} {after}");

        let redacted = redact_secret_shaped_substrings(&text);
        prop_assert!(
            !redacted.contains(&token),
            "embedded token '{token}' survived redaction in output: {redacted}"
        );
    }

    /// UUID-shaped strings must never be classified as secret-shaped,
    /// regardless of which random hex digits make up the UUID -- this is
    /// the property-level guarantee behind the "UUIDs must not be
    /// redacted" false-positive requirement.
    #[test]
    fn prop_looks_like_secret_value_uuid_shaped_strings_are_never_secret(uuid in uuid_like_strategy()) {
        prop_assert!(!looks_like_secret_value(&uuid));
    }

    /// A UUID embedded in arbitrary prose must survive
    /// `redact_secret_shaped_substrings` unchanged, regardless of which
    /// random hex digits make up the UUID.
    #[test]
    fn prop_redact_secret_shaped_substrings_never_redacts_embedded_uuid(
        uuid in uuid_like_strategy(),
        before in prose_strategy(),
        after in prose_strategy(),
    ) {
        let text = format!("{before} {uuid} {after}");
        let redacted = redact_secret_shaped_substrings(&text);
        prop_assert!(
            redacted.contains(&uuid),
            "UUID '{uuid}' must survive redaction unchanged in output: {redacted}"
        );
    }

    /// ISO-8601-shaped timestamps must never be classified as
    /// secret-shaped, regardless of the specific (valid-range) date/time
    /// components drawn.
    #[test]
    fn prop_looks_like_secret_value_iso_timestamp_shaped_strings_are_never_secret(
        timestamp in iso_timestamp_like_strategy(),
    ) {
        prop_assert!(!looks_like_secret_value(&timestamp));
    }

    /// Arbitrary prose built purely from the benign word vocabulary (no
    /// embedded secret-shaped substring by construction) must never be
    /// mutated by `redact_secret_shaped_substrings`.
    #[test]
    fn prop_redact_secret_shaped_substrings_never_mutates_pure_prose(prose in prose_strategy()) {
        prop_assert_eq!(redact_secret_shaped_substrings(&prose), prose);
    }
}
