//! Tests for handlers module

use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

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

/// Test validate_repository_name endpoint with valid name
///
/// Verifies that a valid repository name returns 200 OK with valid=true.
#[tokio::test]
async fn test_validate_repository_name_valid() {
    let app = create_router_without_auth(test_app_state());

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
    assert_eq!(response_json["available"], true);
}

/// Test validate_repository_name endpoint with invalid name
///
/// Verifies that an invalid repository name returns 200 OK with valid=false
/// and includes validation error details.
#[tokio::test]
async fn test_validate_repository_name_invalid() {
    let app = create_router_without_auth(test_app_state());

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
/// Test validate_repository_name endpoint with empty name
///
/// Verifies that empty repository name returns 200 OK with valid=false
/// and appropriate error message.
#[tokio::test]
async fn test_validate_repository_name_empty() {
    let app = create_router_without_auth(test_app_state());

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
    use axum::middleware;

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
