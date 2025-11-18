//! Tests for error handling and HTTP conversion

use super::*;
use axum::http::StatusCode;
use repo_roller_core::AuthenticationError;

#[test]
fn test_authentication_error_invalid_token() {
    let error = AuthenticationError::InvalidToken;
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(
        response.error.message,
        "Invalid or expired authentication token"
    );

    // Verify details contain header information
    let details = response.error.details.expect("Should have details");
    assert_eq!(details["header"], "Authorization");
    assert_eq!(details["scheme"], "Bearer");
}

#[test]
fn test_authentication_error_authentication_failed() {
    let error = AuthenticationError::AuthenticationFailed {
        reason: "GitHub API returned 401".to_string(),
    };
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(
        response.error.message,
        "Authentication failed: GitHub API returned 401"
    );
    assert!(response.error.details.is_none());
}

#[test]
fn test_authentication_error_insufficient_permissions() {
    let error = AuthenticationError::InsufficientPermissions {
        operation: "create_repository".to_string(),
        required: "repository_write".to_string(),
    };
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(
        response.error.message,
        "Insufficient permissions: repository_write permission required"
    );

    // Verify details contain operation and permission
    let details = response.error.details.expect("Should have details");
    assert_eq!(details["operation"], "create_repository");
    assert_eq!(details["required_permission"], "repository_write");
}

#[test]
fn test_authentication_error_user_not_found() {
    let error = AuthenticationError::UserNotFound {
        user_id: "user123".to_string(),
    };
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(response.error.message, "User not found: user123");

    let details = response.error.details.expect("Should have details");
    assert_eq!(details["user_id"], "user123");
}

#[test]
fn test_authentication_error_organization_access_denied() {
    let error = AuthenticationError::OrganizationAccessDenied {
        org: "myorg".to_string(),
    };
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(
        response.error.message,
        "Access denied to organization 'myorg'"
    );

    let details = response.error.details.expect("Should have details");
    assert_eq!(details["organization"], "myorg");
}

#[test]
fn test_authentication_error_session_expired() {
    let error = AuthenticationError::SessionExpired;
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(response.error.message, "Session expired, please log in again");
    assert!(response.error.details.is_none());
}

#[test]
fn test_authentication_error_token_refresh_failed() {
    let error = AuthenticationError::TokenRefreshFailed {
        reason: "Network timeout".to_string(),
    };
    let (status, response) = convert_authentication_error(error);

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.error.code, "AuthenticationError");
    assert_eq!(
        response.error.message,
        "Failed to refresh authentication token: Network timeout"
    );
    assert!(response.error.details.is_none());
}

#[test]
fn test_reporoller_error_authentication_variant() {
    let error = RepoRollerError::Authentication(AuthenticationError::InvalidToken);
    let (status, response) = convert_reporoller_error(&error);

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.error.code, "AuthenticationError");
    assert!(response
        .error
        .message
        .contains("Invalid or expired authentication token"));
}

#[test]
fn test_api_error_from_authentication_error() {
    let auth_error = AuthenticationError::InvalidToken;
    let api_error = ApiError::from(auth_error);

    // Should convert without panic
    assert!(format!("{:?}", api_error.0).contains("Invalid or expired token"));
}

#[test]
fn test_error_response_serialization() {
    let response = ErrorResponse {
        error: ErrorDetails {
            code: "AuthenticationError".to_string(),
            message: "Test message".to_string(),
            details: Some(json!({"key": "value"})),
        },
    };

    let serialized = serde_json::to_string(&response).expect("Should serialize");
    assert!(serialized.contains("AuthenticationError"));
    assert!(serialized.contains("Test message"));
    // Verify camelCase serialization (field names should use camelCase)
    assert!(!serialized.contains("error_code")); // Not snake_case
}

#[test]
fn test_error_response_without_details() {
    let response = ErrorResponse {
        error: ErrorDetails {
            code: "AuthenticationError".to_string(),
            message: "Test message".to_string(),
            details: None,
        },
    };

    let serialized = serde_json::to_string(&response).expect("Should serialize");
    // Details should be omitted when None
    assert!(!serialized.contains("details"));
}

// TODO: Add tests for other RepoRollerError variants:
// - ValidationError → 400 Bad Request
// - RepositoryError → various status codes depending on variant
// - ConfigurationError → 500 Internal Server Error or 400 Bad Request
// - TemplateError → 400 Bad Request or 404 Not Found
// - GitHubError → 502 Bad Gateway or 500 Internal Server Error  
// - SystemError → 500 Internal Server Error
// These tests will be added as error conversion functions are implemented.
