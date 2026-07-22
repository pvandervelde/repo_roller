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

/// Sentinel label value pre-registered for every vector metric at
/// construction time, so its metric family is always present in a Prometheus
/// scrape/gather — even before the first real request/success/failure is
/// recorded.
///
/// `prometheus::Registry::gather()` prunes any `CounterVec`/`HistogramVec`
/// metric family that has zero recorded label combinations (a freshly
/// constructed vector metric starts with an empty children map). Without
/// this pre-seed, `repository_creation_requests_total` (etc.) would be
/// silently absent from a scrape taken immediately after startup, and the
/// `test_prometheus_registration_registers_all_five_metric_families` /
/// cross-crate `/metrics` acceptance tests (which gather with zero prior
/// activity) would fail.
///
/// The value is the maximum valid Unicode scalar value, chosen so it sorts
/// *after* any realistic organization/template/error-category value in
/// `Registry::gather()`'s per-family lexicographic ordering. This means code
/// that inspects the first metric in a family (e.g. in tests) always sees
/// the real, most-recently-relevant series once at least one real call has
/// been recorded, never this sentinel.
///
/// This is the canonical definition; `github_client::api_metrics` and
/// `repo_roller_api::http_metrics` each independently define a byte-identical
/// copy with a short doc pointing back here, rather than sharing this
/// constant, because the three crates have no existing shared low-level
/// dependency — introducing one solely to deduplicate a three-byte literal
/// would be a worse tradeoff than the duplication itself.
const UNSEEDED_SENTINEL: &str = "\u{10FFFF}";

/// Maps a [`RepoRollerError`] to a bounded, enumerable category label.
///
/// # Security
///
/// MUST NOT return (or embed) the error's `Display` text, which may contain
/// organization names, repository names, or upstream API response fragments.
/// The returned string must always be a member of [`KNOWN_ERROR_CATEGORIES`].
///
/// Every top-level `RepoRollerError` variant maps to exactly one bounded
/// category; the match is exhaustive so a new variant added later fails to
/// compile here rather than silently falling through to a catch-all.
pub fn error_category(err: &RepoRollerError) -> &'static str {
    match err {
        RepoRollerError::Validation(_) => "validation",
        RepoRollerError::Repository(_) => "repository",
        RepoRollerError::Configuration(_) => "configuration",
        RepoRollerError::Template(_) => "template",
        RepoRollerError::Authentication(_) => "authentication",
        RepoRollerError::GitHub(_) => "github",
        RepoRollerError::System(_) => "system",
        RepoRollerError::Permission(_) => "permission",
    }
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
    pub fn new(registry: &prometheus::Registry) -> Self {
        use prometheus::{CounterVec, Gauge, HistogramOpts, HistogramVec, Opts};

        let requests = CounterVec::new(
            Opts::new(
                "repository_creation_requests_total",
                "Total repository-creation requests received",
            ),
            &["organization", "template"],
        )
        .expect("Failed to create requests counter vec");

        let successes = CounterVec::new(
            Opts::new(
                "repository_creation_successes_total",
                "Successful repository-creation operations",
            ),
            &["organization", "template"],
        )
        .expect("Failed to create successes counter vec");

        let failures = CounterVec::new(
            Opts::new(
                "repository_creation_failures_total",
                "Failed repository-creation operations",
            ),
            &["organization", "template", "error_category"],
        )
        .expect("Failed to create failures counter vec");

        let duration = HistogramVec::new(
            HistogramOpts::new(
                "repository_creation_duration_seconds",
                "Repository-creation end-to-end duration in seconds",
            )
            .buckets(REPOSITORY_CREATION_DURATION_BUCKETS.to_vec()),
            &["organization", "template"],
        )
        .expect("Failed to create duration histogram vec");

        let active_tasks = Gauge::with_opts(Opts::new(
            "repository_creation_active_tasks",
            "In-flight repository-creation operations",
        ))
        .expect("Failed to create active tasks gauge");

        registry
            .register(Box::new(requests.clone()))
            .expect("Failed to register requests counter vec");
        registry
            .register(Box::new(successes.clone()))
            .expect("Failed to register successes counter vec");
        registry
            .register(Box::new(failures.clone()))
            .expect("Failed to register failures counter vec");
        registry
            .register(Box::new(duration.clone()))
            .expect("Failed to register duration histogram vec");
        registry
            .register(Box::new(active_tasks.clone()))
            .expect("Failed to register active tasks gauge");

        // Pre-seed every vector metric (see `UNSEEDED_SENTINEL` docs) so each
        // family is visible in a scrape immediately, before any real activity.
        requests.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);
        successes.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);
        failures.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);
        duration.with_label_values(&[UNSEEDED_SENTINEL, UNSEEDED_SENTINEL]);

        Self {
            requests,
            successes,
            failures,
            duration,
            active_tasks,
        }
    }
}

impl RepositoryCreationMetrics for PrometheusRepositoryCreationMetrics {
    fn record_request(&self, organization: &str, template: &str) {
        self.requests
            .with_label_values(&[organization, template])
            .inc();
    }

    fn record_success(&self, organization: &str, template: &str, duration_seconds: f64) {
        self.successes
            .with_label_values(&[organization, template])
            .inc();
        self.duration
            .with_label_values(&[organization, template])
            .observe(duration_seconds);
    }

    fn record_failure(
        &self,
        organization: &str,
        template: &str,
        error_category: &str,
        duration_seconds: f64,
    ) {
        self.failures
            .with_label_values(&[organization, template, error_category])
            .inc();
        self.duration
            .with_label_values(&[organization, template])
            .observe(duration_seconds);
    }

    fn increment_active_tasks(&self) {
        self.active_tasks.inc();
    }

    fn decrement_active_tasks(&self) {
        self.active_tasks.dec();
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
