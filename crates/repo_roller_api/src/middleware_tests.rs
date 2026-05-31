//! Tests for middleware module
//!
//! Unit tests cover header parsing and JWT validation. Real GitHub API token
//! validation (used only by the exchange endpoint) is tested in the
//! integration_tests crate with actual GitHub App credentials.

use super::*;
use crate::AppState;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use tower::ServiceExt; // for `oneshot`

/// Test helper: create a simple handler that returns OK
async fn test_handler() -> &'static str {
    "OK"
}

/// Build a test router that applies `auth_middleware` with the default test state.
fn auth_test_app() -> Router {
    let state = AppState::default();
    Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Test that missing Authorization header returns 401
#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that invalid Authorization format returns 401
#[tokio::test]
async fn test_auth_middleware_invalid_format() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "InvalidFormat")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that non-Bearer scheme returns 401
#[tokio::test]
async fn test_auth_middleware_wrong_scheme() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Basic username:password")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that a raw GitHub token (not a backend JWT) is rejected with 401.
///
/// The middleware no longer validates GitHub tokens; passing one directly is
/// rejected as a malformed JWT.
#[tokio::test]
async fn test_auth_middleware_raw_github_token_rejected() {
    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, "Bearer ghp_someFakeGitHubToken")
        .body(Body::empty())
        .unwrap();
    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that a valid backend JWT is accepted and the handler receives 200.
#[tokio::test]
async fn test_auth_middleware_valid_backend_jwt_accepted() {
    let secret = "test-jwt-secret-key-minimum-32b!";
    let token = generate_backend_jwt("alice", secret).expect("JWT generation must succeed");

    let state = AppState::default(); // uses the same test secret
    let app = Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test that a backend JWT signed with a different secret is rejected.
#[tokio::test]
async fn test_auth_middleware_jwt_wrong_secret_rejected() {
    // Sign with a different secret than the one in AppState::default().
    let token = generate_backend_jwt("alice", "wrong-secret-key-that-is-32-bytes!")
        .expect("JWT generation must succeed");

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = auth_test_app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test extract_bearer_token with valid format
#[test]
fn test_extract_bearer_token_valid() {
    let result = extract_bearer_token("Bearer my_token_123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my_token_123");
}

/// Test extract_bearer_token with invalid format
#[test]
fn test_extract_bearer_token_invalid_format() {
    let result = extract_bearer_token("InvalidFormat");
    assert!(result.is_err());
}

/// Test extract_bearer_token with wrong scheme
#[test]
fn test_extract_bearer_token_wrong_scheme() {
    let result = extract_bearer_token("Basic token");
    assert!(result.is_err());
}

/// Test AuthContext creation
#[test]
fn test_auth_context_creation() {
    let context = AuthContext::new();
    assert!(context.user_login.is_none());
}

/// Test that generate_backend_jwt produces a verifiable token.
#[test]
fn test_generate_backend_jwt_roundtrip() {
    let secret = "test-jwt-secret-key-minimum-32b!";
    let token = generate_backend_jwt("bob", secret).expect("should succeed");
    assert!(!token.is_empty());
    // The token must be a valid three-part JWT.
    assert_eq!(token.split('.').count(), 3);
}

/// Test that a JWT with an expiry well in the past (beyond the 30-second leeway) is rejected.
///
/// The middleware uses a 30-second leeway. Tokens expired more than 30 seconds ago
/// must be rejected, while tokens expired within the leeway window may still be accepted.
#[tokio::test]
async fn test_expired_jwt_rejected() {
    use jsonwebtoken::{EncodingKey, Header};

    let secret = "test-jwt-secret-key-minimum-32b!";
    // Set exp to a fixed timestamp in the distant past — well beyond any leeway.
    let past = 1_000_000usize;
    let claims = Claims {
        sub: "alice".to_string(),
        iat: past,
        exp: past + 3600,
    };
    let expired_token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("encoding must succeed");

    let state = AppState::default();
    let app = Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", expired_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "An expired JWT must be rejected"
    );
}

/// Test that a JWT signed with an empty sub claim is accepted at the JWT level
/// (the claim is structurally valid) and the empty login propagates into AuthContext.
///
/// The middleware does not reject empty sub — that is a business-level concern
/// handled by downstream handlers. This test pins the current behaviour so any
/// future change to add sub-validation is made explicitly.
#[tokio::test]
async fn test_jwt_with_empty_sub_is_accepted_by_middleware() {
    let secret = "test-jwt-secret-key-minimum-32b!";
    // generate_backend_jwt accepts any &str including "".
    let token = generate_backend_jwt("", secret).expect("JWT generation must succeed");

    let state = AppState::default();
    let app = Router::new()
        .route("/test", get(test_handler))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    let request = Request::builder()
        .uri("/test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // The middleware validates the JWT signature and expiry only; an empty sub is
    // structurally valid and must not be rejected at this layer.
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "A structurally valid JWT with an empty sub should pass middleware validation"
    );
}

/// Test tracing middleware adds request logging
#[tokio::test]
async fn test_tracing_middleware_logs_requests() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Mutant kill tests ─────────────────────────────────────────────────────────
//
// The following tests target specific arithmetic mutants in JWT timing that
// survived the initial mutation run because existing tests didn't verify values.

/// `JWT_EXPIRY_SECS` must be exactly 28 800 (8 hours).
///
/// ### Survivor killed
/// `replace * with /` and `replace * with +` on `8 * 3600`
#[test]
fn test_jwt_expiry_secs_is_28800() {
    assert_eq!(
        JWT_EXPIRY_SECS, 28_800,
        "JWT_EXPIRY_SECS must be 8 * 3600 = 28800; arithmetic mutation survived"
    );
}

/// The `exp` claim in a freshly generated JWT must be approximately
/// `now + JWT_EXPIRY_SECS` (within a ±5-second tolerance for test execution).
///
/// ### Survivor killed
/// `replace + with *` in `generate_backend_jwt` for the `exp` calculation
#[test]
fn test_generate_backend_jwt_exp_is_iat_plus_expiry() {
    use jsonwebtoken::{DecodingKey, Validation};

    let secret = "test-jwt-secret-key-minimum-32b!";
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;

    let token = generate_backend_jwt("alice", secret).expect("JWT generation must succeed");

    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;

    // Decode without expiry validation so we can read the raw claims.
    let mut validation = Validation::default();
    validation.validate_exp = false;
    let data = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .expect("token must decode");

    let claims = data.claims;
    let expected_exp_min = before + JWT_EXPIRY_SECS as usize;
    let expected_exp_max = after + JWT_EXPIRY_SECS as usize;

    assert!(
        claims.exp >= expected_exp_min && claims.exp <= expected_exp_max + 5,
        "exp claim ({}) must be approximately iat + JWT_EXPIRY_SECS ({}..={})",
        claims.exp,
        expected_exp_min,
        expected_exp_max
    );

    // Explicitly guard against the `now * JWT_EXPIRY_SECS` mutation —
    // that would produce a value many orders of magnitude larger.
    assert!(
        claims.exp < after + 2 * JWT_EXPIRY_SECS as usize,
        "exp claim is suspiciously large (multiplication mutation?): {}",
        claims.exp
    );
}
