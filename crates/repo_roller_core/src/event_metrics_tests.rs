//! Tests for event_metrics module.
//! See docs/spec/interfaces/event-metrics.md for specifications.

use super::EventMetrics;
use super::*;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

// Mock implementation for testing
#[allow(dead_code)]
pub struct MockEventMetrics {
    pub successes: AtomicU64,
    pub failures: AtomicU64,
    pub errors: AtomicU64,
    pub active_tasks: AtomicI64,
}

#[allow(dead_code)]
impl MockEventMetrics {
    pub fn new() -> Self {
        Self {
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            active_tasks: AtomicI64::new(0),
        }
    }

    pub fn success_count(&self) -> u64 {
        self.successes.load(Ordering::Relaxed)
    }

    pub fn failure_count(&self) -> u64 {
        self.failures.load(Ordering::Relaxed)
    }

    pub fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    pub fn active_task_count(&self) -> i64 {
        self.active_tasks.load(Ordering::Relaxed)
    }
}

impl Default for MockEventMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMetrics for MockEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, _duration_ms: u64) {
        self.successes.fetch_add(1, Ordering::Relaxed);
    }

    fn record_delivery_failure(&self, _endpoint_url: &str, _status_code: u16) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }

    fn record_delivery_error(&self, _endpoint_url: &str) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_active_tasks(&self) {
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_active_tasks(&self) {
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }
}

mod prometheus_metrics_tests {
    use super::*;
    use prometheus::Registry;

    #[test]
    fn test_prometheus_metrics_registration() {
        // Arrange: Create fresh registry
        let registry = Registry::new();

        // Act: Create metrics (registers with registry)
        let _metrics = PrometheusEventMetrics::new(&registry);

        // Assert: Metrics should be registered
        let metric_families = registry.gather();
        assert!(
            metric_families.len() >= 5,
            "Should register at least 5 metrics"
        );

        // Verify expected metric names exist
        let metric_names: Vec<String> = metric_families
            .iter()
            .map(|mf| mf.get_name().to_string())
            .collect();

        assert!(
            metric_names.contains(&"notification_delivery_attempts_total".to_string()),
            "Should register attempts counter"
        );
        assert!(
            metric_names.contains(&"notification_delivery_successes_total".to_string()),
            "Should register successes counter"
        );
        assert!(
            metric_names.contains(&"notification_delivery_failures_total".to_string()),
            "Should register failures counter"
        );
        assert!(
            metric_names.contains(&"notification_delivery_duration_seconds".to_string()),
            "Should register duration histogram"
        );
        assert!(
            metric_names.contains(&"notification_active_tasks".to_string()),
            "Should register active tasks gauge"
        );
    }

    #[test]
    fn test_prometheus_success_increments_counters() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Record successes
        metrics.record_delivery_success("https://example.com/webhook", 250);
        metrics.record_delivery_success("https://example.com/webhook", 100);

        // Assert: Verify counters increment
        let metric_families = registry.gather();

        let attempts = find_counter_value(&metric_families, "notification_delivery_attempts_total");
        let successes =
            find_counter_value(&metric_families, "notification_delivery_successes_total");

        assert_eq!(attempts, 2.0, "Should record 2 attempts");
        assert_eq!(successes, 2.0, "Should record 2 successes");
    }

    #[test]
    fn test_prometheus_failure_increments_counters() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Record failures
        metrics.record_delivery_failure("https://example.com/webhook", 500);
        metrics.record_delivery_failure("https://example.com/webhook", 503);

        // Assert: Verify counters
        let metric_families = registry.gather();

        let attempts = find_counter_value(&metric_families, "notification_delivery_attempts_total");
        let failures = find_counter_value(&metric_families, "notification_delivery_failures_total");

        assert_eq!(attempts, 2.0, "Should record 2 attempts");
        assert_eq!(failures, 2.0, "Should record 2 failures");
    }

    #[test]
    fn test_prometheus_error_increments_counters() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Record errors
        metrics.record_delivery_error("https://example.com/webhook");
        metrics.record_delivery_error("https://example.com/webhook");
        metrics.record_delivery_error("https://example.com/webhook");

        // Assert: Verify counters
        let metric_families = registry.gather();

        let attempts = find_counter_value(&metric_families, "notification_delivery_attempts_total");
        let failures = find_counter_value(&metric_families, "notification_delivery_failures_total");

        assert_eq!(attempts, 3.0, "Should record 3 attempts");
        assert_eq!(failures, 3.0, "Should record 3 failures for errors");
    }

    #[test]
    fn test_prometheus_histogram_records_duration() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Record successes with various durations
        metrics.record_delivery_success("https://example.com", 100); // 0.1s
        metrics.record_delivery_success("https://example.com", 500); // 0.5s
        metrics.record_delivery_success("https://example.com", 2000); // 2.0s

        // Assert: Verify histogram recorded samples
        let metric_families = registry.gather();
        let histogram = find_histogram(&metric_families, "notification_delivery_duration_seconds");

        assert!(histogram.is_some(), "Should have duration histogram");
        let hist = histogram.unwrap();
        assert_eq!(hist.get_sample_count(), 3, "Should have 3 samples");

        // Sum should be approximately 2.6 seconds (0.1 + 0.5 + 2.0)
        let sum = hist.get_sample_sum();
        assert!((sum - 2.6).abs() < 0.01, "Sum should be ~2.6s, got {}", sum);
    }

    #[test]
    fn test_prometheus_gauge_tracks_active_tasks() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Increment and decrement
        metrics.increment_active_tasks();
        metrics.increment_active_tasks();
        metrics.increment_active_tasks();

        // Assert: Should be 3
        let metric_families = registry.gather();
        let active = find_gauge_value(&metric_families, "notification_active_tasks");
        assert_eq!(active, 3.0, "Should have 3 active tasks");

        // Act: Decrement one
        metrics.decrement_active_tasks();

        // Assert: Should be 2
        let metric_families = registry.gather();
        let active = find_gauge_value(&metric_families, "notification_active_tasks");
        assert_eq!(active, 2.0, "Should have 2 active tasks after decrement");
    }

    #[test]
    fn test_prometheus_mixed_operations() {
        // Arrange
        let registry = Registry::new();
        let metrics = PrometheusEventMetrics::new(&registry);

        // Act: Mix of successes, failures, errors
        metrics.record_delivery_success("https://example.com", 100);
        metrics.record_delivery_success("https://example.com", 200);
        metrics.record_delivery_failure("https://example.com", 500);
        metrics.record_delivery_error("https://example.com");

        // Assert: Verify all counters
        let metric_families = registry.gather();

        let attempts = find_counter_value(&metric_families, "notification_delivery_attempts_total");
        let successes =
            find_counter_value(&metric_families, "notification_delivery_successes_total");
        let failures = find_counter_value(&metric_families, "notification_delivery_failures_total");

        assert_eq!(attempts, 4.0, "Should record 4 attempts total");
        assert_eq!(successes, 2.0, "Should record 2 successes");
        assert_eq!(failures, 2.0, "Should record 2 failures (1 HTTP + 1 error)");
    }

    #[test]
    fn test_prometheus_thread_safe() {
        // Arrange
        let registry = Registry::new();
        let metrics = std::sync::Arc::new(PrometheusEventMetrics::new(&registry));

        // Act: Spawn multiple threads recording metrics
        let mut handles = vec![];
        for i in 0..10 {
            let metrics_clone = metrics.clone();
            let handle = std::thread::spawn(move || {
                for _ in 0..100 {
                    if i % 2 == 0 {
                        metrics_clone.record_delivery_success("https://example.com", 100);
                    } else {
                        metrics_clone.record_delivery_failure("https://example.com", 500);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        // Assert: Should have recorded all attempts without panics
        let metric_families = registry.gather();
        let attempts = find_counter_value(&metric_families, "notification_delivery_attempts_total");
        assert_eq!(
            attempts, 1000.0,
            "Should record 1000 attempts (10 threads × 100)"
        );
    }

    // Helper functions for extracting metric values
    fn find_counter_value(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
        metric_families
            .iter()
            .find(|mf| mf.get_name() == name)
            .and_then(|mf| mf.get_metric().first())
            .map(|m| m.get_counter().get_value())
            .unwrap_or(0.0)
    }

    fn find_gauge_value(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
        metric_families
            .iter()
            .find(|mf| mf.get_name() == name)
            .and_then(|mf| mf.get_metric().first())
            .map(|m| m.get_gauge().get_value())
            .unwrap_or(0.0)
    }

    fn find_histogram<'a>(
        metric_families: &'a [prometheus::proto::MetricFamily],
        name: &str,
    ) -> Option<&'a prometheus::proto::Histogram> {
        metric_families
            .iter()
            .find(|mf| mf.get_name() == name)
            .and_then(|mf| mf.get_metric().first())
            .map(|m| m.get_histogram())
    }
}

mod noop_metrics_tests {
    use super::*;

    #[test]
    fn test_noop_metrics_are_noops() {
        let metrics = NoOpEventMetrics::new();
        metrics.record_delivery_success("https://example.com", 100);
        metrics.record_delivery_failure("https://example.com", 500);
        metrics.record_delivery_error("https://example.com");
        metrics.increment_active_tasks();
        metrics.decrement_active_tasks();
        // No panics = success
    }

    #[test]
    fn test_noop_metrics_default_constructor() {
        // Arrange & Act: Use both constructors
        let metrics1 = NoOpEventMetrics::new();
        let metrics2 = NoOpEventMetrics::new();

        // Assert: Both work identically (all no-ops)
        metrics1.record_delivery_success("https://example.com", 100);
        metrics2.record_delivery_error("https://example.com");
        // No panics = success
    }

    #[test]
    fn test_noop_metrics_thread_safe() {
        // Arrange
        let metrics = std::sync::Arc::new(NoOpEventMetrics::new());

        // Act: Spawn multiple threads
        let mut handles = vec![];
        for i in 0..10 {
            let metrics_clone = metrics.clone();
            let handle = std::thread::spawn(move || {
                for _ in 0..100 {
                    if i % 3 == 0 {
                        metrics_clone.record_delivery_success("url", 100);
                    } else if i % 3 == 1 {
                        metrics_clone.record_delivery_failure("url", 500);
                    } else {
                        metrics_clone.increment_active_tasks();
                        metrics_clone.decrement_active_tasks();
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
        // No panics = success
    }
}
