//! Tests for routes module

use super::*;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt; // for .oneshot()

#[test]
fn test_router_creation() {
    let state = AppState::default();
    let _router = create_router(state);
    // Router creation should succeed
}

/// Verify the health check endpoint returns 200 without authentication
/// (health check is publicly accessible by design).
#[tokio::test]
async fn test_health_check_endpoint_returns_200() {
    let state = AppState::default();
    // Use the no-auth router so we isolate just the health handler
    let router = create_router_without_auth(state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Verify that a request without a valid Authorization header is rejected (401).
#[tokio::test]
async fn test_protected_endpoint_requires_auth() {
    let state = AppState::default();
    // Use the full router which includes the auth middleware
    let router = create_router(state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/repositories")
        .header("Content-Type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
