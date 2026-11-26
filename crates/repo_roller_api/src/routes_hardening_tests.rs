//! Router-level hardening tests:
//!
//! (a) Request tracing (`TraceLayer`) must never write sensitive request or
//!     response header values (bearer tokens, cookies) to the logs, even at
//!     TRACE verbosity and even though the log subscriber used here performs
//!     NO redaction of its own.
//! (b) The `rate_limit_tracked_keys{scope}` gauge is exported on `/metrics`
//!     for both limiter scopes, carries only the closed `scope` label, and
//!     tracks the number of keys held by each limiter.
//! (c) Default limiter wiring: the per-user limiter evicts the oldest key
//!     when full, the token-endpoint (client IP) limiter fails closed.

use super::*;
use crate::middleware::generate_backend_jwt;
use crate::rate_limit::{FullPolicy, RateLimitConfig, DEFAULT_MAX_TRACKED_KEYS};
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

const BEARER_SECRET: &str = "DISTINCTIVE-TOKEN-9a8b7c6d5e";
const COOKIE_SECRET: &str = "COOKIE-SECRET-VALUE-77aa88bb";
const PROXY_SECRET: &str = "PROXY-CRED-VALUE-31c2d4e5";

// ---------------------------------------------------------------------------
// Log capture (deliberately NOT the sanitising subscriber)
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Capture {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

/// Install a thread-local, unredacted TRACE-level subscriber for the
/// duration of the returned guard. Tests using it run on the current-thread
/// tokio runtime so every span/event of the request lands on this thread.
fn capture_all_logs() -> (Capture, tracing::subscriber::DefaultGuard) {
    let cap = Capture::default();
    let writer = cap.clone();
    let sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let guard = tracing::subscriber::set_default(sub);
    (cap, guard)
}

fn assert_no_secrets(logs: &str) {
    for (name, secret) in [
        ("bearer token", BEARER_SECRET),
        ("cookie value", COOKIE_SECRET),
        ("proxy credential", PROXY_SECRET),
    ] {
        assert!(!logs.contains(secret), "{name} leaked into logs:\n{logs}");
    }
}

fn sensitive_request(method: Method, uri: &str) -> Request<Body> {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {BEARER_SECRET}"))
        .header(header::COOKIE, format!("session={COOKIE_SECRET}"))
        .header(header::PROXY_AUTHORIZATION, format!("Basic {PROXY_SECRET}"))
        .body(Body::empty())
        .unwrap();
    let sa: SocketAddr = "203.0.113.50:40000".parse().unwrap();
    req.extensions_mut().insert(ConnectInfo(sa));
    req
}

// ---------------------------------------------------------------------------
// (a) Header logging
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_router_does_not_log_bearer_or_cookie_on_protected_route() {
    let (cap, _g) = capture_all_logs();
    let router = create_router(AppState::default());
    let resp = router
        .oneshot(sensitive_request(
            Method::GET,
            "/api/v1/orgs/acme/templates",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "bogus token");
    let logs = cap.text();
    assert!(
        logs.contains("/api/v1/orgs/acme/templates"),
        "capture must be live (request span expected), got:\n{logs}"
    );
    assert_no_secrets(&logs);
}

#[tokio::test]
async fn create_router_does_not_log_secrets_on_post_auth_token() {
    let (cap, _g) = capture_all_logs();
    let router = create_router(AppState::default());
    let _ = router
        .oneshot(sensitive_request(Method::POST, "/api/v1/auth/token"))
        .await
        .unwrap();
    let logs = cap.text();
    assert!(
        logs.contains("/api/v1/auth/token"),
        "capture must be live, got:\n{logs}"
    );
    assert_no_secrets(&logs);
}

#[tokio::test]
async fn create_router_does_not_log_secrets_on_public_health_route() {
    let (cap, _g) = capture_all_logs();
    let router = create_router(AppState::default());
    let resp = router
        .oneshot(sensitive_request(Method::GET, "/api/v1/health"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let logs = cap.text();
    assert!(logs.contains("/api/v1/health"), "capture live:\n{logs}");
    assert_no_secrets(&logs);
}

#[tokio::test]
async fn create_router_does_not_log_secrets_when_rate_limited() {
    let (cap, _g) = capture_all_logs();
    let router = create_router(
        AppState::default().with_auth_rate_limit_config(RateLimitConfig::new(true, 1, 60)),
    );
    let mut statuses = Vec::new();
    for _ in 0..3 {
        statuses.push(
            router
                .clone()
                .oneshot(sensitive_request(Method::POST, "/api/v1/auth/token"))
                .await
                .unwrap()
                .status(),
        );
    }
    assert!(
        statuses.contains(&StatusCode::TOO_MANY_REQUESTS),
        "{statuses:?}"
    );
    assert_no_secrets(&cap.text());
}

#[tokio::test]
async fn create_router_does_not_log_secrets_for_authenticated_post() {
    let (cap, _g) = capture_all_logs();
    let jwt = generate_backend_jwt("alice", crate::TEST_JWT_SECRET).unwrap();
    let router = create_router(AppState::default());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/repositories")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
        .header(header::COOKIE, format!("session={COOKIE_SECRET}"))
        .body(Body::from("{}"))
        .unwrap();
    let _ = router.oneshot(req).await.unwrap();
    let logs = cap.text();
    assert!(
        logs.contains("/api/v1/repositories"),
        "capture live:\n{logs}"
    );
    assert!(
        !logs.contains(&jwt),
        "a valid backend JWT leaked into logs:\n{logs}"
    );
    assert_no_secrets(&logs);
}

#[tokio::test]
async fn test_router_without_auth_does_not_log_bearer_or_cookie() {
    let (cap, _g) = capture_all_logs();
    let router = create_router_without_auth(AppState::default());
    let _ = router
        .oneshot(sensitive_request(Method::GET, "/api/v1/health"))
        .await
        .unwrap();
    let logs = cap.text();
    assert!(logs.contains("/api/v1/health"), "capture live:\n{logs}");
    assert_no_secrets(&logs);
}

// ---------------------------------------------------------------------------
// (c) Default limiter wiring
// ---------------------------------------------------------------------------

#[test]
fn default_state_user_limiter_evicts_oldest_and_auth_limiter_fails_closed() {
    let s = AppState::default();
    assert_eq!(s.rate_limiter.full_policy(), FullPolicy::EvictOldest);
    assert_eq!(s.auth_rate_limiter.full_policy(), FullPolicy::FailClosed);
    assert_eq!(s.rate_limiter.max_keys(), DEFAULT_MAX_TRACKED_KEYS);
    assert_eq!(s.auth_rate_limiter.max_keys(), DEFAULT_MAX_TRACKED_KEYS);
}

#[test]
fn config_derived_limiters_keep_their_policies() {
    let s = AppState::default()
        .with_rate_limit_config(RateLimitConfig::new(true, 5, 60))
        .with_auth_rate_limit_config(RateLimitConfig::new(true, 5, 60));
    assert_eq!(s.rate_limiter.full_policy(), FullPolicy::EvictOldest);
    assert_eq!(s.auth_rate_limiter.full_policy(), FullPolicy::FailClosed);
}

// ---------------------------------------------------------------------------
// (b) rate_limit_tracked_keys gauge
// ---------------------------------------------------------------------------

async fn metrics_body(router: &Router) -> String {
    let resp = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn gauge_lines(body: &str) -> Vec<String> {
    body.lines()
        .filter(|l| l.starts_with("rate_limit_tracked_keys") && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn gauge_value(body: &str, scope: &str) -> f64 {
    let needle = format!("scope=\"{scope}\"");
    let line = gauge_lines(body)
        .into_iter()
        .find(|l| l.contains(&needle))
        .unwrap_or_else(|| panic!("no rate_limit_tracked_keys series for {scope}:\n{body}"));
    line.rsplit(' ').next().unwrap().parse().unwrap()
}

fn token_post(addr: &str) -> Request<Body> {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/token")
        .body(Body::empty())
        .unwrap();
    let sa: SocketAddr = addr.parse().unwrap();
    req.extensions_mut().insert(ConnectInfo(sa));
    req
}

fn repo_post(login: &str) -> Request<Body> {
    let jwt = generate_backend_jwt(login, crate::TEST_JWT_SECRET).unwrap();
    Request::builder()
        .method(Method::POST)
        .uri("/api/v1/repositories")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
        .body(Body::from("{}"))
        .unwrap()
}

#[tokio::test]
async fn tracked_keys_gauge_is_exported_for_both_scopes_at_startup() {
    let router = create_router(AppState::default());
    let body = metrics_body(&router).await;
    assert!(
        body.contains("# TYPE rate_limit_tracked_keys gauge"),
        "gauge missing:\n{body}"
    );
    assert_eq!(gauge_value(&body, "user"), 0.0);
    assert_eq!(gauge_value(&body, "auth_ip"), 0.0);
}

#[tokio::test]
async fn auth_ip_gauge_counts_distinct_client_addresses() {
    let router = create_router(AppState::default());
    for a in [
        "203.0.113.1:1",
        "203.0.113.2:1",
        "203.0.113.3:1",
        "203.0.113.3:2",
    ] {
        let _ = router.clone().oneshot(token_post(a)).await.unwrap();
    }
    let body = metrics_body(&router).await;
    assert_eq!(gauge_value(&body, "auth_ip"), 3.0, "{body}");
    assert_eq!(gauge_value(&body, "user"), 0.0, "{body}");
}

#[tokio::test]
async fn user_gauge_counts_distinct_principals() {
    let router = create_router(AppState::default());
    for u in ["alice", "bob", "alice"] {
        let _ = router.clone().oneshot(repo_post(u)).await.unwrap();
    }
    let body = metrics_body(&router).await;
    assert_eq!(gauge_value(&body, "user"), 2.0, "{body}");
    assert_eq!(gauge_value(&body, "auth_ip"), 0.0, "{body}");
}

#[tokio::test]
async fn auth_ip_gauge_never_exceeds_the_configured_cap() {
    let clock = crate::rate_limit::tests::ManualClock::new();
    let limiter = Arc::new(crate::rate_limit::RateLimiter::with_max_keys(
        RateLimitConfig::new(true, 5, 60),
        clock,
        3,
        FullPolicy::FailClosed,
    ));
    let router = create_router(AppState::default().with_auth_rate_limiter(limiter));
    for i in 1..=10 {
        let _ = router
            .clone()
            .oneshot(token_post(&format!("203.0.113.{i}:1")))
            .await
            .unwrap();
    }
    let body = metrics_body(&router).await;
    assert_eq!(gauge_value(&body, "auth_ip"), 3.0, "{body}");
}

#[tokio::test]
async fn tracked_keys_gauge_has_only_the_scope_label_and_no_identities() {
    let router = create_router(AppState::default());
    let _ = router
        .clone()
        .oneshot(token_post("203.0.113.77:1"))
        .await
        .unwrap();
    let _ = router.clone().oneshot(repo_post("carol")).await.unwrap();
    let body = metrics_body(&router).await;
    let lines = gauge_lines(&body);
    assert_eq!(lines.len(), 2, "exactly the two scopes expected: {lines:?}");
    for l in &lines {
        assert!(
            l.contains("scope=\"user\"") || l.contains("scope=\"auth_ip\""),
            "{l}"
        );
        assert!(!l.contains("203.0.113"), "client address leaked: {l}");
        assert!(!l.contains("carol"), "user identity leaked: {l}");
        assert!(!l.contains("login:"), "limiter key leaked: {l}");
        assert!(!l.contains("ip:"), "limiter key leaked: {l}");
    }
}
