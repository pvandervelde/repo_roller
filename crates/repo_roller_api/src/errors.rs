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

// TODO: Import domain errors when error-types.md is implemented
// use repo_roller_core::errors::*;

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
pub struct ApiError(anyhow::Error); // TODO: Change to RepoRollerError when available

impl ApiError {
    /// Create a new API error from any error type
    pub fn new(err: impl Into<anyhow::Error>) -> Self {
        ApiError(err.into())
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!(message.into()))
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!(message.into()))
    }
    
    /// Create a validation error with field information
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!("validation error in field {}: {}", field.into(), message.into()))
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!(message.into()))
    }

    /// Create an internal server error
    pub fn internal(message: impl Into<String>) -> Self {
        ApiError(anyhow::anyhow!(message.into()))
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        ApiError(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // TODO: Implement proper domain error to HTTP conversion
        // when RepoRollerError is available from repo_roller_core
        let (status, error_response) = convert_error(&self.0);

        // Log error server-side
        log_error(&self.0, status);

        (status, Json(error_response)).into_response()
    }
}

/// Convert domain error to HTTP status code and error response
///
/// TODO: Replace with proper IntoHttpError trait implementation
/// when domain errors are available.
fn convert_error(error: &anyhow::Error) -> (StatusCode, ErrorResponse) {
    // Temporary implementation - will be replaced with proper error matching
    let error_msg = error.to_string();

    let (status, code, message) = if error_msg.contains("authentication")
        || error_msg.contains("token")
    {
        (
            StatusCode::UNAUTHORIZED,
            "AuthenticationError",
            error.to_string(),
        )
    } else if error_msg.contains("validation") || error_msg.contains("invalid") {
        (
            StatusCode::BAD_REQUEST,
            "ValidationError",
            error.to_string(),
        )
    } else if error_msg.contains("not found") {
        (StatusCode::NOT_FOUND, "NotFound", error.to_string())
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

// TODO: Implement IntoHttpError trait for each domain error type
// when repo_roller_core/errors.rs is implemented:
//
// pub trait IntoHttpError {
//     fn into_http_error(self) -> (StatusCode, ErrorResponse);
// }
//
// impl IntoHttpError for ValidationError { ... }
// impl IntoHttpError for RepositoryError { ... }
// impl IntoHttpError for ConfigurationError { ... }
// impl IntoHttpError for TemplateError { ... }
// impl IntoHttpError for AuthenticationError { ... }
// impl IntoHttpError for GitHubError { ... }
// impl IntoHttpError for SystemError { ... }
