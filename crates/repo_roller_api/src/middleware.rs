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
use tracing::Instrument;

use crate::errors::{ErrorDetails, ErrorResponse};
use crate::AppState;
use repo_roller_core::AuthenticationError;

/// JWT lifetime for backend-issued tokens.
pub(crate) const JWT_EXPIRY_SECS: u64 = 8 * 3600; // 8 hours

/// Claims embedded in a backend-issued JWT.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// GitHub login of the authenticated user. Absent (not serialized) for
    /// loginless principals, which are identified solely by `pid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Stable opaque principal identifier, present only for principals with
    /// no resolvable GitHub login (e.g. installation tokens). Absent for
    /// ordinary user tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<String>,
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
    sign_claims(Some(user_login), None, secret)
}

/// Issue a backend JWT for a principal with no resolvable login (for example a
/// GitHub App installation token).
///
/// The token carries `principal_id` in the `pid` claim. `auth_middleware`
/// then exposes an `AuthContext` with no `user_login` and
/// `principal_id = Some(principal_id)`, which the rate limiter keys on so
/// distinct principals never share a bucket.
///
/// # Errors
///
/// Returns `AuthError` if JWT encoding fails (e.g. invalid secret).
pub(crate) fn generate_loginless_backend_jwt(
    principal_id: &str,
    secret: &str,
) -> Result<String, AuthError> {
    sign_claims(None, Some(principal_id), secret)
}

/// Sign HS256 claims for `sub` and the optional `pid`.
fn sign_claims(sub: Option<&str>, pid: Option<&str>, secret: &str) -> Result<String, AuthError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = Claims {
        sub: sub.map(str::to_string),
        pid: pid.map(str::to_string),
        iat: now as usize,
        exp: (now + JWT_EXPIRY_SECS) as usize,
    };

    let encoding_key = EncodingKey::from_secret(secret.as_bytes()).map_err(|e| {
        tracing::error!("Failed to build backend JWT encoding key: {}", e);
        AuthError::Authentication(AuthenticationError::AuthenticationFailed {
            reason: "JWT encoding failed".to_string(),
        })
    })?;

    jsonwebtoken::encode(&Header::default(), &claims, &encoding_key).map_err(|e| {
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
    let validation = Validation {
        leeway: 30,
        ..Default::default()
    };
    let decoding_key = DecodingKey::from_secret(secret.as_bytes()).map_err(|e| {
        tracing::debug!("Failed to build backend JWT decoding key: {}", e);
        AuthError::Authentication(AuthenticationError::InvalidToken)
    })?;

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
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
    /// Stable opaque principal identifier from the validated JWT `pid` claim.
    /// `Some` only for loginless principals.
    pub principal_id: Option<String>,
}

/// Derive a stable opaque principal id from a GitHub token.
///
/// Returns the lowercase hex SHA-256 digest of the token. The digest is
/// one-way, so the id never embeds or reveals the token, yet is identical for
/// every request made with the same token.
pub(crate) fn principal_id_for_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(token.as_bytes()))
}

impl AuthContext {
    /// Create a new authentication context.
    ///
    /// Only used in tests — production code constructs `AuthContext` directly
    /// inside `auth_middleware` after JWT validation.
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            user_login: None,
            principal_id: None,
        }
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
/// - JWT carries neither a non-empty `pid` nor a non-empty `sub` (code `MissingPrincipal`)
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

    tracing::debug!("Backend JWT validated");

    // Precedence: a non-empty `pid` claim (loginless principal) wins over
    // `sub`. A token with neither a usable `pid` nor a usable `sub` identifies
    // no principal and is rejected.
    let auth_context = match (
        claims.pid.filter(|p| !p.is_empty()),
        claims.sub.filter(|s| !s.is_empty()),
    ) {
        (Some(pid), _) => AuthContext {
            user_login: None,
            principal_id: Some(pid),
        },
        (None, Some(login)) => AuthContext {
            user_login: Some(login),
            principal_id: None,
        },
        (None, None) => return Err(AuthError::MissingPrincipalIdentifier),
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
/// `base_url` overrides the GitHub API base URL (`None` means
/// `https://api.github.com`).
///
/// # Errors
///
/// Returns `AuthError` if the GitHub API call fails (401, 403, network error).
pub(crate) async fn validate_github_token(
    token: &str,
    base_url: Option<&str>,
) -> Result<(), AuthError> {
    let octocrab = github_client::create_octocrab_client(token, base_url).map_err(|e| {
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
/// non-fatal — the exchange endpoint then issues a loginless token that
/// carries only a `pid` claim.
pub(crate) async fn try_get_user_login(token: &str, base_url: Option<&str>) -> Option<String> {
    let octocrab = github_client::create_octocrab_client(token, base_url).ok()?;
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

/// Extracts a W3C trace context from an inbound HTTP request's headers, for
/// use as the parent context of the `http_request` span built by
/// [`tracing_middleware`].
///
/// # Contract (Task #3 backend half: inbound `traceparent` propagation)
///
/// - Reads the standard `traceparent` header (format
///   `{version:2 hex}-{trace_id:32 hex}-{parent_id:16 hex}-{flags:2 hex}`,
///   see <https://www.w3.org/TR/trace-context/#traceparent-header>) from
///   `headers`, decoding it via `opentelemetry::global::get_text_map_propagator`
///   over an `opentelemetry::propagation::Extractor` adapter -- the
///   idiomatic `opentelemetry`/`tracing-opentelemetry` ecosystem pattern for
///   this ecosystem version (no bespoke parsing).
/// - **Valid header** (well-formed, non-zero trace id, non-zero parent id,
///   lowercase hex throughout): returns an `opentelemetry::Context` carrying
///   a REMOTE `SpanContext` -- `span().span_context().is_valid()` and
///   `.is_remote()` are both `true`, `.trace_id()` / `.span_id()` decode
///   exactly the header's trace-id / parent-id fields. Passing this context
///   to `tracing_opentelemetry::OpenTelemetrySpanExt::set_parent` on a
///   newly-created, **not-yet-entered** span (call `set_parent` before the
///   span is entered/instrumented -- `tracing-opentelemetry` rejects
///   `set_parent` on an already-entered span, see its own
///   `set_parent_fails_after_span_entered` test) makes that span, and every
///   span nested under it, a child of the external trace.
/// - **Absent header, or malformed/unparsable value** (wrong number of
///   dash-delimited parts, non-hex characters, uppercase hex, all-zero trace
///   id, all-zero parent id, empty string, or any other input that is not a
///   spec-valid `traceparent`): returns a context with NO valid span
///   (`span().span_context().is_valid() == false`). Passing this to
///   `set_parent` is harmless and equivalent to a fresh root span -- i.e.
///   today's exact pre-Task-3 behavior. **Must never panic** regardless of
///   header content; the W3C spec requires invalid `traceparent` values to
///   be ignored, not rejected (fail-safe, not fail-closed).
/// - Does not read or require the `tracestate` header -- out of scope for
///   this task (it rides alongside `traceparent` as opaque vendor data; only
///   `traceparent` carries the parent trace/span identity this task needs).
/// - Pure with respect to its `headers` argument: two calls with the same
///   header content return equivalent contexts; a call with no/invalid
///   header never leaks a valid context observed by an earlier call.
///
/// # Precondition (production wiring)
///
/// `opentelemetry::global::get_text_map_propagator` falls back to a no-op
/// propagator (which never decodes anything) until something calls
/// `opentelemetry::global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new())`.
/// Production startup (`tracing_otel::init_tracing_with_otel`) makes that
/// call once, so every inbound `traceparent` header is decoded for real
/// requests. `middleware_tests.rs` / `middleware_proptests.rs` also install
/// the propagator themselves in test setup so these tests do not depend on
/// that production call site.
///
/// Called from [`tracing_middleware`], which attaches the resulting
/// `Context` as the parent of the `http_request` span.
pub(crate) fn extract_trace_context(headers: &HeaderMap) -> opentelemetry::Context {
    /// Local `opentelemetry::propagation::Extractor` adapter over
    /// `axum::http::HeaderMap` -- `opentelemetry-http`'s `HeaderExtractor` is
    /// not a dependency of this crate, so this tiny adapter avoids adding one
    /// solely for this.
    struct HeaderMapExtractor<'a>(&'a HeaderMap);

    impl opentelemetry::propagation::Extractor for HeaderMapExtractor<'_> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|value| value.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|key| key.as_str()).collect()
        }
    }

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderMapExtractor(headers))
    })
}

/// Request tracing middleware.
///
/// Generates a per-request correlation ID (`request_id`) and attaches it as
/// a `tracing` span field on the `http_request` span that wraps the entire
/// request lifecycle. Because it is a span field rather than a one-off event
/// field, every log line emitted anywhere in the call stack while this
/// request is being handled -- including deep inside handlers or downstream
/// crates -- automatically inherits `request_id`. The same value is also
/// returned to the caller as an `X-Request-Id` response header so client-
/// and server-side logs can be correlated.
///
/// # Trace-context propagation (Task #3 backend half)
///
/// If the request carries a valid inbound `traceparent` header, the
/// `http_request` span (and everything nested under it) becomes a CHILD of
/// that external trace via [`extract_trace_context`] -- see its doc comment
/// for the exact contract. Absent or malformed `traceparent` headers fall
/// back to today's behavior unchanged: a fresh root span. This is purely
/// additive and does not alter `request_id` generation/behavior in any way.
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let parent_cx = extract_trace_context(request.headers());

    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
    );
    // `set_parent` must run before the span is entered/instrumented --
    // `tracing-opentelemetry` rejects it on an already-started span. When
    // `parent_cx` carries no valid span context (no/malformed `traceparent`),
    // this is a harmless no-op and the span remains a fresh root, matching
    // today's pre-Task-3 behavior.
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    if let Err(err) = span.set_parent(parent_cx) {
        tracing::debug!(error = %err, "Failed to set inbound trace context as span parent");
    }

    async move {
        tracing::info!("Request started");

        let mut response = next.run(request).await;

        tracing::info!(status = %response.status(), "Request completed");

        if let Ok(header_value) = axum::http::HeaderValue::from_str(&request_id) {
            response.headers_mut().insert("x-request-id", header_value);
        } else {
            tracing::warn!("Failed to build X-Request-Id header from generated request_id");
        }

        response
    }
    .instrument(span)
    .await
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

    /// The token is valid but identifies no principal: it carries neither a
    /// usable login (`sub`) nor a principal id (`pid`).
    MissingPrincipalIdentifier,

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
            AuthError::MissingPrincipalIdentifier => (
                StatusCode::UNAUTHORIZED,
                "MissingPrincipal",
                "The token does not identify a principal; re-authenticate via POST /api/v1/auth/token."
                    .to_string(),
                None,
            ),
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

// `extract_trace_context`'s test-only support (shared by the sibling
// `middleware_tests.rs` / `middleware_proptests.rs` submodules below) lives
// in `middleware_test_support.rs` rather than inline here, to keep this
// production file under the project's `max_file_length` budget.
#[cfg(test)]
#[path = "middleware_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "middleware_principal_tests.rs"]
mod principal_tests;

#[cfg(test)]
#[path = "middleware_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "middleware_proptests.rs"]
mod proptests;
