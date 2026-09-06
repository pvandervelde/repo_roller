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
pub fn sanitize_span_attribute_value(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }

    if SecretShapeScanner::new(raw).contains_any_match() {
        return SPAN_ATTRIBUTE_REDACTED_MARKER.to_string();
    }

    raw.chars().take(MAX_SPAN_ATTRIBUTE_LEN).collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Secret-shape detection internals.
//
// Deliberately independent of (structurally similar to, but not sharing a
// dependency with) `repo_roller_api::logging`'s `SecretShapeScanner` -- see
// the module-level "Why this lives in `github_client`" rationale for why
// this crate must not depend on `repo_roller_api`. This scanner only needs
// to answer "does any secret-shaped substring occur anywhere in this
// value?" (whole-value rejection, not substring replacement), so it is
// simpler than the logging-side scanner, but uses the same O(n) backward
// run-length precomputation to stay linear on pathologically long input
// (see `test_sanitize_span_attribute_value_never_panics_on_pathologically_long_input`
// and the property tests exercising up to 2000-char arbitrary input).
// ─────────────────────────────────────────────────────────────────────────

const GH_TOKEN_PREFIXES: &[&str] = &["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"];
const GH_TOKEN_SUFFIX_MIN_LEN: usize = 20;
const BEARER_TOKEN_MIN_LEN: usize = 8;
const JWT_SEGMENT_MIN_LEN: usize = 10;
const HIGH_ENTROPY_MIN_LEN: usize = 32;

fn is_gh_token_suffix_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_jwt_segment_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn is_high_entropy_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '+' || c == '/'
}

/// Per-position maximal run length of characters satisfying a fixed
/// predicate, computed once for the whole input via a backward
/// dynamic-programming pass so that later lookups are O(1).
struct CharRunIndex {
    run_len: Vec<usize>,
}

impl CharRunIndex {
    fn build(chars: &[char], predicate: impl Fn(char) -> bool) -> Self {
        let mut run_len = vec![0usize; chars.len() + 1];
        for i in (0..chars.len()).rev() {
            run_len[i] = if predicate(chars[i]) { run_len[i + 1] + 1 } else { 0 };
        }
        Self { run_len }
    }

    /// Length of the maximal run starting at `i` (0 if out of bounds or `i`
    /// itself does not satisfy the predicate).
    fn at(&self, i: usize) -> usize {
        self.run_len.get(i).copied().unwrap_or(0)
    }
}

/// Per-position maximal run of [`is_high_entropy_char`] characters, plus
/// whether that specific run contains an uppercase letter / lowercase
/// letter / digit anywhere within it.
struct HighEntropyIndex {
    run_len: Vec<usize>,
    has_upper: Vec<bool>,
    has_lower: Vec<bool>,
    has_digit: Vec<bool>,
}

impl HighEntropyIndex {
    fn build(chars: &[char]) -> Self {
        let n = chars.len();
        let mut run_len = vec![0usize; n + 1];
        let mut has_upper = vec![false; n + 1];
        let mut has_lower = vec![false; n + 1];
        let mut has_digit = vec![false; n + 1];
        for i in (0..n).rev() {
            if is_high_entropy_char(chars[i]) {
                run_len[i] = run_len[i + 1] + 1;
                has_upper[i] = chars[i].is_ascii_uppercase() || has_upper[i + 1];
                has_lower[i] = chars[i].is_ascii_lowercase() || has_lower[i + 1];
                has_digit[i] = chars[i].is_ascii_digit() || has_digit[i + 1];
            }
        }
        Self {
            run_len,
            has_upper,
            has_lower,
            has_digit,
        }
    }

    /// Match length at `i` if the maximal high-entropy run starting there
    /// meets both the length threshold and the mixed-case-plus-digit
    /// requirement, `None` otherwise.
    fn match_len_at(&self, i: usize) -> Option<usize> {
        let len = *self.run_len.get(i)?;
        let qualifies = len >= HIGH_ENTROPY_MIN_LEN
            && self.has_upper.get(i).copied().unwrap_or(false)
            && self.has_lower.get(i).copied().unwrap_or(false)
            && self.has_digit.get(i).copied().unwrap_or(false);
        qualifies.then_some(len)
    }
}

/// Precomputed indices over one input string's characters, used to
/// determine whether a secret-shaped match exists starting at any position.
struct SecretShapeScanner {
    chars: Vec<char>,
    gh_suffix: CharRunIndex,
    jwt_segment: CharRunIndex,
    bearer_token: CharRunIndex,
    high_entropy: HighEntropyIndex,
}

impl SecretShapeScanner {
    fn new(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let gh_suffix = CharRunIndex::build(&chars, is_gh_token_suffix_char);
        let jwt_segment = CharRunIndex::build(&chars, is_jwt_segment_char);
        let bearer_token = CharRunIndex::build(&chars, |c: char| !c.is_whitespace());
        let high_entropy = HighEntropyIndex::build(&chars);
        Self {
            chars,
            gh_suffix,
            jwt_segment,
            bearer_token,
            high_entropy,
        }
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    /// Whether `self.chars[i..]` starts with the literal (ASCII) string
    /// `literal`.
    fn matches_literal(&self, i: usize, literal: &str) -> bool {
        let literal_chars: Vec<char> = literal.chars().collect();
        let end = i + literal_chars.len();
        end <= self.chars.len() && self.chars[i..end] == literal_chars[..]
    }

    /// Length of a match consisting of one of `literals` found at position
    /// `i`, immediately followed by a run of at least `min_run_len`
    /// characters per `run_index`. Shared by the GitHub-token-prefix and
    /// Bearer-header shapes, which are both "fixed literal prefix, then a
    /// minimum-length run of a fixed character class".
    fn literal_prefixed_run_match_len(
        &self,
        i: usize,
        literals: &[&str],
        run_index: &CharRunIndex,
        min_run_len: usize,
    ) -> Option<usize> {
        literals.iter().find_map(|literal| {
            if !self.matches_literal(i, literal) {
                return None;
            }
            let literal_len = literal.chars().count();
            let run_len = run_index.at(i + literal_len);
            (run_len >= min_run_len).then_some(literal_len + run_len)
        })
    }

    fn gh_token_match_len(&self, i: usize) -> Option<usize> {
        self.literal_prefixed_run_match_len(i, GH_TOKEN_PREFIXES, &self.gh_suffix, GH_TOKEN_SUFFIX_MIN_LEN)
    }

    fn bearer_match_len(&self, i: usize) -> Option<usize> {
        self.literal_prefixed_run_match_len(i, &["Bearer ", "bearer "], &self.bearer_token, BEARER_TOKEN_MIN_LEN)
    }

    fn jwt_match_len(&self, i: usize) -> Option<usize> {
        let seg1 = self.jwt_segment.at(i);
        if seg1 < JWT_SEGMENT_MIN_LEN || self.chars.get(i + seg1) != Some(&'.') {
            return None;
        }
        let seg2_start = i + seg1 + 1;
        let seg2 = self.jwt_segment.at(seg2_start);
        if seg2 < JWT_SEGMENT_MIN_LEN || self.chars.get(seg2_start + seg2) != Some(&'.') {
            return None;
        }
        let seg3_start = seg2_start + seg2 + 1;
        let seg3 = self.jwt_segment.at(seg3_start);
        (seg3 >= JWT_SEGMENT_MIN_LEN).then_some(seg1 + 1 + seg2 + 1 + seg3)
    }

    /// Whether any secret-shaped match (any of the four recognised shapes)
    /// starts at any position in the scanned text.
    fn contains_any_match(&self) -> bool {
        (0..self.len()).any(|i| {
            self.gh_token_match_len(i).is_some()
                || self.bearer_match_len(i).is_some()
                || self.jwt_match_len(i).is_some()
                || self.high_entropy.match_len_at(i).is_some()
        })
    }
}

#[cfg(test)]
#[path = "span_attributes_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "span_attributes_proptests.rs"]
mod proptests;
