//! HTTP routing configuration
//!
//! This module defines all HTTP routes and their corresponding handlers.
//!
//! # Route Structure
//!
//! All routes are prefixed with `/api/v1`:
//!
//! - POST   /api/v1/repositories - Create repository
//! - POST   /api/v1/repositories/validate-name - Validate name
//! - POST   /api/v1/repositories/validate - Validate full request
//! - GET    /api/v1/orgs/:org/templates - List templates
//! - GET    /api/v1/orgs/:org/templates/:template - Get template details
//! - POST   /api/v1/orgs/:org/templates/:template/validate - Validate template
//! - GET    /api/v1/orgs/:org/repository-types - List repository types
//! - GET    /api/v1/orgs/:org/repository-types/:type - Get type config
//! - GET    /api/v1/orgs/:org/defaults - Get global defaults
//! - POST   /api/v1/orgs/:org/configuration/preview - Preview configuration
//! - POST   /api/v1/orgs/:org/validate - Validate organization
//! - GET    /api/v1/health - Health check
//!
//! See: .llm/rest-api-review-response.md

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers,
    middleware as api_middleware,
    AppState,
};

/// Create the complete API router with all routes configured.
///
/// This function sets up:
/// - All endpoint routes
/// - Authentication middleware
/// - CORS configuration
/// - Request tracing
/// - Timeout handling
pub fn create_router(state: AppState) -> Router {
    // API v1 routes
    let api_v1 = Router::new()
        // Repository operations
        .route("/repositories", post(handlers::create_repository))
        .route("/repositories/validate-name", post(handlers::validate_repository_name))
        .route("/repositories/validate", post(handlers::validate_repository_request))
        // Organization-specific routes
        .nest("/orgs/:org", organization_routes())
        // Health check (no auth required)
        .route("/health", get(handlers::health_check))
        // Add authentication middleware to all routes except health
        .layer(middleware::from_fn(api_middleware::auth_middleware))
        .layer(middleware::from_fn(api_middleware::tracing_middleware))
        .with_state(state);

    // Root router with API version prefix
    Router::new()
        .nest("/api/v1", api_v1)
}

/// Organization-specific routes (nested under /orgs/:org)
fn organization_routes() -> Router<AppState> {
    Router::new()
        // Template routes
        .route("/templates", get(handlers::list_templates))
        .route("/templates/:template", get(handlers::get_template_details))
        .route("/templates/:template/validate", post(handlers::validate_template))
        // Repository type routes
        .route("/repository-types", get(handlers::list_repository_types))
        .route("/repository-types/:type", get(handlers::get_repository_type_config))
        // Configuration routes
        .route("/defaults", get(handlers::get_global_defaults))
        .route("/configuration/preview", post(handlers::preview_configuration))
        // Organization validation
        .route("/validate", post(handlers::validate_organization))
        // Add organization-specific authorization middleware
        .layer(middleware::from_fn(api_middleware::organization_auth_middleware))
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;
