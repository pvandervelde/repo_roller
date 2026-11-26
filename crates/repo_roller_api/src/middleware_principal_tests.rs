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
    assert_eq!(claims.sub, "bob");
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
