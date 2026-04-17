//! Tests for middleware module
//!
//! Unit tests cover header parsing and JWT validation. Real GitHub API token
//! validation (used only by the exchange endpoint) is tested in the
//! integration_tests crate with actual GitHub App credentials.

use super::*;
use crate::AppState;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use tower::ServiceExt; // for `oneshot`

/// Test helper: create a simple handler that returns OK
async fn test_handler() -> &'static str {
    "OK"
}

/// Build a test router that applies `auth_middleware` with the default test state.
fn auth_test_app() -> Router {
    let state = AppState::default();
    Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Test that missing Authorization header returns 401
#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that invalid Authorization format returns 401
#[tokio::test]
async fn test_auth_middleware_invalid_format() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "InvalidFormat")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that non-Bearer scheme returns 401
#[tokio::test]
async fn test_auth_middleware_wrong_scheme() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Basic username:password")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that a raw GitHub token (not a backend JWT) is rejected with 401.
///
/// The middleware no longer validates GitHub tokens; passing one directly is
/// rejected as a malformed JWT.
#[tokio::test]
async fn test_auth_middleware_raw_github_token_rejected() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer ghp_someFakeGitHubToken")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that a valid backend JWT is accepted and the handler receives 200.
#[tokio::test]
async fn test_auth_middleware_valid_backend_jwt_accepted() {
    let secret = "test-jwt-secret-key-minimum-32b!";
    let token = generate_backend_jwt("alice", secret).expect("JWT generation must succeed");

    let state = AppState::default(); // uses the same test secret
    let app = Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test that a backend JWT signed with a different secret is rejected.
#[tokio::test]
async fn test_auth_middleware_jwt_wrong_secret_rejected() {
    // Sign with a different secret than the one in AppState::default().
    let token = generate_backend_jwt("alice", "wrong-secret-key-that-is-32-bytes!")
        .expect("JWT generation must succeed");

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = auth_test_app().oneshot(request).await.unwrap();
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
    let context = AuthContext::new();
    assert!(context.user_login.is_none());
}

/// Test that generate_backend_jwt produces a verifiable token.
#[test]
fn test_generate_backend_jwt_roundtrip() {
    let secret = "test-jwt-secret-key-minimum-32b!";
    let token = generate_backend_jwt("bob", secret).expect("should succeed");
    assert!(!token.is_empty());
    // The token must be a valid three-part JWT.
    assert_eq!(token.split('.').count(), 3);
}

/// Test tracing middleware adds request logging
#[tokio::test]
async fn test_tracing_middleware_logs_requests() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
