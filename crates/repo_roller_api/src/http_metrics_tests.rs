//! Tests for the http_metrics module (Observability Phase 1).
//!
//! Tier 1: specification tests. Tier 2: adversarial / boundary / cardinality /
//! stub-killing tests. Tier 3 (property-based) tests live in
//! `http_metrics_proptests.rs`.

use super::*;
use axum::{body::Body, http::Request as HttpRequest, routing::get, Router};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tower::ServiceExt;

// ============================================================================
// Mock implementation
// ============================================================================

#[derive(Default)]
pub struct MockHttpMetrics {
    pub call_count: AtomicU64,
    /// (method, route, status_code, duration_seconds) for every recorded request.
    pub recorded: Mutex<Vec<(String, String, u16, f64)>>,
}

impl MockHttpMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn count(&self) -> u64 {
        self.call_count.load(Ordering::Relaxed)
    }
    pub fn routes_recorded(&self) -> Vec<String> {
        self.recorded
            .lock()
            .unwrap()
            .iter()
            .map(|(_, r, _, _)| r.clone())
            .collect()
    }
}

impl HttpMetrics for MockHttpMetrics {
    fn record_request(&self, method: &str, route: &str, status_code: u16, duration_seconds: f64) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        self.recorded.lock().unwrap().push((
            method.to_string(),
            route.to_string(),
            status_code,
            duration_seconds,
        ));
    }
}

// ============================================================================
// Contract tests (step 8)
// ============================================================================

fn assert_survives_standard_call(metrics: &dyn HttpMetrics) {
    metrics.record_request("GET", "/api/v1/health", 200, 0.001);
}

#[test]
fn test_contract_noop_survives_standard_call() {
    assert_survives_standard_call(&NoOpHttpMetrics::new());
}

#[test]
fn test_contract_mock_survives_standard_call() {
    let metrics = MockHttpMetrics::new();
    assert_survives_standard_call(&metrics);
    assert_eq!(metrics.count(), 1);
}

#[test]
fn test_contract_prometheus_survives_standard_call() {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusHttpMetrics::new(&registry);
    assert_survives_standard_call(&metrics);
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_contract_all_implementations_are_send_and_sync() {
    assert_send_sync::<NoOpHttpMetrics>();
    assert_send_sync::<MockHttpMetrics>();
    assert_send_sync::<PrometheusHttpMetrics>();
}

// ============================================================================
// Tier 1: Specification tests (Prometheus-backed)
// ============================================================================

mod prometheus_spec_tests {
    use super::*;

    #[test]
    fn test_prometheus_registration_registers_both_metric_families() {
        let registry = prometheus::Registry::new();
        let _metrics = PrometheusHttpMetrics::new(&registry);

        let families = registry.gather();
        let names: Vec<String> = families.iter().map(|mf| mf.name().to_string()).collect();
        assert!(names.contains(&"http_requests_total".to_string()));
        assert!(names.contains(&"http_request_duration_seconds".to_string()));
    }

    #[test]
    fn test_record_request_increments_requests_counter_with_method_route_status_labels() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);

        metrics.record_request("GET", "/api/v1/health", 200, 0.01);
        metrics.record_request("GET", "/api/v1/health", 200, 0.02);

        let families = registry.gather();
        let value = counter_vec_value(
            &families,
            "http_requests_total",
            &[
                ("method", "GET"),
                ("route", "/api/v1/health"),
                ("status_code", "200"),
            ],
        );
        assert_eq!(value, 2.0);
    }

    #[test]
    fn test_record_request_observes_duration_histogram() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);

        metrics.record_request("POST", "/api/v1/repositories", 201, 1.5);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "http_request_duration_seconds",
            &[("method", "POST"), ("route", "/api/v1/repositories")],
        )
        .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1);
        assert!((hist.get_sample_sum() - 1.5).abs() < 0.01);
    }
}

// ============================================================================
// Tier 2: Adversarial / boundary / stub-killing tests (Prometheus-backed)
// ============================================================================

mod prometheus_adversarial_tests {
    use super::*;

    /// Distinct status codes for the same route/method must be tracked
    /// independently — a stub that ignores the status_code argument fails.
    #[test]
    fn test_distinct_status_codes_tracked_independently() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);

        metrics.record_request("GET", "/api/v1/orgs/{org}/templates", 200, 0.1);
        metrics.record_request("GET", "/api/v1/orgs/{org}/templates", 200, 0.1);
        metrics.record_request("GET", "/api/v1/orgs/{org}/templates", 500, 0.1);

        let families = registry.gather();
        let ok = counter_vec_value(
            &families,
            "http_requests_total",
            &[
                ("method", "GET"),
                ("route", "/api/v1/orgs/{org}/templates"),
                ("status_code", "200"),
            ],
        );
        let err = counter_vec_value(
            &families,
            "http_requests_total",
            &[
                ("method", "GET"),
                ("route", "/api/v1/orgs/{org}/templates"),
                ("status_code", "500"),
            ],
        );
        assert_eq!(ok, 2.0);
        assert_eq!(err, 1.0);
    }

    /// Boundary: the lowest valid HTTP status code class (1xx, e.g. 100) and
    /// the highest (5xx, e.g. 599) must both be recordable without panicking.
    #[test]
    fn test_boundary_status_codes_100_and_599_are_recorded_without_panic() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);

        metrics.record_request("GET", "/api/v1/health", 100, 0.001);
        metrics.record_request("GET", "/api/v1/health", 599, 0.001);

        let families = registry.gather();
        assert_eq!(
            counter_vec_value(
                &families,
                "http_requests_total",
                &[
                    ("method", "GET"),
                    ("route", "/api/v1/health"),
                    ("status_code", "100")
                ]
            ),
            1.0
        );
        assert_eq!(
            counter_vec_value(
                &families,
                "http_requests_total",
                &[
                    ("method", "GET"),
                    ("route", "/api/v1/health"),
                    ("status_code", "599")
                ]
            ),
            1.0
        );
    }

    /// Boundary: a duration of exactly 0.0 seconds must be recorded.
    #[test]
    fn test_zero_duration_is_recorded() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);
        metrics.record_request("GET", "/api/v1/health", 200, 0.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "http_request_duration_seconds",
            &[("method", "GET"), ("route", "/api/v1/health")],
        )
        .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1);
        assert_eq!(hist.get_sample_sum(), 0.0);
    }

    /// Different HTTP methods on the same route must not be merged into one
    /// series — a stub that ignores the method argument fails this.
    #[test]
    fn test_distinct_methods_on_same_route_tracked_independently() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusHttpMetrics::new(&registry);

        metrics.record_request("GET", "/api/v1/repositories", 200, 0.1);
        metrics.record_request("POST", "/api/v1/repositories", 201, 0.1);

        let families = registry.gather();
        let get_count = counter_vec_value(
            &families,
            "http_requests_total",
            &[
                ("method", "GET"),
                ("route", "/api/v1/repositories"),
                ("status_code", "200"),
            ],
        );
        let post_count = counter_vec_value(
            &families,
            "http_requests_total",
            &[
                ("method", "POST"),
                ("route", "/api/v1/repositories"),
                ("status_code", "201"),
            ],
        );
        assert_eq!(get_count, 1.0);
        assert_eq!(post_count, 1.0);
    }
}

// ============================================================================
// Tier 2: middleware behaviour — the critical cardinality/no-leak contract
// ============================================================================

mod middleware_tests {
    use super::*;

    fn test_router(metrics: Arc<dyn HttpMetrics>) -> Router {
        Router::new()
            .route("/orgs/{org}/templates/{template}", get(|| async { "ok" }))
            .route("/health", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                metrics.clone(),
                http_metrics_middleware,
            ))
            .with_state(metrics)
    }

    /// Adversarial (security/cardinality): the middleware MUST record the
    /// route *template* (`"/orgs/{org}/templates/{template}"`), never the
    /// concrete request path (`"/orgs/acme-corp/templates/rust-service"`).
    /// A naive implementation using `req.uri().path()` would pass a
    /// single-request test but fails this one, because it would create a
    /// distinct, unbounded label series per organisation/template name.
    #[tokio::test]
    async fn test_middleware_records_route_template_not_concrete_path() {
        let concrete = Arc::new(MockHttpMetrics::new());
        let dyn_metrics: Arc<dyn HttpMetrics> = concrete.clone();
        let app = test_router(dyn_metrics);

        let request = HttpRequest::builder()
            .uri("/orgs/acme-corp/templates/rust-service")
            .body(Body::empty())
            .unwrap();
        let _ = app.oneshot(request).await.unwrap();

        let routes = concrete.routes_recorded();
        assert_eq!(routes.len(), 1);
        assert_eq!(
            routes[0], "/orgs/{org}/templates/{template}",
            "route label must be the template, not the concrete path"
        );
        assert!(
            !routes[0].contains("acme-corp") && !routes[0].contains("rust-service"),
            "route label must never contain concrete org/template values"
        );
    }

    /// Cardinality invariant: two requests to the same route template with
    /// *different* concrete path parameter values must be recorded under the
    /// exact same route label — proving the label set does not grow with the
    /// number of distinct organisations/templates used in production.
    #[tokio::test]
    async fn test_middleware_collapses_distinct_concrete_paths_into_one_route_label() {
        let concrete = Arc::new(MockHttpMetrics::new());
        let dyn_metrics: Arc<dyn HttpMetrics> = concrete.clone();
        let app = test_router(dyn_metrics);

        for (org, template) in [("acme-corp", "rust-service"), ("other-org", "python-lib")] {
            let request = HttpRequest::builder()
                .uri(format!("/orgs/{org}/templates/{template}"))
                .body(Body::empty())
                .unwrap();
            let _ = app.clone().oneshot(request).await.unwrap();
        }

        let routes = concrete.routes_recorded();
        assert_eq!(routes.len(), 2);
        assert_eq!(
            routes[0], routes[1],
            "both requests must collapse to the same route label"
        );
        assert_eq!(routes[0], "/orgs/{org}/templates/{template}");
    }

    /// The recorded status code must reflect the actual response status.
    #[tokio::test]
    async fn test_middleware_records_actual_response_status_code() {
        let concrete = Arc::new(MockHttpMetrics::new());
        let dyn_metrics: Arc<dyn HttpMetrics> = concrete.clone();
        let app = test_router(dyn_metrics);

        let request = HttpRequest::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        let expected_status = response.status().as_u16();

        let recorded = concrete.recorded.lock().unwrap();
        assert_eq!(recorded[0].2, expected_status);
    }

    /// Duration recorded must be non-negative and finite (kills a stub that
    /// records a hardcoded or nonsensical negative/NaN duration).
    #[tokio::test]
    async fn test_middleware_records_nonnegative_finite_duration() {
        let concrete = Arc::new(MockHttpMetrics::new());
        let dyn_metrics: Arc<dyn HttpMetrics> = concrete.clone();
        let app = test_router(dyn_metrics);

        let request = HttpRequest::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let _ = app.oneshot(request).await.unwrap();

        let recorded = concrete.recorded.lock().unwrap();
        let duration = recorded[0].3;
        assert!(
            duration.is_finite() && duration >= 0.0,
            "duration must be a finite, non-negative number of seconds"
        );
    }
}

// ============================================================================
// Cross-crate shared-registry test (acceptance criterion #2)
//
// This is the only crate that can see all four Prometheus-backed metrics
// types at once: `repo_roller_core::event_metrics` and
// `repo_roller_core::repository_metrics` (via the `repo_roller_core`
// dependency), `github_client::api_metrics` (via the `github_client`
// dependency, itself also a `repo_roller_core` dependency), and this crate's
// own `http_metrics`. `repo_roller_core` cannot depend on `repo_roller_api`,
// so this specific four-way registration test cannot live anywhere else.
// ============================================================================

#[test]
fn test_all_four_metric_families_share_one_registry_without_panic() {
    let registry = prometheus::Registry::new();

    let event_metrics = repo_roller_core::event_metrics::PrometheusEventMetrics::new(&registry);
    let repo_metrics =
        repo_roller_core::repository_metrics::PrometheusRepositoryCreationMetrics::new(&registry);
    let api_metrics = github_client::api_metrics::PrometheusGitHubApiMetrics::new(&registry);
    let http_metrics = PrometheusHttpMetrics::new(&registry);

    // Exercise each so gather() has at least one sample per family.
    use github_client::api_metrics::GitHubApiMetrics as _;
    use repo_roller_core::event_metrics::EventMetrics as _;
    use repo_roller_core::repository_metrics::RepositoryCreationMetrics as _;
    event_metrics.record_delivery_success("https://example.com/webhook", 10);
    repo_metrics.record_request("acme-corp", "rust-service");
    api_metrics.record_call("get_repository");
    http_metrics.record_request("GET", "/api/v1/health", 200, 0.01);

    let families = registry.gather();
    let names: Vec<String> = families.iter().map(|mf| mf.name().to_string()).collect();

    assert!(
        names.iter().any(|n| n.starts_with("notification_")),
        "found: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.starts_with("repository_creation_")),
        "found: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.starts_with("github_api_")),
        "found: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.starts_with("http_request")),
        "found: {names:?}"
    );
}

// ============================================================================
// Tier 2: NoOp implementation tests
// ============================================================================

mod noop_tests {
    use super::*;

    #[test]
    fn test_noop_metrics_are_true_noops_and_never_panic() {
        let metrics = NoOpHttpMetrics::new();
        metrics.record_request("GET", "/api/v1/health", 200, 0.001);
    }

    #[test]
    fn test_noop_metrics_thread_safe() {
        let metrics = std::sync::Arc::new(NoOpHttpMetrics::new());
        let mut handles = vec![];
        for _ in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_request("GET", "/api/v1/health", 200, 0.001);
                }
            }));
        }
        for h in handles {
            h.join().expect("worker thread must not panic");
        }
    }
}

// ============================================================================
// Helper functions for extracting labeled metric values from gathered families
// ============================================================================

fn counter_vec_value(
    metric_families: &[prometheus::proto::MetricFamily],
    name: &str,
    labels: &[(&str, &str)],
) -> f64 {
    find_metric_with_labels(metric_families, name, labels)
        .and_then(|m| m.counter.as_ref())
        .map(|c| c.value())
        .unwrap_or(0.0)
}

fn histogram_vec<'a>(
    metric_families: &'a [prometheus::proto::MetricFamily],
    name: &str,
    labels: &[(&str, &str)],
) -> Option<&'a prometheus::proto::Histogram> {
    find_metric_with_labels(metric_families, name, labels).and_then(|m| m.histogram.as_ref())
}

fn find_metric_with_labels<'a>(
    metric_families: &'a [prometheus::proto::MetricFamily],
    name: &str,
    labels: &[(&str, &str)],
) -> Option<&'a prometheus::proto::Metric> {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)?
        .metric
        .iter()
        .find(|m| {
            labels.iter().all(|(label_name, label_value)| {
                m.label
                    .iter()
                    .any(|lp| lp.name() == *label_name && lp.value() == *label_value)
            })
        })
}
