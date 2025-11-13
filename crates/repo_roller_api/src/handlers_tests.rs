//! Tests for handlers module

use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::routes::create_router;

/// Helper function to create a test app state
fn test_app_state() -> AppState {
    AppState {
        // Empty state for now - will be populated when services are integrated
    }
}

// ============================================================================
// Health Check Tests
// ============================================================================

/// Test that health check handler returns proper JSON response
#[tokio::test]
async fn test_health_check_returns_json() {
    let response = health_check().await;

    // Verify structure exists
    assert_eq!(response.0.status, "healthy");
    assert!(response.0.version.is_some());
    assert!(!response.0.timestamp.is_empty());
    assert!(response.0.error.is_none());
}

/// Test that health check includes version from Cargo.toml
#[tokio::test]
async fn test_health_check_includes_version() {
    let response = health_check().await;

    assert_eq!(response.0.version, Some(env!("CARGO_PKG_VERSION").to_string()));
}

/// Test that health check timestamp is valid ISO 8601
#[tokio::test]
async fn test_health_check_timestamp_format() {
    let response = health_check().await;

    // Should be parseable as ISO 8601
    let parsed = chrono::DateTime::parse_from_rfc3339(&response.0.timestamp);
    assert!(parsed.is_ok(), "Timestamp should be valid ISO 8601 format");
}

// ============================================================================
// Repository Management Handler Tests
// ============================================================================

/// Test create_repository endpoint with valid request
///
/// Verifies that a valid repository creation request returns 201 Created
/// with complete repository information and applied configuration.
#[tokio::test]
async fn test_create_repository_success() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "visibility": "private",
        "team": "platform",
        "repositoryType": "library",
        "variables": {
            "projectName": "Test Project",
            "author": "Test Author"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify response structure
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check required fields in response
    assert!(response_json["repository"].is_object());
    assert!(response_json["repository"]["name"].is_string());
    assert!(response_json["repository"]["fullName"].is_string());
    assert!(response_json["repository"]["url"].is_string());
    assert!(response_json["repository"]["visibility"].is_string());
    assert!(response_json["repository"]["createdAt"].is_string());

    assert!(response_json["configuration"].is_object());
    assert!(response_json["configuration"]["appliedSettings"].is_object());
    assert!(response_json["configuration"]["sources"].is_object());
}

/// Test create_repository endpoint with missing required fields
///
/// Verifies that requests missing required fields return 400 Bad Request
/// with appropriate validation error.
#[tokio::test]
async fn test_create_repository_missing_required_fields() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo"
        // Missing template field
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Test create_repository endpoint with invalid repository name
///
/// Verifies that invalid repository names (e.g., uppercase, special chars)
/// return 400 Bad Request with validation error.
#[tokio::test]
async fn test_create_repository_invalid_name() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "Invalid@Name",  // Invalid characters
        "template": "rust-library",
        "variables": {}
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(error_json["error"].is_object());
    assert!(error_json["error"]["message"].as_str().unwrap().contains("name"));
}

/// Test create_repository endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_create_repository_no_auth() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "variables": {}
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories")
        .header("content-type", "application/json")
        // No authorization header
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test validate_repository_name endpoint with valid name
///
/// Verifies that a valid repository name returns 200 OK with valid=true.
#[tokio::test]
async fn test_validate_repository_name_valid() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "valid-repo-name"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], true);
    assert_eq!(response_json["name"], "valid-repo-name");
    assert!(response_json["errors"].is_array());
}

/// Test validate_repository_name endpoint with invalid name
///
/// Verifies that an invalid repository name returns 200 OK with valid=false
/// and includes validation error details.
#[tokio::test]
async fn test_validate_repository_name_invalid() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "Invalid@Name"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);
    assert_eq!(response_json["name"], "Invalid@Name");

    let errors = response_json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(errors[0]["field"].is_string());
    assert!(errors[0]["message"].is_string());
    assert!(errors[0]["constraint"].is_string());
}

/// Test validate_repository_name endpoint with empty name
///
/// Verifies that empty repository name returns 200 OK with valid=false
/// and appropriate error message.
#[tokio::test]
async fn test_validate_repository_name_empty() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": ""
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);
    assert!(!response_json["errors"].as_array().unwrap().is_empty());
}

/// Test validate_repository_request endpoint with valid complete request
///
/// Verifies that a valid complete repository creation request returns
/// 200 OK with valid=true and no errors.
#[tokio::test]
async fn test_validate_repository_request_valid() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "visibility": "private",
        "team": "platform",
        "repositoryType": "library",
        "variables": {
            "projectName": "Test Project",
            "author": "Test Author"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], true);
    assert!(response_json["errors"].is_array());
    assert!(response_json["errors"].as_array().unwrap().is_empty());
}

/// Test validate_repository_request endpoint with missing template variables
///
/// Verifies that missing required template variables result in valid=false
/// with specific error about the missing variables.
#[tokio::test]
async fn test_validate_repository_request_missing_variables() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "variables": {}  // Missing required variables
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);

    let errors = response_json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
}

/// Test validate_repository_request endpoint with non-existent template
///
/// Verifies that referencing a non-existent template results in valid=false
/// with appropriate error message.
#[tokio::test]
async fn test_validate_repository_request_nonexistent_template() {
    let app = create_router(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "nonexistent-template",
        "variables": {}
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);

    let errors = response_json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e|
        e["message"].as_str().unwrap().contains("template")
    ));
}
