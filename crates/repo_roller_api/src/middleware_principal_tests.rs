//! Tests for the stable principal identifier (`pid` claim) used to key the
//! per-user rate limiter for principals with no resolvable GitHub login.

use super::*;
use crate::AppState;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware,
    routing::get,
    Extension, Json, Router,
};
use base64::Engine as _;
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn whoami(Extension(ctx): Extension<AuthContext>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_login": ctx.user_login,
        "principal_id": ctx.principal_id,
    }))
}

async fn identify(token: &str) -> serde_json::Value {
    let app = Router::new()
        .route("/who", get(whoami))
        .route_layer(middleware::from_fn_with_state(
            AppState::default(),
            auth_middleware,
        ));
    let req = Request::builder()
        .uri("/who")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn payload(token: &str) -> serde_json::Value {
    let part = token.split('.').nth(1).unwrap();
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(part)
        .unwrap();
    serde_json::from_slice(&raw).unwrap()
}

#[test]
fn loginless_jwt_carries_the_principal_id_claim() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    let claims = validate_backend_jwt(&t, crate::TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.pid.as_deref(), Some("inst-42"));
}

#[test]
fn login_jwt_has_no_principal_id_claim() {
    let t = generate_backend_jwt("bob", crate::TEST_JWT_SECRET).unwrap();
    let claims = validate_backend_jwt(&t, crate::TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.pid, None);
    assert_eq!(claims.sub.as_deref(), Some("bob"));
    assert!(
        payload(&t).get("pid").is_none(),
        "pid must not be serialized when absent"
    );
}

#[test]
fn loginless_jwt_is_signed_and_rejected_under_another_secret() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    assert!(validate_backend_jwt(&t, "a-different-secret-of-32-bytes-xx!").is_err());
}

#[tokio::test]
async fn auth_middleware_exposes_loginless_principal_without_a_login() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    let v = identify(&t).await;
    assert!(
        v["user_login"].is_null(),
        "loginless principal must have no login: {v}"
    );
    assert_eq!(v["principal_id"], "inst-42");
}

#[tokio::test]
async fn auth_middleware_exposes_login_principal_without_principal_id() {
    let t = generate_backend_jwt("bob", crate::TEST_JWT_SECRET).unwrap();
    let v = identify(&t).await;
    assert_eq!(v["user_login"], "bob");
    assert!(v["principal_id"].is_null());
}

#[tokio::test]
async fn real_user_named_unknown_keeps_their_login_and_gets_no_principal_id() {
    let t = generate_backend_jwt("unknown", crate::TEST_JWT_SECRET).unwrap();
    let v = identify(&t).await;
    assert_eq!(v["user_login"], "unknown");
    assert!(v["principal_id"].is_null());
}

#[test]
fn principal_id_for_token_is_deterministic_and_non_empty() {
    let a = principal_id_for_token("ghs_installationtokenA");
    assert!(!a.is_empty());
    assert_eq!(a, principal_id_for_token("ghs_installationtokenA"));
}

#[test]
fn principal_id_for_token_differs_between_tokens() {
    assert_ne!(
        principal_id_for_token("ghs_installationtokenA"),
        principal_id_for_token("ghs_installationtokenB")
    );
}

#[test]
fn principal_id_for_token_is_a_hex_digest_that_does_not_embed_the_token() {
    let tok = "ghs_installationtokenA";
    let id = principal_id_for_token(tok);
    assert!(id.len() >= 16 && id.len() <= 128, "len {}", id.len());
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()), "{id}");
    assert!(!id.contains(tok));
}

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn principal_id_never_contains_the_token(tok in "[g-z_]{8,60}") {
            prop_assert!(!principal_id_for_token(&tok).contains(&tok));
        }

        #[test]
        fn distinct_tokens_get_distinct_ids(a in "[ -~]{1,60}", b in "[ -~]{1,60}") {
            prop_assume!(a != b);
            prop_assert_ne!(principal_id_for_token(&a), principal_id_for_token(&b));
        }

        #[test]
        fn pid_round_trips_through_the_jwt(pid in "[a-f0-9]{16,64}") {
            let t = generate_loginless_backend_jwt(&pid, crate::TEST_JWT_SECRET).unwrap();
            let claims = validate_backend_jwt(&t, crate::TEST_JWT_SECRET).unwrap();
            prop_assert_eq!(claims.pid.as_deref(), Some(pid.as_str()));
        }
    }
}

// ---------------------------------------------------------------------------
// No legacy `sub:"unknown"` placeholder; loginless tokens are pid-only.
// Design: `Claims.sub` is `Option<String>`; loginless tokens omit `sub`
// entirely (JWT `sub` is optional), login tokens omit `pid`.
// ---------------------------------------------------------------------------

use jsonwebtoken::{EncodingKey, Header};

fn forge(claims: serde_json::Value) -> String {
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(crate::TEST_JWT_SECRET.as_bytes()).unwrap(),
    )
    .unwrap()
}

fn claims_with(extra: serde_json::Value) -> serde_json::Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut v = serde_json::json!({ "iat": now, "exp": now + 3600 });
    for (k, val) in extra.as_object().unwrap() {
        v[k] = val.clone();
    }
    v
}

async fn call(token: &str) -> (StatusCode, serde_json::Value) {
    let app = Router::new()
        .route("/who", get(whoami))
        .route_layer(middleware::from_fn_with_state(
            AppState::default(),
            auth_middleware,
        ));
    let req = Request::builder()
        .uri("/who")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, v)
}

#[test]
fn loginless_jwt_carries_no_sub_claim_at_all() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    assert!(
        payload(&t).get("sub").is_none(),
        "no placeholder subject may be issued: {}",
        payload(&t)
    );
}

#[test]
fn loginless_jwt_payload_never_contains_the_word_unknown() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    assert!(!payload(&t).to_string().to_lowercase().contains("unknown"));
}

#[test]
fn loginless_jwt_decodes_with_sub_none() {
    let t = generate_loginless_backend_jwt("inst-42", crate::TEST_JWT_SECRET).unwrap();
    let claims = validate_backend_jwt(&t, crate::TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.sub, None);
}

#[test]
fn handlers_source_contains_no_unknown_placeholder_literal() {
    // exchange_github_token must not emit or document an "unknown" login.
    let src = include_str!("handlers.rs");
    assert!(!src.contains("\"unknown\""), "handlers.rs references \"unknown\"");
}

#[test]
fn middleware_source_contains_no_unknown_placeholder_literal_or_constant() {
    let src = include_str!("middleware.rs");
    assert!(!src.contains("\"unknown\""), "middleware.rs references \"unknown\"");
    assert!(!src.contains("LOGINLESS_SUBJECT"));
}

#[tokio::test]
async fn token_without_sub_and_without_pid_is_401_missing_principal() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({})))).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn token_with_null_sub_and_no_pid_is_401_missing_principal() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({ "sub": null })))).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn token_with_empty_sub_and_no_pid_is_401_missing_principal() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({ "sub": "" })))).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn token_with_empty_sub_and_empty_pid_is_401_missing_principal() {
    let (s, v) = call(&forge(claims_with(
        serde_json::json!({ "sub": "", "pid": "" }),
    )))
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn token_with_no_sub_and_empty_pid_is_401_missing_principal() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({ "pid": "" })))).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    assert_eq!(v["error"]["code"], "MissingPrincipal", "{v}");
}

#[tokio::test]
async fn pid_only_token_is_a_loginless_principal() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({ "pid": "inst-77" })))).await;
    assert_eq!(s, StatusCode::OK);
    assert!(v["user_login"].is_null(), "{v}");
    assert_eq!(v["principal_id"], "inst-77");
}

#[tokio::test]
async fn pid_wins_over_sub_when_both_are_present() {
    let (s, v) = call(&forge(claims_with(
        serde_json::json!({ "sub": "alice", "pid": "inst-77" }),
    )))
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(v["user_login"].is_null(), "{v}");
    assert_eq!(v["principal_id"], "inst-77");
}

#[tokio::test]
async fn sub_only_token_named_unknown_is_a_real_login() {
    let (s, v) = call(&forge(claims_with(serde_json::json!({ "sub": "unknown" })))).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["user_login"], "unknown");
    assert!(v["principal_id"].is_null());
}

#[tokio::test]
async fn sub_with_empty_pid_falls_back_to_the_login() {
    let (s, v) = call(&forge(claims_with(
        serde_json::json!({ "sub": "alice", "pid": "" }),
    )))
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["user_login"], "alice");
    assert!(v["principal_id"].is_null());
}

// --- AuthError::MissingPrincipalIdentifier response contract ---------------

async fn render(e: AuthError) -> (StatusCode, serde_json::Value, String) {
    use axum::response::IntoResponse;
    let resp = e.into_response();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    (
        status,
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null),
        text,
    )
}

#[tokio::test]
async fn missing_principal_error_is_401() {
    let (s, _, _) = render(AuthError::MissingPrincipalIdentifier).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_principal_error_has_its_own_code() {
    let (_, v, _) = render(AuthError::MissingPrincipalIdentifier).await;
    assert_eq!(v["error"]["code"], "MissingPrincipal");
}

#[tokio::test]
async fn missing_principal_code_differs_from_missing_token_code() {
    let (_, a, _) = render(AuthError::MissingPrincipalIdentifier).await;
    let (_, b, _) = render(AuthError::MissingToken).await;
    assert_ne!(a["error"]["code"], b["error"]["code"]);
}

#[tokio::test]
async fn missing_principal_message_does_not_tell_the_client_to_provide_a_bearer_token() {
    let (_, v, text) = render(AuthError::MissingPrincipalIdentifier).await;
    let msg = v["error"]["message"].as_str().unwrap().to_lowercase();
    assert!(!msg.is_empty());
    assert!(!msg.contains("bearer"), "{msg}");
    assert!(!msg.contains("provide a valid"), "{msg}");
    assert!(!text.to_lowercase().contains("bearer"), "{text}");
    assert!(!text.contains("Authorization"), "{text}");
}

#[tokio::test]
async fn missing_principal_message_says_the_token_lacks_an_identity() {
    let (_, v, _) = render(AuthError::MissingPrincipalIdentifier).await;
    let msg = v["error"]["message"].as_str().unwrap().to_lowercase();
    assert!(
        msg.contains("principal") || msg.contains("identif"),
        "message should explain the identity problem: {msg}"
    );
}

#[tokio::test]
async fn missing_token_response_is_unchanged() {
    let (_, v, _) = render(AuthError::MissingToken).await;
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Provide a valid Bearer token"));
    assert_eq!(v["error"]["code"], "AuthenticationError");
}

#[tokio::test]
async fn missing_principal_response_does_not_leak_secret_or_claim_names() {
    let (_, _, text) = render(AuthError::MissingPrincipalIdentifier).await;
    assert!(!text.contains(crate::TEST_JWT_SECRET));
    assert!(!text.contains("\"pid\""));
    assert!(!text.contains("\"sub\""));
}
