//! Authentication and authorization middleware
//!
//! This module provides middleware for:
//! - Bearer token authentication
//! - GitHub App installation token validation
//! - Request authorization based on organization access
//!
//! See: specs/interfaces/api-error-handling.md#authentication-error-patterns

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::errors::{ErrorResponse, ErrorDetails};

// TODO: Import auth types when auth_handler is fully implemented
// use auth_handler::{UserAuthenticationService, Token};

/// Authentication context attached to requests after successful authentication.
///
/// This is stored in request extensions and can be extracted by handlers.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Bearer token from Authorization header
    pub token: String,

    /// Installation ID (extracted from token or validated separately)
    pub installation_id: Option<u64>,

    /// Organization name (if validated)
    pub organization: Option<String>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(token: String) -> Self {
        Self {
            token,
            installation_id: None,
            organization: None,
        }
    }

    /// Create context with installation ID
    pub fn with_installation_id(mut self, id: u64) -> Self {
        self.installation_id = Some(id);
        self
    }

    /// Create context with organization
    pub fn with_organization(mut self, org: String) -> Self {
        self.organization = Some(org);
        self
    }
}

/// Authentication middleware that validates Bearer tokens.
///
/// Extracts the Authorization header, validates the token,
/// and attaches authentication context to the request.
///
/// Returns 401 if:
/// - Authorization header is missing
/// - Token is invalid or expired
/// - Token cannot be validated
///
/// # Example
///
/// ```rust,ignore
/// let app = Router::new()
///     .route("/api/v1/repositories", post(create_repository))
///     .layer(middleware::from_fn(auth_middleware));
/// ```
pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AuthError::MissingToken)?;

    // Validate Bearer token format
    let token = extract_bearer_token(auth_header)?;

    // TODO: Validate token with auth_handler when available
    // For now, perform basic validation (non-empty, reasonable length)
    validate_token_format(&token)?;

    // Create authentication context
    let auth_context = AuthContext::new(token);

    // Attach context to request extensions
    request.extensions_mut().insert(auth_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header.
///
/// Expected format: "Bearer <token>"
fn extract_bearer_token(auth_header: &str) -> Result<String, AuthError> {
    let parts: Vec<&str> = auth_header.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(AuthError::InvalidFormat);
    }

    if parts[0].to_lowercase() != "bearer" {
        return Err(AuthError::InvalidScheme);
    }

    Ok(parts[1].to_string())
}

/// Validate token format (basic validation until auth_handler is complete).
///
/// Checks:
/// - Token is not empty
/// - Token has reasonable length (10-500 characters)
/// - Token contains only valid characters
fn validate_token_format(token: &str) -> Result<(), AuthError> {
    if token.is_empty() {
        return Err(AuthError::EmptyToken);
    }

    if token.len() < 10 || token.len() > 500 {
        return Err(AuthError::InvalidLength);
    }

    // GitHub tokens are typically base64-like with some special chars
    if !token.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err(AuthError::InvalidCharacters);
    }

    Ok(())
}

/// Organization authorization middleware.
///
/// Verifies that the authenticated user has access to the
/// organization specified in the request path.
///
/// Returns 403 if user doesn't have access to the organization.
pub async fn organization_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement organization authorization
    // 1. Extract organization from path parameters
    // 2. Get authentication context from request extensions
    // 3. Verify user has access to organization
    // 4. Call next middleware/handler

    // Placeholder: pass through all requests
    Ok(next.run(request).await)
}

/// Request tracing middleware.
///
/// Adds request ID and logging context for observability.
pub async fn tracing_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Generate request ID
    let request_id = uuid::Uuid::new_v4().to_string();

    // Log request start
    tracing::info!(
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        "Request started"
    );

    // Add request ID to response headers will be done by response interceptor
    let response = next.run(request).await;

    tracing::info!(
        request_id = %request_id,
        status = %response.status(),
        "Request completed"
    );

    response
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    /// Authorization header is missing
    MissingToken,

    /// Authorization header format is invalid
    InvalidFormat,

    /// Authorization scheme is not "Bearer"
    InvalidScheme,

    /// Token is empty
    EmptyToken,

    /// Token length is invalid
    InvalidLength,

    /// Token contains invalid characters
    InvalidCharacters,

    /// Token validation failed
    ValidationFailed(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message): (StatusCode, &str, String) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Authentication required. Provide a valid Bearer token in the Authorization header.".to_string(),
            ),
            AuthError::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Invalid Authorization header format. Expected: 'Bearer <token>'".to_string(),
            ),
            AuthError::InvalidScheme => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Invalid authorization scheme. Only 'Bearer' tokens are supported.".to_string(),
            ),
            AuthError::EmptyToken => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Authentication token cannot be empty.".to_string(),
            ),
            AuthError::InvalidLength => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Authentication token has invalid length.".to_string(),
            ),
            AuthError::InvalidCharacters => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Authentication token contains invalid characters.".to_string(),
            ),
            AuthError::ValidationFailed(msg) => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                msg,
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: Some(json!({
                    "header": "Authorization",
                    "scheme": "Bearer"
                })),
            },
        };

        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
#[path = "middleware_tests.rs"]
mod tests;
