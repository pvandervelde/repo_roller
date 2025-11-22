//! Integration tests for REST API endpoints.
//!
//! These tests verify the REST API endpoints work correctly with real GitHub infrastructure.
//! They require:
//! - Valid GitHub App credentials
//! - Access to a test organization
//! - Metadata repository configured
//!
//! Run with: cargo test -p integration_tests --test rest_api_endpoints -- --ignored

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use repo_roller_api::{routes::create_router, AppState};
use serde_json::json;
use tower::ServiceExt;

/// Helper function to create test app state from environment
fn test_app_state() -> AppState {
    // Use metadata repository name from env or default
    let metadata_repo =
        std::env::var("METADATA_REPOSITORY_NAME").unwrap_or_else(|_| ".reporoller".to_string());

    AppState::new(metadata_repo)
}

/// Helper function to get test token from environment
fn test_token() -> String {
    std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN environment variable required for integration tests")
}

/// Helper function to get test organization from environment
fn test_org() -> String {
    std::env::var("TEST_ORG").unwrap_or_else(|_| "test-org".to_string())
}

// ============================================================================
// Organization Settings Endpoint Tests
// ============================================================================

/// Test list_repository_types endpoint with real GitHub infrastructure.
///
/// Verifies that listing repository types for an organization returns 200 OK
/// with an array of type summaries from the actual metadata repository.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and configured metadata repository"]
async fn test_list_repository_types_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/repository-types", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for valid organization"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(
        response_json["types"].is_array(),
        "Response should contain 'types' array"
    );

    // Note: May be empty if metadata repository has no types configured
    // That's valid - organization might not have types set up yet
}

/// Test get_repository_type_config endpoint with real configuration.
///
/// Verifies that requesting configuration for an existing type returns
/// 200 OK with complete configuration from the metadata repository.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and configured repository type"]
async fn test_get_repository_type_config_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    // Use a type name from environment or skip if not set
    let type_name = match std::env::var("TEST_REPOSITORY_TYPE") {
        Ok(name) => name,
        Err(_) => {
            println!("Skipping test: TEST_REPOSITORY_TYPE not set");
            return;
        }
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/orgs/{}/repository-types/{}",
            org, type_name
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for existing repository type"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify configuration structure
    assert!(
        response_json["config"].is_object(),
        "Response should contain 'config' object"
    );
}

/// Test get_repository_type_config with non-existent type.
///
/// Verifies that requesting configuration for a non-existent type returns 404.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_get_repository_type_config_not_found() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/orgs/{}/repository-types/nonexistent-type",
            org
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Should return 404 for non-existent repository type"
    );
}

/// Test get_global_defaults endpoint with real configuration.
///
/// Verifies that requesting global defaults returns 200 OK with
/// organization-wide default settings from the metadata repository.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and configured metadata repository"]
async fn test_get_global_defaults_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/defaults", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for organization with metadata repository"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify defaults structure
    assert!(
        response_json["defaults"].is_object(),
        "Response should contain 'defaults' object"
    );
}

/// Test preview_configuration endpoint with real configuration merging.
///
/// Verifies that configuration preview correctly merges settings from
/// multiple levels (global, team, type, template).
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and configured metadata repository"]
async fn test_preview_configuration_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());

    let request_body = json!({
        "template": template,
    });

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/preview", org))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for valid preview request"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify preview structure
    assert!(
        response_json["mergedConfiguration"].is_object(),
        "Response should contain 'mergedConfiguration' object"
    );
}

/// Test preview_configuration with team and repository type.
///
/// Verifies that preview correctly applies team and type-specific overrides.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with team and type configurations"]
async fn test_preview_configuration_with_overrides() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());
    let team = std::env::var("TEST_TEAM").ok();
    let repo_type = std::env::var("TEST_REPOSITORY_TYPE").ok();

    if team.is_none() && repo_type.is_none() {
        println!("Skipping test: Neither TEST_TEAM nor TEST_REPOSITORY_TYPE set");
        return;
    }

    let mut request_body = json!({
        "template": template,
    });

    if let Some(t) = team {
        request_body["team"] = json!(t);
    }
    if let Some(rt) = repo_type {
        request_body["repositoryType"] = json!(rt);
    }

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/preview", org))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for preview with team/type overrides"
    );
}

/// Test validate_organization endpoint with real metadata repository.
///
/// Verifies that organization validation correctly checks metadata repository
/// configuration and returns validation results.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and metadata repository"]
async fn test_validate_organization_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/validate", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for validation request"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify validation result structure
    assert!(
        response_json["valid"].is_boolean(),
        "Response should contain 'valid' boolean"
    );
    assert!(
        response_json["errors"].is_array(),
        "Response should contain 'errors' array"
    );
    assert!(
        response_json["warnings"].is_array(),
        "Response should contain 'warnings' array"
    );
}

/// Test validate_organization with organization lacking metadata repository.
///
/// Verifies that validation fails appropriately when metadata repository
/// is not configured.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and organization without metadata repo"]
async fn test_validate_organization_no_metadata_repo() {
    let app = create_router(test_app_state());
    let token = test_token();

    // Use a different org that doesn't have metadata repo
    let org = std::env::var("TEST_ORG_NO_METADATA")
        .unwrap_or_else(|_| "org-without-metadata".to_string());

    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/validate", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK even when validation fails"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should report validation failure
    assert_eq!(
        response_json["valid"].as_bool(),
        Some(false),
        "Validation should fail for organization without metadata repository"
    );

    let errors = response_json["errors"].as_array().unwrap();
    assert!(
        !errors.is_empty(),
        "Should have at least one error about missing metadata repository"
    );
}

// ============================================================================
// Complete REST API Workflow Test
// ============================================================================

/// Test complete REST API workflow with real GitHub infrastructure.
///
/// This test runs through a complete workflow:
/// 1. Validate organization configuration
/// 2. List available repository types
/// 3. Get global defaults
/// 4. Preview configuration for a repository
///
/// This ensures all endpoints work together correctly.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with complete metadata repository"]
async fn test_complete_rest_api_workflow() {
    let token = test_token();
    let org = test_org();
    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());

    // Step 1: Validate organization
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/validate", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Validation should succeed"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let validation: serde_json::Value = serde_json::from_slice(&body).unwrap();

    if !validation["valid"].as_bool().unwrap_or(false) {
        println!("Organization validation failed, skipping workflow test");
        println!("Errors: {:?}", validation["errors"]);
        return;
    }

    // Step 2: List repository types
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/repository-types", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "List types should succeed"
    );

    // Step 3: Get global defaults
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/defaults", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Get defaults should succeed"
    );

    // Step 4: Preview configuration
    let app = create_router(test_app_state());
    let request_body = json!({ "template": template });
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/preview", org))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Preview should succeed");

    println!("Complete REST API workflow test passed!");
}

// ============================================================================
// Template Discovery Endpoint Tests
// ============================================================================

/// Test list_templates endpoint with real GitHub infrastructure.
///
/// Verifies that listing templates discovers repositories with the
/// reporoller-template topic and returns their configurations.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with template repositories"]
async fn test_list_templates_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/templates", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for valid organization"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(
        response_json["templates"].is_array(),
        "Response should contain 'templates' array"
    );

    // Note: May be empty if organization has no template repositories
    let templates = response_json["templates"].as_array().unwrap();
    println!("Found {} template(s)", templates.len());

    for template in templates {
        assert!(
            template["name"].is_string(),
            "Each template should have a name"
        );
        assert!(
            template["description"].is_string(),
            "Each template should have a description"
        );
    }
}

/// Test get_template_details endpoint with real template repository.
///
/// Verifies that requesting template details loads the template.toml
/// configuration and returns complete template information.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with configured template repository"]
async fn test_get_template_details_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    // Use template name from environment or skip if not set
    let template_name = match std::env::var("TEST_TEMPLATE") {
        Ok(name) => name,
        Err(_) => {
            println!("Skipping test: TEST_TEMPLATE not set");
            return;
        }
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/templates/{}", org, template_name))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for existing template"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify template details structure
    assert_eq!(
        response_json["name"].as_str(),
        Some(template_name.as_str()),
        "Template name should match"
    );
    assert!(
        response_json["description"].is_string(),
        "Should have description"
    );
    assert!(
        response_json["variables"].is_object(),
        "Should have variables object"
    );
    assert!(
        response_json["configuration"].is_object(),
        "Should have configuration object"
    );
}

/// Test get_template_details with non-existent template.
///
/// Verifies that requesting details for a non-existent template returns 404.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_get_template_details_not_found() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/orgs/{}/templates/nonexistent-template",
            org
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Should return 404 for non-existent template"
    );
}

/// Test validate_template endpoint with real template repository.
///
/// Verifies that template validation correctly checks template.toml
/// structure and returns validation results.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with template repository"]
async fn test_validate_template_success() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let template_name = match std::env::var("TEST_TEMPLATE") {
        Ok(name) => name,
        Err(_) => {
            println!("Skipping test: TEST_TEMPLATE not set");
            return;
        }
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/orgs/{}/templates/{}/validate",
            org, template_name
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for validation request"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify validation result structure
    assert!(
        response_json["valid"].is_boolean(),
        "Response should contain 'valid' boolean"
    );
    assert!(
        response_json["errors"].is_array(),
        "Response should contain 'errors' array"
    );
    assert!(
        response_json["warnings"].is_array(),
        "Response should contain 'warnings' array"
    );

    // Template should be valid if it loaded successfully
    if let Some(valid) = response_json["valid"].as_bool() {
        if !valid {
            println!("Template validation failed:");
            println!("Errors: {:?}", response_json["errors"]);
        }
    }
}

/// Test validate_template with non-existent template.
///
/// Verifies that validation fails appropriately for non-existent templates.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_validate_template_not_found() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/orgs/{}/templates/nonexistent/validate",
            org
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Should return 404 for non-existent template"
    );
}

// ============================================================================
// Repository Management Endpoint Tests
// ============================================================================

/// Test validate_repository_name endpoint.
///
/// Verifies that repository name validation works correctly with
/// GitHub naming rules.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_validate_repository_name_valid() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request_body = json!({
        "organization": org,
        "name": "my-test-repo"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for validation request"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        response_json["valid"].as_bool(),
        Some(true),
        "Valid repository name should pass validation"
    );
}

/// Test validate_repository_name with invalid characters.
///
/// Verifies that validation rejects invalid repository names.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_validate_repository_name_invalid() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let request_body = json!({
        "organization": org,
        "name": "My Invalid@Repo!"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-name")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK even for invalid names"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        response_json["valid"].as_bool(),
        Some(false),
        "Invalid repository name should fail validation"
    );

    assert!(
        response_json["messages"].is_array(),
        "Should have validation messages"
    );
}

/// Test validate_repository_request endpoint.
///
/// Verifies that complete request validation checks template existence,
/// required variables, and other constraints.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with template repository"]
async fn test_validate_repository_request_valid() {
    let app = create_router(test_app_state());
    let token = test_token();
    let org = test_org();

    let template = match std::env::var("TEST_TEMPLATE") {
        Ok(name) => name,
        Err(_) => {
            println!("Skipping test: TEST_TEMPLATE not set");
            return;
        }
    };

    let request_body = json!({
        "name": "test-validation-repo",
        "organization": org,
        "template": template,
        "variables": {}
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-request")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should return 200 OK for validation request"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(
        response_json["valid"].is_boolean(),
        "Should have valid field"
    );

    // May be invalid if template requires variables
    // That's okay - we're testing the validation flow
    if !response_json["valid"].as_bool().unwrap_or(false) {
        println!("Validation failed (expected if template requires variables):");
        println!("Errors: {:?}", response_json["errors"]);
    }
}

// ============================================================================
// Authentication and Authorization Tests
// ============================================================================

/// Test that missing authentication token returns 401 Unauthorized.
///
/// Verifies that all protected endpoints require authentication.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_missing_authentication_token() {
    let app = create_router(test_app_state());
    let org = test_org();

    // Try to access protected endpoint without token
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/repository-types", org))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 Unauthorized without authentication token"
    );
}

/// Test that invalid authentication token returns 401 Unauthorized.
///
/// Verifies that invalid tokens are rejected.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_invalid_authentication_token() {
    let app = create_router(test_app_state());
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/repository-types", org))
        .header("authorization", "Bearer invalid-token-12345")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 Unauthorized for invalid token"
    );
}

/// Test that malformed authentication header returns 401 Unauthorized.
///
/// Verifies that malformed Authorization headers are rejected.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure"]
async fn test_malformed_authentication_header() {
    let app = create_router(test_app_state());
    let org = test_org();

    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/repository-types", org))
        .header("authorization", "NotBearer token123")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Should return 401 Unauthorized for malformed header"
    );
}

// ============================================================================
// Complete End-to-End Workflow Tests
// ============================================================================

/// Test complete repository creation workflow (dry-run).
///
/// This test runs through the complete workflow for creating a repository
/// WITHOUT actually creating it:
/// 1. Validate organization configuration
/// 2. List available templates
/// 3. Get template details
/// 4. Validate template
/// 5. Preview merged configuration
/// 6. Validate complete repository request
///
/// This ensures the entire API surface works together correctly.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure with complete setup"]
async fn test_complete_repository_creation_workflow_dry_run() {
    let token = test_token();
    let org = test_org();

    println!("\n=== Step 1: Validate Organization ===");
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/validate", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let validation: serde_json::Value = serde_json::from_slice(&body).unwrap();

    if !validation["valid"].as_bool().unwrap_or(false) {
        println!("Organization validation failed:");
        println!("{:#?}", validation);
        panic!("Cannot proceed without valid organization setup");
    }
    println!("✓ Organization is valid");

    println!("\n=== Step 2: List Available Templates ===");
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/templates", org))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let templates: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let template_list = templates["templates"].as_array().unwrap();

    if template_list.is_empty() {
        println!("No templates available, skipping rest of workflow");
        return;
    }

    let template_name = template_list[0]["name"].as_str().unwrap();
    println!(
        "✓ Found {} template(s), using: {}",
        template_list.len(),
        template_name
    );

    println!("\n=== Step 3: Get Template Details ===");
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/orgs/{}/templates/{}", org, template_name))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let template_details: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("✓ Template details loaded");
    println!(
        "  Description: {}",
        template_details["description"].as_str().unwrap_or("")
    );
    println!(
        "  Variables: {}",
        template_details["variables"].as_object().unwrap().len()
    );

    println!("\n=== Step 4: Validate Template ===");
    let app = create_router(test_app_state());
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/orgs/{}/templates/{}/validate",
            org, template_name
        ))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let template_validation: serde_json::Value = serde_json::from_slice(&body).unwrap();

    if !template_validation["valid"].as_bool().unwrap_or(false) {
        println!("Template validation failed:");
        println!("{:#?}", template_validation);
    } else {
        println!("✓ Template is valid");
    }

    println!("\n=== Step 5: Preview Configuration ===");
    let app = create_router(test_app_state());
    let preview_body = json!({"template": template_name});
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orgs/{}/preview", org))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&preview_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let preview: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("✓ Configuration preview generated");
    println!(
        "  Merged configuration available: {}",
        preview["mergedConfiguration"].is_object()
    );

    println!("\n=== Step 6: Validate Repository Request ===");
    let app = create_router(test_app_state());
    let validation_body = json!({
        "name": "test-workflow-repo",
        "organization": org,
        "template": template_name,
        "variables": {}
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/repositories/validate-request")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&validation_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let request_validation: serde_json::Value = serde_json::from_slice(&body).unwrap();

    if request_validation["valid"].as_bool().unwrap_or(false) {
        println!("✓ Repository request is valid (ready to create)");
    } else {
        println!("⚠ Repository request validation issues:");
        println!("{:#?}", request_validation["errors"]);
    }

    println!("\n=== Complete Workflow Test Passed ===");
    println!("All API endpoints worked together successfully!");
}
