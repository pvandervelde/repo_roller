//! Tests for middleware module
//!
//! Unit tests use mock implementations. Real GitHub API validation
//! is tested in the integration_tests crate.

use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    middleware,
    Router,
    routing::get,
};
use tower::ServiceExt; // for `oneshot`

/// Test helper: create a simple handler that returns OK
async fn test_handler() -> &'static str {
    "OK"
}

/// Test that missing Authorization header returns 401
#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that invalid Authorization format returns 401
#[tokio::test]
async fn test_auth_middleware_invalid_format() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "InvalidFormat")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that non-Bearer scheme returns 401
#[tokio::test]
async fn test_auth_middleware_wrong_scheme() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Basic username:password")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that empty token returns 401
#[tokio::test]
async fn test_auth_middleware_empty_token() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer ")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test extract_bearer_token with valid format
#[test]
fn test_extract_bearer_token_valid() {
    let result = extract_bearer_token("Bearer my_token_123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my_token_123");
}

/// Test extract_bearer_token with invalid format
#[test]
fn test_extract_bearer_token_invalid_format() {
    let result = extract_bearer_token("InvalidFormat");
    assert!(result.is_err());
}

/// Test extract_bearer_token with wrong scheme
#[test]
fn test_extract_bearer_token_wrong_scheme() {
    let result = extract_bearer_token("Basic token");
    assert!(result.is_err());
}

/// Test AuthContext creation
#[test]
fn test_auth_context_creation() {
    let context = AuthContext::new("test_token".to_string());
    assert_eq!(context.token, "test_token");
    assert!(context.installation_id.is_none());
    assert!(context.organization.is_none());
}

/// Test AuthContext with installation ID
#[test]
fn test_auth_context_with_installation_id() {
    let context = AuthContext::new("test_token".to_string())
        .with_installation_id(12345);

    assert_eq!(context.installation_id, Some(12345));
}

/// Test AuthContext with organization
#[test]
fn test_auth_context_with_organization() {
    let context = AuthContext::new("test_token".to_string())
        .with_organization("myorg".to_string());

    assert_eq!(context.organization, Some("myorg".to_string()));
}

/// Test tracing middleware adds request logging
#[tokio::test]
async fn test_tracing_middleware_logs_requests() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// Note: Real GitHub API token validation is tested in integration_tests crate
// with actual GitHub App credentials. These unit tests focus on header parsing
// and format validation only.
