// Observability Phase 1: per-endpoint HTTP request metrics.
//
// Mirrors `repo_roller_core::event_metrics::EventMetrics` /
// `repo_roller_core::repository_metrics` / `github_client::api_metrics` so
// that the HTTP layer never depends on `prometheus::*` types directly outside
// this module (hexagonal-architecture constraint, docs/spec/constraints.md:138).
//
// TDD RED PHASE: `PrometheusHttpMetrics` and `http_metrics_middleware` are
// stubbed with `todo!()` and MUST be implemented by the Coder.
// `NoOpHttpMetrics` is fully implemented (a no-op has no real logic to defer).
//
// ## Metric name / label design (Tester's working assumption — not present in
// the injected Interface Contract; flagged as a spec gap in the test report)
//
// | Metric name                    | Type      | Labels                         |
// |----------------------------------|-----------|--------------------------------|
// | http_requests_total               | Counter   | method, route, status_code     |
// | http_request_duration_seconds     | Histogram | method, route                  |
//
// `route` MUST be the axum route **template** (e.g.
// `"/api/v1/orgs/{org}/templates/{template}"`), obtained via
// `axum::extract::MatchedPath`, never the concrete request path (which
// contains unbounded, potentially sensitive organisation/repository/template
// names — see SECURITY RULE: "Metric labels must be bounded/enumerable
// values only ... unbounded label cardinality is also a Prometheus
// operational hazard, not just a leak risk"). `status_code` is rendered as
// its string form (e.g. `"404"`), which is bounded (three digits).

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Sentinel label value pre-registered for every vector metric at
/// construction time, so its metric family is always present in a Prometheus
/// scrape — even before the first real HTTP request is recorded.
///
/// `prometheus::Registry::gather()` prunes any `CounterVec`/`HistogramVec`
/// metric family that has zero recorded label combinations. Without this
/// pre-seed, `http_requests_total` (etc.) would be absent from a scrape
/// taken immediately after startup. See the canonical rationale (including
/// why the sentinel value is the maximum valid Unicode scalar) documented on
/// `repo_roller_core::repository_metrics::UNSEEDED_SENTINEL` — this constant
/// is intentionally duplicated per-crate rather than shared, since sharing it
/// would require a new cross-crate dependency for three bytes of literal.
const UNSEEDED_SENTINEL: &str = "\u{10FFFF}";

/// Route label used when no route template matched the request (e.g. a 404
/// on an unknown path). Bounded and fixed, never the raw concrete path.
const UNMATCHED_ROUTE: &str = "unmatched";

/// Method label used for any HTTP method outside the fixed, known set this
/// API actually handles. Bounded and fixed, mirroring [`UNMATCHED_ROUTE`]'s
/// treatment of unknown routes.
///
/// # Security
/// `http_metrics_middleware` is the outermost layer wrapping *every*
/// request, including ones that hit no route (`route = "unmatched"`). An
/// unauthenticated caller can send an arbitrary HTTP method token against a
/// nonexistent path, so `method` must be bounded independently of routing —
/// otherwise it reintroduces the same unbounded-label-cardinality class of
/// issue that `route` is already guarded against.
const UNKNOWN_METHOD: &str = "other";

/// Maps `method` to one of a fixed, bounded set of known HTTP method labels,
/// falling back to [`UNKNOWN_METHOD`] for anything else.
fn normalize_method(method: &str) -> &'static str {
    match method {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "DELETE" => "DELETE",
        "PATCH" => "PATCH",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        "CONNECT" => "CONNECT",
        "TRACE" => "TRACE",
        _ => UNKNOWN_METHOD,
    }
}

/// Histogram buckets for HTTP request duration, in seconds. Standard
/// sub-second web-latency buckets (distinct from the wider repository-creation
/// buckets, since HTTP handler-level latency for most endpoints is expected
/// to be sub-second even though the repository-creation *business operation*
/// it sometimes wraps can take up to 120s — the create_repository handler
/// itself returns quickly because event notification delivery is fire-and-forget).
pub const HTTP_REQUEST_DURATION_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Abstraction for recording per-endpoint HTTP request metrics.
///
/// # Thread Safety
/// All implementations MUST be thread-safe (Send + Sync).
pub trait HttpMetrics: Send + Sync {
    /// Records one completed HTTP request.
    ///
    /// `route` MUST be the route template, never the concrete path.
    /// `status_code` is the numeric HTTP status code (100-599).
    fn record_request(&self, method: &str, route: &str, status_code: u16, duration_seconds: f64);
}

/// Prometheus-backed implementation of [`HttpMetrics`].
///
/// # Panics
/// `new` panics if metrics cannot be registered (duplicate names against the
/// supplied registry).
pub struct PrometheusHttpMetrics {
    requests: prometheus::CounterVec,
    duration: prometheus::HistogramVec,
}

impl PrometheusHttpMetrics {
    /// Creates a new Prometheus metrics collector, registering both metric
    /// families against `registry`.
    ///
    /// # Panics
    /// Panics if metrics cannot be registered (duplicate names).
    pub fn new(registry: &prometheus::Registry) -> Self {
        use prometheus::{CounterVec, HistogramOpts, HistogramVec, Opts};

        let requests = CounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests handled"),
            &["method", "route", "status_code"],
        )
        .expect("Failed to create requests counter vec");

        let duration = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request handler duration in seconds",
            )
            .buckets(HTTP_REQUEST_DURATION_BUCKETS.to_vec()),
            &["method", "route"],
        )
        .expect("Failed to create duration histogram vec");

        registry
            .register(Box::new(requests.clone()))
            .expect("Failed to register requests counter vec");
        registry
            .register(Box::new(duration.clone()))
            .expect("Failed to register duration histogram vec");

        // Pre-seed vector metrics (see `UNSEEDED_SENTINEL` docs) so each
        // family is visible in a scrape immediately, before any real activity.
        requests.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);
        duration.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);

        Self { requests, duration }
    }
}

impl HttpMetrics for PrometheusHttpMetrics {
    fn record_request(&self, method: &str, route: &str, status_code: u16, duration_seconds: f64) {
        self.requests
            .with_label_values(&[method, route, &status_code.to_string()])
            .inc();
        self.duration
            .with_label_values(&[method, route])
            .observe(duration_seconds);
    }
}

/// No-op implementation of [`HttpMetrics`] for testing or when metrics are
/// disabled. Zero overhead; this is the complete, final implementation, not
/// a stub.
///
/// Only constructed in tests today — production always uses
/// `PrometheusHttpMetrics` (see `AppState::new`). Unlike its sibling no-op
/// types in `repo_roller_core::event_metrics`/`repository_metrics` and
/// `github_client::api_metrics` (library crates, where unused `pub` items are
/// exempt from the dead-code lint as part of the crate's public API),
/// `repo_roller_api` is a binary crate with no external consumers, so the
/// lint fires here without an explicit allow.
#[derive(Default)]
#[allow(dead_code)]
pub struct NoOpHttpMetrics;

#[allow(dead_code)]
impl NoOpHttpMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl HttpMetrics for NoOpHttpMetrics {
    fn record_request(
        &self,
        _method: &str,
        _route: &str,
        _status_code: u16,
        _duration_seconds: f64,
    ) {
    }
}

/// Axum middleware that records one [`HttpMetrics::record_request`] call per
/// completed HTTP request.
///
/// Must use [`MatchedPath`] (the route *template*, e.g.
/// `"/api/v1/orgs/{org}/templates/{template}"`) as the `route` label — never
/// `req.uri().path()` (the concrete path), which would leak unbounded
/// organisation/template names into a Prometheus label and cause label-set
/// cardinality to grow unboundedly with usage.
///
/// When no route matched (e.g. 404 on an unknown path), `route` should fall
/// back to a fixed, bounded sentinel such as `"unmatched"` rather than the
/// raw path.
pub async fn http_metrics_middleware(
    State(metrics): State<Arc<dyn HttpMetrics>>,
    req: Request,
    next: Next,
) -> Response {
    let method = normalize_method(req.method().as_str());
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str().to_string())
        .unwrap_or_else(|| UNMATCHED_ROUTE.to_string());

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration_seconds = start.elapsed().as_secs_f64();
    let status_code = response.status().as_u16();

    metrics.record_request(method, &route, status_code, duration_seconds);

    response
}

#[cfg(test)]
#[path = "http_metrics_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "http_metrics_proptests.rs"]
mod proptests;
