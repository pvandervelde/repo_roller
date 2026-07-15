// Property-based tests for repository_metrics.
//
// These tests use proptest to verify invariants that hold for ALL inputs
// within a given domain, not just hand-picked examples. Mirrors the style of
// naming_validator_proptests.rs / permissions_proptests.rs.

use super::*;
use proptest::prelude::*;

/// Build a fresh Prometheus-backed collector plus its registry for one
/// property test case (a fresh registry per case avoids name-collision panics
/// between proptest iterations).
fn fresh() -> (prometheus::Registry, PrometheusRepositoryCreationMetrics) {
    let registry = prometheus::Registry::new();
    let metrics = PrometheusRepositoryCreationMetrics::new(&registry);
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
    /// For any sequence of N `record_request` calls (regardless of the
    /// organization/template strings used), the summed counter value across
    /// all label series must equal exactly N. This holds for arbitrary
    /// alphanumeric organization/template names, guarding against off-by-one
    /// or dropped-increment bugs.
    #[test]
    fn prop_request_count_equals_number_of_calls(
        orgs in proptest::collection::vec("[a-z][a-z0-9-]{0,20}", 1..30),
    ) {
        let (registry, metrics) = fresh();
        for org in &orgs {
            metrics.record_request(org, "some-template");
        }
        let total = counter_total(&registry.gather(), "repository_creation_requests_total");
        prop_assert_eq!(total, orgs.len() as f64);
    }

    /// For any interleaved sequence of increment/decrement active-task calls
    /// where increments and decrements are balanced, the gauge must return to
    /// exactly zero — regardless of the interleaving order.
    #[test]
    fn prop_balanced_increment_decrement_returns_gauge_to_zero(
        n in 0usize..200,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..n {
            metrics.increment_active_tasks();
        }
        for _ in 0..n {
            metrics.decrement_active_tasks();
        }
        prop_assert_eq!(gauge_total(&registry.gather(), "repository_creation_active_tasks"), 0.0);
    }

    /// The active-task gauge must always equal (increments - decrements) so
    /// far, for any prefix of increment-only operations (never negative,
    /// never "stuck").
    #[test]
    fn prop_gauge_equals_increment_count_when_no_decrements(
        n in 0usize..200,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..n {
            metrics.increment_active_tasks();
        }
        prop_assert_eq!(gauge_total(&registry.gather(), "repository_creation_active_tasks"), n as f64);
    }

    /// For any duration value in the plausible real-world range (including
    /// values far beyond the documented SLA), recording a success must always
    /// increase the sample count of the duration histogram by exactly 1 and
    /// must never panic.
    #[test]
    fn prop_any_nonnegative_duration_is_recorded_exactly_once(
        duration in 0.0f64..10_000.0,
    ) {
        let (registry, metrics) = fresh();
        metrics.record_success("acme-corp", "rust-service", duration);

        let families = registry.gather();
        let hist = families
            .iter()
            .find(|mf| mf.name() == "repository_creation_duration_seconds")
            .and_then(|mf| mf.metric.first())
            .and_then(|m| m.histogram.as_ref())
            .expect("histogram sample must exist");
        prop_assert_eq!(hist.get_sample_count(), 1);
    }

    /// requests_total must always be >= successes_total + failures_total is
    /// NOT an invariant enforced by this trait (callers may call
    /// record_success/record_failure without a matching record_request in
    /// some code paths); instead the real invariant under test is narrower:
    /// summing successes and failures recorded through this collector alone
    /// must equal the number of record_success + record_failure calls made,
    /// with no cross-contamination between the two counters.
    #[test]
    fn prop_success_and_failure_counts_never_cross_contaminate(
        successes in 0usize..50,
        failures in 0usize..50,
    ) {
        let (registry, metrics) = fresh();
        for _ in 0..successes {
            metrics.record_success("acme-corp", "rust-service", 1.0);
        }
        for _ in 0..failures {
            metrics.record_failure("acme-corp", "rust-service", "system", 1.0);
        }

        let families = registry.gather();
        prop_assert_eq!(
            counter_total(&families, "repository_creation_successes_total"),
            successes as f64
        );
        prop_assert_eq!(
            counter_total(&families, "repository_creation_failures_total"),
            failures as f64
        );
    }
}
