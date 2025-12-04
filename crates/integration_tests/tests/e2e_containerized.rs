//! End-to-end integration tests using containerized API server
//!
//! These tests verify the complete deployment stack by:
//! 1. Starting a container with the API server (assumes image is built)
//! 2. Making HTTP requests via reqwest
//! 3. Verifying responses and GitHub state
//!
//! ## Running Tests
//!
//! ### In CI/CD (Image Already Built)
//! ```bash
//! # Workflow builds image first, then runs tests
//! cargo test -p integration_tests --test e2e_containerized -- --ignored --test-threads=1
//! ```
//!
//! ### Local Development (Build Image First)
//! ```bash
//! # Option 1: Build image with Docker directly (recommended)
//! docker build -t repo_roller_api:test -f crates/repo_roller_api/Dockerfile .
//! cargo test -p integration_tests --test e2e_containerized -- --ignored --test-threads=1
//!
//! # Option 2: Let test build image (slower, uncomment build_image() calls)
//! cargo test -p integration_tests --test e2e_containerized -- --ignored --test-threads=1
//! ```

use anyhow::Result;
use integration_tests::container::{ApiContainer, ApiContainerConfig};
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Helper to get GitHub installation token from environment.
///
/// Creates a proper GitHub App installation token using the app credentials.
/// This is required because the auth middleware validates tokens by calling
/// list_installations(), which requires an installation token (not a PAT).
async fn get_github_installation_token() -> Result<String> {
    use auth_handler::{GitHubAuthService, UserAuthenticationService};

    let app_id = std::env::var("GITHUB_APP_ID")
        .map_err(|_| anyhow::anyhow!("GITHUB_APP_ID not set"))?
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("GITHUB_APP_ID must be a valid number"))?;

    let private_key = std::env::var("GITHUB_APP_PRIVATE_KEY")
        .map_err(|_| anyhow::anyhow!("GITHUB_APP_PRIVATE_KEY not set"))?;

    let org = std::env::var("TEST_ORG").map_err(|_| anyhow::anyhow!("TEST_ORG not set"))?;

    tracing::info!("Generating installation token for org: {}", org);

    // Create auth service and get installation token for the test organization
    let auth_service = GitHubAuthService::new(app_id, private_key);
    let token = auth_service
        .get_installation_token_for_org(&org)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get installation token: {}", e))?;

    tracing::info!("Successfully generated installation token");

    Ok(token)
}

/// Helper to assert response status with detailed error logging.
async fn assert_status_with_body(
    response: reqwest::Response,
    expected: StatusCode,
    context: &str,
) -> Result<reqwest::Response> {
    let status = response.status();
    if status != expected {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read body>".to_string());
        panic!(
            "{}\n  Expected: {}\n  Got: {}\n  Response body: {}",
            context, expected, status, body
        );
    }
    Ok(response)
}

// ============================================================================
// Health Check Tests
// ============================================================================

/// Test that the containerized API server responds to health checks.
///
/// This verifies:
/// - Container starts and binds to correct port
/// - Health endpoint is accessible
/// - Server responds correctly to HTTP requests
///
/// Note: Assumes Docker image "repo_roller_api:test" is already built.
#[tokio::test]
#[ignore = "Requires Docker daemon running and pre-built image"]
async fn test_e2e_health_check() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let mut container = ApiContainer::new(config).await?;

    // Start container (image should be pre-built)
    // Uncomment to build during test (slower): container.build_image().await?;
    let base_url = container.start().await?;

    // Test health endpoint
    let client = Client::new();
    let response = client
        .get(format!("{}/api/v1/health", base_url))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Health check should return 200 OK"
    );

    // Cleanup
    container.stop().await?;
    Ok(())
}

// ============================================================================
// Organization Settings E2E Tests
// ============================================================================

/// Test listing repository types through containerized API.
///
/// Verifies:
/// - Authentication middleware works in container
/// - Organization settings endpoints accessible
/// - Response serialization correct
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image"]
async fn test_e2e_list_repository_types() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let client = Client::new();
    let token = get_github_installation_token().await?;
    let response = client
        .get(format!("{}/api/v1/orgs/{}/repository-types", base_url, org))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let response = assert_status_with_body(
        response,
        StatusCode::OK,
        "Should return 200 OK for repository types",
    )
    .await?;

    let json: serde_json::Value = response.json().await?;
    assert!(
        json["types"].is_array(),
        "Response should contain 'types' array"
    );

    container.stop().await?;
    Ok(())
}

/// Test getting global defaults through containerized API.
///
/// Verifies:
/// - Configuration system works in container
/// - Metadata repository access works
/// - Configuration serialization correct
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image"]
async fn test_e2e_get_global_defaults() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let client = Client::new();
    let token = get_github_installation_token().await?;
    let response = client
        .get(format!("{}/api/v1/orgs/{}/defaults", base_url, org))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let response = assert_status_with_body(
        response,
        StatusCode::OK,
        "Should return 200 OK for global defaults",
    )
    .await?;

    let json: serde_json::Value = response.json().await?;
    assert!(
        json["defaults"]["repository"].is_object(),
        "Response should contain 'defaults.repository' settings. Got: {}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", json))
    );

    container.stop().await?;
    Ok(())
}

// ============================================================================
// Configuration Preview E2E Tests
// ============================================================================

/// Test configuration preview through containerized API.
///
/// Verifies:
/// - Configuration merging works in container
/// - Preview endpoint returns merged configuration
/// - Source tracking included in response
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image"]
async fn test_e2e_configuration_preview() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());

    let client = Client::new();
    let token = get_github_installation_token().await?;
    let request_body = json!({
        "template": template
    });

    let response = client
        .post(format!(
            "{}/api/v1/orgs/{}/configuration/preview",
            base_url, org
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response = assert_status_with_body(
        response,
        StatusCode::OK,
        "Configuration preview should return 200 OK",
    )
    .await?;

    let json: serde_json::Value = response.json().await?;
    assert!(
        json["mergedConfiguration"].is_object(),
        "Response should contain 'mergedConfiguration'. Got: {}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", json))
    );
    assert!(
        json["sources"].is_object(),
        "Response should contain 'sources' tracking. Got: {}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| format!("{:?}", json))
    );

    container.stop().await?;
    Ok(())
}

// ============================================================================
// Repository Creation E2E Tests
// ============================================================================

/// Test complete repository creation workflow through containerized API.
///
/// This is the most comprehensive E2E test, verifying:
/// - Complete HTTP request/response cycle
/// - Repository creation in GitHub
/// - Configuration application
/// - Response includes repository metadata
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
///
/// **Prerequisites**: TEST_TEMPLATE environment variable must reference an existing
/// template in the metadata repository (e.g., templates/template-test-basic/).
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image - CREATES REAL REPOSITORY"]
async fn test_e2e_create_repository_with_global_defaults() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let repo_name = format!("e2e-test-global-{}", chrono::Utc::now().timestamp());
    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());

    // List available templates first to help debug
    let client = Client::new();
    let token = get_github_installation_token().await?;

    let list_response = client
        .get(format!("{}/api/v1/orgs/{}/templates", base_url, org))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if list_response.status().is_success() {
        if let Ok(json) = list_response.json::<serde_json::Value>().await {
            if let Some(templates) = json["templates"].as_array() {
                let available: Vec<&str> = templates
                    .iter()
                    .filter_map(|t| t["name"].as_str())
                    .collect();
                tracing::info!("Available templates: {:?}", available);
                tracing::info!("Requesting template: {}", template);
            }
        }
    }
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "template": template,
        "visibility": "private"
    });

    let response = client
        .post(format!("{}/api/v1/repositories", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response = assert_status_with_body(
        response,
        StatusCode::CREATED,
        "Repository creation should return 201 Created",
    )
    .await?;

    let json: serde_json::Value = response.json().await?;
    assert_eq!(
        json["repository"]["name"], repo_name,
        "Response should contain created repository name"
    );
    assert!(
        json["repository"]["url"].is_string(),
        "Response should contain repository URL"
    );

    // TODO: Add verification using integration_tests::verification module
    // - Load expected configuration from metadata repository
    // - Verify custom properties were set
    // - Verify repository settings match global defaults

    tracing::info!("✓ Created repository via E2E test: {}", repo_name);
    tracing::warn!(
        "⚠ Remember to clean up test repository: {}/{}",
        org,
        repo_name
    );

    container.stop().await?;
    Ok(())
}

// ============================================================================
// Template Discovery E2E Tests
// ============================================================================

/// Test listing templates through containerized API.
///
/// Verifies:
/// - Template discovery works in container
/// - GitHub topic search functions correctly
/// - Template metadata serialization
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image"]
async fn test_e2e_list_templates() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let client = Client::new();
    let token = get_github_installation_token().await?;
    let response = client
        .get(format!("{}/api/v1/orgs/{}/templates", base_url, org))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Template list should return 200 OK"
    );

    let json: serde_json::Value = response.json().await?;
    assert!(
        json["templates"].is_array(),
        "Response should contain 'templates' array"
    );

    container.stop().await?;
    Ok(())
}

// ============================================================================
// Validation E2E Tests
// ============================================================================

/// Test organization validation through containerized API.
///
/// Verifies:
/// - Validation endpoints work in container
/// - Metadata repository validation
/// - Error handling for invalid configurations
#[tokio::test]
#[ignore = "Requires Docker and real GitHub infrastructure with pre-built image"]
async fn test_e2e_validate_organization() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let mut container = ApiContainer::new(config).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let client = Client::new();
    let token = get_github_installation_token().await?;
    let response = client
        .post(format!("{}/api/v1/orgs/{}/validate", base_url, org))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Organization validation should return 200 OK"
    );

    let json: serde_json::Value = response.json().await?;
    assert!(
        json["valid"].is_boolean(),
        "Response should contain 'valid' boolean"
    );

    container.stop().await?;
    Ok(())
}
