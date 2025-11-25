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
use github_client::GitHubClient;
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

    // Validate token against GitHub API and extract organization
    let organization = validate_token_and_extract_org(&token).await?;

    // Create authentication context with validated organization
    let auth_context = AuthContext::new(token).with_organization(organization);

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

/// Validate token against GitHub API and extract organization name.
///
/// This function:
/// 1. Creates a GitHub client with the provided token
/// 2. Lists installations accessible with the token
/// 3. Extracts the organization from the installation
///
/// # Errors
///
/// Returns `AuthError::ValidationFailed` if:
/// - Token is invalid or expired (GitHub returns 401)
/// - Token has no associated installations
/// - GitHub API request fails
async fn validate_token_and_extract_org(token: &str) -> Result<String, AuthError> {
    // Create GitHub client with the installation token
    let octocrab = github_client::create_token_client(token).map_err(|e| {
        tracing::warn!("Failed to create GitHub client with token: {}", e);
        AuthError::from(AuthenticationError::InvalidToken)
    })?;

    let client = GitHubClient::new(octocrab);

    // List installations to validate token and extract organization
    let installations = client.list_installations().await.map_err(|e| {
        tracing::warn!("Token validation failed - GitHub API error: {}", e);
        AuthError::from(AuthenticationError::AuthenticationFailed {
            reason: "Failed to validate token with GitHub API".to_string(),
        })
    })?;

    // Installation tokens should have at least one installation
    if installations.is_empty() {
        tracing::warn!("Token has no associated installations");
        return Err(AuthError::from(AuthenticationError::AuthenticationFailed {
            reason: "Token has no associated GitHub App installations".to_string(),
        }));
    }

    // Extract organization from the first installation
    // (Installation tokens are scoped to a single installation)
    let org_name = installations[0].account.login.clone();

    tracing::info!(
        organization = %org_name,
        "Token validated successfully"
    );

    Ok(org_name)
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
