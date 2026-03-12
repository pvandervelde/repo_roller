//! Tests for the permission_manager module.
//!
//! Tests are organised into three groups:
//!
//! 1. **Pure result-type tests** — unit tests for `ApplyPermissionsResult`
//!    helpers that require no async runtime.
//! 2. **Policy rejection tests** — async tests that verify the `PermissionManager`
//!    propagates `PolicyEngine` errors without making any GitHub API calls.
//! 3. **GitHub interaction tests** — async tests that use wiremock to simulate
//!    the GitHub REST API and verify the correct endpoints are called.

use std::collections::HashMap;

use github_client::GitHubClient;
use octocrab::Octocrab;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::permissions::{
    AccessLevel, OrganizationPermissionPolicies, PermissionGrant, PermissionHierarchy,
    PermissionRequest, PermissionScope, PermissionType, RepositoryContext, UserPermissionRequests,
};
use crate::policy_engine::PolicyEngine;
use crate::{OrganizationName, RepositoryName};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Creates a [`GitHubClient`] pointed at a wiremock server for testing.
fn create_test_github_client(server_uri: &str) -> GitHubClient {
    let octocrab = Octocrab::builder()
        .base_uri(server_uri)
        .expect("valid URI from wiremock")
        .personal_token("test-token".to_string())
        .build()
        .expect("octocrab builder succeeds with valid base_uri");
    GitHubClient::new(octocrab)
}

/// Builds a minimal valid [`PermissionRequest`] for the given org/repo.
fn make_request(org: &str, repo: &str, emergency_access: bool) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access,
        justification: "test".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new(org).expect("valid org name"),
            repository: RepositoryName::new(repo).expect("valid repo name"),
        },
        requested_permissions: vec![],
        requestor: "test-user".to_string(),
    }
}

/// Builds an empty, permissive [`PermissionHierarchy`] that passes all validation.
fn make_empty_hierarchy() -> PermissionHierarchy {
    PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies::default(),
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    }
}

/// Builds a hierarchy whose restrictions forbid the given permission
/// (ceiling set to `Read`, request would be `Admin`) to trigger
/// `PermissionError::ExceedsOrganizationLimits`.
fn make_exceeded_hierarchy(
    permission_type: PermissionType,
    scope: PermissionScope,
) -> PermissionHierarchy {
    PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![PermissionGrant {
                conditions: vec![],
                expiration: None,
                level: AccessLevel::Read, // ceiling = Read
                permission_type,
                scope,
            }],
        },
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    }
}

/// Returns a request that exceeds (`Admin`) the `Read` ceiling set in
/// [`make_exceeded_hierarchy`].
fn make_exceeding_request(
    org: &str,
    repo: &str,
    permission_type: PermissionType,
    scope: PermissionScope,
) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "test".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new(org).expect("valid org name"),
            repository: RepositoryName::new(repo).expect("valid repo name"),
        },
        requested_permissions: vec![PermissionGrant {
            conditions: vec![],
            expiration: None,
            level: AccessLevel::Admin, // exceeds Read ceiling
            permission_type,
            scope,
        }],
        requestor: "test-user".to_string(),
    }
}

// ─── Part 1: result-type unit tests ─────────────────────────────────────────

#[test]
fn test_apply_permissions_result_new_has_zero_counters() {
    let r = ApplyPermissionsResult::new();
    assert_eq!(r.teams_applied, 0);
    assert_eq!(r.teams_skipped, 0);
    assert_eq!(r.collaborators_applied, 0);
    assert_eq!(r.collaborators_removed, 0);
    assert_eq!(r.collaborators_skipped, 0);
    assert!(r.failed_teams.is_empty());
    assert!(r.failed_collaborators.is_empty());
}

#[test]
fn test_apply_permissions_result_default_equals_new() {
    let new = ApplyPermissionsResult::new();
    let default = ApplyPermissionsResult::default();
    assert_eq!(new, default);
}

#[test]
fn test_apply_permissions_result_is_success_when_no_failures() {
    let mut r = ApplyPermissionsResult::new();
    assert!(r.is_success(), "Empty result must be success");

    r.teams_applied = 2;
    r.collaborators_applied = 3;
    r.collaborators_skipped = 1;
    assert!(r.is_success(), "No failures → success even with counts");
}

#[test]
fn test_apply_permissions_result_is_not_success_when_team_fails() {
    let mut r = ApplyPermissionsResult::new();
    r.failed_teams.push("the-team".to_string());
    assert!(!r.is_success());
}

#[test]
fn test_apply_permissions_result_is_not_success_when_collaborator_fails() {
    let mut r = ApplyPermissionsResult::new();
    r.failed_collaborators.push("alice".to_string());
    assert!(!r.is_success());
}

#[test]
fn test_apply_permissions_result_has_changes_when_teams_applied() {
    let mut r = ApplyPermissionsResult::new();
    r.teams_applied = 1;
    assert!(r.has_changes());
}

#[test]
fn test_apply_permissions_result_has_changes_when_collaborators_applied() {
    let mut r = ApplyPermissionsResult::new();
    r.collaborators_applied = 1;
    assert!(r.has_changes());
}

#[test]
fn test_apply_permissions_result_has_changes_when_collaborators_removed() {
    let mut r = ApplyPermissionsResult::new();
    r.collaborators_removed = 1;
    assert!(r.has_changes());
}

#[test]
fn test_apply_permissions_result_has_no_changes_when_only_skipped() {
    let mut r = ApplyPermissionsResult::new();
    r.teams_skipped = 5;
    r.collaborators_skipped = 3;
    assert!(!r.has_changes());
}

#[test]
fn test_apply_permissions_result_has_no_changes_when_empty() {
    assert!(!ApplyPermissionsResult::new().has_changes());
}

// ─── Part 2: policy rejection tests ──────────────────────────────────────────

#[tokio::test]
async fn test_policy_rejection_returns_policy_denied_error() {
    // Build a hierarchy with a ceiling of Read for Push/Repository scope.
    // The request asks for Admin — PolicyEngine will return ExceedsOrganizationLimits.
    // The PermissionManager should propagate this as PolicyDenied.
    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let hierarchy = make_exceeded_hierarchy(PermissionType::Push, PermissionScope::Repository);
    let request = make_exceeding_request(
        "my-org",
        "my-repo",
        PermissionType::Push,
        PermissionScope::Repository,
    );

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, PermissionManagerError::PolicyDenied(_)),
        "Expected PolicyDenied, got: {err}"
    );
    // Verify no GitHub API calls were made (server received nothing).
    assert!(server
        .received_requests()
        .await
        .is_none_or(|r| r.is_empty()));
}

#[tokio::test]
async fn test_emergency_access_returns_requires_approval_error() {
    // A request with emergency_access = true triggers RequiresApproval from
    // the PolicyEngine. The PermissionManager must surface this as
    // PermissionManagerError::RequiresApproval without making any GitHub API calls.
    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let request = make_request("my-org", "my-repo", true /* emergency_access */);
    let hierarchy = make_empty_hierarchy();

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;

    assert!(result.is_err());
    assert!(
        matches!(
            result.unwrap_err(),
            PermissionManagerError::RequiresApproval { .. }
        ),
        "Emergency access must yield RequiresApproval"
    );
    assert!(server
        .received_requests()
        .await
        .is_none_or(|r| r.is_empty()));
}

// ─── Part 3: GitHub interaction tests ────────────────────────────────────────

#[tokio::test]
async fn test_team_permission_applied_successfully() {
    let server = MockServer::start().await;

    // GitHub PUT /orgs/{org}/teams/{slug}/repos/{org}/{repo} → 200 {}
    Mock::given(method("PUT"))
        .and(path("/orgs/my-org/teams/the-team/repos/my-org/my-repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut teams = HashMap::new();
    teams.insert("the-team".to_string(), AccessLevel::Write);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &teams,
            &HashMap::new(),
        )
        .await
        .expect("apply_repository_permissions should succeed");

    assert_eq!(result.teams_applied, 1);
    assert_eq!(result.teams_skipped, 0);
    assert!(result.failed_teams.is_empty());
    assert!(result.is_success());
    assert!(result.has_changes());
}

#[tokio::test]
async fn test_team_none_access_level_is_skipped() {
    // AccessLevel::None on a team should be skipped with a warning — no API call.
    let server = MockServer::start().await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut teams = HashMap::new();
    teams.insert("the-team".to_string(), AccessLevel::None);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &teams,
            &HashMap::new(),
        )
        .await
        .expect("should succeed even when skipping");

    assert_eq!(result.teams_applied, 0);
    assert_eq!(result.teams_skipped, 1);
    assert!(result.failed_teams.is_empty());
    assert!(result.is_success());
    assert!(!result.has_changes());
    assert!(server
        .received_requests()
        .await
        .is_none_or(|r| r.is_empty()));
}

#[tokio::test]
async fn test_team_api_error_is_recorded_as_failure() {
    let server = MockServer::start().await;

    // GitHub returns 404 (NotFound) → GitHubClient maps to Error::NotFound
    Mock::given(method("PUT"))
        .and(path("/orgs/my-org/teams/missing-team/repos/my-org/my-repo"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com"
        })))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut teams = HashMap::new();
    teams.insert("missing-team".to_string(), AccessLevel::Write);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &teams,
            &HashMap::new(),
        )
        .await
        .expect("method returns Ok even when individual operations fail");

    assert_eq!(result.teams_applied, 0);
    assert_eq!(result.failed_teams, vec!["missing-team"]);
    assert!(!result.is_success());
}

#[tokio::test]
async fn test_new_collaborator_is_added() {
    // The collaborator list (GET .../collaborators) is NOT fetched when no
    // AccessLevel::None entry is present — PUT is idempotent and handles both
    // new and existing collaborators without a membership pre-check.
    let server = MockServer::start().await;

    // Add collaborator endpoint.
    Mock::given(method("PUT"))
        .and(path("/repos/my-org/my-repo/collaborators/alice"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("alice".to_string(), AccessLevel::Write);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await
        .expect("should succeed");

    assert_eq!(result.collaborators_applied, 1);
    assert_eq!(result.collaborators_skipped, 0);
    assert!(result.is_success());
    assert!(result.has_changes());
}

#[tokio::test]
async fn test_existing_collaborator_permission_is_updated() {
    // The collaborator list is NOT fetched for non-None levels — the idempotent
    // PUT endpoint handles both new and existing collaborators without a
    // membership pre-check. GitHub returns 200 (or 204) for updates.
    let server = MockServer::start().await;

    // GitHub's PUT endpoint returns 204 No Content when updating an existing
    // collaborator's permission. Use 200 with matching body to work around
    // octocrab's optional-deserialisation quirks in the test harness.
    Mock::given(method("PUT"))
        .and(path("/repos/my-org/my-repo/collaborators/alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("alice".to_string(), AccessLevel::Write);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await
        .expect("should succeed");

    assert_eq!(result.collaborators_applied, 1);
    assert_eq!(result.collaborators_skipped, 0);
    assert!(result.is_success());
    assert!(
        result.has_changes(),
        "Updating a collaborator permission is a change"
    );
}

#[tokio::test]
async fn test_collaborator_removed_when_access_level_none_and_existing() {
    let server = MockServer::start().await;

    // alice is a current collaborator.
    Mock::given(method("GET"))
        .and(path("/repos/my-org/my-repo/collaborators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": 1, "login": "alice" }
        ])))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Fallback empty page so pagination stops cleanly.
    Mock::given(method("GET"))
        .and(path("/repos/my-org/my-repo/collaborators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    // Remove collaborator endpoint → 204 No Content.
    Mock::given(method("DELETE"))
        .and(path("/repos/my-org/my-repo/collaborators/alice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("alice".to_string(), AccessLevel::None);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await
        .expect("should succeed");

    assert_eq!(result.collaborators_removed, 1);
    assert_eq!(result.collaborators_applied, 0);
    assert_eq!(result.collaborators_skipped, 0);
    assert!(result.is_success());
    assert!(result.has_changes());
}

#[tokio::test]
async fn test_collaborator_removal_skipped_when_not_existing() {
    // AccessLevel::None for a user who is NOT a collaborator → skip, no DELETE call.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/my-org/my-repo/collaborators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("bob".to_string(), AccessLevel::None);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await
        .expect("should succeed");

    assert_eq!(result.collaborators_removed, 0);
    assert_eq!(result.collaborators_skipped, 1);
    assert!(result.is_success());
    assert!(!result.has_changes());
}

#[tokio::test]
async fn test_collaborator_api_error_is_recorded_as_failure() {
    let server = MockServer::start().await;

    // Empty collaborators list → charlie is new.
    Mock::given(method("GET"))
        .and(path("/repos/my-org/my-repo/collaborators"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Add collaborator fails with 404 (repository not found).
    Mock::given(method("PUT"))
        .and(path("/repos/my-org/my-repo/collaborators/charlie"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com"
        })))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("charlie".to_string(), AccessLevel::Write);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await
        .expect("method returns Ok even when add fails");

    assert_eq!(result.collaborators_applied, 0);
    assert_eq!(result.failed_collaborators, vec!["charlie"]);
    assert!(!result.is_success());
}

#[tokio::test]
async fn test_list_collaborators_error_returns_github_error() {
    // When `list_repository_collaborators` fails, the whole call returns
    // PermissionManagerError::GitHubError.  The list is only fetched when at
    // least one collaborator carries AccessLevel::None (removal check), so use
    // None here to trigger the fetch path.
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/my-org/my-repo/collaborators"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let mut collaborators = HashMap::new();
    collaborators.insert("dave".to_string(), AccessLevel::None);

    let result = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &make_request("my-org", "my-repo", false),
            &make_empty_hierarchy(),
            &HashMap::new(),
            &collaborators,
        )
        .await;

    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), PermissionManagerError::GitHubError(_)),
        "List failure must yield GitHubError"
    );
}
