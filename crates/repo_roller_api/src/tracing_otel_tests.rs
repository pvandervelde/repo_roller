//! Unit and adversarial tests for `tracing_otel`.

use super::*;
use opentelemetry_sdk::trace::InMemorySpanExporter;
use std::sync::{Arc, Mutex};
use tracing::Instrument;
use tracing_subscriber::layer::SubscriberExt;

// ---------------------------------------------------------------------
// Tier 1: Specification tests -- resolve_otel_settings.
// ---------------------------------------------------------------------

#[test]
fn test_resolve_otel_settings_all_unset_defaults_to_disabled() {
    let settings = resolve_otel_settings(None, None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_all_unset_uses_default_endpoint() {
    let settings = resolve_otel_settings(None, None, None);
    assert_eq!(settings.otlp_endpoint, DEFAULT_OTLP_ENDPOINT);
}

#[test]
fn test_resolve_otel_settings_all_unset_uses_default_service_name() {
    let settings = resolve_otel_settings(None, None, None);
    assert_eq!(settings.service_name, DEFAULT_SERVICE_NAME);
}

#[test]
fn test_resolve_otel_settings_enabled_true_lowercase_enables_export() {
    let settings = resolve_otel_settings(Some("true"), None, None);
    assert!(settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_true_mixed_case_enables_export() {
    let settings = resolve_otel_settings(Some("True"), None, None);
    assert!(settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_one_enables_export() {
    let settings = resolve_otel_settings(Some("1"), None, None);
    assert!(settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_false_stays_disabled() {
    let settings = resolve_otel_settings(Some("false"), None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_zero_stays_disabled() {
    let settings = resolve_otel_settings(Some("0"), None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_garbage_value_stays_disabled() {
    let settings = resolve_otel_settings(Some("yes-please"), None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_enabled_empty_string_stays_disabled() {
    let settings = resolve_otel_settings(Some(""), None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_custom_endpoint_is_used_verbatim() {
    let settings = resolve_otel_settings(None, Some("http://collector:4317"), None);
    assert_eq!(settings.otlp_endpoint, "http://collector:4317");
}

#[test]
fn test_resolve_otel_settings_empty_endpoint_falls_back_to_default() {
    let settings = resolve_otel_settings(None, Some(""), None);
    assert_eq!(settings.otlp_endpoint, DEFAULT_OTLP_ENDPOINT);
}

#[test]
fn test_resolve_otel_settings_custom_service_name_is_used_verbatim() {
    let settings = resolve_otel_settings(None, None, Some("my-custom-service"));
    assert_eq!(settings.service_name, "my-custom-service");
}

#[test]
fn test_resolve_otel_settings_empty_service_name_falls_back_to_default() {
    let settings = resolve_otel_settings(None, None, Some(""));
    assert_eq!(settings.service_name, DEFAULT_SERVICE_NAME);
}

// ---------------------------------------------------------------------
// Tier 2: Adversarial / boundary tests -- resolve_otel_settings.
// ---------------------------------------------------------------------

#[test]
fn test_resolve_otel_settings_never_panics_on_pathological_input() {
    let raw = "x".repeat(10_000);
    let _ = resolve_otel_settings(Some(&raw), Some(&raw), Some(&raw));
}

#[test]
fn test_resolve_otel_settings_never_panics_on_non_ascii_unicode() {
    let _ = resolve_otel_settings(Some("有効"), Some("🚀"), Some("サービス"));
}

#[test]
fn test_resolve_otel_settings_whitespace_only_enabled_value_stays_disabled() {
    let settings = resolve_otel_settings(Some("   "), None, None);
    assert!(!settings.enabled);
}

#[test]
fn test_resolve_otel_settings_two_calls_with_different_input_are_independent() {
    // Stub-killing: guards against any implementation that leaks state
    // across calls (this function is documented as pure).
    let disabled = resolve_otel_settings(Some("false"), None, None);
    let enabled = resolve_otel_settings(Some("true"), None, None);
    assert!(!disabled.enabled);
    assert!(enabled.enabled);
}

// ---------------------------------------------------------------------
// Tier 2: OTel bridge integration -- build_otel_layer_with_exporter.
// ---------------------------------------------------------------------

/// Builds a `Registry`-based subscriber layered with an OTel layer backed by
/// `exporter`, runs `body` under it, force-flushes the tracer provider, and
/// returns the exported spans.
async fn run_under_otel_layer<F, Fut>(exporter: InMemorySpanExporter, body: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let layer = build_otel_layer_with_exporter(exporter, "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);
    body().await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_otel_layer_with_exporter_exports_a_span_with_the_right_name() {
    let exporter = InMemorySpanExporter::default();
    run_under_otel_layer(exporter.clone(), || async {
        tracing::info_span!("repository_creation").in_scope(|| {});
    })
    .await;

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    assert!(
        spans
            .iter()
            .any(|s| s.name.as_ref() == "repository_creation"),
        "expected an exported span named 'repository_creation', got: {spans:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_otel_layer_with_exporter_preserves_parent_child_nesting() {
    let exporter = InMemorySpanExporter::default();
    run_under_otel_layer(exporter.clone(), || async {
        async {
            tracing::info_span!("authentication").in_scope(|| {});
        }
        .instrument(tracing::info_span!("repository_creation"))
        .await;
    })
    .await;

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let parent = spans
        .iter()
        .find(|s| s.name.as_ref() == "repository_creation")
        .expect("parent span must be exported");
    let child = spans
        .iter()
        .find(|s| s.name.as_ref() == "authentication")
        .expect("child span must be exported");
    assert_eq!(
        child.parent_span_id,
        parent.span_context.span_id(),
        "authentication span's parent_span_id must equal repository_creation's span_id"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_otel_layer_with_exporter_two_top_level_spans_are_not_parent_child() {
    // Stub-killing: guards against an implementation that always links every
    // span to the immediately-preceding one regardless of actual nesting.
    let exporter = InMemorySpanExporter::default();
    run_under_otel_layer(exporter.clone(), || async {
        tracing::info_span!("first_span").in_scope(|| {});
        tracing::info_span!("second_span").in_scope(|| {});
    })
    .await;

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let first = spans
        .iter()
        .find(|s| s.name.as_ref() == "first_span")
        .unwrap();
    let second = spans
        .iter()
        .find(|s| s.name.as_ref() == "second_span")
        .unwrap();
    assert_ne!(
        second.parent_span_id,
        first.span_context.span_id(),
        "two independently-created top-level spans must not be linked as parent/child"
    );
}

// ---------------------------------------------------------------------
// Tier 2: Composition with Task 1's logging layer.
// ---------------------------------------------------------------------

/// A `std::io::Write` + `MakeWriter` that appends every write to a shared,
/// in-memory buffer, for asserting on captured log output.
#[derive(Clone, Default)]
struct BufferWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl BufferWriter {
    fn captured(&self) -> String {
        String::from_utf8_lossy(&self.buffer.lock().unwrap()).to_string()
    }
}

impl std::io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufferWriter {
    type Writer = BufferWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_composed_subscriber_for_test_redacts_sensitive_log_fields() {
    let writer = BufferWriter::default();
    let subscriber = build_composed_subscriber_for_test(writer.clone(), None);
    let _guard = tracing::subscriber::set_default(subscriber);

    tracing::info!(
        github_token = "ghp_supersecrettoken1234567890",
        "authenticated"
    );

    let captured = writer.captured();
    assert!(
        !captured.contains("supersecrettoken"),
        "composed subscriber must redact sensitive fields exactly like \
         logging::build_sanitizing_json_subscriber, captured: {captured}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_composed_subscriber_for_test_still_writes_non_sensitive_fields() {
    let writer = BufferWriter::default();
    let subscriber = build_composed_subscriber_for_test(writer.clone(), None);
    let _guard = tracing::subscriber::set_default(subscriber);

    tracing::info!(organization = "my-org", "starting creation");

    let captured = writer.captured();
    assert!(
        captured.contains("my-org"),
        "non-sensitive fields must still reach the writer unredacted, captured: {captured}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_composed_subscriber_for_test_composes_otel_layer_and_log_sanitization() {
    let writer = BufferWriter::default();
    let exporter = InMemorySpanExporter::default();
    let otel_layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = build_composed_subscriber_for_test(writer.clone(), Some(otel_layer));
    let _guard = tracing::subscriber::set_default(subscriber);

    async {
        tracing::info!(
            github_token = "ghp_supersecrettoken1234567890",
            "authenticated"
        );
    }
    .instrument(tracing::info_span!("repository_creation"))
    .await;

    let captured = writer.captured();
    assert!(
        !captured.contains("supersecrettoken"),
        "log sanitization must still apply when an OTel layer is also attached, captured: {captured}"
    );

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    assert!(
        spans
            .iter()
            .any(|s| s.name.as_ref() == "repository_creation"),
        "OTel export must still occur when the sanitizing log layer is also attached, \
         got: {spans:?}"
    );
}

// ---------------------------------------------------------------------
// Tier 2: Production entry point double-init guard.
// ---------------------------------------------------------------------

/// Exercises `init_tracing_with_otel`'s documented panic-on-double-init
/// contract, mirroring `logging.rs`'s
/// `test_init_tracing_second_call_panics_on_double_init`. Calling a
/// process-global-installing function directly is inherently a one-shot,
/// process-wide side effect; this test accepts that tradeoff exactly as the
/// `logging.rs` precedent does.
///
/// The first call's outcome is deliberately **not** asserted (matching the
/// `logging.rs` precedent's `let _ = catch_unwind(..)` pattern). Rust's
/// `tracing` crate allows at most one global default subscriber per
/// process, and `logging::init_tracing` is exercised by its own
/// double-init test (`logging_tests.rs`) in this SAME test binary, calling
/// the SAME underlying `tracing::subscriber::set_global_default` process
/// resource. `cargo test` runs test functions from the same binary
/// concurrently in a nondeterministic order, so whichever of these two
/// tests' first calls happens to run second in wall-clock time will
/// observe an already-installed global default and fail an
/// "expect first call to succeed" assertion -- not because
/// `init_tracing_with_otel` is broken, but because "am I the very first
/// global-subscriber install in this whole process" is not a property any
/// single test can deterministically control when another, independent
/// test performs the same kind of process-wide side effect. The meaningful
/// guarantee this test verifies -- that a *second* call to
/// `init_tracing_with_otel` always panics once a global default exists --
/// holds regardless of which call (this test's first call, or some other
/// test's) actually performed the installation.
// ---------------------------------------------------------------------
// Tier 3: Mutation-kill tests (QA audit pass, #2 tracing).
// ---------------------------------------------------------------------

#[test]
fn test_build_production_otel_layer_returns_none_when_disabled() {
    // Kills (in combination with the "enabled" test below):
    // `313:5 replace build_production_otel_layer -> ... with None` would
    // pass this specific case trivially, but the paired "enabled" test
    // below distinguishes it from the real implementation.
    let settings = OtelSettings {
        enabled: false,
        otlp_endpoint: DEFAULT_OTLP_ENDPOINT.to_string(),
        service_name: DEFAULT_SERVICE_NAME.to_string(),
    };
    assert!(build_production_otel_layer(&settings).is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_production_otel_layer_returns_some_when_enabled_with_valid_endpoint() {
    // Kills: `313:8 delete ! in build_production_otel_layer` (inverts the
    // enabled check, returning None whenever export IS enabled) and
    // `313:5 replace build_production_otel_layer -> ... with None`
    // (unconditionally returns None). Both mutants collapse this case --
    // which must return `Some` -- to `None`.
    let settings = OtelSettings {
        enabled: true,
        otlp_endpoint: DEFAULT_OTLP_ENDPOINT.to_string(),
        service_name: "test-service".to_string(),
    };
    assert!(
        build_production_otel_layer(&settings).is_some(),
        "enabled settings with a well-formed endpoint must produce a layer"
    );
}

// ---------------------------------------------------------------------
// Tier 3: Adversarial -- repeated construction / mem::forget leak concern.
// ---------------------------------------------------------------------

/// `build_otel_layer_with_exporter` deliberately `mem::forget`s one
/// `SdkTracerProvider` reference per call (see its doc comment) to avoid
/// auto-shutdown wiping already-exported spans before a caller inspects
/// them. This is a documented, reviewed trade-off, not a bug -- but calling
/// it many times in a loop (as a hostile/careless caller, or a test suite
/// that repeatedly exercises the OTel-enabled code path, might) should
/// still be *functionally* safe: no crash, no cross-contamination between
/// independently-constructed layers/exporters, no observable slowdown. This
/// test does not attempt to assert on process memory (not reliably
/// observable from within the test itself); it asserts the process-visible
/// contract that actually matters -- correctness -- holds under repetition.
#[tokio::test(flavor = "current_thread")]
async fn test_build_otel_layer_with_exporter_repeated_construction_in_loop_is_safe() {
    for i in 0..200 {
        let exporter = InMemorySpanExporter::default();
        let layer = build_otel_layer_with_exporter(exporter.clone(), &format!("loop-service-{i}"));
        let subscriber = tracing_subscriber::registry().with(layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        tracing::info_span!("loop_span").in_scope(|| {});

        let spans = exporter
            .get_finished_spans()
            .expect("in-memory exporter must report finished spans");
        assert_eq!(
            spans.len(),
            1,
            "iteration {i}: expected exactly one span from this iteration's own \
             exporter, got {spans:?} -- cross-contamination from a leaked prior \
             provider would be a genuine defect"
        );
        assert_eq!(spans[0].name.as_ref(), "loop_span");
    }
}

#[test]
fn test_init_tracing_with_otel_first_call_succeeds_second_call_panics_on_double_init() {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(init_tracing_with_otel));

    let second_call =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(init_tracing_with_otel));
    assert!(
        second_call.is_err(),
        "init_tracing_with_otel() must panic on a second call once a global default \
         subscriber has already been installed by the first"
    );
}

// ---------------------------------------------------------------------
// RED tests: HIGH #2 security-remediation -- OTLP exporter TLS support.
//
// `build_otlp_span_exporter` builds the production span exporter via
// `opentelemetry_otlp::SpanExporter::builder().with_tonic().with_endpoint(..)
// .build()`, and the workspace `Cargo.toml` declares
// `opentelemetry-otlp = { version = "0.32", features = ["grpc-tonic"] }` --
// none of `tls`, `tls-ring`, `tls-aws-lc`, or `tls-provider-agnostic` is
// enabled. Confirmed empirically (RED-phase probe, not committed): today
// `build_otlp_span_exporter("https://collector.example.com:4317")` returns
// `Err(..)` with the message "uses HTTPS but no TLS feature is enabled;
// enable one of the `tls-ring`, `tls-aws-lc`, or `tls-provider-agnostic`
// features". An `http://` endpoint "works" but sends every span in
// cleartext, violating `docs/spec/security/threat-model.md` Layer 1 ("TLS
// encryption for all communications").
// ---------------------------------------------------------------------

#[test]
fn test_build_otlp_span_exporter_with_https_endpoint_constructs_successfully() {
    // Narrowest possible test per the task brief: validates exporter
    // *construction* succeeds against an `https://` endpoint -- not that
    // data actually round-trips to a real HTTPS collector (out of scope for
    // a unit test; no live collector is contacted here or by the underlying
    // tonic channel builder, which is lazy).
    let result = build_otlp_span_exporter("https://collector.example.com:4317");
    assert!(
        result.is_ok(),
        "constructing the production OTLP exporter against an https:// endpoint must succeed \
         once a TLS feature (tls-ring / tls-aws-lc / tls-provider-agnostic) is enabled on \
         opentelemetry-otlp; got: {result:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_build_production_otel_layer_returns_some_for_https_endpoint() {
    // Mirrors the existing `..._returns_some_when_enabled_with_valid_endpoint`
    // mutation-kill test, but specifically for an `https://` endpoint --
    // proving the fail-safe-off exporter-construction-failure path is not
    // silently swallowing every HTTPS attempt.
    let settings = OtelSettings {
        enabled: true,
        otlp_endpoint: "https://collector.example.com:4317".to_string(),
        service_name: "test-service".to_string(),
    };
    assert!(
        build_production_otel_layer(&settings).is_some(),
        "enabled settings with a well-formed https:// endpoint must produce a layer once TLS \
         support is compiled in -- a None here means HTTPS export is still silently disabled"
    );
}

#[test]
fn test_build_production_otel_layer_reports_exporter_build_failure_via_tracing_not_only_eprintln() {
    // `build_production_otel_layer`'s current implementation reports
    // exporter-construction failure via `eprintln!` only -- invisible to
    // any production log-aggregation pipeline built on `tracing`. Uses a
    // malformed endpoint ("http://[invalid", an unparsable URI) that fails
    // exporter construction independently of TLS-feature status, so this
    // test's RED-ness is not coupled to the HIGH #2 TLS fix landing first.
    let writer = BufferWriter::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let settings = OtelSettings {
        enabled: true,
        otlp_endpoint: "http://[invalid".to_string(),
        service_name: "test-service".to_string(),
    };
    let layer = build_production_otel_layer(&settings);
    assert!(
        layer.is_none(),
        "a malformed, unparsable endpoint must still fail exporter construction (fail-safe-off)"
    );

    let captured = writer.captured();
    assert!(
        !captured.is_empty(),
        "exporter-construction failure must be reported via `tracing::warn!`/`tracing::error!` \
         (visible in production log aggregation), not only `eprintln!` (which bypasses \
         `tracing` entirely and is invisible to any tracing-based log pipeline); captured \
         tracing output was empty"
    );
}
