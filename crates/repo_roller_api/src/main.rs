//! RepoRoller REST API Server
//!
//! Main binary for running the API server in production or development.
//!
//! # Environment Variables
//!
//! - `API_PORT`: Port to listen on (default: 8080)
//! - `API_HOST`: Host to bind to (default: 0.0.0.0)
//! - `RUST_LOG`: Log level (default: info)
//! - `METADATA_REPOSITORY_NAME`: Name of metadata repository (default: .reporoller)
//! - `GITHUB_APP_ID`: GitHub App ID (required)
//! - `GITHUB_APP_PRIVATE_KEY`: GitHub App private key in PEM format (required)

use std::env;

mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod server;
mod translation;

use server::{ApiConfig, ApiServer};

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
#[derive(Clone)]
pub struct AppState {
    /// Metadata repository name for organization settings
    pub metadata_repository_name: String,
    /// Shared event metrics, initialised once at startup.
    ///
    /// Cloned (Arc clone, not a new allocation) for each handler invocation so
    /// that metric values accumulate across requests.
    pub event_metrics: std::sync::Arc<repo_roller_core::event_metrics::PrometheusEventMetrics>,
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
    ) -> Self {
        let registry = prometheus::Registry::new();
        Self {
            metadata_repository_name: metadata_repository_name.into(),
            event_metrics: std::sync::Arc::new(
                repo_roller_core::event_metrics::PrometheusEventMetrics::new(&registry),
            ),
            github_api_base_url: None,
            auth_service: std::sync::Arc::new(auth_handler::GitHubAuthService::new(
                github_app_id,
                github_app_private_key,
            )),
            #[cfg(test)]
            mock_installation_token: None,
        }
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
impl Default for AppState {
    fn default() -> Self {
        Self {
            metadata_repository_name: ".reporoller".to_string(),
            event_metrics: {
                let registry = prometheus::Registry::new();
                std::sync::Arc::new(
                    repo_roller_core::event_metrics::PrometheusEventMetrics::new(&registry),
                )
            },
            github_api_base_url: None,
            auth_service: std::sync::Arc::new(auth_handler::GitHubAuthService::new(0u64, "")),
            mock_installation_token: None,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

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
    let github_app_private_key = env::var("GITHUB_APP_PRIVATE_KEY")
        .expect("GITHUB_APP_PRIVATE_KEY environment variable is required");

    // Create app state and server
    let state = AppState::new(metadata_repo.clone(), github_app_id, github_app_private_key);
    let server = ApiServer::new(config, state);

    tracing::info!("Starting RepoRoller API server");
    tracing::info!("API version: {}", API_VERSION);
    tracing::info!("Metadata repository: {}", metadata_repo);
    tracing::info!("GitHub App ID: {}", github_app_id);

    // Start server with graceful shutdown
    server.serve().await
}
