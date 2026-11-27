//! Route-coverage tests for rate limiting.
//!
//! `ROUTE_TABLE` is the explicit, reviewed inventory of every route and
//! method the production router serves. Tests drive the real `create_router`:
//!
//! * every non-safe-method route is limited after N+1 calls (by the per-user
//!   limiter, or by the per-IP limiter for the public token endpoint);
//! * safe-method routes never consume budget;
//! * the router answers 405 for every (path, method) pair NOT in the table,
//!   so a new method on an existing route is caught;
//! * a source guard compares the paths registered in `routes.rs` with the
//!   table, so a new route that is not added here fails the build.

use super::*;
use crate::middleware::generate_backend_jwt;
use crate::rate_limit::{RateLimitConfig, RateLimiter};
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Auth {
    /// Behind `auth_middleware` + per-user limiter.
    User,
    /// Public, per-client-IP limiter (token endpoint).
    PublicIpLimited,
    /// Public, no limiter, safe method.
    PublicOpen,
}

struct Entry {
    method: Method,
    path: &'static str,
    auth: Auth,
}

const fn e(method: Method, path: &'static str, auth: Auth) -> Entry {
    Entry { method, path, auth }
}

/// Every route the production router serves. Keep in sync with `routes.rs`
/// (the guard test fails otherwise).
fn route_table() -> Vec<Entry> {
    use Auth::*;
    vec![
        e(Method::POST, "/api/v1/repositories", User),
        e(Method::POST, "/api/v1/repositories/validate-name", User),
        e(Method::POST, "/api/v1/repositories/validate", User),
        e(Method::GET, "/api/v1/orgs/{org}/templates", User),
        e(Method::GET, "/api/v1/orgs/{org}/templates/{template}", User),
        e(
            Method::POST,
            "/api/v1/orgs/{org}/templates/{template}/validate",
            User,
        ),
        e(Method::GET, "/api/v1/orgs/{org}/repository-types", User),
        e(
            Method::GET,
            "/api/v1/orgs/{org}/repository-types/{type}",
            User,
        ),
        e(Method::GET, "/api/v1/orgs/{org}/defaults", User),
        e(
            Method::POST,
            "/api/v1/orgs/{org}/configuration/preview",
            User,
        ),
        e(
            Method::GET,
            "/api/v1/orgs/{org}/creation-capabilities",
            User,
        ),
        e(Method::POST, "/api/v1/orgs/{org}/validate", User),
        e(Method::GET, "/api/v1/orgs/{org}/teams", User),
        e(Method::POST, "/api/v1/auth/token", PublicIpLimited),
        e(Method::GET, "/api/v1/health", PublicOpen),
        e(Method::GET, "/metrics", PublicOpen),
    ]
}

const SAFE: [Method; 3] = [Method::GET, Method::HEAD, Method::OPTIONS];
const PROBE_METHODS: [Method; 5] = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
];

fn is_safe(m: &Method) -> bool {
    SAFE.contains(m)
}

fn concrete(path: &str) -> String {
    path.replace("{org}", "acme")
        .replace("{template}", "tpl")
        .replace("{type}", "lib")
}

fn token(user: &str) -> String {
    generate_backend_jwt(user, crate::TEST_JWT_SECRET).unwrap()
}

fn request(entry_method: &Method, path: &str, auth: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method(entry_method.clone())
        .uri(concrete(path))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = auth {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let mut r = b.body(Body::from("{}")).unwrap();
    let sa: SocketAddr = "203.0.113.50:4000".parse().unwrap();
    r.extensions_mut().insert(ConnectInfo(sa));
    r
}

/// Router whose GitHub-facing handlers hit an instant-404 mock server.
async fn router_for(state: AppState) -> (Router, wiremock::MockServer) {
    let mock = wiremock::MockServer::start().await;
    let state = state
        .with_mock_installation_token("mock-token")
        .with_github_api_base_url(mock.uri());
    (create_router(state), mock)
}

async fn status(router: &Router, req: Request<Body>) -> StatusCode {
    router.clone().oneshot(req).await.unwrap().status()
}

const N: u32 = 2;

#[test]
fn table_has_no_duplicate_method_path_pairs() {
    let t = route_table();
    let set: BTreeSet<_> = t.iter().map(|x| (x.method.to_string(), x.path)).collect();
    assert_eq!(set.len(), t.len());
}

#[test]
fn table_declares_at_least_one_limited_route_of_each_kind() {
    let t = route_table();
    assert!(t
        .iter()
        .any(|x| x.auth == Auth::User && !is_safe(&x.method)));
    assert!(t.iter().any(|x| x.auth == Auth::PublicIpLimited));
    assert!(t.iter().any(|x| is_safe(&x.method)));
}

#[tokio::test]
async fn every_unsafe_user_route_is_limited_after_n_plus_one_calls() {
    for (i, entry) in route_table()
        .iter()
        .filter(|x| x.auth == Auth::User && !is_safe(&x.method))
        .enumerate()
    {
        let (router, _mock) = router_for(
            AppState::default().with_rate_limit_config(RateLimitConfig::new(true, N, 60)),
        )
        .await;
        let t = token(&format!("route-cov-user-{i}"));
        for call in 1..=N {
            let s = status(&router, request(&entry.method, entry.path, Some(&t))).await;
            assert_ne!(
                s,
                StatusCode::TOO_MANY_REQUESTS,
                "{} {} call {call}/{N} limited too early",
                entry.method,
                entry.path
            );
            assert_ne!(
                s,
                StatusCode::UNAUTHORIZED,
                "{} {} rejected valid token",
                entry.method,
                entry.path
            );
        }
        assert_eq!(
            status(&router, request(&entry.method, entry.path, Some(&t))).await,
            StatusCode::TOO_MANY_REQUESTS,
            "{} {} is NOT rate limited",
            entry.method,
            entry.path
        );
    }
}

#[tokio::test]
async fn every_public_ip_limited_route_is_limited_after_n_plus_one_calls() {
    for entry in route_table()
        .iter()
        .filter(|x| x.auth == Auth::PublicIpLimited)
    {
        let (router, _mock) = router_for(
            AppState::default().with_auth_rate_limit_config(RateLimitConfig::new(true, N, 60)),
        )
        .await;
        for _ in 0..N {
            // No Authorization header: handler answers 401 without network.
            assert_eq!(
                status(&router, request(&entry.method, entry.path, None)).await,
                StatusCode::UNAUTHORIZED
            );
        }
        assert_eq!(
            status(&router, request(&entry.method, entry.path, None)).await,
            StatusCode::TOO_MANY_REQUESTS,
            "{} {} is NOT rate limited",
            entry.method,
            entry.path
        );
    }
}

#[tokio::test]
async fn safe_method_routes_never_consume_user_budget() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 1, 60)));
    let (router, _mock) = router_for(AppState::default().with_rate_limiter(limiter.clone())).await;
    let t = token("route-cov-safe");
    for entry in route_table().iter().filter(|x| is_safe(&x.method)) {
        let auth = (entry.auth == Auth::User).then_some(t.as_str());
        for _ in 0..4 {
            assert_ne!(
                status(&router, request(&entry.method, entry.path, auth)).await,
                StatusCode::TOO_MANY_REQUESTS,
                "{} {} must never be limited",
                entry.method,
                entry.path
            );
        }
    }
    assert_eq!(limiter.tracked_keys(), 0, "a safe route consumed budget");
}

#[tokio::test]
async fn router_answers_405_for_every_method_not_in_the_table() {
    let table = route_table();
    let paths: BTreeSet<&str> = table.iter().map(|x| x.path).collect();
    // Huge budget so probing never trips the limiter.
    let (router, _mock) = router_for(
        AppState::default()
            .with_rate_limit_config(RateLimitConfig::new(true, 10_000, 60))
            .with_auth_rate_limit_config(RateLimitConfig::new(true, 10_000, 60)),
    )
    .await;
    let t = token("route-cov-probe");
    for path in paths {
        for m in PROBE_METHODS {
            let declared = table.iter().any(|x| x.path == path && x.method == m);
            let s = status(&router, request(&m, path, Some(&t))).await;
            if declared {
                assert_ne!(s, StatusCode::METHOD_NOT_ALLOWED, "{m} {path} declared");
            } else {
                assert_eq!(
                    s,
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{m} {path} is served by the router but missing from ROUTE_TABLE"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Drift guard: paths registered in routes.rs must equal the table's paths.
// ---------------------------------------------------------------------------

/// String literals directly following each `<marker>(` in `src`.
fn literals_after(src: &str, marker: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find(marker) {
        rest = &rest[i + marker.len()..];
        let trimmed = rest.trim_start();
        if let Some(body) = trimmed.strip_prefix('"') {
            if let Some(end) = body.find('"') {
                out.push(body[..end].to_string());
            }
        }
    }
    out
}

fn registered_paths_in_source() -> (BTreeSet<String>, BTreeSet<String>) {
    let src = include_str!("routes.rs").replace("\r\n", "\n");
    let production = src
        .split("/// Create a router for testing without authentication")
        .next()
        .unwrap()
        .to_string();
    let (top, org) = production
        .split_once("fn organization_routes()")
        .expect("organization_routes() not found in routes.rs");
    let nests: BTreeSet<String> = literals_after(&production, ".nest(").into_iter().collect();
    let mut paths = BTreeSet::new();
    for p in literals_after(top, ".route(") {
        if p == "/metrics" {
            paths.insert(p);
        } else {
            paths.insert(format!("/api/v1{p}"));
        }
    }
    for p in literals_after(org, ".route(") {
        paths.insert(format!("/api/v1/orgs/{{org}}{p}"));
    }
    (paths, nests)
}

#[test]
fn nest_prefixes_match_the_assumptions_of_the_guard() {
    let (_, nests) = registered_paths_in_source();
    let expected: BTreeSet<String> = ["/api/v1", "/orgs/{org}"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        nests, expected,
        "routes.rs nests changed: update the guard and ROUTE_TABLE"
    );
}

#[test]
fn route_table_covers_exactly_the_paths_registered_in_routes_rs() {
    let (registered, _) = registered_paths_in_source();
    let table: BTreeSet<String> = route_table().iter().map(|x| x.path.to_string()).collect();
    let missing: Vec<_> = registered.difference(&table).collect();
    let stale: Vec<_> = table.difference(&registered).collect();
    assert!(
        missing.is_empty(),
        "routes registered in routes.rs but absent from ROUTE_TABLE: {missing:?}"
    );
    assert!(
        stale.is_empty(),
        "ROUTE_TABLE paths no longer registered in routes.rs: {stale:?}"
    );
}

#[test]
fn guard_parser_finds_a_plausible_number_of_routes() {
    let (registered, _) = registered_paths_in_source();
    assert!(registered.len() >= 10, "{registered:?}");
}

// ---------------------------------------------------------------------------
// Router-level: tokens that carry no principal identifier.
// ---------------------------------------------------------------------------

fn forge(claims: serde_json::Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(crate::TEST_JWT_SECRET.as_bytes()).unwrap(),
    )
    .unwrap()
}

fn now_claims() -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    serde_json::json!({ "iat": now, "exp": now + 3600 })
}

#[tokio::test]
async fn principalless_token_on_a_limited_route_is_401_missing_principal_never_429() {
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig::new(true, 1, 60)));
    let router = create_router(AppState::default().with_rate_limiter(limiter.clone()));
    let tok = forge(now_claims());
    for _ in 0..4 {
        let resp = router
            .clone()
            .oneshot(request(&Method::POST, "/api/v1/repositories", Some(&tok)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
    }
    assert_eq!(limiter.tracked_keys(), 0);
}

#[tokio::test]
async fn principalless_token_is_rejected_on_safe_routes_too() {
    let mut claims = now_claims();
    claims["sub"] = serde_json::json!("");
    let tok = forge(claims);
    let router = create_router(AppState::default());
    let resp = router
        .clone()
        .oneshot(request(
            &Method::GET,
            "/api/v1/orgs/{org}/teams",
            Some(&tok),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}
