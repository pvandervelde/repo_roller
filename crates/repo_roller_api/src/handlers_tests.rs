//! Tests for handlers module

use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::routes::create_router_without_auth;

/// Helper function to create a test app state
fn test_app_state() -> AppState {
    AppState::default()
}

// ============================================================================
// Health Check Tests
// ============================================================================

/// Test that health check handler returns proper JSON response
#[tokio::test]
async fn test_health_check_returns_json() {
    let response = health_check().await;

    // Verify structure exists
    assert_eq!(response.0.status, "healthy");
    assert!(response.0.version.is_some());
    assert!(!response.0.timestamp.is_empty());
    assert!(response.0.error.is_none());
}

/// Test that health check includes version from Cargo.toml
#[tokio::test]
async fn test_health_check_includes_version() {
    let response = health_check().await;

    assert_eq!(
        response.0.version,
        Some(env!("CARGO_PKG_VERSION").to_string())
    );
}

/// Test that health check timestamp is valid ISO 8601
#[tokio::test]
async fn test_health_check_timestamp_format() {
    let response = health_check().await;

    // Should be parseable as ISO 8601
    let parsed = chrono::DateTime::parse_from_rfc3339(&response.0.timestamp);
    assert!(parsed.is_ok(), "Timestamp should be valid ISO 8601 format");
}

// ============================================================================
// Repository Management Handler Tests
// ============================================================================

/// Test validate_repository_name endpoint with valid name.
///
/// The handler now requires `AuthContext` and calls the GitHub API for
/// availability. With a fake token the GitHub API returns an error;
/// the handler degrades gracefully and still responds 200 with valid=true.
#[tokio::test]
async fn test_validate_repository_name_valid() {
    let app = create_router_without_auth(test_app_state()).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new(
                    "test-token-123".to_string(),
                ));
            next.run(req).await
        },
    ));

    let request_body = json!({
        "organization": "testorg",
        "name": "valid-repo-name"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], true);
    // GitHub call fails with fake token — handler degrades: available=true with a warning message.
    assert_eq!(response_json["available"], true);
    // A warning message must be present when the GitHub API call fails (degraded path).
    let msgs = response_json["messages"]
        .as_array()
        .expect("messages should be a JSON array in the degraded path");
    assert!(
        !msgs.is_empty(),
        "Expected at least one warning message when GitHub availability check degrades"
    );
    assert!(
        msgs.iter().any(|m| m
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains("could not verify")),
        "Warning message should indicate that availability could not be verified"
    );
}

/// Test validate_repository_name endpoint with invalid name.
///
/// Format-invalid names short-circuit before any GitHub API call;
/// valid=false, available=false is always returned.
#[tokio::test]
async fn test_validate_repository_name_invalid() {
    let app = create_router_without_auth(test_app_state()).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new(
                    "test-token-123".to_string(),
                ));
            next.run(req).await
        },
    ));

    let request_body = json!({
        "organization": "testorg",
        "name": "Invalid@Name"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);
    assert_eq!(response_json["available"], false);

    // Check messages field exists and has content
    assert!(response_json["messages"].is_array());
    let messages = response_json["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
}
/// Test validate_repository_name endpoint with empty name.
///
/// Empty names fail the format check and short-circuit; no GitHub call is made.
#[tokio::test]
async fn test_validate_repository_name_empty() {
    let app = create_router_without_auth(test_app_state()).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new(
                    "test-token-123".to_string(),
                ));
            next.run(req).await
        },
    ));

    let request_body = json!({
        "organization": "testorg",
        "name": ""
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], false);
    assert_eq!(response_json["available"], false);
    assert!(response_json["messages"].is_array());
    assert!(!response_json["messages"].as_array().unwrap().is_empty());
}

/// Test validate_repository_request endpoint with valid complete request
///
/// Verifies that a valid complete repository creation request returns
/// 200 OK with valid=true and no errors.
#[tokio::test]
async fn test_validate_repository_request_valid() {
    let app = create_router_without_auth(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "visibility": "private",
        "team": "platform",
        "repositoryType": "library",
        "variables": {
            "projectName": "Test Project",
            "author": "Test Author"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["valid"], true);
    // Errors field is omitted when empty due to skip_serializing_if
    if let Some(errors) = response_json.get("errors") {
        assert!(errors.as_array().unwrap().is_empty());
    }
}

/// Test validate_repository_request endpoint with missing template variables
///
/// Verifies that the validate endpoint performs only structural validation.
/// Template variable completeness is deferred to the creation step where the
/// template configuration is available.
#[tokio::test]
async fn test_validate_repository_request_missing_variables() {
    let app = create_router_without_auth(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "rust-library",
        "variables": {}  // No variables provided
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Structural validation passes: name, org, and template are all non-empty.
    // Variable completeness is validated during creation, not during this pre-check.
    assert_eq!(
        response_json["valid"], true,
        "validate endpoint should return valid=true for structurally correct request"
    );
}

/// Test validate_repository_request endpoint with a template name
///
/// Verifies that the validate endpoint performs only structural validation.
/// Template existence is deferred to the creation step where GitHub API is available.
#[tokio::test]
async fn test_validate_repository_request_nonexistent_template() {
    let app = create_router_without_auth(test_app_state());

    let request_body = json!({
        "organization": "testorg",
        "name": "test-repo",
        "template": "nonexistent-template",
        "variables": {}
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate")
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token-123")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Template existence is not checked in structural validation (requires GitHub API).
    // The request is structurally valid: non-empty org, name, and template name.
    assert_eq!(
        response_json["valid"], true,
        "validate endpoint should return valid=true for structurally correct request"
    );
}

// ============================================================================
// check_org_naming_rules unit tests
//
// These tests call the helper function directly with a mock provider so they
// exercise the naming-rule logic without a live GitHub connection.
// ============================================================================

/// Minimal mock that lets tests configure what `load_global_defaults` returns.
struct MockNamingRuleProvider {
    naming_rules: Option<Vec<config_manager::RepositoryNamingRulesConfig>>,
}

impl MockNamingRuleProvider {
    /// Provider that returns no naming rules (empty org config).
    fn with_no_rules() -> Self {
        Self { naming_rules: None }
    }

    /// Provider that returns the supplied naming rules.
    fn with_rules(rules: Vec<config_manager::RepositoryNamingRulesConfig>) -> Self {
        Self {
            naming_rules: Some(rules),
        }
    }
}

#[async_trait::async_trait]
impl config_manager::MetadataRepositoryProvider for MockNamingRuleProvider {
    async fn discover_metadata_repository(
        &self,
        _org: &str,
    ) -> config_manager::ConfigurationResult<config_manager::MetadataRepository> {
        Ok(config_manager::MetadataRepository {
            organization: "test-org".to_string(),
            repository_name: "test-meta".to_string(),
            discovery_method: config_manager::DiscoveryMethod::ConfigurationBased {
                repository_name: "test-meta".to_string(),
            },
            last_updated: chrono::Utc::now(),
        })
    }

    async fn load_global_defaults(
        &self,
        _repo: &config_manager::MetadataRepository,
    ) -> config_manager::ConfigurationResult<config_manager::GlobalDefaults> {
        Ok(config_manager::GlobalDefaults {
            naming_rules: self.naming_rules.clone(),
            ..Default::default()
        })
    }

    async fn load_team_configuration(
        &self,
        _repo: &config_manager::MetadataRepository,
        _team: &str,
    ) -> config_manager::ConfigurationResult<Option<config_manager::TeamConfig>> {
        Ok(None)
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &config_manager::MetadataRepository,
        _repo_type: &str,
    ) -> config_manager::ConfigurationResult<Option<config_manager::RepositoryTypeConfig>> {
        Ok(None)
    }

    async fn load_standard_labels(
        &self,
        _repo: &config_manager::MetadataRepository,
    ) -> config_manager::ConfigurationResult<
        std::collections::HashMap<String, config_manager::LabelConfig>,
    > {
        Ok(std::collections::HashMap::new())
    }

    async fn load_global_webhooks(
        &self,
        _repo: &config_manager::MetadataRepository,
    ) -> config_manager::ConfigurationResult<Vec<config_manager::settings::WebhookConfig>> {
        Ok(vec![])
    }

    async fn list_available_repository_types(
        &self,
        _repo: &config_manager::MetadataRepository,
    ) -> config_manager::ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn validate_repository_structure(
        &self,
        _repo: &config_manager::MetadataRepository,
    ) -> config_manager::ConfigurationResult<()> {
        Ok(())
    }

    async fn list_templates(&self, _org: &str) -> config_manager::ConfigurationResult<Vec<String>> {
        Ok(vec![])
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> config_manager::ConfigurationResult<config_manager::TemplateConfig> {
        Err(config_manager::ConfigurationError::FileNotFound {
            path: "template.toml".to_string(),
        })
    }
}

/// When the org has no naming rules configured, any format-valid name passes.
#[tokio::test]
async fn test_check_org_naming_rules_no_rules_returns_empty() {
    let provider = MockNamingRuleProvider::with_no_rules();
    let messages = check_org_naming_rules("my-service", "test-org", &provider).await;
    assert!(
        messages.is_empty(),
        "Expected no messages when no org naming rules are configured"
    );
}

/// A format-valid name that satisfies all org naming rules produces no messages.
#[tokio::test]
async fn test_check_org_naming_rules_valid_name_passes_rules() {
    let rules = vec![config_manager::RepositoryNamingRulesConfig {
        required_prefix: Some("svc-".to_string()),
        ..Default::default()
    }];
    let provider = MockNamingRuleProvider::with_rules(rules);
    let messages = check_org_naming_rules("svc-billing", "test-org", &provider).await;
    assert!(
        messages.is_empty(),
        "Expected no messages for a name that satisfies the prefix rule"
    );
}

/// A name that violates an org-level naming rule produces a non-empty message list.
#[tokio::test]
async fn test_check_org_naming_rules_prefix_violation_returns_message() {
    let rules = vec![config_manager::RepositoryNamingRulesConfig {
        required_prefix: Some("svc-".to_string()),
        ..Default::default()
    }];
    let provider = MockNamingRuleProvider::with_rules(rules);
    let messages = check_org_naming_rules("billing", "test-org", &provider).await;
    assert!(
        !messages.is_empty(),
        "Expected error messages when the name violates the required prefix rule"
    );
    assert!(
        messages[0].contains("svc-"),
        "Error message should mention the required prefix"
    );
}

/// A name that matches a forbidden pattern returns an error message.
#[tokio::test]
async fn test_check_org_naming_rules_forbidden_pattern_violation_returns_message() {
    let rules = vec![config_manager::RepositoryNamingRulesConfig {
        forbidden_patterns: vec!["noncompliant".to_string()],
        ..Default::default()
    }];
    let provider = MockNamingRuleProvider::with_rules(rules);
    let messages = check_org_naming_rules("my-noncompliant-repo", "test-org", &provider).await;
    assert!(
        !messages.is_empty(),
        "Expected error messages when the name matches a forbidden pattern"
    );
}

/// A name matching a reserved word is rejected.
#[tokio::test]
async fn test_check_org_naming_rules_reserved_word_returns_message() {
    let rules = vec![config_manager::RepositoryNamingRulesConfig {
        reserved_words: vec!["test".to_string()],
        ..Default::default()
    }];
    let provider = MockNamingRuleProvider::with_rules(rules);
    let messages = check_org_naming_rules("test", "test-org", &provider).await;
    assert!(
        !messages.is_empty(),
        "Expected error messages when the name is a reserved word"
    );
}

// ============================================================================
// check_repository_availability unit tests
//
// These tests call the helper directly with a wiremock-backed GitHubClient
// so they exercise the availability logic without a live GitHub connection.
// ============================================================================

/// Repository does not exist (GitHub returns 404) → available=true, no message.
#[tokio::test]
async fn test_check_repository_availability_not_found_returns_available() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/new-repo"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .personal_token("x".to_string())
        .build()
        .unwrap();
    let client = github_client::GitHubClient::new(octocrab);

    let (available, message) = check_repository_availability(&client, "testorg", "new-repo").await;

    assert!(
        available,
        "Repository that does not exist should be available"
    );
    assert!(
        message.is_none(),
        "No warning message expected when the repository is confirmed free"
    );
}

/// Repository already exists (GitHub returns 200) → available=false, message says taken.
#[tokio::test]
async fn test_check_repository_availability_exists_returns_not_available() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/existing-repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "existing-repo",
            "full_name": "testorg/existing-repo",
            "private": false,
            "html_url": "https://github.com/testorg/existing-repo",
            "url": "https://api.github.com/repos/testorg/existing-repo",
            "owner": {
                "login": "testorg",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "avatar_url": "https://avatars.githubusercontent.com/u/1?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/testorg",
                "html_url": "https://github.com/testorg",
                "followers_url": "https://api.github.com/users/testorg/followers",
                "following_url": "https://api.github.com/users/testorg/following{/other_user}",
                "gists_url": "https://api.github.com/users/testorg/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/testorg/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/testorg/subscriptions",
                "organizations_url": "https://api.github.com/users/testorg/orgs",
                "repos_url": "https://api.github.com/users/testorg/repos",
                "events_url": "https://api.github.com/users/testorg/events{/privacy}",
                "received_events_url": "https://api.github.com/users/testorg/received_events",
                "type": "Organization",
                "site_admin": false,
                "patch_url": null,
                "email": null
            }
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .personal_token("x".to_string())
        .build()
        .unwrap();
    let client = github_client::GitHubClient::new(octocrab);

    let (available, message) =
        check_repository_availability(&client, "testorg", "existing-repo").await;

    assert!(
        !available,
        "Repository that already exists should not be available"
    );
    assert!(
        message.is_some(),
        "Expected a message explaining the name is taken"
    );
    let msg = message.unwrap();
    assert!(
        msg.contains("existing-repo"),
        "Message should mention the repository name; got: {msg}"
    );
}

/// GitHub returns a non-404 error → available=true (warn-only), message warns the check failed.
#[tokio::test]
async fn test_check_repository_availability_api_error_returns_available_with_warning() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/testorg/some-repo"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .unwrap()
        .personal_token("x".to_string())
        .build()
        .unwrap();
    let client = github_client::GitHubClient::new(octocrab);

    let (available, message) = check_repository_availability(&client, "testorg", "some-repo").await;

    assert!(
        available,
        "API errors should not block the user: available must be true"
    );
    assert!(
        message.is_some(),
        "Expected a warning message when the availability check fails"
    );
    let msg = message.unwrap();
    assert!(
        msg.to_lowercase().contains("could not verify"),
        "Warning should mention that availability could not be verified; got: {msg}"
    );
}

/// Handler-level test: when the GitHub API confirms the repository already
/// exists, the handler returns `available=false`.
///
/// Uses a wiremock server to avoid real network calls and to control the
/// response precisely. AppState::with_github_api_base_url() redirects all
/// GitHub API calls from the handler to the mock server.
#[tokio::test]
async fn test_validate_repository_name_returns_available_false_when_repo_exists() {
    let mock_server = MockServer::start().await;

    // The handler calls GET /repos/{org}/{name} to check existence.
    Mock::given(method("GET"))
        .and(path("/repos/testorg/existing-repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 42,
            "name": "existing-repo",
            "full_name": "testorg/existing-repo",
            "private": false,
            "html_url": "https://github.com/testorg/existing-repo",
            "url": "https://api.github.com/repos/testorg/existing-repo",
            "owner": {
                "login": "testorg",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "avatar_url": "https://avatars.githubusercontent.com/u/1?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/testorg",
                "html_url": "https://github.com/testorg",
                "followers_url": "https://api.github.com/users/testorg/followers",
                "following_url": "https://api.github.com/users/testorg/following{/other_user}",
                "gists_url": "https://api.github.com/users/testorg/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/testorg/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/testorg/subscriptions",
                "organizations_url": "https://api.github.com/users/testorg/orgs",
                "repos_url": "https://api.github.com/users/testorg/repos",
                "events_url": "https://api.github.com/users/testorg/events{/privacy}",
                "received_events_url": "https://api.github.com/users/testorg/received_events",
                "type": "Organization",
                "site_admin": false,
                "patch_url": null,
                "email": null
            }
        })))
        .mount(&mock_server)
        .await;

    // Point the handler's GitHub client at the mock server.
    let state = AppState::default().with_github_api_base_url(mock_server.uri());

    let app = create_router_without_auth(state).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new("x".to_string()));
            next.run(req).await
        },
    ));

    let request_body = json!({
        "organization": "testorg",
        "name": "existing-repo"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Format is valid, but the name is taken — available must be false.
    assert_eq!(response_json["valid"], true, "name format is valid");
    assert_eq!(
        response_json["available"], false,
        "name is already taken; available must be false"
    );
    assert!(
        response_json["messages"].is_array(),
        "messages should be present when name is taken"
    );
}

/// Handler-level test: empty organization field returns valid=false.
#[tokio::test]
async fn test_validate_repository_name_empty_org_returns_invalid() {
    let app = create_router_without_auth(test_app_state()).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new(
                    "test-token-123".to_string(),
                ));
            next.run(req).await
        },
    ));

    let request_body = json!({
        "organization": "",
        "name": "valid-name"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        response_json["valid"], false,
        "empty org should fail validation"
    );
    assert_eq!(response_json["available"], false);
    let msgs = response_json["messages"]
        .as_array()
        .expect("messages should be present");
    assert!(!msgs.is_empty());
}

// ============================================================================
// list_organization_teams handler tests
// ============================================================================

/// Verify that GET /api/v1/orgs/:org/teams is routed correctly.
///
/// The handler requires a real GitHub token to call the GitHub API.  With the
/// no-auth test router we inject a thin fake `AuthContext` to get past the
/// extension extraction, and the handler will attempt (and fail) the GitHub
/// API call because the token is not a valid installation token.
/// That results in a 500 from `create_token_client` / the octocrab client
/// rather than a 404 (which would mean the route is missing).
///
/// This test therefore verifies route wiring without needing a live GitHub
/// connection.
#[tokio::test]
async fn test_list_organization_teams_route_is_registered() {
    // Build a router that injects a fake AuthContext so the handler can
    // extract it, then let the GitHub API call fail naturally.
    let app = create_router_without_auth(test_app_state()).layer(middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut()
                .insert(crate::middleware::AuthContext::new(
                    "test-token".to_string(),
                ));
            next.run(req).await
        },
    ));

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/orgs/testorg/teams")
        .header("authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // The route exists — a 404 would mean the route is not registered.
    // The GitHub API call will fail (fake token), producing a 500.
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected route to be registered; got 404 which means the route is missing"
    );
}
