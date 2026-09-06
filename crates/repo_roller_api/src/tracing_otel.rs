//! OpenTelemetry/OTLP span-export wiring for the API binary.
//!
//! This file (together with its test siblings `tracing_otel_tests.rs` and
//! `tracing_otel_proptests.rs`) defines the informal interface contract for
//! Task #2 ("Distributed tracing: OpenTelemetry + OTLP"), mirroring the
//! precedent set by `logging.rs` for Task #1 (no dedicated interface spec
//! file exists for this concern either).
//!
//! ## Contract
//!
//! ### Configuration resolution
//! - [`resolve_otel_settings`] is a **pure** function (mirrors
//!   `logging::resolve_log_format`): given the raw values of the
//!   [`OTEL_ENABLED_ENV_VAR`], [`OTEL_ENDPOINT_ENV_VAR`], and
//!   [`OTEL_SERVICE_NAME_ENV_VAR`] environment variables (each `None` if
//!   unset), it returns the [`OtelSettings`] the API binary should use. Must
//!   never panic, regardless of input.
//! - **Fail-safe default is `enabled: false`.** Unlike `logging.rs`'s
//!   "JSON is the safe default", the safe default here is "OTel export is
//!   OFF" -- an unset or unrecognised `OTEL_TRACES_ENABLED` value must NOT
//!   silently turn on a background exporter that attempts network I/O to an
//!   unconfigured/default endpoint in an environment that never asked for
//!   it. Only the case-insensitive literal values `"true"` or `"1"` enable
//!   export; every other value (including empty string, `"false"`, `"0"`,
//!   garbage, or `None`) resolves to `enabled: false`.
//! - `otlp_endpoint` defaults to [`DEFAULT_OTLP_ENDPOINT`] when
//!   [`OTEL_ENDPOINT_ENV_VAR`] is unset or empty; otherwise it is the raw
//!   env var value verbatim (no validation -- an invalid URL is a
//!   production-startup-time concern for whatever OTLP exporter constructor
//!   consumes this value, not this pure function's job).
//! - `service_name` defaults to [`DEFAULT_SERVICE_NAME`] when
//!   [`OTEL_SERVICE_NAME_ENV_VAR`] is unset or empty; otherwise it is the raw
//!   env var value verbatim.
//!
//! ### Span-export layer construction (testable seam)
//! - [`build_otel_layer_with_exporter`] builds a `tracing_subscriber::Layer`
//!   that bridges `tracing` spans to OpenTelemetry spans via a caller-
//!   supplied `opentelemetry_sdk::trace::SpanExporter`, using a
//!   `SimpleSpanProcessor` (synchronous export, no batching) so that a
//!   caller can deterministically `force_flush` and inspect exported spans
//!   in a test without batching-related timing flakiness. Production code
//!   should ultimately wire a real OTLP exporter through this same
//!   function (or an equivalent one added by the Coder that constructs the
//!   real OTLP exporter and delegates here) -- this function itself must
//!   remain exporter-agnostic so tests can inject
//!   `opentelemetry_sdk::trace::InMemorySpanExporter` and assert on
//!   captured `SpanData` without a live OTLP collector.
//! - Every span produced anywhere in the crate graph while this layer is
//!   attached to the active subscriber is translated to an OpenTelemetry
//!   span with the SAME name and the SAME parent/child nesting as the
//!   `tracing` span tree (this is `tracing-opentelemetry`'s own guarantee;
//!   the Coder's job is correct composition, not reimplementing it).
//!
//! ### Composition with Task 1's logging layer
//! - [`build_composed_subscriber_for_test`] builds one complete
//!   `tracing::Subscriber` for the API binary that combines:
//!     1. Task 1's JSON sanitization behaviour -- every emitted log line
//!        passes through `logging::redact_sensitive_fields` before reaching
//!        `writer`, byte-for-byte equivalent to what
//!        `logging::build_sanitizing_json_subscriber` guarantees today --
//!        and
//!     2. this task's OTel span-export layer, attached via `.with()` when
//!        (and only when) `otel_layer` is `Some`.
//!
//!   Built on `tracing_subscriber::registry()` -- **not** the full
//!   `tracing_subscriber::fmt::Subscriber` that
//!   `logging::build_sanitizing_json_subscriber` returns -- specifically so
//!   the OTel layer can be composed via `.with()`. This is deliberately a
//!   distinct construction from `logging::build_sanitizing_json_subscriber`
//!   (which remains untouched -- Task 1 is complete and audited) rather
//!   than a wrapper around it, because a `fmt::Subscriber` cannot itself be
//!   layered under `Registry::with()`. The Coder does not need private
//!   access to `logging.rs`'s internals to implement this: every function
//!   this contract depends on (`logging::redact_sensitive_fields`) is
//!   already `pub`.
//! - Exposed chiefly for testing; production start-up should call
//!   [`init_tracing_with_otel`] instead.
//!
//! ### Production entry point
//! - [`init_tracing_with_otel`] installs the process-global default
//!   subscriber for the API binary, replacing the current
//!   `logging::init_tracing()` call site in `main.rs`: format/redaction as
//!   `logging::init_tracing` today, PLUS an OTel export layer attached
//!   whenever `resolve_otel_settings(..).enabled` is `true`. Like
//!   `logging::init_tracing`, this may be called at most once per process
//!   and panics if a global default is already installed.
//!
//! ## Known spec gaps / judgement calls (flagged for the architect / Tech Lead)
//! - The exact env var names ([`OTEL_ENABLED_ENV_VAR`] = `OTEL_TRACES_ENABLED`,
//!   reusing the OpenTelemetry-conventional [`OTEL_ENDPOINT_ENV_VAR`] =
//!   `OTEL_EXPORTER_OTLP_ENDPOINT` and [`OTEL_SERVICE_NAME_ENV_VAR`] =
//!   `OTEL_SERVICE_NAME`) are a Tester judgement call: `OTEL_EXPORTER_OTLP_ENDPOINT`
//!   and `OTEL_SERVICE_NAME` are the standard OpenTelemetry SDK environment
//!   variable names; `OTEL_TRACES_ENABLED` is not a standard OTel env var
//!   but is needed here because this codebase's fail-safe-off requirement
//!   (see above) has no standard-OTel equivalent (the real OTel SDK
//!   env-based config has no simple global on/off switch).
//! - Whether a disabled OTel layer should still be constructed as a no-op
//!   `Layer` versus omitted entirely from the `.with()` chain is left to the
//!   Coder; `build_composed_subscriber_for_test` and
//!   `init_tracing_with_otel` accept `Option<...>` specifically so "omitted
//!   entirely" is the natural, zero-cost choice.

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

/// Name of the environment variable that enables OTel span export.
///
/// Not a standard OpenTelemetry SDK env var (see module docs "Known spec
/// gaps") -- introduced because this codebase's fail-safe-off default has
/// no standard-OTel equivalent.
pub const OTEL_ENABLED_ENV_VAR: &str = "OTEL_TRACES_ENABLED";

/// Name of the (standard OpenTelemetry SDK) environment variable holding the
/// OTLP collector endpoint.
pub const OTEL_ENDPOINT_ENV_VAR: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Name of the (standard OpenTelemetry SDK) environment variable holding the
/// service name attached to every exported span's resource attributes.
pub const OTEL_SERVICE_NAME_ENV_VAR: &str = "OTEL_SERVICE_NAME";

/// Default OTLP collector endpoint used when [`OTEL_ENDPOINT_ENV_VAR`] is
/// unset or empty -- matches the local-dev collector wired up in
/// `examples/local-stack/docker-compose.yml`.
pub const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";

/// Default service name used when [`OTEL_SERVICE_NAME_ENV_VAR`] is unset or
/// empty.
pub const DEFAULT_SERVICE_NAME: &str = "repo-roller-api";

/// Resolved OpenTelemetry/OTLP export configuration for the API process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtelSettings {
    /// Whether OTel span export should be attached to the global subscriber
    /// at all. Fail-safe default is `false` -- see module-level contract.
    pub enabled: bool,
    /// OTLP collector endpoint (gRPC/HTTP, depending on the exporter the
    /// Coder wires up).
    pub otlp_endpoint: String,
    /// Service name attached to every exported span's resource attributes.
    pub service_name: String,
}

/// Resolves [`OtelSettings`] from the raw values of the three
/// OTel-related environment variables, or `None` if a given variable is
/// unset. See the module-level contract for the exact matching/default
/// rules. Never panics.
pub fn resolve_otel_settings(
    enabled_raw: Option<&str>,
    endpoint_raw: Option<&str>,
    service_name_raw: Option<&str>,
) -> OtelSettings {
    let enabled = enabled_raw
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false);

    let otlp_endpoint = endpoint_raw
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_OTLP_ENDPOINT.to_string());

    let service_name = service_name_raw
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_string());

    OtelSettings {
        enabled,
        otlp_endpoint,
        service_name,
    }
}

/// Builds a `tracing_subscriber::Layer` that bridges `tracing` spans to
/// OpenTelemetry spans exported via `exporter`, using a synchronous
/// `SimpleSpanProcessor` (no batching) so span export is deterministic for
/// tests.
///
/// See the module-level "Span-export layer construction" contract section
/// for the exact composition requirements. `service_name` is attached to the
/// tracer's resource attributes.
///
/// # Type parameters
/// `S` is the subscriber type this layer will ultimately be attached to
/// (typically `tracing_subscriber::Registry`); `E` is any
/// `opentelemetry_sdk::trace::SpanExporter` implementation -- production
/// code supplies the real OTLP exporter, tests supply
/// `opentelemetry_sdk::trace::InMemorySpanExporter`.
pub fn build_otel_layer_with_exporter<S, E>(
    exporter: E,
    service_name: &str,
) -> Box<dyn Layer<S> + Send + Sync>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span> + Send + Sync + 'static,
    E: opentelemetry_sdk::trace::SpanExporter + 'static,
{
    use opentelemetry::trace::TracerProvider as _;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_span_processor(opentelemetry_sdk::trace::SimpleSpanProcessor::new(exporter))
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name.to_string())
                .build(),
        )
        .build();

    let tracer = provider.tracer(service_name.to_string());

    // `SdkTracerProvider` is reference-counted: dropping its *last* strong
    // reference auto-triggers `shutdown()` (documented on
    // `SdkTracerProvider`). For exporters with "reset on shutdown" semantics
    // (e.g. `InMemorySpanExporter`, used by this file's own tests), that
    // shutdown silently clears already-exported spans out from under a
    // caller who is still holding the exporter handle -- exactly what
    // happens when the returned layer (the only other owner of `provider`,
    // via `tracer`) is dropped at the end of a scoped test subscriber's
    // lifetime, before the caller inspects the exporter. Production OTLP
    // export has no equivalent "reset" semantics, and this crate exposes no
    // `shutdown_tracing()` entry point that could otherwise release this
    // reference before process exit, so leaking one extra strong reference
    // for the process lifetime is a deliberate, documented trade-off (keep
    // the provider alive until the process exits) rather than an oversight.
    std::mem::forget(provider.clone());

    Box::new(tracing_opentelemetry::layer().with_tracer(tracer))
}

/// Builds one complete `tracing::Subscriber` combining Task 1's JSON
/// sanitization behaviour with this task's optional OTel export layer.
///
/// See the module-level "Composition with Task 1's logging layer" contract
/// section. Exposed chiefly for testing; production start-up should call
/// [`init_tracing_with_otel`] instead.
#[allow(dead_code)] // No production caller by design -- see doc comment above; still exercised by this file's own tests.
pub fn build_composed_subscriber_for_test<W>(
    writer: W,
    otel_layer: Option<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>>,
) -> Box<dyn tracing::Subscriber + Send + Sync>
where
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(SanitizingMakeWriter { inner: writer });

    Box::new(
        tracing_subscriber::registry()
            .with(otel_layer)
            .with(json_layer),
    )
}

/// Initializes the process-global tracing subscriber for the API binary,
/// combining Task 1's sanitized JSON logging with this task's OTel span
/// export (attached only when [`resolve_otel_settings`] reports `enabled`).
///
/// Intended to replace the current `logging::init_tracing()` call site in
/// `main.rs`.
///
/// # Panics
/// Panics if a global default subscriber has already been installed for
/// this process -- mirrors `logging::init_tracing`'s existing
/// panic-on-double-init behaviour. Callable at most once per process.
pub fn init_tracing_with_otel() {
    let log_format =
        crate::logging::resolve_log_format(std::env::var(crate::logging::LOG_FORMAT_ENV_VAR).ok().as_deref());
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let env_filter = tracing_subscriber::EnvFilter::new(filter);

    let settings = resolve_otel_settings(
        std::env::var(OTEL_ENABLED_ENV_VAR).ok().as_deref(),
        std::env::var(OTEL_ENDPOINT_ENV_VAR).ok().as_deref(),
        std::env::var(OTEL_SERVICE_NAME_ENV_VAR).ok().as_deref(),
    );
    let otel_layer = build_production_otel_layer(&settings);

    match log_format {
        crate::logging::LogFormat::Pretty => {
            let subscriber = tracing_subscriber::registry()
                .with(otel_layer)
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer());
            tracing::subscriber::set_global_default(subscriber)
                .expect("global tracing subscriber must not already be installed");
        }
        crate::logging::LogFormat::Json => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_writer(SanitizingMakeWriter { inner: std::io::stdout });
            let subscriber = tracing_subscriber::registry()
                .with(otel_layer)
                .with(env_filter)
                .with(json_layer);
            tracing::subscriber::set_global_default(subscriber)
                .expect("global tracing subscriber must not already be installed");
        }
    }
}

/// Builds the production OTel layer from resolved [`OtelSettings`], or
/// `None` when export is disabled or the OTLP exporter fails to construct.
///
/// Exporter construction failure (e.g. an unparsable endpoint) is treated as
/// fail-safe-off rather than a startup panic -- mirrors the module's
/// documented preference for silently disabling network export over
/// silently enabling it (or, worse here, crashing the whole process over an
/// observability concern).
fn build_production_otel_layer(
    settings: &OtelSettings,
) -> Option<Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync>> {
    if !settings.enabled {
        return None;
    }

    match build_otlp_span_exporter(&settings.otlp_endpoint) {
        Ok(exporter) => Some(build_otel_layer_with_exporter(exporter, &settings.service_name)),
        Err(err) => {
            eprintln!(
                "Failed to build OTLP span exporter for endpoint '{}': {err}. \
                 Distributed tracing will be disabled for this process.",
                settings.otlp_endpoint
            );
            None
        }
    }
}

/// Constructs the real gRPC/OTLP span exporter used in production, pointed
/// at `endpoint`. Matches the docker-compose local-stack's `OTEL_EXPORTER_OTLP_ENDPOINT`
/// default (`http://localhost:4317`), a standard OTLP/gRPC port.
fn build_otlp_span_exporter(
    endpoint: &str,
) -> Result<opentelemetry_otlp::SpanExporter, opentelemetry_otlp::ExporterBuildError> {
    use opentelemetry_otlp::WithExportConfig;

    opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
}

/// A `std::io::Write` wrapper that redacts sensitive field values (via
/// [`crate::logging::redact_sensitive_fields`]) before forwarding bytes to
/// the wrapped writer.
///
/// A separate, locally-owned type from `logging::SanitizingWriter` (which is
/// private to that module) -- reuses the module's `pub` redaction function
/// rather than its internal `io::Write` plumbing, per this module's
/// "Composition with Task 1's logging layer" contract.
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

        let mut sanitized = crate::logging::redact_sensitive_fields(body).into_bytes();
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

#[cfg(test)]
#[path = "tracing_otel_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tracing_otel_proptests.rs"]
mod proptests;
