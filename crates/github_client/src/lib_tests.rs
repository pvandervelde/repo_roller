//! Unit tests for the github_client crate.

use super::*; // Import items from lib.rs
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate}; // For constructing mock bodies

#[ctor::ctor(unsafe)]
fn init_default_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

// --- Test Constants ---
const TEST_APP_ID: u64 = 12345;

// Pre-generated RSA-2048 test key (PKCS8 PEM). For tests only — never use in production.
// Generated with: openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 | openssl pkcs8 -topk8 -nocrypt
const TEST_RSA_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDLRZFg/A26veyg
1H1vIG8FFeTgiShtI+qBQsocdr4Gno6MO2vpdMS7ZXykOemvxfZFo2EBMHxuJYu2
GYK71KRifRILdeYhFJC3f0Szcx9AmjcrHJOzvWzijf2X74cN7Scg63FTup4X4YMF
CJ3r59eMU28Yx2NhsGxmXcPc6xMOKUq1eklT3Xzl4C1EDzEjxYlQXdgElkm8Hcn/
u/GYvzxCJtWYHT3FkDMd0/sxMEccGWU07GeUV5dpYYfheiswfxnUjb2OsrUejCsj
KLuiU/Xj/eIwpaSE48/8l1g+zhqMZqjN/E52rqw47IqK0WQ80fRcPPcrZ/uV/s6n
0FDr5u9RAgMBAAECggEAfUcv9U0DDS+WkfzEHeQoaEL5H6WIw9MZ6XaHaAmKzjuU
cTsCJAzyXuiKbBDBRLLGOF8cHwaJ+bysIZwRZcAFXCnnICcWoRayplUKu2pdPYyO
KcTgByvc2rhWLfjTDLM3pWNWIhtm6iLzg/uukMD/qJ2XX/cq+f8on+J0H0jGIrgY
md4D3HS7k2dxHjcHmV7oLHFv5uc6oUqDCNr5gVdYe5iqA3KBWiba3VT4CtGgPruK
l53S6HPXNjAb4x+y1d4fYoHSkX750MrVsjIT6iAHI0gQMSVqACVBS4K3ck+Og36z
CT3+ERmr2wL5JFCjxOXOxL870uSO5lYN4U4JKVn4kQKBgQDwxLSkSWrQ3N68P0cO
VcgVHW//yFBJe4LDpf2hFUYSssupTaCWm7qx/fuPXiQtUSJl04GgTt+c0a+Y4y96
ccKXhTpidsEnkCvZZ1IHUZcIam3KMenMHtH1XbmEhJSJBKXUs2APjPRFnAOdQ4ia
Nc7skEIqRA0iShEFEodw3F7vbQKBgQDYIZiwzr1iezX0iQlugpOn4iwTZIGrmR8v
BPwTk91KujNNhud7ZSGmSh1oj8igIbvpieOkQH0SCWxD6lK9b8O7mHCjGrj3BDCn
cWS4YhRKIpdHL9MN5qRHcKFhotgDrztIVNZ4rwbdANfJrqJGh08qSPDJ/MgJH8jf
j6txCn989QKBgEgkWJAlY3kjxSWYvDzCXUpeYLG+nrf15y+ibAW5Lx7pLfcTyHt4
PA+bL6Y8qhBeEDV8k4mqOHxvo/Lml9JTheAKzny0YpntLAZfXAfPXbbq1OzR2eEM
lxKzO6RGaY9IsjuyZ6UhMgwtfDPCCYtTdiYR75FkhJ7ynHjtnWGm1lddAoGAW57y
YZFii0r79mvZSbWk7nhcguQKbDn/FCmiypHWL4V7lhPoMVEP0LR+fwLk460pQbko
fUqxOmIwOSIubQviB7Z5mUsMufpWvcS+E8PQIhVD1/OYHr51SaufXgpwVC+GjAUa
3hIkcK+YynBzbj6R+rTtDlNQYT3OI6d93++5V7kCgYEAoE/4vPrDtNyaNi39h3TF
seOmBxWp9ZSiyaDAaJckPop1Y0kqwZNaAe9LXKpYp15PRyD4ehRLNbJLxx6UFHYf
5IN2MmSKZDV9jdJsjelyucVio1J3lyN4JeCRXly11n/2615GiWaZJ6Q4dflDz0iS
JC29NBAKJUVTF8zCccVfB2E=
-----END PRIVATE KEY-----";

fn create_test_pem() -> &'static str {
    TEST_RSA_PRIVATE_KEY_PEM
}

#[tokio::test]
async fn test_create_org_repository_success() {
    let mock_server = MockServer::start().await;
    let org_name = "test-org";
    let payload = RepositoryCreatePayload {
        name: "test-repo".to_string(),
        description: Some("A test repository".to_string()),
        ..Default::default()
    };

    Mock::given(method("POST"))
        .and(path(format!("/orgs/{org_name}/repos")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 123456,
            "name": payload.name,
            "description": payload.description,
            "url": "https://api.github.com/repos/test-org/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.create_org_repository(org_name, &payload).await;

    if let Err(e) = &result {
        eprintln!("create_org_repository error: {e:?}");
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_user_repository_success() {
    let mock_server = MockServer::start().await;
    let payload = RepositoryCreatePayload {
        name: "test-repo".to_string(),
        description: Some("A test repository".to_string()),
        ..Default::default()
    };

    Mock::given(method("POST"))
        .and(path("/user/repos"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 123456,
            "name": payload.name,
            "description": payload.description,
            "url": "https://api.github.com/user/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.create_user_repository(&payload).await;

    if let Err(e) = &result {
        eprintln!("create_user_repository error: {e:?}");
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_repository_success() {
    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "owner": {
                "login": owner,
                "id": 78910,
                "node_id": "MDQ6VXNlcjc4OTEw",
                "avatar_url": "https://avatars.githubusercontent.com/u/78910?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/test-owner",
                "html_url": "https://github.com/test-owner",
                "followers_url": "https://api.github.com/users/test-owner/followers",
                "following_url": "https://api.github.com/users/test-owner/following{/other_user}",
                "gists_url": "https://api.github.com/users/test-owner/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/test-owner/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/test-owner/subscriptions",
                "organizations_url": "https://api.github.com/users/test-owner/orgs",
                "repos_url": "https://api.github.com/users/test-owner/repos",
                "events_url": "https://api.github.com/users/test-owner/events{/privacy}",
                "received_events_url": "https://api.github.com/users/test-owner/received_events",
                "type": "User",
                "site_admin": false,
                "patch_url": null,
                "email": null
            },
            "url": "https://api.github.com/repos/test-owner/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_repository(owner, repo).await;

    if let Err(e) = &result {
        eprintln!("get_repository error: {e:?}");
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_repository_settings_success() {
    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";
    let settings = RepositorySettingsUpdate {
        description: Some("Updated description".to_string()),
        ..Default::default()
    };

    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "description": settings.description,
            "url": "https://api.github.com/repos/test-owner/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .update_repository_settings(owner, repo, &settings)
        .await;

    if let Err(e) = &result {
        eprintln!("update_repository_settings error: {e:?}");
    }
    assert!(result.is_ok());
}

// ============================================================================
// RepositorySettingsUpdate.default_branch Tests (RED: #255 follow-up Gap 2 --
// "history mirror never sets the new repo's default branch on GitHub")
// ============================================================================
//
// `create_history_mirror_repository` (repo_roller_core) needs to align a
// newly created history-mirror repository's recorded `default_branch` with
// the source's default branch via `update_repository_settings`. GitHub's
// `PATCH /repos/{owner}/{repo}` endpoint accepts `default_branch` natively,
// so this is a pure additive field on `RepositorySettingsUpdate` -- no new
// client method needed. These tests pin down its serde contract and that
// `update_repository_settings` actually transmits it.

#[test]
fn test_repository_settings_update_default_branch_serializes_when_present() {
    let settings = RepositorySettingsUpdate {
        default_branch: Some("develop".to_string()),
        ..Default::default()
    };

    let value = serde_json::to_value(&settings).expect("RepositorySettingsUpdate must serialize");

    assert_eq!(
        value.get("default_branch"),
        Some(&json!("develop")),
        "default_branch must be present in the serialized payload when set: {value:?}"
    );
}

#[test]
fn test_repository_settings_update_default_branch_omitted_when_none() {
    let settings = RepositorySettingsUpdate {
        description: Some("unrelated field set".to_string()),
        ..Default::default()
    };

    let value = serde_json::to_value(&settings).expect("RepositorySettingsUpdate must serialize");

    assert!(
        value.get("default_branch").is_none(),
        "default_branch must be omitted (skip_serializing_if) when None, got: {value:?}"
    );
}

#[test]
fn test_repository_settings_update_default_branch_set_does_not_affect_unrelated_fields() {
    // Adversarial: setting default_branch must not accidentally also set or
    // suppress unrelated fields (catches a copy-paste mutation that aliases
    // fields, or a serde attribute applied to the wrong field).
    let settings = RepositorySettingsUpdate {
        default_branch: Some("trunk".to_string()),
        ..Default::default()
    };

    let value = serde_json::to_value(&settings).expect("RepositorySettingsUpdate must serialize");

    assert_eq!(value.get("default_branch"), Some(&json!("trunk")));
    assert!(value.get("description").is_none());
    assert!(value.get("homepage").is_none());
    assert!(value.get("private").is_none());
    assert!(value.get("has_issues").is_none());
    assert!(value.get("has_projects").is_none());
    assert!(value.get("has_wiki").is_none());
}

#[test]
fn test_repository_settings_update_default_branch_distinct_values_serialize_distinctly() {
    // Stub-killing: a hardcoded "main" in the (de)serialization path would
    // fail this test even though it might pass the "develop" test above.
    let settings = RepositorySettingsUpdate {
        default_branch: Some("release/2026.1".to_string()),
        ..Default::default()
    };

    let value = serde_json::to_value(&settings).expect("RepositorySettingsUpdate must serialize");

    assert_eq!(value.get("default_branch"), Some(&json!("release/2026.1")));
}

#[tokio::test]
async fn test_update_repository_settings_sends_default_branch_in_patch_body() {
    use wiremock::matchers::body_partial_json;

    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";
    let settings = RepositorySettingsUpdate {
        default_branch: Some("develop".to_string()),
        ..Default::default()
    };

    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}")))
        .and(body_partial_json(json!({ "default_branch": "develop" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "default_branch": "develop",
            "url": format!("https://api.github.com/repos/{owner}/{repo}")
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .update_repository_settings(owner, repo, &settings)
        .await;

    if let Err(e) = &result {
        eprintln!("update_repository_settings error: {e:?}");
    }
    assert!(
        result.is_ok(),
        "update_repository_settings with default_branch set must succeed"
    );

    mock_server.verify().await; // wiremock asserts the body_partial_json-matched PATCH was received exactly once
}

#[tokio::test]
async fn test_update_repository_settings_without_default_branch_does_not_send_it() {
    use wiremock::matchers::body_partial_json;

    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";
    let settings = RepositorySettingsUpdate {
        description: Some("no branch change here".to_string()),
        ..Default::default()
    };

    // A mock that would only match if `default_branch` were present in the
    // body must never be hit when the caller didn't set it.
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}")))
        .and(body_partial_json(json!({ "default_branch": "main" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "url": format!("https://api.github.com/repos/{owner}/{repo}")
        })))
        .expect(0)
        .mount(&mock_server)
        .await;

    // A permissive mock handles the actual (default_branch-less) request so
    // the call still completes successfully.
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "url": format!("https://api.github.com/repos/{owner}/{repo}")
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .update_repository_settings(owner, repo, &settings)
        .await;

    assert!(result.is_ok());
    mock_server.verify().await; // wiremock asserts the default_branch-matching mock's expect(0) held
}

#[tokio::test]
#[ignore = "Integration test - requires real GitHub App setup. octocrab models need complex mock data structure"]
async fn test_list_installations_success() {
    let mock_server = MockServer::start().await;

    // Mock the installations endpoint
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 12345,
                "account": {
                    "id": 1,
                    "login": "test-org",
                    "type": "Organization",
                    "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                    "avatar_url": "https://github.com/images/error/test-org_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/orgs/test-org",
                    "html_url": "https://github.com/test-org",
                    "followers_url": "https://api.github.com/orgs/test-org/followers",
                    "following_url": "https://api.github.com/orgs/test-org/following{/other_user}",
                    "gists_url": "https://api.github.com/orgs/test-org/gists{/gist_id}",
                    "starred_url": "https://api.github.com/orgs/test-org/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/orgs/test-org/subscriptions",
                    "organizations_url": "https://api.github.com/orgs/test-org/orgs",
                    "repos_url": "https://api.github.com/orgs/test-org/repos",
                    "events_url": "https://api.github.com/orgs/test-org/events{/privacy}",
                    "received_events_url": "https://api.github.com/orgs/test-org/received_events",
                    "site_admin": false
                },
                "repository_selection": "selected",
                "node_id": "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uMTIzNDU="
            },
            {
                "id": 67890,
                "account": {
                    "id": 2,
                    "login": "another-org",
                    "type": "Organization",
                    "node_id": "MDEyOk9yZ2FuaXphdGlvbjI=",
                    "avatar_url": "https://github.com/images/error/another-org_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/orgs/another-org",
                    "html_url": "https://github.com/another-org",
                    "followers_url": "https://api.github.com/orgs/another-org/followers",
                    "following_url": "https://api.github.com/orgs/another-org/following{/other_user}",
                    "gists_url": "https://api.github.com/orgs/another-org/gists{/gist_id}",
                    "starred_url": "https://api.github.com/orgs/another-org/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/orgs/another-org/subscriptions",
                    "organizations_url": "https://api.github.com/orgs/another-org/orgs",
                    "repos_url": "https://api.github.com/orgs/another-org/repos",
                    "events_url": "https://api.github.com/orgs/another-org/events{/privacy}",
                    "received_events_url": "https://api.github.com/orgs/another-org/received_events",
                    "site_admin": false
                },
                "repository_selection": "all",
                "node_id": "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uNjc4OTA="
            }
        ])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_installations().await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in list_installations: {e:?}");
            panic!("Expected Ok, got Err: {e:?}");
        }
    }
    let installations = result.unwrap();
    assert_eq!(installations.len(), 2);
    assert_eq!(installations[0].id, 12345);
    assert_eq!(installations[0].account.login, "test-org");
    assert_eq!(installations[1].id, 67890);
    assert_eq!(installations[1].account.login, "another-org");
}

#[tokio::test]
async fn test_list_installations_empty() {
    let mock_server = MockServer::start().await;

    // Mock empty installations response
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_installations().await;

    assert!(result.is_ok());
    let installations = result.unwrap();
    assert_eq!(installations.len(), 0);
}

#[tokio::test]
#[ignore = "Integration test - requires real GitHub App setup. octocrab models need complex mock data structure"]
async fn test_get_installation_token_for_org_success() {
    let mock_server = MockServer::start().await;
    let org_name = "test-org";
    let installation_id = 12345;
    let test_token = "ghs_test_token_12345";

    // Mock the installations endpoint
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": installation_id,
                "account": {
                    "id": 1,
                    "login": org_name,
                    "type": "Organization",
                    "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                    "avatar_url": "https://github.com/images/error/test-org_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/orgs/test-org",
                    "html_url": "https://github.com/test-org",
                    "followers_url": "https://api.github.com/orgs/test-org/followers",
                    "following_url": "https://api.github.com/orgs/test-org/following{/other_user}",
                    "gists_url": "https://api.github.com/orgs/test-org/gists{/gist_id}",
                    "starred_url": "https://api.github.com/orgs/test-org/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/orgs/test-org/subscriptions",
                    "organizations_url": "https://api.github.com/orgs/test-org/orgs",
                    "repos_url": "https://api.github.com/orgs/test-org/repos",
                    "events_url": "https://api.github.com/orgs/test-org/events{/privacy}",
                    "received_events_url": "https://api.github.com/orgs/test-org/received_events",
                    "site_admin": false
                },
                "repository_selection": "selected",
                "node_id": "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uMTIzNDU="
            }
        ])))
        .mount(&mock_server)
        .await;

    // Mock the installation token endpoint
    Mock::given(method("POST"))
        .and(path(format!(
            "/app/installations/{installation_id}/access_tokens"
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "token": test_token,
            "expires_at": "2025-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_installation_token_for_org(org_name).await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in get_installation_token_for_org: {e:?}");
            panic!("Expected Ok, got Err: {e:?}");
        }
    }
    let token = result.unwrap();
    assert_eq!(token, test_token);
}

#[tokio::test]
async fn test_get_installation_token_for_org_not_found() {
    let mock_server = MockServer::start().await;
    let org_name = "nonexistent-org";

    // Mock empty installations response (org not found)
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 12345,
                "account": {
                    "id": 1,
                    "login": "different-org",
                    "type": "Organization",
                    "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                    "avatar_url": "https://github.com/images/error/different-org_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/orgs/different-org",
                    "html_url": "https://github.com/different-org",
                    "followers_url": "https://api.github.com/orgs/different-org/followers",
                    "following_url": "https://api.github.com/orgs/different-org/following{/other_user}",
                    "gists_url": "https://api.github.com/orgs/different-org/gists{/gist_id}",
                    "starred_url": "https://api.github.com/orgs/different-org/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/orgs/different-org/subscriptions",
                    "organizations_url": "https://api.github.com/orgs/different-org/orgs",
                    "repos_url": "https://api.github.com/orgs/different-org/repos",
                    "events_url": "https://api.github.com/orgs/different-org/events{/privacy}",
                    "received_events_url": "https://api.github.com/orgs/different-org/received_events",
                    "site_admin": false
                },
                "repository_selection": "selected",
                "node_id": "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uMTIzNDU="
            }
        ])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_installation_token_for_org(org_name).await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "Integration test - requires real GitHub App setup. octocrab models need complex mock data structure"]
async fn test_get_installation_token_case_insensitive() {
    let mock_server = MockServer::start().await;
    let org_name_lower = "test-org";
    let org_name_upper = "TEST-ORG";
    let installation_id = 12345;
    let test_token = "ghs_test_token_12345";

    // Mock with lowercase org name
    Mock::given(method("GET"))
        .and(path("/app/installations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": installation_id,
                "account": {
                    "id": 1,
                    "login": org_name_lower,
                    "type": "Organization",
                    "node_id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                    "avatar_url": "https://github.com/images/error/test-org_happy.gif",
                    "gravatar_id": "",
                    "url": "https://api.github.com/orgs/test-org",
                    "html_url": "https://github.com/test-org",
                    "followers_url": "https://api.github.com/orgs/test-org/followers",
                    "following_url": "https://api.github.com/orgs/test-org/following{/other_user}",
                    "gists_url": "https://api.github.com/orgs/test-org/gists{/gist_id}",
                    "starred_url": "https://api.github.com/orgs/test-org/starred{/owner}{/repo}",
                    "subscriptions_url": "https://api.github.com/orgs/test-org/subscriptions",
                    "organizations_url": "https://api.github.com/orgs/test-org/orgs",
                    "repos_url": "https://api.github.com/orgs/test-org/repos",
                    "events_url": "https://api.github.com/orgs/test-org/events{/privacy}",
                    "received_events_url": "https://api.github.com/orgs/test-org/received_events",
                    "site_admin": false
                },
                "repository_selection": "selected",
                "node_id": "MDIzOkludGVncmF0aW9uSW5zdGFsbGF0aW9uMTIzNDU="
            }
        ])))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path(format!(
            "/app/installations/{installation_id}/access_tokens"
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "token": test_token,
            "expires_at": "2025-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    // Test that uppercase org name finds lowercase match
    let result = client.get_installation_token_for_org(org_name_upper).await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in get_installation_token_case_insensitive: {e:?}");
            panic!("Expected Ok, got Err: {e:?}");
        }
    }
    let token = result.unwrap();
    assert_eq!(token, test_token);
}

/// Verify that set_repository_custom_properties sends correct API request.
///
/// Tests that the method properly formats the PATCH request to GitHub's
/// custom properties endpoint with the correct payload structure.
#[tokio::test]
async fn test_set_repository_custom_properties_success() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    let payload = CustomPropertiesPayload::new(vec![
        json!({
            "property_name": "repository_type",
            "value": "library"
        }),
        json!({
            "property_name": "team",
            "value": "backend"
        }),
    ]);

    // GitHub API endpoint for custom properties
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/custom-properties")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({}))) // 200 OK with empty JSON body
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_repository_custom_properties(owner, repo, &payload)
        .await;

    assert!(
        result.is_ok(),
        "Setting custom properties should succeed, got: {:?}",
        result
    );
}

/// Verify that setting custom properties handles API errors correctly.
#[tokio::test]
async fn test_set_repository_custom_properties_api_error() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    let payload = CustomPropertiesPayload::new(vec![json!({
        "property_name": "invalid_property",
        "value": "test"
    })]);

    // GitHub API returns 422 if property doesn't exist
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/custom-properties")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "Custom property not found",
            "errors": [
                {
                    "resource": "CustomProperty",
                    "code": "not_found",
                    "field": "property_name"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_repository_custom_properties(owner, repo, &payload)
        .await;

    assert!(
        result.is_err(),
        "Setting invalid custom property should fail"
    );
}

/// Verify that setting empty custom properties list succeeds.
#[tokio::test]
async fn test_set_repository_custom_properties_empty() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    let payload = CustomPropertiesPayload::new(vec![]);

    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/custom-properties")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_repository_custom_properties(owner, repo, &payload)
        .await;

    assert!(
        result.is_ok(),
        "Setting empty custom properties should succeed"
    );
}

/// Verify that setting custom properties handles permission errors.
#[tokio::test]
async fn test_set_repository_custom_properties_permission_denied() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    let payload = CustomPropertiesPayload::new(vec![json!({
        "property_name": "repository_type",
        "value": "library"
    })]);

    // GitHub API returns 403 if app lacks permissions
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/custom-properties")))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "message": "Resource not accessible by integration",
            "documentation_url": "https://docs.github.com/rest/repos/custom-properties"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_repository_custom_properties(owner, repo, &payload)
        .await;

    assert!(
        result.is_err(),
        "Setting custom properties without permission should fail"
    );
}

/// Verify that setting custom properties handles repository not found.
#[tokio::test]
async fn test_set_repository_custom_properties_repo_not_found() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "nonexistent-repo";

    let payload = CustomPropertiesPayload::new(vec![json!({
        "property_name": "repository_type",
        "value": "library"
    })]);

    // GitHub API returns 404 if repository doesn't exist
    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/custom-properties")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_repository_custom_properties(owner, repo, &payload)
        .await;

    assert!(
        result.is_err(),
        "Setting custom properties on nonexistent repo should fail"
    );
}

// --- Tests for search_repositories_by_topic ---
//
// Note: search_repositories_by_topic() is a thin wrapper over search_repositories(),
// constructing a formatted query string. Complex mocking of octocrab's search API
// with GitHub App authentication is not worthwhile for such a simple function.
//
// Full integration tests against real GitHub API are in:
// crates/integration_tests/tests/github_metadata_provider_test.rs
//
// The wrapper is tested by verifying:
// 1. It compiles with correct signature
// 2. Integration tests confirm it works against real GitHub
// 3. The underlying search_repositories() is already production-tested

// --- Tests for list_directory_contents ---
//
// Note: Mixed file/directory listing test is covered by integration tests
// in crates/integration_tests/tests/directory_listing_tests.rs which test
// against real GitHub API. See test_filter_directories_from_mixed_entries()
// and test_list_directory_contents_types_directory() for full coverage.

/// Test listing an empty directory.
///
/// Verifies that an empty directory returns an empty vector (not an error).
#[tokio::test]
async fn test_list_directory_contents_empty_directory() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let dir_path = "empty-dir";
    let branch = "main";

    // Mock GitHub Contents API response for empty directory
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/contents/{dir_path}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .list_directory_contents(owner, repo, dir_path, branch)
        .await;

    assert!(result.is_ok(), "Empty directory should return Ok");
    let entries = result.unwrap();
    assert_eq!(entries.len(), 0, "Empty directory should have no entries");
}

/// Test listing a path that doesn't exist.
///
/// Verifies that a 404 response is mapped to Error::NotFound.
#[tokio::test]
async fn test_list_directory_contents_path_not_found() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let dir_path = "nonexistent";
    let branch = "main";

    // Mock GitHub Contents API 404 response
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/contents/{dir_path}")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest/repos/contents#get-repository-content"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .list_directory_contents(owner, repo, dir_path, branch)
        .await;

    assert!(result.is_err(), "Should return error for non-existent path");
    assert!(
        matches!(result.unwrap_err(), Error::NotFound),
        "Should return NotFound error for 404 response"
    );
}

/// Test listing a path that is a file, not a directory.
///
/// Verifies that when the path points to a file (GitHub returns an object instead
/// of an array), we return Error::InvalidResponse.
#[tokio::test]
async fn test_list_directory_contents_path_is_file() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let file_path = "README.md";
    let branch = "main";

    // Mock GitHub Contents API response for a file (single object, not array)
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/contents/{file_path}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "README.md",
            "path": "README.md",
            "type": "file",
            "sha": "abc123",
            "size": 1234,
            "download_url": "https://raw.githubusercontent.com/test-org/test-repo/main/README.md",
            "content": "SGVsbG8gV29ybGQ="
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .list_directory_contents(owner, repo, file_path, branch)
        .await;

    assert!(result.is_err(), "Should return error when path is a file");
    assert!(
        matches!(result.unwrap_err(), Error::InvalidResponse),
        "Should return InvalidResponse when path is file, not directory"
    );
}

/// Test authentication error (401 Unauthorized).
///
/// Verifies that authentication failures are mapped to Error::AuthError.
#[tokio::test]
async fn test_list_directory_contents_auth_error() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let dir_path = "types";
    let branch = "main";

    // Mock GitHub Contents API 401 response
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/contents/{dir_path}")))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "message": "Bad credentials",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .list_directory_contents(owner, repo, dir_path, branch)
        .await;

    assert!(result.is_err(), "Should return error for auth failure");
    assert!(
        matches!(result.unwrap_err(), Error::AuthError(_)),
        "Should return AuthError for 401 response"
    );
}

/// Test rate limit exceeded error.
///
/// Verifies that rate limit errors are detected and mapped to Error::RateLimitExceeded.
#[tokio::test]
async fn test_list_directory_contents_rate_limit() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let dir_path = "types";
    let branch = "main";

    // Mock GitHub Contents API 403 response with rate limit headers
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/contents/{dir_path}")))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(json!({
                    "message": "API rate limit exceeded",
                    "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
                }))
                .insert_header("X-RateLimit-Remaining", "0")
                .insert_header("X-RateLimit-Reset", "1234567890"),
        )
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .list_directory_contents(owner, repo, dir_path, branch)
        .await;

    assert!(result.is_err(), "Should return error for rate limit");
    assert!(
        matches!(result.unwrap_err(), Error::RateLimitExceeded),
        "Should return RateLimitExceeded for 403 with rate limit"
    );
}

// --- Webhook & Label Method Tests ---
// NOTE: Direct octocrab.get/post/patch/delete tests removed due to mock server
// incompatibility with GitHub App authentication. These methods will be tested
// via integration tests against real GitHub API.

// ============================================================================
// Team API Method Tests
// ============================================================================

/// Test listing organization teams with a single page of results.
#[tokio::test]
async fn test_list_organization_teams_returns_all_teams() {
    let mock_server = MockServer::start().await;
    let org = "test-org";

    // First page returns 2 teams; second page (empty) ends pagination.
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "slug": "backend", "name": "Backend", "description": "The backend team"},
            {"id": 2, "slug": "frontend", "name": "Frontend", "description": null}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_organization_teams(org).await;

    if let Err(e) = &result {
        eprintln!("list_organization_teams error: {e:?}");
    }
    let teams = result.expect("Expected Ok result");
    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].slug, "backend");
    assert_eq!(teams[0].name, "Backend");
    assert_eq!(teams[0].description, Some("The backend team".to_string()));
    assert_eq!(teams[1].slug, "frontend");
    assert!(teams[1].description.is_none());
}

/// Test that list_organization_teams paginates across multiple pages.
#[tokio::test]
async fn test_list_organization_teams_paginates_multiple_pages() {
    let mock_server = MockServer::start().await;
    let org = "test-org";

    // Page 1: 2 teams
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "slug": "alpha", "name": "Alpha"},
            {"id": 2, "slug": "beta",  "name": "Beta"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 1 team
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 3, "slug": "gamma", "name": "Gamma"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3 (empty): ends pagination
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_organization_teams(org).await;

    if let Err(e) = &result {
        eprintln!("list_organization_teams pagination error: {e:?}");
    }
    let teams = result.expect("Expected Ok result for pagination");
    assert_eq!(teams.len(), 3, "Should have collected teams from all pages");
    assert_eq!(teams[0].slug, "alpha");
    assert_eq!(teams[1].slug, "beta");
    assert_eq!(teams[2].slug, "gamma");
}

/// Test that list_organization_teams returns an empty vec when the org has no teams.
#[tokio::test]
async fn test_list_organization_teams_returns_empty_when_no_teams() {
    let mock_server = MockServer::start().await;
    let org = "test-org";

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_organization_teams(org).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "Expected empty team list");
}

/// Test that list_organization_teams returns an error on API failure.
#[tokio::test]
async fn test_list_organization_teams_returns_error_on_api_failure() {
    let mock_server = MockServer::start().await;
    let org = "test-org";

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams")))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_organization_teams(org).await;

    assert!(result.is_err(), "Expected error on API failure");
}

/// Test listing team members with a single page of results.
#[tokio::test]
async fn test_get_team_members_returns_all_members() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "backend-engineers";

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 101, "login": "alice"},
            {"id": 102, "login": "bob"},
            {"id": 103, "login": "carol"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_team_members(org, team_slug).await;

    if let Err(e) = &result {
        eprintln!("get_team_members error: {e:?}");
    }
    let members = result.expect("Expected Ok result");
    assert_eq!(members.len(), 3);
    assert_eq!(members[0].login, "alice");
    assert_eq!(members[1].login, "bob");
    assert_eq!(members[2].login, "carol");
}

/// Test that get_team_members paginates across multiple pages.
#[tokio::test]
async fn test_get_team_members_paginates_multiple_pages() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "large-team";

    // Page 1: 2 members
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "login": "user1"},
            {"id": 2, "login": "user2"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 1 member
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 3, "login": "user3"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 3 (empty): ends pagination
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_team_members(org, team_slug).await;

    if let Err(e) = &result {
        eprintln!("get_team_members pagination error: {e:?}");
    }
    let members = result.expect("Expected Ok result for pagination");
    assert_eq!(
        members.len(),
        3,
        "Should have collected members from all pages"
    );
}

/// Test that get_team_members returns an empty vec when the team has no members.
#[tokio::test]
async fn test_get_team_members_returns_empty_when_no_members() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "empty-team";

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_team_members(org, team_slug).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "Expected empty member list");
}

/// Test that get_team_members returns an error when the team does not exist (404).
#[tokio::test]
async fn test_get_team_members_returns_error_on_not_found() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "nonexistent-team";

    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/teams/{team_slug}/members")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_team_members(org, team_slug).await;

    assert!(result.is_err(), "Expected error for non-existent team");
}

/// Test that add_team_to_repository succeeds on 204 No Content.
#[tokio::test]
async fn test_add_team_to_repository_succeeds() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "backend-engineers";
    let repo = "my-service";
    let permission = "push";

    // GitHub returns 204 No Content with an empty body on success.
    Mock::given(method("PUT"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{org}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .add_team_to_repository(org, team_slug, repo, permission)
        .await;

    if let Err(e) = &result {
        eprintln!("add_team_to_repository error: {e:?}");
    }
    assert!(result.is_ok(), "Expected Ok on successful team addition");
}

/// Test that add_team_to_repository returns an error on API failure.
#[tokio::test]
async fn test_add_team_to_repository_returns_error_on_api_failure() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "backend-engineers";
    let repo = "my-service";
    let permission = "push";

    Mock::given(method("PUT"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{org}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "Unprocessable Entity",
            "errors": [{"message": "Invalid permission level"}]
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .add_team_to_repository(org, team_slug, repo, permission)
        .await;

    assert!(result.is_err(), "Expected error on API failure");
}

/// Test that set_team_repository_permission succeeds (updates an existing permission).
#[tokio::test]
async fn test_set_team_repository_permission_succeeds() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "backend-engineers";
    let repo_owner = "test-org";
    let repo = "my-service";
    let permission = "maintain";

    // GitHub returns 204 No Content on success (empty body).
    Mock::given(method("PUT"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_team_repository_permission(org, team_slug, repo_owner, repo, permission)
        .await;

    if let Err(e) = &result {
        eprintln!("set_team_repository_permission error: {e:?}");
    }
    assert!(
        result.is_ok(),
        "Expected Ok on successful permission update"
    );
}

/// Test that set_team_repository_permission returns an error when the team/repo does not exist (404).
#[tokio::test]
async fn test_set_team_repository_permission_returns_error_on_not_found() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "nonexistent-team";
    let repo_owner = "test-org";
    let repo = "my-service";
    let permission = "push";

    Mock::given(method("PUT"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_team_repository_permission(org, team_slug, repo_owner, repo, permission)
        .await;

    assert!(
        result.is_err(),
        "Expected error for non-existent team or repository"
    );
}

// --- get_team_repository_permission Tests ---

/// Test that get_team_repository_permission returns the role when the team has access.
#[tokio::test]
async fn test_get_team_repository_permission_returns_role_when_access_exists() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "platform-team";
    let repo_owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1296269,
            "name": "my-service",
            "role_name": "maintain"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_team_repository_permission(org, team_slug, repo_owner, repo)
        .await;

    if let Err(e) = &result {
        eprintln!("get_team_repository_permission error: {e:?}");
    }
    assert!(result.is_ok(), "Expected Ok for team with access");
    assert_eq!(
        result.unwrap(),
        Some("maintain".to_string()),
        "Expected 'maintain' role_name"
    );
}

/// Test that get_team_repository_permission returns None when team has no access (404).
#[tokio::test]
async fn test_get_team_repository_permission_returns_none_when_no_access() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "unknown-team";
    let repo_owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_team_repository_permission(org, team_slug, repo_owner, repo)
        .await;

    assert!(result.is_ok(), "Expected Ok(None) for team without access");
    assert_eq!(
        result.unwrap(),
        None,
        "Expected None when team has no repo access (404)"
    );
}

/// Test that get_team_repository_permission returns Ok(None) on empty body (e.g. 200/204 with no JSON).
///
/// This covers the edge case where GitHub returns an empty response body,
/// which normally produces a serde EOF error. The function treats this as
/// "no permission recorded" rather than a hard failure.
#[tokio::test]
async fn test_get_team_repository_permission_returns_none_on_empty_body() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "platform-team";
    let repo_owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_team_repository_permission(org, team_slug, repo_owner, repo)
        .await;

    assert!(
        result.is_ok(),
        "Expected Ok(None) for empty-body response, got: {result:?}"
    );
    assert_eq!(
        result.unwrap(),
        None,
        "Expected None when response body is empty"
    );
}

/// Test that get_team_repository_permission returns an error on non-404 API failure.
#[tokio::test]
async fn test_get_team_repository_permission_returns_error_on_api_failure() {
    let mock_server = MockServer::start().await;
    let org = "test-org";
    let team_slug = "platform-team";
    let repo_owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!(
            "/orgs/{org}/teams/{team_slug}/repos/{repo_owner}/{repo}"
        )))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_team_repository_permission(org, team_slug, repo_owner, repo)
        .await;

    assert!(result.is_err(), "Expected Err on non-404 API failure");
}

// --- get_repository_team_permission Tests ---

/// Test that get_repository_team_permission returns the role when the team is in the list.
#[tokio::test]
async fn test_get_repository_team_permission_returns_role_when_team_found() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let team_slug = "platform-team";

    // The real GET /repos/{owner}/{repo}/teams response uses `permission` (not
    // `role_name`). Use `permission` in mock JSON to match the real API.
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "slug": "platform-team",
                "name": "Platform Team",
                "permission": "maintain",
                "role_name": null
            },
            {
                "id": 2,
                "slug": "security-team",
                "name": "Security Team",
                "permission": "admin",
                "role_name": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_repository_team_permission(owner, repo, team_slug)
        .await;

    assert!(result.is_ok(), "Expected Ok for team found in list");
    assert_eq!(
        result.unwrap(),
        Some("maintain".to_string()),
        "Expected 'maintain' permission for platform-team"
    );
}

/// Test that get_repository_team_permission normalises legacy permission strings:
/// `pull` → `read` and `push` → `write`.
#[tokio::test]
async fn test_get_repository_team_permission_normalises_pull_and_push() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "slug": "read-team",  "permission": "pull",  "role_name": null },
            { "id": 2, "slug": "write-team", "permission": "push",  "role_name": null },
            { "id": 3, "slug": "triage-team","permission": "triage","role_name": null }
        ])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    // `pull` normalised to `read`
    let r1 = client
        .get_repository_team_permission(owner, repo, "read-team")
        .await;
    assert_eq!(
        r1.unwrap(),
        Some("read".to_string()),
        "pull should normalise to read"
    );

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "slug": "read-team",  "permission": "pull",  "role_name": null },
            { "id": 2, "slug": "write-team", "permission": "push",  "role_name": null },
            { "id": 3, "slug": "triage-team","permission": "triage","role_name": null }
        ])))
        .mount(&mock_server)
        .await;

    // `push` normalised to `write`
    let r2 = client
        .get_repository_team_permission(owner, repo, "write-team")
        .await;
    assert_eq!(
        r2.unwrap(),
        Some("write".to_string()),
        "push should normalise to write"
    );

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "slug": "read-team",  "permission": "pull",  "role_name": null },
            { "id": 2, "slug": "write-team", "permission": "push",  "role_name": null },
            { "id": 3, "slug": "triage-team","permission": "triage","role_name": null }
        ])))
        .mount(&mock_server)
        .await;

    // `triage` passes through unchanged
    let r3 = client
        .get_repository_team_permission(owner, repo, "triage-team")
        .await;
    assert_eq!(
        r3.unwrap(),
        Some("triage".to_string()),
        "triage should pass through unchanged"
    );
}

/// Test that get_repository_team_permission returns None when the team is not in the list.
#[tokio::test]
async fn test_get_repository_team_permission_returns_none_when_team_not_found() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let team_slug = "unknown-team";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "slug": "platform-team",
                "name": "Platform Team",
                "permission": "push",
                "role_name": null
            }
        ])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_repository_team_permission(owner, repo, team_slug)
        .await;

    assert!(
        result.is_ok(),
        "Expected Ok(None) when team absent from list"
    );
    assert_eq!(result.unwrap(), None, "Expected None for unknown-team");
}

/// Test that get_repository_team_permission returns None when the team list is empty.
#[tokio::test]
async fn test_get_repository_team_permission_returns_none_on_empty_list() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let team_slug = "platform-team";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_repository_team_permission(owner, repo, team_slug)
        .await;

    assert!(result.is_ok(), "Expected Ok(None) for empty team list");
    assert_eq!(result.unwrap(), None, "Expected None when list is empty");
}

/// Test that get_repository_team_permission returns an error on API failure.
#[tokio::test]
async fn test_get_repository_team_permission_returns_error_on_api_failure() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let team_slug = "platform-team";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/teams")))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_repository_team_permission(owner, repo, team_slug)
        .await;

    assert!(result.is_err(), "Expected Err on API failure");
}

// --- get_collaborator_permission Tests ---

/// Test that get_collaborator_permission returns the role when the collaborator exists.
#[tokio::test]
async fn test_get_collaborator_permission_returns_role_for_existing_collaborator() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let username = "alice";

    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}/permission"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "permission": "write",
            "role_name": "write",
            "user": { "id": 42, "login": "alice" }
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_collaborator_permission(owner, repo, username)
        .await;

    if let Err(e) = &result {
        eprintln!("get_collaborator_permission error: {e:?}");
    }
    assert!(result.is_ok(), "Expected Ok for existing collaborator");
    assert_eq!(result.unwrap(), "write", "Expected 'write' role_name");
}

/// Test that get_collaborator_permission returns NotFound when user is not a collaborator.
#[tokio::test]
async fn test_get_collaborator_permission_returns_not_found_for_non_collaborator() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let username = "unknown-user";

    Mock::given(method("GET"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}/permission"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_collaborator_permission(owner, repo, username)
        .await;

    assert!(
        matches!(result, Err(Error::NotFound)),
        "Expected NotFound error when user is not a collaborator"
    );
}

// --- Collaborator Method Tests ---

/// Test that list_repository_collaborators returns all collaborators from a single page.
#[tokio::test]
async fn test_list_repository_collaborators_returns_all_collaborators() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";

    // First page returns 2 collaborators; second page (empty) ends pagination.
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "login": "alice"},
            {"id": 2, "login": "bob"}
        ])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_repository_collaborators(owner, repo).await;

    if let Err(e) = &result {
        eprintln!("list_repository_collaborators error: {e:?}");
    }
    let collaborators = result.unwrap();
    assert_eq!(collaborators.len(), 2);
    assert_eq!(collaborators[0].login, "alice");
    assert_eq!(collaborators[1].login, "bob");
}

/// Test that list_repository_collaborators paginates when multiple pages exist.
#[tokio::test]
async fn test_list_repository_collaborators_paginates_multiple_pages() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "big-project";

    // Page 1: full page of 1 item (small per_page for test simplicity)
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .and(wiremock::matchers::query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 10, "login": "user10"}
        ])))
        .mount(&mock_server)
        .await;

    // Page 2: empty → signals end of pagination
    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .and(wiremock::matchers::query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_repository_collaborators(owner, repo).await;

    let collaborators = result.unwrap();
    assert_eq!(collaborators.len(), 1);
    assert_eq!(collaborators[0].login, "user10");
}

/// Test that list_repository_collaborators returns an empty Vec when the repository has no collaborators.
#[tokio::test]
async fn test_list_repository_collaborators_returns_empty_when_none() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "empty-project";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_repository_collaborators(owner, repo).await;

    assert!(result.unwrap().is_empty());
}

/// Test that list_repository_collaborators returns an error on API failure.
#[tokio::test]
async fn test_list_repository_collaborators_returns_error_on_api_failure() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{owner}/{repo}/collaborators")))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_repository_collaborators(owner, repo).await;

    assert!(result.is_err(), "Expected error on API failure");
}

/// Test that add_repository_collaborator succeeds (GitHub returns 204, mocked as 200 + {}).
#[tokio::test]
async fn test_add_repository_collaborator_succeeds() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let username = "newcollab";
    let permission = "push";

    // GitHub returns 204 No Content; mock as 200 + {} to avoid octocrab deserialization error.
    Mock::given(method("PUT"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .add_repository_collaborator(owner, repo, username, permission)
        .await;

    if let Err(e) = &result {
        eprintln!("add_repository_collaborator error: {e:?}");
    }
    assert!(result.is_ok(), "Expected Ok on successful collaborator add");
}

/// Test that add_repository_collaborator returns NotFound when the repository does not exist.
#[tokio::test]
async fn test_add_repository_collaborator_returns_not_found_for_missing_repo() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "nonexistent-repo";
    let username = "someone";
    let permission = "push";

    Mock::given(method("PUT"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .add_repository_collaborator(owner, repo, username, permission)
        .await;

    assert!(
        result.is_err(),
        "Expected error for non-existent repository"
    );
    assert!(
        matches!(result.unwrap_err(), Error::NotFound),
        "Expected NotFound error"
    );
}

/// Test that set_collaborator_permission succeeds (delegates to same PUT endpoint).
#[tokio::test]
async fn test_set_collaborator_permission_succeeds() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let username = "existingcollab";
    let permission = "maintain";

    Mock::given(method("PUT"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .set_collaborator_permission(owner, repo, username, permission)
        .await;

    if let Err(e) = &result {
        eprintln!("set_collaborator_permission error: {e:?}");
    }
    assert!(
        result.is_ok(),
        "Expected Ok on successful permission update"
    );
}

/// Test that remove_repository_collaborator succeeds (GitHub returns 204, mocked as 200 + {}).
#[tokio::test]
async fn test_remove_repository_collaborator_succeeds() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "my-service";
    let username = "leavingcollab";

    // GitHub returns 204 No Content; mock as 200 + {} to keep octocrab happy.
    Mock::given(method("DELETE"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .remove_repository_collaborator(owner, repo, username)
        .await;

    if let Err(e) = &result {
        eprintln!("remove_repository_collaborator error: {e:?}");
    }
    assert!(
        result.is_ok(),
        "Expected Ok on successful collaborator removal"
    );
}

/// Test that remove_repository_collaborator returns an error when the repository or collaborator is not found.
#[tokio::test]
async fn test_remove_repository_collaborator_returns_not_found() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "nonexistent-repo";
    let username = "ghostuser";

    Mock::given(method("DELETE"))
        .and(path(format!(
            "/repos/{owner}/{repo}/collaborators/{username}"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .remove_repository_collaborator(owner, repo, username)
        .await;

    assert!(
        result.is_err(),
        "Expected error for non-existent repository"
    );
    assert!(
        matches!(result.unwrap_err(), Error::NotFound),
        "Expected NotFound error"
    );
}

// ============================================================================
// Fork & Mirror operations (RCA-004/005/006) — RepositoryClient contract tests
//
// See docs/spec/interfaces/fork-mirror-operations.md §5.
// ============================================================================

/// A representative GitHub pull-request API response body, used by all
/// `create_pull_request`/`mark_pull_request_ready_for_review` contract tests
/// below so octocrab's `pulls::PullRequest` model has everything it expects.
fn sample_pull_request_json(number: u64, draft: bool, state: &str) -> serde_json::Value {
    json!({
        "id": 1_000_000 + number,
        "node_id": format!("PR_kwDOABCDEF{number}"),
        "html_url": format!("https://github.com/test-org/test-repo/pull/{number}"),
        "diff_url": format!("https://github.com/test-org/test-repo/pull/{number}.diff"),
        "patch_url": format!("https://github.com/test-org/test-repo/pull/{number}.patch"),
        "issue_url": format!("https://api.github.com/repos/test-org/test-repo/issues/{number}"),
        "number": number,
        "state": state,
        "locked": false,
        "title": "Mirror source contents",
        "user": {
            "login": "reporoller-bot",
            "id": 1,
            "node_id": "U_1",
            "avatar_url": "https://avatars.githubusercontent.com/u/1?v=4",
            "gravatar_id": "",
            "url": "https://api.github.com/users/reporoller-bot",
            "html_url": "https://github.com/reporoller-bot",
            "type": "Bot",
            "site_admin": false
        },
        "body": "Mirrors the current contents of the source repository.",
        "created_at": "2026-10-26T00:00:00Z",
        "updated_at": "2026-10-26T00:00:00Z",
        "closed_at": null,
        "merged_at": null,
        "merge_commit_sha": null,
        "assignee": null,
        "assignees": [],
        "requested_reviewers": [],
        "requested_teams": [],
        "labels": [],
        "milestone": null,
        "draft": draft,
        "commits_url": format!("https://api.github.com/repos/test-org/test-repo/pulls/{number}/commits"),
        "review_comments_url": format!("https://api.github.com/repos/test-org/test-repo/pulls/{number}/comments"),
        "review_comment_url": "https://api.github.com/repos/test-org/test-repo/pulls/comments{/number}",
        "comments_url": format!("https://api.github.com/repos/test-org/test-repo/issues/{number}/comments"),
        "statuses_url": "https://api.github.com/repos/test-org/test-repo/statuses/abc123",
        "head": {
            "label": "test-org:mirror-branch",
            "ref": "mirror-branch",
            "sha": "abc123",
            "user": { "login": "test-org", "id": 2, "node_id": "U_2", "avatar_url": "https://avatars.githubusercontent.com/u/2?v=4", "gravatar_id": "", "url": "https://api.github.com/users/test-org", "html_url": "https://github.com/test-org", "type": "Organization", "site_admin": false },
            "repo": {
                "id": 123456,
                "name": "test-repo",
                "full_name": "test-org/test-repo",
                "private": true,
                "owner": { "login": "test-org", "id": 2, "node_id": "U_2", "avatar_url": "https://avatars.githubusercontent.com/u/2?v=4", "gravatar_id": "", "url": "https://api.github.com/users/test-org", "html_url": "https://github.com/test-org", "type": "Organization", "site_admin": false },
                "html_url": "https://github.com/test-org/test-repo",
                "url": "https://api.github.com/repos/test-org/test-repo"
            }
        },
        "base": {
            "label": "test-org:main",
            "ref": "main",
            "sha": "def456",
            "user": { "login": "test-org", "id": 2, "node_id": "U_2", "avatar_url": "https://avatars.githubusercontent.com/u/2?v=4", "gravatar_id": "", "url": "https://api.github.com/users/test-org", "html_url": "https://github.com/test-org", "type": "Organization", "site_admin": false },
            "repo": {
                "id": 123456,
                "name": "test-repo",
                "full_name": "test-org/test-repo",
                "private": true,
                "owner": { "login": "test-org", "id": 2, "node_id": "U_2", "avatar_url": "https://avatars.githubusercontent.com/u/2?v=4", "gravatar_id": "", "url": "https://api.github.com/users/test-org", "html_url": "https://github.com/test-org", "type": "Organization", "site_admin": false },
                "html_url": "https://github.com/test-org/test-repo",
                "url": "https://api.github.com/repos/test-org/test-repo"
            }
        },
        "_links": {
            "self": { "href": format!("https://api.github.com/repos/test-org/test-repo/pulls/{number}") },
            "html": { "href": format!("https://github.com/test-org/test-repo/pull/{number}") },
            "issue": { "href": format!("https://api.github.com/repos/test-org/test-repo/issues/{number}") },
            "comments": { "href": format!("https://api.github.com/repos/test-org/test-repo/issues/{number}/comments") },
            "review_comments": { "href": format!("https://api.github.com/repos/test-org/test-repo/pulls/{number}/comments") },
            "review_comment": { "href": "https://api.github.com/repos/test-org/test-repo/pulls/comments{/number}" },
            "commits": { "href": format!("https://api.github.com/repos/test-org/test-repo/pulls/{number}/commits") },
            "statuses": { "href": "https://api.github.com/repos/test-org/test-repo/statuses/abc123" }
        },
        "author_association": "NONE",
        "auto_merge": null,
        "active_lock_reason": null,
        "merged": false,
        "mergeable": null,
        "rebaseable": null,
        "mergeable_state": "unknown",
        "merged_by": null,
        "comments": 0,
        "review_comments": 0,
        "maintainer_can_modify": false,
        "commits": 1,
        "additions": 1,
        "deletions": 0,
        "changed_files": 1
    })
}

fn test_client(mock_server: &MockServer) -> GitHubClient {
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app_from_rsa_pem(TEST_APP_ID.into(), create_test_pem().as_bytes())
        .unwrap()
        .build()
        .unwrap();
    GitHubClient { client: octocrab }
}

// --- create_fork ---------------------------------------------------------

#[tokio::test]
async fn test_create_fork_success_returns_the_new_repository() {
    let mock_server = MockServer::start().await;
    let source_owner = "upstream-org";
    let source_repo = "upstream-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{source_owner}/{source_repo}/forks")))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "id": 42,
            "node_id": "R_kgDOfork",
            "name": source_repo,
            "full_name": format!("fork-owner/{source_repo}"),
            "private": false,
            "owner": { "login": "fork-owner", "id": 9, "node_id": "U_9", "avatar_url": "https://avatars.githubusercontent.com/u/9?v=4", "gravatar_id": "", "url": "https://api.github.com/users/fork-owner", "html_url": "https://github.com/fork-owner", "type": "Organization", "site_admin": false },
            "html_url": format!("https://github.com/fork-owner/{source_repo}"),
            "fork": true,
            "url": format!("https://api.github.com/repos/fork-owner/{source_repo}")
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client
        .create_fork(source_owner, source_repo, Some("fork-owner"))
        .await;

    if let Err(e) = &result {
        eprintln!("create_fork error: {e:?}");
    }
    let repo = result.expect("fork creation should succeed");
    assert_eq!(repo.name(), source_repo);
}

#[tokio::test]
async fn test_create_fork_into_authenticated_identity_when_target_owner_is_none() {
    let mock_server = MockServer::start().await;
    let source_owner = "upstream-org";
    let source_repo = "upstream-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{source_owner}/{source_repo}/forks")))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "id": 43,
            "node_id": "R_kgDOfork2",
            "name": source_repo,
            "full_name": format!("authenticated-user/{source_repo}"),
            "private": false,
            "owner": { "login": "authenticated-user", "id": 10, "node_id": "U_10", "avatar_url": "https://avatars.githubusercontent.com/u/10?v=4", "gravatar_id": "", "url": "https://api.github.com/users/authenticated-user", "html_url": "https://github.com/authenticated-user", "type": "User", "site_admin": false },
            "html_url": format!("https://github.com/authenticated-user/{source_repo}"),
            "fork": true,
            "url": format!("https://api.github.com/repos/authenticated-user/{source_repo}")
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client.create_fork(source_owner, source_repo, None).await;

    if let Err(e) = &result {
        eprintln!("create_fork (no target_owner) error: {e:?}");
    }
    assert!(result.is_ok());
}

/// Stub-killing / error-path: the source repository does not exist.
#[tokio::test]
async fn test_create_fork_source_not_found_returns_error() {
    let mock_server = MockServer::start().await;
    let source_owner = "upstream-org";
    let source_repo = "does-not-exist";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{source_owner}/{source_repo}/forks")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest/repos/forks#create-a-fork"
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client
        .create_fork(source_owner, source_repo, Some("fork-owner"))
        .await;

    assert!(
        result.is_err(),
        "forking a nonexistent source repository must fail"
    );
}

/// Stub-killing / error-path: a repository with the fork's resulting name
/// already exists at the target owner (422).
#[tokio::test]
async fn test_create_fork_name_conflict_at_target_owner_returns_error() {
    let mock_server = MockServer::start().await;
    let source_owner = "upstream-org";
    let source_repo = "upstream-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{source_owner}/{source_repo}/forks")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "Repository creation failed.",
            "errors": [{"resource": "Repository", "code": "custom", "field": "name", "message": "name already exists on this account"}]
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client
        .create_fork(source_owner, source_repo, Some("fork-owner"))
        .await;

    assert!(result.is_err());
}

// --- create_pull_request --------------------------------------------------

#[tokio::test]
async fn test_create_pull_request_success_returns_pull_request_not_draft() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{owner}/{repo}/pulls")))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sample_pull_request_json(7, false, "open")),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let payload = PullRequestCreatePayload {
        title: "Mirror source contents".to_string(),
        head: "mirror-branch".to_string(),
        base: "main".to_string(),
        body: Some("Mirrors the current contents of the source repository.".to_string()),
        draft: false,
    };

    let result = client.create_pull_request(owner, repo, &payload).await;

    if let Err(e) = &result {
        eprintln!("create_pull_request error: {e:?}");
    }
    let pull_request = result.expect("pull request creation should succeed");
    assert_eq!(pull_request.number, 7);
    assert!(
        !pull_request.draft,
        "PR created with draft: false must not be draft"
    );
    assert_eq!(pull_request.state, "open");
}

/// Stub-killing: a hardcoded `draft: false` return would pass the test above
/// but fail here, where the mocked API response says the PR IS a draft.
#[tokio::test]
async fn test_create_pull_request_surfaces_draft_flag_from_response() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{owner}/{repo}/pulls")))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sample_pull_request_json(8, true, "open")),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let payload = PullRequestCreatePayload {
        title: "t".to_string(),
        head: "branch".to_string(),
        base: "main".to_string(),
        body: None,
        draft: true,
    };

    let result = client
        .create_pull_request(owner, repo, &payload)
        .await
        .expect("creation should succeed");

    assert!(
        result.draft,
        "response says draft:true, PullRequest.draft must reflect that"
    );
}

#[tokio::test]
async fn test_create_pull_request_branch_not_found_returns_error() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{owner}/{repo}/pulls")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "Validation Failed",
            "errors": [{"resource": "PullRequest", "code": "invalid", "field": "head", "message": "head branch does not exist"}]
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let payload = PullRequestCreatePayload {
        title: "t".to_string(),
        head: "nonexistent-branch".to_string(),
        base: "main".to_string(),
        body: None,
        draft: false,
    };

    let result = client.create_pull_request(owner, repo, &payload).await;

    assert!(
        result.is_err(),
        "PR creation against a nonexistent branch must fail"
    );
}

#[tokio::test]
async fn test_create_pull_request_already_exists_returns_error() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";

    Mock::given(method("POST"))
        .and(path(format!("/repos/{owner}/{repo}/pulls")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "message": "Validation Failed",
            "errors": [{"resource": "PullRequest", "code": "custom", "message": "A pull request already exists for test-org:mirror-branch."}]
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let payload = PullRequestCreatePayload {
        title: "t".to_string(),
        head: "mirror-branch".to_string(),
        base: "main".to_string(),
        body: None,
        draft: false,
    };

    let result = client.create_pull_request(owner, repo, &payload).await;

    assert!(result.is_err());
}

// --- mark_pull_request_ready_for_review -----------------------------------

#[tokio::test]
async fn test_mark_pull_request_ready_for_review_success_returns_non_draft() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let pr_number = 9;

    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/pulls/{pr_number}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(sample_pull_request_json(pr_number, false, "open")),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client
        .mark_pull_request_ready_for_review(owner, repo, pr_number)
        .await;

    if let Err(e) = &result {
        eprintln!("mark_pull_request_ready_for_review error: {e:?}");
    }
    let pull_request = result.expect("transition should succeed");
    assert_eq!(pull_request.number, pr_number);
    assert!(!pull_request.draft);
}

#[tokio::test]
async fn test_mark_pull_request_ready_for_review_not_found_returns_error() {
    let mock_server = MockServer::start().await;
    let owner = "test-org";
    let repo = "test-repo";
    let pr_number = 999;

    Mock::given(method("PATCH"))
        .and(path(format!("/repos/{owner}/{repo}/pulls/{pr_number}")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest/pulls/pulls#update-a-pull-request"
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let result = client
        .mark_pull_request_ready_for_review(owner, repo, pr_number)
        .await;

    assert!(result.is_err(), "transitioning a nonexistent PR must fail");
}

// --- PullRequestCreatePayload / PullRequest type sanity --------------------

#[test]
fn test_pull_request_create_payload_default_is_not_draft() {
    let payload = PullRequestCreatePayload::default();
    assert!(
        !payload.draft,
        "Default::default() must not silently create a draft PR"
    );
    assert!(payload.body.is_none());
}

#[test]
fn test_pull_request_create_payload_is_cloneable() {
    let payload = PullRequestCreatePayload {
        title: "t".to_string(),
        head: "h".to_string(),
        base: "b".to_string(),
        body: Some("body".to_string()),
        draft: true,
    };
    let cloned = payload.clone();
    assert_eq!(cloned.title, payload.title);
    assert_eq!(cloned.draft, payload.draft);
}
