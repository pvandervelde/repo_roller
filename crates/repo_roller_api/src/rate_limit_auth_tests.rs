//! Unit tests for the token-exchange (public endpoint) limiter pieces and the
//! principal-key derivation: auth config parsing, `client_key`, `principal_key`.

use super::*;
use axum::body::Body;
use axum::extract::ConnectInfo;
use std::collections::HashMap;
use std::net::SocketAddr;

fn cfg(pairs: &[(&str, &str)]) -> RateLimitConfig {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    RateLimitConfig::auth_from_env_vars(|k| map.get(k).cloned())
}

fn req(addr: Option<&str>, headers: &[(&str, &str)]) -> Request {
    let mut b = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/token");
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    let mut r = b.body(Body::empty()).unwrap();
    if let Some(a) = addr {
        let sa: SocketAddr = a.parse().unwrap();
        r.extensions_mut().insert(ConnectInfo(sa));
    }
    r
}

fn ctx(login: Option<&str>, pid: Option<&str>) -> AuthContext {
    AuthContext {
        user_login: login.map(str::to_string),
        principal_id: pid.map(str::to_string),
    }
}

// ---------------------------------------------------------------------------
// Auth limiter config
// ---------------------------------------------------------------------------

#[test]
fn auth_defaults_are_20_requests_per_60_seconds_enabled() {
    let c = cfg(&[]);
    assert!(c.enabled);
    assert_eq!(c.max_requests, 20);
    assert_eq!(c.window, Duration::from_secs(60));
    assert_eq!(DEFAULT_AUTH_MAX_REQUESTS, 20);
    assert_eq!(DEFAULT_AUTH_WINDOW_SECS, 60);
}

#[test]
fn auth_values_are_parsed_from_dedicated_variables() {
    let c = cfg(&[
        ("RATE_LIMIT_AUTH_REQUESTS", "5"),
        ("RATE_LIMIT_AUTH_WINDOW_SECS", "30"),
    ]);
    assert_eq!(c.max_requests, 5);
    assert_eq!(c.window, Duration::from_secs(30));
}

#[test]
fn auth_config_ignores_the_per_user_variables() {
    let c = cfg(&[("RATE_LIMIT_REQUESTS", "3"), ("RATE_LIMIT_WINDOW_SECS", "7")]);
    assert_eq!(c.max_requests, 20);
    assert_eq!(c.window, Duration::from_secs(60));
}

#[test]
fn per_user_config_ignores_the_auth_variables() {
    let map: HashMap<String, String> = [
        ("RATE_LIMIT_AUTH_REQUESTS", "3"),
        ("RATE_LIMIT_AUTH_WINDOW_SECS", "7"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let c = RateLimitConfig::from_env_vars(|k| map.get(k).cloned());
    assert_eq!(c.max_requests, DEFAULT_MAX_REQUESTS);
    assert_eq!(c.window, Duration::from_secs(DEFAULT_WINDOW_SECS));
}

#[test]
fn rate_limit_enabled_false_disables_the_auth_limiter() {
    assert!(!cfg(&[("RATE_LIMIT_ENABLED", "false")]).enabled);
    assert!(!cfg(&[("RATE_LIMIT_ENABLED", "0")]).enabled);
    assert!(!cfg(&[("RATE_LIMIT_ENABLED", "FALSE")]).enabled);
}

#[test]
fn garbage_enabled_value_fails_closed_to_enabled_for_auth() {
    assert!(cfg(&[("RATE_LIMIT_ENABLED", "maybe")]).enabled);
    assert!(cfg(&[("RATE_LIMIT_ENABLED", "")]).enabled);
}

#[test]
fn invalid_auth_numbers_fall_back_to_defaults() {
    for bad in ["0", "-1", "abc", "", "1.5", "99999999999999999999"] {
        let c = cfg(&[
            ("RATE_LIMIT_AUTH_REQUESTS", bad),
            ("RATE_LIMIT_AUTH_WINDOW_SECS", bad),
        ]);
        assert_eq!(c.max_requests, 20, "requests for {bad:?}");
        assert_eq!(c.window, Duration::from_secs(60), "window for {bad:?}");
    }
}

#[test]
fn auth_requests_and_window_fall_back_independently() {
    let c = cfg(&[
        ("RATE_LIMIT_AUTH_REQUESTS", "4"),
        ("RATE_LIMIT_AUTH_WINDOW_SECS", "nope"),
    ]);
    assert_eq!(c.max_requests, 4);
    assert_eq!(c.window, Duration::from_secs(60));
}

// ---------------------------------------------------------------------------
// client_key: connection address only
// ---------------------------------------------------------------------------

#[test]
fn same_ip_with_different_ports_yields_same_key() {
    assert_eq!(
        client_key(&req(Some("203.0.113.7:1111"), &[])),
        client_key(&req(Some("203.0.113.7:2222"), &[]))
    );
}

#[test]
fn different_ips_yield_different_keys() {
    assert_ne!(
        client_key(&req(Some("203.0.113.7:1111"), &[])),
        client_key(&req(Some("203.0.113.8:1111"), &[]))
    );
}

#[test]
fn different_ipv6_addresses_yield_different_keys() {
    assert_ne!(
        client_key(&req(Some("[2001:db8::1]:443"), &[])),
        client_key(&req(Some("[2001:db8::2]:443"), &[]))
    );
}

#[test]
fn ipv4_and_ipv6_keys_differ() {
    assert_ne!(
        client_key(&req(Some("127.0.0.1:80"), &[])),
        client_key(&req(Some("[::1]:80"), &[]))
    );
}

#[test]
fn ipv4_mapped_ipv6_shares_key_with_the_plain_ipv4() {
    assert_eq!(
        client_key(&req(Some("[::ffff:203.0.113.7]:80"), &[])),
        client_key(&req(Some("203.0.113.7:80"), &[]))
    );
}

#[test]
fn forwarding_headers_never_change_the_key() {
    let base = client_key(&req(Some("203.0.113.7:1111"), &[]));
    for (h, v) in [
        ("x-forwarded-for", "198.51.100.1"),
        ("x-forwarded-for", "198.51.100.1, 198.51.100.2"),
        ("forwarded", "for=198.51.100.1"),
        ("x-real-ip", "198.51.100.1"),
        ("x-forwarded-host", "evil.example"),
        ("true-client-ip", "198.51.100.1"),
        ("cf-connecting-ip", "198.51.100.1"),
    ] {
        assert_eq!(
            client_key(&req(Some("203.0.113.7:1111"), &[(h, v)])),
            base,
            "header {h} must not influence the key"
        );
    }
}

#[test]
fn forwarding_header_without_connect_info_does_not_create_an_ip_key() {
    assert_eq!(
        client_key(&req(None, &[("x-forwarded-for", "198.51.100.1")])),
        NO_CLIENT_ADDR_KEY
    );
}

#[test]
fn missing_connect_info_uses_the_documented_fallback_key() {
    assert_eq!(client_key(&req(None, &[])), NO_CLIENT_ADDR_KEY);
    assert!(!NO_CLIENT_ADDR_KEY.is_empty());
}

#[test]
fn a_real_client_never_collides_with_the_fallback_key() {
    for a in ["203.0.113.7:1", "[::1]:1", "0.0.0.0:1", "255.255.255.255:1"] {
        assert_ne!(client_key(&req(Some(a), &[])), NO_CLIENT_ADDR_KEY, "{a}");
    }
}

#[test]
fn client_key_is_deterministic() {
    let a = client_key(&req(Some("203.0.113.7:1"), &[]));
    let b = client_key(&req(Some("203.0.113.7:1"), &[]));
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// principal_key: namespaced, lowercased, never shared
// ---------------------------------------------------------------------------

#[test]
fn login_keys_are_case_insensitive() {
    assert_eq!(
        principal_key(&ctx(Some("Alice"), None)),
        principal_key(&ctx(Some("ALICE"), None))
    );
    assert_eq!(
        principal_key(&ctx(Some("alice"), None)),
        principal_key(&ctx(Some("aLiCe"), None))
    );
    assert!(principal_key(&ctx(Some("Alice"), None)).is_some());
}

#[test]
fn different_logins_yield_different_keys() {
    assert_ne!(
        principal_key(&ctx(Some("alice"), None)),
        principal_key(&ctx(Some("bob"), None))
    );
}

#[test]
fn principal_ids_yield_distinct_stable_keys() {
    let a1 = principal_key(&ctx(None, Some("inst-a")));
    let a2 = principal_key(&ctx(None, Some("inst-a")));
    let b = principal_key(&ctx(None, Some("inst-b")));
    assert!(a1.is_some());
    assert_eq!(a1, a2);
    assert_ne!(a1, b);
}

#[test]
fn a_user_literally_named_unknown_does_not_collide_with_a_loginless_principal() {
    assert_ne!(
        principal_key(&ctx(Some("unknown"), None)),
        principal_key(&ctx(None, Some("unknown")))
    );
}

#[test]
fn login_and_principal_id_with_same_text_do_not_collide() {
    assert_ne!(
        principal_key(&ctx(Some("abc123"), None)),
        principal_key(&ctx(None, Some("abc123")))
    );
}

#[test]
fn context_with_no_identifier_has_no_key() {
    assert_eq!(principal_key(&ctx(None, None)), None);
}

#[test]
fn empty_login_without_principal_id_has_no_key() {
    assert_eq!(principal_key(&ctx(Some(""), None)), None);
}

#[test]
fn loginless_principals_never_share_the_old_fixed_bucket() {
    let a = principal_key(&ctx(None, Some("x1")));
    let b = principal_key(&ctx(None, Some("x2")));
    assert_ne!(a, b);
    assert_ne!(a.as_deref(), Some("<unknown-user>"));
}

mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn login_case_never_changes_key(login in "[A-Za-z0-9-]{1,39}") {
            prop_assert_eq!(
                principal_key(&ctx(Some(&login), None)),
                principal_key(&ctx(Some(&login.to_uppercase()), None))
            );
        }

        #[test]
        fn login_and_pid_namespaces_never_collide(s in "[A-Za-z0-9-]{1,39}") {
            prop_assert_ne!(
                principal_key(&ctx(Some(&s), None)),
                principal_key(&ctx(None, Some(&s)))
            );
        }

        #[test]
        fn distinct_pids_never_share_a_key(a in "[a-f0-9]{8,64}", b in "[a-f0-9]{8,64}") {
            prop_assume!(a != b);
            prop_assert_ne!(
                principal_key(&ctx(None, Some(&a))),
                principal_key(&ctx(None, Some(&b)))
            );
        }

        #[test]
        fn client_key_depends_only_on_ip(
            a in any::<[u8; 4]>(), p1 in 1u16..=65535, p2 in 1u16..=65535, xff in "[ -~]{0,40}"
        ) {
            let ip = std::net::Ipv4Addr::from(a);
            let r1 = req(Some(&format!("{ip}:{p1}")), &[]);
            let hdr_val = xff.trim().to_string();
            let hdrs: Vec<(&str, &str)> = if hdr_val.is_empty() {
                vec![]
            } else {
                vec![("x-forwarded-for", hdr_val.as_str())]
            };
            let r2 = req(Some(&format!("{ip}:{p2}")), &hdrs);
            prop_assert_eq!(client_key(&r1), client_key(&r2));
        }

        #[test]
        fn distinct_ipv4_never_share_a_client_key(a in any::<[u8; 4]>(), b in any::<[u8; 4]>()) {
            prop_assume!(a != b);
            let ka = client_key(&req(Some(&format!("{}:1", std::net::Ipv4Addr::from(a))), &[]));
            let kb = client_key(&req(Some(&format!("{}:1", std::net::Ipv4Addr::from(b))), &[]));
            prop_assert_ne!(ka, kb);
        }
    }
}
