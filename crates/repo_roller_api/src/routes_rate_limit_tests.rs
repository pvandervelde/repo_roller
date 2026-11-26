//! Router-level tests for rate limiting (Task #284).
//!
//! Drive the production `create_router` (real auth middleware) with real
//! backend-signed JWTs via `tower::ServiceExt::oneshot`. Limiter time is
//! controlled with `ManualClock`; nothing sleeps.
//!
//! Handler outcome for the `{}` bodies used here is irrelevant (it is a
//! validation error); only 429 / 401 / non-429 matter.

use super::*;
use crate::middleware::generate_backend_jwt;
use crate::rate_limit::tests::{limiter, ManualClock};
use crate::rate_limit::{RateLimitConfig, RateLimiter};
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

const REPOS: &str = "/api/v1/repositories";

fn token_for(user: &str) -> String {
    generate_backend_jwt(user, crate::TEST_JWT_SECRET).expect("jwt")
}

fn post(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    b.body(Body::from("{}")).unwrap()
}

fn get(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut b = Request::builder().method(Method::GET).uri(uri);
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    b.body(Body::empty()).unwrap()
}

async fn status(router: &Router, req: Request<Body>) -> StatusCode {
    router.clone().oneshot(req).await.unwrap().status()
}

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn router_with(max: u32, window_secs: u64) -> Router {
    let state =
        AppState::default().with_rate_limit_config(RateLimitConfig::new(true, max, window_secs));
    create_router(state)
}

/// Sends `n` POSTs for `user` and returns the statuses.
async fn burst(router: &Router, user: &str, n: usize) -> Vec<StatusCode> {
    let t = token_for(user);
    let mut out = Vec::new();
    for _ in 0..n {
        out.push(status(router, post(REPOS, Some(&t))).await);
    }
    out
}

// ---------------------------------------------------------------------------
// Core behaviour
// ---------------------------------------------------------------------------

#[tokio::test]
async fn first_n_posts_pass_and_next_is_429() {
    let router = router_with(3, 60);
    let s = burst(&router, "alice-rl-user", 4).await;
    assert!(
        s[..3].iter().all(|c| *c != StatusCode::TOO_MANY_REQUESTS),
        "{s:?}"
    );
    assert!(
        s[..3].iter().all(|c| *c != StatusCode::UNAUTHORIZED),
        "{s:?}"
    );
    assert_eq!(s[3], StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn limit_of_one_rejects_second_request() {
    let router = router_with(1, 60);
    let s = burst(&router, "alice-rl-user", 2).await;
    assert_ne!(s[0], StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(s[1], StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn stays_429_on_continued_requests() {
    let router = router_with(1, 60);
    let s = burst(&router, "alice-rl-user", 6).await;
    assert!(
        s[1..].iter().all(|c| *c == StatusCode::TOO_MANY_REQUESTS),
        "{s:?}"
    );
}

#[tokio::test]
async fn retry_after_header_is_integer_seconds_within_window() {
    let router = router_with(1, 60);
    let t = token_for("alice-rl-user");
    status(&router, post(REPOS, Some(&t))).await;
    let resp = router.clone().oneshot(post(REPOS, Some(&t))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let v = resp
        .headers()
        .get(header::RETRY_AFTER)
        .expect("Retry-After header required on 429")
        .to_str()
        .unwrap()
        .to_string();
    let secs: u64 = v
        .parse()
        .expect("Retry-After must be an integer number of seconds");
    assert!(
        (1..=60).contains(&secs),
        "Retry-After {secs} outside 1..=60"
    );
}

#[tokio::test]
async fn non_429_responses_carry_no_retry_after() {
    let router = router_with(5, 60);
    let t = token_for("alice-rl-user");
    let resp = router.clone().oneshot(post(REPOS, Some(&t))).await.unwrap();
    assert!(resp.headers().get(header::RETRY_AFTER).is_none());
}

#[tokio::test]
async fn body_is_json_error_in_existing_api_format() {
    let router = router_with(1, 60);
    let t = token_for("alice-rl-user");
    status(&router, post(REPOS, Some(&t))).await;
    let resp = router.clone().oneshot(post(REPOS, Some(&t))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("application/json"), "content-type {ct}");
    let v: serde_json::Value = serde_json::from_str(&body_string(resp).await).expect("json body");
    assert_eq!(v["error"]["code"], "RateLimitExceeded");
    assert!(!v["error"]["message"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn body_does_not_leak_user_identity_or_internal_state() {
    let router = router_with(1, 60);
    let t = token_for("alice-rl-user");
    status(&router, post(REPOS, Some(&t))).await;
    let resp = router.clone().oneshot(post(REPOS, Some(&t))).await.unwrap();
    let body = body_string(resp).await;
    assert!(
        !body.contains("alice-rl-user"),
        "body leaks user id: {body}"
    );
    for needle in [
        "Instant",
        "Mutex",
        "HashMap",
        "RateLimiter",
        "src/",
        "panicked",
    ] {
        assert!(
            !body.contains(needle),
            "body leaks internals ({needle}): {body}"
        );
    }
}

// ---------------------------------------------------------------------------
// Isolation, reset, ordering relative to auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn exhausting_one_user_does_not_affect_another() {
    let router = router_with(2, 60);
    let a = burst(&router, "alice-rl-user", 3).await;
    assert_eq!(a[2], StatusCode::TOO_MANY_REQUESTS);
    let b = burst(&router, "bob-rl-user", 2).await;
    assert!(
        b.iter().all(|c| *c != StatusCode::TOO_MANY_REQUESTS),
        "{b:?}"
    );
    let b3 = burst(&router, "bob-rl-user", 1).await;
    assert_eq!(b3[0], StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn same_user_with_two_distinct_tokens_shares_one_budget() {
    let router = router_with(1, 60);
    let t1 = token_for("alice-rl-user");
    status(&router, post(REPOS, Some(&t1))).await;
    // Fresh token (iat may differ) for the same `sub`.
    let t2 = token_for("alice-rl-user");
    assert_eq!(
        status(&router, post(REPOS, Some(&t2))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn budget_resets_after_window() {
    let clock = ManualClock::new();
    let limiter = Arc::new(RateLimiter::with_clock(
        RateLimitConfig::new(true, 1, 60),
        clock.clone(),
    ));
    let router = create_router(AppState::default().with_rate_limiter(limiter));
    let t = token_for("alice-rl-user");
    assert_ne!(
        status(&router, post(REPOS, Some(&t))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        status(&router, post(REPOS, Some(&t))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    clock.advance(Duration::from_secs(60));
    assert_ne!(
        status(&router, post(REPOS, Some(&t))).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn retry_after_header_reflects_remaining_window() {
    let clock = ManualClock::new();
    let limiter = Arc::new(RateLimiter::with_clock(
        RateLimitConfig::new(true, 1, 60),
        clock.clone(),
    ));
    let router = create_router(AppState::default().with_rate_limiter(limiter));
    let t = token_for("alice-rl-user");
    status(&router, post(REPOS, Some(&t))).await;
    clock.advance(Duration::from_secs(45));
    let resp = router.clone().oneshot(post(REPOS, Some(&t))).await.unwrap();
    assert_eq!(
        resp.headers()
            .get(header::RETRY_AFTER)
            .unwrap()
            .to_str()
            .unwrap(),
        "15"
    );
}

#[tokio::test]
async fn unauthenticated_requests_still_get_401_never_429() {
    let router = router_with(1, 60);
    for _ in 0..5 {
        assert_eq!(
            status(&router, post(REPOS, None)).await,
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn invalid_token_requests_get_401_and_do_not_consume_anyones_budget() {
    let router = router_with(1, 60);
    for _ in 0..5 {
        assert_eq!(
            status(&router, post(REPOS, Some("not-a-jwt"))).await,
            StatusCode::UNAUTHORIZED
        );
    }
    let s = burst(&router, "alice-rl-user", 1).await;
    assert_ne!(s[0], StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn unauthenticated_flood_does_not_create_limiter_state() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 1, 60)));
    let router = create_router(AppState::default().with_rate_limiter(limiter.clone()));
    for _ in 0..5 {
        status(&router, post(REPOS, None)).await;
    }
    assert_eq!(
        limiter.tracked_keys(),
        0,
        "rate limiting must run after auth"
    );
}

#[tokio::test]
async fn authenticated_users_are_tracked_by_sub() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 5, 60)));
    let router = create_router(AppState::default().with_rate_limiter(limiter.clone()));
    burst(&router, "alice-rl-user", 1).await;
    burst(&router, "bob-rl-user", 1).await;
    assert_eq!(limiter.tracked_keys(), 2);
}

// ---------------------------------------------------------------------------
// Scope: GET / health / metrics unaffected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_endpoint_is_never_rate_limited() {
    let router = router_with(1, 60);
    for _ in 0..10 {
        assert_eq!(
            status(&router, get("/api/v1/health", None)).await,
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn metrics_endpoint_is_never_rate_limited() {
    let router = router_with(1, 60);
    for _ in 0..10 {
        assert_eq!(status(&router, get("/metrics", None)).await, StatusCode::OK);
    }
}

#[tokio::test]
async fn authenticated_get_requests_are_not_rate_limited_and_do_not_consume_budget() {
    // Handlers for /orgs/* call GitHub; point them at an empty wiremock server
    // (answers 404 instantly) with a mock installation token. The status the
    // handler maps that to is irrelevant; only 429-ness matters.
    let mock = wiremock::MockServer::start().await;
    let state = AppState::default()
        .with_rate_limit_config(RateLimitConfig::new(true, 1, 60))
        .with_mock_installation_token("mock-token")
        .with_github_api_base_url(mock.uri());
    let router = create_router(state);
    let t = token_for("alice-rl-user");
    for _ in 0..5 {
        let s = status(&router, get("/api/v1/orgs/acme/templates", Some(&t))).await;
        assert_ne!(s, StatusCode::TOO_MANY_REQUESTS);
    }
    let s = burst(&router, "alice-rl-user", 1).await;
    assert_ne!(
        s[0],
        StatusCode::TOO_MANY_REQUESTS,
        "GETs must not eat the POST budget"
    );
}

#[tokio::test]
async fn disabled_config_never_limits() {
    let router = create_router(
        AppState::default().with_rate_limit_config(RateLimitConfig::new(false, 1, 60)),
    );
    let s = burst(&router, "alice-rl-user", 10).await;
    assert!(
        s.iter().all(|c| *c != StatusCode::TOO_MANY_REQUESTS),
        "{s:?}"
    );
}

#[tokio::test]
async fn limiter_helper_clock_is_wired_through_router_state() {
    // Guards against a stub that ignores the injected limiter in AppState.
    let (l, _clock) = limiter(1, 60);
    let l = Arc::new(l);
    let router = create_router(AppState::default().with_rate_limiter(l.clone()));
    burst(&router, "alice-rl-user", 1).await;
    assert_eq!(l.tracked_keys(), 1);
}

// ---------------------------------------------------------------------------
// Metrics and logging
// ---------------------------------------------------------------------------

fn metric_lines(body: &str, name: &str) -> Vec<String> {
    body.lines()
        .filter(|l| l.starts_with(name) && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[tokio::test]
async fn rejection_is_counted_in_metrics_with_bounded_labels() {
    let router = router_with(1, 60);
    burst(&router, "alice-rl-user", 3).await; // 1 allowed, 2 rejected
    let resp = router.clone().oneshot(get("/metrics", None)).await.unwrap();
    let body = body_string(resp).await;
    let lines = metric_lines(&body, "rate_limit_rejections_total");
    let real: Vec<_> = lines.iter().filter(|l| !l.contains('\u{10FFFF}')).collect();
    assert!(
        !real.is_empty(),
        "rate_limit_rejections_total has no real sample:\n{body}"
    );
    let total: f64 = real
        .iter()
        .map(|l| l.rsplit(' ').next().unwrap().parse::<f64>().unwrap())
        .sum();
    assert_eq!(total, 2.0, "two rejections expected: {real:?}");
}

#[tokio::test]
async fn metrics_never_contain_user_identity() {
    let router = router_with(1, 60);
    burst(&router, "alice-rl-user", 3).await;
    let resp = router.clone().oneshot(get("/metrics", None)).await.unwrap();
    let body = body_string(resp).await;
    assert!(
        !body.contains("alice-rl-user"),
        "user id leaked into /metrics"
    );
}

#[tokio::test]
async fn rejection_label_cardinality_is_independent_of_user_count() {
    let router = router_with(1, 60);
    for i in 0..8 {
        burst(&router, &format!("user-{i}-rl"), 2).await;
    }
    let resp = router.clone().oneshot(get("/metrics", None)).await.unwrap();
    let body = body_string(resp).await;
    let real: Vec<_> = metric_lines(&body, "rate_limit_rejections_total")
        .into_iter()
        .filter(|l| !l.contains('\u{10FFFF}'))
        .collect();
    assert!(
        real.len() <= 2,
        "label series must not scale with users: {real:?}"
    );
}

#[tokio::test]
async fn http_metrics_record_429_status() {
    let router = router_with(1, 60);
    burst(&router, "alice-rl-user", 2).await;
    let resp = router.clone().oneshot(get("/metrics", None)).await.unwrap();
    let body = body_string(resp).await;
    assert!(
        metric_lines(&body, "http_requests_total")
            .iter()
            .any(|l| l.contains("status_code=\"429\"")),
        "429 must be visible in http_requests_total:\n{body}"
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

#[tokio::test]
async fn rejection_is_logged_at_warn() {
    let buf = LogBuf::default();
    let sub = tracing_subscriber::fmt()
        .with_writer(buf.clone())
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);
    let router = router_with(1, 60);
    burst(&router, "alice-rl-user", 2).await;
    let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    let hit = out
        .lines()
        .any(|l| l.contains("WARN") && l.to_lowercase().contains("rate limit"));
    assert!(hit, "expected a WARN log mentioning 'rate limit':\n{out}");
}

#[tokio::test]
async fn allowed_requests_emit_no_rate_limit_warning() {
    let buf = LogBuf::default();
    let sub = tracing_subscriber::fmt()
        .with_writer(buf.clone())
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(sub);
    let router = router_with(5, 60);
    burst(&router, "alice-rl-user", 2).await;
    let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        !out.lines()
            .any(|l| l.contains("WARN") && l.to_lowercase().contains("rate limit")),
        "no rate-limit warning expected:\n{out}"
    );
}
