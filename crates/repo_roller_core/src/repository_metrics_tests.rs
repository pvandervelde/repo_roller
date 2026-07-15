//! Tests for the repository_metrics module (Observability Phase 1).
//!
//! Tier 1: specification tests (one per behavioural assertion in the module doc).
//! Tier 2: adversarial / boundary / stub-killing tests.
//! Tier 3 (property-based) tests live in `repository_metrics_proptests.rs`.

use super::*;
use crate::errors::{
    AuthenticationError, ConfigurationError, RepositoryError, SystemError, TemplateError,
    ValidationError,
};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;

// ============================================================================
// Mock implementation (mirrors MockEventMetrics in event_metrics_tests.rs)
// ============================================================================

/// In-memory mock used to assert *which* values were recorded and *how many
/// times*, independent of any Prometheus wiring. This lets adversarial tests
/// target the trait contract itself rather than a specific backend.
#[derive(Default)]
pub struct MockRepositoryCreationMetrics {
    pub requests: AtomicU64,
    pub successes: AtomicU64,
    pub failures: AtomicU64,
    pub active_tasks: AtomicI64,
    /// Every (organization, template) pair passed to `record_request`, in order.
    pub request_labels: Mutex<Vec<(String, String)>>,
    /// Every error_category passed to `record_failure`, in order.
    pub failure_categories: Mutex<Vec<String>>,
    /// Every duration passed to `record_success` or `record_failure`, in order.
    pub durations: Mutex<Vec<f64>>,
}

impl MockRepositoryCreationMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_count(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }
    pub fn success_count(&self) -> u64 {
        self.successes.load(Ordering::Relaxed)
    }
    pub fn failure_count(&self) -> u64 {
        self.failures.load(Ordering::Relaxed)
    }
    pub fn active_task_count(&self) -> i64 {
        self.active_tasks.load(Ordering::Relaxed)
    }
}

impl RepositoryCreationMetrics for MockRepositoryCreationMetrics {
    fn record_request(&self, organization: &str, template: &str) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.request_labels
            .lock()
            .unwrap()
            .push((organization.to_string(), template.to_string()));
    }

    fn record_success(&self, _organization: &str, _template: &str, duration_seconds: f64) {
        self.successes.fetch_add(1, Ordering::Relaxed);
        self.durations.lock().unwrap().push(duration_seconds);
    }

    fn record_failure(
        &self,
        _organization: &str,
        _template: &str,
        error_category: &str,
        duration_seconds: f64,
    ) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        self.failure_categories
            .lock()
            .unwrap()
            .push(error_category.to_string());
        self.durations.lock().unwrap().push(duration_seconds);
    }

    fn increment_active_tasks(&self) {
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_active_tasks(&self) {
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }
}

// ============================================================================
// Contract tests — every implementation must satisfy this basic behaviour.
// (Step 8: contract tests for interface abstractions.)
// ============================================================================

/// Runs a fixed call sequence against any `RepositoryCreationMetrics`
/// implementation and asserts it never panics. Used against NoOp, Mock, and
/// Prometheus (fresh registry) so the same contract is enforced for all three.
fn assert_survives_standard_call_sequence(metrics: &dyn RepositoryCreationMetrics) {
    metrics.record_request("acme-corp", "rust-service");
    metrics.increment_active_tasks();
    metrics.record_success("acme-corp", "rust-service", 12.5);
    metrics.decrement_active_tasks();
    metrics.record_request("acme-corp", "");
    metrics.increment_active_tasks();
    metrics.record_failure("acme-corp", "", "github", 3.2);
    metrics.decrement_active_tasks();
}

#[test]
fn test_contract_noop_survives_standard_call_sequence() {
    let metrics = NoOpRepositoryCreationMetrics::new();
    assert_survives_standard_call_sequence(&metrics);
}

#[test]
fn test_contract_mock_survives_standard_call_sequence() {
    let metrics = MockRepositoryCreationMetrics::new();
    assert_survives_standard_call_sequence(&metrics);
    assert_eq!(metrics.request_count(), 2);
    assert_eq!(metrics.success_count(), 1);
    assert_eq!(metrics.failure_count(), 1);
    assert_eq!(metrics.active_task_count(), 0, "increments/decrements balanced");
}

#[test]
fn test_contract_prometheus_survives_standard_call_sequence() {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
    assert_survives_standard_call_sequence(&metrics);
}

/// Compile-time contract: the trait object must be usable behind an `Arc`
/// shared across threads and stored in `AppState`-like containers.
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_contract_all_implementations_are_send_and_sync() {
    assert_send_sync::<NoOpRepositoryCreationMetrics>();
    assert_send_sync::<MockRepositoryCreationMetrics>();
    assert_send_sync::<PrometheusRepositoryCreationMetrics>();
}

// ============================================================================
// Tier 1: Specification tests (Prometheus-backed)
// ============================================================================

mod prometheus_spec_tests {
    use super::*;

    #[test]
    fn test_prometheus_registration_registers_all_five_metric_families() {
        let registry = prometheus::Registry::new();
        let _metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        let metric_families = registry.gather();
        let names: Vec<String> = metric_families.iter().map(|mf| mf.name().to_string()).collect();

        for expected in [
            "repository_creation_requests_total",
            "repository_creation_successes_total",
            "repository_creation_failures_total",
            "repository_creation_duration_seconds",
            "repository_creation_active_tasks",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "expected metric family '{expected}' to be registered, found: {names:?}"
            );
        }
    }

    #[test]
    fn test_record_request_increments_requests_counter() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_request("acme-corp", "rust-service");
        metrics.record_request("acme-corp", "rust-service");

        let value = counter_vec_value(
            &registry.gather(),
            "repository_creation_requests_total",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        );
        assert_eq!(value, 2.0, "should record 2 requests for the same org/template pair");
    }

    #[test]
    fn test_record_success_increments_successes_counter() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success("acme-corp", "rust-service", 5.0);

        let value = counter_vec_value(
            &registry.gather(),
            "repository_creation_successes_total",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        );
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_record_success_observes_duration_histogram() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success("acme-corp", "rust-service", 45.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist for this label set");
        assert_eq!(hist.get_sample_count(), 1);
        assert!((hist.get_sample_sum() - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_record_failure_increments_failures_counter_with_error_category_label() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_failure("acme-corp", "rust-service", "github", 2.0);

        let value = counter_vec_value(
            &registry.gather(),
            "repository_creation_failures_total",
            &[
                ("organization", "acme-corp"),
                ("template", "rust-service"),
                ("error_category", "github"),
            ],
        );
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_record_failure_observes_duration_histogram() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_failure("acme-corp", "rust-service", "validation", 1.5);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist even on failure path");
        assert_eq!(hist.get_sample_count(), 1);
    }

    #[test]
    fn test_increment_active_tasks_increments_gauge() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.increment_active_tasks();
        metrics.increment_active_tasks();
        metrics.increment_active_tasks();

        assert_eq!(gauge_value(&registry.gather(), "repository_creation_active_tasks"), 3.0);
    }

    #[test]
    fn test_decrement_active_tasks_decrements_gauge() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.increment_active_tasks();
        metrics.increment_active_tasks();
        metrics.decrement_active_tasks();

        assert_eq!(gauge_value(&registry.gather(), "repository_creation_active_tasks"), 1.0);
    }

    #[test]
    fn test_histogram_buckets_include_the_documented_wide_buckets() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
        metrics.record_success("acme-corp", "rust-service", 1.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist");

        let upper_bounds: Vec<f64> = hist.get_bucket().iter().map(|b| b.upper_bound()).collect();
        for expected in REPOSITORY_CREATION_DURATION_BUCKETS {
            assert!(
                upper_bounds.iter().any(|b| (b - expected).abs() < 1e-9),
                "expected bucket boundary {expected} to be present, found {upper_bounds:?}"
            );
        }
    }
}

// ============================================================================
// Tier 2: Adversarial / boundary / stub-killing tests (Prometheus-backed)
// ============================================================================

mod prometheus_adversarial_tests {
    use super::*;

    /// Boundary: duration of exactly 0.0 seconds must be recorded, not dropped
    /// or treated as "no observation".
    #[test]
    fn test_duration_of_zero_seconds_is_recorded_without_panic() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
        metrics.record_success("acme-corp", "rust-service", 0.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1);
        assert_eq!(hist.get_sample_sum(), 0.0);
    }

    /// Boundary: a duration exactly at the widest documented bucket boundary
    /// (120s, the repository-creation SLA) must fall within the +Inf bucket
    /// (i.e. not be silently clamped or rejected).
    #[test]
    fn test_duration_at_120_seconds_boundary_is_recorded() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
        metrics.record_success("acme-corp", "rust-service", 120.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1);
    }

    /// Boundary N+1: a duration exceeding every finite bucket must still be
    /// recorded (it lands in the +Inf bucket) rather than causing a panic or
    /// being silently dropped.
    #[test]
    fn test_duration_exceeding_widest_bucket_is_still_recorded() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
        metrics.record_success("acme-corp", "rust-service", 999.0);

        let families = registry.gather();
        let hist = histogram_vec(
            &families,
            "repository_creation_duration_seconds",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        )
        .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1, "over-SLA duration must still be counted");
        assert!((hist.get_sample_sum() - 999.0).abs() < 0.01);
    }

    /// Side-effect isolation: metrics for one organization/template pair must
    /// not leak into another pair's counters. Kills a stub that hardcodes a
    /// single global counter regardless of label arguments.
    #[test]
    fn test_distinct_organizations_are_tracked_independently() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_request("org-a", "template-x");
        metrics.record_request("org-a", "template-x");
        metrics.record_request("org-b", "template-x");

        let families = registry.gather();
        let org_a = counter_vec_value(
            &families,
            "repository_creation_requests_total",
            &[("organization", "org-a"), ("template", "template-x")],
        );
        let org_b = counter_vec_value(
            &families,
            "repository_creation_requests_total",
            &[("organization", "org-b"), ("template", "template-x")],
        );
        assert_eq!(org_a, 2.0);
        assert_eq!(org_b, 1.0, "org-b's single request must not be merged into org-a's count");
    }

    /// Stub-killing: successes and failures must be counted in genuinely
    /// separate counters. A stub that always increments "successes" (or
    /// always increments "failures") regardless of which method was called
    /// would pass a naive single-assertion test but fails this one.
    #[test]
    fn test_success_and_failure_counters_are_independent() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success("acme-corp", "rust-service", 1.0);
        metrics.record_success("acme-corp", "rust-service", 1.0);
        metrics.record_failure("acme-corp", "rust-service", "system", 1.0);

        let families = registry.gather();
        let successes = counter_vec_value(
            &families,
            "repository_creation_successes_total",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        );
        let failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[
                ("organization", "acme-corp"),
                ("template", "rust-service"),
                ("error_category", "system"),
            ],
        );
        assert_eq!(successes, 2.0);
        assert_eq!(failures, 1.0);
    }

    /// Stub-killing: distinct error categories must not be collapsed into one
    /// label value. A stub that hardcodes `"error"` regardless of the
    /// `error_category` argument would fail this test.
    #[test]
    fn test_distinct_error_categories_produce_distinct_label_series() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_failure("acme-corp", "rust-service", "github", 1.0);
        metrics.record_failure("acme-corp", "rust-service", "validation", 1.0);
        metrics.record_failure("acme-corp", "rust-service", "validation", 1.0);

        let families = registry.gather();
        let github_failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[
                ("organization", "acme-corp"),
                ("template", "rust-service"),
                ("error_category", "github"),
            ],
        );
        let validation_failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[
                ("organization", "acme-corp"),
                ("template", "rust-service"),
                ("error_category", "validation"),
            ],
        );
        assert_eq!(github_failures, 1.0);
        assert_eq!(validation_failures, 2.0);
    }

    /// Active-tasks gauge must never go negative when increments and
    /// decrements are balanced, even under interleaving.
    #[test]
    fn test_active_tasks_gauge_returns_to_zero_after_balanced_operations() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        for _ in 0..50 {
            metrics.increment_active_tasks();
        }
        for _ in 0..50 {
            metrics.decrement_active_tasks();
        }

        assert_eq!(gauge_value(&registry.gather(), "repository_creation_active_tasks"), 0.0);
    }

    /// Thread safety: concurrent recording from many threads must not panic
    /// and must account for every call (mirrors `test_prometheus_thread_safe`
    /// in event_metrics_tests.rs).
    #[test]
    fn test_prometheus_metrics_are_thread_safe_under_concurrent_load() {
        let registry = prometheus::Registry::new();
        let metrics = std::sync::Arc::new(PrometheusRepositoryCreationMetrics::new(&registry));

        let mut handles = vec![];
        for i in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    if i % 2 == 0 {
                        m.record_request("acme-corp", "rust-service");
                        m.increment_active_tasks();
                        m.decrement_active_tasks();
                    } else {
                        m.record_failure("acme-corp", "rust-service", "system", 0.5);
                    }
                }
            }));
        }
        for h in handles {
            h.join().expect("worker thread must not panic");
        }

        let value = counter_vec_value(
            &registry.gather(),
            "repository_creation_requests_total",
            &[("organization", "acme-corp"), ("template", "rust-service")],
        );
        assert_eq!(value, 500.0, "5 threads x 100 requests each");
    }
}

// ============================================================================
// Tier 2: `error_category` mapping — bounded-label / no-secret-leak tests
// ============================================================================

mod error_category_tests {
    use super::*;

    /// Every currently-defined `RepoRollerError` variant must map to a member
    /// of the documented bounded set. This is an exhaustiveness net: if a new
    /// `RepoRollerError` variant is added without updating `error_category`,
    /// a `todo!()`/`unreachable!()` panic (or an out-of-set string) will fail
    /// this test rather than silently leaking a `Debug`-formatted variant name.
    #[test]
    fn test_error_category_is_within_known_bounded_set_for_every_variant() {
        let samples: Vec<RepoRollerError> = vec![
            RepoRollerError::Validation(ValidationError::empty_field("name")),
            RepoRollerError::Repository(RepositoryError::CreationFailed {
                reason: "boom".into(),
            }),
            RepoRollerError::Configuration(ConfigurationError::FileNotFound {
                path: "x".into(),
            }),
            RepoRollerError::Template(TemplateError::TemplateNotFound { name: "t".into() }),
            RepoRollerError::Authentication(AuthenticationError::InvalidToken),
            RepoRollerError::GitHub(crate::errors::GitHubError::RateLimitExceeded {
                reset_at: "now".into(),
            }),
            RepoRollerError::System(SystemError::Internal {
                reason: "oops".into(),
            }),
        ];

        for err in &samples {
            let category = error_category(err);
            assert!(
                KNOWN_ERROR_CATEGORIES.contains(&category),
                "category '{category}' for {err:?} must be a member of KNOWN_ERROR_CATEGORIES"
            );
        }
    }

    /// Security: the category label must never contain the free-text `reason`
    /// field embedded in the error. A naive implementation using
    /// `err.to_string()` (or a `Debug` dump) as the label would leak
    /// arbitrary, unbounded, potentially sensitive text into a Prometheus
    /// label — this test catches exactly that mistake.
    #[test]
    fn test_error_category_never_contains_the_embedded_free_text_reason() {
        let sensitive_marker = "super-secret-installation-token-value-12345";
        let err = RepoRollerError::System(SystemError::Internal {
            reason: sensitive_marker.to_string(),
        });

        let category = error_category(&err);
        assert!(
            !category.contains(sensitive_marker),
            "category label leaked the embedded free-text reason"
        );
        assert!(
            !category.contains("secret") && !category.contains("token"),
            "category label must not echo fragments of the free-text reason"
        );
    }

    /// Boundary: distinct `RepoRollerError` top-level variants map to
    /// distinct categories (not all collapsed into e.g. "error").
    #[test]
    fn test_validation_and_github_errors_map_to_different_categories() {
        let validation_err = RepoRollerError::Validation(ValidationError::empty_field("name"));
        let github_err = RepoRollerError::GitHub(crate::errors::GitHubError::NetworkError {
            reason: "timeout".into(),
        });

        assert_ne!(error_category(&validation_err), error_category(&github_err));
    }
}

// ============================================================================
// Tier 2: NoOp implementation tests
// ============================================================================

mod noop_tests {
    use super::*;

    #[test]
    fn test_noop_metrics_are_true_noops_and_never_panic() {
        let metrics = NoOpRepositoryCreationMetrics::new();
        metrics.record_request("org", "template");
        metrics.record_success("org", "template", 1.0);
        metrics.record_failure("org", "template", "system", 1.0);
        metrics.increment_active_tasks();
        metrics.decrement_active_tasks();
        // No panics = success.
    }

    #[test]
    fn test_noop_metrics_default_constructs_identically_to_new() {
        let a = NoOpRepositoryCreationMetrics::new();
        let b = NoOpRepositoryCreationMetrics::default();
        a.record_request("org", "template");
        b.record_request("org", "template");
    }

    #[test]
    fn test_noop_metrics_thread_safe() {
        let metrics = std::sync::Arc::new(NoOpRepositoryCreationMetrics::new());
        let mut handles = vec![];
        for _ in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_request("org", "template");
                    m.increment_active_tasks();
                    m.decrement_active_tasks();
                }
            }));
        }
        for h in handles {
            h.join().expect("worker thread must not panic");
        }
    }
}

// ============================================================================
// Cross-metrics registration test (acceptance criterion #2)
// ============================================================================

/// Registering `PrometheusEventMetrics` and `PrometheusRepositoryCreationMetrics`
/// against the *same* shared registry (as `AppState::new` must do in
/// production) must not panic, and both families must be independently
/// gatherable in a single scrape.
#[test]
fn test_event_metrics_and_repository_creation_metrics_share_one_registry_without_panic() {
    use crate::event_metrics::EventMetrics as _;

    let registry = prometheus::Registry::new();
    let event_metrics = crate::event_metrics::PrometheusEventMetrics::new(&registry);
    let repo_metrics = PrometheusRepositoryCreationMetrics::new(&registry);

    event_metrics.record_delivery_success("https://example.com/webhook", 100);
    repo_metrics.record_request("acme-corp", "rust-service");

    let families = registry.gather();
    let names: Vec<String> = families.iter().map(|mf| mf.name().to_string()).collect();
    assert!(names.contains(&"notification_delivery_attempts_total".to_string()));
    assert!(names.contains(&"repository_creation_requests_total".to_string()));
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
                m.label.iter().any(|lp| {
                    lp.name() == *label_name && lp.value() == *label_value
                })
            })
        })
}
