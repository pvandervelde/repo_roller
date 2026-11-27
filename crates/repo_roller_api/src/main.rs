//! RepoRoller REST API Server
//!
//! Main binary for running the API server in production or development.
//!
//! # Environment Variables
//!
//! - `API_PORT`: Port to listen on (default: 8080)
//! - `API_HOST`: Host to bind to (default: 0.0.0.0)
//! - `RUST_LOG`: Log level (default: info)
//! - `LOG_FORMAT`: Log output format -- `json` (default, machine-readable) or
//!   `pretty` (human-readable, for local development)
//! - `METADATA_REPOSITORY_NAME`: Name of metadata repository (default: .reporoller)
//! - `GITHUB_APP_ID`: GitHub App ID (required)
//! - `GITHUB_APP_PRIVATE_KEY`: GitHub App private key in PEM format (required)
//! - `OTEL_TRACES_ENABLED`: Enable OpenTelemetry/OTLP distributed trace export --
//!   `true`/`1` (case-insensitive) to enable; unset or any other value leaves it
//!   disabled (fail-safe-off default, see `tracing_otel` module docs)
//! - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP/gRPC collector endpoint (default:
//!   `http://localhost:4317`)
//! - `OTEL_SERVICE_NAME`: Service name attached to exported spans' resource
//!   attributes (default: `repo-roller-api`)

use std::env;

mod errors;
mod handlers;
mod http_metrics;
mod logging;
mod middleware;
mod models;
mod rate_limit;
mod routes;
mod sanitizing_writer;
mod server;
mod tracing_otel;
mod translation;

use server::{ApiConfig, ApiServer};

/// Bundle of every Prometheus-backed metrics collector used by `AppState`,
/// all registered against the same freshly created [`prometheus::Registry`].
///
/// Constructed exclusively by [`MetricsBundle::new`] and destructured into
/// the corresponding `AppState` fields, so `AppState::new` and the
/// `#[cfg(test)] Default` impl never duplicate the "construct registry,
/// register every collector, wrap the registry in an `Arc`" sequence. A bare
/// `prometheus::Registry` panics if the same metric name is registered
/// twice, so any future metrics collector must be added to
/// `MetricsBundle::new` rather than duplicated at each `AppState`
/// construction site.
struct MetricsBundle {
    registry: std::sync::Arc<prometheus::Registry>,
    event_metrics: std::sync::Arc<repo_roller_core::event_metrics::PrometheusEventMetrics>,
    repository_creation_metrics:
        std::sync::Arc<dyn repo_roller_core::repository_metrics::RepositoryCreationMetrics>,
    http_metrics: std::sync::Arc<dyn crate::http_metrics::HttpMetrics>,
    github_api_metrics: std::sync::Arc<dyn github_client::GitHubApiMetrics>,
    rate_limit_rejections: prometheus::IntCounterVec,
    rate_limit_tracked_keys: prometheus::IntGaugeVec,
}

impl MetricsBundle {
    /// Creates a fresh registry and registers every Prometheus-backed
    /// metrics collector against it.
    ///
    /// # Panics
    /// Panics if any collector fails to register (duplicate metric names) —
    /// unreachable in practice since this is the only place collectors are
    /// constructed against a freshly created registry.
    fn new() -> Self {
        let registry = prometheus::Registry::new();
        let event_metrics = std::sync::Arc::new(
            repo_roller_core::event_metrics::PrometheusEventMetrics::new(&registry),
        );
        let repository_creation_metrics: std::sync::Arc<
            dyn repo_roller_core::repository_metrics::RepositoryCreationMetrics,
        > = std::sync::Arc::new(
            repo_roller_core::repository_metrics::PrometheusRepositoryCreationMetrics::new(
                &registry,
            ),
        );
        let http_metrics: std::sync::Arc<dyn crate::http_metrics::HttpMetrics> =
            std::sync::Arc::new(crate::http_metrics::PrometheusHttpMetrics::new(&registry));
        let github_api_metrics: std::sync::Arc<dyn github_client::GitHubApiMetrics> =
            std::sync::Arc::new(github_client::PrometheusGitHubApiMetrics::new(&registry));

        // Labelled by `scope` only (a closed set: "user", "auth_ip"); never by
        // user identity or client address, which would be unbounded.
        let rate_limit_rejections = prometheus::IntCounterVec::new(
            prometheus::Opts::new(
                "rate_limit_rejections_total",
                "Total requests rejected by the application-level rate limiters",
            ),
            &["scope"],
        )
        .expect("Failed to create rate limit rejections counter");
        registry
            .register(Box::new(rate_limit_rejections.clone()))
            .expect("Failed to register rate limit rejections counter");
        // Pre-seed both scopes so each series is exported at 0 from startup.
        for scope in [
            crate::rate_limit::SCOPE_USER,
            crate::rate_limit::SCOPE_AUTH_IP,
        ] {
            rate_limit_rejections.with_label_values(&[scope]);
        }

        // Current number of keys each limiter tracks (bounded by the key cap).
        // Labelled by `scope` only; never exposes identities or addresses.
        let rate_limit_tracked_keys = prometheus::IntGaugeVec::new(
            prometheus::Opts::new(
                "rate_limit_tracked_keys",
                "Number of keys currently tracked by the application-level rate limiters",
            ),
            &["scope"],
        )
        .expect("Failed to create rate limit tracked keys gauge");
        registry
            .register(Box::new(rate_limit_tracked_keys.clone()))
            .expect("Failed to register rate limit tracked keys gauge");
        for scope in [
            crate::rate_limit::SCOPE_USER,
            crate::rate_limit::SCOPE_AUTH_IP,
        ] {
            rate_limit_tracked_keys.with_label_values(&[scope]).set(0);
        }

        Self {
            registry: std::sync::Arc::new(registry),
            event_metrics,
            repository_creation_metrics,
            http_metrics,
            github_api_metrics,
            rate_limit_rejections,
            rate_limit_tracked_keys,
        }
    }
}

/// API version
pub const API_VERSION: &str = "v1";

/// Default API port
pub const DEFAULT_PORT: u16 = 8080;

/// Application state shared across handlers
///
/// Contains shared configuration for API handlers.
/// GitHub operations use installation tokens minted from the stored App
/// credentials; no caller-supplied token is forwarded to the GitHub API.
///
/// Event metrics are initialised once at startup and cloned per request so that
/// Prometheus counters accumulate across requests and remain scrapable at a
/// `/metrics` endpoint. Creating a new `PrometheusEventMetrics` per request
/// would panic on the second call because the counters are already registered
/// in the shared registry.
///
/// All Prometheus-backed metrics collectors (`event_metrics`,
/// `repository_creation_metrics`, `http_metrics`, `github_api_metrics`) are
/// registered against the single shared `metrics_registry`, so one `/metrics`
/// scrape (see `routes::create_router`) returns every metric family.
#[derive(Clone)]
pub struct AppState {
    /// Metadata repository name for organization settings
    pub metadata_repository_name: String,
    /// Shared event metrics, initialised once at startup.
    ///
    /// Cloned (Arc clone, not a new allocation) for each handler invocation so
    /// that metric values accumulate across requests.
    pub event_metrics: std::sync::Arc<repo_roller_core::event_metrics::PrometheusEventMetrics>,
    /// Shared repository-creation metrics, initialised once at startup and
    /// registered against `metrics_registry`.
    pub(crate) repository_creation_metrics:
        std::sync::Arc<dyn repo_roller_core::repository_metrics::RepositoryCreationMetrics>,
    /// Shared per-endpoint HTTP request metrics, initialised once at startup
    /// and registered against `metrics_registry`. Consumed by
    /// `http_metrics::http_metrics_middleware` via `from_fn_with_state`.
    pub(crate) http_metrics: std::sync::Arc<dyn crate::http_metrics::HttpMetrics>,
    /// Shared GitHub API call metrics, initialised once at startup and
    /// registered against `metrics_registry`. Threaded through per-request
    /// `GitHubClient` construction in `handlers.rs`, mirroring how
    /// `event_metrics` is threaded through `EventNotificationContext`.
    pub(crate) github_api_metrics: std::sync::Arc<dyn github_client::GitHubApiMetrics>,
    /// Shared Prometheus registry that every metrics collector above
    /// registers against. Retained (rather than dropped after constructing
    /// the collectors) so the `/metrics` handler can call `.gather()` on it
    /// for every scrape.
    pub(crate) metrics_registry: std::sync::Arc<prometheus::Registry>,
    /// Optional GitHub API base URL override.
    ///
    /// When `None` the default `https://api.github.com` is used. Set this to
    /// target a GitHub Enterprise instance or (in tests) a wiremock server.
    pub(crate) github_api_base_url: Option<String>,
    /// GitHub App authentication service used to mint installation tokens.
    ///
    /// Stored as an `Arc` so that cloned `AppState` values (one per request)
    /// share the same underlying service without copying the private key.
    pub(crate) auth_service: std::sync::Arc<auth_handler::GitHubAuthService>,
    /// HMAC-SHA256 secret used to sign and verify backend-issued JWTs.
    ///
    /// Loaded from the `JWT_SECRET` environment variable at startup.
    /// Must be at least 32 bytes.  Never logged.
    pub(crate) jwt_secret: secrecy::SecretString,
    /// Per-user rate limiter shared by all requests (Task #284).
    pub(crate) rate_limiter: std::sync::Arc<crate::rate_limit::RateLimiter>,
    /// Per-client-address limiter for the public token-exchange endpoint.
    pub(crate) auth_rate_limiter: std::sync::Arc<crate::rate_limit::RateLimiter>,
    /// Counter of requests rejected by the rate limiter, registered against
    /// `metrics_registry` as `rate_limit_rejections_total`.
    pub(crate) rate_limit_rejections: prometheus::IntCounterVec,
    /// Gauge of keys tracked per limiter scope, refreshed at scrape time and
    /// registered as `rate_limit_tracked_keys`.
    pub(crate) rate_limit_tracked_keys: prometheus::IntGaugeVec,
    /// Pre-minted token injected in tests to bypass `GitHubAuthService`.
    ///
    /// When `Some`, `get_installation_token` returns this value without calling
    /// the real GitHub API. Always `None` in production.
    #[cfg(test)]
    pub(crate) mock_installation_token: Option<String>,
}

impl AppState {
    /// Create new application state with configuration.
    ///
    /// # Arguments
    ///
    /// * `metadata_repository_name` - Name of the metadata repository
    /// * `github_app_id` - GitHub App ID for minting installation tokens
    /// * `github_app_private_key` - GitHub App private key (PEM format)
    pub fn new(
        metadata_repository_name: impl Into<String>,
        github_app_id: u64,
        github_app_private_key: impl Into<String>,
        jwt_secret: impl Into<String>,
    ) -> Self {
        let MetricsBundle {
            registry: metrics_registry,
            event_metrics,
            repository_creation_metrics,
            http_metrics,
            github_api_metrics,
            rate_limit_rejections,
            rate_limit_tracked_keys,
        } = MetricsBundle::new();
        Self {
            metadata_repository_name: metadata_repository_name.into(),
            event_metrics,
            repository_creation_metrics,
            http_metrics,
            github_api_metrics,
            metrics_registry,
            github_api_base_url: None,
            auth_service: std::sync::Arc::new(auth_handler::GitHubAuthService::new(
                github_app_id,
                github_app_private_key,
            )),
            jwt_secret: secrecy::SecretString::from(jwt_secret.into()),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(
                crate::rate_limit::RateLimitConfig::from_env(),
            )),
            auth_rate_limiter: std::sync::Arc::new(
                crate::rate_limit::RateLimiter::new_fail_closed(
                    crate::rate_limit::RateLimitConfig::auth_from_env(),
                ),
            ),
            rate_limit_rejections,
            rate_limit_tracked_keys,
            #[cfg(test)]
            mock_installation_token: None,
        }
    }

    /// Replace the token-exchange limiter's configuration (fresh limiter).
    ///
    /// The token-exchange limiter fails closed when its key cap is reached.
    pub fn with_auth_rate_limit_config(self, config: crate::rate_limit::RateLimitConfig) -> Self {
        self.with_auth_rate_limiter(std::sync::Arc::new(
            crate::rate_limit::RateLimiter::new_fail_closed(config),
        ))
    }

    /// Replace the token-exchange limiter (e.g. one with an injected clock).
    pub fn with_auth_rate_limiter(
        mut self,
        limiter: std::sync::Arc<crate::rate_limit::RateLimiter>,
    ) -> Self {
        self.auth_rate_limiter = limiter;
        self
    }

    /// Replace the rate limiter's configuration (fresh, empty limiter).
    pub fn with_rate_limit_config(self, config: crate::rate_limit::RateLimitConfig) -> Self {
        self.with_rate_limiter(std::sync::Arc::new(crate::rate_limit::RateLimiter::new(
            config,
        )))
    }

    /// Copy each limiter's current key count into the
    /// `rate_limit_tracked_keys{scope}` gauge.
    pub(crate) fn update_rate_limit_tracked_keys(&self) {
        for (scope, limiter) in [
            (crate::rate_limit::SCOPE_USER, &self.rate_limiter),
            (crate::rate_limit::SCOPE_AUTH_IP, &self.auth_rate_limiter),
        ] {
            self.rate_limit_tracked_keys
                .with_label_values(&[scope])
                .set(i64::try_from(limiter.tracked_keys()).unwrap_or(i64::MAX));
        }
    }

    /// Replace the rate limiter (e.g. one with an injected clock).
    pub fn with_rate_limiter(
        mut self,
        limiter: std::sync::Arc<crate::rate_limit::RateLimiter>,
    ) -> Self {
        self.rate_limiter = limiter;
        self
    }

    /// Override the GitHub API base URL.
    ///
    /// Useful for GitHub Enterprise deployments and for pointing at a mock
    /// server in integration tests.
    pub fn with_github_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.github_api_base_url = Some(url.into());
        self
    }

    /// Inject a pre-minted installation token used in unit tests.
    ///
    /// When set, `get_installation_token` returns this token without calling
    /// the GitHub App API, allowing tests to point the GitHub client at a
    /// wiremock server without real App credentials.
    #[cfg(test)]
    pub fn with_mock_installation_token(mut self, token: impl Into<String>) -> Self {
        self.mock_installation_token = Some(token.into());
        self
    }

    /// Mint a GitHub App installation token for `org`.
    ///
    /// In production this uses the stored `GitHubAuthService` (holding the App
    /// credentials) to request a token from the GitHub API.
    /// In tests the value set by `with_mock_installation_token` is returned
    /// directly, bypassing the real App authentication flow.
    pub(crate) async fn get_installation_token(
        &self,
        org: &str,
    ) -> Result<String, crate::errors::ApiError> {
        #[cfg(test)]
        if let Some(ref token) = self.mock_installation_token {
            return Ok(token.clone());
        }
        use auth_handler::UserAuthenticationService as _;
        self.auth_service
            .get_installation_token_for_org(org)
            .await
            .map_err(|e| {
                crate::errors::ApiError::internal(format!(
                    "Failed to get GitHub App installation token for organisation '{}': {}",
                    org, e
                ))
            })
    }
}

#[cfg(test)]
/// Fixed 32-byte JWT secret used exclusively in test builds.
///
/// Shared by `AppState::default()` and every test that signs tokens manually
/// (e.g. in `middleware_tests`). Having a single constant means a rename or
/// rotation is a one-place change that produces a compile error if any
/// consumer is left behind.
pub(crate) const TEST_JWT_SECRET: &str = "test-jwt-secret-key-minimum-32b!";

#[cfg(test)]
impl Default for AppState {
    fn default() -> Self {
        let MetricsBundle {
            registry: metrics_registry,
            event_metrics,
            repository_creation_metrics,
            http_metrics,
            github_api_metrics,
            rate_limit_rejections,
            rate_limit_tracked_keys,
        } = MetricsBundle::new();
        Self {
            metadata_repository_name: ".reporoller".to_string(),
            event_metrics,
            repository_creation_metrics,
            http_metrics,
            github_api_metrics,
            metrics_registry,
            github_api_base_url: None,
            auth_service: std::sync::Arc::new(auth_handler::GitHubAuthService::new(0u64, "")),
            jwt_secret: secrecy::SecretString::from(TEST_JWT_SECRET.to_string()),
            rate_limiter: std::sync::Arc::new(crate::rate_limit::RateLimiter::new(
                crate::rate_limit::RateLimitConfig::default(),
            )),
            auth_rate_limiter: std::sync::Arc::new(
                crate::rate_limit::RateLimiter::new_fail_closed(
                    crate::rate_limit::RateLimitConfig::new(
                        true,
                        crate::rate_limit::DEFAULT_AUTH_MAX_REQUESTS,
                        crate::rate_limit::DEFAULT_AUTH_WINDOW_SECS,
                    ),
                ),
            ),
            rate_limit_rejections,
            rate_limit_tracked_keys,
            mock_installation_token: None,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Both ring (via reqwest) and aws-lc-rs (via octocrab) are compiled into
    // the dep tree. Rustls 0.23 requires an explicit default when both providers
    // are present; without this call the first TLS connection panics.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    // Initialize tracing. JSON output is the default (production-safe,
    // machine-readable); set LOG_FORMAT=pretty for human-readable output
    // during local development. Distributed tracing (OpenTelemetry/OTLP
    // span export) is fail-safe-off by default -- set OTEL_TRACES_ENABLED=true
    // to export spans to an OTLP collector. See `tracing_otel::init_tracing_with_otel`
    // for details (replaces the former `logging::init_tracing()` call site).
    tracing_otel::init_tracing_with_otel();

    // Load configuration from environment
    let config = ApiConfig {
        port: env::var("API_PORT")
            .unwrap_or_else(|_| DEFAULT_PORT.to_string())
            .parse()
            .expect("Invalid API_PORT"),
        host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
    };

    let metadata_repo =
        env::var("METADATA_REPOSITORY_NAME").unwrap_or_else(|_| ".reporoller".to_string());

    let github_app_id = env::var("GITHUB_APP_ID")
        .expect("GITHUB_APP_ID environment variable is required")
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");

    // GITHUB_APP_PRIVATE_KEY is a secret — never log its value.
    // Normalize literal \n sequences to real newlines so the key works whether
    // it was stored with escaped newlines (common in env files and CI secrets)
    // or with real newlines (common when read directly from a .pem file).
    let github_app_private_key = env::var("GITHUB_APP_PRIVATE_KEY")
        .expect("GITHUB_APP_PRIVATE_KEY environment variable is required")
        .replace("\\n", "\n");

    // JWT_SECRET signs backend-issued JWTs — never log its value.
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable is required");
    assert!(
        jwt_secret.len() >= 32,
        "JWT_SECRET must be at least 32 characters"
    );

    // Create app state and server
    let state = AppState::new(
        metadata_repo.clone(),
        github_app_id,
        github_app_private_key,
        jwt_secret,
    );
    let server = ApiServer::new(config, state);

    tracing::info!("Starting RepoRoller API server");
    tracing::info!("API version: {}", API_VERSION);
    tracing::info!("Metadata repository: {}", metadata_repo);
    tracing::info!("GitHub App ID: {}", github_app_id);

    // Start server with graceful shutdown
    server.serve().await
}

#[cfg(test)]
mod test_crypto_provider {
    #[ctor::ctor(unsafe)]
    fn init_default_crypto_provider() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }
}
