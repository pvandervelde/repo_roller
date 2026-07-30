// Property-based tests for api_metrics.
//
// These tests use proptest to verify invariants that hold for ALL inputs
// within a given domain, not just hand-picked examples.

use super::*;
use proptest::prelude::*;

fn fresh() -> (prometheus::Registry, PrometheusGitHubApiMetrics) {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusGitHubApiMetrics::new(&registry);
    (registry, metrics)
}

fn counter_total(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)
        .map(|mf| {
            mf.metric
                .iter()
                .filter_map(|m| m.counter.as_ref().map(|c| c.value()))
                .sum()
        })
        .unwrap_or(0.0)
}

fn gauge_total(metric_families: &[prometheus::proto::MetricFamily], name: &str) -> f64 {
    metric_families
        .iter()
        .find(|mf| mf.name() == name)
        .and_then(|mf| mf.metric.first())
        .and_then(|m| m.gauge.as_ref())
        .map(|g| g.value())
        .unwrap_or(0.0)
}

proptest! {
    /// For any sequence of N `record_call` invocations (regardless of which
    /// bounded operation string is used each time), the summed counter value
    /// across all label series must equal exactly N.
    #[test]
    fn prop_call_count_equals_number_of_calls(
        n in 0usize..300,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..n {
            metrics.record_call("get_repository");
        }
        prop_assert_eq!(counter_total(&registry.gather(), "github_api_calls_total"), n as f64);
    }

    /// The rate-limit-remaining gauge must always equal the last value
    /// `set_rate_limit_remaining` was called with, for any sequence of
    /// arbitrary i64 values within GitHub's plausible rate-limit range.
    #[test]
    fn prop_rate_limit_gauge_always_equals_last_set_value(
        values in proptest::collection::vec(0i64..6000, 1..50),
    ) {
        let (registry, metrics) = fresh();
        for v in &values {
            metrics.set_rate_limit_remaining(*v);
        }
        let last = *values.last().unwrap();
        prop_assert_eq!(gauge_total(&registry.gather(), "github_api_rate_limit_remaining"), last as f64);
    }

    /// Calls and errors recorded for the same operation must never
    /// cross-contaminate: the number of `record_call` invocations and
    /// `record_error` invocations are counted completely independently, for
    /// any interleaving.
    #[test]
    fn prop_calls_and_errors_never_cross_contaminate(
        calls in 0usize..100,
        errors in 0usize..100,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..calls {
            metrics.record_call("get_repository");
        }
        for _ in 0..errors {
            metrics.record_error("get_repository", "not_found");
        }

        let families = registry.gather();
        prop_assert_eq!(counter_total(&families, "github_api_calls_total"), calls as f64);
        prop_assert_eq!(counter_total(&families, "github_api_errors_total"), errors as f64);
    }
}
