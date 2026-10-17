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

// ─────────────────────────────────────────────────────────────────────────
// Inbound W3C traceparent extraction / propagation (Task #3, backend half)
// ─────────────────────────────────────────────────────────────────────────
//
// Contract under test, NOT YET IMPLEMENTED as of this writing (RED phase):
// `extract_trace_context` is `todo!()`-stubbed in `middleware.rs`, and
// `tracing_middleware` does not yet call it or react to a `traceparent`
// header in any way. These tests are expected to FAIL until the Coder:
//   1. implements `extract_trace_context` per its doc-comment contract, and
//   2. wires it into `tracing_middleware`, calling
//      `tracing_opentelemetry::OpenTelemetrySpanExt::set_parent` on the
//      `http_request` span BEFORE that span is entered/instrumented (a span
//      cannot have its parent set after being entered -- see
//      `tracing-opentelemetry`'s own `set_parent_fails_after_span_entered`
//      test).
//
// Unit-level tests below call `extract_trace_context` directly; integration
// -level tests exercise the full HTTP pipeline via an in-memory OTel
// exporter, reusing `tracing_otel::build_otel_layer_with_exporter` (Task
// #2's own testable seam) rather than reinventing span-capture
// infrastructure. See also `middleware_proptests.rs` for property-based
// coverage of `extract_trace_context` across generated header values.

use crate::tracing_otel::build_otel_layer_with_exporter;
use axum::http::HeaderValue;
use opentelemetry::trace::{SpanId, TraceContextExt, TraceId};
use opentelemetry_sdk::trace::InMemorySpanExporter;
use tracing_subscriber::layer::SubscriberExt;

// `ensure_trace_context_propagator_installed` and `headers_with_traceparent`
// are shared with the sibling `middleware_proptests.rs` module via
// `middleware_test_support.rs` -- see that module's doc comment for why.
use super::test_support::{ensure_trace_context_propagator_installed, headers_with_traceparent};

/// A W3C-valid `traceparent` header value (trace id / parent id taken
/// verbatim from the W3C trace-context spec's own worked example).
const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
const VALID_TRACEPARENT_TRACE_ID_HEX: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
const VALID_TRACEPARENT_PARENT_ID_HEX: &str = "00f067aa0ba902b7";

/// A second, DIFFERENT well-formed `traceparent`, for differentiation tests.
const VALID_TRACEPARENT_B: &str = "00-5c3a1e7b2f4d49a6b7c8d9e0f1a2b3c4-a1b2c3d4e5f60718-01";

/// Known-invalid `traceparent` values per the W3C trace-context spec
/// (mirrors `opentelemetry_sdk`'s own internal `extract_data_invalid`
/// conformance table), each paired with a human-readable reason for test
/// failure diagnostics.
fn known_invalid_traceparents() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "0000-00000000000000000000000000000000-0000000000000000-01",
            "wrong version length",
        ),
        (
            "00-ab00000000000000000000000000000000-cd00000000000000-01",
            "wrong trace ID length",
        ),
        (
            "00-ab000000000000000000000000000000-cd0000000000000000-01",
            "wrong span ID length",
        ),
        (
            "00-ab000000000000000000000000000000-cd00000000000000-0100",
            "wrong trace flag length",
        ),
        (
            "qw-00000000000000000000000000000000-0000000000000000-01",
            "bogus version",
        ),
        (
            "00-qw000000000000000000000000000000-cd00000000000000-01",
            "bogus trace ID",
        ),
        (
            "00-ab000000000000000000000000000000-qw00000000000000-01",
            "bogus span ID",
        ),
        (
            "00-ab000000000000000000000000000000-cd00000000000000-qw",
            "bogus trace flag",
        ),
        (
            "A0-00000000000000000000000000000000-0000000000000000-01",
            "upper case version",
        ),
        (
            "00-AB000000000000000000000000000000-cd00000000000000-01",
            "upper case trace ID",
        ),
        (
            "00-ab000000000000000000000000000000-CD00000000000000-01",
            "upper case span ID",
        ),
        (
            "00-ab000000000000000000000000000000-cd00000000000000-A1",
            "upper case trace flag",
        ),
        (
            "00-00000000000000000000000000000000-0000000000000000-01",
            "zero trace ID and span ID",
        ),
        (
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",
            "missing options",
        ),
        ("", "empty string"),
        ("not-a-traceparent-at-all", "complete garbage"),
    ]
}

// ---- Tier 1/2: unit tests on `extract_trace_context` directly ----------

/// Spec: a well-formed inbound `traceparent` decodes to a valid, remote
/// span context.
#[test]
fn test_extract_trace_context_with_valid_traceparent_yields_valid_remote_context() {
    ensure_trace_context_propagator_installed();
    let headers = headers_with_traceparent(VALID_TRACEPARENT);

    let cx = extract_trace_context(&headers);
    let span_ref = cx.span();
    let span_context = span_ref.span_context();

    assert!(
        span_context.is_valid(),
        "a well-formed traceparent header must decode to a valid span context"
    );
    assert!(
        span_context.is_remote(),
        "a span context decoded from an inbound header must be marked remote"
    );
}

/// Spec: the decoded trace id / parent id must match the header's fields
/// exactly, not merely "some" valid-looking id.
#[test]
fn test_extract_trace_context_with_valid_traceparent_preserves_trace_and_parent_id() {
    ensure_trace_context_propagator_installed();
    let headers = headers_with_traceparent(VALID_TRACEPARENT);

    let cx = extract_trace_context(&headers);
    let span_ref = cx.span();
    let span_context = span_ref.span_context();

    assert_eq!(
        span_context.trace_id(),
        TraceId::from_hex(VALID_TRACEPARENT_TRACE_ID_HEX).unwrap(),
        "decoded trace id must match the header's trace-id field exactly"
    );
    assert_eq!(
        span_context.span_id(),
        SpanId::from_hex(VALID_TRACEPARENT_PARENT_ID_HEX).unwrap(),
        "decoded span id must match the header's parent-id field exactly"
    );
}

/// Spec: absence of the header is the common case and must fall back to an
/// invalid (fresh-root) context, never panicking or fabricating one.
#[test]
fn test_extract_trace_context_without_traceparent_header_yields_invalid_context() {
    ensure_trace_context_propagator_installed();
    let headers = HeaderMap::new();

    let cx = extract_trace_context(&headers);

    assert!(
        !cx.span().span_context().is_valid(),
        "no traceparent header must decode to an invalid (fresh-root) context"
    );
}

/// Adversarial / fail-safe: every spec-known-invalid `traceparent` shape
/// must be rejected as invalid (never accepted, never panics).
#[test]
fn test_extract_trace_context_with_known_invalid_traceparent_values_yields_invalid_context() {
    ensure_trace_context_propagator_installed();

    for (value, reason) in known_invalid_traceparents() {
        let headers = headers_with_traceparent(value);
        let cx = extract_trace_context(&headers);
        assert!(
            !cx.span().span_context().is_valid(),
            "traceparent value '{}' ({}) must be rejected as invalid, not accepted",
            value,
            reason
        );
    }
}

/// Adversarial: pathologically long garbage must not panic.
#[test]
fn test_extract_trace_context_with_pathological_long_garbage_never_panics() {
    ensure_trace_context_propagator_installed();
    let garbage = "x".repeat(10_000);
    let headers = headers_with_traceparent(&garbage);
    let _ = extract_trace_context(&headers);
}

/// Adversarial: a header value that is not valid UTF-8/ASCII text (legal at
/// the raw `http` crate level, only rejected by `Extractor` adapters that
/// call `.to_str()`) must not panic -- it should be treated the same as an
/// absent header.
#[test]
fn test_extract_trace_context_with_non_utf8_header_bytes_never_panics() {
    ensure_trace_context_propagator_installed();
    let mut headers = HeaderMap::new();
    if let Ok(header_value) = HeaderValue::from_bytes(&[0xff, 0xfe, 0x00, 0x01]) {
        headers.insert("traceparent", header_value);
    }
    let _ = extract_trace_context(&headers);
}

/// Stub-killing: a hardcoded/cached return would pass the single-header
/// tests above but fail to distinguish between two different valid inputs.
#[test]
fn test_extract_trace_context_different_valid_headers_yield_different_trace_ids() {
    ensure_trace_context_propagator_installed();

    let cx_a = extract_trace_context(&headers_with_traceparent(VALID_TRACEPARENT));
    let cx_b = extract_trace_context(&headers_with_traceparent(VALID_TRACEPARENT_B));

    assert_ne!(
        cx_a.span().span_context().trace_id(),
        cx_b.span().span_context().trace_id(),
        "two distinct inbound traceparent headers must decode to two distinct trace ids -- \
         a hardcoded return would fail this"
    );
}

/// Stub-killing / state isolation: a call with no header must not inherit a
/// valid context left over from a previous call with a valid one.
#[test]
fn test_extract_trace_context_calls_are_independent_no_state_leak_between_calls() {
    ensure_trace_context_propagator_installed();

    let valid_cx = extract_trace_context(&headers_with_traceparent(VALID_TRACEPARENT));
    assert!(valid_cx.span().span_context().is_valid());

    let empty_cx = extract_trace_context(&HeaderMap::new());
    assert!(
        !empty_cx.span().span_context().is_valid(),
        "a call with no traceparent header must not inherit a valid context from a \
         previous call with a valid one -- rules out accidental shared/cached state"
    );
}

// ---- Tier 1/2: integration tests through the full `tracing_middleware` --
// -- HTTP pipeline, asserting on actually-exported OTel spans. ------------

/// Handler that creates a nested child span, simulating how
/// `repo_roller_core` / `github_client` create business-context and
/// GitHub-API spans well below the middleware layer.
async fn handler_with_nested_business_span() -> &'static str {
    async {
        tracing::info!("doing business logic");
    }
    .instrument(tracing::info_span!("business_operation"))
    .await;
    "OK"
}

/// Spec (core acceptance criterion): a request carrying a valid inbound
/// `traceparent` header produces an exported `http_request` span sharing
/// the inbound trace id, parented directly on the inbound parent-id.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_with_valid_traceparent_makes_http_request_span_child_of_inbound_trace(
) {
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let http_span = spans
        .iter()
        .find(|s| s.name.as_ref() == "http_request")
        .expect("an http_request span must be exported");

    assert_eq!(
        http_span.span_context.trace_id(),
        TraceId::from_hex(VALID_TRACEPARENT_TRACE_ID_HEX).unwrap(),
        "the http_request span must share the trace id from the inbound traceparent header"
    );
    assert_eq!(
        http_span.parent_span_id,
        SpanId::from_hex(VALID_TRACEPARENT_PARENT_ID_HEX).unwrap(),
        "the http_request span's parent must be the inbound traceparent's parent-id"
    );
}

/// Spec (no-regression): absent `traceparent` must behave exactly as today
/// -- the `http_request` span remains a fresh root span (no parent).
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_without_traceparent_header_behavior_is_unchanged_fresh_root_span()
{
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let http_span = spans
        .iter()
        .find(|s| s.name.as_ref() == "http_request")
        .expect("an http_request span must be exported");

    assert_eq!(
        http_span.parent_span_id,
        SpanId::INVALID,
        "with no inbound traceparent header, the http_request span must remain a fresh root \
         span (no parent) -- today's unchanged behavior"
    );
}

/// Stub-killing: two independent requests with no inbound trace context
/// must never share a trace id -- rules out a fixed/global root context.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_two_requests_without_traceparent_get_distinct_trace_ids() {
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let _ = app
            .clone()
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let _ = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let http_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.name.as_ref() == "http_request")
        .collect();
    assert_eq!(
        http_spans.len(),
        2,
        "expected exactly two http_request spans, got: {http_spans:?}"
    );
    assert_ne!(
        http_spans[0].span_context.trace_id(),
        http_spans[1].span_context.trace_id(),
        "two independent requests with no inbound trace context must never share a trace id"
    );
}

/// Fail-safe: a malformed `traceparent` must not crash the request, and
/// must fall back to the same fresh-root behavior as an absent header.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_with_malformed_traceparent_falls_back_to_fresh_root_without_panicking(
) {
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .header("traceparent", "garbage-not-a-traceparent")
        .body(Body::empty())
        .unwrap();

    let response = {
        let _guard = tracing::subscriber::set_default(subscriber);
        app.oneshot(request).await.unwrap()
    };

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "a malformed traceparent header must not crash/500 the request"
    );

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let http_span = spans
        .iter()
        .find(|s| s.name.as_ref() == "http_request")
        .expect("an http_request span must be exported");
    assert_eq!(
        http_span.parent_span_id,
        SpanId::INVALID,
        "a malformed traceparent header must fall back to a fresh root span, matching the \
         absent-header case"
    );
}

/// Fail-safe, exhaustive: every spec-known-invalid `traceparent` shape must
/// be handled gracefully through the full HTTP pipeline, not just at the
/// unit level.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_with_various_malformed_traceparent_values_never_panics_and_falls_back_to_root(
) {
    ensure_trace_context_propagator_installed();

    for (value, reason) in known_invalid_traceparents() {
        let exporter = InMemorySpanExporter::default();
        let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
        let subscriber = tracing_subscriber::registry().with(layer);

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(tracing_middleware));

        let mut builder = Request::builder().uri("/test");
        if let Ok(header_value) = HeaderValue::from_str(value) {
            builder = builder.header("traceparent", header_value);
        }
        let request = builder.body(Body::empty()).unwrap();

        let response = {
            let _guard = tracing::subscriber::set_default(subscriber);
            app.oneshot(request).await.unwrap()
        };

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "malformed traceparent '{}' ({}) must not crash the request",
            value,
            reason
        );

        let spans = exporter
            .get_finished_spans()
            .expect("in-memory exporter must report finished spans");
        let http_span = spans
            .iter()
            .find(|s| s.name.as_ref() == "http_request")
            .expect("an http_request span must be exported");
        assert_eq!(
            http_span.parent_span_id,
            SpanId::INVALID,
            "malformed traceparent '{}' ({}) must fall back to a fresh root span",
            value,
            reason
        );
    }
}

/// Spec (acceptance criterion: "everything nested under it"): a business-
/// context child span created deep in a handler must inherit the SAME
/// trace id as the inbound `traceparent`, not just the immediate
/// `http_request` span.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_nested_child_spans_inherit_inbound_trace_id() {
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(handler_with_nested_business_span))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let _ = app.oneshot(request).await.unwrap();
    }

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let business_span = spans
        .iter()
        .find(|s| s.name.as_ref() == "business_operation")
        .expect("business_operation span must be exported");

    assert_eq!(
        business_span.span_context.trace_id(),
        TraceId::from_hex(VALID_TRACEPARENT_TRACE_ID_HEX).unwrap(),
        "a span nested under http_request must inherit the inbound trace id, not just the \
         http_request span itself"
    );
}

/// Stub-killing: two requests carrying the SAME inbound trace id must
/// produce exported spans sharing that same trace id -- proves genuine
/// extraction rather than an occasional coincidental match against a
/// randomly-generated fresh root.
#[tokio::test(flavor = "current_thread")]
async fn test_tracing_middleware_same_inbound_trace_id_across_two_requests_yields_same_exported_trace_id(
) {
    ensure_trace_context_propagator_installed();

    let exporter = InMemorySpanExporter::default();
    let layer = build_otel_layer_with_exporter(exporter.clone(), "test-service");
    let subscriber = tracing_subscriber::registry().with(layer);

    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let req1 = Request::builder()
            .uri("/test")
            .header("traceparent", VALID_TRACEPARENT)
            .body(Body::empty())
            .unwrap();
        let req2 = Request::builder()
            .uri("/test")
            .header("traceparent", VALID_TRACEPARENT)
            .body(Body::empty())
            .unwrap();
        let _ = app.clone().oneshot(req1).await.unwrap();
        let _ = app.oneshot(req2).await.unwrap();
    }

    let spans = exporter
        .get_finished_spans()
        .expect("in-memory exporter must report finished spans");
    let http_spans: Vec<_> = spans
        .iter()
        .filter(|s| s.name.as_ref() == "http_request")
        .collect();
    assert_eq!(
        http_spans.len(),
        2,
        "expected exactly two http_request spans"
    );
    assert_eq!(
        http_spans[0].span_context.trace_id(),
        http_spans[1].span_context.trace_id(),
        "two requests carrying the SAME inbound trace id must produce exported spans sharing \
         that same trace id"
    );
    assert_eq!(
        http_spans[0].span_context.trace_id(),
        TraceId::from_hex(VALID_TRACEPARENT_TRACE_ID_HEX).unwrap()
    );
}

// ---- Orthogonality: Task 1's request_id must be completely unaffected ---

/// `request_id` (Task 1) is an orthogonal correlation mechanism -- it must
/// keep working exactly as before even when a `traceparent` header is also
/// present.
#[tokio::test]
async fn test_request_id_header_still_present_and_valid_when_traceparent_header_supplied() {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let header_value = request_id_header(&response);
    assert!(
        uuid::Uuid::parse_str(&header_value).is_ok(),
        "X-Request-Id must remain a valid UUID even when an inbound traceparent header is \
         present"
    );
}

/// Adversarial: `request_id` must stay independent of the inbound trace id
/// -- two requests sharing the SAME `traceparent` must still get two
/// DIFFERENT `request_id` values.
#[tokio::test]
async fn test_request_id_still_distinct_per_request_even_when_both_requests_share_same_inbound_trace_id(
) {
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let req1 = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();
    let req2 = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();

    let resp1 = app.clone().oneshot(req1).await.unwrap();
    let resp2 = app.oneshot(req2).await.unwrap();

    let id1 = request_id_header(&resp1);
    let id2 = request_id_header(&resp2);
    assert_ne!(
        id1, id2,
        "request_id (Task 1's per-request correlation id) must stay independent of the \
         inbound trace id -- two requests sharing the SAME traceparent must still get \
         DIFFERENT request_id values"
    );
}

/// Regression guard: the full request_id-per-log-line consistency
/// guarantee (Task 1) must keep holding when a `traceparent` header is also
/// present -- guards against an implementation that, while reparenting the
/// span, accidentally replaces it instead of just reparenting it (which
/// would break every other span-inherited field, not just the parent link).
#[tokio::test]
async fn test_request_id_log_consistency_still_holds_when_traceparent_header_present() {
    let buffer = CaptureBuffer::default();
    let subscriber = build_json_subscriber(buffer.clone());

    let app = Router::new()
        .route("/test", get(nested_logging_handler))
        .layer(middleware::from_fn(tracing_middleware));

    let request = Request::builder()
        .uri("/test")
        .header("traceparent", VALID_TRACEPARENT)
        .body(Body::empty())
        .unwrap();

    let response = {
        let _guard = tracing::subscriber::set_default(subscriber);
        app.oneshot(request).await.unwrap()
    };

    let header_request_id = request_id_header(&response);
    let lines = parse_json_lines(&buffer.contents());
    assert!(
        lines.len() >= 3,
        "expected at least 3 log lines; got {}: {:?}",
        lines.len(),
        lines
    );

    for line in &lines {
        let found = find_field_recursive(line, "request_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("log line is missing a request_id field: {line}"));
        assert_eq!(
            found, header_request_id,
            "request_id consistency (Task 1) must hold unchanged even when an inbound \
             traceparent header is present; offending line: {line}"
        );
    }
}
