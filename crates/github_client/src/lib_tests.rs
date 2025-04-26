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
async fn test_create_file_success() {
    // Arrange: Start a mock server and configure the endpoint
    let mock_server = MockServer::start().await;
    let owner = "test-owner";
    let repo = "test-repo";
    let file_path = "README.md";
    let file_content = b"Hello, world!";
    let commit_message = "Initial commit";

    // The expected API path
    let api_path = format!("/repos/{}/{}/contents/{}", owner, repo, file_path);

    // Mock the GitHub API response for file creation
    Mock::given(method("PUT"))
        .and(path(api_path.clone()))
        .and(header("accept", "application/vnd.github+json"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "content": {
                "name": file_path,
                "path": file_path,
                "sha": "fake-sha"
            },
            "commit": {
                "message": commit_message
            }
        })))
        .mount(&mock_server)
        .await;

    // Build a GitHubClient with the mock server as the base URL
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(create_test_pem().as_bytes()).unwrap();
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .app(TEST_APP_ID.into(), key)
        .build()
        .unwrap();
    let client = GitHubClient { client: octocrab };

    // Act: Call create_file
    let result = client
        .create_file(owner, repo, file_path, file_content, commit_message)
        .await;

    // Assert: Should succeed
    if let Err(e) = &result {
        eprintln!("create_file error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_org_repository_success() {
    let mock_server = MockServer::start().await;
    let org_name = "test-org";
    let payload = RepositoryCreatePayload {
        name: "test-repo",
        description: Some("A test repository"),
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
        name: "test-repo",
        description: Some("A test repository"),
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
        description: Some("Updated description"),
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
async fn test_new_client_success() {
    let pem = create_test_pem();
    let result = GitHubClient::new(TEST_APP_ID, pem).await;
    assert!(result.is_ok());
}
