//! Tests for error handling and HTTP conversion

use super::*;
use axum::{http::StatusCode, response::IntoResponse};
use repo_roller_core::{AuthenticationError, RepoRollerError};

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
    assert_eq!(
        response.error.message,
        "Session expired, please log in again"
    );
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

// ============================================================================
// RepoRollerError variant conversion tests
// ============================================================================

#[test]
fn test_validation_error_empty_field_returns_400() {
    use repo_roller_core::ValidationError;
    let error = RepoRollerError::Validation(ValidationError::EmptyField {
        field: "name".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_validation_error_invalid_repository_name_returns_400() {
    use repo_roller_core::ValidationError;
    let error = RepoRollerError::Validation(ValidationError::InvalidRepositoryName {
        reason: "contains invalid characters".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_repository_error_already_exists_returns_409() {
    use repo_roller_core::RepositoryError;
    let error = RepoRollerError::Repository(RepositoryError::AlreadyExists {
        org: "my-org".to_string(),
        name: "my-repo".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[test]
fn test_repository_error_not_found_returns_404() {
    use repo_roller_core::RepositoryError;
    let error = RepoRollerError::Repository(RepositoryError::NotFound {
        org: "my-org".to_string(),
        name: "my-repo".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_repository_error_creation_failed_returns_500() {
    use repo_roller_core::RepositoryError;
    let error = RepoRollerError::Repository(RepositoryError::CreationFailed {
        reason: "GitHub API error".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_configuration_error_not_found_returns_404() {
    use config_manager::ConfigurationError;
    let error = RepoRollerError::Configuration(ConfigurationError::MetadataRepositoryNotFound {
        org: "my-org".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_configuration_error_invalid_returns_400() {
    use config_manager::ConfigurationError;
    let error = RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
        field: "webhooks[0].url".to_string(),
        reason: "must use HTTPS".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_template_error_not_found_returns_404() {
    use repo_roller_core::TemplateError;
    let error = RepoRollerError::Template(TemplateError::TemplateNotFound {
        name: "rust-service".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_template_error_syntax_returns_400() {
    use repo_roller_core::TemplateError;
    let error = RepoRollerError::Template(TemplateError::SyntaxError {
        file: "README.md".to_string(),
        reason: "unclosed block".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_github_error_api_failure_returns_502() {
    use repo_roller_core::GitHubError;
    let error = RepoRollerError::GitHub(GitHubError::ApiRequestFailed {
        status: 503,
        message: "Service Unavailable".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_github_error_rate_limit_returns_429() {
    use repo_roller_core::GitHubError;
    let error = RepoRollerError::GitHub(GitHubError::RateLimitExceeded {
        reset_at: "2026-01-01T00:00:00Z".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_system_error_internal_returns_500() {
    use repo_roller_core::SystemError;
    let error = RepoRollerError::System(SystemError::Internal {
        reason: "unexpected panic".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_system_error_filesystem_returns_500() {
    use repo_roller_core::SystemError;
    let error = RepoRollerError::System(SystemError::FileSystem {
        operation: "read".to_string(),
        reason: "permission denied".to_string(),
    });
    let api_error = ApiError::from(error);
    let response = api_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
