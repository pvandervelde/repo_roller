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
pub mod translation;

// Re-export key types for convenience
pub use errors::{ApiError, ErrorResponse};
pub use models::{request, response};
pub use server::{ApiConfig, ApiServer};
pub use translation::{
    http_create_repository_request_to_domain,
    domain_repository_creation_result_to_http,
};

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
