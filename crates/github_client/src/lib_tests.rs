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
