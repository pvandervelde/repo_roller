# Event Metrics Interface

**Architectural Layer**: Infrastructure Abstraction
**Module Path**: `repo_roller_core/src/event_metrics.rs`
**Specification**: [outbound-event-notifications.md](../design/outbound-event-notifications.md#metrics-collection)

## Overview

The EventMetrics trait defines the interface for recording event delivery metrics. This abstraction allows the business logic to record observability data without depending on specific metrics systems (Prometheus, StatsD, CloudWatch, etc.).

## Responsibilities (RDD)

**Knows**:

- Metric names and types
- Recording semantics

**Does**:

- Records delivery attempts, successes, failures
- Records delivery latency
- Tracks active background tasks

**Collaborates With**:

- EventPublisher (producer of metrics)
- Metrics backend (consumer of metrics)

## Dependencies

### External Dependencies

- Implementation-specific (varies by metrics backend)
- `prometheus` for Prometheus implementation
- No dependencies for NoOp implementation

## Public Interface

### EventMetrics Trait

**Purpose**: Abstraction for recording event delivery metrics.

**Signature**:

```rust
pub trait EventMetrics: Send + Sync {
    /// Records a successful event delivery.
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    /// * `duration_ms` - Delivery duration in milliseconds
    fn record_delivery_success(&self, endpoint_url: &str, duration_ms: u64);

    /// Records a failed event delivery with HTTP status code.
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    /// * `status_code` - HTTP response status code
    fn record_delivery_failure(&self, endpoint_url: &str, status_code: u16);

    /// Records a delivery error (network, timeout, etc.).
    ///
    /// # Arguments
    /// * `endpoint_url` - Destination URL (for labeling)
    fn record_delivery_error(&self, endpoint_url: &str);

    /// Increments active background task counter.
    fn increment_active_tasks(&self);

    /// Decrements active background task counter.
    fn decrement_active_tasks(&self);
}
```

**Contract**:

- MUST be thread-safe (Send + Sync)
- MUST be non-blocking (metrics recording should not block delivery)
- MUST NOT panic on metric recording failures
- SHOULD aggregate by endpoint URL where applicable

**Behavior**:

- Counters increment monotonically
- Histograms record duration distributions
- Gauges track current state (active tasks)
- Failed metric recording logged but does not affect functionality

## Implementations

### PrometheusEventMetrics

**Purpose**: Records metrics in Prometheus format.

**Use Case**: Production deployments with Prometheus monitoring

**Signature**:

```rust
pub struct PrometheusEventMetrics {
    delivery_attempts: Counter,
    delivery_successes: Counter,
    delivery_failures: Counter,
    delivery_duration: Histogram,
    active_tasks: Gauge,
}

impl PrometheusEventMetrics {
    /// Creates a new Prometheus metrics collector.
    ///
    /// # Arguments
    /// * `registry` - Prometheus registry to register metrics with
    ///
    /// # Panics
    /// Panics if metrics cannot be registered (duplicate names)
    pub fn new(registry: &Registry) -> Self;
}
```

**Metrics Definitions**:

| Metric Name | Type | Description | Labels |
|-------------|------|-------------|--------|
| `notification_delivery_attempts_total` | Counter | Total notification delivery attempts | - |
| `notification_delivery_successes_total` | Counter | Successful notification deliveries | - |
| `notification_delivery_failures_total` | Counter | Failed notification deliveries | - |
| `notification_delivery_duration_seconds` | Histogram | Notification delivery duration | - |
| `notification_active_tasks` | Gauge | Active notification delivery tasks | - |

**Histogram Buckets**:

```rust
vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0]
// 100ms, 500ms, 1s, 2.5s, 5s, 10s
```

**Example**:

```rust
use prometheus::Registry;

let registry = Registry::new();
let metrics = PrometheusEventMetrics::new(&registry);

// Record successful delivery
metrics.record_delivery_success("https://example.com/webhook", 250);

// Record failed delivery
metrics.record_delivery_failure("https://example.com/webhook", 500);
```

**Implementation Details**:

```rust
impl EventMetrics for PrometheusEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, duration_ms: u64) {
        self.delivery_attempts.inc();
        self.delivery_successes.inc();
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
```

**Note**: Initial implementation does not include endpoint URL labels. This can be added in a future enhancement if per-endpoint metrics are needed.

### NoOpEventMetrics

**Purpose**: No-op implementation for testing or when metrics are disabled.

**Use Case**: Local development, testing, environments without metrics infrastructure

**Signature**:

```rust
pub struct NoOpEventMetrics;

impl NoOpEventMetrics {
    pub fn new() -> Self {
        Self
    }
}
```

**Behavior**:

- All methods are no-ops
- Zero overhead
- Safe to use anywhere

**Example**:

```rust
let metrics = NoOpEventMetrics::new();
metrics.record_delivery_success("...", 100); // Does nothing
```

**Implementation**:

```rust
impl EventMetrics for NoOpEventMetrics {
    fn record_delivery_success(&self, _endpoint_url: &str, _duration_ms: u64) {}
    fn record_delivery_failure(&self, _endpoint_url: &str, _status_code: u16) {}
    fn record_delivery_error(&self, _endpoint_url: &str) {}
    fn increment_active_tasks(&self) {}
    fn decrement_active_tasks(&self) {}
}
```

## Integration Pattern

### Initialization

The application creates a metrics collector at startup:

```rust
// In main.rs or application initialization

use prometheus::Registry;

let registry = Registry::new();
let metrics: Arc<dyn EventMetrics> = Arc::new(PrometheusEventMetrics::new(&registry));

// Expose metrics endpoint
let metrics_app = axum::Router::new()
    .route("/metrics", get(|| async move {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        buffer
    }));
```

### Usage in EventPublisher

```rust
// In event_publisher.rs

async fn deliver_to_endpoint(
    endpoint: &NotificationEndpoint,
    metrics: &dyn EventMetrics,
    // ... other params
) -> DeliveryResult {
    let start = std::time::Instant::now();

    // Send HTTP request
    match client.post(&endpoint.url).send().await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            if response.status().is_success() {
                metrics.record_delivery_success(&endpoint.url, duration_ms);
                DeliveryResult { success: true, /* ... */ }
            } else {
                metrics.record_delivery_failure(&endpoint.url, response.status().as_u16());
                DeliveryResult { success: false, /* ... */ }
            }
        }
        Err(e) => {
            metrics.record_delivery_error(&endpoint.url);
            DeliveryResult { success: false, /* ... */ }
        }
    }
}
```

### Background Task Tracking

```rust
// In event_publisher.rs

pub async fn publish_repository_created(/* ... */) -> Vec<DeliveryResult> {
    metrics.increment_active_tasks();

    // Ensure decrement happens even if panic
    let _guard = scopeguard::guard(metrics, |m| {
        m.decrement_active_tasks();
    });

    // Perform delivery...

    // Guard drops automatically, decrementing counter
}
```

## Metrics Dashboard

### Prometheus Queries

**Delivery Success Rate**:

```promql
rate(notification_delivery_successes_total[5m])
  /
rate(notification_delivery_attempts_total[5m])
```

**Delivery Failure Rate**:

```promql
rate(notification_delivery_failures_total[5m])
  /
rate(notification_delivery_attempts_total[5m])
```

**Average Delivery Latency**:

```promql
rate(notification_delivery_duration_seconds_sum[5m])
  /
rate(notification_delivery_duration_seconds_count[5m])
```

**P95 Delivery Latency**:

```promql
histogram_quantile(0.95,
  rate(notification_delivery_duration_seconds_bucket[5m])
)
```

**Active Background Tasks**:

```promql
notification_active_tasks
```

### Grafana Dashboard

Recommended panels:

- **Success Rate** (single stat + graph)
- **Failure Rate** (single stat + graph)
- **Delivery Attempts** (counter graph)
- **Latency Percentiles** (P50, P95, P99)
- **Active Tasks** (gauge)

## Testing Strategy

### Unit Tests

**PrometheusEventMetrics**:

- ✅ Metrics registered correctly
- ✅ Counters increment
- ✅ Histograms record values
- ✅ Gauges increment/decrement
- ✅ Thread-safe concurrent access

**NoOpEventMetrics**:

- ✅ All methods are no-ops
- ✅ No panics

### Integration Tests

**Mock EventMetrics**:

```rust
pub struct MockEventMetrics {
    pub successes: AtomicU64,
    pub failures: AtomicU64,
    pub errors: AtomicU64,
    pub active_tasks: AtomicI64,
}

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

impl EventMetrics for MockEventMetrics {
    fn record_delivery_success(&self, _: &str, _: u64) {
        self.successes.fetch_add(1, Ordering::Relaxed);
    }

    fn record_delivery_failure(&self, _: &str, _: u16) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }

    fn record_delivery_error(&self, _: &str) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_active_tasks(&self) {
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_active_tasks(&self) {
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }
}
```

**Usage in Tests**:

```rust
#[tokio::test]
async fn test_metrics_recorded_on_delivery() {
    let metrics = Arc::new(MockEventMetrics::new());

    // Perform delivery
    let result = publish_repository_created(
        &result,
        &request,
        &config,
        "user",
        &config_manager,
        &secret_resolver,
        &metrics,
    ).await;

    // Verify metrics
    assert_eq!(metrics.success_count(), 1);
    assert_eq!(metrics.failure_count(), 0);
    assert_eq!(metrics.active_task_count(), 0); // Task completed
}
```

## Performance Considerations

**Latency**:

- Metric recording: < 100μs (microseconds)
- Non-blocking (in-memory updates)
- Negligible impact on delivery latency

**Resource Usage**:

- Memory: O(1) per metric
- CPU: Minimal (atomic operations)
- No network calls

**Thread Safety**:

- All implementations use atomic operations
- No locks required
- Safe for concurrent access

**Overhead**:

- PrometheusEventMetrics: ~100 bytes per metric
- NoOpEventMetrics: Zero runtime overhead

## Future Enhancements

Out of scope for initial implementation:

- Per-endpoint labels (requires label management)
- Per-event-type labels
- Metric aggregation by organization/team
- Custom metric exporters (StatsD, CloudWatch)
- Metric retention policies
- Alert rule recommendations

## References

- [Event Publisher Interface](event-publisher.md)
- [Design Document](../design/outbound-event-notifications.md#metrics-collection)
- [Prometheus Documentation](https://prometheus.io/docs/concepts/metric_types/)
