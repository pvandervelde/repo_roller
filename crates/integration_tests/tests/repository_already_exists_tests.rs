//! Integration-level contract tests for task #282 / GitHub issue #282:
//! "repo-roller must guarantee it never mutates an existing repository".
//!
//! # Scope and an important architectural limitation
//!
//! The acceptance criteria for #282 call for an end-to-end proof, through
//! the public `repo_roller_core::create_repository` entry point, that zero
//! mutation calls occur when the target repository already exists. That
//! exact proof is NOT achievable in this black-box integration-test crate
//! under the current architecture:
//!
//! - `repo_roller_core::create_repository` always authenticates via
//!   `auth_handler::GitHubAuthService`, which calls the REAL GitHub App JWT
//!   endpoints with no base-URL override.
//! - Even if authentication is mocked, `repo_roller_core::setup_github_clients`
//!   always builds its `installation_repo_client` via
//!   `github_client::create_token_client`, which has no `base_uri` override
//!   seam either (unlike `github_client::create_octocrab_client`, which
//!   does, but is not what `setup_github_clients` calls).
//!
//! As a result, `repo_roller_core::create_repository_inner`'s actual call
//! site for the new existence-check gate cannot be pointed at a
//! `wiremock::MockServer` from this crate without either live GitHub
//! credentials or a production-code dependency-injection change -- neither
//! of which is in scope for the Tester role. This is documented as a
//! spec/architecture gap in the Tester's report for task #282.
//!
//! What IS fully testable here, network-free, is the `github_client`-level
//! contract the gate and the race-condition corollary are built on:
//! `RepositoryClient::get_repository` and `create_org_repository`'s new
//! `Error::AlreadyExists` detection. These tests exercise exactly those
//! building blocks through the real `GitHubClient` against
//! `integration_tests::GitHubMockServer`, including a structural
//! "zero mutation calls" proof via wiremock's `.expect(0)` assertion on the
//! repository-creation endpoint.

use github_client::{GitHubClient, RepositoryClient, RepositoryCreatePayload};
use integration_tests::mock_github::GitHubMockServer;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

/// Builds a [`GitHubClient`] authenticated with a personal-access-token
/// style client pointed at `server`'s URI. wiremock does not validate
/// credentials, so any non-empty token works.
fn client_for(server: &GitHubMockServer) -> GitHubClient {
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(server.uri())
        .expect("valid URI from GitHubMockServer")
        .personal_token("integration-test-token".to_string())
        .build()
        .expect("octocrab builder succeeds with a valid base_uri");
    GitHubClient::new(octocrab)
}

/// Contract test: `RepositoryClient::get_repository` (the trait method the
/// new existence-check gate depends on) must report `Ok(_)` when the mock
/// server has a 200 response registered for the target repository.
#[tokio::test]
async fn test_get_repository_contract_reports_existing_repository() {
    let server = GitHubMockServer::new().await.expect("mock server starts");
    server
        .mock_repository_exists("contract-org", "contract-repo")
        .await;

    let client = client_for(&server);
    let as_trait: &dyn RepositoryClient = &client;

    let result = as_trait
        .get_repository("contract-org", "contract-repo")
        .await;

    assert!(
        result.is_ok(),
        "expected Ok(repo) for an existing repository, got {result:?}"
    );
}

/// Contract test: `RepositoryClient::get_repository` must report
/// `Err(github_client::Error::NotFound)` when the mock server has a 404
/// response registered -- the other half of the two-outcome contract the
/// gate's "safe to proceed" branch depends on.
#[tokio::test]
async fn test_get_repository_contract_reports_not_found_for_missing_repository() {
    let server = GitHubMockServer::new().await.expect("mock server starts");
    server
        .mock_repository_not_found("contract-org", "missing-repo")
        .await;

    let client = client_for(&server);
    let as_trait: &dyn RepositoryClient = &client;

    let result = as_trait
        .get_repository("contract-org", "missing-repo")
        .await;

    assert!(
        matches!(result, Err(github_client::Error::NotFound)),
        "expected Error::NotFound for a missing repository, got {result:?}"
    );
}

/// Contract test for the race-condition corollary: when the GitHub create
/// call ITSELF reports "already exists" (422, via the pre-existing
/// `mock_repository_already_exists` fixture), `create_org_repository` must
/// map it to `Error::AlreadyExists` -- not a generic failure that would be
/// swallowed into `GitHubError::NetworkError` at the `repo_roller_core`
/// layer (task #282 race-condition corollary; Security Rules: the 409
/// mapping must apply to this trigger path too, not just the pre-check
/// path).
#[tokio::test]
async fn test_create_org_repository_contract_maps_race_condition_conflict_to_already_exists() {
    let server = GitHubMockServer::new().await.expect("mock server starts");
    server
        .mock_repository_already_exists("race-org", "race-repo")
        .await;

    let client = client_for(&server);
    let payload = RepositoryCreatePayload {
        name: "race-repo".to_string(),
        ..Default::default()
    };

    let result = client.create_org_repository("race-org", &payload).await;

    assert!(
        matches!(result, Err(github_client::Error::AlreadyExists)),
        "expected Error::AlreadyExists for the race-condition 422 response, got {:?}",
        result.as_ref().err()
    );
}

/// Structural "zero mutation calls" proof (task #282 acceptance criterion
/// 5), at the `github_client`/wiremock level: this test wires
/// `mock_repository_exists` for the existence check AND mounts the
/// repository-creation endpoint with `.expect(0)` -- wiremock will fail the
/// test on `.verify()` if that endpoint is hit even once. It then exercises
/// exactly the check-then-create control-flow contract task #282 requires
/// (`get_repository` first; only call `create_org_repository` if the check
/// says "does not exist"), proving that control flow makes zero repository-
/// creation calls when the repository already exists.
///
/// This proves the `github_client`-level building blocks behave correctly
/// under the required pattern; it does NOT prove `repo_roller_core::
/// create_repository_inner` actually wires the gate this way end-to-end --
/// see the module-level doc comment above for why that stronger claim is
/// architecturally out of reach for this crate today.
#[tokio::test]
async fn test_check_then_create_pattern_makes_zero_repository_creation_calls_when_repository_exists(
) {
    let server = GitHubMockServer::new().await.expect("mock server starts");
    server
        .mock_repository_exists("zero-mutation-org", "zero-mutation-repo")
        .await;

    // Explicitly assert the creation endpoint is never hit: wiremock fails
    // `.verify()` below if `.expect(0)` is violated.
    Mock::given(method("POST"))
        .and(path("/orgs/zero-mutation-org/repos"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(server.mock_server())
        .await;

    let client = client_for(&server);
    let as_trait: &dyn RepositoryClient = &client;

    let existence_check = as_trait
        .get_repository("zero-mutation-org", "zero-mutation-repo")
        .await;
    if existence_check.is_err() {
        // Only reached when the repository does NOT exist -- deliberately
        // unreachable in this test.
        let payload = RepositoryCreatePayload {
            name: "zero-mutation-repo".to_string(),
            ..Default::default()
        };
        let _ = client
            .create_org_repository("zero-mutation-org", &payload)
            .await;
    }

    assert!(
        existence_check.is_ok(),
        "the mocked repository must be reported as existing"
    );
    server.mock_server().verify().await;
}

/// Regression guard, paired with the zero-mutation-calls proof above: when
/// the repository does NOT exist, the same check-then-create pattern must
/// still reach the creation endpoint exactly once.
#[tokio::test]
async fn test_check_then_create_pattern_proceeds_to_create_when_repository_does_not_exist() {
    let server = GitHubMockServer::new().await.expect("mock server starts");
    server
        .mock_repository_not_found("proceed-org", "proceed-repo")
        .await;

    Mock::given(method("POST"))
        .and(path("/orgs/proceed-org/repos"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 1,
            "name": "proceed-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "full_name": "proceed-org/proceed-repo",
            "private": false,
            "url": "https://api.github.com/repos/proceed-org/proceed-repo"
        })))
        .expect(1)
        .mount(server.mock_server())
        .await;

    let client = client_for(&server);
    let as_trait: &dyn RepositoryClient = &client;

    let existence_check = as_trait.get_repository("proceed-org", "proceed-repo").await;
    if existence_check.is_err() {
        let payload = RepositoryCreatePayload {
            name: "proceed-repo".to_string(),
            ..Default::default()
        };
        let create_result = client.create_org_repository("proceed-org", &payload).await;
        assert!(
            create_result.is_ok(),
            "got {:?}",
            create_result.as_ref().err()
        );
    } else {
        panic!("expected the existence check to report NotFound");
    }

    server.mock_server().verify().await;
}
