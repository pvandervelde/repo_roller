// GENERATED FROM: docs/spec/interfaces/event-metrics.md
// Metrics recording abstraction for event delivery observability

//! Event delivery metrics abstraction and implementations.
//!
//! This module provides the [`EventMetrics`] trait and two implementations:
//! - [`PrometheusEventMetrics`]: production Prometheus-backed metrics
//! - [`NoOpEventMetrics`]: zero-overhead no-op for testing or disabled monitoring
//!
//! # Prometheus Dashboard Queries
//!
//! The following PromQL queries can be used to build dashboards:
//!
//! **Delivery success rate (5 min window)**:
//! ```promql
//! rate(notification_delivery_successes_total[5m])
//!   /
//! rate(notification_delivery_attempts_total[5m])
//! ```
//!
//! **Delivery failure rate (5 min window)**:
//! ```promql
//! rate(notification_delivery_failures_total[5m])
//!   /
//! rate(notification_delivery_attempts_total[5m])
//! ```
//!
//! **Average delivery latency (5 min window)**:
//! ```promql
//! rate(notification_delivery_duration_seconds_sum[5m])
//!   /
//! rate(notification_delivery_duration_seconds_count[5m])
//! ```
//!
//! **P95 delivery latency (5 min window)**:
//! ```promql
//! histogram_quantile(0.95,
//!   rate(notification_delivery_duration_seconds_bucket[5m])
//! )
//! ```
//!
//! **Active background notification tasks**:
//! ```promql
//! notification_active_tasks
//! ```
//!
//! # Recommended Grafana Panels
//!
//! - **Success Rate**: single stat + graph (`notification_delivery_successes_total`)
//! - **Failure Rate**: single stat + graph (`notification_delivery_failures_total`)
//! - **Delivery Attempts**: counter graph (`notification_delivery_attempts_total`)
//! - **Latency Percentiles**: P50, P95, P99 from `notification_delivery_duration_seconds`
//! - **Active Tasks**: gauge (`notification_active_tasks`)

/// Abstraction for recording event delivery metrics.
///
/// Implementations record metrics to various backends (Prometheus, StatsD, etc.).
///
/// # Thread Safety
/// All implementations MUST be thread-safe (Send + Sync).
///
/// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
pub trait EventMetrics: Send + Sync {
    /// Records a successful event delivery.
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    /// * `duration_ms` - Delivery duration in milliseconds
    ///
    /// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
    fn record_delivery_success(&self, endpoint_url: &str, duration_ms: u64);

    /// Records a failed event delivery with HTTP status code.
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    /// * `status_code` - HTTP response status code
    ///
    /// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
    fn record_delivery_failure(&self, endpoint_url: &str, status_code: u16);

    /// Records a delivery error (network, timeout, etc.).
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    ///
    /// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
    fn record_delivery_error(&self, endpoint_url: &str);

    /// Increments active background task counter.
    ///
    /// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
    fn increment_active_tasks(&self);

    /// Decrements active background task counter.
    ///
    /// See docs/spec/interfaces/event-metrics.md#eventmetrics-trait
    fn decrement_active_tasks(&self);
}

/// Prometheus metrics collector for event delivery.
///
/// Records metrics in Prometheus format for production monitoring.
///
/// # Metrics
/// - `notification_delivery_attempts_total` (Counter)
/// - `notification_delivery_successes_total` (Counter)
/// - `notification_delivery_failures_total` (Counter)
/// - `notification_delivery_duration_seconds` (Histogram)
/// - `notification_active_tasks` (Gauge)
///
/// See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics
pub struct PrometheusEventMetrics {
    delivery_attempts: prometheus::Counter,
    delivery_successes: prometheus::Counter,
    delivery_failures: prometheus::Counter,
    delivery_duration: prometheus::Histogram,
    active_tasks: prometheus::Gauge,
}

impl PrometheusEventMetrics {
    /// Creates a new Prometheus metrics collector.
    ///
    /// # Arguments
    /// * `registry` - Prometheus registry to register metrics with
    ///
    /// # Panics
    /// Panics if metrics cannot be registered (duplicate names)
    ///
    /// See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics
    pub fn new(registry: &prometheus::Registry) -> Self {
        use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts};

        // Define histogram buckets: 100ms, 500ms, 1s, 2.5s, 5s, 10s
        let buckets = vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0];

        // Create metrics
        let delivery_attempts = Counter::with_opts(Opts::new(
            "notification_delivery_attempts_total",
            "Total notification delivery attempts",
        ))
        .expect("Failed to create attempts counter");

        let delivery_successes = Counter::with_opts(Opts::new(
            "notification_delivery_successes_total",
            "Successful notification deliveries",
        ))
        .expect("Failed to create successes counter");

        let delivery_failures = Counter::with_opts(Opts::new(
            "notification_delivery_failures_total",
            "Failed notification deliveries",
        ))
        .expect("Failed to create failures counter");

        let delivery_duration = Histogram::with_opts(
            HistogramOpts::new(
                "notification_delivery_duration_seconds",
                "Notification delivery duration in seconds",
            )
            .buckets(buckets),
        )
        .expect("Failed to create duration histogram");

        let active_tasks = Gauge::with_opts(Opts::new(
            "notification_active_tasks",
            "Active notification delivery tasks",
        ))
        .expect("Failed to create active tasks gauge");

        // Register metrics with registry
        registry
            .register(Box::new(delivery_attempts.clone()))
            .expect("Failed to register attempts counter");
        registry
            .register(Box::new(delivery_successes.clone()))
            .expect("Failed to register successes counter");
        registry
            .register(Box::new(delivery_failures.clone()))
            .expect("Failed to register failures counter");
        registry
            .register(Box::new(delivery_duration.clone()))
            .expect("Failed to register duration histogram");
        registry
            .register(Box::new(active_tasks.clone()))
            .expect("Failed to register active tasks gauge");

        Self {
            delivery_attempts,
            delivery_successes,
            delivery_failures,
            delivery_duration,
            active_tasks,
        }
    }
}

impl EventMetrics for PrometheusEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, duration_ms: u64) {
        self.delivery_attempts.inc();
        self.delivery_successes.inc();
        // Convert milliseconds to seconds for histogram
        self.delivery_duration.observe(duration_ms as f64 / 1000.0);
    }

    fn record_delivery_failure(&self, _endpoint_url: &str, _status_code: u16) {
        self.delivery_attempts.inc();
        self.delivery_failures.inc();
    }

    fn record_delivery_error(&self, _endpoint_url: &str) {
        self.delivery_attempts.inc();
        self.delivery_failures.inc();
    }

    fn increment_active_tasks(&self) {
        self.active_tasks.inc();
    }

    fn decrement_active_tasks(&self) {
        self.active_tasks.dec();
    }
}

/// No-op metrics implementation for testing or when metrics are disabled.
///
/// All methods are no-ops with zero overhead.
///
/// See docs/spec/interfaces/event-metrics.md#noopeventmetrics
pub struct NoOpEventMetrics;

impl NoOpEventMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpEventMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMetrics for NoOpEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, _duration_ms: u64) {}
    fn record_delivery_failure(&self, _endpoint_url: &str, _status_code: u16) {}
    fn record_delivery_error(&self, _endpoint_url: &str) {}
    fn increment_active_tasks(&self) {}
    fn decrement_active_tasks(&self) {}
}

#[cfg(test)]
#[path = "event_metrics_tests.rs"]
mod tests;
