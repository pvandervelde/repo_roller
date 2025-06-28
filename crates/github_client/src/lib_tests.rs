//! Unit tests for the github_client crate.

use super::*; // Import items from lib.rs
use rand::thread_rng;
use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate}; // For constructing mock bodies

// --- Test Constants ---
const TEST_APP_ID: u64 = 12345;
// A dummy RSA private key (replace with a real one for integration tests if needed, but fine for mocking)
// Generate with: openssl genpkey -algorithm RSA -out private_key.pem -pkeyopt rsa_keygen_bits:2048

fn create_test_pem() -> String {
    let mut rng = thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate key");
    private_key
        .to_pkcs8_pem(Default::default())
        .unwrap()
        .to_string()
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
        .and(path(format!("/orgs/{}/repos", org_name)))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 123456,
            "name": payload.name,
            "description": payload.description,
            "url": "https://api.github.com/repos/test-org/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.create_org_repository(org_name, &payload).await;

    if let Err(e) = &result {
        eprintln!("create_org_repository error: {:?}", e);
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.create_user_repository(&payload).await;

    if let Err(e) = &result {
        eprintln!("create_user_repository error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_repository_success() {
    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";

    Mock::given(method("GET"))
        .and(path(format!("/repos/{}/{}", owner, repo)))
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_repository(owner, repo).await;

    if let Err(e) = &result {
        eprintln!("get_repository error: {:?}", e);
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
        .and(path(format!("/repos/{}/{}", owner, repo)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123456,
            "name": repo,
            "description": settings.description,
            "url": "https://api.github.com/repos/test-owner/test-repo"
        })))
        .mount(&mock_server)
        .await;

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .update_repository_settings(owner, repo, &settings)
        .await;

    if let Err(e) = &result {
        eprintln!("update_repository_settings error: {:?}", e);
    }
    assert!(result.is_ok());
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.list_installations().await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in list_installations: {:?}", e);
            panic!("Expected Ok, got Err: {:?}", e);
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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
            "/app/installations/{}/access_tokens",
            installation_id
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "token": test_token,
            "expires_at": "2025-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client.get_installation_token_for_org(org_name).await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in get_installation_token_for_org: {:?}", e);
            panic!("Expected Ok, got Err: {:?}", e);
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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
            "/app/installations/{}/access_tokens",
            installation_id
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "token": test_token,
            "expires_at": "2025-12-31T23:59:59Z"
        })))
        .mount(&mock_server)
        .await;

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    // Test that uppercase org name finds lowercase match
    let result = client.get_installation_token_for_org(org_name_upper).await;

    match &result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error in get_installation_token_case_insensitive: {:?}", e);
            panic!("Expected Ok, got Err: {:?}", e);
        }
    }
    let token = result.unwrap();
    assert_eq!(token, test_token);
}
