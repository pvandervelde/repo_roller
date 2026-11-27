//! Method-scope and missing-principal tests for `rate_limit_middleware`.
//!
//! Contract: every method except GET, HEAD and OPTIONS is rate limited
//! (POST, PUT, PATCH, DELETE, TRACE, CONNECT and extension methods); the
//! safe methods never consume budget and never need a principal. A context
//! with no usable identifier is rejected with 401 `MissingPrincipal`
//! (never pooled, never 429); a request with no `AuthContext` at all stays
//! the plain `AuthenticationError` 401.

use super::*;
use crate::AppState;
use axum::body::Body;
use axum::extract::Request as AxumRequest;
use axum::http::{Method, Request};
use axum::routing::any;
use axum::Router;
use http_body_util::BodyExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

fn ctx_login(login: &str) -> AuthContext {
    AuthContext {
        user_login: Some(login.to_string()),
        principal_id: None,
    }
}

fn ctx_none() -> AuthContext {
    AuthContext {
        user_login: None,
        principal_id: None,
    }
}

/// `/x` accepts any method behind `rate_limit_middleware`, with `ctx` injected.
fn any_router(state: AppState, ctx: Option<AuthContext>, hits: Arc<AtomicUsize>) -> Router {
    Router::new()
        .route(
            "/x",
            any(move || {
                let h = hits.clone();
                async move {
                    h.fetch_add(1, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn(
            move |mut req: AxumRequest, next: axum::middleware::Next| {
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

fn req(method: &Method) -> Request<Body> {
    Request::builder()
        .method(method.clone())
        .uri("/x")
        .body(Body::empty())
        .unwrap()
}

async fn st(router: &Router, method: &Method) -> StatusCode {
    router.clone().oneshot(req(method)).await.unwrap().status()
}

fn limited_state(max: u32) -> (AppState, Arc<RateLimiter>) {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, max, 60)));
    (
        AppState::default().with_rate_limiter(limiter.clone()),
        limiter,
    )
}

async fn assert_limited_after_one(method: Method) {
    let (state, limiter) = limited_state(1);
    let hits = Arc::new(AtomicUsize::new(0));
    let router = any_router(state, Some(ctx_login("scope-user")), hits.clone());
    assert_eq!(st(&router, &method).await, StatusCode::OK, "{method} #1");
    assert_eq!(
        st(&router, &method).await,
        StatusCode::TOO_MANY_REQUESTS,
        "{method} #2 must be limited"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 1, "{method}: handler ran once");
    assert_eq!(limiter.tracked_keys(), 1);
}

async fn assert_never_limited(method: Method) {
    let (state, limiter) = limited_state(1);
    let hits = Arc::new(AtomicUsize::new(0));
    let router = any_router(state, Some(ctx_login("scope-user")), hits.clone());
    for i in 0..6 {
        assert_eq!(
            st(&router, &method).await,
            StatusCode::OK,
            "{method} #{i} must never be limited"
        );
    }
    assert_eq!(hits.load(Ordering::SeqCst), 6);
    assert_eq!(
        limiter.tracked_keys(),
        0,
        "{method} must not consume budget"
    );
    // The full budget is still available to a non-safe request.
    assert_eq!(st(&router, &Method::POST).await, StatusCode::OK);
}

#[tokio::test]
async fn post_is_rate_limited() {
    assert_limited_after_one(Method::POST).await;
}

#[tokio::test]
async fn put_is_rate_limited() {
    assert_limited_after_one(Method::PUT).await;
}

#[tokio::test]
async fn patch_is_rate_limited() {
    assert_limited_after_one(Method::PATCH).await;
}

#[tokio::test]
async fn delete_is_rate_limited() {
    assert_limited_after_one(Method::DELETE).await;
}

#[tokio::test]
async fn trace_is_rate_limited() {
    assert_limited_after_one(Method::TRACE).await;
}

#[tokio::test]
async fn extension_method_is_rate_limited() {
    assert_limited_after_one(Method::from_bytes(b"PURGE").unwrap()).await;
}

#[tokio::test]
async fn get_never_consumes_budget() {
    assert_never_limited(Method::GET).await;
}

#[tokio::test]
async fn head_never_consumes_budget() {
    assert_never_limited(Method::HEAD).await;
}

#[tokio::test]
async fn options_never_consumes_budget() {
    assert_never_limited(Method::OPTIONS).await;
}

#[tokio::test]
async fn different_unsafe_methods_share_one_budget_per_principal() {
    let (state, _) = limited_state(2);
    let router = any_router(
        state,
        Some(ctx_login("scope-user")),
        Arc::new(AtomicUsize::new(0)),
    );
    assert_eq!(st(&router, &Method::POST).await, StatusCode::OK);
    assert_eq!(st(&router, &Method::DELETE).await, StatusCode::OK);
    assert_eq!(
        st(&router, &Method::PUT).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn unsafe_methods_are_counted_in_the_user_rejection_metric() {
    let (state, _) = limited_state(1);
    let metrics = state.rate_limit_rejections.clone();
    let router = any_router(
        state,
        Some(ctx_login("scope-user")),
        Arc::new(AtomicUsize::new(0)),
    );
    st(&router, &Method::DELETE).await;
    st(&router, &Method::DELETE).await;
    assert_eq!(metrics.with_label_values(&[SCOPE_USER]).get(), 1);
}

#[tokio::test]
async fn safe_methods_without_any_identifier_pass_through() {
    for m in [Method::GET, Method::HEAD, Method::OPTIONS] {
        let (state, _) = limited_state(1);
        let hits = Arc::new(AtomicUsize::new(0));
        let router = any_router(state, Some(ctx_none()), hits.clone());
        assert_eq!(st(&router, &m).await, StatusCode::OK, "{m}");
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
}

// ---------------------------------------------------------------------------
// Missing principal identifier
// ---------------------------------------------------------------------------

async fn body_json(resp: axum::response::Response) -> (String, serde_json::Value) {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let v = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    (text, v)
}

#[tokio::test]
async fn identifier_less_context_yields_401_missing_principal_for_every_unsafe_method() {
    for m in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        let (state, limiter) = limited_state(100);
        let hits = Arc::new(AtomicUsize::new(0));
        let router = any_router(state, Some(ctx_none()), hits.clone());
        let resp = router.clone().oneshot(req(&m)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "{m}");
        let (_, v) = body_json(resp).await;
        assert_eq!(v["error"]["code"], "MissingPrincipal", "{m}: {v}");
        assert_eq!(hits.load(Ordering::SeqCst), 0, "{m}: handler must not run");
        assert_eq!(limiter.tracked_keys(), 0, "{m}: no shared bucket");
    }
}

#[tokio::test]
async fn empty_login_and_empty_pid_context_is_missing_principal() {
    let (state, _) = limited_state(100);
    let ctx = AuthContext {
        user_login: Some(String::new()),
        principal_id: Some(String::new()),
    };
    let router = any_router(state, Some(ctx), Arc::new(AtomicUsize::new(0)));
    let resp = router.clone().oneshot(req(&Method::POST)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let (_, v) = body_json(resp).await;
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn missing_principal_rejection_does_not_say_provide_a_bearer_token() {
    let (state, _) = limited_state(100);
    let router = any_router(state, Some(ctx_none()), Arc::new(AtomicUsize::new(0)));
    let resp = router.clone().oneshot(req(&Method::POST)).await.unwrap();
    let (text, v) = body_json(resp).await;
    let msg = v["error"]["message"].as_str().unwrap().to_lowercase();
    assert!(!msg.contains("bearer"), "{msg}");
    assert!(!text.contains("Authorization"), "{text}");
}

#[tokio::test]
async fn missing_principal_is_never_429_even_with_a_budget_of_one() {
    let (state, _) = limited_state(1);
    let router = any_router(state, Some(ctx_none()), Arc::new(AtomicUsize::new(0)));
    for _ in 0..4 {
        assert_eq!(st(&router, &Method::POST).await, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn missing_principal_rejection_does_not_count_as_a_rate_limit_rejection() {
    let (state, _) = limited_state(1);
    let metrics = state.rate_limit_rejections.clone();
    let router = any_router(state, Some(ctx_none()), Arc::new(AtomicUsize::new(0)));
    st(&router, &Method::POST).await;
    st(&router, &Method::POST).await;
    assert_eq!(metrics.with_label_values(&[SCOPE_USER]).get(), 0);
}

#[tokio::test]
async fn absent_auth_context_stays_the_plain_authentication_error() {
    let (state, limiter) = limited_state(100);
    let router = any_router(state, None, Arc::new(AtomicUsize::new(0)));
    let resp = router.clone().oneshot(req(&Method::POST)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let (_, v) = body_json(resp).await;
    assert_eq!(v["error"]["code"], "AuthenticationError", "{v}");
    assert_eq!(limiter.tracked_keys(), 0);
}

#[test]
fn a_real_login_named_unknown_has_its_own_key_distinct_from_any_principal_id() {
    let login = principal_key(&AuthContext {
        user_login: Some("unknown".into()),
        principal_id: None,
    });
    assert_eq!(login.as_deref(), Some("login:unknown"));
    for pid in ["unknown", "login:unknown", "0123456789abcdef"] {
        let p = principal_key(&AuthContext {
            user_login: None,
            principal_id: Some(pid.into()),
        });
        assert_ne!(p, login, "pid {pid:?} collides with login:unknown");
    }
}
