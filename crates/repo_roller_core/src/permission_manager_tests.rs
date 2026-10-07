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

// ─────────────────────────────────────────────────────────────────────────
// RED tests: `apply_repository_permissions`'s `owner`/`repo` span
// attributes are NOT currently sanitized (Security Reviewer follow-up
// pass on remaining unsanitized `#[instrument(fields(...))]` sites).
//
// Elsewhere the Security Reviewer characterised `owner`/`repo`-style
// fields as lower-risk GitHub-validated identifiers. THIS call site is a
// partial exception: `apply_repository_permissions` takes raw `&str`
// parameters directly (unlike `request.repository_context`, which uses the
// validated `OrganizationName`/`RepositoryName` newtypes) -- nothing in
// this function's signature guarantees a caller has passed an
// already-GitHub-validated value, so a caller bug or a future call site
// could pass attacker-influenced content straight through. These tests use
// `make_exceeding_request`/`make_exceeded_hierarchy` to hit the
// `PolicyDenied` fast-fail path (see `test_policy_rejection_returns_policy_denied_error`
// above) so no GitHub API call is made -- the `#[instrument]` span and its
// fields are captured at call time, before the policy check runs.
// ─────────────────────────────────────────────────────────────────────────

struct FieldValueCollector(std::collections::HashMap<String, String>);

impl tracing::field::Visit for FieldValueCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}

#[derive(Debug, Clone)]
struct RecordedSpanWithValues {
    name: &'static str,
    fields: std::collections::HashMap<String, String>,
}

#[derive(Clone, Default)]
struct SpanValueRecorder {
    spans: std::sync::Arc<std::sync::Mutex<Vec<RecordedSpanWithValues>>>,
}

impl SpanValueRecorder {
    fn recorded(&self) -> Vec<RecordedSpanWithValues> {
        self.spans
            .lock()
            .expect("span value recorder mutex poisoned")
            .clone()
    }
}

impl<S> tracing_subscriber::Layer<S> for SpanValueRecorder
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span_ref = ctx
            .span(id)
            .expect("span must exist in registry for on_new_span");
        let mut collector = FieldValueCollector(std::collections::HashMap::new());
        attrs.record(&mut collector);
        self.spans
            .lock()
            .expect("span value recorder mutex poisoned")
            .push(RecordedSpanWithValues {
                name: span_ref.name(),
                fields: collector.0,
            });
    }
}

#[tokio::test]
async fn test_apply_repository_permissions_span_owner_field_is_redacted_when_secret_shaped() {
    use tracing_subscriber::layer::SubscriberExt;

    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let hierarchy = make_exceeded_hierarchy(PermissionType::Push, PermissionScope::Repository);
    let request = make_exceeding_request(
        "my-org",
        "my-repo",
        PermissionType::Push,
        PermissionScope::Repository,
    );
    let secret_shaped_owner = format!("ghp_{}", "e".repeat(24));

    let result = manager
        .apply_repository_permissions(
            &secret_shaped_owner,
            "my-repo",
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;
    assert!(
        result.is_err(),
        "expected PolicyDenied fast-fail with no GitHub API call"
    );
    assert!(server
        .received_requests()
        .await
        .is_none_or(|r| r.is_empty()));

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "apply_repository_permissions")
        .expect("apply_repository_permissions must create a span");
    let recorded_owner = span
        .fields
        .get("owner")
        .expect("apply_repository_permissions span must record an `owner` field");
    assert_eq!(
        recorded_owner,
        github_client::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped owner must be fully redacted before being attached as a span \
         attribute, recorded value: {recorded_owner:?}"
    );
}

#[tokio::test]
async fn test_apply_repository_permissions_span_owner_field_short_benign_value_passes_through_unchanged(
) {
    // Stub-killing: guards against a lazy "always redact" fix.
    use tracing_subscriber::layer::SubscriberExt;

    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let hierarchy = make_exceeded_hierarchy(PermissionType::Push, PermissionScope::Repository);
    let request = make_exceeding_request(
        "my-org",
        "my-repo",
        PermissionType::Push,
        PermissionScope::Repository,
    );

    let _ = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "apply_repository_permissions")
        .expect("apply_repository_permissions must create a span");
    let recorded_owner = span
        .fields
        .get("owner")
        .expect("apply_repository_permissions span must record an `owner` field");
    assert_eq!(
        recorded_owner, "my-org",
        "an ordinary, short, non-secret-shaped owner must pass through sanitization \
         unchanged, recorded value: {recorded_owner:?}"
    );
}

#[tokio::test]
async fn test_apply_repository_permissions_span_repo_field_is_redacted_when_secret_shaped() {
    use tracing_subscriber::layer::SubscriberExt;

    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let hierarchy = make_exceeded_hierarchy(PermissionType::Push, PermissionScope::Repository);
    let request = make_exceeding_request(
        "my-org",
        "my-repo",
        PermissionType::Push,
        PermissionScope::Repository,
    );
    let secret_shaped_repo = format!("ghp_{}", "f".repeat(24));

    let _ = manager
        .apply_repository_permissions(
            "my-org",
            &secret_shaped_repo,
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "apply_repository_permissions")
        .expect("apply_repository_permissions must create a span");
    let recorded_repo = span
        .fields
        .get("repo")
        .expect("apply_repository_permissions span must record a `repo` field");
    assert_eq!(
        recorded_repo,
        github_client::span_attributes::SPAN_ATTRIBUTE_REDACTED_MARKER,
        "a GitHub-token-shaped repo must be fully redacted before being attached as a span \
         attribute, recorded value: {recorded_repo:?}"
    );
}

#[tokio::test]
async fn test_apply_repository_permissions_span_repo_field_short_benign_value_passes_through_unchanged(
) {
    use tracing_subscriber::layer::SubscriberExt;

    let server = MockServer::start().await;
    let client = create_test_github_client(&server.uri());
    let manager = PermissionManager::new(client, PolicyEngine::new());

    let recorder = SpanValueRecorder::default();
    let subscriber = tracing_subscriber::registry().with(recorder.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let hierarchy = make_exceeded_hierarchy(PermissionType::Push, PermissionScope::Repository);
    let request = make_exceeding_request(
        "my-org",
        "my-repo",
        PermissionType::Push,
        PermissionScope::Repository,
    );

    let _ = manager
        .apply_repository_permissions(
            "my-org",
            "my-repo",
            &request,
            &hierarchy,
            &HashMap::new(),
            &HashMap::new(),
        )
        .await;

    let spans = recorder.recorded();
    let span = spans
        .iter()
        .find(|s| s.name == "apply_repository_permissions")
        .expect("apply_repository_permissions must create a span");
    let recorded_repo = span
        .fields
        .get("repo")
        .expect("apply_repository_permissions span must record a `repo` field");
    assert_eq!(
        recorded_repo, "my-repo",
        "an ordinary, short, non-secret-shaped repo must pass through sanitization \
         unchanged, recorded value: {recorded_repo:?}"
    );
}
