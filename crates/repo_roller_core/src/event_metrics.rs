// GENERATED FROM: docs/spec/interfaces/event-metrics.md
// Metrics recording abstraction for event delivery observability

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
    _delivery_attempts: prometheus::Counter,
    _delivery_successes: prometheus::Counter,
    _delivery_failures: prometheus::Counter,
    _delivery_duration: prometheus::Histogram,
    _active_tasks: prometheus::Gauge,
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
    pub fn new(_registry: &prometheus::Registry) -> Self {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
    }
}

impl EventMetrics for PrometheusEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, _duration_ms: u64) {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
    }

    fn record_delivery_failure(&self, _endpoint_url: &str, _status_code: u16) {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
    }

    fn record_delivery_error(&self, _endpoint_url: &str) {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
    }

    fn increment_active_tasks(&self) {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
    }

    fn decrement_active_tasks(&self) {
        unimplemented!("See docs/spec/interfaces/event-metrics.md#prometheuseventmetrics")
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
