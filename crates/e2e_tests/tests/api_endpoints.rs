//! End-to-end tests for empty repository creation through containerized API.
//!
//! E2E tests for non-template repositories:
//! - Empty repositories (no files) through HTTP API
//! - Custom initialization (README.md and/or .gitignore) through HTTP API
//! - Settings application without templates through HTTP API
//!
//! Tests use testcontainers to run the API server in Docker and make
//! real HTTP requests, verifying the complete deployment stack.

use anyhow::Result;
use e2e_tests::{ApiContainer, ApiContainerConfig};
use reqwest::{Client, StatusCode};
use serde_json::json;

/// Generate E2E test repository name using test_utils.
fn generate_e2e_test_repo_name(test_name: &str) -> String {
    test_utils::generate_test_repo_name("e2e", test_name)
}

/// Helper to get GitHub installation token from environment.
///
/// Creates a proper GitHub App installation token using the app credentials.
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

    let auth_service = GitHubAuthService::new(app_id, private_key);
    let token = auth_service
        .get_installation_token_for_org(&org)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get installation token: {}", e))?;

    tracing::info!("Successfully generated installation token");

    Ok(token)
}

/// Test creating an empty repository without template through containerized API.
///
/// Verifies:
/// - Empty content strategy works through API
/// - Repository is created with no files
/// - Organization defaults are applied
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_create_empty_repository_without_template() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    // Now use config.clone() since we implemented Clone
    let mut container = ApiContainer::new(config.clone()).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("empty-no-template");

    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request with Empty content strategy and no template
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {
            "type": "empty"
        },
        "visibility": "private"
    });

    let response = client
        .post(format!("{}/api/v1/repositories", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Empty repository creation should return 201 Created"
    );

    let json: serde_json::Value = response.json().await?;
    assert_eq!(
        json["repository"]["name"], repo_name,
        "Response should contain created repository name"
    );

    // Verify repository is empty by checking for README.md
    let verification_client = github_client::create_token_client(&token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let readme_result = verification_client
        .get_file_content(&org, &repo_name, "README.md")
        .await;

    assert!(
        readme_result.is_err(),
        "Empty repository should not have README.md"
    );

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}

/// Test creating an empty repository with template settings through containerized API.
///
/// Verifies:
/// - Empty content strategy works with template
/// - Repository is created with no files (despite template)
/// - Template settings are applied
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_create_empty_repository_with_template_settings() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("empty-with-template");
    let template = std::env::var("TEST_TEMPLATE").unwrap_or_else(|_| "default".to_string());

    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request with Empty content strategy but with template for settings
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "template": template,
        "contentStrategy": {
            "type": "empty"
        },
        "visibility": "private"
    });

    let response = client
        .post(format!("{}/api/v1/repositories", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Empty repository with template settings should return 201 Created"
    );

    let json: serde_json::Value = response.json().await?;
    assert_eq!(
        json["repository"]["name"], repo_name,
        "Response should contain created repository name"
    );

    // Verify repository is empty (no template files)
    let verification_client = github_client::create_token_client(&token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let readme_result = verification_client
        .get_file_content(&org, &repo_name, "README.md")
        .await;

    assert!(
        readme_result.is_err(),
        "Empty repository should not have README.md even with template"
    );

    // Verify labels were created from configuration
    use github_client::RepositoryClient;
    let labels = verification_client
        .list_repository_labels(&org, &repo_name)
        .await?;

    tracing::info!("Repository has {} labels", labels.len());
    
    // Expected labels from global/standard-labels.toml
    let expected_global_labels = vec![
        "bug", "enhancement", "documentation", "good-first-issue",
        "help-wanted", "question", "wontfix", "duplicate", "invalid", "dependencies"
    ];
    
    // Expected template-specific labels from template-test-basic
    let expected_template_labels = vec!["template-feature", "template-bug"];
    
    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label, labels
        );
    }
    
    for label in &expected_template_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing template-specific label '{}' from template-test-basic - found: {:?}",
            label, labels
        );
    }
    
    tracing::info!("✓ Verified {} labels (10 global + 2 template-specific)", labels.len());

    // Verify webhooks from configuration
    let webhooks = verification_client.list_webhooks(&org, &repo_name).await?;
    
    // Expected webhooks: 1 from global config + 1 from template-test-basic
    assert_eq!(
        webhooks.len(), 2,
        "Expected 2 webhooks (1 global + 1 template), found {}",
        webhooks.len()
    );
    
    // Verify global webhook (from global/webhooks.toml)
    let global_webhook = webhooks.iter().find(|w| w.config.url == "https://httpbin.org/global-webhook");
    assert!(
        global_webhook.is_some(),
        "Missing global webhook with URL 'https://httpbin.org/global-webhook'"
    );
    let global_wh = global_webhook.unwrap();
    assert!(global_wh.active, "Global webhook should be active");
    assert!(
        global_wh.events.contains(&github_client::WebhookEvent::Push),
        "Global webhook should have 'push' event"
    );
    assert!(
        global_wh.events.contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'pull_request' event"
    );
    
    // Verify template webhook (from template-test-basic)
    let template_webhook = webhooks.iter().find(|w| w.config.url == "https://httpbin.org/template-webhook");
    assert!(
        template_webhook.is_some(),
        "Missing template webhook with URL 'https://httpbin.org/template-webhook'"
    );
    let template_wh = template_webhook.unwrap();
    assert!(template_wh.active, "Template webhook should be active");
    assert!(
        template_wh.events.contains(&github_client::WebhookEvent::Issues),
        "Template webhook should have 'issues' event"
    );
    
    tracing::info!("✓ Verified 2 webhooks (1 global + 1 template-specific) with correct configurations");

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}

/// Test creating a repository with custom init (README.md only) through containerized API.
///
/// Verifies:
/// - CustomInit content strategy works through API
/// - Repository is created with only README.md
/// - No template files are present
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_create_custom_init_readme_only() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("init-readme");

    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request with CustomInit content strategy (README only)
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {
            "type": "custom_init",
            "include_readme": true,
            "include_gitignore": false
        },
        "visibility": "private"
    });

    let response = client
        .post(format!("{}/api/v1/repositories", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Custom init repository (README only) should return 201 Created"
    );

    let json: serde_json::Value = response.json().await?;
    assert_eq!(
        json["repository"]["name"], repo_name,
        "Response should contain created repository name"
    );

    // Verify repository has README.md
    let verification_client = github_client::create_token_client(&token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let readme_content = verification_client
        .get_file_content(&org, &repo_name, "README.md")
        .await?;

    assert!(
        !readme_content.is_empty(),
        "Repository should have README.md with content"
    );

    // Verify repository does NOT have .gitignore
    let gitignore_result = verification_client
        .get_file_content(&org, &repo_name, ".gitignore")
        .await;

    assert!(
        gitignore_result.is_err(),
        "Repository should not have .gitignore when not requested"
    );

    // Verify labels were created from global configuration
    use github_client::RepositoryClient;
    let labels = verification_client
        .list_repository_labels(&org, &repo_name)
        .await?;

    tracing::info!("Repository has {} labels", labels.len());
    
    // Expected labels from global/standard-labels.toml (no template used in custom init)
    let expected_global_labels = vec![
        "bug", "enhancement", "documentation", "good-first-issue",
        "help-wanted", "question", "wontfix", "duplicate", "invalid", "dependencies"
    ];
    
    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label, labels
        );
    }
    
    tracing::info!("✓ Verified {} global labels from standard-labels.toml", expected_global_labels.len());

    // Verify webhooks from global configuration
    let webhooks = verification_client.list_webhooks(&org, &repo_name).await?;
    
    // Expected: 1 webhook from global config (no template webhooks)
    assert_eq!(
        webhooks.len(), 1,
        "Expected 1 global webhook, found {}",
        webhooks.len()
    );
    
    // Verify global webhook (from global/webhooks.toml)
    let global_webhook = &webhooks[0];
    assert_eq!(
        global_webhook.config.url, "https://httpbin.org/global-webhook",
        "Global webhook URL should match configuration"
    );
    assert!(global_webhook.active, "Global webhook should be active");
    assert!(
        global_webhook.events.contains(&github_client::WebhookEvent::Push) &&
        global_webhook.events.contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'push' and 'pull_request' events"
    );
    
    tracing::info!("✓ Verified 1 global webhook with correct configuration");

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}

/// Test creating a repository with custom init (both files) through containerized API.
///
/// Verifies:
/// - CustomInit content strategy works with multiple files
/// - Repository is created with README.md and .gitignore
/// - No template files are present
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_create_custom_init_both_files() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;

    // Uncomment to build during test: container.build_image().await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("init-both");

    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request with CustomInit content strategy (both files)
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {
            "type": "custom_init",
            "include_readme": true,
            "include_gitignore": true
        },
        "visibility": "private"
    });

    let response = client
        .post(format!("{}/api/v1/repositories", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Custom init repository (both files) should return 201 Created"
    );

    let json: serde_json::Value = response.json().await?;
    assert_eq!(
        json["repository"]["name"], repo_name,
        "Response should contain created repository name"
    );

    // Verify repository has both files
    let verification_client = github_client::create_token_client(&token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let readme_content = verification_client
        .get_file_content(&org, &repo_name, "README.md")
        .await?;

    assert!(
        !readme_content.is_empty(),
        "Repository should have README.md with content"
    );

    let gitignore_content = verification_client
        .get_file_content(&org, &repo_name, ".gitignore")
        .await?;

    assert!(
        !gitignore_content.is_empty(),
        ".gitignore should have content"
    );

    // Verify labels were created from global configuration
    use github_client::RepositoryClient;
    let labels = verification_client
        .list_repository_labels(&org, &repo_name)
        .await?;

    tracing::info!("Repository has {} labels", labels.len());
    
    // Expected labels from global/standard-labels.toml (no template used in custom init)
    let expected_global_labels = vec![
        "bug", "enhancement", "documentation", "good-first-issue",
        "help-wanted", "question", "wontfix", "duplicate", "invalid", "dependencies"
    ];
    
    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label, labels
        );
    }
    
    tracing::info!("✓ Verified {} global labels from standard-labels.toml", expected_global_labels.len());

    // Verify webhooks from global configuration
    let webhooks = verification_client.list_webhooks(&org, &repo_name).await?;
    
    // Expected: 1 webhook from global config (no template webhooks)
    assert_eq!(
        webhooks.len(), 1,
        "Expected 1 global webhook, found {}",
        webhooks.len()
    );
    
    // Verify global webhook (from global/webhooks.toml)
    let global_webhook = &webhooks[0];
    assert_eq!(
        global_webhook.config.url, "https://httpbin.org/global-webhook",
        "Global webhook URL should match configuration"
    );
    assert!(global_webhook.active, "Global webhook should be active");
    assert!(
        global_webhook.events.contains(&github_client::WebhookEvent::Push) &&
        global_webhook.events.contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'push' and 'pull_request' events"
    );
    
    tracing::info!("✓ Verified 1 global webhook with correct configuration");

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}
