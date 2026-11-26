//! Property-based (Tier 3) tests for `create_org_repository`'s new 422
//! "already exists" detection heuristic (task #282 / GitHub issue #282
//! race-condition corollary).
//!
//! The fixed-example tests in `lib_tests.rs` prove the heuristic correctly
//! classifies a handful of hand-picked message strings. These property
//! tests generalise that proof across a wide range of generated message
//! shapes to rule out both false negatives (missing a real "already
//! exists" phrasing) and false positives (misclassifying an unrelated 422
//! as a conflict).
//!
//! `create_org_repository` is async and this crate has no existing
//! async-proptest precedent, so each case drives a dedicated
//! `tokio::runtime::Runtime` synchronously from the `proptest!` body. A
//! fresh `wiremock::MockServer` is started per case (each case needs its
//! own mocked response body), so the case count is deliberately kept low
//! to bound test wall-clock time.

use super::*;
use proptest::prelude::*;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a [`GitHubClient`] authenticated with a personal-access-token
/// style client pointed at `mock_server`, mirroring
/// `repo_roller_core::lib_tests::wiremock_github_client`'s simpler
/// (non-GitHub-App) auth style -- sufficient here since wiremock does not
/// validate credentials.
fn proptest_client(mock_server: &MockServer) -> GitHubClient {
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(mock_server.uri())
        .expect("valid URI from wiremock")
        .personal_token("proptest-token".to_string())
        .build()
        .expect("octocrab builder succeeds with valid base_uri");
    GitHubClient { client: octocrab }
}

/// Runs `create_org_repository` against a fresh mock server that returns a
/// 422 with the given `message`, returning the resulting `Error`.
fn create_org_repository_with_422_message(message: &str) -> Error {
    let message = message.to_string();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
        let mock_server = MockServer::start().await;
        let org_name = "prop-org";
        let payload = RepositoryCreatePayload {
            name: "prop-repo".to_string(),
            ..Default::default()
        };

        Mock::given(method("POST"))
            .and(path(format!("/orgs/{org_name}/repos")))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({ "message": message })))
            .mount(&mock_server)
            .await;

        let client = proptest_client(&mock_server);
        match client.create_org_repository(org_name, &payload).await {
            Err(e) => e,
            Ok(_) => panic!("a 422 response must always surface as an Err"),
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 20, .. ProptestConfig::default() })]

    /// Invariant: ANY 422 message containing the literal phrase
    /// "already exists" -- regardless of surrounding text -- must map to
    /// `Error::AlreadyExists`.
    #[test]
    fn prop_422_message_containing_already_exists_phrase_always_maps_to_already_exists(
        prefix in "[a-zA-Z0-9 .:_-]{0,24}",
        suffix in "[a-zA-Z0-9 .:_-]{0,24}",
    ) {
        let message = format!("{prefix} already exists {suffix}");

        let error = create_org_repository_with_422_message(&message);

        prop_assert!(
            matches!(error, Error::AlreadyExists),
            "message {message:?} must map to Error::AlreadyExists, got {error:?}"
        );
    }

    /// Invariant: ANY 422 message containing the underscored phrase
    /// "already_exists" must also map to `Error::AlreadyExists` (GitHub
    /// uses both spaced and underscored wording across different
    /// endpoints/error codes).
    #[test]
    fn prop_422_message_containing_underscored_already_exists_always_maps_to_already_exists(
        prefix in "[a-zA-Z0-9 .:_-]{0,24}",
        suffix in "[a-zA-Z0-9 .:_-]{0,24}",
    ) {
        let message = format!("{prefix} already_exists {suffix}");

        let error = create_org_repository_with_422_message(&message);

        prop_assert!(
            matches!(error, Error::AlreadyExists),
            "message {message:?} must map to Error::AlreadyExists, got {error:?}"
        );
    }

    /// Invariant: 422 messages drawn from a fixed pool of unrelated
    /// validation failures (none containing "already exists" / "already_exists")
    /// must NEVER be misclassified as `Error::AlreadyExists`. Generated from a
    /// pool, rather than fully arbitrary strings, so the property is
    /// guaranteed not to accidentally contain the target phrase.
    #[test]
    fn prop_422_message_without_already_exists_phrase_never_maps_to_already_exists(
        message_index in 0usize..5,
    ) {
        let pool = [
            "name can only contain alphanumeric characters, hyphens, and underscores",
            "Validation Failed",
            "owner does not have permission to create repositories in this organization",
            "repository limit exceeded for this plan",
            "topic is not a valid identifier",
        ];
        let message = pool[message_index];

        let error = create_org_repository_with_422_message(message);

        prop_assert!(
            !matches!(error, Error::AlreadyExists),
            "unrelated message {message:?} must NOT map to Error::AlreadyExists, got {error:?}"
        );
    }
}
