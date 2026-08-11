//! Specification, adversarial, and contract tests for `crate::logging`.
//!
//! Tier 1 (specification): one test per behavioural rule stated in
//! `logging.rs`'s module doc comment.
//! Tier 2 (adversarial): boundary cases (case sensitivity, substring
//! collisions, non-string JSON value types, malformed input), stub-killing
//! tests, and side-effect verification (redaction happens / does not
//! over-redact).
//!
//! Property-based (Tier 3) tests live in `logging_proptests.rs`.

use super::*;
use std::io::Write;
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────────────────────────────────
// resolve_log_format — Tier 1 (specification)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_resolve_log_format_none_defaults_to_json() {
    assert_eq!(resolve_log_format(None), LogFormat::Json);
}

#[test]
fn test_resolve_log_format_pretty_lowercase_selects_pretty() {
    assert_eq!(resolve_log_format(Some("pretty")), LogFormat::Pretty);
}

#[test]
fn test_resolve_log_format_pretty_uppercase_selects_pretty() {
    assert_eq!(
        resolve_log_format(Some("PRETTY")),
        LogFormat::Pretty,
        "matching must be case-insensitive"
    );
}

#[test]
fn test_resolve_log_format_pretty_mixed_case_selects_pretty() {
    assert_eq!(resolve_log_format(Some("PreTTy")), LogFormat::Pretty);
}

#[test]
fn test_resolve_log_format_explicit_json_stays_json() {
    assert_eq!(resolve_log_format(Some("json")), LogFormat::Json);
}

// ─────────────────────────────────────────────────────────────────────────
// resolve_log_format — Tier 2 (adversarial / boundary / stub-killing)
// ─────────────────────────────────────────────────────────────────────────

/// A stub that always returns `Json` regardless of input would pass every
/// Tier 1 test above except `test_resolve_log_format_pretty_lowercase_selects_pretty`
/// (and its case variants). This test independently re-asserts the Pretty
/// branch is real dispatch, not a coincidental match, by pairing it with an
/// explicit Json-branch assertion in the same test.
#[test]
fn test_resolve_log_format_distinguishes_pretty_from_json_in_same_test() {
    assert_ne!(
        resolve_log_format(Some("pretty")),
        resolve_log_format(Some("json")),
        "Pretty and Json must be genuinely distinct outcomes, not a hardcoded constant return"
    );
}

#[test]
fn test_resolve_log_format_empty_string_defaults_to_json() {
    assert_eq!(
        resolve_log_format(Some("")),
        LogFormat::Json,
        "an empty (but present) LOG_FORMAT value must not panic or select Pretty"
    );
}

#[test]
fn test_resolve_log_format_unrecognised_value_defaults_to_json_not_panic() {
    assert_eq!(
        resolve_log_format(Some("verbose-please")),
        LogFormat::Json,
        "unrecognised values must fail safe to Json, never panic or silently select Pretty"
    );
}

#[test]
fn test_resolve_log_format_whitespace_only_value_defaults_to_json() {
    assert_eq!(resolve_log_format(Some("   ")), LogFormat::Json);
}

// ─────────────────────────────────────────────────────────────────────────
// is_sensitive_field_name — Tier 1 (specification: every named deny-list
// entry from the task description must be individually recognised)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_is_sensitive_field_name_token_is_sensitive() {
    assert!(is_sensitive_field_name("token"));
}

#[test]
fn test_is_sensitive_field_name_password_is_sensitive() {
    assert!(is_sensitive_field_name("password"));
}

#[test]
fn test_is_sensitive_field_name_secret_is_sensitive() {
    assert!(is_sensitive_field_name("secret"));
}

#[test]
fn test_is_sensitive_field_name_authorization_is_sensitive() {
    assert!(is_sensitive_field_name("authorization"));
}

#[test]
fn test_is_sensitive_field_name_github_token_is_sensitive() {
    assert!(is_sensitive_field_name("github_token"));
}

#[test]
fn test_is_sensitive_field_name_jwt_is_sensitive() {
    assert!(is_sensitive_field_name("jwt"));
}

#[test]
fn test_is_sensitive_field_name_api_key_is_sensitive() {
    assert!(is_sensitive_field_name("api_key"));
}

// ─────────────────────────────────────────────────────────────────────────
// is_sensitive_field_name — Tier 2 (adversarial / boundary / stub-killing)
// ─────────────────────────────────────────────────────────────────────────

/// Stub-killing: a stub that always returns `true` would pass every test
/// above. This test requires at least one common, clearly benign field name
/// used elsewhere in this codebase's structured logging
/// (`crates/repo_roller_api/src/middleware.rs`'s `tracing_middleware`) to be
/// classified as NOT sensitive.
#[test]
fn test_is_sensitive_field_name_request_id_is_not_sensitive() {
    assert!(
        !is_sensitive_field_name("request_id"),
        "request_id is the correlation ID this task adds -- it must never be redacted"
    );
}

#[test]
fn test_is_sensitive_field_name_method_is_not_sensitive() {
    assert!(!is_sensitive_field_name("method"));
}

#[test]
fn test_is_sensitive_field_name_status_is_not_sensitive() {
    assert!(!is_sensitive_field_name("status"));
}

#[test]
fn test_is_sensitive_field_name_message_is_not_sensitive() {
    assert!(!is_sensitive_field_name("message"));
}

#[test]
fn test_is_sensitive_field_name_user_login_is_not_sensitive() {
    assert!(!is_sensitive_field_name("user_login"));
}

#[test]
fn test_is_sensitive_field_name_matching_is_case_insensitive() {
    assert!(is_sensitive_field_name("TOKEN"));
    assert!(is_sensitive_field_name("Password"));
    assert!(is_sensitive_field_name("AUTHORIZATION"));
}

/// Boundary: a compound field name containing a deny-listed fragment as a
/// substring must still be caught (this is the documented, deliberately
/// over-redacting policy -- see the module contract's "Known spec gaps").
#[test]
fn test_is_sensitive_field_name_substring_match_catches_compound_names() {
    assert!(is_sensitive_field_name("db_password"));
    assert!(is_sensitive_field_name("x-authorization-header"));
    assert!(is_sensitive_field_name("user_jwt_token"));
}

/// Boundary: the empty string is not itself any deny-list entry and must
/// not panic or spuriously match.
#[test]
fn test_is_sensitive_field_name_empty_string_is_not_sensitive() {
    assert!(!is_sensitive_field_name(""));
}

/// Boundary: a single-character difference from a deny-listed word must NOT
/// match -- guards against an overly permissive fuzzy-match stub.
#[test]
fn test_is_sensitive_field_name_near_miss_is_not_sensitive() {
    assert!(
        !is_sensitive_field_name("tokeb"),
        "a near-miss that is not actually a substring match must not be flagged"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// redact_sensitive_fields — Tier 1 (specification)
// ─────────────────────────────────────────────────────────────────────────

/// A JSON log line shaped like `tracing_subscriber::fmt().json()`'s real
/// output: the event's own fields nested under `"fields"`, a `message` key
/// alongside them, and a `token` field that must never survive redaction.
fn realistic_json_line_with_secret() -> String {
    serde_json::json!({
        "timestamp": "2024-01-01T00:00:00.000000Z",
        "level": "INFO",
        "fields": {
            "message": "Backend JWT validated",
            "token": "ghp_superSecretValue123"
        },
        "target": "repo_roller_api::middleware"
    })
    .to_string()
}

#[test]
fn test_redact_sensitive_fields_removes_raw_token_value() {
    let redacted = redact_sensitive_fields(&realistic_json_line_with_secret());
    assert!(
        !redacted.contains("ghp_superSecretValue123"),
        "raw secret value must not survive redaction; got: {redacted}"
    );
}

#[test]
fn test_redact_sensitive_fields_inserts_redacted_marker() {
    let redacted = redact_sensitive_fields(&realistic_json_line_with_secret());
    let parsed: serde_json::Value =
        serde_json::from_str(&redacted).expect("redacted output must remain valid JSON");
    assert_eq!(
        parsed["fields"]["token"],
        serde_json::Value::String(REDACTED_MARKER.to_string())
    );
}

#[test]
fn test_redact_sensitive_fields_preserves_non_sensitive_fields() {
    let redacted = redact_sensitive_fields(&realistic_json_line_with_secret());
    let parsed: serde_json::Value =
        serde_json::from_str(&redacted).expect("redacted output must remain valid JSON");
    assert_eq!(parsed["fields"]["message"], "Backend JWT validated");
    assert_eq!(parsed["level"], "INFO");
    assert_eq!(parsed["target"], "repo_roller_api::middleware");
}

#[test]
fn test_redact_sensitive_fields_non_json_input_passed_through_unchanged() {
    let pretty_line = "2024-01-01T00:00:00Z  INFO repo_roller_api: Request started";
    assert_eq!(redact_sensitive_fields(pretty_line), pretty_line);
}

// ─────────────────────────────────────────────────────────────────────────
// redact_sensitive_fields — Tier 2 (adversarial / boundary / stub-killing)
// ─────────────────────────────────────────────────────────────────────────

/// Stub-killing: a stub that always returns the exact input unchanged would
/// pass the non-JSON-passthrough test above but must fail here.
#[test]
fn test_redact_sensitive_fields_actually_mutates_lines_containing_secrets() {
    let line = realistic_json_line_with_secret();
    assert_ne!(
        redact_sensitive_fields(&line),
        line,
        "a line containing a sensitive field must not be returned unchanged"
    );
}

/// Stub-killing: a stub that always returns an empty object / constant
/// string would fail to preserve non-sensitive content.
#[test]
fn test_redact_sensitive_fields_does_not_over_redact_field_free_of_secrets() {
    let line = serde_json::json!({
        "timestamp": "2024-01-01T00:00:00.000000Z",
        "level": "INFO",
        "fields": { "message": "Request completed", "status": 200 },
        "target": "repo_roller_api::middleware"
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(parsed["fields"]["message"], "Request completed");
    assert_eq!(parsed["fields"]["status"], 200);
}

/// Adversarial: the deny-list must match on FIELD NAMES, not scan prose
/// values for keywords. A `message` field whose text happens to mention
/// "token" must survive verbatim.
#[test]
fn test_redact_sensitive_fields_does_not_scan_message_prose_for_keywords() {
    let line = serde_json::json!({
        "timestamp": "2024-01-01T00:00:00.000000Z",
        "level": "WARN",
        "fields": { "message": "user token expired, please re-authenticate" },
        "target": "repo_roller_api::middleware"
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(
        parsed["fields"]["message"], "user token expired, please re-authenticate",
        "field VALUES must never be scanned for deny-listed words, only field NAMES"
    );
}

/// Adversarial: multiple distinct sensitive fields in the same line must
/// all be redacted, not just the first one encountered.
#[test]
fn test_redact_sensitive_fields_redacts_multiple_sensitive_fields_at_once() {
    let line = serde_json::json!({
        "fields": {
            "message": "token exchange",
            "token": "raw-token-value",
            "password": "raw-password-value",
            "api_key": "raw-api-key-value"
        }
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    assert!(!redacted.contains("raw-token-value"));
    assert!(!redacted.contains("raw-password-value"));
    assert!(!redacted.contains("raw-api-key-value"));

    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(parsed["fields"]["token"], REDACTED_MARKER);
    assert_eq!(parsed["fields"]["password"], REDACTED_MARKER);
    assert_eq!(parsed["fields"]["api_key"], REDACTED_MARKER);
}

/// Adversarial / boundary: sensitive fields nested deeper than one level
/// (e.g. under a span's recorded fields) must also be redacted -- guards
/// against a stub that only inspects the top-level object or only the
/// `"fields"` key by name.
#[test]
fn test_redact_sensitive_fields_recurses_into_nested_span_object() {
    let line = serde_json::json!({
        "fields": { "message": "processing" },
        "span": { "name": "http_request", "authorization": "Bearer raw-jwt-value" },
        "spans": [{ "name": "http_request", "authorization": "Bearer raw-jwt-value" }]
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    assert!(
        !redacted.contains("raw-jwt-value"),
        "nested sensitive fields (span/spans) must be redacted too"
    );
}

/// Boundary: redaction must apply regardless of the JSON value's type, not
/// just strings -- a stub that only handles `Value::String` would leak a
/// numeric, null, or structured secret value.
#[test]
fn test_redact_sensitive_fields_redacts_non_string_value_types() {
    let line = serde_json::json!({
        "fields": {
            "secret": 123456789,
            "jwt": null,
            "api_key": ["part-a", "part-b"]
        }
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    assert!(!redacted.contains("123456789"));
    assert!(!redacted.contains("part-a"));
    assert!(!redacted.contains("part-b"));

    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(parsed["fields"]["secret"], REDACTED_MARKER);
    assert_eq!(parsed["fields"]["jwt"], REDACTED_MARKER);
    assert_eq!(parsed["fields"]["api_key"], REDACTED_MARKER);
}

/// Adversarial: values containing JSON-significant characters (quotes,
/// backslashes) must not corrupt the output -- guards against a naive
/// string-replace/regex implementation instead of a real JSON parse +
/// re-serialize.
#[test]
fn test_redact_sensitive_fields_handles_values_with_special_characters() {
    let line = serde_json::json!({
        "fields": {
            "message": "contains \"quotes\" and a \\backslash",
            "token": "value-with-\"quote\"-and-\\backslash"
        }
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    let parsed: serde_json::Value = serde_json::from_str(&redacted)
        .expect("output must remain valid, parseable JSON even with escaped characters");
    assert_eq!(
        parsed["fields"]["message"],
        "contains \"quotes\" and a \\backslash"
    );
    assert_eq!(parsed["fields"]["token"], REDACTED_MARKER);
}

/// Idempotency: redacting an already-redacted line must be a no-op
/// (structurally) -- guards against double-escaping or marker-mangling on
/// repeated application.
#[test]
fn test_redact_sensitive_fields_is_idempotent() {
    let line = realistic_json_line_with_secret();
    let once = redact_sensitive_fields(&line);
    let twice = redact_sensitive_fields(&once);

    let v1: serde_json::Value = serde_json::from_str(&once).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&twice).unwrap();
    assert_eq!(v1, v2);
}

/// Adversarial: malformed JSON (truncated) must not panic and must be
/// returned unchanged rather than crashing the logging pipeline.
#[test]
fn test_redact_sensitive_fields_truncated_json_does_not_panic() {
    let truncated = r#"{"fields": {"token": "abc"#;
    let result = redact_sensitive_fields(truncated);
    assert_eq!(result, truncated);
}

/// Adversarial: an empty string input must not panic.
#[test]
fn test_redact_sensitive_fields_empty_string_does_not_panic() {
    assert_eq!(redact_sensitive_fields(""), "");
}

// ─────────────────────────────────────────────────────────────────────────
// build_sanitizing_json_subscriber — Contract tests
// ─────────────────────────────────────────────────────────────────────────
//
// Contract under test: any subscriber built via
// `build_sanitizing_json_subscriber` must, for every emitted `tracing`
// event:
//   1. Never allow a sensitive field's raw value to reach the writer.
//   2. Preserve non-sensitive fields and the event message unchanged.
//   3. Produce output that is still valid, parseable JSON per line.
// This mirrors how a repository/hasher trait's contract tests would be
// written -- these three properties must hold for the subscriber regardless
// of internal implementation choice (custom `MakeWriter` vs. custom
// `FormatFields` vs. a `Layer`).

/// In-memory writer used only by these tests to capture subscriber output
/// without touching stdout/stderr.
#[derive(Clone, Default)]
struct CaptureBuffer(Arc<Mutex<Vec<u8>>>);

impl CaptureBuffer {
    fn contents(&self) -> String {
        String::from_utf8_lossy(
            &self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
        .to_string()
    }
}

impl Write for CaptureBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureBuffer {
    type Writer = CaptureBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn test_configured_subscriber_never_leaks_raw_sensitive_value() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_sanitizing_json_subscriber(buffer.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(token = "leak-me-if-you-can", "token exchange succeeded");
    }

    let output = buffer.contents();
    assert!(
        !output.contains("leak-me-if-you-can"),
        "raw sensitive value must never reach the configured subscriber's writer; got: {output}"
    );
    assert!(
        output.contains(REDACTED_MARKER),
        "the redaction marker must appear in place of the sensitive value; got: {output}"
    );
}

#[test]
fn test_configured_subscriber_preserves_non_sensitive_fields() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_sanitizing_json_subscriber(buffer.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(user_login = "alice", "user authenticated");
    }

    let output = buffer.contents();
    assert!(
        output.contains("alice"),
        "non-sensitive field values must survive through the sanitizing subscriber; got: {output}"
    );
    assert!(
        output.contains("user authenticated"),
        "the log message itself must survive through the sanitizing subscriber; got: {output}"
    );
}

#[test]
fn test_configured_subscriber_output_is_valid_json_per_line() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_sanitizing_json_subscriber(buffer.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(password = "hunter2", "login attempt");
    }

    let output = buffer.contents();
    for line in output.lines().filter(|l| !l.trim().is_empty()) {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("sanitized log line was not valid JSON: {line:?}: {e}"));
    }
}

/// Adversarial: does not over-redact -- an event with NO sensitive fields at
/// all must produce output identical in content to what an unsanitized JSON
/// subscriber would produce (message + non-sensitive fields all present).
// ─────────────────────────────────────────────────────────────────────────
// QA adversarial pass (manual — no fuzz harness exists for this crate; see
// audit report). Findings pinned as regression tests below.
// ─────────────────────────────────────────────────────────────────────────

/// Adversarial: pathologically deep JSON nesting (500 levels) must not
/// panic or stack-overflow `redact_value_in_place`'s recursion. `serde_json`
/// itself rejects input past its internal recursion limit (128 by default)
/// with a parse error rather than recursing further, so this exercises the
/// documented non-JSON passthrough path for input that is syntactically
/// valid JSON but too deep to parse.
#[test]
fn test_redact_sensitive_fields_pathologically_deep_nesting_does_not_panic() {
    let mut nested = String::from("1");
    for _ in 0..500 {
        nested = format!("{{\"a\":{nested}}}");
    }
    let result = std::panic::catch_unwind(|| redact_sensitive_fields(&nested));
    assert!(result.is_ok(), "500-level-deep nested JSON must not panic");
}

/// Adversarial: a large (50k-element) JSON array must be processed without
/// panicking and must still redact every sensitive field within it (no
/// truncation / early-exit on large input).
#[test]
fn test_redact_sensitive_fields_large_array_processes_fully() {
    let arr: Vec<_> = (0..50_000)
        .map(|i| serde_json::json!({"token": i}))
        .collect();
    let line = serde_json::json!({"fields": arr}).to_string();

    let redacted = redact_sensitive_fields(&line);
    let parsed: serde_json::Value =
        serde_json::from_str(&redacted).expect("output must remain valid JSON");
    let fields = parsed["fields"].as_array().expect("fields must be an array");
    assert_eq!(fields.len(), 50_000, "no elements must be dropped");
    assert!(
        fields
            .iter()
            .all(|v| v["token"] == serde_json::Value::String(REDACTED_MARKER.to_string())),
        "every element's token field must be redacted, not just the first/last"
    );
}

/// KNOWN GAP (documented, not fixed by this audit -- flagged for Tech
/// Lead / Security Reviewer decision): `is_sensitive_field_name`'s
/// substring match operates on the raw lowercase string and does not
/// perform Unicode confusable/homoglyph normalization. A field name using
/// full-width, Cyrillic-substituted, or other visually-similar-but-
/// codepoint-distinct characters in place of ASCII letters evades the
/// deny-list even though it would render as a near-identical word.
///
/// This is a real residual risk only if a field *name* (not value) can
/// ever be attacker-influenced (e.g. dynamically-keyed data logged
/// verbatim) rather than fixed at the `tracing::info!(name = ...)`
/// call site. Pinned here as CURRENT behaviour so a future change is
/// visible in the diff, not silently reverted.
#[test]
fn test_is_sensitive_field_name_unicode_homoglyph_evasion_known_gap() {
    assert!(is_sensitive_field_name("TOKEN"));
    assert!(
        !is_sensitive_field_name("\u{ff54}\u{ff4f}\u{ff4b}\u{ff45}\u{ff4e}"), // fullwidth "token"
        "KNOWN GAP: full-width Unicode 'token' is not currently detected as sensitive"
    );
    assert!(
        !is_sensitive_field_name("p\u{0430}ssword"), // Cyrillic 'а' (U+0430) substituted for Latin 'a'
        "KNOWN GAP: Cyrillic-homoglyph-substituted 'password' is not currently detected as sensitive"
    );
}

/// Adversarial / documents-by-contract: redaction matches JSON object
/// *keys*, never values, and only within an object. If the entire "log
/// line" happens to be a bare JSON string (not an object) -- which never
/// happens for real `tracing_subscriber::fmt().json()` output, but is
/// worth pinning explicitly -- no redaction occurs because there is no key
/// to match against. This is consistent with, not a violation of, the
/// documented "field names are matched, never field values" contract.
#[test]
fn test_redact_sensitive_fields_top_level_json_string_is_not_scanned_for_secrets() {
    let line = serde_json::Value::String("token=abc123secret".to_string()).to_string();
    let redacted = redact_sensitive_fields(&line);
    assert_eq!(
        redacted, line,
        "a bare top-level JSON string has no object key to redact and must pass through \
         unchanged; production output is always an object so this path is not reachable \
         via the real subscriber, but is pinned here as documented behaviour"
    );
}

/// SIGNIFICANT FINDING (documented in audit report, not fixed by this audit
/// -- requires a `SanitizingWriter` redesign, out of QA's write-tests-only
/// mandate; escalate to Tech Lead / Security Reviewer):
///
/// `SanitizingWriter`'s entire redaction guarantee depends on the
/// undocumented-at-the-type-level runtime assumption (stated in its own doc
/// comment) that `tracing_subscriber::fmt`'s formatter always delivers one
/// complete JSON event (body + trailing newline) via exactly one `write()`
/// call. This test simulates that assumption being violated -- e.g. by a
/// different `MakeWriter` composition, a future `tracing-subscriber`
/// version, or any writer that batches/splits differently -- by manually
/// splitting a single formatted event across two `write()` calls, mid
/// multi-byte UTF-8 character.
///
/// Result: neither fragment is valid JSON on its own, so
/// `redact_sensitive_fields`'s documented non-JSON passthrough fires for
/// BOTH fragments, and the raw sensitive value reaches the inner writer
/// **unredacted** (mangled by lossy UTF-8 replacement, but the field name
/// and most of the secret's bytes are still present in the clear). This is
/// a silent sanitization bypass, not a panic -- worse, from a security
/// standpoint, than a crash. Currently non-exploitable in production
/// because `init_tracing` only ever wraps `std::io::stdout` directly (no
/// `tracing_appender::non_blocking` or other batching writer is used
/// anywhere in this crate as of this audit -- confirmed via workspace
/// search), but the guarantee is fragile and version-dependent.
#[test]
fn test_sanitizing_writer_split_write_across_utf8_boundary_leaks_secret_known_gap() {
    let inner: Vec<u8> = Vec::new();
    let mut writer = SanitizingWriter { inner };
    let full = "{\"fields\":{\"token\":\"s\u{e9}cret\"}}\n".as_bytes().to_vec();

    // Split mid-way through the 2-byte UTF-8 encoding of 'é' (0xC3 0xA9),
    // simulating a writer that does not deliver the whole line in one call.
    let split_point = full.iter().position(|&b| b == 0xC3).unwrap() + 1;
    let (part1, part2) = full.split_at(split_point);

    writer.write_all(part1).expect("write must not error");
    writer.write_all(part2).expect("write must not error");

    let out = String::from_utf8_lossy(&writer.inner).to_string();
    assert!(
        out.contains("token"),
        "KNOWN GAP: under a split write, the sensitive field name and a mangled \
         remnant of its raw value currently reach the writer WITHOUT redaction \
         (marker not applied) because each fragment fails to parse as JSON on its \
         own; got: {out:?}"
    );
    assert!(
        !out.contains(REDACTED_MARKER),
        "confirms the redaction marker is NOT applied under this failure mode \
         (i.e. this documents a bypass, not a successful redaction): {out:?}"
    );
}

/// Adversarial: single-character and empty-ish malformed inputs must never
/// panic, regardless of whether they happen to be valid JSON primitives
/// (e.g. `"1"`) or invalid fragments.
#[test]
fn test_redact_sensitive_fields_single_char_and_empty_inputs_never_panic() {
    for input in ["", "1", "\"", "n", "{", "}", "[", "]", ":", ","] {
        let result = std::panic::catch_unwind(|| redact_sensitive_fields(input));
        assert!(result.is_ok(), "panicked on input {input:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Mutant kill tests (QA audit, Task #1)
// ─────────────────────────────────────────────────────────────────────────
//
// The following tests target specific `cargo mutants` survivors found while
// auditing this module. Each documents which mutant it kills.

/// An inner writer that records whether `flush()` was called on it, so a
/// test can observe `SanitizingWriter::flush` actually forwarding to the
/// wrapped writer rather than being a no-op.
#[derive(Clone, Default)]
struct FlushTrackingWriter {
    flushed: Arc<Mutex<bool>>,
}

impl Write for FlushTrackingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        *self.flushed.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        Ok(())
    }
}

/// ### Survivor killed
/// `logging.rs`: `<impl Write for SanitizingWriter<W>>::flush` body replaced
/// with `Ok(())`, discarding the call to `self.inner.flush()`. No existing
/// test observed whether `flush()` propagated to the inner writer, so the
/// mutant survived. This test wraps a writer that records whether its own
/// `flush()` was invoked and asserts `SanitizingWriter::flush()` really
/// forwards the call.
#[test]
fn test_sanitizing_writer_flush_forwards_to_inner_writer() {
    let tracker = FlushTrackingWriter::default();
    let mut writer = SanitizingWriter {
        inner: tracker.clone(),
    };

    Write::flush(&mut writer).expect("flush must succeed");

    assert!(
        *tracker.flushed.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
        "SanitizingWriter::flush must forward to the inner writer's flush()"
    );
}

/// ### Survivor killed
/// `logging.rs`: `init_tracing`'s entire body (the format-dispatch `match`)
/// replaced with `()`, so the function silently does nothing instead of
/// installing a process-global subscriber. No existing test called
/// `init_tracing` directly (it is a one-shot, process-global side effect),
/// so the mutant survived.
///
/// This test exploits `init_tracing`'s own documented panic-on-double-init
/// behaviour (mirroring `tracing_subscriber`'s `...::init()`): the real
/// implementation installs a global default subscriber on its first call,
/// so a second call must panic. Under the `()` mutant, neither call does
/// anything, so no panic is ever observed and this test fails -- correctly
/// flagging the mutant. No other test in this binary installs a global
/// default subscriber (see `logging.rs` / `middleware_tests.rs`, which all
/// use the thread-local `tracing::subscriber::set_default` guard instead),
/// so the first call below is expected to succeed exactly once for the
/// life of the test binary.
#[test]
fn test_init_tracing_second_call_panics_on_double_init() {
    // Absorb the first call's outcome (expected: succeeds and installs the
    // global default). Wrapped in `catch_unwind` defensively in case some
    // other global state in the test binary already caused an installation
    // -- either way, the *second* call below must observe an
    // already-installed global default and panic.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(init_tracing));

    let second_call = std::panic::catch_unwind(std::panic::AssertUnwindSafe(init_tracing));
    assert!(
        second_call.is_err(),
        "init_tracing() must panic on a second call once a global default subscriber \
         is already installed; a no-op mutant would let both calls succeed silently"
    );
}

#[test]
fn test_configured_subscriber_does_not_redact_when_nothing_sensitive_is_present() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_sanitizing_json_subscriber(buffer.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!(status = 200, method = "GET", "request completed");
    }

    let output = buffer.contents();
    assert!(
        !output.contains(REDACTED_MARKER),
        "no field was sensitive; the marker must not appear at all: {output}"
    );
    assert!(output.contains("200"));
    assert!(output.contains("GET"));
}

// ─────────────────────────────────────────────────────────────────────────
// looks_like_secret_value — Tier 1 (specification: true cases)
//
// Security remediation: HIGH-severity finding -- key-name redaction does
// not catch a secret embedded in free text via `{:?}`/`{}` interpolation.
// See `crates/config_manager/src/github_metadata_provider.rs:626`.
// ─────────────────────────────────────────────────────────────────────────

/// A realistic-length GitHub PAT suffix (36 chars, GitHub's real length)
/// used to build whole secret-shaped strings across this test module.
const GH_TOKEN_SUFFIX_36: &str = "abcdefghij1234567890ABCDEFGHIJ123456";

#[test]
fn test_looks_like_secret_value_ghp_prefixed_token_is_secret() {
    let value = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_gho_prefixed_token_is_secret() {
    let value = format!("gho_{GH_TOKEN_SUFFIX_36}");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_ghs_prefixed_token_is_secret() {
    let value = format!("ghs_{GH_TOKEN_SUFFIX_36}");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_ghu_prefixed_token_is_secret() {
    let value = format!("ghu_{GH_TOKEN_SUFFIX_36}");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_ghr_prefixed_token_is_secret() {
    let value = format!("ghr_{GH_TOKEN_SUFFIX_36}");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_github_pat_prefixed_token_is_secret() {
    let value = format!("github_pat_{GH_TOKEN_SUFFIX_36}extra_suffix_chars_1234567890");
    assert!(looks_like_secret_value(&value));
}

#[test]
fn test_looks_like_secret_value_bearer_header_shape_is_secret() {
    assert!(looks_like_secret_value("Bearer abcdefgh12345678"));
}

#[test]
fn test_looks_like_secret_value_lowercase_bearer_header_shape_is_secret() {
    assert!(looks_like_secret_value("bearer abcdefgh12345678"));
}

#[test]
fn test_looks_like_secret_value_jwt_shape_is_secret() {
    // A real (expired, non-secret) example JWT shape: header.payload.signature.
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    assert!(looks_like_secret_value(jwt));
}

#[test]
fn test_looks_like_secret_value_generic_high_entropy_token_is_secret() {
    // 32 chars, mixed upper/lower/digit, alphabet restricted to [A-Za-z0-9+/].
    let token = "aB3xZ9kLm2QpR7sT1vW8yU4nC6dF0gH5";
    assert_eq!(token.chars().count(), 32);
    assert!(looks_like_secret_value(token));
}

// ─────────────────────────────────────────────────────────────────────────
// looks_like_secret_value — Tier 1 (specification: false-positive guards)
//
// Explicitly required by the security remediation task: ordinary prose,
// short strings, common words, URLs, UUIDs, and ISO timestamps must never
// be flagged, or the redaction layer would make logs useless.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_looks_like_secret_value_empty_string_is_not_secret() {
    assert!(!looks_like_secret_value(""));
}

#[test]
fn test_looks_like_secret_value_ordinary_word_is_not_secret() {
    assert!(!looks_like_secret_value("hello"));
}

#[test]
fn test_looks_like_secret_value_ordinary_sentence_is_not_secret() {
    assert!(!looks_like_secret_value("the quick brown fox jumps"));
}

#[test]
fn test_looks_like_secret_value_short_alphanumeric_string_is_not_secret() {
    assert!(!looks_like_secret_value("abc123"));
}

#[test]
fn test_looks_like_secret_value_url_is_not_secret() {
    assert!(!looks_like_secret_value(
        "https://example.com/path/to/resource?query=value"
    ));
}

#[test]
fn test_looks_like_secret_value_uuid_is_not_secret() {
    assert!(!looks_like_secret_value("550e8400-e29b-41d4-a716-446655440000"));
}

#[test]
fn test_looks_like_secret_value_iso_timestamp_is_not_secret() {
    assert!(!looks_like_secret_value("2024-01-01T00:00:00.000000Z"));
}

/// Git commit SHAs (40 lowercase hex chars) are common, benign log content
/// and must not be flagged by the generic high-entropy heuristic -- they
/// fail the "must contain an uppercase letter" requirement.
#[test]
fn test_looks_like_secret_value_git_commit_sha_is_not_secret() {
    assert!(!looks_like_secret_value(
        "d5a2058e2bf054263371b7fac126c1dfcf7f9af"
    ));
}

// ─────────────────────────────────────────────────────────────────────────
// looks_like_secret_value — Tier 2 (adversarial / boundary / stub-killing)
// ─────────────────────────────────────────────────────────────────────────

/// Stub-killing: a stub that always returns `true` would pass every "true
/// case" test above but must fail every false-positive-guard test above.
/// Stub-killing: a stub that always returns `false` would pass every
/// false-positive-guard test but must fail every "true case" test above.
/// This test independently re-asserts both outcomes are genuinely distinct
/// within a single test.
#[test]
fn test_looks_like_secret_value_distinguishes_secret_from_ordinary_word_in_same_test() {
    let secret = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    assert_ne!(
        looks_like_secret_value(&secret),
        looks_like_secret_value("hello"),
        "a real token and an ordinary word must not classify the same way"
    );
}

/// Boundary: the GitHub-prefix suffix threshold is 20 chars. 19 chars must
/// NOT match (N-1), 20 chars MUST match (N).
#[test]
fn test_looks_like_secret_value_ghp_suffix_boundary_n_minus_1_is_not_secret() {
    let value = format!("ghp_{}", &GH_TOKEN_SUFFIX_36[..19]);
    assert!(
        !looks_like_secret_value(&value),
        "a 19-char suffix is one short of the 20-char threshold and must not match"
    );
}

#[test]
fn test_looks_like_secret_value_ghp_suffix_boundary_n_is_secret() {
    let value = format!("ghp_{}", &GH_TOKEN_SUFFIX_36[..20]);
    assert!(
        looks_like_secret_value(&value),
        "a 20-char suffix meets the threshold exactly and must match"
    );
}

#[test]
fn test_looks_like_secret_value_ghp_suffix_boundary_n_plus_1_is_secret() {
    let value = format!("ghp_{}", &GH_TOKEN_SUFFIX_36[..21]);
    assert!(looks_like_secret_value(&value));
}

/// Boundary: a bare prefix with no suffix at all must not match.
#[test]
fn test_looks_like_secret_value_bare_ghp_prefix_alone_is_not_secret() {
    assert!(!looks_like_secret_value("ghp_"));
}

/// Boundary: `Bearer` with no token attached must not match.
#[test]
fn test_looks_like_secret_value_bearer_with_no_token_is_not_secret() {
    assert!(!looks_like_secret_value("Bearer "));
    assert!(!looks_like_secret_value("Bearer"));
}

/// Boundary: JWT shape requires exactly three dot-separated segments; two
/// segments (a decimal-like `a.b` shape, e.g. a version number) must not
/// match.
#[test]
fn test_looks_like_secret_value_two_segment_string_is_not_jwt_shaped() {
    assert!(!looks_like_secret_value(
        "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"
    ));
}

/// Boundary: a JWT-shaped string whose segments are too short (< 10 chars
/// each) must not match -- guards against a naive "contains two dots"
/// implementation.
#[test]
fn test_looks_like_secret_value_short_segment_dotted_string_is_not_jwt_shaped() {
    assert!(!looks_like_secret_value("a.b.c"));
    assert!(!looks_like_secret_value("1.2.3"));
}

/// Boundary: the generic high-entropy threshold is 32 chars. 31 chars must
/// NOT match (N-1), 32 chars MUST match (N), provided mixed case + digit
/// are present.
#[test]
fn test_looks_like_secret_value_high_entropy_boundary_n_minus_1_is_not_secret() {
    let token = "aB3xZ9kLm2QpR7sT1vW8yU4nC6dF0gH"; // 31 chars
    assert_eq!(token.chars().count(), 31);
    assert!(!looks_like_secret_value(token));
}

#[test]
fn test_looks_like_secret_value_high_entropy_boundary_n_is_secret() {
    let token = "aB3xZ9kLm2QpR7sT1vW8yU4nC6dF0gH5"; // 32 chars
    assert_eq!(token.chars().count(), 32);
    assert!(looks_like_secret_value(token));
}

/// Boundary: a 32+ char run that is entirely lowercase (no uppercase, no
/// digit) must not match the generic high-entropy heuristic -- guards
/// against a length-only implementation that would flag long ordinary
/// words/sentences-without-spaces.
#[test]
fn test_looks_like_secret_value_long_all_lowercase_run_is_not_secret() {
    let value = "thisisaveryverylongwordwithnouppercaseordigits";
    assert!(value.chars().count() >= 32);
    assert!(!looks_like_secret_value(value));
}

/// Adversarial: does not panic on pathologically long input.
#[test]
fn test_looks_like_secret_value_very_long_input_does_not_panic() {
    let long_input = "x".repeat(100_000);
    let result = std::panic::catch_unwind(|| looks_like_secret_value(&long_input));
    assert!(result.is_ok());
}

/// Adversarial: does not panic on non-ASCII unicode input.
#[test]
fn test_looks_like_secret_value_unicode_input_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        looks_like_secret_value("ghp_日本語テキストsecret値123456789012345")
    });
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────────────────
// redact_secret_shaped_substrings — Tier 1 (specification)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_redact_secret_shaped_substrings_removes_ghp_token_embedded_in_prose() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let text = format!("Successfully loaded template configuration: token={token}");

    let redacted = redact_secret_shaped_substrings(&text);
    assert!(
        !redacted.contains(&token),
        "raw token must not survive redaction; got: {redacted}"
    );
}

#[test]
fn test_redact_secret_shaped_substrings_inserts_marker_in_place_of_embedded_token() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let text = format!("Successfully loaded template configuration: token={token}");

    let redacted = redact_secret_shaped_substrings(&text);
    assert!(
        redacted.contains(REDACTED_MARKER),
        "the redaction marker must appear in place of the embedded token; got: {redacted}"
    );
}

#[test]
fn test_redact_secret_shaped_substrings_preserves_surrounding_prose() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let text = format!("Successfully loaded template configuration: token={token}");

    let redacted = redact_secret_shaped_substrings(&text);
    assert!(
        redacted.starts_with("Successfully loaded template configuration: token="),
        "text surrounding the embedded secret must survive unchanged; got: {redacted}"
    );
}

#[test]
fn test_redact_secret_shaped_substrings_removes_embedded_bearer_jwt() {
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let text = format!("Authorization: Bearer {jwt}");

    let redacted = redact_secret_shaped_substrings(&text);
    assert!(!redacted.contains(jwt));
}

#[test]
fn test_redact_secret_shaped_substrings_text_without_secrets_is_unchanged() {
    let text = "Successfully loaded template configuration for org/repo";
    assert_eq!(redact_secret_shaped_substrings(text), text);
}

// ─────────────────────────────────────────────────────────────────────────
// redact_secret_shaped_substrings — Tier 2 (adversarial / boundary /
// stub-killing)
// ─────────────────────────────────────────────────────────────────────────

/// Stub-killing: a stub that always returns the input unchanged would pass
/// the "no secrets" test above but must fail here.
#[test]
fn test_redact_secret_shaped_substrings_actually_mutates_text_containing_a_secret() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let text = format!("token={token}");
    assert_ne!(redact_secret_shaped_substrings(&text), text);
}

/// Stub-killing: a stub that always returns a constant/empty string would
/// fail to preserve prose that contains no secret.
#[test]
fn test_redact_secret_shaped_substrings_does_not_over_redact_secret_free_prose() {
    let text = "Request completed successfully in 42 milliseconds for user alice";
    assert_eq!(redact_secret_shaped_substrings(text), text);
}

/// Adversarial: multiple distinct embedded secrets in the same text must
/// all be redacted, not just the first one encountered.
#[test]
fn test_redact_secret_shaped_substrings_redacts_multiple_embedded_secrets() {
    let token_a = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let token_b = format!("gho_{GH_TOKEN_SUFFIX_36}");
    let text = format!("first={token_a} second={token_b}");

    let redacted = redact_secret_shaped_substrings(&text);
    assert!(!redacted.contains(&token_a));
    assert!(!redacted.contains(&token_b));
}

/// Adversarial (false-positive guard at substring-scan level): a UUID and
/// an ISO timestamp embedded in otherwise-ordinary prose must survive
/// unchanged -- this is the concrete scenario a log line like "Request
/// <uuid> completed at <timestamp>" exercises.
#[test]
fn test_redact_secret_shaped_substrings_does_not_redact_embedded_uuid_or_timestamp() {
    let text =
        "Request 550e8400-e29b-41d4-a716-446655440000 completed at 2024-01-01T00:00:00.000000Z";
    assert_eq!(redact_secret_shaped_substrings(text), text);
}

/// Idempotency: redacting already-redacted text must be a no-op.
#[test]
fn test_redact_secret_shaped_substrings_is_idempotent() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let text = format!("token={token}");

    let once = redact_secret_shaped_substrings(&text);
    let twice = redact_secret_shaped_substrings(&once);
    assert_eq!(once, twice);
}

/// Adversarial: empty string input must not panic.
#[test]
fn test_redact_secret_shaped_substrings_empty_string_does_not_panic() {
    assert_eq!(redact_secret_shaped_substrings(""), "");
}

/// Adversarial: does not panic on pathologically long or unicode input.
#[test]
fn test_redact_secret_shaped_substrings_long_unicode_input_does_not_panic() {
    let long_input = format!("日本語 {} 中文テキスト", "x".repeat(50_000));
    let result = std::panic::catch_unwind(|| redact_secret_shaped_substrings(&long_input));
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────────────────
// redact_sensitive_fields — extended Tier 1/2: value-shaped redaction
// integrated into the JSON pipeline (the concrete security remediation
// regression tests).
// ─────────────────────────────────────────────────────────────────────────

/// This is the concrete scenario from the Security Reviewer's HIGH-severity
/// finding: `crates/config_manager/src/github_metadata_provider.rs:626`
/// logs `tracing::debug!("...: {:?}", config)`, dumping an entire untrusted
/// `TemplateConfig` at DEBUG level under the `message` field -- a field
/// name that key-based redaction correctly leaves alone. A raw GitHub token
/// embedded in that dumped text must still be redacted.
#[test]
fn test_redact_sensitive_fields_redacts_secret_shaped_value_embedded_in_message_field() {
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");
    let line = serde_json::json!({
        "timestamp": "2024-01-01T00:00:00.000000Z",
        "level": "DEBUG",
        "fields": {
            "message": format!(
                "Successfully loaded template configuration: TemplateConfig {{ token: \"{token}\" }}"
            )
        },
        "target": "config_manager::github_metadata_provider"
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    assert!(
        !redacted.contains(&token),
        "a secret-shaped substring embedded in a non-sensitively-named field's \
         text must still be redacted; got: {redacted}"
    );
}

/// The same scenario, but the secret is embedded in a field with an
/// unrelated, non-deny-listed name (`description`) rather than `message` --
/// value-shaped redaction must apply regardless of which key holds the
/// string.
#[test]
fn test_redact_sensitive_fields_redacts_jwt_embedded_in_non_sensitive_named_field() {
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let line = serde_json::json!({
        "fields": {
            "description": format!("template default value: {jwt}")
        }
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    assert!(!redacted.contains(jwt));
}

/// False-positive guard at the full pipeline level: a UUID embedded in the
/// `message` field must survive unchanged -- value-shaped redaction must
/// not make ordinary log lines useless.
#[test]
fn test_redact_sensitive_fields_does_not_redact_uuid_embedded_in_message() {
    let line = serde_json::json!({
        "fields": {
            "message": "Request 550e8400-e29b-41d4-a716-446655440000 completed"
        }
    })
    .to_string();

    let redacted = redact_sensitive_fields(&line);
    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(
        parsed["fields"]["message"],
        "Request 550e8400-e29b-41d4-a716-446655440000 completed"
    );
}

/// Regression guard: adding value-shaped redaction must not break the
/// original key-name-based redaction contract established before this
/// remediation.
#[test]
fn test_redact_sensitive_fields_still_redacts_key_matched_fields_after_value_level_addition() {
    let redacted = redact_sensitive_fields(&realistic_json_line_with_secret());
    let parsed: serde_json::Value = serde_json::from_str(&redacted).unwrap();
    assert_eq!(
        parsed["fields"]["token"],
        serde_json::Value::String(REDACTED_MARKER.to_string())
    );
}

// ─────────────────────────────────────────────────────────────────────────
// build_sanitizing_json_subscriber — extended contract test: the exact
// vulnerable idiom (`tracing::debug!("...: {:?}", value)`) must not leak a
// secret-shaped value through the configured subscriber.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn test_configured_subscriber_redacts_secret_shaped_value_inside_formatted_debug_message() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_sanitizing_json_subscriber(buffer.clone());
    let token = format!("ghp_{GH_TOKEN_SUFFIX_36}");

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        // Mirrors the vulnerable call site's idiom: a struct's Debug output
        // interpolated directly into a free-text message.
        #[derive(Debug)]
        struct FakeTemplateConfig<'a> {
            token: &'a str,
        }
        let config = FakeTemplateConfig { token: &token };
        tracing::debug!("Successfully loaded template configuration: {:?}", config);
    }

    let output = buffer.contents();
    eprintln!("DEBUG PROBE 2 output = {output:?}");
    assert!(
        !output.contains(&token),
        "a secret-shaped value dumped via {{:?}} interpolation into the \
         message must never reach the configured subscriber's writer; got: {output}"
    );
}
