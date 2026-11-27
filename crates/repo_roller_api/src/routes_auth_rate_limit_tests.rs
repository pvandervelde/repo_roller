//! Router-level tests for the follow-up to #284:
//!
//! (a) `POST /api/v1/auth/token` (public) is limited by its own limiter keyed
//!     by the connection address (`ConnectInfo<SocketAddr>`), never by
//!     forwarding headers.
//! (b) Principals with no resolvable login are isolated (keyed by the JWT
//!     `pid` claim) and never share one bucket; a missing identifier is
//!     rejected rather than pooled.
//!
//! The token endpoint is exercised without an `Authorization` header: the
//! handler answers 401 without any network call, so "allowed" == 401 and
//! "limited" == 429. `ConnectInfo` is injected through request extensions.

use super::*;
use crate::middleware::{generate_backend_jwt, generate_loginless_backend_jwt, AuthContext};
use crate::rate_limit::tests::ManualClock;
use crate::rate_limit::{rate_limit_middleware, RateLimitConfig, RateLimiter};
use axum::body::Body;
use axum::extract::{ConnectInfo, Request as AxumRequest};
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

const AUTH_TOKEN: &str = "/api/v1/auth/token";
const REPOS: &str = "/api/v1/repositories";
const IP_A: &str = "203.0.113.10:40000";
const IP_B: &str = "203.0.113.11:40000";

fn auth_req(addr: Option<&str>, headers: &[(&str, &str)]) -> Request<Body> {
    let mut b = Request::builder().method(Method::POST).uri(AUTH_TOKEN);
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    let mut req = b.body(Body::empty()).unwrap();
    if let Some(a) = addr {
        let sa: SocketAddr = a.parse().unwrap();
        req.extensions_mut().insert(ConnectInfo(sa));
    }
    req
}

fn plain(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn repos_post(token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(REPOS)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from("{}"))
        .unwrap()
}

async fn st(router: &Router, req: Request<Body>) -> StatusCode {
    router.clone().oneshot(req).await.unwrap().status()
}

async fn body_text(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn auth_router(max: u32, window_secs: u64) -> Router {
    create_router(
        AppState::default().with_auth_rate_limit_config(RateLimitConfig::new(
            true,
            max,
            window_secs,
        )),
    )
}

fn clocked_auth_router(max: u32, window_secs: u64) -> (Router, Arc<ManualClock>, Arc<RateLimiter>) {
    let clock = ManualClock::new();
    let limiter = Arc::new(RateLimiter::with_clock(
        RateLimitConfig::new(true, max, window_secs),
        clock.clone(),
    ));
    let router = create_router(AppState::default().with_auth_rate_limiter(limiter.clone()));
    (router, clock, limiter)
}

fn user_token(login: &str) -> String {
    generate_backend_jwt(login, crate::TEST_JWT_SECRET).unwrap()
}

fn loginless_token(pid: &str) -> String {
    generate_loginless_backend_jwt(pid, crate::TEST_JWT_SECRET).unwrap()
}

// ---------------------------------------------------------------------------
// (a) token endpoint limited per client IP
// ---------------------------------------------------------------------------

#[tokio::test]
async fn token_endpoint_allows_n_requests_then_429() {
    let router = auth_router(3, 60);
    let mut out = Vec::new();
    for _ in 0..4 {
        out.push(st(&router, auth_req(Some(IP_A), &[])).await);
    }
    assert!(
        out[..3].iter().all(|s| *s == StatusCode::UNAUTHORIZED),
        "{out:?}"
    );
    assert_eq!(out[3], StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn token_endpoint_limit_of_one_rejects_the_second_request() {
    let router = auth_router(1, 60);
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn token_endpoint_stays_429_while_window_is_open() {
    let router = auth_router(1, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    for _ in 0..5 {
        assert_eq!(
            st(&router, auth_req(Some(IP_A), &[])).await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}

#[tokio::test]
async fn default_token_budget_is_twenty_per_client() {
    let router = create_router(AppState::default());
    for i in 0..20 {
        assert_ne!(
            st(&router, auth_req(Some(IP_A), &[])).await,
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} must pass"
        );
    }
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn different_client_ips_have_isolated_budgets() {
    let router = auth_router(1, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        st(&router, auth_req(Some(IP_B), &[])).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        st(&router, auth_req(Some(IP_B), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn same_ip_different_source_ports_share_one_budget() {
    let router = auth_router(1, 60);
    st(&router, auth_req(Some("203.0.113.10:1111"), &[])).await;
    assert_eq!(
        st(&router, auth_req(Some("203.0.113.10:2222"), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn rotating_x_forwarded_for_does_not_evade_the_limit() {
    let router = auth_router(1, 60);
    st(
        &router,
        auth_req(Some(IP_A), &[("x-forwarded-for", "198.51.100.1")]),
    )
    .await;
    for i in 2..6 {
        let spoof = format!("198.51.100.{i}");
        assert_eq!(
            st(
                &router,
                auth_req(Some(IP_A), &[("x-forwarded-for", &spoof)])
            )
            .await,
            StatusCode::TOO_MANY_REQUESTS,
            "spoofed XFF {spoof} must not grant a fresh budget"
        );
    }
}

#[tokio::test]
async fn other_forwarding_headers_do_not_evade_the_limit() {
    for (h, v) in [
        ("forwarded", "for=198.51.100.99"),
        ("x-real-ip", "198.51.100.99"),
        ("true-client-ip", "198.51.100.99"),
        ("cf-connecting-ip", "198.51.100.99"),
    ] {
        let router = auth_router(1, 60);
        st(&router, auth_req(Some(IP_A), &[])).await;
        assert_eq!(
            st(&router, auth_req(Some(IP_A), &[(h, v)])).await,
            StatusCode::TOO_MANY_REQUESTS,
            "header {h} must be ignored"
        );
    }
}

#[tokio::test]
async fn attacker_cannot_burn_a_victims_budget_by_spoofing_their_ip() {
    let router = auth_router(1, 60);
    // Attacker (IP_A) claims to be the victim (IP_B) via headers, repeatedly.
    for _ in 0..3 {
        st(
            &router,
            auth_req(Some(IP_A), &[("x-forwarded-for", "203.0.113.11")]),
        )
        .await;
    }
    assert_eq!(
        st(&router, auth_req(Some(IP_B), &[])).await,
        StatusCode::UNAUTHORIZED,
        "victim must still have a full budget"
    );
}

#[tokio::test]
async fn limiter_tracks_connection_addresses_not_header_values() {
    let (router, _clock, limiter) = clocked_auth_router(100, 60);
    for i in 0..5 {
        let spoof = format!("198.51.100.{i}");
        st(
            &router,
            auth_req(Some(IP_A), &[("x-forwarded-for", &spoof)]),
        )
        .await;
    }
    assert_eq!(limiter.tracked_keys(), 1);
    st(&router, auth_req(Some(IP_B), &[])).await;
    assert_eq!(limiter.tracked_keys(), 2);
}

#[tokio::test]
async fn missing_connect_info_is_handled_without_panic_and_shares_the_fallback_bucket() {
    let router = auth_router(1, 60);
    assert_eq!(
        st(&router, auth_req(None, &[])).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        st(
            &router,
            auth_req(None, &[("x-forwarded-for", "198.51.100.7")])
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS,
        "requests with no peer address share one fail-closed bucket"
    );
}

#[tokio::test]
async fn fallback_bucket_exhaustion_does_not_affect_real_clients() {
    let router = auth_router(1, 60);
    st(&router, auth_req(None, &[])).await;
    st(&router, auth_req(None, &[])).await;
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn token_endpoint_429_has_integer_retry_after_within_window() {
    let router = auth_router(1, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    let resp = router
        .clone()
        .oneshot(auth_req(Some(IP_A), &[]))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let secs: u64 = resp
        .headers()
        .get(header::RETRY_AFTER)
        .expect("Retry-After required")
        .to_str()
        .unwrap()
        .parse()
        .expect("integer seconds");
    assert!((1..=60).contains(&secs), "{secs}");
}

#[tokio::test]
async fn token_endpoint_retry_after_reflects_remaining_window() {
    let (router, clock, _l) = clocked_auth_router(1, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    clock.advance(Duration::from_secs(45));
    let resp = router
        .clone()
        .oneshot(auth_req(Some(IP_A), &[]))
        .await
        .unwrap();
    assert_eq!(resp.headers().get(header::RETRY_AFTER).unwrap(), "15");
}

#[tokio::test]
async fn token_endpoint_non_429_carries_no_retry_after() {
    let router = auth_router(5, 60);
    let resp = router
        .clone()
        .oneshot(auth_req(Some(IP_A), &[]))
        .await
        .unwrap();
    assert!(resp.headers().get(header::RETRY_AFTER).is_none());
}

#[tokio::test]
async fn token_endpoint_429_body_is_standard_json_error_without_leaks() {
    let router = auth_router(1, 60);
    st(
        &router,
        auth_req(Some(IP_A), &[("x-forwarded-for", "198.51.100.77")]),
    )
    .await;
    let resp = router
        .clone()
        .oneshot(auth_req(
            Some(IP_A),
            &[("x-forwarded-for", "198.51.100.77")],
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("application/json"), "{ct}");
    let body = body_text(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(v["error"]["code"], "RateLimitExceeded");
    assert!(!v["error"]["message"].as_str().unwrap_or("").is_empty());
    assert!(
        !body.contains("203.0.113.10"),
        "body leaks client ip: {body}"
    );
    assert!(
        !body.contains("198.51.100.77"),
        "body leaks header ip: {body}"
    );
    for needle in ["Mutex", "HashMap", "RateLimiter", "panicked"] {
        assert!(!body.contains(needle), "{needle}: {body}");
    }
}

#[tokio::test]
async fn token_endpoint_budget_resets_after_window() {
    let (router, clock, _l) = clocked_auth_router(1, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    clock.advance(Duration::from_secs(60));
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn disabled_config_never_limits_the_token_endpoint() {
    let router = create_router(
        AppState::default().with_auth_rate_limit_config(RateLimitConfig::new(false, 1, 60)),
    );
    for _ in 0..10 {
        assert_eq!(
            st(&router, auth_req(Some(IP_A), &[])).await,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn only_post_is_limited_on_the_token_route_and_gets_do_not_consume_budget() {
    let router = auth_router(1, 60);
    for _ in 0..5 {
        let mut r = plain(Method::GET, AUTH_TOKEN);
        r.extensions_mut()
            .insert(ConnectInfo(IP_A.parse::<SocketAddr>().unwrap()));
        assert_ne!(st(&router, r).await, StatusCode::TOO_MANY_REQUESTS);
    }
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn auth_limiter_memory_is_bounded_by_eviction_of_expired_clients() {
    let (router, clock, limiter) = clocked_auth_router(5, 60);
    let addr = |i: usize| format!("10.0.{}.{}:5000", i / 250, i % 250 + 1);
    for i in 0..300 {
        st(&router, auth_req(Some(&addr(i)), &[])).await;
    }
    assert_eq!(
        limiter.tracked_keys(),
        300,
        "one key per distinct client ip"
    );
    clock.advance(Duration::from_secs(61));
    for i in 300..600 {
        st(&router, auth_req(Some(&addr(i)), &[])).await;
    }
    assert!(
        limiter.tracked_keys() <= 300,
        "expired clients must be evicted, tracked {}",
        limiter.tracked_keys()
    );
}

// ---- separate budgets ------------------------------------------------------

#[tokio::test]
async fn auth_limiter_is_separate_from_the_per_user_limiter() {
    let state = AppState::default()
        .with_rate_limit_config(RateLimitConfig::new(true, 1, 60))
        .with_auth_rate_limit_config(RateLimitConfig::new(true, 1, 60));
    let router = create_router(state);
    let t = user_token("alice-sep-user");
    assert_ne!(
        st(&router, repos_post(&t)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        st(&router, repos_post(&t)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    // User budget exhausted, token endpoint budget untouched.
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        st(&router, auth_req(Some(IP_A), &[])).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    // Token endpoint exhausted, other users' budgets unaffected.
    assert_ne!(
        st(&router, repos_post(&user_token("bob-sep-user"))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn per_user_config_change_does_not_alter_the_auth_limiter() {
    let router = create_router(
        AppState::default().with_rate_limit_config(RateLimitConfig::new(true, 1, 60)),
    );
    for _ in 0..5 {
        assert_eq!(
            st(&router, auth_req(Some(IP_A), &[])).await,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn auth_limiter_is_not_consulted_for_other_routes() {
    let (router, _c, auth_limiter) = clocked_auth_router(100, 60);
    for _ in 0..3 {
        st(&router, plain(Method::GET, "/api/v1/health")).await;
        st(&router, plain(Method::POST, REPOS)).await;
        st(&router, repos_post(&user_token("alice-iso-user"))).await;
    }
    assert_eq!(auth_limiter.tracked_keys(), 0);
}

#[tokio::test]
async fn token_requests_do_not_touch_the_per_user_limiter() {
    let user_limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 100, 60)));
    let router = create_router(AppState::default().with_rate_limiter(user_limiter.clone()));
    for _ in 0..3 {
        st(&router, auth_req(Some(IP_A), &[])).await;
    }
    assert_eq!(user_limiter.tracked_keys(), 0);
}

// ---- metrics and logging ---------------------------------------------------

fn metric_lines(body: &str, name: &str) -> Vec<String> {
    body.lines()
        .filter(|l| l.starts_with(name) && !l.starts_with('#'))
        .filter(|l| !l.contains('\u{10FFFF}'))
        .map(str::to_string)
        .collect()
}

fn value(line: &str) -> f64 {
    line.rsplit(' ').next().unwrap().parse().unwrap()
}

async fn metrics_body(router: &Router) -> String {
    body_text(
        router
            .clone()
            .oneshot(plain(Method::GET, "/metrics"))
            .await
            .unwrap(),
    )
    .await
}

#[tokio::test]
async fn token_rejections_are_counted_under_scope_auth_ip() {
    let router = auth_router(1, 60);
    for _ in 0..3 {
        st(&router, auth_req(Some(IP_A), &[])).await; // 1 allowed, 2 rejected
    }
    let lines = metric_lines(&metrics_body(&router).await, "rate_limit_rejections_total");
    let auth: Vec<_> = lines
        .iter()
        .filter(|l| l.contains("scope=\"auth_ip\""))
        .collect();
    assert_eq!(auth.len(), 1, "{lines:?}");
    assert_eq!(value(auth[0]), 2.0);
}

#[tokio::test]
async fn user_and_token_rejections_are_counted_under_distinct_scopes() {
    let state = AppState::default()
        .with_rate_limit_config(RateLimitConfig::new(true, 1, 60))
        .with_auth_rate_limit_config(RateLimitConfig::new(true, 1, 60));
    let router = create_router(state);
    let t = user_token("alice-met-user");
    st(&router, repos_post(&t)).await;
    st(&router, repos_post(&t)).await; // 1 user rejection
    st(&router, auth_req(Some(IP_A), &[])).await;
    st(&router, auth_req(Some(IP_A), &[])).await;
    st(&router, auth_req(Some(IP_A), &[])).await; // 2 auth rejections
    let lines = metric_lines(&metrics_body(&router).await, "rate_limit_rejections_total");
    let sum = |scope: &str| -> f64 {
        lines
            .iter()
            .filter(|l| l.contains(&format!("scope=\"{scope}\"")))
            .map(|l| value(l))
            .sum()
    };
    assert_eq!(sum("user"), 1.0, "{lines:?}");
    assert_eq!(sum("auth_ip"), 2.0, "{lines:?}");
}

#[tokio::test]
async fn rejection_metric_has_bounded_cardinality_and_no_client_ip() {
    let router = auth_router(1, 60);
    for i in 0..8 {
        let a = format!("203.0.113.{}:9000", i + 20);
        st(&router, auth_req(Some(&a), &[])).await;
        st(&router, auth_req(Some(&a), &[])).await;
    }
    let body = metrics_body(&router).await;
    let lines = metric_lines(&body, "rate_limit_rejections_total");
    assert!(!lines.is_empty());
    assert!(
        lines.len() <= 2,
        "series must not scale with clients: {lines:?}"
    );
    assert!(
        !body.contains("203.0.113."),
        "client ip leaked into /metrics"
    );
}

#[derive(Clone, Default)]
struct LogBuf(Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogBuf {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogBuf {
    type Writer = LogBuf;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn warn_capture() -> (LogBuf, tracing::subscriber::DefaultGuard) {
    let buf = LogBuf::default();
    let sub = tracing_subscriber::fmt()
        .with_writer(buf.clone())
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .finish();
    let guard = tracing::subscriber::set_default(sub);
    (buf, guard)
}

#[tokio::test]
async fn token_rejection_is_logged_at_warn_without_the_bearer_secret() {
    let (buf, _g) = warn_capture();
    let router = auth_router(1, 60);
    let secret = "ghp_supersecrettokenvalue123";
    let with_secret = |ip| auth_req(Some(ip), &[("authorization", &format!("Bearer {secret}"))]);
    st(&router, with_secret(IP_A)).await;
    st(&router, with_secret(IP_A)).await;
    let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        out.lines()
            .any(|l| l.contains("WARN") && l.to_lowercase().contains("rate limit")),
        "expected WARN mentioning rate limit:\n{out}"
    );
    assert!(
        !out.contains("supersecrettoken"),
        "secret leaked into logs:\n{out}"
    );
}

#[tokio::test]
async fn allowed_token_requests_emit_no_rate_limit_warning() {
    let (buf, _g) = warn_capture();
    let router = auth_router(5, 60);
    st(&router, auth_req(Some(IP_A), &[])).await;
    st(&router, auth_req(Some(IP_A), &[])).await;
    let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(!out.to_lowercase().contains("rate limit"), "{out}");
}

// ---------------------------------------------------------------------------
// (b) loginless principals are isolated, never pooled
// ---------------------------------------------------------------------------

fn user_router(max: u32) -> Router {
    create_router(AppState::default().with_rate_limit_config(RateLimitConfig::new(true, max, 60)))
}

#[tokio::test]
async fn distinct_loginless_principals_have_independent_budgets() {
    let router = user_router(1);
    let a = loginless_token("inst-aaaaaaaaaaaaaaaa");
    let b = loginless_token("inst-bbbbbbbbbbbbbbbb");
    assert_ne!(
        st(&router, repos_post(&a)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        st(&router, repos_post(&a)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        st(&router, repos_post(&b)).await,
        StatusCode::TOO_MANY_REQUESTS,
        "principal B must not share principal A's bucket"
    );
}

#[tokio::test]
async fn same_loginless_principal_shares_one_budget_across_tokens() {
    let router = user_router(1);
    let t1 = loginless_token("inst-aaaaaaaaaaaaaaaa");
    let t2 = loginless_token("inst-aaaaaaaaaaaaaaaa");
    st(&router, repos_post(&t1)).await;
    assert_eq!(
        st(&router, repos_post(&t2)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn user_named_unknown_is_not_throttled_by_a_loginless_principal() {
    let router = user_router(1);
    let loginless = loginless_token("unknown");
    st(&router, repos_post(&loginless)).await;
    assert_eq!(
        st(&router, repos_post(&loginless)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        st(&router, repos_post(&user_token("unknown"))).await,
        StatusCode::TOO_MANY_REQUESTS,
        "login 'unknown' must not collide with a loginless principal"
    );
}

#[tokio::test]
async fn loginless_principal_is_not_throttled_by_a_user_named_unknown() {
    let router = user_router(1);
    let u = user_token("unknown");
    st(&router, repos_post(&u)).await;
    assert_eq!(
        st(&router, repos_post(&u)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        st(&router, repos_post(&loginless_token("unknown"))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn login_keys_are_case_insensitive_end_to_end() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 1, 60)));
    let router = create_router(AppState::default().with_rate_limiter(limiter.clone()));
    st(&router, repos_post(&user_token("Alice-Case-User"))).await;
    assert_eq!(
        st(&router, repos_post(&user_token("ALICE-CASE-USER"))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(limiter.tracked_keys(), 1);
}

#[tokio::test]
async fn loginless_and_login_principals_are_tracked_as_separate_keys() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 5, 60)));
    let router = create_router(AppState::default().with_rate_limiter(limiter.clone()));
    st(
        &router,
        repos_post(&loginless_token("inst-aaaaaaaaaaaaaaaa")),
    )
    .await;
    st(
        &router,
        repos_post(&loginless_token("inst-bbbbbbbbbbbbbbbb")),
    )
    .await;
    st(&router, repos_post(&user_token("unknown"))).await;
    assert_eq!(limiter.tracked_keys(), 3);
}

/// Direct middleware harness: injects an arbitrary (or no) `AuthContext`
/// ahead of `rate_limit_middleware`.
fn mw_router(state: AppState, ctx: Option<AuthContext>, reached: Arc<AtomicBool>) -> Router {
    Router::new()
        .route(
            "/x",
            axum::routing::post(move || {
                let r = reached.clone();
                async move {
                    r.store(true, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn(
            move |mut req: AxumRequest, next: middleware::Next| {
                let ctx = ctx.clone();
                async move {
                    if let Some(c) = ctx {
                        req.extensions_mut().insert(c);
                    }
                    next.run(req).await
                }
            },
        ))
        .with_state(state)
}

fn x_post() -> Request<Body> {
    plain(Method::POST, "/x")
}

#[tokio::test]
async fn context_without_any_identifier_is_rejected_not_pooled() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 100, 60)));
    let state = AppState::default().with_rate_limiter(limiter.clone());
    let reached = Arc::new(AtomicBool::new(false));
    let router = mw_router(
        state,
        Some(AuthContext {
            user_login: None,
            principal_id: None,
        }),
        reached.clone(),
    );
    for _ in 0..3 {
        let s = st(&router, x_post()).await;
        assert!(
            s == StatusCode::UNAUTHORIZED || s == StatusCode::FORBIDDEN,
            "expected 401/403, got {s}"
        );
    }
    assert!(!reached.load(Ordering::SeqCst), "handler must not run");
    assert_eq!(limiter.tracked_keys(), 0, "no shared bucket may be created");
}

#[tokio::test]
async fn missing_auth_context_is_rejected_not_pooled() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 100, 60)));
    let state = AppState::default().with_rate_limiter(limiter.clone());
    let reached = Arc::new(AtomicBool::new(false));
    let router = mw_router(state, None, reached.clone());
    let s = st(&router, x_post()).await;
    assert!(
        s == StatusCode::UNAUTHORIZED || s == StatusCode::FORBIDDEN,
        "expected 401/403, got {s}"
    );
    assert!(!reached.load(Ordering::SeqCst));
    assert_eq!(limiter.tracked_keys(), 0);
}

#[tokio::test]
async fn identifier_less_context_never_yields_429() {
    let state = AppState::default().with_rate_limit_config(RateLimitConfig::new(true, 1, 60));
    let router = mw_router(
        state,
        Some(AuthContext {
            user_login: None,
            principal_id: None,
        }),
        Arc::new(AtomicBool::new(false)),
    );
    for _ in 0..5 {
        assert_ne!(st(&router, x_post()).await, StatusCode::TOO_MANY_REQUESTS);
    }
}

#[tokio::test]
async fn context_with_only_principal_id_is_limited_per_principal() {
    let state = AppState::default().with_rate_limit_config(RateLimitConfig::new(true, 1, 60));
    let reached = Arc::new(AtomicBool::new(false));
    let router = mw_router(
        state,
        Some(AuthContext {
            user_login: None,
            principal_id: Some("inst-aaaaaaaaaaaaaaaa".to_string()),
        }),
        reached.clone(),
    );
    assert_eq!(st(&router, x_post()).await, StatusCode::OK);
    assert!(reached.load(Ordering::SeqCst));
    assert_eq!(st(&router, x_post()).await, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn get_requests_without_identifier_are_not_rejected_by_the_limiter() {
    let state = AppState::default();
    let router = Router::new()
        .route("/x", axum::routing::get(|| async { StatusCode::OK }))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .with_state(state);
    assert_eq!(st(&router, plain(Method::GET, "/x")).await, StatusCode::OK);
}
