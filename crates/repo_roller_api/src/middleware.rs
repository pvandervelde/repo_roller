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

use crate::errors::{ErrorDetails, ErrorResponse};
use repo_roller_core::AuthenticationError;

/// Authentication context attached to requests after successful authentication.
///
/// This is stored in request extensions and can be extracted by handlers.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Bearer token from Authorization header
    pub token: String,

    /// Organization name (if validated)
    pub organization: Option<String>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(token: String) -> Self {
        Self {
            token,
            organization: None,
        }
    }

    /// Create context with organization
    pub fn with_organization(mut self, org: String) -> Self {
        self.organization = Some(org);
        self
    }
}

/// Authentication middleware that validates Bearer tokens.
///
/// Extracts the Authorization header, validates the token against GitHub API,
/// extracts the organization from the installation, and attaches authentication
/// context to the request.
///
/// Returns 401 if:
/// - Authorization header is missing
/// - Token is invalid or expired
/// - Token cannot be validated against GitHub
/// - Token has no associated installation
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
        .ok_or(AuthError::MissingToken)?;

    // Validate Bearer token format
    let token = extract_bearer_token(auth_header)?;

    // Validate token against GitHub API
    validate_token(&token).await?;

    // Create authentication context without organization
    // Organization will be extracted from path parameters by handlers
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

/// Validate token against GitHub API.
///
/// This function validates an installation token by making a simple API call
/// that installation tokens are authorized to make. We use the rate_limit
/// endpoint which is always accessible and requires authentication.
///
/// # Errors
///
/// Returns `AuthError::ValidationFailed` if:
/// - Token is invalid or expired (GitHub returns 401)
/// - GitHub API request fails
async fn validate_token(token: &str) -> Result<(), AuthError> {
    // Create GitHub client with the installation token
    let octocrab = github_client::create_token_client(token).map_err(|e| {
        tracing::warn!("Failed to create GitHub client with token: {}", e);
        AuthError::from(AuthenticationError::InvalidToken)
    })?;

    // Validate the token by calling a simple API that installation tokens can access
    // We'll use the rate_limit endpoint which is always accessible
    octocrab
        .ratelimit()
        .get()
        .await
        .map_err(|e| {
            tracing::warn!("Token validation failed - GitHub API error: {}", e);
            AuthError::from(AuthenticationError::AuthenticationFailed {
                reason: "Failed to validate token with GitHub API".to_string(),
            })
        })?;

    tracing::info!("Token validated successfully");

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
    // Organization authorization is a future enhancement (Task 9.7)
    // Will implement:
    // 1. Extract organization from path parameters
    // 2. Get authentication context from request extensions
    // 3. Verify user has access to organization (RBAC)
    // 4. Check team-based permissions

    // Currently: pass through all authenticated requests
    Ok(next.run(request).await)
}

/// Request tracing middleware.
///
/// Adds request ID and logging context for observability.
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
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

    /// Domain authentication error
    Authentication(AuthenticationError),
}

impl From<AuthenticationError> for AuthError {
    fn from(err: AuthenticationError) -> Self {
        AuthError::Authentication(err)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Authentication required. Provide a valid Bearer token in the Authorization header."
                    .to_string(),
                Some(json!({
                    "header": "Authorization",
                    "scheme": "Bearer"
                })),
            ),
            AuthError::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Invalid Authorization header format. Expected: 'Bearer <token>'".to_string(),
                Some(json!({
                    "header": "Authorization",
                    "expected_format": "Bearer <token>"
                })),
            ),
            AuthError::InvalidScheme => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Invalid authorization scheme. Only 'Bearer' tokens are supported.".to_string(),
                Some(json!({
                    "header": "Authorization",
                    "supported_scheme": "Bearer"
                })),
            ),
            AuthError::Authentication(auth_err) => {
                // Delegate to domain error conversion
                use crate::errors::convert_authentication_error;
                let (status, response) = convert_authentication_error(auth_err);
                return (status, Json(response)).into_response();
            }
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

#[cfg(test)]
#[path = "middleware_tests.rs"]
mod tests;
