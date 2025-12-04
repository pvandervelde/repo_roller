//! Error handling and HTTP error conversion
//!
//! This module defines how domain errors are translated to HTTP error responses.
//! It implements the error mapping specified in:
//! - `specs/interfaces/api-error-handling.md`
//!
//! # Architecture
//!
//! Domain errors from `repo_roller_core` are converted to HTTP responses with
//! appropriate status codes and error messages. This conversion happens at the
//! HTTP boundary and never exposes internal implementation details.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Import domain errors from repo_roller_core
use repo_roller_core::{AuthenticationError, RepoRollerError};

/// Standard error response for all API errors.
///
/// All error responses follow this consistent structure to provide
/// machine-readable error codes and human-readable messages.
///
/// See: specs/interfaces/api-error-handling.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetails,
}

/// Error details structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetails {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional context (optional, type varies by error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Axum response wrapper for API errors
///
/// This type wraps domain errors and converts them to appropriate
/// HTTP responses when returning from handlers.
///
/// # Example
///
/// ```rust,ignore
/// async fn handler() -> Result<Json<Response>, ApiError> {
///     let result = domain_operation().await?; // Converts domain error to ApiError
///     Ok(Json(result.into()))
/// }
/// ```
#[derive(Debug)]
pub struct ApiError(anyhow::Error);

impl ApiError {
    /// Create a validation error with field information
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!(
            "validation error in field {}: {}",
            field.into(),
            message.into()
        ))
    }

    /// Create an internal error with a message
    ///
    /// Used for unexpected failures that should result in a 500 Internal Server Error.
    pub fn internal(message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!("internal error: {}", message.into()))
    }
}

impl From<RepoRollerError> for ApiError {
    fn from(err: RepoRollerError) -> Self {
        // Store the error directly in anyhow for later downcasting
        ApiError(anyhow::Error::new(err))
    }
}

impl From<AuthenticationError> for ApiError {
    fn from(err: AuthenticationError) -> Self {
        ApiError::from(RepoRollerError::Authentication(err))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Try to downcast to RepoRollerError for proper conversion
        let (status, error_response) =
            if let Some(repo_error) = self.0.downcast_ref::<RepoRollerError>() {
                convert_reporoller_error(repo_error)
            } else {
                // Fallback for non-RepoRollerError types (temporary during migration)
                convert_error(&self.0)
            };

        // Log error server-side
        log_error(&self.0, status);

        (status, Json(error_response)).into_response()
    }
}

/// Convert domain error to HTTP status code and error response
///
/// Provides error conversion for anyhow errors.
/// Domain-specific errors use the From<RepoRollerError> implementation.
fn convert_error(error: &anyhow::Error) -> (StatusCode, ErrorResponse) {
    // Fallback error conversion for anyhow errors
    let error_msg = error.to_string();

    let (status, code, message) =
        if error_msg.contains("authentication") || error_msg.contains("token") {
            (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                error.to_string(),
            )
        } else if error_msg.contains("not found") {
            (StatusCode::NOT_FOUND, "NotFound", error.to_string())
        } else if error_msg.contains("validation") || error_msg.contains("invalid") {
            (
                StatusCode::BAD_REQUEST,
                "ValidationError",
                error.to_string(),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "An internal error occurred".to_string(),
            )
        };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: None,
            },
        },
    )
}

/// Log error with appropriate level based on HTTP status
fn log_error(error: &anyhow::Error, status: StatusCode) {
    match status {
        StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY => {
            tracing::error!("API error: {} - {}", status, error);
        }
        StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
            tracing::warn!("API error: {} - {}", status, error);
        }
        _ => {
            tracing::info!("API error: {} - {}", status, error);
        }
    }
}

/// Convert AuthenticationError to HTTP status code and error response.
///
/// Maps authentication errors to appropriate HTTP status codes following
/// the specification in `specs/interfaces/api-error-handling.md`.
///
/// See: specs/interfaces/api-error-handling.md#authentication-error-patterns
pub fn convert_authentication_error(error: AuthenticationError) -> (StatusCode, ErrorResponse) {
    let (status, code, message, details) = match error {
        AuthenticationError::InvalidToken => (
            StatusCode::UNAUTHORIZED,
            "AuthenticationError",
            "Invalid or expired authentication token".to_string(),
            Some(json!({
                "header": "Authorization",
                "scheme": "Bearer"
            })),
        ),
        AuthenticationError::AuthenticationFailed { reason } => (
            StatusCode::UNAUTHORIZED,
            "AuthenticationError",
            format!("Authentication failed: {}", reason),
            None,
        ),
        AuthenticationError::InsufficientPermissions {
            operation,
            required,
        } => (
            StatusCode::FORBIDDEN,
            "AuthenticationError",
            format!("Insufficient permissions: {} permission required", required),
            Some(json!({
                "operation": operation,
                "required_permission": required
            })),
        ),
        AuthenticationError::UserNotFound { user_id } => (
            StatusCode::NOT_FOUND,
            "AuthenticationError",
            format!("User not found: {}", user_id),
            Some(json!({
                "user_id": user_id
            })),
        ),
        AuthenticationError::OrganizationAccessDenied { org } => (
            StatusCode::FORBIDDEN,
            "AuthenticationError",
            format!("Access denied to organization '{}'", org),
            Some(json!({
                "organization": org
            })),
        ),
        AuthenticationError::SessionExpired => (
            StatusCode::UNAUTHORIZED,
            "AuthenticationError",
            "Session expired, please log in again".to_string(),
            None,
        ),
        AuthenticationError::TokenRefreshFailed { reason } => (
            StatusCode::UNAUTHORIZED,
            "AuthenticationError",
            format!("Failed to refresh authentication token: {}", reason),
            None,
        ),
    };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details,
            },
        },
    )
}

/// Convert RepoRollerError to HTTP status code and error response.
///
/// Delegates to specific error type converters based on the error variant.
fn convert_reporoller_error(error: &RepoRollerError) -> (StatusCode, ErrorResponse) {
    match error {
        RepoRollerError::Authentication(e) => convert_authentication_error(e.clone()),
        RepoRollerError::Validation(e) => convert_validation_error(e),
        RepoRollerError::Repository(e) => convert_repository_error(e),
        RepoRollerError::Configuration(e) => convert_configuration_error(e),
        RepoRollerError::Template(e) => convert_template_error(e),
        RepoRollerError::GitHub(e) => convert_github_error(e),
        RepoRollerError::System(e) => convert_system_error(e),
    }
}

/// Convert ValidationError to HTTP error response.
///
/// Validation errors result in 400 Bad Request with details about what failed validation.
fn convert_validation_error(
    error: &repo_roller_core::ValidationError,
) -> (StatusCode, ErrorResponse) {
    use repo_roller_core::ValidationError;

    let (code, message, details) = match error {
        ValidationError::EmptyField { field } => (
            "ValidationError",
            format!("Field '{}' cannot be empty", field),
            Some(json!({ "field": field })),
        ),
        ValidationError::TooLong { field, actual, max } => (
            "ValidationError",
            format!(
                "Field '{}' is too long: {} characters (max: {})",
                field, actual, max
            ),
            Some(json!({ "field": field, "actual": actual, "max": max })),
        ),
        ValidationError::TooShort { field, actual, min } => (
            "ValidationError",
            format!(
                "Field '{}' is too short: {} characters (min: {})",
                field, actual, min
            ),
            Some(json!({ "field": field, "actual": actual, "min": min })),
        ),
        ValidationError::InvalidFormat { field, reason } => (
            "ValidationError",
            format!("Field '{}' has invalid format: {}", field, reason),
            Some(json!({ "field": field, "reason": reason })),
        ),
        ValidationError::InvalidRepositoryName { reason } => (
            "ValidationError",
            format!("Invalid repository name: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        ValidationError::InvalidOrganizationName { reason } => (
            "ValidationError",
            format!("Invalid organization name: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        ValidationError::InvalidTemplateName { reason } => (
            "ValidationError",
            format!("Invalid template name: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        ValidationError::RequiredFieldMissing { field } => (
            "ValidationError",
            format!("Required field '{}' is missing", field),
            Some(json!({ "field": field })),
        ),
        ValidationError::PatternMismatch {
            field,
            pattern,
            value,
        } => (
            "ValidationError",
            format!(
                "Field '{}' must match pattern '{}', got: '{}'",
                field, pattern, value
            ),
            Some(json!({ "field": field, "pattern": pattern, "value": value })),
        ),
        ValidationError::LengthConstraint {
            field,
            min,
            max,
            actual,
        } => (
            "ValidationError",
            format!(
                "Field '{}' length must be between {} and {}, got {}",
                field, min, max, actual
            ),
            Some(json!({ "field": field, "min": min, "max": max, "actual": actual })),
        ),
        ValidationError::InvalidOption {
            field,
            options,
            value,
        } => (
            "ValidationError",
            format!(
                "Field '{}' must be one of {:?}, got: '{}'",
                field, options, value
            ),
            Some(json!({ "field": field, "options": options, "value": value })),
        ),
    };

    (
        StatusCode::BAD_REQUEST,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details,
            },
        },
    )
}

/// Convert RepositoryError to HTTP error response.
fn convert_repository_error(
    error: &repo_roller_core::RepositoryError,
) -> (StatusCode, ErrorResponse) {
    use repo_roller_core::RepositoryError;

    let (status, code, message, details) = match error {
        RepositoryError::AlreadyExists { org, name } => (
            StatusCode::CONFLICT,
            "RepositoryAlreadyExists",
            format!("Repository '{}/{}' already exists", org, name),
            Some(json!({ "organization": org, "name": name })),
        ),
        RepositoryError::NotFound { org, name } => (
            StatusCode::NOT_FOUND,
            "RepositoryNotFound",
            format!("Repository '{}/{}' not found", org, name),
            Some(json!({ "organization": org, "name": name })),
        ),
        RepositoryError::CreationFailed { reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "RepositoryCreationFailed",
            format!("Failed to create repository: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        RepositoryError::PushFailed { reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "RepositoryPushFailed",
            format!("Failed to push repository content: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        RepositoryError::SettingsApplicationFailed { setting, reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SettingsApplicationFailed",
            format!("Failed to apply setting '{}': {}", setting, reason),
            Some(json!({ "setting": setting, "reason": reason })),
        ),
        RepositoryError::OperationTimeout { timeout_secs } => (
            StatusCode::REQUEST_TIMEOUT,
            "RepositoryOperationTimeout",
            format!(
                "Repository operation timed out after {} seconds",
                timeout_secs
            ),
            Some(json!({ "timeout_secs": timeout_secs })),
        ),
    };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details,
            },
        },
    )
}

/// Convert ConfigurationError to HTTP error response.
fn convert_configuration_error(
    error: &config_manager::ConfigurationError,
) -> (StatusCode, ErrorResponse) {
    use config_manager::ConfigurationError;

    let (status, code, message) = match error {
        ConfigurationError::MetadataRepositoryNotFound { org } => (
            StatusCode::NOT_FOUND,
            "MetadataRepositoryNotFound",
            format!("Metadata repository not found for organization '{}'", org),
        ),
        ConfigurationError::FileNotFound { path } => (
            StatusCode::NOT_FOUND,
            "ConfigurationFileNotFound",
            format!("Configuration file not found: {}", path),
        ),
        ConfigurationError::InvalidConfiguration { field, reason } => (
            StatusCode::BAD_REQUEST,
            "InvalidConfiguration",
            format!("Invalid configuration in field '{}': {}", field, reason),
        ),
        ConfigurationError::ParseError { reason } => (
            StatusCode::BAD_REQUEST,
            "ConfigurationParseError",
            format!("Failed to parse configuration: {}", reason),
        ),
        ConfigurationError::OverrideNotPermitted { setting, reason } => (
            StatusCode::FORBIDDEN,
            "OverrideNotAllowed",
            format!("Override not permitted for '{}': {}", setting, reason),
        ),
        ConfigurationError::ValidationFailed { error_count, .. } => (
            StatusCode::BAD_REQUEST,
            "ConfigurationValidationFailed",
            format!(
                "Configuration validation failed with {} error(s)",
                error_count
            ),
        ),
        ConfigurationError::FileAccessError { path, reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ConfigurationFileAccessError",
            format!("Cannot access configuration file '{}': {}", path, reason),
        ),
        ConfigurationError::RequiredConfigMissing { key } => (
            StatusCode::BAD_REQUEST,
            "RequiredConfigMissing",
            format!("Required configuration '{}' is missing", key),
        ),
        ConfigurationError::HierarchyResolutionFailed { reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ConfigurationHierarchyFailed",
            format!("Configuration hierarchy resolution failed: {}", reason),
        ),
    };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: None,
            },
        },
    )
}

/// Convert TemplateError to HTTP error response.
fn convert_template_error(error: &repo_roller_core::TemplateError) -> (StatusCode, ErrorResponse) {
    use repo_roller_core::TemplateError;

    let (status, code, message, details) = match error {
        TemplateError::TemplateNotFound { name } => (
            StatusCode::NOT_FOUND,
            "TemplateNotFound",
            format!("Template '{}' not found", name),
            Some(json!({ "name": name })),
        ),
        TemplateError::FetchFailed { reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "TemplateFetchFailed",
            format!("Failed to fetch template: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        TemplateError::SyntaxError { file, reason } => (
            StatusCode::BAD_REQUEST,
            "TemplateSyntaxError",
            format!("Syntax error in template file '{}': {}", file, reason),
            Some(json!({ "file": file, "reason": reason })),
        ),
        TemplateError::SubstitutionFailed { variable, reason } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "TemplateSubstitutionFailed",
            format!(
                "Variable substitution failed for '{}': {}",
                variable, reason
            ),
            Some(json!({ "variable": variable, "reason": reason })),
        ),
        TemplateError::RequiredVariableMissing { variable } => (
            StatusCode::BAD_REQUEST,
            "RequiredVariableMissing",
            format!("Required template variable '{}' is missing", variable),
            Some(json!({ "variable": variable })),
        ),
        TemplateError::ProcessingTimeout { timeout_secs } => (
            StatusCode::REQUEST_TIMEOUT,
            "TemplateProcessingTimeout",
            format!(
                "Template processing timed out after {} seconds",
                timeout_secs
            ),
            Some(json!({ "timeout_secs": timeout_secs })),
        ),
        TemplateError::SecurityViolation { reason } => (
            StatusCode::FORBIDDEN,
            "TemplateSecurityViolation",
            format!("Template security violation: {}", reason),
            Some(json!({ "reason": reason })),
        ),
        TemplateError::PathTraversalAttempt { path } => (
            StatusCode::FORBIDDEN,
            "PathTraversalAttempt",
            format!("Path traversal attempt detected: {}", path),
            Some(json!({ "path": path })),
        ),
    };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details,
            },
        },
    )
}

/// Convert GitHubError to HTTP error response.
fn convert_github_error(error: &repo_roller_core::GitHubError) -> (StatusCode, ErrorResponse) {
    use repo_roller_core::GitHubError;

    let (status, code, message) = match error {
        GitHubError::ApiRequestFailed {
            status: api_status,
            message: api_message,
        } => (
            StatusCode::BAD_GATEWAY,
            "GitHubApiRequestFailed",
            format!(
                "GitHub API request failed ({}): {}",
                api_status, api_message
            ),
        ),
        GitHubError::AuthenticationFailed { reason } => (
            StatusCode::UNAUTHORIZED,
            "GitHubAuthenticationFailed",
            format!("GitHub authentication failed: {}", reason),
        ),
        GitHubError::RateLimitExceeded { reset_at } => (
            StatusCode::TOO_MANY_REQUESTS,
            "GitHubRateLimitExceeded",
            format!("GitHub API rate limit exceeded, resets at {}", reset_at),
        ),
        GitHubError::ResourceNotFound { resource } => (
            StatusCode::NOT_FOUND,
            "GitHubResourceNotFound",
            format!("GitHub resource not found: {}", resource),
        ),
        GitHubError::NetworkError { reason } => (
            StatusCode::BAD_GATEWAY,
            "GitHubNetworkError",
            format!("Network error communicating with GitHub: {}", reason),
        ),
        GitHubError::InvalidResponse { reason } => (
            StatusCode::BAD_GATEWAY,
            "GitHubInvalidResponse",
            format!("Invalid response from GitHub API: {}", reason),
        ),
        GitHubError::AppNotInstalled { org } => (
            StatusCode::FORBIDDEN,
            "GitHubAppNotInstalled",
            format!("GitHub App not installed on organization '{}'", org),
        ),
    };

    (
        status,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: None,
            },
        },
    )
}

/// Convert SystemError to HTTP error response.
fn convert_system_error(error: &repo_roller_core::SystemError) -> (StatusCode, ErrorResponse) {
    use repo_roller_core::SystemError;

    let (code, message) = match error {
        SystemError::Internal { reason } => (
            "InternalError",
            format!("Internal system error: {}", reason),
        ),
        SystemError::FileSystem { operation, reason } => (
            "FileSystemError",
            format!("File system error during '{}': {}", operation, reason),
        ),
        SystemError::GitOperation { operation, reason } => (
            "GitOperationError",
            format!("Git operation '{}' failed: {}", operation, reason),
        ),
        SystemError::Network { reason } => ("NetworkError", format!("Network error: {}", reason)),
        SystemError::Serialization { reason } => (
            "SerializationError",
            format!("Serialization error: {}", reason),
        ),
        SystemError::Deserialization { reason } => (
            "DeserializationError",
            format!("Deserialization error: {}", reason),
        ),
        SystemError::ResourceUnavailable { resource } => (
            "ResourceUnavailable",
            format!("Resource unavailable: {}", resource),
        ),
    };

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: None,
            },
        },
    )
}

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;
