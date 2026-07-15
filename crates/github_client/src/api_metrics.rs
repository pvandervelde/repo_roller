// Observability Phase 1: GitHub API call metrics abstraction.
//
// Mirrors `repo_roller_core::event_metrics::EventMetrics` / `repository_metrics`
// so that GitHub API call sites never depend on `prometheus::*` types directly
// (hexagonal-architecture constraint, docs/spec/constraints.md:138).
//
// TDD RED PHASE: `PrometheusGitHubApiMetrics` is stubbed with `todo!()` and
// MUST be implemented by the Coder. `NoOpGitHubApiMetrics` is fully
// implemented here (a no-op has no real logic to defer).
//
// ## Metric name / label design (Tester's working assumption — not present in
// the injected Interface Contract; flagged as a spec gap in the test report)
//
// | Metric name                     | Type    | Labels                        |
// |----------------------------------|---------|-------------------------------|
// | github_api_calls_total           | Counter | operation                     |
// | github_api_errors_total          | Counter | operation, status_category    |
// | github_api_rate_limit_remaining  | Gauge   | -                             |
//
// `operation` MUST be a bounded, source-controlled string identifying the
// GitHub API method invoked (e.g. `"get_repository"`, `"create_org_repository"`),
// never the request path, org name, or repo name — those are unbounded
// (attacker/user-controlled) values and must never become label values.
//
// `status_category` MUST be one of [`KNOWN_STATUS_CATEGORIES`] — derived from
// the [`crate::errors::Error`] variant, never the raw error `Display` text
// (which may embed upstream GitHub API response fragments).

/// Bounded set of status categories usable as a Prometheus label value.
pub const KNOWN_STATUS_CATEGORIES: [&str; 8] = [
    "api_error",
    "auth_error",
    "deserialization",
    "access_token_failure",
    "installation_not_found",
    "invalid_response",
    "not_found",
    "rate_limit_exceeded",
];

/// Maps a [`crate::errors::Error`] to a bounded, enumerable status category.
///
/// # Security
///
/// MUST NOT return (or embed) the error's `Display` text. Some `Error`
/// variants embed upstream GitHub API response fragments or repository/org
/// names, which are unbounded and must never become a label value.
///
/// # Panics
/// Stub: panics via `todo!()`. The Coder must implement an exhaustive match.
pub fn status_category(_err: &crate::errors::Error) -> &'static str {
    todo!("Coder: implement exhaustive github_client::Error -> bounded status_category mapping")
}

/// Abstraction for recording GitHub API call metrics.
///
/// # Thread Safety
/// All implementations MUST be thread-safe (Send + Sync).
pub trait GitHubApiMetrics: Send + Sync {
    /// Records that a GitHub API operation was invoked (successfully or not).
    ///
    /// `operation` MUST be a bounded, source-controlled identifier (see
    /// module docs) — never a raw URL, org name, or repo name.
    fn record_call(&self, operation: &str);

    /// Records that a GitHub API operation returned an error.
    ///
    /// `status_category` MUST be a member of [`KNOWN_STATUS_CATEGORIES`].
    fn record_error(&self, operation: &str, status_category: &str);

    /// Sets the most recently observed GitHub API rate-limit-remaining value.
    ///
    /// This is a point-in-time gauge (`set`, not `add`): each call overwrites
    /// the previous value, it does not accumulate.
    fn set_rate_limit_remaining(&self, remaining: i64);
}

/// Prometheus-backed implementation of [`GitHubApiMetrics`].
///
/// # Panics
/// `new` panics if metrics cannot be registered (duplicate names against the
/// supplied registry).
pub struct PrometheusGitHubApiMetrics {
    calls: prometheus::CounterVec,
    errors: prometheus::CounterVec,
    rate_limit_remaining: prometheus::Gauge,
}

impl PrometheusGitHubApiMetrics {
    /// Creates a new Prometheus metrics collector, registering all three
    /// metric families against `registry`.
    ///
    /// # Panics
    /// Panics if metrics cannot be registered (duplicate names).
    pub fn new(_registry: &prometheus::Registry) -> Self {
        todo!("Coder: register github_api_* metric families against the shared registry")
    }
}

impl GitHubApiMetrics for PrometheusGitHubApiMetrics {
    fn record_call(&self, _operation: &str) {
        todo!("Coder: increment github_api_calls_total{{operation}}")
    }

    fn record_error(&self, _operation: &str, _status_category: &str) {
        todo!("Coder: increment github_api_errors_total{{operation,status_category}}")
    }

    fn set_rate_limit_remaining(&self, _remaining: i64) {
        todo!("Coder: set github_api_rate_limit_remaining gauge")
    }
}

/// No-op implementation of [`GitHubApiMetrics`] for testing or when metrics
/// are disabled. Zero overhead; every method is a true no-op — this is the
/// complete, final implementation, not a stub.
#[derive(Default)]
pub struct NoOpGitHubApiMetrics;

impl NoOpGitHubApiMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl GitHubApiMetrics for NoOpGitHubApiMetrics {
    fn record_call(&self, _operation: &str) {}
    fn record_error(&self, _operation: &str, _status_category: &str) {}
    fn set_rate_limit_remaining(&self, _remaining: i64) {}
}

#[cfg(test)]
#[path = "api_metrics_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "api_metrics_proptests.rs"]
mod proptests;
