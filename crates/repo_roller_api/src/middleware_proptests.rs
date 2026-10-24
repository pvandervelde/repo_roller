//! Property-based tests for `middleware::extract_trace_context`.
//!
//! Mirrors the sibling `tracing_otel_proptests.rs` pattern: this module is
//! registered via `#[path = "middleware_proptests.rs"] mod proptests;` at
//! the bottom of `middleware.rs`.
//!
//! These tests install the W3C `TraceContextPropagator` as the process-
//! global text-map propagator in their own setup (see
//! `middleware_test_support::ensure_trace_context_propagator_installed`,
//! shared with `middleware_tests.rs`) so they do not depend on
//! `tracing_otel::init_tracing_with_otel` having done so first -- that
//! production call site is a separate, flagged spec gap (see
//! `extract_trace_context`'s doc comment in `middleware.rs`). Installing the
//! propagator is backed by an `RwLock`, not a `OnceLock`, so repeated/
//! concurrent installs of the SAME stateless propagator type across many
//! proptest cases and test threads are safe -- unlike `tracing`'s
//! global-default-subscriber, which may only be installed once per process.

use super::test_support::{ensure_trace_context_propagator_installed, headers_with_traceparent};
use super::*;
use opentelemetry::trace::{SpanId, TraceContextExt, TraceId};
use proptest::prelude::*;

proptest! {
    /// `extract_trace_context` must never panic for ANY string placed in the
    /// `traceparent` header -- the W3C spec requires invalid values to be
    /// ignored, not rejected, and a parser panic would turn a single
    /// malformed client header into a process-wide outage (every request
    /// sharing that worker thread would be affected).
    #[test]
    fn prop_extract_trace_context_never_panics_on_arbitrary_traceparent_value(
        raw in "[\\x20-\\x7e]{0,200}"
    ) {
        ensure_trace_context_propagator_installed();
        let headers = headers_with_traceparent(&raw);
        let _ = extract_trace_context(&headers);
    }

    /// Any well-formed `traceparent` (valid version/flags, non-zero
    /// lowercase-hex trace id and parent id) must round-trip EXACTLY: the
    /// returned context's trace id and span id must equal the values
    /// encoded in the header, for arbitrary values across the id space --
    /// not just the one or two fixed examples used by the unit tests. This
    /// is immune to a hardcoded-return stub, which could satisfy a single
    /// fixed-value unit test but not arbitrary generated ids.
    #[test]
    fn prop_extract_trace_context_well_formed_header_round_trips_trace_and_span_id(
        trace_id_hex in "[1-9a-f][0-9a-f]{31}",
        span_id_hex in "[1-9a-f][0-9a-f]{15}",
        flags_hex in "0[01]",
    ) {
        ensure_trace_context_propagator_installed();
        let traceparent = format!("00-{trace_id_hex}-{span_id_hex}-{flags_hex}");
        let headers = headers_with_traceparent(&traceparent);

        let cx = extract_trace_context(&headers);
        let span_ref = cx.span();
        let span_context = span_ref.span_context();

        prop_assert!(
            span_context.is_valid(),
            "well-formed traceparent '{}' must decode to a valid span context",
            traceparent
        );
        prop_assert!(
            span_context.is_remote(),
            "a span context decoded from an inbound header must be marked remote"
        );
        prop_assert_eq!(
            span_context.trace_id(),
            TraceId::from_hex(&trace_id_hex).unwrap()
        );
        prop_assert_eq!(
            span_context.span_id(),
            SpanId::from_hex(&span_id_hex).unwrap()
        );
    }

    /// Two DIFFERENT well-formed trace ids must decode to two DIFFERENT
    /// trace ids in the returned context -- kills a stub that extracts
    /// *something* but always returns the same fixed/cached context
    /// regardless of input.
    #[test]
    fn prop_extract_trace_context_distinct_inputs_yield_distinct_trace_ids(
        trace_id_a in "[1-9a-f][0-9a-f]{31}",
        trace_id_b in "[1-9a-f][0-9a-f]{31}",
    ) {
        prop_assume!(trace_id_a != trace_id_b);
        ensure_trace_context_propagator_installed();

        let span_id_hex = "00f067aa0ba902b7";
        let cx_a = extract_trace_context(&headers_with_traceparent(&format!(
            "00-{trace_id_a}-{span_id_hex}-01"
        )));
        let cx_b = extract_trace_context(&headers_with_traceparent(&format!(
            "00-{trace_id_b}-{span_id_hex}-01"
        )));

        prop_assert_ne!(
            cx_a.span().span_context().trace_id(),
            cx_b.span().span_context().trace_id(),
            "distinct inbound trace ids must decode to distinct trace ids"
        );
    }
}
