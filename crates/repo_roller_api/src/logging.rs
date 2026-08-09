//! Structured logging configuration for the API binary: JSON/pretty format
//! selection and a sanitization layer that redacts sensitive field values
//! before they reach any log sink.
//!
//! This file (together with its test siblings `logging_tests.rs` and
//! `logging_proptests.rs`) defines the informal interface contract for
//! Task #1 ("Logging: JSON output, correlation ID, sanitization layer"),
//! since no dedicated interface spec file exists for it (see task context:
//! "No dedicated interface spec file for logging/middleware exists").
//!
//! ## Contract
//!
//! ### Format selection
//! - [`resolve_log_format`] is a **pure** function: given the raw value of
//!   the `LOG_FORMAT` environment variable (or `None` if unset), it returns
//!   the [`LogFormat`] the API binary's subscriber should use. It must never
//!   panic, regardless of input.
//! - Default behaviour (variable unset, empty string, or any unrecognised
//!   value) is [`LogFormat::Json`] -- the safe, machine-readable default for
//!   production. An unrecognised value must NOT panic or fall back to
//!   pretty; JSON is the fail-safe default.
//! - The literal value `"pretty"`, matched case-insensitively, selects
//!   [`LogFormat::Pretty`].
//! - Production wiring: `main.rs` should call
//!   `resolve_log_format(std::env::var(LOG_FORMAT_ENV_VAR).ok().as_deref())`
//!   and switch between `tracing_subscriber::fmt().json()` and
//!   `tracing_subscriber::fmt()` (default/pretty) accordingly. This module
//!   does not itself wire `main.rs` -- see [`init_tracing`] for the
//!   production entry point that should replace the current
//!   `tracing_subscriber::fmt()...init()` call at
//!   `crates/repo_roller_api/src/main.rs:284-286`.
//!
//! ### Sensitive-field redaction
//! - [`SENSITIVE_FIELD_NAMES`] is the deny-list of field-name fragments
//!   (case-insensitive) that must never appear paired with their original
//!   value in emitted log output.
//! - [`is_sensitive_field_name`] reports whether a given field name matches
//!   the deny-list. Matching is **substring-based and case-insensitive**
//!   (e.g. `"github_token"`, `"db_password"`, `"Authorization"`,
//!   `"x-authorization-header"` all match) -- deliberately biased toward
//!   over-redaction: the cost of an unnecessary redaction is a slightly less
//!   useful log line, while the cost of a missed redaction is a leaked
//!   secret. Must never panic on any input, including the empty string.
//! - [`redact_sensitive_fields`] takes one formatted JSON log line (the
//!   shape `tracing_subscriber::fmt().json()` produces, where an event's own
//!   key/value arguments are nested under a `"fields"` object and any active
//!   span's recorded fields appear under `"span"` / `"spans"` -- see
//!   `logging_tests::realistic_json_line_with_secret` for a concrete
//!   fixture) and returns an equivalent JSON line with every sensitive-named
//!   *object key*'s value replaced by [`REDACTED_MARKER`], regardless of the
//!   value's JSON type (string, number, boolean, null, array, object).
//!   Redaction recurses into nested objects and arrays at any depth (a
//!   sensitive field nested inside `"span"` must be redacted exactly like
//!   one nested inside `"fields"`). Field **names** are matched, never field
//!   **values** -- a `message` field whose prose happens to contain the word
//!   "token" must NOT be redacted, because `message` is not itself a
//!   sensitive field name. Non-JSON input (e.g. a pretty-formatted line, or
//!   garbage) is returned byte-for-byte unchanged -- this function must
//!   never panic on malformed input. The function must be idempotent:
//!   `redact_sensitive_fields(&redact_sensitive_fields(x)) == redact_sensitive_fields(x)`
//!   (structurally, i.e. once parsed back to JSON).
//! - [`build_sanitizing_json_subscriber`] wires [`redact_sensitive_fields`]
//!   into a real `tracing::Subscriber` (JSON-formatted) so that redaction is
//!   applied to every emitted line before any byte reaches the supplied
//!   writer. Exposed chiefly for testing (a test can supply an in-memory
//!   writer and inspect captured output); production start-up should go
//!   through [`init_tracing`] instead.
//! - [`init_tracing`] installs the process-global default subscriber for the
//!   API binary: format selected via `resolve_log_format` reading the
//!   `LOG_FORMAT` env var, level filter via `RUST_LOG` (default `"info"`),
//!   sanitization always applied regardless of format. Like
//!   `tracing_subscriber::...::init()`, this may be called at most once per
//!   process and panics if a global default is already installed.
//!
//! ## Known spec gaps (flagged for the architect / Tech Lead)
//!
//! - The exact redaction marker text is not mandated by
//!   `docs/spec/assertions.md` or the task description -- only "a fixed
//!   redaction marker (e.g. `"[REDACTED]"`)" (example wording). This
//!   contract fixes it to [`REDACTED_MARKER`] = `"[REDACTED]"`. Tests
//!   reference the constant symbolically rather than the literal string so a
//!   future change to the marker text only requires updating one
//!   definition.
//! - Whether redaction must also apply to the CLI's pretty-format output is
//!   not addressed by the acceptance criteria (task description explicitly
//!   scopes the CLI change as optional/"if trivial", and only the API
//!   binary's JSON output is covered by the acceptance criteria). This file
//!   does not test CLI-side sanitization.
//! - Whether `LOG_FORMAT` values should be trimmed of surrounding whitespace
//!   before matching is unspecified; tests do not exercise whitespace-padded
//!   values.
//! - The precise substring-match policy for `is_sensitive_field_name` (as
//!   opposed to exact-match-only) is a Tester judgement call, not stated
//!   explicitly in the task description beyond the seven example names.
//!   Substring matching was chosen to satisfy the "auto_sanitization"
//!   security rule's intent (defense-in-depth, never leak secrets) at the
//!   cost of occasionally redacting a field that merely mentions a
//!   deny-listed word in a compound name.

/// Log output format for the API binary's tracing subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Machine-readable JSON lines (production default).
    Json,
    /// Human-readable "pretty" output (opt-in via `LOG_FORMAT=pretty`).
    Pretty,
}

/// Name of the environment variable that selects human-readable output.
///
/// Set to `"pretty"` to opt out of the default JSON format for local
/// `cargo run` development.
pub const LOG_FORMAT_ENV_VAR: &str = "LOG_FORMAT";

/// Resolves the desired [`LogFormat`] from the raw value of the
/// `LOG_FORMAT` environment variable (or `None` if the variable is unset).
///
/// See the module-level contract for the exact matching rules.
pub fn resolve_log_format(raw_env_value: Option<&str>) -> LogFormat {
    match raw_env_value {
        Some(value) if value.eq_ignore_ascii_case("pretty") => LogFormat::Pretty,
        _ => LogFormat::Json,
    }
}

/// Case-insensitive, substring-matched deny-list of field-name fragments
/// whose values must never appear in emitted log output in raw form.
pub const SENSITIVE_FIELD_NAMES: &[&str] = &[
    "token",
    "password",
    "secret",
    "authorization",
    "github_token",
    "jwt",
    "api_key",
];

/// Fixed replacement value written in place of a redacted field's value.
pub const REDACTED_MARKER: &str = "[REDACTED]";

/// Returns `true` if `field_name` matches [`SENSITIVE_FIELD_NAMES`] per the
/// contract documented on this module (case-insensitive substring match).
pub fn is_sensitive_field_name(field_name: &str) -> bool {
    let lowercased = field_name.to_lowercase();
    SENSITIVE_FIELD_NAMES
        .iter()
        .any(|fragment| lowercased.contains(fragment))
}

/// Recursively walks a mutable JSON value, replacing the value of every
/// object key that matches [`is_sensitive_field_name`] with
/// [`REDACTED_MARKER`], and recursing into nested objects/arrays otherwise.
fn redact_value_in_place(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map.iter_mut() {
                if is_sensitive_field_name(key) {
                    *nested = serde_json::Value::String(REDACTED_MARKER.to_string());
                } else {
                    redact_value_in_place(nested);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                redact_value_in_place(item);
            }
        }
        _ => {}
    }
}

/// Redacts every sensitive-named field's value in a single formatted JSON
/// log line. See the module-level contract for the exact behaviour
/// required, including recursion, non-JSON passthrough, and idempotency.
pub fn redact_sensitive_fields(json_line: &str) -> String {
    let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(json_line) else {
        return json_line.to_string();
    };

    redact_value_in_place(&mut parsed);

    serde_json::to_string(&parsed).unwrap_or_else(|_| json_line.to_string())
}

/// A `std::io::Write` wrapper that redacts sensitive field values (via
/// [`redact_sensitive_fields`]) before forwarding bytes to the wrapped
/// writer.
///
/// Relies on `tracing_subscriber::fmt`'s implementation detail (confirmed
/// against `tracing-subscriber` 0.3.23's `fmt_layer.rs`) that a single
/// formatted event -- JSON body plus trailing newline -- is always delivered
/// to the writer via exactly one `write_all` call. This lets us treat every
/// `write` call as "one complete log line" rather than needing to buffer
/// across calls.
struct SanitizingWriter<W> {
    inner: W,
}

impl<W: std::io::Write> std::io::Write for SanitizingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        let (body, had_trailing_newline) = match text.strip_suffix('\n') {
            Some(stripped) => (stripped, true),
            None => (text.as_ref(), false),
        };

        let mut sanitized = redact_sensitive_fields(body).into_bytes();
        if had_trailing_newline {
            sanitized.push(b'\n');
        }

        self.inner.write_all(&sanitized)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// A `tracing_subscriber::fmt::MakeWriter` that wraps every writer it
/// produces in a [`SanitizingWriter`].
#[derive(Clone)]
struct SanitizingMakeWriter<W> {
    inner: W,
}

impl<'a, W> tracing_subscriber::fmt::MakeWriter<'a> for SanitizingMakeWriter<W>
where
    W: tracing_subscriber::fmt::MakeWriter<'a>,
{
    type Writer = SanitizingWriter<W::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        SanitizingWriter {
            inner: self.inner.make_writer(),
        }
    }
}

/// Builds a JSON-formatted `tracing::Subscriber` that writes to `writer` and
/// redacts sensitive fields (per [`SENSITIVE_FIELD_NAMES`] /
/// [`redact_sensitive_fields`]) before any bytes reach it.
///
/// Exposed chiefly for testing -- production start-up should call
/// [`init_tracing`] instead.
pub fn build_sanitizing_json_subscriber<W>(writer: W) -> Box<dyn tracing::Subscriber + Send + Sync>
where
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let sanitizing_writer = SanitizingMakeWriter { inner: writer };
    Box::new(
        tracing_subscriber::fmt()
            .json()
            .with_writer(sanitizing_writer)
            .finish(),
    )
}

/// Initializes the process-global tracing subscriber for the API binary.
///
/// Format is selected via [`resolve_log_format`] reading the
/// [`LOG_FORMAT_ENV_VAR`] environment variable; the level filter is read
/// from `RUST_LOG` (defaulting to `"info"` if unset, matching today's
/// `crates/repo_roller_api/src/main.rs:284-286` behaviour). Sanitization is
/// always applied, regardless of format.
///
/// # Panics
/// Panics if a global default subscriber has already been installed for
/// this process -- mirrors `tracing_subscriber::...::init()`'s existing
/// panic-on-double-init behaviour. Callable at most once per process.
pub fn init_tracing() {
    let format = resolve_log_format(std::env::var(LOG_FORMAT_ENV_VAR).ok().as_deref());
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let env_filter = tracing_subscriber::EnvFilter::new(filter);

    match format {
        LogFormat::Pretty => {
            tracing_subscriber::fmt().with_env_filter(env_filter).init();
        }
        LogFormat::Json => {
            use tracing_subscriber::layer::SubscriberExt;

            let subscriber = build_sanitizing_json_subscriber(std::io::stdout).with(env_filter);
            tracing::subscriber::set_global_default(subscriber)
                .expect("global tracing subscriber must not already be installed");
        }
    }
}

#[cfg(test)]
#[path = "logging_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "logging_proptests.rs"]
mod proptests;
