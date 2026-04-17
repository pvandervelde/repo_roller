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
//! - GET    /api/v1/orgs/:org/teams - List organization teams
//! - GET    /api/v1/health - Health check
//!
//! See: .llm/rest-api-review-response.md

use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

use crate::{handlers, middleware as api_middleware, AppState};

/// Build a CORS layer restricted to the origin specified by the `FRONTEND_ORIGIN`
/// environment variable.
///
/// If `FRONTEND_ORIGIN` is not set, returns a `CorsLayer` with no allowed origins —
/// all cross-origin browser requests will be blocked, which is the secure default
/// for a production deployment where the SvelteKit frontend proxies all API calls.
///
/// Panics at startup if `FRONTEND_ORIGIN` is set to a value that is not a valid
/// HTTP header value (invalid bytes / control characters).
fn build_cors_layer() -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600));

    match std::env::var("FRONTEND_ORIGIN") {
        Ok(origin) if !origin.is_empty() => {
            let value = HeaderValue::from_str(&origin).unwrap_or_else(|_| {
                panic!(
                    "FRONTEND_ORIGIN '{}' is not a valid HTTP header value",
                    origin
                )
            });
            tracing::info!("CORS: allowing requests from origin '{}'", origin);
            base.allow_origin(value)
        }
        _ => {
            tracing::warn!(
                "FRONTEND_ORIGIN is not set — CORS will block all cross-origin browser requests. \
                 Set FRONTEND_ORIGIN to the deployed frontend URL in production."
            );
            base
        }
    }
}

/// Create the complete API router with all routes configured.
///
/// This function sets up:
/// - All endpoint routes
/// - Authentication middleware
/// - CORS configuration (restricted to FRONTEND_ORIGIN env var)
/// - Request tracing
/// - Timeout handling
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors_layer();

    // Configure request tracing
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(DefaultOnResponse::new().include_headers(true));

    // Configure request timeout (30 seconds)
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(30));

    // Protected API routes (require authentication)
    let protected_routes = Router::new()
        // Repository operations
        .route("/repositories", post(handlers::create_repository))
        .route(
            "/repositories/validate-name",
            post(handlers::validate_repository_name),
        )
        .route(
            "/repositories/validate",
            post(handlers::validate_repository_request),
        )
        // Organization-specific routes
        .nest("/orgs/:org", organization_routes())
        // Auth middleware reads jwt_secret from AppState; from_fn_with_state
        // supplies the state to the middleware function.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            api_middleware::auth_middleware,
        ));

    // API v1 routes (includes both protected and public routes)
    let api_v1 = Router::new()
        .merge(protected_routes)
        // Public routes (no authentication required)
        .route("/auth/token", post(handlers::exchange_github_token))
        .route("/health", get(handlers::health_check))
        // Add remaining middleware layers
        .layer(middleware::from_fn(api_middleware::tracing_middleware))
        .layer(timeout_layer)
        .layer(trace_layer)
        .layer(cors)
        .with_state(state);

    // Root router with API version prefix
    Router::new().nest("/api/v1", api_v1)
}

/// Organization-specific routes (nested under /orgs/:org)
fn organization_routes() -> Router<AppState> {
    Router::new()
        // Template routes
        .route("/templates", get(handlers::list_templates))
        .route("/templates/:template", get(handlers::get_template_details))
        .route(
            "/templates/:template/validate",
            post(handlers::validate_template),
        )
        // Repository type routes
        .route("/repository-types", get(handlers::list_repository_types))
        .route(
            "/repository-types/:type",
            get(handlers::get_repository_type_config),
        )
        // Configuration routes
        .route("/defaults", get(handlers::get_global_defaults))
        .route(
            "/configuration/preview",
            post(handlers::preview_configuration),
        )
        // Organization validation
        .route("/validate", post(handlers::validate_organization))
        // Team routes
        .route("/teams", get(handlers::list_organization_teams))
        // Add organization-specific authorization middleware
        .layer(middleware::from_fn(
            api_middleware::organization_auth_middleware,
        ))
}

/// Create a router for testing without authentication middleware.
///
/// This function creates the same route structure as `create_router` but
/// without the authentication middleware layer. This allows tests to bypass
/// authentication while still testing handler logic, request validation,
/// and response formatting.
///
/// # Security Note
///
/// This function is only available in test builds and should never be used
/// in production code.
#[cfg(test)]
pub fn create_router_without_auth(state: AppState) -> Router {
    use axum::{
        http::{header, Method},
        routing::{get, post},
        Router,
    };
    use std::time::Duration;
    use tower_http::{
        cors::CorsLayer,
        timeout::TimeoutLayer,
        trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    };

    // Test router: CORS wildcard is acceptable here because this router is
    // only instantiated in unit/integration tests, never in production.
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600));

    // Configure request tracing
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(DefaultOnResponse::new().include_headers(true));

    // Configure request timeout (30 seconds)
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(30));

    // API v1 routes without auth middleware
    let api_v1 = Router::new()
        // Repository operations
        .route("/repositories", post(handlers::create_repository))
        .route(
            "/repositories/validate-name",
            post(handlers::validate_repository_name),
        )
        .route(
            "/repositories/validate",
            post(handlers::validate_repository_request),
        )
        // Organization-specific routes (without org-specific auth)
        .nest("/orgs/:org", organization_routes_without_auth())
        // Health check
        .route("/health", get(handlers::health_check))
        // Add middleware layers (without auth_middleware)
        .layer(middleware::from_fn(api_middleware::tracing_middleware))
        .layer(timeout_layer)
        .layer(trace_layer)
        .layer(cors)
        .with_state(state);

    // Root router with API version prefix
    Router::new().nest("/api/v1", api_v1)
}

/// Organization-specific routes for testing (without authentication)
#[cfg(test)]
fn organization_routes_without_auth() -> Router<AppState> {
    Router::new()
        // Template routes
        .route("/templates", get(handlers::list_templates))
        .route("/templates/:template", get(handlers::get_template_details))
        .route(
            "/templates/:template/validate",
            post(handlers::validate_template),
        )
        // Repository type routes
        .route("/repository-types", get(handlers::list_repository_types))
        .route(
            "/repository-types/:type",
            get(handlers::get_repository_type_config),
        )
        // Configuration routes
        .route("/defaults", get(handlers::get_global_defaults))
        .route(
            "/configuration/preview",
            post(handlers::preview_configuration),
        )
        // Organization validation
        .route("/validate", post(handlers::validate_organization))
        // Team routes
        .route("/teams", get(handlers::list_organization_teams))
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;
