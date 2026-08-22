//! Business-context span-attribute sanitization for OpenTelemetry export.
//!
//! This file (together with its test siblings `span_attributes_tests.rs` and
//! `span_attributes_proptests.rs`) defines the informal interface contract for
//! Task #2 ("Distributed tracing: OpenTelemetry + OTLP"), mirroring the
//! precedent set by `repo_roller_api::logging` for Task #1 (no dedicated
//! interface spec file exists for this concern either).
//!
//! ## Why this lives in `github_client`
//!
//! Business-context values (organization login, repository name, template
//! name, requesting user) are attached as span *attributes* so they are
//! visible in the exported OpenTelemetry trace (per
//! `docs/spec/operations/observability.md` § "Distributed Tracing" ->
//! "Context Enrichment"). Per the Security Rules for this task, span
//! attributes are a data path **entirely separate** from `tracing` log lines:
//! `repo_roller_api::logging`'s redaction layer (Task #1) only ever sees
//! formatted JSON *log* output, never span attributes recorded via
//! `tracing::Span::record`/`#[instrument(fields(...))]`, because
//! `tracing-opentelemetry` reads span field values directly off the
//! `tracing` span, upstream of any `fmt` layer. This module is therefore an
//! independent, second line of defense against ever attaching a raw secret,
//! token, or unbounded attacker-controlled string to a span.
//!
//! `github_client` is the natural home for this contract (rather than a new
//! shared crate) because it is already the lowest-level crate depended on by
//! both `config_manager` and `repo_roller_core` -- the two crates that
//! instrument business-context spans per the task's "Key Trace Points"
//! (authentication, configuration resolution, template processing,
//! repository creation). Introducing a new shared crate solely to host one
//! small function would repeat the tradeoff `repository_metrics.rs` already
//! rejected for `error_category`/`UNSEEDED_SENTINEL`-style helpers: "the
//! three crates have no existing shared low-level dependency -- introducing
//! one solely to deduplicate ... would be a worse tradeoff than the
//! duplication itself." Here, unlike that case, a real shared low-level
//! dependency (`github_client`) already exists, so reuse -- not duplication
//! -- is the right call.
//!
//! ## Contract
//!
//! - [`sanitize_span_attribute_value`] is the single sanitization entry
//!   point. Every business-context value attached to a span as an attribute
//!   intended for OpenTelemetry export (organization login, repository name,
//!   template name, requestor/user login, etc.) MUST be passed through this
//!   function first. Raw, unsanitized values must never be passed directly
//!   to `tracing::field::debug`/`Span::record`/`#[instrument(fields(...))]`
//!   for any value that originates from an HTTP request body, GitHub API
//!   response, or any other attacker-influenceable source.
//! - Must never panic on any input, including the empty string, non-ASCII
//!   unicode, or pathologically long input (defense-in-depth: a panic here
//!   would take down the whole request-handling task, turning an
//!   observability concern into an availability incident).
//! - **Secret-shape rejection (whole-value, defense-in-depth):** if `raw`
//!   contains *anywhere within it* a substring matching one of the four
//!   secret shapes below, the ENTIRE value is discarded and
//!   [`SPAN_ATTRIBUTE_REDACTED_MARKER`] is returned instead of the original
//!   value (not just the matched substring -- a business-context field that
//!   contains anything secret-shaped is inherently suspicious enough that
//!   returning a partially-redacted value is not worth the residual risk of
//!   leaking the surrounding context). This reimplements (independently --
//!   `github_client` does not and must not depend on `repo_roller_api`) the
//!   same four secret shapes documented in
//!   `repo_roller_api::logging`'s "Value-shaped secret redaction" section:
//!     1. GitHub token prefix (`ghp_`/`gho_`/`ghu_`/`ghs_`/`ghr_`/`github_pat_`)
//!        followed by >= 20 chars of `[A-Za-z0-9_]`.
//!     2. Bearer-header shape: `Bearer `/`bearer ` followed by >= 8
//!        non-whitespace characters.
//!     3. JWT shape: three `.`-separated segments, each >= 10 chars of
//!        `[A-Za-z0-9_-]`.
//!     4. Generic high-entropy run: a contiguous run of >= 32 characters from
//!        `[A-Za-z0-9+/]` containing at least one uppercase letter, one
//!        lowercase letter, and one digit.
//! - **Length bounding (cardinality/DoS safety):** when no secret shape is
//!   found, the value is truncated to at most [`MAX_SPAN_ATTRIBUTE_LEN`]
//!   Unicode scalar values (`char`s), never splitting a multi-byte UTF-8
//!   character. No ellipsis or other marker is appended on truncation --
//!   callers must not assume a truncated value remains unique, which is the
//!   point: this bounds worst-case attribute cardinality/size exactly as
//!   `repository_metrics.rs`'s bounded `error_category` enum bounds
//!   Prometheus label cardinality, applying the same "Observability Phase 1"
//!   cardinality-safety instinct to span attributes.
//! - The empty string is returned unchanged (never redacted -- an empty
//!   value is never secret-shaped and is already within the length bound).
//! - Idempotent: `sanitize_span_attribute_value(&sanitize_span_attribute_value(x))
//!   == sanitize_span_attribute_value(x)` for all `x` (a value that has
//!   already been sanitized must not change further if it is accidentally
//!   sanitized twice).
//!
//! ## Known spec gaps (flagged for the architect / Tech Lead)
//!
//! - The exact value of [`MAX_SPAN_ATTRIBUTE_LEN`] (128) is a Tester
//!   judgement call; `docs/spec/operations/observability.md` does not
//!   mandate a specific bound. 128 chars comfortably fits any realistic
//!   GitHub login, repository name, or template name (GitHub's own repo/org
//!   name limit is 100 chars) while still bounding pathological input.
//! - Whether secret-shape detection should discard the *whole* value (this
//!   contract) versus redacting only the matched substring (as
//!   `repo_roller_api::logging::redact_secret_shaped_substrings` does for
//!   free-text log messages) is a deliberate, distinct judgement call for
//!   this *separate* data path: span attributes are expected to be short,
//!   single-purpose identifiers (an org login, a template name), not free
//!   prose, so there is no legitimate reason for partial content after a
//!   secret-shaped match to be worth preserving.

/// Maximum number of Unicode scalar values (`char`s) retained in a
/// business-context span attribute value after sanitization.
///
/// See the module-level contract for the exact truncation behaviour.
pub const MAX_SPAN_ATTRIBUTE_LEN: usize = 128;

/// Fixed replacement value substituted for a span attribute value that is
/// rejected outright as secret-shaped.
///
/// Distinct constant from `repo_roller_api::logging::REDACTED_MARKER` even
/// though the literal text is the same today -- the two crates have no
/// shared dependency and must not be coupled by a shared constant (see the
/// module-level "Why this lives in `github_client`" rationale).
pub const SPAN_ATTRIBUTE_REDACTED_MARKER: &str = "[REDACTED]";

/// Sanitizes a business-context value before it is attached to a `tracing`
/// span as an attribute intended for OpenTelemetry export.
///
/// See the module-level contract for the exact secret-shape-rejection and
/// length-bounding rules this function must implement.
///
/// # Contract summary
/// - Never panics on any input.
/// - Returns [`SPAN_ATTRIBUTE_REDACTED_MARKER`] unconditionally if `raw`
///   contains any secret-shaped substring (GitHub token prefix, Bearer
///   header, JWT, or generic high-entropy run -- see module docs).
/// - Otherwise returns `raw` truncated to at most [`MAX_SPAN_ATTRIBUTE_LEN`]
///   `char`s (never splitting a UTF-8 code point).
/// - Idempotent and total (defined for every `&str` input, including the
///   empty string).
#[allow(dead_code)] // Will gain callers once phase-boundary spans are instrumented (Task #2).
pub fn sanitize_span_attribute_value(raw: &str) -> String {
    let _ = raw;
    todo!(
        "Task #2 (distributed tracing): implement secret-shape rejection + \
         length bounding per the module-level contract in span_attributes.rs. \
         See span_attributes_tests.rs / span_attributes_proptests.rs for the \
         full behavioural contract this must satisfy."
    )
}

#[cfg(test)]
#[path = "span_attributes_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "span_attributes_proptests.rs"]
mod proptests;
