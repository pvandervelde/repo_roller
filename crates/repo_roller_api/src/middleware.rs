//! Authentication and authorization middleware
//!
//! This module provides middleware for:
//! - Backend JWT validation (issued by `POST /api/v1/auth/token`)
//! - GitHub token exchange helpers (used by the exchange endpoint)
//! - Request tracing
//!
//! # Authentication flow
//!
//! 1. The frontend calls `POST /api/v1/auth/token` with a GitHub user token.
//! 2. The exchange endpoint validates the GitHub token once against GitHub,
//!    resolves the user login, and returns a short-lived backend-signed JWT.
//! 3. The frontend presents that backend JWT as `Authorization: Bearer` on
//!    every subsequent request.
//! 4. `auth_middleware` validates the JWT locally — no GitHub API call is made
//!    per protected request.
//!
//! See: specs/interfaces/api-error-handling.md#authentication-error-patterns

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::errors::{ErrorDetails, ErrorResponse};
use crate::AppState;
use repo_roller_core::AuthenticationError;

/// JWT lifetime for backend-issued tokens.
pub(crate) const JWT_EXPIRY_SECS: u64 = 8 * 3600; // 8 hours

/// Claims embedded in a backend-issued JWT.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// GitHub login of the authenticated user.
    pub sub: String,
    /// Issued-at timestamp (seconds since UNIX epoch).
    pub iat: usize,
    /// Expiry timestamp (seconds since UNIX epoch).
    pub exp: usize,
}

/// Sign a new backend JWT for `user_login`.
///
/// Uses HS256 with the provided secret bytes.  The caller is responsible for
/// exposing `AppState::jwt_secret` via `secrecy::ExposeSecret` before calling
/// this function.
///
/// # Errors
///
/// Returns `AuthError` if JWT encoding fails (e.g. invalid secret).
pub(crate) fn generate_backend_jwt(user_login: &str, secret: &str) -> Result<String, AuthError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = Claims {
        sub: user_login.to_string(),
        iat: now as usize,
        exp: (now + JWT_EXPIRY_SECS) as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to encode backend JWT: {}", e);
        AuthError::Authentication(AuthenticationError::AuthenticationFailed {
            reason: "JWT encoding failed".to_string(),
        })
    })
}

/// Validate a backend JWT and return its claims.
///
/// Rejects tokens with invalid signatures, expired timestamps, or wrong
/// algorithm.  Uses HS256 with the provided secret bytes.
fn validate_backend_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> {
    // Use 30-second leeway to tolerate minor clock drift between the issuing
    // server and the validating server without meaningfully extending the
    // effective 8-hour session window.
    let mut validation = Validation::default();
    validation.leeway = 30;
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            tracing::debug!("Rejected expired backend JWT");
            AuthError::TokenExpired
        }
        _ => {
            tracing::debug!("Rejected invalid backend JWT: {}", e);
            AuthError::Authentication(AuthenticationError::InvalidToken)
        }
    })
}

/// Authentication context attached to requests after successful authentication.
///
/// This is stored in request extensions and can be extracted by handlers.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// GitHub login of the authenticated actor.
    ///
    /// Populated from the `sub` claim of the validated backend JWT.
    /// `None` only when the JWT was issued without a resolvable GitHub login
    /// (e.g. the GitHub `/user` call failed during token exchange).
    pub user_login: Option<String>,
}

impl AuthContext {
    /// Create a new authentication context.
    ///
    /// Only used in tests — production code constructs `AuthContext` directly
    /// inside `auth_middleware` after JWT validation.
    #[cfg(test)]
    pub fn new() -> Self {
        Self { user_login: None }
    }
}

/// Authentication middleware that validates backend-issued JWTs.
///
/// Extracts the `Authorization: Bearer` header and validates it as a
/// backend-signed JWT.  The JWT must have been issued by
/// `POST /api/v1/auth/token`; raw GitHub tokens are rejected.
///
/// Returns 401 if:
/// - Authorization header is missing
/// - Header format is not `Bearer <token>`
/// - JWT signature is invalid
/// - JWT is expired
///
/// # Example
///
/// ```rust,ignore
/// let app = Router::new()
///     .route("/api/v1/repositories", post(create_repository))
///     .route_layer(middleware::from_fn_with_state(state, auth_middleware));
/// ```
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    use secrecy::ExposeSecret;

    // Extract Authorization header.
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Validate Bearer token format.
    let token = extract_bearer_token(auth_header)?;

    // Validate the backend JWT locally — no network call.
    let claims = validate_backend_jwt(&token, state.jwt_secret.expose_secret())?;

    tracing::debug!(user_login = %claims.sub, "Backend JWT validated");

    let auth_context = AuthContext {
        user_login: Some(claims.sub),
    };

    request.extensions_mut().insert(auth_context);
    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header.
///
/// Expected format: `Bearer <token>`
pub(crate) fn extract_bearer_token(auth_header: &str) -> Result<String, AuthError> {
    let parts: Vec<&str> = auth_header.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(AuthError::InvalidFormat);
    }

    if parts[0].to_lowercase() != "bearer" {
        return Err(AuthError::InvalidScheme);
    }

    Ok(parts[1].to_string())
}

/// Validate a GitHub user token against the GitHub API.
///
/// Calls `GET /rate_limit` — a lightweight endpoint accessible to all valid
/// tokens — to confirm the token has not been revoked or expired.
///
/// Used only by the token exchange endpoint (`POST /api/v1/auth/token`).
///
/// # Errors
///
/// Returns `AuthError` if the GitHub API call fails (401, 403, network error).
pub(crate) async fn validate_github_token(token: &str) -> Result<(), AuthError> {
    let octocrab = github_client::create_token_client(token).map_err(|e| {
        tracing::warn!("Failed to create GitHub client with token: {}", e);
        AuthError::from(AuthenticationError::InvalidToken)
    })?;

    octocrab.ratelimit().get().await.map_err(|e| {
        tracing::warn!("GitHub token validation failed: {}", e);
        AuthError::from(AuthenticationError::AuthenticationFailed {
            reason: "Failed to validate token with GitHub API".to_string(),
        })
    })?;

    Ok(())
}

/// Attempt to retrieve the GitHub login for the token bearer.
///
/// Calls `GET /user`.  Succeeds for PAT and OAuth tokens; fails silently for
/// installation tokens (which are not scoped to a user).  Failure is
/// non-fatal — the exchange endpoint falls back to `"unknown"`.
pub(crate) async fn try_get_user_login(token: &str) -> Option<String> {
    let octocrab = github_client::create_token_client(token).ok()?;
    match octocrab.current().user().await {
        Ok(user) => Some(user.login.clone()),
        Err(e) => {
            tracing::debug!(
                "Could not resolve user login from token (expected for installation tokens): {}",
                e
            );
            None
        }
    }
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
    Ok(next.run(request).await)
}

/// Request tracing middleware.
///
/// Adds request ID and logging context for observability.
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();

    tracing::info!(
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        "Request started"
    );

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

    /// Backend JWT has expired; the client must re-authenticate via `/auth/token`
    TokenExpired,

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
            AuthError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "TokenExpired",
                "Session expired. Please re-authenticate via POST /api/v1/auth/token."
                    .to_string(),
                None,
            ),
            AuthError::Authentication(auth_err) => {
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
