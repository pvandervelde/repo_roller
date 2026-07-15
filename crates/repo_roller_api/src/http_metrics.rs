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
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Histogram buckets for HTTP request duration, in seconds. Standard
/// sub-second web-latency buckets (distinct from the wider repository-creation
/// buckets, since HTTP handler-level latency for most endpoints is expected
/// to be sub-second even though the repository-creation *business operation*
/// it sometimes wraps can take up to 120s — the create_repository handler
/// itself returns quickly because event notification delivery is fire-and-forget).
pub const HTTP_REQUEST_DURATION_BUCKETS: [f64; 11] =
    [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

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
    pub fn new(_registry: &prometheus::Registry) -> Self {
        todo!("Coder: register http_request_* metric families against the shared registry")
    }
}

impl HttpMetrics for PrometheusHttpMetrics {
    fn record_request(&self, _method: &str, _route: &str, _status_code: u16, _duration_seconds: f64) {
        todo!("Coder: increment http_requests_total{{method,route,status_code}} and observe duration histogram")
    }
}

/// No-op implementation of [`HttpMetrics`] for testing or when metrics are
/// disabled. Zero overhead; this is the complete, final implementation, not
/// a stub.
#[derive(Default)]
pub struct NoOpHttpMetrics;

impl NoOpHttpMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl HttpMetrics for NoOpHttpMetrics {
    fn record_request(&self, _method: &str, _route: &str, _status_code: u16, _duration_seconds: f64) {}
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
///
/// # Panics (stub)
/// Stub: panics via `todo!()`. The Coder must implement this middleware.
pub async fn http_metrics_middleware(
    State(_metrics): State<Arc<dyn HttpMetrics>>,
    _req: Request,
    _next: Next,
) -> Response {
    todo!("Coder: time the request, extract MatchedPath (or \"unmatched\"), and call metrics.record_request")
}

#[cfg(test)]
#[path = "http_metrics_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "http_metrics_proptests.rs"]
mod proptests;
