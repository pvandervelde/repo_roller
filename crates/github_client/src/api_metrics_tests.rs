//! Tests for the api_metrics module (Observability Phase 1).
//!
//! Tier 1: specification tests. Tier 2: adversarial / boundary / stub-killing
//! tests. Tier 3 (property-based) tests live in `api_metrics_proptests.rs`.

use super::*;
use crate::errors::Error;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;

// ============================================================================
// Mock implementation
// ============================================================================

#[derive(Default)]
pub struct MockGitHubApiMetrics {
    pub calls: AtomicU64,
    pub errors: AtomicU64,
    pub rate_limit_remaining: AtomicI64,
    pub call_operations: Mutex<Vec<String>>,
    pub error_categories: Mutex<Vec<String>>,
}

impl MockGitHubApiMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn call_count(&self) -> u64 {
        self.calls.load(Ordering::Relaxed)
    }
    pub fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
    pub fn rate_limit(&self) -> i64 {
        self.rate_limit_remaining.load(Ordering::Relaxed)
    }
}

impl GitHubApiMetrics for MockGitHubApiMetrics {
    fn record_call(&self, operation: &str) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.call_operations.lock().unwrap().push(operation.to_string());
    }

    fn record_error(&self, _operation: &str, status_category: &str) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.error_categories.lock().unwrap().push(status_category.to_string());
    }

    fn set_rate_limit_remaining(&self, remaining: i64) {
        self.rate_limit_remaining.store(remaining, Ordering::Relaxed);
    }
}

// ============================================================================
// Contract tests (step 8): every implementation must satisfy this behaviour.
// ============================================================================

fn assert_survives_standard_call_sequence(metrics: &dyn GitHubApiMetrics) {
    metrics.record_call("get_repository");
    metrics.record_error("get_repository", "not_found");
    metrics.set_rate_limit_remaining(4999);
    metrics.set_rate_limit_remaining(4998);
}

#[test]
fn test_contract_noop_survives_standard_call_sequence() {
    assert_survives_standard_call_sequence(&NoOpGitHubApiMetrics::new());
}

#[test]
fn test_contract_mock_survives_standard_call_sequence() {
    let metrics = MockGitHubApiMetrics::new();
    assert_survives_standard_call_sequence(&metrics);
    assert_eq!(metrics.call_count(), 1);
    assert_eq!(metrics.error_count(), 1);
    assert_eq!(metrics.rate_limit(), 4998, "last-set value wins, not first");
}

#[test]
fn test_contract_prometheus_survives_standard_call_sequence() {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusGitHubApiMetrics::new(&registry);
    assert_survives_standard_call_sequence(&metrics);
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_contract_all_implementations_are_send_and_sync() {
    assert_send_sync::<NoOpGitHubApiMetrics>();
    assert_send_sync::<MockGitHubApiMetrics>();
    assert_send_sync::<PrometheusGitHubApiMetrics>();
}

// ============================================================================
// Tier 1: Specification tests (Prometheus-backed)
// ============================================================================

mod prometheus_spec_tests {
    use super::*;

    #[test]
    fn test_prometheus_registration_registers_all_three_metric_families() {
        let registry = prometheus::Registry::new();
        let _metrics = PrometheusGitHubApiMetrics::new(&registry);

        let families = registry.gather();
        let names: Vec<String> = families.iter().map(|mf| mf.name().to_string()).collect();
        for expected in [
            "github_api_calls_total",
            "github_api_errors_total",
            "github_api_rate_limit_remaining",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "expected metric family '{expected}' registered, found {names:?}"
            );
        }
    }

    #[test]
    fn test_record_call_increments_calls_counter_for_operation() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.record_call("get_repository");
        metrics.record_call("get_repository");

        let families = registry.gather();
        let value = counter_vec_value(&families, "github_api_calls_total", &[("operation", "get_repository")]);
        assert_eq!(value, 2.0);
    }

    #[test]
    fn test_record_error_increments_errors_counter_with_status_category_label() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.record_error("create_org_repository", "rate_limit_exceeded");

        let families = registry.gather();
        let value = counter_vec_value(
            &families,
            "github_api_errors_total",
            &[
                ("operation", "create_org_repository"),
                ("status_category", "rate_limit_exceeded"),
            ],
        );
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_set_rate_limit_remaining_sets_gauge_value() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.set_rate_limit_remaining(4321);

        assert_eq!(gauge_value(&registry.gather(), "github_api_rate_limit_remaining"), 4321.0);
    }
}

// ============================================================================
// Tier 2: Adversarial / boundary / stub-killing tests (Prometheus-backed)
// ============================================================================

mod prometheus_adversarial_tests {
    use super::*;

    /// Gauges must reflect the *last* set value, not the first, sum, or max —
    /// `set_rate_limit_remaining` is a point-in-time overwrite, not an
    /// accumulator. This kills a stub that mistakenly uses `.add()`/`.inc()`.
    #[test]
    fn test_rate_limit_gauge_reflects_last_value_not_sum() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.set_rate_limit_remaining(5000);
        metrics.set_rate_limit_remaining(10);
        metrics.set_rate_limit_remaining(4500);

        let value = gauge_value(&registry.gather(), "github_api_rate_limit_remaining");
        assert_eq!(value, 4500.0, "gauge must equal the last-set value, not 5000+10+4500");
    }

    /// Boundary: rate limit remaining of exactly 0 (fully exhausted) must be
    /// representable — a stub treating 0 as "unset"/falsy would fail this.
    #[test]
    fn test_rate_limit_gauge_can_be_set_to_zero() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.set_rate_limit_remaining(100);
        metrics.set_rate_limit_remaining(0);

        assert_eq!(gauge_value(&registry.gather(), "github_api_rate_limit_remaining"), 0.0);
    }

    /// Distinct operations must be tracked independently — a stub that
    /// collapses all operations into one counter fails this.
    #[test]
    fn test_distinct_operations_tracked_independently() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.record_call("get_repository");
        metrics.record_call("get_repository");
        metrics.record_call("list_installations");

        let families = registry.gather();
        let get_repo = counter_vec_value(&families, "github_api_calls_total", &[("operation", "get_repository")]);
        let list_inst =
            counter_vec_value(&families, "github_api_calls_total", &[("operation", "list_installations")]);
        assert_eq!(get_repo, 2.0);
        assert_eq!(list_inst, 1.0);
    }

    /// Calls and errors are independent counters: recording N calls and M
    /// errors must not cause the calls counter to include the errors (or
    /// vice versa). Kills a stub that increments both counters on every call.
    #[test]
    fn test_calls_and_errors_counters_are_independent() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusGitHubApiMetrics::new(&registry);

        metrics.record_call("get_repository");
        metrics.record_call("get_repository");
        metrics.record_call("get_repository");
        metrics.record_error("get_repository", "not_found");

        let families = registry.gather();
        let calls = counter_vec_value(&families, "github_api_calls_total", &[("operation", "get_repository")]);
        let errors = counter_vec_value(
            &families,
            "github_api_errors_total",
            &[("operation", "get_repository"), ("status_category", "not_found")],
        );
        assert_eq!(calls, 3.0, "record_error must not also increment the calls counter");
        assert_eq!(errors, 1.0);
    }

    #[test]
    fn test_prometheus_metrics_are_thread_safe_under_concurrent_load() {
        let registry = prometheus::Registry::new();
        let metrics = std::sync::Arc::new(PrometheusGitHubApiMetrics::new(&registry));

        let mut handles = vec![];
        for i in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_call("get_repository");
                    if i % 2 == 0 {
                        m.set_rate_limit_remaining(i);
                    }
                }
            }));
        }
        for h in handles {
            h.join().expect("worker thread must not panic");
        }

        let families = registry.gather();
        let calls = counter_vec_value(&families, "github_api_calls_total", &[("operation", "get_repository")]);
        assert_eq!(calls, 1000.0);
    }
}

// ============================================================================
// Tier 2: `status_category` mapping — bounded-label / no-secret-leak tests
// ============================================================================

mod status_category_tests {
    use super::*;

    #[test]
    fn test_status_category_is_within_known_bounded_set_for_every_variant() {
        let samples: Vec<Error> = vec![
            Error::ApiError(),
            Error::AuthError("token invalid".into()),
            Error::FailedToCreateAccessToken("owner".into(), "repo".into(), 42),
            Error::FailedToFindAppInstallation("owner".into(), "repo".into(), 42),
            Error::InvalidResponse,
            Error::NotFound,
            Error::RateLimitExceeded,
        ];

        for err in &samples {
            let category = status_category(err);
            assert!(
                KNOWN_STATUS_CATEGORIES.contains(&category),
                "category '{category}' for {err:?} must be a member of KNOWN_STATUS_CATEGORIES"
            );
        }
    }

    /// Security: the category must never leak the free-text detail embedded
    /// in `AuthError(String)`, `FailedToCreateAccessToken(owner, repo, app_id)`,
    /// etc. A naive `err.to_string()`-based mapping would leak org/repo names
    /// (and, in upstream error-wrapping cases, potentially token fragments)
    /// into an unbounded Prometheus label.
    #[test]
    fn test_status_category_never_leaks_embedded_free_text() {
        let sensitive = "owner-with-secret-installation-token-abc123";
        let err = Error::AuthError(sensitive.to_string());

        let category = status_category(&err);
        assert!(!category.contains(sensitive), "category leaked the embedded auth error text");
        assert!(!category.contains("token"), "category must not echo the free-text detail");
    }

    #[test]
    fn test_not_found_and_rate_limit_exceeded_map_to_different_categories() {
        assert_ne!(status_category(&Error::NotFound), status_category(&Error::RateLimitExceeded));
    }
}

// ============================================================================
// Tier 2: NoOp implementation tests
// ============================================================================

mod noop_tests {
    use super::*;

    #[test]
    fn test_noop_metrics_are_true_noops_and_never_panic() {
        let metrics = NoOpGitHubApiMetrics::new();
        metrics.record_call("get_repository");
        metrics.record_error("get_repository", "not_found");
        metrics.set_rate_limit_remaining(100);
    }

    #[test]
    fn test_noop_metrics_thread_safe() {
        let metrics = std::sync::Arc::new(NoOpGitHubApiMetrics::new());
        let mut handles = vec![];
        for _ in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_call("get_repository");
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

fn gauge_value(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)
        .and_then(|mf| mf.metric.first())
        .and_then(|m| m.gauge.as_ref())
        .map(|g| g.value())
        .unwrap_or(0.0)
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
            labels
                .iter()
                .all(|(label_name, label_value)| m.label.iter().any(|lp| lp.name() == *label_name && lp.value() == *label_value))
        })
}
