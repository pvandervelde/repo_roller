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
    AppState::default()
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
    assert!(response_json["createdAt"].is_string());

    assert!(response_json["appliedConfiguration"].is_object());
}/// Test create_repository endpoint with missing required fields
///
/// Verifies that requests missing required fields return 422 Unprocessable Entity
/// (Axum's default for JSON deserialization errors).
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

    // Axum returns 422 for JSON deserialization failures
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
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
    assert_eq!(response_json["available"], true);
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
    assert_eq!(response_json["available"], false);

    // Check messages field exists and has content
    assert!(response_json["messages"].is_array());
    let messages = response_json["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
}/// Test validate_repository_name endpoint with empty name
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
    assert!(response_json["messages"].is_array());
    assert!(!response_json["messages"].as_array().unwrap().is_empty());
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
    // Errors field is omitted when empty due to skip_serializing_if
    if let Some(errors) = response_json.get("errors") {
        assert!(errors.as_array().unwrap().is_empty());
    }
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
        e["message"].as_str().unwrap().contains("template") ||
        e["field"].as_str().unwrap().contains("template")
    ));
}

// ============================================================================
// Template Discovery Handler Tests
// ============================================================================

/// Test list_templates endpoint returns available templates
///
/// Verifies that listing templates for an organization returns 200 OK
/// with an array of template summaries.
#[tokio::test]
async fn test_list_templates_success() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/templates")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["templates"].is_array());

    let templates = response_json["templates"].as_array().unwrap();
    assert!(!templates.is_empty(), "Should return at least one template");

    // Check template structure
    let first_template = &templates[0];
    assert!(first_template["name"].is_string());
    assert!(first_template["description"].is_string());
    assert!(first_template["variables"].is_array());
}

/// Test list_templates endpoint with no templates available
///
/// Verifies that when no templates exist, returns 200 OK with empty array.
#[tokio::test]
async fn test_list_templates_empty() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/emptyorg/templates")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_json["templates"].is_array());
    // Empty array is valid - organization may not have templates yet
}

/// Test list_templates endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_list_templates_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/templates")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test get_template_details endpoint with valid template
///
/// Verifies that requesting details for an existing template returns
/// 200 OK with complete template information including variables.
#[tokio::test]
async fn test_get_template_details_success() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/templates/rust-library")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check required fields
    assert!(response_json["name"].is_string());
    assert_eq!(response_json["name"], "rust-library");
    assert!(response_json["description"].is_string());
    assert!(response_json["variables"].is_object());
    assert!(response_json["configuration"].is_object());
}

/// Test get_template_details endpoint with non-existent template
///
/// Verifies that requesting a template that doesn't exist returns
/// 404 Not Found with appropriate error message.
#[tokio::test]
async fn test_get_template_details_not_found() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/templates/nonexistent-template")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(error_json["error"].is_object());
    assert!(error_json["error"]["message"].as_str().unwrap().contains("template"));
}

/// Test get_template_details endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_get_template_details_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/templates/rust-library")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test validate_template endpoint with valid template
///
/// Verifies that validating a well-formed template returns 200 OK
/// with valid=true.
#[tokio::test]
async fn test_validate_template_success() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/templates/rust-library/validate")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], true);
    // errors field should be empty or not present for valid templates
}

/// Test validate_template endpoint with invalid template structure
///
/// Verifies that validating a malformed template returns 200 OK
/// with valid=false and includes validation error details.
#[tokio::test]
async fn test_validate_template_invalid() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/templates/invalid-template/validate")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);

    // Should have errors array with validation issues
    if response_json.get("errors").is_some() {
        let errors = response_json["errors"].as_array().unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0]["field"].is_string());
        assert!(errors[0]["message"].is_string());
    }
}

/// Test validate_template endpoint with non-existent template
///
/// Verifies that validating a template that doesn't exist returns
/// 404 Not Found.
#[tokio::test]
async fn test_validate_template_not_found() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/templates/nonexistent/validate")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test validate_template endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_validate_template_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/templates/rust-library/validate")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Organization Settings Handler Tests
// ============================================================================

/// NOTE: Integration tests for organization settings endpoints moved to:
/// integration_tests/tests/rest_api_endpoints.rs
///
/// These tests require real GitHub infrastructure with configured metadata repositories.
/// The following tests verify authentication and basic request validation only.

/// Test list_repository_types endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_list_repository_types_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/repository-types")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test get_repository_type_config endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_get_repository_type_config_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/repository-types/library")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test get_global_defaults endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_get_global_defaults_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/defaults")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test preview_configuration endpoint with non-existent template
///
/// Verifies that preview fails gracefully with 404 for invalid template.
#[tokio::test]
async fn test_preview_configuration_template_not_found() {
    let app = create_router(test_app_state());

    let request_body = serde_json::json!({
        "template": "nonexistent-template"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/configuration/preview")
        .header("authorization", "Bearer test-token-123")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test preview_configuration endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_preview_configuration_no_auth() {
    let app = create_router(test_app_state());

    let request_body = serde_json::json!({
        "template": "rust-library"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/configuration/preview")
        .header("content-type", "application/json")
        // No authorization header
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test validate_organization endpoint with invalid configuration
///
/// Verifies that validation returns 200 OK with valid=false and error details
/// when organization configuration has issues.
#[tokio::test]
async fn test_validate_organization_invalid() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/invalidorg/validate")
        .header("authorization", "Bearer test-token-123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);

    // Should have validation errors
    let errors = response_json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(errors[0]["field"].is_string());
    assert!(errors[0]["message"].is_string());
}

/// Test validate_organization endpoint without authentication
///
/// Verifies that unauthenticated requests return 401 Unauthorized.
#[tokio::test]
async fn test_validate_organization_no_auth() {
    let app = create_router(test_app_state());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs/testorg/validate")
        // No authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
