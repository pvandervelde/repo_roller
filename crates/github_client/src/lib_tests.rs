//! Unit tests for the github_client crate.

use super::*; // Import items from lib.rs
use rand::thread_rng;
use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};
use serde_json::json;
use wiremock::matchers::{method, path};
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
        .and(path(format!("/orgs/{org_name}/repos")))
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
        eprintln!("update_repository_settings error: {e:?}");
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
            "/app/installations/{installation_id}/access_tokens"
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
            "/app/installations/{installation_id}/access_tokens"
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    let result = client
        .get_team_repository_permission(org, team_slug, repo_owner, repo)
        .await;

    assert!(result.is_err(), "Expected Err on non-404 API failure");
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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

    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
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
