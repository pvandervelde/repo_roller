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
    fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    fn record_success(&self, duration_seconds: f64) {
        self.successes.fetch_add(1, Ordering::Relaxed);
        self.durations.lock().unwrap().push(duration_seconds);
    }

    fn record_failure(&self, error_category: &str, duration_seconds: f64) {
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
    metrics.record_request();
    metrics.increment_active_tasks();
    metrics.record_success(12.5);
    metrics.decrement_active_tasks();
    metrics.record_request();
    metrics.increment_active_tasks();
    metrics.record_failure("github", 3.2);
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
    assert_eq!(
        metrics.active_task_count(),
        0,
        "increments/decrements balanced"
    );
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
        let names: Vec<String> = metric_families
            .iter()
            .map(|mf| mf.name().to_string())
            .collect();

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

        metrics.record_request();
        metrics.record_request();

        let value = counter_value(&registry.gather(), "repository_creation_requests_total");
        assert_eq!(value, 2.0, "should record 2 requests");
    }

    #[test]
    fn test_record_success_increments_successes_counter() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success(5.0);

        let value = counter_value(&registry.gather(), "repository_creation_successes_total");
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_record_success_observes_duration_histogram() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success(45.0);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
            .expect("histogram sample should exist");
        assert_eq!(hist.get_sample_count(), 1);
        assert!((hist.get_sample_sum() - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_record_failure_increments_failures_counter_with_error_category_label() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_failure("github", 2.0);

        let value = counter_vec_value(
            &registry.gather(),
            "repository_creation_failures_total",
            &[("error_category", "github")],
        );
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_record_failure_observes_duration_histogram() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_failure("validation", 1.5);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
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

        assert_eq!(
            gauge_value(&registry.gather(), "repository_creation_active_tasks"),
            3.0
        );
    }

    #[test]
    fn test_decrement_active_tasks_decrements_gauge() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.increment_active_tasks();
        metrics.increment_active_tasks();
        metrics.decrement_active_tasks();

        assert_eq!(
            gauge_value(&registry.gather(), "repository_creation_active_tasks"),
            1.0
        );
    }

    #[test]
    fn test_histogram_buckets_include_the_documented_wide_buckets() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
        metrics.record_success(1.0);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
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
        metrics.record_success(0.0);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
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
        metrics.record_success(120.0);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
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
        metrics.record_success(999.0);

        let families = registry.gather();
        let hist = histogram_value(&families, "repository_creation_duration_seconds")
            .expect("histogram sample should exist");
        assert_eq!(
            hist.get_sample_count(),
            1,
            "over-SLA duration must still be counted"
        );
        assert!((hist.get_sample_sum() - 999.0).abs() < 0.01);
    }

    /// Stub-killing: successes and failures must be counted in genuinely
    /// separate counters. A stub that always increments "successes" (or
    /// always increments "failures") regardless of which method was called
    /// would pass a naive single-assertion test but fails this one.
    #[test]
    fn test_success_and_failure_counters_are_independent() {
        let registry = prometheus::Registry::new();
        let metrics = PrometheusRepositoryCreationMetrics::new(&registry);

        metrics.record_success(1.0);
        metrics.record_success(1.0);
        metrics.record_failure("system", 1.0);

        let families = registry.gather();
        let successes = counter_value(&families, "repository_creation_successes_total");
        let failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[("error_category", "system")],
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

        metrics.record_failure("github", 1.0);
        metrics.record_failure("validation", 1.0);
        metrics.record_failure("validation", 1.0);

        let families = registry.gather();
        let github_failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[("error_category", "github")],
        );
        let validation_failures = counter_vec_value(
            &families,
            "repository_creation_failures_total",
            &[("error_category", "validation")],
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

        assert_eq!(
            gauge_value(&registry.gather(), "repository_creation_active_tasks"),
            0.0
        );
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
                        m.record_request();
                        m.increment_active_tasks();
                        m.decrement_active_tasks();
                    } else {
                        m.record_failure("system", 0.5);
                    }
                }
            }));
        }
        for h in handles {
            h.join().expect("worker thread must not panic");
        }

        let value = counter_value(&registry.gather(), "repository_creation_requests_total");
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
            RepoRollerError::Configuration(ConfigurationError::FileNotFound { path: "x".into() }),
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

    /// Mutation-kill: asserts the *exact* category string for every
    /// `RepoRollerError` variant, not merely "is in the known set" or
    /// "differs from one other variant".
    ///
    /// `test_error_category_is_within_known_bounded_set_for_every_variant`
    /// and `test_validation_and_github_errors_map_to_different_categories`
    /// both still pass if two arms of the `error_category` match are
    /// transposed (e.g. `Validation` and `Repository` swapped) because they
    /// only check set membership and pairwise inequality, never the specific
    /// mapping. Confirmed by manual mutation during the Observability Phase 1
    /// QA audit: swapping the `Validation`/`Repository` arms left the whole
    /// `repository_metrics` test suite green. This test pins every
    /// variant -> category mapping exactly so any arm transposition fails.
    #[test]
    fn test_error_category_maps_every_variant_to_its_exact_expected_category() {
        let cases: Vec<(RepoRollerError, &str)> = vec![
            (
                RepoRollerError::Validation(ValidationError::empty_field("name")),
                "validation",
            ),
            (
                RepoRollerError::Repository(RepositoryError::CreationFailed {
                    reason: "boom".into(),
                }),
                "repository",
            ),
            (
                RepoRollerError::Configuration(ConfigurationError::FileNotFound {
                    path: "x".into(),
                }),
                "configuration",
            ),
            (
                RepoRollerError::Template(TemplateError::TemplateNotFound { name: "t".into() }),
                "template",
            ),
            (
                RepoRollerError::Authentication(AuthenticationError::InvalidToken),
                "authentication",
            ),
            (
                RepoRollerError::GitHub(crate::errors::GitHubError::RateLimitExceeded {
                    reset_at: "now".into(),
                }),
                "github",
            ),
            (
                RepoRollerError::System(SystemError::Internal {
                    reason: "oops".into(),
                }),
                "system",
            ),
            (
                RepoRollerError::Permission(crate::permissions::PermissionError::BelowBaseline {
                    permission_type: crate::permissions::PermissionType::Push,
                    level: crate::permissions::AccessLevel::None,
                    minimum_required: crate::permissions::AccessLevel::Read,
                }),
                "permission",
            ),
        ];

        for (err, expected) in &cases {
            assert_eq!(
                error_category(err),
                *expected,
                "expected {err:?} to map to category '{expected}', got '{}'",
                error_category(err)
            );
        }
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
        metrics.record_request();
        metrics.record_success(1.0);
        metrics.record_failure("system", 1.0);
        metrics.increment_active_tasks();
        metrics.decrement_active_tasks();
        // No panics = success.
    }

    #[test]
    fn test_noop_metrics_default_constructs_identically_to_new() {
        let a = NoOpRepositoryCreationMetrics::new();
        let b = NoOpRepositoryCreationMetrics::default();
        a.record_request();
        b.record_request();
    }

    #[test]
    fn test_noop_metrics_thread_safe() {
        let metrics = std::sync::Arc::new(NoOpRepositoryCreationMetrics::new());
        let mut handles = vec![];
        for _ in 0..10 {
            let m = metrics.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    m.record_request();
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
    repo_metrics.record_request();

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

/// Reads the value of a plain (unlabeled) `Counter` metric family — i.e. a
/// family with exactly one, label-less `Metric` entry. Used for
/// `repository_creation_requests_total`/`_successes_total`, which are no
/// longer `CounterVec`s (see the module-level SECURITY REMEDIATION note).
fn counter_value(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)
        .and_then(|mf| mf.metric.first())
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

/// Reads the sample of a plain (unlabeled) `Histogram` metric family. Used
/// for `repository_creation_duration_seconds`, which is no longer a
/// `HistogramVec` (see the module-level SECURITY REMEDIATION note).
fn histogram_value<'a>(
    metric_families: &'a [prometheus::proto::MetricFamily],
    name: &str,
) -> Option<&'a prometheus::proto::Histogram> {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)?
        .metric
        .first()?
        .histogram
        .as_ref()
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
