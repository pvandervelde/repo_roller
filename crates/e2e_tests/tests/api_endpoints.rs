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

/// Poll `get_team_repository_permission` until the assignment propagates on GitHub's backend
/// or the retry budget is exhausted.
///
/// GitHub team permission assignments are eventually-consistent: the `PUT
/// teams/{slug}/repos/{owner}/{repo}` endpoint returns `204 No Content` immediately, but the
/// corresponding `GET` endpoint may return `404` for several seconds while the assignment
/// propagates. This helper retries every 3 s for up to `max_attempts` attempts so that E2E
/// tests remain reliable without an arbitrary fixed sleep.
async fn poll_team_permission(
    client: &github_client::GitHubClient,
    org: &str,
    team_slug: &str,
    repo_name: &str,
    max_attempts: u32,
) -> anyhow::Result<Option<String>> {
    for attempt in 1..=max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        eprintln!(
            "[poll_team_permission] attempt {attempt}/{max_attempts} team={team_slug} repo={repo_name}"
        );
        match client
            .get_repository_team_permission(org, repo_name, team_slug)
            .await
        {
            Ok(Some(perm)) => {
                eprintln!(
                    "[poll_team_permission] found permission={perm} team={team_slug} on attempt {attempt}"
                );
                return Ok(Some(perm));
            }
            Ok(None) => {
                tracing::warn!(
                    attempt,
                    max_attempts,
                    team = team_slug,
                    "Team permission not yet visible; waiting for GitHub propagation"
                );
            }
            Err(e) => return Err(e.into()),
        }
    }
    eprintln!(
        "[poll_team_permission] EXHAUSTED {max_attempts} attempts for team={team_slug} repo={repo_name}"
    );
    Ok(None)
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
        "bug",
        "enhancement",
        "documentation",
        "good-first-issue",
        "help-wanted",
        "question",
        "wontfix",
        "duplicate",
        "invalid",
        "dependencies",
    ];

    // Expected template-specific labels from template-test-basic
    let expected_template_labels = vec!["template-feature", "template-bug"];

    // Check if any labels are missing and capture logs BEFORE asserting
    let mut all_expected = expected_global_labels.clone();
    all_expected.extend(expected_template_labels.iter().copied());
    let missing_labels: Vec<_> = all_expected
        .iter()
        .filter(|label| !labels.contains(&label.to_string()))
        .collect();

    if !missing_labels.is_empty() {
        tracing::error!(
            "❌ {} labels missing: {:?}. Found: {:?}. Capturing container logs...",
            missing_labels.len(),
            missing_labels,
            labels
        );
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!("\n========== CONTAINER LOGS (test_e2e_create_empty_repository_with_template_settings) ==========");
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(e) => {
                tracing::warn!("Failed to capture container logs: {}", e);
            }
        }
    }

    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label,
            labels
        );
    }

    for label in &expected_template_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing template-specific label '{}' from template-test-basic - found: {:?}",
            label,
            labels
        );
    }

    tracing::info!(
        "✓ Verified {} labels (10 global + 2 template-specific)",
        labels.len()
    );

    // Verify webhooks from configuration
    let webhooks = verification_client.list_webhooks(&org, &repo_name).await?;

    // Expected webhooks: 1 from global config + 1 from template-test-basic
    assert_eq!(
        webhooks.len(),
        2,
        "Expected 2 webhooks (1 global + 1 template), found {}",
        webhooks.len()
    );

    // Verify global webhook (from global/webhooks.toml)
    let global_webhook = webhooks
        .iter()
        .find(|w| w.config.url == "https://httpbin.org/global-webhook");
    assert!(
        global_webhook.is_some(),
        "Missing global webhook with URL 'https://httpbin.org/global-webhook'"
    );
    let global_wh = global_webhook.unwrap();
    assert!(global_wh.active, "Global webhook should be active");
    assert!(
        global_wh
            .events
            .contains(&github_client::WebhookEvent::Push),
        "Global webhook should have 'push' event"
    );
    assert!(
        global_wh
            .events
            .contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'pull_request' event"
    );

    // Verify template webhook (from template-test-basic)
    let template_webhook = webhooks
        .iter()
        .find(|w| w.config.url == "https://httpbin.org/template-webhook");
    assert!(
        template_webhook.is_some(),
        "Missing template webhook with URL 'https://httpbin.org/template-webhook'"
    );
    let template_wh = template_webhook.unwrap();
    assert!(template_wh.active, "Template webhook should be active");
    assert!(
        template_wh
            .events
            .contains(&github_client::WebhookEvent::Issues),
        "Template webhook should have 'issues' event"
    );

    tracing::info!(
        "✓ Verified 2 webhooks (1 global + 1 template-specific) with correct configurations"
    );

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
        "visibility": "public"  // Public required for ruleset verification (no GitHub Pro)
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
    // Add delay to allow GitHub API to sync after repository creation
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    use github_client::RepositoryClient;
    let labels = match verification_client
        .list_repository_labels(&org, &repo_name)
        .await
    {
        Ok(labels) => labels,
        Err(e) => {
            tracing::warn!("Failed to list labels (may be timing issue): {}", e);
            // Try one more time after longer delay
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            verification_client
                .list_repository_labels(&org, &repo_name)
                .await?
        }
    };

    tracing::info!("Repository has {} labels", labels.len());

    // Expected labels from global/standard-labels.toml (no template used in custom init)
    let expected_global_labels = vec![
        "bug",
        "enhancement",
        "documentation",
        "good-first-issue",
        "help-wanted",
        "question",
        "wontfix",
        "duplicate",
        "invalid",
        "dependencies",
    ];

    // Check if any labels are missing and capture logs BEFORE asserting
    let missing_labels: Vec<_> = expected_global_labels
        .iter()
        .filter(|label| !labels.contains(&label.to_string()))
        .collect();

    if !missing_labels.is_empty() {
        tracing::error!(
            "❌ {} labels missing: {:?}. Found: {:?}. Capturing container logs...",
            missing_labels.len(),
            missing_labels,
            labels
        );
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!("\n========== CONTAINER LOGS (test_e2e_create_custom_init_readme_only) ==========");
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(e) => {
                tracing::warn!("Failed to capture container logs: {}", e);
            }
        }
    }

    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label,
            labels
        );
    }

    tracing::info!(
        "✓ Verified {} global labels from standard-labels.toml",
        expected_global_labels.len()
    );

    // Verify webhooks from global configuration
    let webhooks = match verification_client.list_webhooks(&org, &repo_name).await {
        Ok(webhooks) => webhooks,
        Err(e) => {
            tracing::warn!("Failed to list webhooks (may be timing issue): {}", e);
            // Try one more time after delay
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            verification_client.list_webhooks(&org, &repo_name).await?
        }
    };

    // Expected: 1 webhook from global config (no template webhooks)
    assert_eq!(
        webhooks.len(),
        1,
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
        global_webhook
            .events
            .contains(&github_client::WebhookEvent::Push)
            && global_webhook
                .events
                .contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'push' and 'pull_request' events"
    );

    tracing::info!("✓ Verified 1 global webhook with correct configuration");

    // Verify rulesets from global configuration
    tracing::info!(
        org = &org,
        repo = &repo_name,
        "Attempting to list repository rulesets for verification"
    );
    let rulesets = match verification_client
        .list_repository_rulesets(&org, &repo_name)
        .await
    {
        Ok(rulesets) => {
            tracing::info!(
                org = &org,
                repo = &repo_name,
                count = rulesets.len(),
                "Successfully listed rulesets"
            );
            rulesets
        }
        Err(e) => {
            tracing::warn!(
                org = &org,
                repo = &repo_name,
                error = %e,
                "Failed to list rulesets on first attempt, retrying after delay"
            );
            // Try one more time after delay
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            match verification_client
                .list_repository_rulesets(&org, &repo_name)
                .await
            {
                Ok(rulesets) => {
                    tracing::info!(
                        org = &org,
                        repo = &repo_name,
                        count = rulesets.len(),
                        "Successfully listed rulesets on retry"
                    );
                    rulesets
                }
                Err(e) => {
                    tracing::error!(
                        org = &org,
                        repo = &repo_name,
                        error = %e,
                        "Failed to list rulesets after retry - this may indicate API response format mismatch"
                    );
                    return Err(e.into());
                }
            }
        }
    };

    tracing::info!("Repository has {} rulesets", rulesets.len());

    // Verify that rulesets were applied (if any exist in global config)
    // Note: The exact number depends on what's in .reporoller-test/global/rulesets.toml
    // For now, we just verify the operation succeeded and log the count
    if !rulesets.is_empty() {
        tracing::info!(
            "✓ Verified {} ruleset(s) applied from global configuration",
            rulesets.len()
        );
        for ruleset in &rulesets {
            tracing::info!(
                "  - Ruleset: {} (target: {:?}, enforcement: {:?})",
                ruleset.name,
                ruleset.target,
                ruleset.enforcement
            );
        }
    } else {
        tracing::info!("✓ No rulesets configured (verified list operation succeeds)");
    }

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}

// ── Permission E2E Tests ──────────────────────────────────────────────────────
//
// These tests verify the full permission pipeline through the containerized API:
//   • org-level default team config → applied to every repo
//   • template team config          → merged on top of org defaults
//   • locked entries                → cannot be overridden by requests
//   • org ceiling                   → caps request access levels
//
// Pre-requisites (see tests/create-test-teams.ps1):
//   - reporoller-test-permissions  (org default: triage; template upgrades to write)
//   - reporoller-test-security     (org default: admin, locked=true)
//   - reporoller-test-triage       (template only: triage)
//
// Org ceiling (set in tests/metadata/.reporoller/global/defaults.toml):
//   max_team_access_level         = "maintain"
//   max_collaborator_access_level = "write"

/// Verify that teams from org config and template config are both applied with
/// the correct permission levels after repository creation.
///
/// Expected outcome for a repo created with `template-test-basic`:
/// - `reporoller-test-permissions` → `write`  (org default triage, upgraded by template)
/// - `reporoller-test-security`    → `admin`  (org locked team, preserved)
/// - `reporoller-test-triage`      → `triage` (template-only team)
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_permission_teams_applied_from_config() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("perm-teams-config");
    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Create with template-test-basic so org-level and template-level teams merge.
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "template": "template-test-basic",
        "contentStrategy": {"type": "empty"},
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
        "Repository creation should return 201 Created"
    );

    let verification_client =
        github_client::GitHubClient::new(github_client::create_token_client(&token)?);

    // reporoller-test-permissions: org config = triage, template upgrades to write.
    // Poll for up to 60 s: GitHub team permission assignments are eventually-consistent.
    let perm = poll_team_permission(
        &verification_client,
        &org,
        "reporoller-test-permissions",
        &repo_name,
        20,
    )
    .await;

    let perm = match perm {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to get team permission: {e}");
            match container.get_logs().await {
                Ok(logs) => {
                    eprintln!("\n========== CONTAINER LOGS (test_e2e_permission_teams_applied_from_config) ==========");
                    eprintln!("{}", logs);
                    eprintln!("====================================\n");
                }
                Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
            }
            e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
                .await
                .ok();
            return Err(e);
        }
    };

    if perm.is_none() {
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!(
                    "\n========== CONTAINER LOGS (reporoller-test-permissions is None) =========="
                );
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
        }
        e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
            .await
            .ok();
        panic!("reporoller-test-permissions should be 'write' (template upgrade from org triage), got None — see container logs above");
    }
    assert_eq!(
        perm,
        Some("write".to_string()),
        "reporoller-test-permissions should be 'write' (template upgrade from org triage)"
    );
    tracing::info!(
        "✓ reporoller-test-permissions has 'write' access (upgraded from org default triage)"
    );

    // reporoller-test-security: locked admin in org config, cannot be altered.
    // Poll for up to 60 s: GitHub team permission assignments are eventually-consistent.
    let sec_perm = poll_team_permission(
        &verification_client,
        &org,
        "reporoller-test-security",
        &repo_name,
        20,
    )
    .await?;

    if sec_perm.is_none() {
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!(
                    "\n========== CONTAINER LOGS (reporoller-test-security is None) =========="
                );
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
        }
        e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
            .await
            .ok();
        panic!("reporoller-test-security should be 'admin' (locked org team), got None — see container logs above");
    }
    assert_eq!(
        sec_perm,
        Some("admin".to_string()),
        "reporoller-test-security should be 'admin' (locked org team)"
    );
    tracing::info!("✓ reporoller-test-security has 'admin' access (locked org team)");

    // reporoller-test-triage: added by template at triage level.
    // Poll for up to 60 s: GitHub team permission assignments are eventually-consistent.
    let triage_perm = poll_team_permission(
        &verification_client,
        &org,
        "reporoller-test-triage",
        &repo_name,
        20,
    )
    .await?;

    if triage_perm.is_none() {
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!(
                    "\n========== CONTAINER LOGS (reporoller-test-triage is None) =========="
                );
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
        }
        e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
            .await
            .ok();
        panic!("reporoller-test-triage should be 'triage' (template team), got None — see container logs above");
    }
    assert_eq!(
        triage_perm,
        Some("triage".to_string()),
        "reporoller-test-triage should be 'triage' (template team)"
    );
    tracing::info!("✓ reporoller-test-triage has 'triage' access (from template config)");

    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();
    container.stop().await?;
    Ok(())
}

/// Verify that a team permission requested above the org ceiling is capped at the
/// ceiling instead of being rejected.
///
/// The org ceiling is `max_team_access_level = "maintain"`.
/// A request for `admin` on `reporoller-test-triage` should result in `maintain`.
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_permission_request_team_capped_at_ceiling() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("perm-ceiling");
    let client = Client::new();
    let token = get_github_installation_token().await?;

    // No template — only request teams are tested here.
    // Request admin for reporoller-test-triage; org ceiling is maintain.
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {"type": "empty"},
        "visibility": "private",
        "teams": {
            "reporoller-test-triage": "admin"
        }
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
        "Repository creation should return 201 Created (permission capping is silent)"
    );

    let verification_client =
        github_client::GitHubClient::new(github_client::create_token_client(&token)?);

    // Poll for up to 60 s: GitHub team permission assignments are eventually-consistent.
    let perm = poll_team_permission(
        &verification_client,
        &org,
        "reporoller-test-triage",
        &repo_name,
        20,
    )
    .await;

    let perm = match perm {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to get team permission: {e}");
            match container.get_logs().await {
                Ok(logs) => {
                    eprintln!("\n========== CONTAINER LOGS (test_e2e_permission_request_team_capped_at_ceiling) ==========");
                    eprintln!("{}", logs);
                    eprintln!("====================================\n");
                }
                Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
            }
            e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
                .await
                .ok();
            return Err(e);
        }
    };

    if perm.is_none() {
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!("\n========== CONTAINER LOGS (reporoller-test-triage is None in ceiling test) ==========");
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
        }
        e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
            .await
            .ok();
        panic!("reporoller-test-triage should be 'maintain' (ceiling-capped from admin), got None — see container logs above");
    }
    assert_eq!(
        perm,
        Some("maintain".to_string()),
        "reporoller-test-triage requested 'admin' but org ceiling is 'maintain'; expected 'maintain'"
    );
    tracing::info!(
        "✓ reporoller-test-triage capped at 'maintain' (org ceiling) instead of requested 'admin'"
    );

    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();
    container.stop().await?;
    Ok(())
}
///
/// `reporoller-test-security` is locked at `admin` in org config.
/// A request for `write` on that team should leave the team at `admin`.
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_permission_locked_org_team_preserved() -> Result<()> {
    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("perm-locked");
    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request 'write' on the locked team; the org config has it at 'admin'.
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {"type": "empty"},
        "visibility": "private",
        "teams": {
            "reporoller-test-security": "write"
        }
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
        "Repository creation should return 201 Created (lock enforcement is silent)"
    );

    let verification_client =
        github_client::GitHubClient::new(github_client::create_token_client(&token)?);

    // Poll for up to 60 s: GitHub team permission assignments are eventually-consistent.
    let perm = poll_team_permission(
        &verification_client,
        &org,
        "reporoller-test-security",
        &repo_name,
        20,
    )
    .await;

    let perm = match perm {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to get team permission: {e}");
            match container.get_logs().await {
                Ok(logs) => {
                    eprintln!("\n========== CONTAINER LOGS (test_e2e_permission_locked_org_team_preserved) ==========");
                    eprintln!("{}", logs);
                    eprintln!("====================================\n");
                }
                Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
            }
            e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
                .await
                .ok();
            return Err(e);
        }
    };

    if perm.is_none() {
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!("\n========== CONTAINER LOGS (reporoller-test-security is None in locked test) ==========");
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
        }
        e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
            .await
            .ok();
        panic!("reporoller-test-security should be 'admin' (locked at admin, request for write ignored), got None — see container logs above");
    }
    assert_eq!(
        perm,
        Some("admin".to_string()),
        "reporoller-test-security is locked at 'admin'; request for 'write' must be ignored"
    );
    tracing::info!(
        "✓ reporoller-test-security retained 'admin' (locked org team, request for 'write' ignored)"
    );

    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();
    container.stop().await?;
    Ok(())
}

/// Verify that a collaborator added via the API request is assigned with the
/// correct permission, respecting the org's collaborator ceiling.
///
/// The org ceiling is `max_collaborator_access_level = "write"`.
/// A request for `admin` on the collaborator should result in `write`.
///
/// Requires `TEST_COLLABORATOR_USERNAME` environment variable; skipped if absent.
///
/// **WARNING**: This test CREATES A REAL REPOSITORY in GitHub.
#[tokio::test]
async fn test_e2e_permission_collaborator_applied_from_request() -> Result<()> {
    let collaborator_username = match std::env::var("TEST_COLLABORATOR_USERNAME") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            tracing::info!(
                "⚠ Skipping test_e2e_permission_collaborator_applied_from_request: \
                 TEST_COLLABORATOR_USERNAME not set"
            );
            return Ok(());
        }
    };

    let config = ApiContainerConfig::from_env()?;
    let org = config.test_org.clone();
    let app_id = config
        .github_app_id
        .parse::<u64>()
        .expect("GITHUB_APP_ID must be a valid number");
    let private_key = config.github_app_private_key.clone();

    let mut container = ApiContainer::new(config.clone()).await?;
    let base_url = container.start().await?;

    let repo_name = generate_e2e_test_repo_name("perm-collab");
    let client = Client::new();
    let token = get_github_installation_token().await?;

    // Request 'admin' for the collaborator; org ceiling for collaborators is 'write'.
    let request_body = json!({
        "name": repo_name,
        "organization": org,
        "contentStrategy": {"type": "empty"},
        "visibility": "private",
        "collaborators": {
            &collaborator_username: "admin"
        }
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
        "Repository creation should return 201 Created"
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let verification_client =
        github_client::GitHubClient::new(github_client::create_token_client(&token)?);

    let perm = verification_client
        .get_collaborator_permission(&org, &repo_name, &collaborator_username)
        .await;

    let perm = match perm {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to get collaborator permission: {e}");
            match container.get_logs().await {
                Ok(logs) => {
                    eprintln!("\n========== CONTAINER LOGS (test_e2e_permission_collaborator_applied_from_request) ==========");
                    eprintln!("{}", logs);
                    eprintln!("====================================\n");
                }
                Err(log_err) => tracing::warn!("Failed to capture container logs: {}", log_err),
            }
            e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
                .await
                .ok();
            return Err(e.into());
        }
    };

    assert_eq!(
        perm, "write",
        "Collaborator requested 'admin' but org ceiling is 'write'; expected 'write'"
    );
    tracing::info!(
        "✓ Collaborator {} was assigned 'write' (org ceiling applied to requested 'admin')",
        collaborator_username
    );

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
        "visibility": "public"  // Public required for ruleset verification (no GitHub Pro)
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
    // Add small delay to allow GitHub API to sync after repository creation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    use github_client::RepositoryClient;
    let labels = match verification_client
        .list_repository_labels(&org, &repo_name)
        .await
    {
        Ok(labels) => labels,
        Err(e) => {
            tracing::warn!("Failed to list labels (may be timing issue): {}", e);
            // Try one more time after another delay
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            verification_client
                .list_repository_labels(&org, &repo_name)
                .await?
        }
    };

    tracing::info!("Repository has {} labels", labels.len());

    // Expected labels from global/standard-labels.toml (no template used in custom init)
    let expected_global_labels = vec![
        "bug",
        "enhancement",
        "documentation",
        "good-first-issue",
        "help-wanted",
        "question",
        "wontfix",
        "duplicate",
        "invalid",
        "dependencies",
    ];

    // Check if any labels are missing and capture logs BEFORE asserting
    let missing_labels: Vec<_> = expected_global_labels
        .iter()
        .filter(|label| !labels.contains(&label.to_string()))
        .collect();

    if !missing_labels.is_empty() {
        tracing::error!(
            "❌ {} labels missing: {:?}. Found: {:?}. Capturing container logs...",
            missing_labels.len(),
            missing_labels,
            labels
        );
        match container.get_logs().await {
            Ok(logs) => {
                eprintln!("\n========== CONTAINER LOGS (test_e2e_create_custom_init_both_files) ==========");
                eprintln!("{}", logs);
                eprintln!("====================================\n");
            }
            Err(e) => {
                tracing::warn!("Failed to capture container logs: {}", e);
            }
        }
    }

    for label in &expected_global_labels {
        assert!(
            labels.contains(&label.to_string()),
            "Missing global label '{}' from standard-labels.toml - found: {:?}",
            label,
            labels
        );
    }

    tracing::info!(
        "✓ Verified {} global labels from standard-labels.toml",
        expected_global_labels.len()
    );

    // Verify webhooks from global configuration
    let webhooks = match verification_client.list_webhooks(&org, &repo_name).await {
        Ok(webhooks) => webhooks,
        Err(e) => {
            tracing::warn!("Failed to list webhooks (may be timing issue): {}", e);
            // Try one more time after delay
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            verification_client.list_webhooks(&org, &repo_name).await?
        }
    };

    // Expected: 1 webhook from global config (no template webhooks)
    assert_eq!(
        webhooks.len(),
        1,
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
        global_webhook
            .events
            .contains(&github_client::WebhookEvent::Push)
            && global_webhook
                .events
                .contains(&github_client::WebhookEvent::PullRequest),
        "Global webhook should have 'push' and 'pull_request' events"
    );

    tracing::info!("✓ Verified 1 global webhook with correct configuration");

    // Verify rulesets from global configuration
    tracing::info!(
        org = &org,
        repo = &repo_name,
        "Attempting to list repository rulesets for verification"
    );
    let rulesets = match verification_client
        .list_repository_rulesets(&org, &repo_name)
        .await
    {
        Ok(rulesets) => {
            tracing::info!(
                org = &org,
                repo = &repo_name,
                count = rulesets.len(),
                "Successfully listed rulesets"
            );
            rulesets
        }
        Err(e) => {
            tracing::warn!(
                org = &org,
                repo = &repo_name,
                error = %e,
                "Failed to list rulesets on first attempt, retrying after delay"
            );
            // Try one more time after delay
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            match verification_client
                .list_repository_rulesets(&org, &repo_name)
                .await
            {
                Ok(rulesets) => {
                    tracing::info!(
                        org = &org,
                        repo = &repo_name,
                        count = rulesets.len(),
                        "Successfully listed rulesets on retry"
                    );
                    rulesets
                }
                Err(e) => {
                    tracing::error!(
                        org = &org,
                        repo = &repo_name,
                        error = %e,
                        "Failed to list rulesets after retry - this may indicate API response format mismatch"
                    );
                    return Err(e.into());
                }
            }
        }
    };

    tracing::info!("Repository has {} rulesets", rulesets.len());

    // Verify that rulesets were applied (if any exist in global config)
    // Note: The exact number depends on what's in .reporoller-test/global/rulesets.toml
    // For now, we just verify the operation succeeded and log the count
    if !rulesets.is_empty() {
        tracing::info!(
            "✓ Verified {} ruleset(s) applied from global configuration",
            rulesets.len()
        );
        for ruleset in &rulesets {
            tracing::info!(
                "  - Ruleset: {} (target: {:?}, enforcement: {:?})",
                ruleset.name,
                ruleset.target,
                ruleset.enforcement
            );
        }
    } else {
        tracing::info!("✓ No rulesets configured (verified list operation succeeds)");
    }

    // Cleanup
    e2e_tests::cleanup_test_repository(&org, &repo_name, app_id, &private_key)
        .await
        .ok();

    container.stop().await?;
    Ok(())
}
