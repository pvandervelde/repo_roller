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
    let token =
        generate_backend_jwt("alice", crate::TEST_JWT_SECRET).expect("JWT generation must succeed");

    let state = AppState::default(); // uses crate::TEST_JWT_SECRET
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
    let token = generate_backend_jwt("bob", crate::TEST_JWT_SECRET).expect("should succeed");
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

    let secret = crate::TEST_JWT_SECRET;
    // Set exp to a fixed timestamp in the distant past — well beyond any leeway.
    // 1_000_000 seconds after the Unix epoch = 1970-01-12 13:46:40 UTC.
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
    // generate_backend_jwt accepts any &str including "".
    let token =
        generate_backend_jwt("", crate::TEST_JWT_SECRET).expect("JWT generation must succeed");

    let state = AppState::default(); // uses crate::TEST_JWT_SECRET
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

    let secret = crate::TEST_JWT_SECRET;
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
        // The +5 on the upper bound absorbs any extra jitter beyond the
        // before/after window (e.g. scheduler latency between the two
        // SystemTime::now() calls). The lower bound needs no slack because
        // `before` is measured before token generation.
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

// ── Correlation ID / request span tests (Task #1) ──────────────────────────
//
// Contract under test (task acceptance criteria, not yet implemented in
// `tracing_middleware` as of this writing -- these tests are expected to
// FAIL until the Coder wraps request handling in a
// `tracing::info_span!("http_request", request_id = %request_id, ...)` and
// adds the same value as an `X-Request-Id` response header):
//
//   1. Every response carries an `X-Request-Id` header whose value is a
//      valid UUID.
//   2. Two different requests receive two different `request_id` values
//      (no hardcoded / reused ID).
//   3. Every log line emitted while handling ONE request -- including ones
//      emitted deep in a handler's call stack, not just the two explicit
//      `tracing::info!` calls inside `tracing_middleware` itself -- carries
//      the SAME `request_id`, and that value matches the `X-Request-Id`
//      response header exactly.
//   4. An event logged with no `http_request` span active must never
//      acquire a `request_id` field (rules out a global/static ID).
//   5. Concurrent requests must never cross-contaminate each other's
//      `request_id` in their respective log trails.
//
// These tests build their own minimal JSON-capturing subscriber rather than
// reusing `crate::logging`'s (separate, also-not-yet-implemented) sanitizing
// subscriber, so that a failure here unambiguously points at
// `tracing_middleware`, not at unrelated `logging` module gaps.

use std::sync::{Arc, Mutex};

/// In-memory writer used only by these tests to capture JSON tracing output.
#[derive(Clone, Default)]
struct CaptureBuffer(Arc<Mutex<Vec<u8>>>);

impl CaptureBuffer {
    fn contents(&self) -> String {
        String::from_utf8_lossy(
            &self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
        .to_string()
    }
}

impl std::io::Write for CaptureBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureBuffer {
    type Writer = CaptureBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Builds a plain JSON tracing subscriber writing to `buffer`. Deliberately
/// independent of `crate::logging::build_sanitizing_json_subscriber`.
fn build_json_subscriber(buffer: CaptureBuffer) -> impl tracing::Subscriber + Send + Sync {
    tracing_subscriber::fmt()
        .json()
        .with_writer(buffer)
        .with_ansi(false)
        .finish()
}

/// Parses newline-delimited JSON output into a `Vec<serde_json::Value>`,
/// skipping blank lines. Panics (failing the calling test with a clear
/// message) if any non-blank line is not valid JSON.
fn parse_json_lines(contents: &str) -> Vec<serde_json::Value> {
    contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("captured log line was not valid JSON: {l:?}: {e}"))
        })
        .collect()
}

/// Recursively searches a parsed JSON value tree for the first occurrence of
/// object key `key`, at any depth. Used so these tests don't couple
/// themselves to exactly where `tracing_subscriber`'s JSON formatter places
/// span-inherited fields (`"span"`, `"spans"`, or otherwise).
fn find_field_recursive<'v>(
    value: &'v serde_json::Value,
    key: &str,
) -> Option<&'v serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(v) = map.get(key) {
                return Some(v);
            }
            map.values().find_map(|v| find_field_recursive(v, key))
        }
        serde_json::Value::Array(arr) => arr.iter().find_map(|v| find_field_recursive(v, key)),
        _ => None,
    }
}

/// Extracts the `X-Request-Id` response header as a `String`, panicking
/// with a descriptive message if it is absent.
fn request_id_header(response: &axum::response::Response) -> String {
    response
        .headers()
        .get("x-request-id")
        .unwrap_or_else(|| panic!("response must carry an X-Request-Id header"))
        .to_str()
        .expect("X-Request-Id header value must be valid ASCII/UTF-8")
        .to_string()
}

/// Handler used to simulate logging that happens deep in a real request's
/// call stack (mirroring how `repo_roller_core` / `github_client` log from
/// well below the middleware layer), not just inside `tracing_middleware`
/// itself.
async fn nested_logging_handler() -> &'static str {
    tracing::info!("handler processing started");
    deeply_nested_log_call().await;
    "OK"
}

async fn deeply_nested_log_call() {
    tracing::warn!("simulated deep-stack log (e.g. repo_roller_core / github_client)");
}

/// Test 1 (spec): the response must carry a well-formed `X-Request-Id`.
#[tokio::test]
async fn test_tracing_middleware_response_carries_valid_uuid_x_request_id_header() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    let header_value = request_id_header(&response);
    assert!(
        uuid::Uuid::parse_str(&header_value).is_ok(),
        "X-Request-Id header value '{}' must be a valid UUID",
        header_value
    );
}

/// Test 2 (spec / stub-killing): a hardcoded or reused request ID would
/// pass test 1 but fail here.
#[tokio::test]
async fn test_tracing_middleware_generates_distinct_request_id_per_request() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let first = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let second = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let first_id = request_id_header(&first);
    let second_id = request_id_header(&second);
    assert_ne!(
        first_id, second_id,
        "two separate requests must never share the same request_id"
    );
}

/// Test 3 (spec, core span-inheritance contract): every log line emitted
/// while handling one request -- including from a handler and a function it
/// calls -- must carry the SAME `request_id`, matching the response header.
#[tokio::test]
async fn test_request_id_is_consistent_across_every_log_line_for_one_request() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_json_subscriber(buffer.clone());

    let app = Router::new()
        .route("/test", get(nested_logging_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = {
        let _guard = tracing::subscriber::set_default(subscriber);
        app.oneshot(request).await.unwrap()
    };

    let header_request_id = request_id_header(&response);

    let lines = parse_json_lines(&buffer.contents());
    assert!(
        lines.len() >= 3,
        "expected at least 3 log lines (request started, handler log(s), request completed); got {}: {:?}",
        lines.len(),
        lines
    );

    for line in &lines {
        let found = find_field_recursive(line, "request_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("log line is missing a request_id field entirely: {line}"));
        assert_eq!(
            found, header_request_id,
            "every log line emitted while handling this request must carry the SAME \
             request_id as the X-Request-Id response header; offending line: {line}"
        );
    }
}

/// Test 4 (side-effect absence / stub-killing): an event logged with no
/// active `http_request` span must never acquire a `request_id` field --
/// rules out a global/static request ID being force-injected everywhere.
#[tokio::test]
async fn test_events_outside_any_http_request_span_carry_no_request_id_field() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_json_subscriber(buffer.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::info!("event emitted with no active http_request span");
    }

    let lines = parse_json_lines(&buffer.contents());
    assert_eq!(lines.len(), 1, "expected exactly one captured log line");
    assert!(
        find_field_recursive(&lines[0], "request_id").is_none(),
        "an event logged outside any HTTP request must not carry a request_id field; found: {}",
        lines[0]
    );
}

/// Test 5 (adversarial / concurrency isolation): concurrent requests must
/// never cross-contaminate each other's `request_id` in their log trails --
/// guards against a shared mutable/global ID instead of a genuine per-request
/// span field.
#[tokio::test]
async fn test_concurrent_requests_never_share_request_id_across_their_log_trails() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_json_subscriber(buffer.clone());

    let app = Router::new()
        .route("/test", get(nested_logging_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let _guard = tracing::subscriber::set_default(subscriber);

    const CONCURRENT_REQUESTS: usize = 5;
    let mut handles = Vec::with_capacity(CONCURRENT_REQUESTS);
    for _ in 0..CONCURRENT_REQUESTS {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
            let response = app.oneshot(request).await.unwrap();
            request_id_header(&response)
        }));
    }

    let mut ids = Vec::with_capacity(CONCURRENT_REQUESTS);
    for handle in handles {
        ids.push(handle.await.expect("spawned request task must not panic"));
    }

    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(
        unique_ids.len(),
        ids.len(),
        "each concurrent request must receive its own distinct request_id; got {:?}",
        ids
    );

    let lines = parse_json_lines(&buffer.contents());
    for line in &lines {
        if let Some(found) = find_field_recursive(line, "request_id").and_then(|v| v.as_str()) {
            assert!(
                ids.contains(&found.to_string()),
                "log line request_id '{}' does not match any of the {} issued request IDs {:?} \
                 -- possible cross-request leak",
                found,
                CONCURRENT_REQUESTS,
                ids
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// QA audit: kill test for a `cargo mutants` survivor in
// `organization_auth_middleware`.
// ─────────────────────────────────────────────────────────────────────────

/// ### Survivor killed
/// `middleware.rs`: `organization_auth_middleware`'s body replaced with
/// `Ok(Default::default())`, discarding the call to `next.run(request)`
/// entirely. No existing test exercised this middleware, so a mutant that
/// short-circuits the request pipeline and returns an empty default
/// response (status 200, empty body) instead of forwarding to the next
/// handler survived undetected -- despite this middleware being documented
/// as a currently-passthrough authorization layer that the request MUST
/// still reach the downstream handler through. This test asserts the
/// downstream handler's distinctive response (a non-default status code and
/// a non-empty body) is what actually comes back, which a `Default::default()`
/// stub cannot produce.
#[tokio::test]
async fn test_organization_auth_middleware_forwards_request_to_next_handler() {
    async fn distinctive_handler() -> (StatusCode, &'static str) {
        (
            StatusCode::IM_A_TEAPOT,
            "distinctive-body-from-downstream-handler",
        )
    }

    let app = Router::new()
        .route("/test", get(distinctive_handler))
        .layer(middleware::from_fn(organization_auth_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::IM_A_TEAPOT,
        "organization_auth_middleware must forward the request to the next handler \
         and return its actual response, not a default-constructed one"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("reading response body must succeed");
    assert_eq!(
        body_bytes.as_ref(),
        b"distinctive-body-from-downstream-handler",
        "the downstream handler's body must survive unchanged through the middleware"
    );
}
