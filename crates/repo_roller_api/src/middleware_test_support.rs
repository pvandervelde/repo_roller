//! Shared test support for `middleware::extract_trace_context`.
//!
//! Both `middleware_tests.rs` and `middleware_proptests.rs` are sibling
//! `#[path = ...]` submodules of `middleware` and each needs the same two
//! small helpers (install the W3C propagator globally; build a `HeaderMap`
//! carrying a single `traceparent` header). Extracted here during Task #3's
//! REFACTOR pass -- both files previously carried byte-for-byte duplicate
//! definitions of these two functions.
//!
//! Registered via `#[cfg(test)] #[path = "middleware_test_support.rs"] mod
//! test_support;` in `middleware.rs`; call sites reach it as
//! `super::test_support::*` (both consumer files `use` it that way).

use super::HeaderMap;

/// Installs the W3C `TraceContextPropagator` as the global text-map
/// propagator, so `extract_trace_context`'s use of
/// `opentelemetry::global::get_text_map_propagator` actually decodes
/// `traceparent` headers instead of falling back to the no-op default.
/// Backed by an `RwLock` (see `opentelemetry::global::set_text_map_propagator`),
/// so repeated/concurrent calls across many tests and proptest cases are
/// safe -- unlike `tracing`'s once-only global default subscriber.
pub(super) fn ensure_trace_context_propagator_installed() {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );
}

/// Builds a `HeaderMap` with a single `traceparent` header set to `value`,
/// or an empty `HeaderMap` if `value` is not a legal HTTP header value
/// (e.g. contains a control character) -- in which case the caller's
/// assertions should treat this identically to "header absent".
pub(super) fn headers_with_traceparent(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = axum::http::HeaderValue::from_str(value) {
        headers.insert("traceparent", header_value);
    }
    headers
}
