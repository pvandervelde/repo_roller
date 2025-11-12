//! RepoRoller REST API
//!
//! This crate provides the REST API for RepoRoller repository automation.
//! It exposes HTTP endpoints for repository creation, template discovery,
//! and organization configuration management.
//!
//! # Architecture
//!
//! This crate exists in the HTTP layer and handles:
//! - HTTP request/response translation
//! - Authentication middleware
//! - Error mapping from domain to HTTP
//! - Routing and server configuration
//!
//! **CRITICAL**: This crate must never be imported by business logic.
//! The dependency flows: HTTP API → Business Logic, never the reverse.
//!
//! # Specifications
//!
//! See:
//! - `specs/interfaces/api-request-types.md` - Request models
//! - `specs/interfaces/api-response-types.md` - Response models
//! - `specs/interfaces/api-error-handling.md` - Error handling
//! - `.llm/rest-api-implementation-guide.md` - Implementation guide

pub mod errors;
pub mod middleware;
pub mod models;
pub mod handlers;
pub mod routes;
pub mod server;

// Re-export key types for convenience
pub use errors::{ApiError, ErrorResponse};
pub use models::{request, response};
pub use server::{ApiConfig, ApiServer};

/// API version
pub const API_VERSION: &str = "v1";

/// Default API port
pub const DEFAULT_PORT: u16 = 8080;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    // TODO: Add service dependencies
    // pub repository_service: Arc<dyn RepositoryService>,
    // pub config_manager: Arc<dyn ConfigurationManager>,
    // pub auth_service: Arc<dyn AuthenticationService>,
}

impl AppState {
    /// Create new application state with service dependencies
    pub fn new() -> Self {
        // TODO: Initialize with actual service implementations
        Self {}
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
