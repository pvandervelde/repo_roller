//! HTTP server configuration and startup
//!
//! This module provides the main server configuration and startup logic.

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;

use crate::{routes, AppState, DEFAULT_PORT};

/// API server configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Port to listen on
    pub port: u16,

    /// Host to bind to
    pub host: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: "0.0.0.0".to_string(),
        }
    }
}

/// API server
pub struct ApiServer {
    config: ApiConfig,
    state: AppState,
}

impl ApiServer {
    /// Create a new API server with the given configuration.
    pub fn new(config: ApiConfig, state: AppState) -> Self {
        Self { config, state }
    }

    /// Build the Axum router with all routes and middleware.
    pub fn router(&self) -> Router {
        routes::create_router(self.state.clone())
    }

    /// Start the server and listen for requests.
    ///
    /// This method blocks until the server is shut down gracefully via
    /// CTRL+C (SIGINT) or SIGTERM signal.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind to the configured address.
    pub async fn serve(self) -> anyhow::Result<()> {
        let addr = SocketAddr::from((
            self.config.host.parse::<std::net::IpAddr>()?,
            self.config.port,
        ));

        tracing::info!("Starting API server on {}", addr);

        let listener = TcpListener::bind(addr).await?;
        let app = self.router();

        // Serve with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        tracing::info!("Server shutdown complete");

        Ok(())
    }
}

/// Wait for shutdown signal (CTRL+C or SIGTERM)
///
/// This function waits for either:
/// - CTRL+C (SIGINT) on all platforms
/// - SIGTERM on Unix platforms
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received CTRL+C, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
