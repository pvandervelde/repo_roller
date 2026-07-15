// Property-based tests for http_metrics.
//
// These tests use proptest to verify invariants that hold for ALL inputs
// within a given domain, not just hand-picked examples.

use super::*;
use proptest::prelude::*;

fn fresh() -> (prometheus::Registry, PrometheusHttpMetrics) {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusHttpMetrics::new(&registry);
    (registry, metrics)
}

fn counter_total(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)
        .map(|mf| mf.metric.iter().filter_map(|m| m.counter.as_ref().map(|c| c.value())).sum())
        .unwrap_or(0.0)
}

proptest! {
    /// For any valid HTTP status code (100-599) and any non-negative
    /// duration, recording a request must never panic and must increment the
    /// total request counter by exactly 1.
    #[test]
    fn prop_any_valid_status_code_is_recorded_without_panic(
        status_code in 100u16..600,
        duration in 0.0f64..300.0,
    ) {
        let (registry, metrics) = fresh();
        metrics.record_request("GET", "/api/v1/health", status_code, duration);
        prop_assert_eq!(counter_total(&registry.gather(), "http_requests_total"), 1.0);
    }

    /// For any sequence of N requests to the same method/route/status
    /// combination, the counter must equal exactly N (no double counting, no
    /// dropped increments).
    #[test]
    fn prop_request_count_equals_number_of_calls(
        n in 0usize..300,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..n {
            metrics.record_request("GET", "/api/v1/health", 200, 0.01);
        }
        prop_assert_eq!(counter_total(&registry.gather(), "http_requests_total"), n as f64);
    }
}
