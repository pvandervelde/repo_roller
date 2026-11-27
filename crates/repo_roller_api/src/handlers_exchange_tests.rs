//! Tests for `exchange_github_token` with GitHub stubbed by wiremock.
//!
//! Contract: `POST /api/v1/auth/token` validates the presented GitHub token
//! with `GET /rate_limit` and resolves the login with `GET /user`, both
//! against `AppState::github_api_base_url` (default `https://api.github.com`).
//! - login resolved: JWT has `sub=<login>` and no `pid`;
//! - `/user` fails after validation passed: loginless JWT with
//!   `pid = principal_id_for_token(token)`, no `sub`, never the string "unknown";
//! - token rejected by GitHub, or no usable bearer token: 401, no JWT issued,
//!   and `/user` is never called.

use crate::middleware::principal_id_for_token;
use crate::routes::create_router;
use crate::{AppState, TEST_JWT_SECRET};
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use base64::Engine;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::matchers::{header as wm_header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const GH_TOKEN: &str = "ghp_exchange_test_token_0123456789";

fn rate_limit_body() -> serde_json::Value {
    let rate =
        serde_json::json!({"limit": 5000, "used": 1, "remaining": 4999, "reset": 1700000000u64});
    serde_json::json!({
        "resources": {"core": rate, "search": rate},
        "rate": rate
    })
}

fn user_body(login: &str) -> serde_json::Value {
    let u = |p: &str| format!("https://api.github.com/{p}");
    serde_json::json!({
        "login": login, "id": 4242, "node_id": "MDQ6VXNlcjQyNDI=",
        "avatar_url": u("a"), "gravatar_id": "", "url": u("users/x"),
        "html_url": "https://github.com/x", "followers_url": u("f"),
        "following_url": u("fg"), "gists_url": u("g"), "starred_url": u("s"),
        "subscriptions_url": u("sub"), "organizations_url": u("o"),
        "repos_url": u("r"), "events_url": u("e"), "received_events_url": u("re"),
        "type": "User", "site_admin": false, "name": null, "company": null,
        "blog": "", "location": null, "hireable": null, "bio": null,
        "twitter_username": null, "public_repos": 1, "public_gists": 0,
        "followers": 0, "following": 0,
        "created_at": "2020-01-01T00:00:00Z", "updated_at": "2020-01-01T00:00:00Z"
    })
}

fn bearer() -> String {
    format!("Bearer {GH_TOKEN}")
}

async fn mock_rate_limit_ok(server: &MockServer, expected_calls: u64) {
    Mock::given(method("GET"))
        .and(path("/rate_limit"))
        .and(wm_header("authorization", bearer().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(rate_limit_body()))
        .expect(expected_calls)
        .mount(server)
        .await;
}

async fn mock_user_ok(server: &MockServer, login: &str, expected_calls: u64) {
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(wm_header("authorization", bearer().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_body(login)))
        .expect(expected_calls)
        .mount(server)
        .await;
}

fn exchange_request(auth: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/token");
    if let Some(a) = auth {
        b = b.header(header::AUTHORIZATION, a);
    }
    b.body(Body::empty()).unwrap()
}

async fn exchange(server: &MockServer, auth: Option<&str>) -> (StatusCode, String) {
    let state = AppState::default().with_github_api_base_url(server.uri());
    let resp = create_router(state)
        .oneshot(exchange_request(auth))
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

fn issued_token(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).expect("200 body must be JSON");
    v["token"].as_str().expect("token field").to_string()
}

/// Decode and signature-verify the issued JWT; returns its claims.
fn claims_of(token: &str) -> serde_json::Value {
    let key = jsonwebtoken::DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()).unwrap();
    jsonwebtoken::decode::<serde_json::Value>(token, &key, &jsonwebtoken::Validation::default())
        .expect("issued JWT must verify against the server secret")
        .claims
}

#[tokio::test]
async fn exchange_with_resolvable_login_returns_200() {
    let server = MockServer::start().await;
    mock_rate_limit_ok(&server, 1).await;
    mock_user_ok(&server, "octo-Cat", 1).await;
    let (status, _) = exchange(&server, Some(&bearer())).await;
    assert_eq!(status, StatusCode::OK);
    server.verify().await;
}

#[tokio::test]
async fn exchange_with_resolvable_login_issues_jwt_with_sub_equal_to_login() {
    let server = MockServer::start().await;
    mock_rate_limit_ok(&server, 1).await;
    mock_user_ok(&server, "octo-Cat", 1).await;
    let (_, body) = exchange(&server, Some(&bearer())).await;
    assert_eq!(claims_of(&issued_token(&body))["sub"], "octo-Cat");
}

#[tokio::test]
async fn exchange_with_resolvable_login_issues_jwt_without_pid() {
    let server = MockServer::start().await;
    mock_rate_limit_ok(&server, 1).await;
    mock_user_ok(&server, "octo-Cat", 1).await;
    let (_, body) = exchange(&server, Some(&bearer())).await;
    assert!(claims_of(&issued_token(&body)).get("pid").is_none());
}

#[tokio::test]
async fn exchange_user_lookup_uses_the_callers_github_token() {
    let server = MockServer::start().await;
    mock_rate_limit_ok(&server, 1).await;
    mock_user_ok(&server, "someone", 1).await;
    let (_, body) = exchange(&server, Some(&bearer())).await;
    assert_eq!(claims_of(&issued_token(&body))["sub"], "someone");
    server.verify().await;
}

async fn loginless_exchange(user_status: u16) -> (StatusCode, String) {
    let server = MockServer::start().await;
    mock_rate_limit_ok(&server, 1).await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(user_status)
                .set_body_json(serde_json::json!({"message": "nope"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let r = exchange(&server, Some(&bearer())).await;
    server.verify().await;
    r
}

#[tokio::test]
async fn exchange_when_user_endpoint_returns_500_still_returns_200() {
    assert_eq!(loginless_exchange(500).await.0, StatusCode::OK);
}

#[tokio::test]
async fn exchange_when_user_endpoint_returns_500_issues_pid_from_github_token() {
    let (_, body) = loginless_exchange(500).await;
    assert_eq!(
        claims_of(&issued_token(&body))["pid"],
        principal_id_for_token(GH_TOKEN)
    );
}

#[tokio::test]
async fn exchange_when_user_endpoint_returns_404_issues_pid_from_github_token() {
    let (status, body) = loginless_exchange(404).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        claims_of(&issued_token(&body))["pid"],
        principal_id_for_token(GH_TOKEN)
    );
}

#[tokio::test]
async fn exchange_loginless_jwt_has_no_sub_claim() {
    let (_, body) = loginless_exchange(500).await;
    assert!(claims_of(&issued_token(&body)).get("sub").is_none());
}

#[tokio::test]
async fn exchange_loginless_jwt_never_contains_unknown_placeholder() {
    let (_, body) = loginless_exchange(500).await;
    let token = issued_token(&body);
    let payload = token.split('.').nth(1).unwrap();
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .unwrap();
    let text = String::from_utf8(raw).unwrap().to_lowercase();
    assert!(!text.contains("unknown"), "payload was: {text}");
}

#[tokio::test]
async fn exchange_loginless_pid_differs_between_distinct_github_tokens() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rate_limit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(rate_limit_body()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    let (_, a) = exchange(&server, Some("Bearer inst-token-A")).await;
    let (_, b) = exchange(&server, Some("Bearer inst-token-B")).await;
    let pid_a = claims_of(&issued_token(&a))["pid"].clone();
    let pid_b = claims_of(&issued_token(&b))["pid"].clone();
    assert_eq!(pid_a, principal_id_for_token("inst-token-A"));
    assert_ne!(pid_a, pid_b);
}

async fn mount_rejecting_rate_limit(server: &MockServer, status: u16) {
    Mock::given(method("GET"))
        .and(path("/rate_limit"))
        .respond_with(
            ResponseTemplate::new(status)
                .set_body_json(serde_json::json!({"message": "Bad credentials"})),
        )
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn exchange_rejected_github_token_returns_401() {
    let server = MockServer::start().await;
    mount_rejecting_rate_limit(&server, 401).await;
    let (status, _) = exchange(&server, Some(&bearer())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    server.verify().await;
}

#[tokio::test]
async fn exchange_rejected_github_token_issues_no_jwt_and_skips_user_lookup() {
    let server = MockServer::start().await;
    mount_rejecting_rate_limit(&server, 401).await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user_body("x")))
        .expect(0)
        .mount(&server)
        .await;
    let (_, body) = exchange(&server, Some(&bearer())).await;
    assert!(!body.contains("\"token\""), "body leaked a token: {body}");
    server.verify().await;
}

#[tokio::test]
async fn exchange_github_server_error_during_validation_returns_401() {
    let server = MockServer::start().await;
    mount_rejecting_rate_limit(&server, 503).await;
    let (status, _) = exchange(&server, Some(&bearer())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    server.verify().await;
}

#[tokio::test]
async fn exchange_without_authorization_header_returns_401_and_calls_no_github_endpoint() {
    let server = MockServer::start().await;
    let (status, body) = exchange(&server, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(!body.contains("\"token\""));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn exchange_with_empty_bearer_token_returns_401_without_calling_github() {
    let server = MockServer::start().await;
    let (status, body) = exchange(&server, Some("Bearer ")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(!body.contains("\"token\""));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn exchange_with_non_bearer_scheme_returns_401_without_calling_github() {
    let server = MockServer::start().await;
    let (status, _) = exchange(&server, Some("Basic dXNlcjpwYXNz")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(server.received_requests().await.unwrap().is_empty());
}
