//! Tests for the `ActionsManager` module (#276/#277).
//!
//! `ActionsManager::apply_action_settings` is the orchestration layer
//! between the merged `config_manager::settings::ActionSettings` and the two
//! new GitHub REST calls on `github_client::RepositoryClient`
//! (`set_actions_workflow_permissions`, `set_actions_access_level`). These
//! tests exercise it against a local mock `RepositoryClient` that records
//! every call made to the two new methods, so we can assert both "the right
//! call happened" and -- just as important for a stub-killing suite -- "no
//! call happened when it shouldn't have".
//!
//! Per the interface contract, the 5 pre-existing `ActionSettings` fields
//! (`enabled`, `allowed_actions`, `github_owned_allowed`, `verified_allowed`,
//! `patterns_allowed`) are explicitly OUT OF SCOPE for this manager and must
//! never trigger an API call.

use super::*;
use crate::{GitHubError, RepoRollerError};
use async_trait::async_trait;
use config_manager::settings::ActionSettings;
use config_manager::OverridableValue;
use github_client::{
    ActionsAccessLevelUpdate, ActionsWorkflowPermissionsUpdate, BranchProtection,
    CreateWebhookParams, Error as GHError, PullRequest, PullRequestCreatePayload, Repository,
    RepositoryClient, RepositoryCreatePayload, RepositoryRuleset, RepositorySettingsUpdate,
    UpdateWebhookParams, Webhook,
};
use std::sync::{Arc, Mutex};

/// Configurable outcome for a tracked mock call: either success, or a
/// failure carrying a reason string (fresh `github_client::Error` is
/// constructed per-call since `Error` is not `Clone`).
#[derive(Clone)]
enum MockOutcome {
    Success,
    Failure(String),
}

/// Local mock `RepositoryClient` for `ActionsManager` tests.
///
/// Implements the full trait (required since `RepositoryClient` is a
/// polymorphic trait object contract) but only the two new methods under
/// test have interesting behavior; every other method is a trivial stub
/// that must never be called by `ActionsManager::apply_action_settings`.
struct MockRepositoryClient {
    workflow_permissions_outcome: MockOutcome,
    access_level_outcome: MockOutcome,
    workflow_permissions_calls: Arc<Mutex<Vec<(String, String, ActionsWorkflowPermissionsUpdate)>>>,
    access_level_calls: Arc<Mutex<Vec<(String, String, ActionsAccessLevelUpdate)>>>,
}

impl MockRepositoryClient {
    fn new() -> Self {
        Self {
            workflow_permissions_outcome: MockOutcome::Success,
            access_level_outcome: MockOutcome::Success,
            workflow_permissions_calls: Arc::new(Mutex::new(Vec::new())),
            access_level_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_workflow_permissions_failure(mut self, reason: &str) -> Self {
        self.workflow_permissions_outcome = MockOutcome::Failure(reason.to_string());
        self
    }

    fn with_access_level_failure(mut self, reason: &str) -> Self {
        self.access_level_outcome = MockOutcome::Failure(reason.to_string());
        self
    }

    fn workflow_permissions_call_count(&self) -> usize {
        self.workflow_permissions_calls.lock().unwrap().len()
    }

    fn access_level_call_count(&self) -> usize {
        self.access_level_calls.lock().unwrap().len()
    }

    fn last_workflow_permissions_call(
        &self,
    ) -> Option<(String, String, ActionsWorkflowPermissionsUpdate)> {
        self.workflow_permissions_calls
            .lock()
            .unwrap()
            .last()
            .cloned()
    }

    fn last_access_level_call(&self) -> Option<(String, String, ActionsAccessLevelUpdate)> {
        self.access_level_calls.lock().unwrap().last().cloned()
    }
}

#[async_trait]
impl RepositoryClient for MockRepositoryClient {
    async fn create_org_repository(
        &self,
        _owner: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<Repository, GHError> {
        Ok(Repository::new(
            payload.name.clone(),
            format!("mock-org/{}", payload.name),
            "MOCK_NODE_ID".to_string(),
            false,
        ))
    }

    async fn create_user_repository(
        &self,
        _payload: &RepositoryCreatePayload,
    ) -> Result<Repository, GHError> {
        Err(GHError::AuthError("not used in these tests".to_string()))
    }

    async fn update_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
        _settings: &RepositorySettingsUpdate,
    ) -> Result<Repository, GHError> {
        Err(GHError::AuthError("not used in these tests".to_string()))
    }

    async fn get_installation_token_for_org(&self, _org_name: &str) -> Result<String, GHError> {
        Ok("mock-token".to_string())
    }

    async fn get_organization_default_branch(&self, _org_name: &str) -> Result<String, GHError> {
        Ok("main".to_string())
    }

    async fn set_repository_custom_properties(
        &self,
        _owner: &str,
        _repo: &str,
        _payload: &github_client::CustomPropertiesPayload,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn search_repositories(&self, _query: &str) -> Result<Vec<Repository>, GHError> {
        Ok(vec![])
    }

    async fn get_custom_properties(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<std::collections::HashMap<String, String>, GHError> {
        Ok(std::collections::HashMap::new())
    }

    async fn list_repository_labels(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<String>, GHError> {
        Ok(vec![])
    }

    async fn create_label(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _color: &str,
        _description: &str,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn get_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Repository, GHError> {
        Err(GHError::AuthError("not used in these tests".to_string()))
    }

    async fn get_branch_protection(
        &self,
        _owner: &str,
        _repo: &str,
        _branch: &str,
    ) -> Result<Option<BranchProtection>, GHError> {
        Ok(None)
    }

    async fn list_repository_files(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<String>, GHError> {
        Ok(vec![])
    }

    async fn list_webhooks(&self, _owner: &str, _repo: &str) -> Result<Vec<Webhook>, GHError> {
        Ok(vec![])
    }

    async fn create_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _params: &CreateWebhookParams<'_>,
    ) -> Result<Webhook, GHError> {
        Err(GHError::AuthError("not used in these tests".to_string()))
    }

    async fn update_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _webhook_id: u64,
        _params: &UpdateWebhookParams<'_>,
    ) -> Result<Webhook, GHError> {
        Err(GHError::AuthError("not used in these tests".to_string()))
    }

    async fn delete_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _webhook_id: u64,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn list_repository_rulesets(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<RepositoryRuleset>, GHError> {
        Ok(Vec::new())
    }

    async fn create_repository_ruleset(
        &self,
        _owner: &str,
        _repo: &str,
        ruleset: &RepositoryRuleset,
    ) -> Result<RepositoryRuleset, GHError> {
        let mut result = ruleset.clone();
        result.id = Some(1);
        Ok(result)
    }

    async fn update_repository_ruleset(
        &self,
        _owner: &str,
        _repo: &str,
        _ruleset_id: u64,
        ruleset: &RepositoryRuleset,
    ) -> Result<RepositoryRuleset, GHError> {
        Ok(ruleset.clone())
    }

    async fn update_label(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _new_name: &str,
        _color: &str,
        _description: &str,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn delete_label(&self, _owner: &str, _repo: &str, _name: &str) -> Result<(), GHError> {
        Ok(())
    }

    async fn create_fork(
        &self,
        _source_owner: &str,
        source_repo: &str,
        target_owner: Option<&str>,
    ) -> Result<Repository, GHError> {
        let owner = target_owner.unwrap_or("mock-owner");
        Ok(Repository::new(
            source_repo.to_string(),
            format!("{owner}/{source_repo}"),
            "MOCK_NODE_ID".to_string(),
            false,
        ))
    }

    async fn create_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        payload: &PullRequestCreatePayload,
    ) -> Result<PullRequest, GHError> {
        Ok(PullRequest {
            number: 1,
            html_url: String::new(),
            draft: payload.draft,
            state: "open".to_string(),
        })
    }

    async fn mark_pull_request_ready_for_review(
        &self,
        _owner: &str,
        _repo: &str,
        pr_number: u64,
    ) -> Result<PullRequest, GHError> {
        Ok(PullRequest {
            number: pr_number,
            html_url: String::new(),
            draft: false,
            state: "open".to_string(),
        })
    }

    async fn add_team_to_repository(
        &self,
        _org: &str,
        _team_slug: &str,
        _repo: &str,
        _permission: &str,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn list_repository_collaborators(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<github_client::Collaborator>, GHError> {
        Ok(vec![])
    }

    async fn add_repository_collaborator(
        &self,
        _owner: &str,
        _repo: &str,
        _username: &str,
        _permission: &str,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn remove_repository_collaborator(
        &self,
        _owner: &str,
        _repo: &str,
        _username: &str,
    ) -> Result<(), GHError> {
        Ok(())
    }

    async fn set_actions_workflow_permissions(
        &self,
        owner: &str,
        repo: &str,
        payload: &ActionsWorkflowPermissionsUpdate,
    ) -> Result<(), GHError> {
        self.workflow_permissions_calls.lock().unwrap().push((
            owner.to_string(),
            repo.to_string(),
            payload.clone(),
        ));
        match &self.workflow_permissions_outcome {
            MockOutcome::Success => Ok(()),
            MockOutcome::Failure(reason) => Err(GHError::AuthError(reason.clone())),
        }
    }

    async fn set_actions_access_level(
        &self,
        owner: &str,
        repo: &str,
        payload: &ActionsAccessLevelUpdate,
    ) -> Result<(), GHError> {
        self.access_level_calls.lock().unwrap().push((
            owner.to_string(),
            repo.to_string(),
            payload.clone(),
        ));
        match &self.access_level_outcome {
            MockOutcome::Success => Ok(()),
            MockOutcome::Failure(reason) => Err(GHError::AuthError(reason.clone())),
        }
    }
}

// ============================================================================
// Tier 1: Specification tests -- which calls happen for which fields
// ============================================================================

#[tokio::test]
async fn test_actions_manager_can_be_constructed_with_new() {
    let mock = Arc::new(MockRepositoryClient::new());
    let _manager = ActionsManager::new(mock as Arc<dyn RepositoryClient>);
    // Compiles and constructs -- no panics.
}

#[tokio::test]
async fn test_apply_action_settings_calls_workflow_permissions_when_default_workflow_permissions_set(
) {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 1);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_calls_workflow_permissions_when_can_approve_pull_request_reviews_set(
) {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 1);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_calls_access_level_when_access_level_set() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.access_level_call_count(), 1);
    assert_eq!(mock.workflow_permissions_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_calls_both_endpoints_when_all_three_new_fields_set() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("read".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(false)),
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 1);
    assert_eq!(mock.access_level_call_count(), 1);
}

#[tokio::test]
async fn test_apply_action_settings_makes_no_calls_when_all_three_new_fields_absent() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let result = manager
        .apply_action_settings("acme", "widgets", &ActionSettings::default())
        .await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_passes_correct_owner_and_repo_to_workflow_permissions_call() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    };

    manager
        .apply_action_settings("my-org", "my-repo", &actions)
        .await
        .expect("should succeed");

    let (owner, repo, _payload) = mock
        .last_workflow_permissions_call()
        .expect("call recorded");
    assert_eq!(owner, "my-org");
    assert_eq!(repo, "my-repo");
}

#[tokio::test]
async fn test_apply_action_settings_passes_correct_owner_and_repo_to_access_level_call() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        access_level: Some(OverridableValue::allowed("none".to_string())),
        ..Default::default()
    };

    manager
        .apply_action_settings("my-org", "my-repo", &actions)
        .await
        .expect("should succeed");

    let (owner, repo, _payload) = mock.last_access_level_call().expect("call recorded");
    assert_eq!(owner, "my-org");
    assert_eq!(repo, "my-repo");
}

#[tokio::test]
async fn test_apply_action_settings_builds_workflow_permissions_payload_with_only_default_workflow_permissions(
) {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    let (_, _, payload) = mock
        .last_workflow_permissions_call()
        .expect("call recorded");
    assert_eq!(
        payload.default_workflow_permissions,
        Some("write".to_string())
    );
    assert_eq!(
        payload.can_approve_pull_request_reviews, None,
        "unset field must not be synthesized into the payload"
    );
}

#[tokio::test]
async fn test_apply_action_settings_builds_workflow_permissions_payload_with_only_can_approve_pull_request_reviews(
) {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    let (_, _, payload) = mock
        .last_workflow_permissions_call()
        .expect("call recorded");
    assert_eq!(payload.default_workflow_permissions, None);
    assert_eq!(payload.can_approve_pull_request_reviews, Some(true));
}

#[tokio::test]
async fn test_apply_action_settings_builds_workflow_permissions_payload_with_both_fields() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("read".to_string())),
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(false)),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    let (_, _, payload) = mock
        .last_workflow_permissions_call()
        .expect("call recorded");
    assert_eq!(
        payload.default_workflow_permissions,
        Some("read".to_string())
    );
    assert_eq!(payload.can_approve_pull_request_reviews, Some(false));
}

#[tokio::test]
async fn test_apply_action_settings_builds_access_level_payload_with_correct_value() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    let (_, _, payload) = mock.last_access_level_call().expect("call recorded");
    assert_eq!(payload.access_level, "enterprise");
}

// ============================================================================
// Tier 2: Adversarial -- legacy fields must never trigger a call
// ============================================================================

#[tokio::test]
async fn test_apply_action_settings_ignores_enabled_field() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        enabled: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_ignores_allowed_actions_field() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        allowed_actions: Some(OverridableValue::allowed("selected".to_string())),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_ignores_github_owned_allowed_field() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        github_owned_allowed: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_ignores_verified_allowed_field() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        verified_allowed: Some(OverridableValue::allowed(false)),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

#[tokio::test]
async fn test_apply_action_settings_ignores_patterns_allowed_field() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        patterns_allowed: Some(vec!["actions/checkout@*".to_string()]),
        ..Default::default()
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

/// Stub-killing: a combined setting of all 5 legacy fields together must
/// still make zero calls -- rules out an implementation that only guards
/// against a single legacy field in isolation.
#[tokio::test]
async fn test_apply_action_settings_ignores_all_five_legacy_fields_combined() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        enabled: Some(OverridableValue::allowed(true)),
        allowed_actions: Some(OverridableValue::allowed("all".to_string())),
        github_owned_allowed: Some(OverridableValue::allowed(true)),
        verified_allowed: Some(OverridableValue::allowed(true)),
        patterns_allowed: Some(vec!["org/*".to_string()]),
        default_workflow_permissions: None,
        can_approve_pull_request_reviews: None,
        access_level: None,
    };

    manager
        .apply_action_settings("acme", "widgets", &actions)
        .await
        .expect("should succeed");

    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

// ============================================================================
// Tier 2: Adversarial -- error propagation
// ============================================================================

#[tokio::test]
async fn test_apply_action_settings_propagates_workflow_permissions_error() {
    let mock = Arc::new(MockRepositoryClient::new().with_workflow_permissions_failure("boom"));
    let manager = ActionsManager::new(mock as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(
        result.is_err(),
        "a failing GitHub call must propagate, not be swallowed"
    );
    match result.unwrap_err() {
        RepoRollerError::GitHub(GitHubError::NetworkError { .. }) => {}
        other => panic!("expected RepoRollerError::GitHub(NetworkError), got: {other:?}"),
    }
}

#[tokio::test]
async fn test_apply_action_settings_propagates_access_level_error() {
    let mock = Arc::new(MockRepositoryClient::new().with_access_level_failure("denied"));
    let manager = ActionsManager::new(mock as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        access_level: Some(OverridableValue::allowed("enterprise".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RepoRollerError::GitHub(GitHubError::NetworkError { .. }) => {}
        other => panic!("expected RepoRollerError::GitHub(NetworkError), got: {other:?}"),
    }
}

/// Documents/pins the sequential-call contract implied by the interface
/// spec ("if X: call; map errors. if Y: call; map errors."): when both
/// `default_workflow_permissions` and `access_level` are set and the
/// workflow-permissions call fails, the function must return the error
/// (via `?`) before ever attempting the access-level call.
#[tokio::test]
async fn test_apply_action_settings_does_not_attempt_access_level_call_after_workflow_permissions_failure(
) {
    let mock = Arc::new(MockRepositoryClient::new().with_workflow_permissions_failure("boom"));
    let manager = ActionsManager::new(mock.clone() as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_err());
    assert_eq!(
        mock.access_level_call_count(),
        0,
        "the access-level call must never be attempted once the workflow-permissions call has failed"
    );
}

#[tokio::test]
async fn test_apply_action_settings_result_is_ok_when_both_calls_succeed() {
    let mock = Arc::new(MockRepositoryClient::new());
    let manager = ActionsManager::new(mock as Arc<dyn RepositoryClient>);

    let actions = ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    };

    let result = manager
        .apply_action_settings("acme", "widgets", &actions)
        .await;

    assert!(result.is_ok(), "got: {result:?}");
}
