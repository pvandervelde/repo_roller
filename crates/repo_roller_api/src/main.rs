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
/// Actual service instances are created per-request using authentication context.
#[derive(Clone)]
pub struct AppState {
    /// Metadata repository name for organization settings
    pub metadata_repository_name: String,
}

impl AppState {
    /// Create new application state with configuration
    ///
    /// # Arguments
    ///
    /// * `metadata_repository_name` - Name of the metadata repository
    pub fn new(metadata_repository_name: impl Into<String>) -> Self {
        Self {
            metadata_repository_name: metadata_repository_name.into(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(".reporoller")
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

    // Create app state and server
    let state = AppState::new(metadata_repo.clone());
    let server = ApiServer::new(config, state);

    tracing::info!("Starting RepoRoller API server");
    tracing::info!("API version: {}", API_VERSION);
    tracing::info!("Metadata repository: {}", metadata_repo);

    // Start server with graceful shutdown
    server.serve().await
}
