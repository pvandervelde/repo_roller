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
