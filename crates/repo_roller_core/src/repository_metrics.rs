// Observability Phase 1: repository-creation metrics abstraction.
//
// Mirrors the existing `EventMetrics` pattern (see `event_metrics.rs` and
// docs/spec/interfaces/event-metrics.md) so that business logic never depends
// on `prometheus::*` types directly (hexagonal-architecture constraint, see
// docs/spec/constraints.md:138).
//
// TDD RED PHASE: This file intentionally contains only enough structure for
// the test suite (`repository_metrics_tests.rs`, `repository_metrics_proptests.rs`)
// to compile. The `PrometheusRepositoryCreationMetrics` implementation and the
// `error_category` mapping function are stubbed with `todo!()` / `unimplemented!()`
// and MUST be implemented by the Coder. `NoOpRepositoryCreationMetrics` is fully
// implemented here because a no-op has no real logic to defer (identical in
// spirit to `NoOpEventMetrics`).
//
// ## Metric name / label design (Tester's working assumption — not present in
// the injected Interface Contract; flagged as a spec gap in the test report)
//
// | Metric name                              | Type      | Labels                              |
// |-------------------------------------------|-----------|--------------------------------------|
// | repository_creation_requests_total        | Counter   | organization, template               |
// | repository_creation_successes_total       | Counter   | organization, template               |
// | repository_creation_failures_total        | Counter   | organization, template, error_category |
// | repository_creation_duration_seconds      | Histogram | organization, template               |
// | repository_creation_active_tasks          | Gauge     | -                                     |
//
// Histogram buckets (seconds): 0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0
// (wider buckets than the notification histogram because repository creation
// target is < 2 minutes per .tech-decisions.yml performance section).
//
// `error_category` MUST return one of a small, fixed, enumerable set of
// strings derived from the top-level `RepoRollerError` variant — never the
// `Display`/`to_string()` free-text `reason` field, which may contain
// org/repo names, URLs, or (in pathological upstream cases) fragments of raw
// API error bodies. See docs SECURITY RULE: "Metric labels must be
// bounded/enumerable values only — never raw user input, error messages, or
// tokens."

use crate::errors::RepoRollerError;

/// Histogram buckets for repository-creation duration, in seconds.
///
/// Wider than the notification-delivery histogram (`event_metrics.rs`)
/// because repository creation involves template fetch, git operations, and
/// multiple GitHub API round-trips; target SLA is < 120s.
pub const REPOSITORY_CREATION_DURATION_BUCKETS: [f64; 9] =
    [0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0];

/// Bounded set of error categories usable as a Prometheus label value.
///
/// Every `RepoRollerError` variant maps to exactly one of these. Adding a new
/// `RepoRollerError` variant without updating this list (and `error_category`)
/// is a compile-time-invisible but test-visible gap — see
/// `test_error_category_is_exhaustive_over_known_variants`.
pub const KNOWN_ERROR_CATEGORIES: [&str; 8] = [
    "validation",
    "repository",
    "configuration",
    "template",
    "authentication",
    "github",
    "system",
    "permission",
];

/// Maps a [`RepoRollerError`] to a bounded, enumerable category label.
///
/// # Security
///
/// MUST NOT return (or embed) the error's `Display` text, which may contain
/// organization names, repository names, or upstream API response fragments.
/// The returned string must always be a member of [`KNOWN_ERROR_CATEGORIES`].
///
/// # Panics
///
/// Stub: panics via `todo!()`. The Coder must implement an exhaustive match.
pub fn error_category(_err: &RepoRollerError) -> &'static str {
    todo!("Coder: implement exhaustive RepoRollerError -> bounded category mapping")
}

/// Abstraction for recording repository-creation metrics.
///
/// Mirrors [`crate::event_metrics::EventMetrics`]. Implementations record
/// metrics to various backends (Prometheus, StatsD, etc.) without leaking
/// backend-specific types into business logic.
///
/// # Thread Safety
/// All implementations MUST be thread-safe (Send + Sync).
pub trait RepositoryCreationMetrics: Send + Sync {
    /// Records that a repository-creation request was received for
    /// `organization`/`template`.
    ///
    /// `template` should be an empty string (`""`) for empty-repository /
    /// no-template creations, never `Option`-encoded as a label (Prometheus
    /// labels cannot be absent).
    fn record_request(&self, organization: &str, template: &str);

    /// Records a successful repository creation and its end-to-end duration.
    fn record_success(&self, organization: &str, template: &str, duration_seconds: f64);

    /// Records a failed repository creation, its bounded error category, and
    /// the duration elapsed before failure.
    fn record_failure(
        &self,
        organization: &str,
        template: &str,
        error_category: &str,
        duration_seconds: f64,
    );

    /// Increments the in-flight repository-creation task gauge.
    fn increment_active_tasks(&self);

    /// Decrements the in-flight repository-creation task gauge.
    fn decrement_active_tasks(&self);
}

/// Prometheus-backed implementation of [`RepositoryCreationMetrics`].
///
/// # Panics
/// `new` panics if metrics cannot be registered (duplicate names against the
/// supplied registry) — same contract as [`crate::event_metrics::PrometheusEventMetrics::new`].
pub struct PrometheusRepositoryCreationMetrics {
    requests: prometheus::CounterVec,
    successes: prometheus::CounterVec,
    failures: prometheus::CounterVec,
    duration: prometheus::HistogramVec,
    active_tasks: prometheus::Gauge,
}

impl PrometheusRepositoryCreationMetrics {
    /// Creates a new Prometheus metrics collector, registering all five
    /// metric families against `registry`.
    ///
    /// # Panics
    /// Panics if metrics cannot be registered (duplicate names).
    pub fn new(_registry: &prometheus::Registry) -> Self {
        todo!("Coder: register repository_creation_* metric families against the shared registry")
    }
}

impl RepositoryCreationMetrics for PrometheusRepositoryCreationMetrics {
    fn record_request(&self, _organization: &str, _template: &str) {
        todo!("Coder: increment repository_creation_requests_total{{organization,template}}")
    }

    fn record_success(&self, _organization: &str, _template: &str, _duration_seconds: f64) {
        todo!("Coder: increment successes counter and observe duration histogram")
    }

    fn record_failure(
        &self,
        _organization: &str,
        _template: &str,
        _error_category: &str,
        _duration_seconds: f64,
    ) {
        todo!("Coder: increment failures counter (with error_category label) and observe duration histogram")
    }

    fn increment_active_tasks(&self) {
        todo!("Coder: increment repository_creation_active_tasks gauge")
    }

    fn decrement_active_tasks(&self) {
        todo!("Coder: decrement repository_creation_active_tasks gauge")
    }
}

/// No-op implementation of [`RepositoryCreationMetrics`] for testing or when
/// metrics are disabled. Zero overhead; every method is a true no-op — this
/// is the complete, final implementation, not a stub.
#[derive(Default)]
pub struct NoOpRepositoryCreationMetrics;

impl NoOpRepositoryCreationMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl RepositoryCreationMetrics for NoOpRepositoryCreationMetrics {
    fn record_request(&self, _organization: &str, _template: &str) {}
    fn record_success(&self, _organization: &str, _template: &str, _duration_seconds: f64) {}
    fn record_failure(
        &self,
        _organization: &str,
        _template: &str,
        _error_category: &str,
        _duration_seconds: f64,
    ) {
    }
    fn increment_active_tasks(&self) {}
    fn decrement_active_tasks(&self) {}
}

#[cfg(test)]
#[path = "repository_metrics_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "repository_metrics_proptests.rs"]
mod proptests;
