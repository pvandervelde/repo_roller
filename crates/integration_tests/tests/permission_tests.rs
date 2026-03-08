//! Permission system integration tests.
//!
//! These tests verify that team and collaborator permissions are correctly
//! applied to repositories using the [`PermissionManager`] against real GitHub
//! infrastructure (glitchgrove organization).
//!
//! ## What these tests cover
//!
//! - Team permission application via `PermissionManager`
//! - Collaborator permission application via `PermissionManager`
//! - Idempotency: re-applying the same permissions skips already-set entries
//! - Collaborator removal: `AccessLevel::None` removes an existing collaborator
//! - Permission hierarchy enforcement: `PolicyEngine` rejection propagates correctly
//!
//! ## Required environment variables
//!
//! | Variable                     | Description                                              |
//! |------------------------------|----------------------------------------------------------|
//! | `GITHUB_APP_ID`              | GitHub App numeric ID                                    |
//! | `GITHUB_APP_PRIVATE_KEY`     | GitHub App private key (PEM)                             |
//! | `TEST_ORG`                   | GitHub organization used for test repositories           |
//! | `TEST_TEAM_SLUG`             | Slug of an existing team in `TEST_ORG` (e.g. `developers`) |
//! | `TEST_COLLABORATOR_USERNAME` | GitHub username of a user to add/remove as collaborator  |
//!
//! ## Test isolation
//!
//! Every test creates its own uniquely-named repository and deletes it in a
//! `cleanup` step that runs even on failure (`.await.ok()` pattern).

use std::collections::HashMap;

use anyhow::Result;
use auth_handler::{GitHubAuthService, UserAuthenticationService};
use github_client::{
    create_app_client, create_token_client, GitHubClient, RepositoryClient, RepositoryCreatePayload,
};
use integration_tests::{RepositoryCleanup, TestConfig};
use repo_roller_core::{
    permission_manager::PermissionManager,
    permissions::{
        AccessLevel, OrganizationPermissionPolicies, PermissionGrant, PermissionHierarchy,
        PermissionRequest, PermissionScope, PermissionType, RepositoryContext,
        UserPermissionRequests,
    },
    policy_engine::PolicyEngine,
    OrganizationName, RepositoryName,
};
use tracing::info;
use uuid::Uuid;

// ── Logging ───────────────────────────────────────────────────────────────────

fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_test_writer()
        .try_init();
}

// ── Environment helpers ───────────────────────────────────────────────────────

/// Configuration extended with permission-specific environment variables.
struct PermissionTestConfig {
    base: TestConfig,
    /// Slug of an existing team in the test org (e.g. `developers`).
    team_slug: String,
    /// GitHub username of a user to add/remove as a direct collaborator.
    ///
    /// `None` when `TEST_COLLABORATOR_USERNAME` is not set; collaborator tests
    /// skip gracefully rather than failing.
    collaborator_username: Option<String>,
}

impl PermissionTestConfig {
    fn from_env() -> Result<Self> {
        let base = TestConfig::from_env()?;

        let team_slug = std::env::var("TEST_TEAM_SLUG").map_err(|_| {
            anyhow::anyhow!(
                "TEST_TEAM_SLUG environment variable not set. \
                 Provide the slug of an existing team in the test org."
            )
        })?;

        // Optional — collaborator tests skip when this is absent.
        let collaborator_username = std::env::var("TEST_COLLABORATOR_USERNAME").ok();
        if collaborator_username.is_none() {
            tracing::warn!(
                "TEST_COLLABORATOR_USERNAME not set — collaborator integration tests will be skipped"
            );
        }

        Ok(Self {
            base,
            team_slug,
            collaborator_username,
        })
    }
}

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Creates dependencies needed across multiple permission tests.
async fn create_test_dependencies(
    config: &PermissionTestConfig,
) -> Result<(GitHubClient, PermissionManager)> {
    let auth_service = GitHubAuthService::new(
        config.base.github_app_id,
        config.base.github_app_private_key.clone(),
    );
    let installation_token = auth_service
        .get_installation_token_for_org(&config.base.test_org)
        .await?;

    let github_client = GitHubClient::new(create_token_client(&installation_token)?);
    let permission_manager = PermissionManager::new(github_client.clone(), PolicyEngine::new());

    Ok((github_client, permission_manager))
}

/// Creates a private test repository and returns its [`RepositoryName`].
async fn create_test_repo(
    github_client: &GitHubClient,
    org: &str,
    test_label: &str,
) -> Result<RepositoryName> {
    let unique_suffix = Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("x")
        .to_string();
    let repo_name = RepositoryName::new(format!(
        "integration-test-perm-{test_label}-{unique_suffix}"
    ))?;

    let payload = RepositoryCreatePayload {
        name: repo_name.as_ref().to_string(),
        description: Some(format!(
            "Transient test repo for permission integration test: {test_label}"
        )),
        private: Some(true),
        ..Default::default()
    };
    github_client.create_org_repository(org, &payload).await?;

    // Brief stabilisation pause — GitHub sometimes needs a moment after creation.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("Created test repository: {}/{}", org, repo_name.as_ref());
    Ok(repo_name)
}

/// Deletes a test repository, swallowing errors so cleanup never panics.
async fn cleanup_test_repo(config: &PermissionTestConfig, repo_name: &RepositoryName) {
    match create_app_client(
        config.base.github_app_id,
        &config.base.github_app_private_key,
    )
    .await
    {
        Ok(cleanup_octocrab) => {
            let cleanup = RepositoryCleanup::new(
                GitHubClient::new(cleanup_octocrab),
                config.base.test_org.clone(),
            );
            cleanup.delete_repository(repo_name.as_ref()).await.ok();
            info!("Cleaned up test repository: {}", repo_name.as_ref());
        }
        Err(e) => {
            tracing::warn!(
                repo = repo_name.as_ref(),
                error = %e,
                "Failed to create cleanup client — repository may need manual deletion"
            );
        }
    }
}

/// Builds a minimal valid [`PermissionRequest`] for the given org and repo.
fn minimal_permission_request(org: &str, repo_name: &RepositoryName) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "integration-test".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new(org).expect("valid org name"),
            repository: repo_name.clone(),
        },
        requested_permissions: vec![],
        requestor: "integration-test-runner".to_string(),
    }
}

/// Builds an empty [`PermissionHierarchy`] (no org restrictions, no template).
fn empty_hierarchy() -> PermissionHierarchy {
    PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies::default(),
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    }
}

// ── Integration tests ─────────────────────────────────────────────────────────

/// Verify that a team can be granted repository access via `PermissionManager`.
///
/// Creates a private repository, applies `write` permission to the test team,
/// asserts `teams_applied == 1`, then deletes the repository.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and TEST_TEAM_SLUG env var"]
async fn test_team_permission_applied_to_repository() -> Result<()> {
    init_test_logging();
    info!("Running test_team_permission_applied_to_repository");

    let config = PermissionTestConfig::from_env()?;
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name = create_test_repo(&github_client, &config.base.test_org, "team-apply").await?;

    let request = minimal_permission_request(&config.base.test_org, &repo_name);
    let hierarchy = empty_hierarchy();
    let teams: HashMap<String, AccessLevel> =
        [(config.team_slug.clone(), AccessLevel::Write)].into();
    let collaborators: HashMap<String, AccessLevel> = HashMap::new();

    let result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    // Cleanup before asserting so the repository is always removed.
    cleanup_test_repo(&config, &repo_name).await;

    let result = result?;
    info!(
        teams_applied = result.teams_applied,
        teams_skipped = result.teams_skipped,
        "Team permission result"
    );

    assert_eq!(
        result.teams_applied, 1,
        "Expected exactly one team to be applied"
    );
    assert_eq!(result.teams_skipped, 0, "Expected no teams to be skipped");
    assert!(result.is_success(), "Expected overall success");
    assert!(
        result.has_changes(),
        "Expected has_changes for applied team"
    );

    info!("✓ test_team_permission_applied_to_repository passed");
    Ok(())
}

/// Verify that a direct collaborator can be added via `PermissionManager`.
///
/// Creates a private repository, adds the test collaborator with `read`
/// permission, verifies they appear in `list_repository_collaborators`, then
/// deletes the repository.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and TEST_COLLABORATOR_USERNAME env var"]
async fn test_collaborator_permission_applied_to_repository() -> Result<()> {
    init_test_logging();
    info!("Running test_collaborator_permission_applied_to_repository");

    let config = PermissionTestConfig::from_env()?;
    let Some(collaborator_username) = config.collaborator_username.clone() else {
        tracing::warn!("Skipping test_collaborator_permission_applied_to_repository: TEST_COLLABORATOR_USERNAME not set");
        return Ok(());
    };
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name = create_test_repo(&github_client, &config.base.test_org, "collab-apply").await?;

    let request = minimal_permission_request(&config.base.test_org, &repo_name);
    let hierarchy = empty_hierarchy();
    let teams: HashMap<String, AccessLevel> = HashMap::new();
    let collaborators: HashMap<String, AccessLevel> =
        [(collaborator_username.clone(), AccessLevel::Read)].into();

    let apply_result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    // List collaborators to verify state before cleanup.
    let list_result = github_client
        .list_repository_collaborators(&config.base.test_org, repo_name.as_ref())
        .await;

    cleanup_test_repo(&config, &repo_name).await;

    let apply = apply_result?;
    info!(
        collaborators_applied = apply.collaborators_applied,
        collaborators_skipped = apply.collaborators_skipped,
        "Collaborator permission result"
    );

    assert_eq!(
        apply.collaborators_applied, 1,
        "Expected exactly one collaborator to be applied"
    );
    assert_eq!(
        apply.collaborators_skipped, 0,
        "Expected no collaborators to be skipped"
    );
    assert!(apply.is_success(), "Expected overall success");

    let collaborators_list = list_result?;
    let found = collaborators_list
        .iter()
        .any(|c| c.login.to_lowercase() == collaborator_username.to_lowercase());
    assert!(
        found,
        "Collaborator '{}' not found in repository collaborators list: {:?}",
        collaborator_username,
        collaborators_list
            .iter()
            .map(|c| &c.login)
            .collect::<Vec<_>>()
    );

    info!("✓ test_collaborator_permission_applied_to_repository passed");
    Ok(())
}

/// Verify that applying the same permissions twice skips on the second call.
///
/// Creates a repository, applies collaborator access, then applies the same
/// permissions again. The second call should report `collaborators_skipped == 1`
/// not `collaborators_applied == 1`.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and TEST_COLLABORATOR_USERNAME env var"]
async fn test_permission_application_is_idempotent() -> Result<()> {
    init_test_logging();
    info!("Running test_permission_application_is_idempotent");

    let config = PermissionTestConfig::from_env()?;
    let Some(collaborator_username) = config.collaborator_username.clone() else {
        tracing::warn!("Skipping test_permission_application_is_idempotent: TEST_COLLABORATOR_USERNAME not set");
        return Ok(());
    };
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name = create_test_repo(&github_client, &config.base.test_org, "idempotent").await?;

    let request = minimal_permission_request(&config.base.test_org, &repo_name);
    let hierarchy = empty_hierarchy();
    let teams: HashMap<String, AccessLevel> = HashMap::new();
    let collaborators: HashMap<String, AccessLevel> =
        [(collaborator_username.clone(), AccessLevel::Write)].into();

    // First application — should add the collaborator.
    let first_result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    // Second application — same inputs, should skip the already-present collaborator.
    let second_result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    cleanup_test_repo(&config, &repo_name).await;

    let first = first_result?;
    let second = second_result?;

    info!(
        first_applied = first.collaborators_applied,
        first_skipped = first.collaborators_skipped,
        second_applied = second.collaborators_applied,
        second_skipped = second.collaborators_skipped,
        "Idempotency results"
    );

    assert_eq!(
        first.collaborators_applied, 1,
        "First call should apply collaborator"
    );
    assert_eq!(first.collaborators_skipped, 0, "First call should not skip");

    assert_eq!(
        second.collaborators_applied, 0,
        "Second call should not re-apply"
    );
    assert_eq!(
        second.collaborators_skipped, 1,
        "Second call should skip already-present collaborator"
    );

    info!("✓ test_permission_application_is_idempotent passed");
    Ok(())
}

/// Verify that setting `AccessLevel::None` removes an existing collaborator.
///
/// Adds a collaborator directly, then uses `PermissionManager` with
/// `AccessLevel::None` to remove them, and verifies they no longer appear in
/// the repository collaborator list.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure and TEST_COLLABORATOR_USERNAME env var"]
async fn test_collaborator_removed_when_access_level_none() -> Result<()> {
    init_test_logging();
    info!("Running test_collaborator_removed_when_access_level_none");

    let config = PermissionTestConfig::from_env()?;
    let Some(collaborator_username) = config.collaborator_username.clone() else {
        tracing::warn!("Skipping test_collaborator_removed_when_access_level_none: TEST_COLLABORATOR_USERNAME not set");
        return Ok(());
    };
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name =
        create_test_repo(&github_client, &config.base.test_org, "collab-remove").await?;

    // Add the collaborator directly so we know they exist.
    github_client
        .add_repository_collaborator(
            &config.base.test_org,
            repo_name.as_ref(),
            &collaborator_username,
            "pull",
        )
        .await?;
    info!("Added collaborator '{}' directly", collaborator_username);

    // Brief stabilisation.
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let request = minimal_permission_request(&config.base.test_org, &repo_name);
    let hierarchy = empty_hierarchy();
    let teams: HashMap<String, AccessLevel> = HashMap::new();
    // AccessLevel::None triggers removal for an existing collaborator.
    let collaborators: HashMap<String, AccessLevel> =
        [(collaborator_username.clone(), AccessLevel::None)].into();

    let remove_result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    let list_after = github_client
        .list_repository_collaborators(&config.base.test_org, repo_name.as_ref())
        .await;

    cleanup_test_repo(&config, &repo_name).await;

    let removed = remove_result?;
    info!(
        removed = removed.collaborators_removed,
        "Collaborator removal result"
    );

    assert_eq!(
        removed.collaborators_removed, 1,
        "Expected one collaborator to be removed"
    );
    assert!(
        removed.is_success(),
        "Expected overall success after removal"
    );

    let list = list_after?;
    let still_present = list
        .iter()
        .any(|c| c.login.to_lowercase() == collaborator_username.to_lowercase());
    assert!(
        !still_present,
        "Collaborator '{}' should have been removed from the repository",
        collaborator_username,
    );

    info!("✓ test_collaborator_removed_when_access_level_none passed");
    Ok(())
}

/// Verify that `PermissionManager` propagates a `PolicyDenied` error correctly.
///
/// Configures a hierarchy that restricts all `Admin` permission types to a
/// maximum of `Write`, then submits a request for `Admin`. Asserts that
/// `apply_repository_permissions` returns `PermissionManagerError::PolicyDenied`.
///
/// This test exercises the hierarchy-enforcement path without creating a
/// GitHub repository — the policy engine runs before any API calls.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure — tagged ignore for consistency with other integration tests"]
async fn test_permission_hierarchy_enforcement_blocks_restricted_permission() -> Result<()> {
    init_test_logging();
    info!("Running test_permission_hierarchy_enforcement_blocks_restricted_permission");

    let config = PermissionTestConfig::from_env()?;
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name = create_test_repo(&github_client, &config.base.test_org, "policy-block").await?;

    // Hierarchy: org restricts Admin Push to a maximum of Write.
    let restriction = PermissionGrant {
        conditions: vec![],
        expiration: None,
        level: AccessLevel::Write, // ceiling
        permission_type: PermissionType::Admin,
        scope: PermissionScope::Team,
    };
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![restriction],
        },
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    };

    // Request Admin — exceeds the Write ceiling.
    let request = PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "integration-test-policy-enforcement".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new(&config.base.test_org).expect("valid org name"),
            repository: repo_name.clone(),
        },
        requested_permissions: vec![PermissionGrant {
            conditions: vec![],
            expiration: None,
            level: AccessLevel::Admin,
            permission_type: PermissionType::Admin,
            scope: PermissionScope::Team,
        }],
        requestor: "integration-test-runner".to_string(),
    };

    let teams: HashMap<String, AccessLevel> =
        [(config.team_slug.clone(), AccessLevel::Admin)].into();
    let collaborators: HashMap<String, AccessLevel> = HashMap::new();

    let result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    cleanup_test_repo(&config, &repo_name).await;

    assert!(
        result.is_err(),
        "Expected PolicyDenied error but got: {:?}",
        result
    );

    use repo_roller_core::permission_manager::PermissionManagerError;
    match result.unwrap_err() {
        PermissionManagerError::PolicyDenied(_) => {
            info!("✓ PolicyDenied error returned as expected");
        }
        other => {
            panic!("Expected PolicyDenied but got: {other:?}");
        }
    }

    info!("✓ test_permission_hierarchy_enforcement_blocks_restricted_permission passed");
    Ok(())
}

/// Verify that applying both teams and collaborators together succeeds.
///
/// Creates a repository, applies one team and one collaborator simultaneously,
/// and asserts both counts in the result.
#[tokio::test]
#[ignore = "Requires real GitHub infrastructure, TEST_TEAM_SLUG and TEST_COLLABORATOR_USERNAME env vars"]
async fn test_team_and_collaborator_applied_together() -> Result<()> {
    init_test_logging();
    info!("Running test_team_and_collaborator_applied_together");

    let config = PermissionTestConfig::from_env()?;
    let Some(collaborator_username) = config.collaborator_username.clone() else {
        tracing::warn!("Skipping test_team_and_collaborator_applied_together: TEST_COLLABORATOR_USERNAME not set");
        return Ok(());
    };
    let (github_client, permission_manager) = create_test_dependencies(&config).await?;

    let repo_name = create_test_repo(&github_client, &config.base.test_org, "combined").await?;

    let request = minimal_permission_request(&config.base.test_org, &repo_name);
    let hierarchy = empty_hierarchy();
    let teams: HashMap<String, AccessLevel> =
        [(config.team_slug.clone(), AccessLevel::Triage)].into();
    let collaborators: HashMap<String, AccessLevel> =
        [(collaborator_username.clone(), AccessLevel::Read)].into();

    let result = permission_manager
        .apply_repository_permissions(
            &config.base.test_org,
            repo_name.as_ref(),
            &request,
            &hierarchy,
            &teams,
            &collaborators,
        )
        .await;

    cleanup_test_repo(&config, &repo_name).await;

    let result = result?;
    info!(
        teams_applied = result.teams_applied,
        collaborators_applied = result.collaborators_applied,
        "Combined permission result"
    );

    assert_eq!(result.teams_applied, 1, "Expected one team applied");
    assert_eq!(
        result.collaborators_applied, 1,
        "Expected one collaborator applied"
    );
    assert!(result.is_success(), "Expected overall success");
    assert!(result.has_changes(), "Expected has_changes");

    info!("✓ test_team_and_collaborator_applied_together passed");
    Ok(())
}

// ── Protection-policy tests (no GitHub required) ─────────────────────────────

/// Verify that a locked config entry is NOT altered by a request that tries to
/// change it.  The config-established level is preserved.
///
/// These tests exercise `merge_access_map_with_policy` directly without hitting
/// GitHub infrastructure — they do not need the `#[ignore]` guard.
#[test]
fn test_ppp_locked_entry_not_altered_by_request() {
    use repo_roller_core::{merge_access_map_with_policy, permissions::AccessLevel};
    use std::collections::{HashMap, HashSet};

    let config: HashMap<String, String> = [("security-ops".to_string(), "admin".to_string())]
        .into_iter()
        .collect();
    let locked: HashSet<String> = ["security-ops".to_string()].into_iter().collect();

    // Request tries to lower security-ops from admin to write (change on a locked entry).
    let request = [("security-ops", AccessLevel::Write)];
    let result = merge_access_map_with_policy(&config, &locked, None, &request, "team");

    assert_eq!(
        result.get("security-ops"),
        Some(&AccessLevel::Admin),
        "Locked entry must keep config level despite request trying to change it"
    );
}

/// Verify that a config-established entry CANNOT be demoted by a request.
/// The higher config level is preserved.
#[test]
fn test_ppp_config_entry_cannot_be_demoted_by_request() {
    use repo_roller_core::{merge_access_map_with_policy, permissions::AccessLevel};
    use std::collections::{HashMap, HashSet};

    let config: HashMap<String, String> = [("platform".to_string(), "maintain".to_string())]
        .into_iter()
        .collect();
    let locked: HashSet<String> = HashSet::new();

    // Request tries to demote platform from maintain to read.
    let request = [("platform", AccessLevel::Read)];
    let result = merge_access_map_with_policy(&config, &locked, None, &request, "team");

    assert_eq!(
        result.get("platform"),
        Some(&AccessLevel::Maintain),
        "Demotion attempt must be ignored; config level preserved"
    );
}

/// Verify that a request entry exceeding the org ceiling is capped, not rejected.
#[test]
fn test_ppp_request_team_capped_at_org_ceiling() {
    use repo_roller_core::{merge_access_map_with_policy, permissions::AccessLevel};
    use std::collections::{HashMap, HashSet};

    let config: HashMap<String, String> = HashMap::new();
    let locked: HashSet<String> = HashSet::new();

    // Ceiling is "maintain"; request asks for "admin".
    let request = [("new-team", AccessLevel::Admin)];
    let result =
        merge_access_map_with_policy(&config, &locked, Some("maintain"), &request, "team");

    assert_eq!(
        result.get("new-team"),
        Some(&AccessLevel::Maintain),
        "Request level exceeding ceiling must be capped at ceiling"
    );
}

/// Verify that a config entry above the ceiling is NOT capped — the ceiling
/// only applies to request entries.
#[test]
fn test_ppp_ceiling_does_not_affect_config_entries() {
    use repo_roller_core::{merge_access_map_with_policy, permissions::AccessLevel};
    use std::collections::{HashMap, HashSet};

    let config: HashMap<String, String> = [("security-ops".to_string(), "admin".to_string())]
        .into_iter()
        .collect();
    let locked: HashSet<String> = HashSet::new();

    // Ceiling is "write", but the config entry is "admin" (legitimately above ceiling).
    let result = merge_access_map_with_policy(&config, &locked, Some("write"), &[], "team");

    assert_eq!(
        result.get("security-ops"),
        Some(&AccessLevel::Admin),
        "Config-established entries must not be affected by the ceiling"
    );
}

