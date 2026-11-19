//! Tests for configuration module.

// Note: `resolve_organization_configuration` itself is integration-style and
// requires a live GitHub client, a real metadata repository, and (for the
// #255 follow-up "designated template is never auto-discovered" fix) real
// template repositories to exercise its new auto-discovery branch
// meaningfully. By design it constructs its own `GitHubMetadataProvider` /
// `TemplateLoader` / `OrganizationSettingsManager` internally from a bare
// installation token with no injection seam -- per
// docs/spec/interfaces/fork-mirror-operations.md and the #255 follow-up gap
// note ("it must call the settings_manager it already constructs
// internally"), this is intentional and not something this test suite
// should work around with a seam of its own. Comprehensive behavioral
// testing of the full resolution pipeline (including live network calls) is
// done at the integration test level in the integration_tests crate, where
// real (credentialed) GitHub App clients are used -- see
// crates/integration_tests/src/test_runner.rs. Unit tests here focus on
// ensuring the module compiles and exports the expected functions, plus the
// one piece of the #255 follow-up fix that *is* synchronous and
// network-free: `effective_template_name` below.
//
// `effective_template_name` isolates the decision of which template name
// `resolve_organization_configuration` must use to build its
// `ConfigurationContext`, given:
//   (a) the caller's explicit `template_name` (empty string `""` when the
//       caller gave none),
//   (b) the `TemplateDesignation` implied by the request's `CreationMode`
//       (`None` for `CreationMode::Standard`; see
//       `designation_for_creation_mode` in `lib_tests.rs`), and
//   (c) the `ConfigurationResult<Option<String>>` returned by calling
//       `settings_manager.find_designated_template(organization,
//       designation)` -- which `resolve_organization_configuration` must
//       only actually invoke when `template_name.is_empty() &&
//       designation.is_some()`.
//
// `resolve_organization_configuration` MUST delegate to this exact function
// (or logic equivalent to it) to decide the effective template name, and
// MUST propagate any `Err` it returns (notably `AmbiguousTemplateDesignation`)
// rather than swallowing it into the metadata-repository-unavailable
// fallback path (that fallback is for CRF-001 -- "metadata repository
// cannot be discovered" -- a different, legitimate case).
//
// This test file pins down that decision table so the auto-discovery wiring
// (the actual #255 follow-up defect: "load_creation_config only loads a
// template when the caller's request.template is explicitly Some(...)")
// cannot regress to "always ignore designation" or "always prefer discovery
// over an explicit caller choice".

use config_manager::{ConfigurationError, TemplateDesignation};

use super::*;

/// Verify that the configuration module compiles and exports expected functions.
///
/// This test ensures the module's public interface is available.
/// Full behavioral testing is done in integration tests.
#[test]
fn test_configuration_module_compiles() {
    // This test just needs to compile to verify the module structure is correct.
    // The actual functions are tested via integration tests with mocked clients.

    // Verify types are accessible
    let _: Option<config_manager::MergedConfiguration> = None;
}

// ============================================================================
// effective_template_name Tests (RED: #255 follow-up Gap 1 --
// "designated template is never auto-discovered")
// ============================================================================

#[test]
fn test_effective_template_name_explicit_template_wins_over_discovered_name() {
    let result = effective_template_name(
        "explicit-template",
        Some(TemplateDesignation::Fork),
        Ok(Some("discovered-template".to_string())),
    );

    assert_eq!(
        result,
        Ok(Some("explicit-template".to_string())),
        "an explicitly named template must always win over auto-discovery"
    );
}

#[test]
fn test_effective_template_name_explicit_template_wins_even_over_ambiguous_discovery_error() {
    // Adversarial: proves the explicit-template branch short-circuits
    // BEFORE even inspecting the discovery result. A lazy implementation
    // that always evaluates/propagates `discovery_result` regardless of
    // `template_name` would fail this test even though it might pass the
    // "wins over Ok(Some(..))" test above.
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "Fork".to_string(),
        templates: vec!["fork-a".to_string(), "fork-b".to_string()],
    });

    let result = effective_template_name(
        "explicit-template",
        Some(TemplateDesignation::Fork),
        ambiguous,
    );

    assert_eq!(result, Ok(Some("explicit-template".to_string())));
}

#[test]
fn test_effective_template_name_standard_mode_ignores_discovered_name() {
    let result = effective_template_name("", None, Ok(Some("ignored-template".to_string())));

    assert_eq!(
        result,
        Ok(None),
        "CreationMode::Standard (designation = None) must never surface a discovered name"
    );
}

#[test]
fn test_effective_template_name_no_designation_returns_none_even_if_discovery_errored() {
    // Even if a (buggy) caller passed an Err discovery_result alongside
    // designation = None, Standard mode must not propagate it -- discovery
    // should never have been attempted in the first place when there is no
    // designation to discover.
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "Fork".to_string(),
        templates: vec!["fork-a".to_string(), "fork-b".to_string()],
    });

    let result = effective_template_name("", None, ambiguous);

    assert_eq!(result, Ok(None));
}

#[test]
fn test_effective_template_name_auto_discovers_designated_template_when_found() {
    let result = effective_template_name(
        "",
        Some(TemplateDesignation::SourceHistoryMirror),
        Ok(Some("history-mirror-settings".to_string())),
    );

    assert_eq!(result, Ok(Some("history-mirror-settings".to_string())));
}

#[test]
fn test_effective_template_name_distinct_designations_can_discover_distinct_names() {
    // Stub-killing: a lazy implementation that ignores `designation`/
    // `discovery_result` and always returns a fixed name would fail here,
    // since the two calls below must surface the two distinct discovered
    // names unchanged.
    let fork_result = effective_template_name(
        "",
        Some(TemplateDesignation::Fork),
        Ok(Some("fork-settings".to_string())),
    );
    let mirror_result = effective_template_name(
        "",
        Some(TemplateDesignation::SourceOnlyMirror),
        Ok(Some("mirror-settings".to_string())),
    );

    assert_eq!(fork_result, Ok(Some("fork-settings".to_string())));
    assert_eq!(mirror_result, Ok(Some("mirror-settings".to_string())));
    assert_ne!(fork_result, mirror_result);
}

#[test]
fn test_effective_template_name_no_designated_template_found_returns_none_not_error() {
    let result = effective_template_name("", Some(TemplateDesignation::SourceOnlyMirror), Ok(None));

    assert_eq!(
        result,
        Ok(None),
        "no designated template existing for the organization is a valid outcome, not an error"
    );
}

#[test]
fn test_effective_template_name_propagates_ambiguous_designation_error() {
    let ambiguous = Err(ConfigurationError::AmbiguousTemplateDesignation {
        org: "acme-corp".to_string(),
        designation: "SourceOnlyMirror".to_string(),
        templates: vec!["mirror-a".to_string(), "mirror-b".to_string()],
    });

    let result =
        effective_template_name("", Some(TemplateDesignation::SourceOnlyMirror), ambiguous);

    assert!(
        matches!(
            result,
            Err(ConfigurationError::AmbiguousTemplateDesignation { .. })
        ),
        "an ambiguous designation must propagate as an error, not be swallowed: {result:?}"
    );
}

#[test]
fn test_effective_template_name_whitespace_only_name_counts_as_explicit() {
    // Boundary: the contract is literally `template_name.is_empty()`, not
    // `.trim().is_empty()`. A single space must take the explicit-template
    // path (and therefore ignore discovery entirely), not the auto-discovery
    // path.
    let result = effective_template_name(
        " ",
        Some(TemplateDesignation::Fork),
        Ok(Some("discovered-template".to_string())),
    );

    assert_eq!(result, Ok(Some(" ".to_string())));
}

// ============================================================================
// `apply_repository_configuration` -- ActionsManager wiring (#276/#277)
//
// Prior to this change `apply_repository_configuration` had no unit test
// coverage at all (see the module-level comment above: it requires a mock
// `RepositoryClient`). These tests introduce exactly that mock, scoped to
// what's needed to pin down the *new* wiring: "when any of the 3 new
// `ActionSettings` fields is set on `merged_config.actions`, ActionsManager
// must be invoked, and its errors must propagate rather than being
// swallowed" (contrast with the non-fatal-warn convention used elsewhere in
// lib.rs for team/collaborator permissions -- this path must NOT swallow).
// ============================================================================

use async_trait::async_trait;
use config_manager::settings::ActionSettings;
use config_manager::{MergedConfiguration, OverridableValue};
use github_client::{
    ActionsAccessLevelUpdate, ActionsWorkflowPermissionsUpdate, BranchProtection,
    CreateWebhookParams, Error as GHError, PullRequest, PullRequestCreatePayload, Repository,
    RepositoryClient, RepositoryCreatePayload, RepositoryRuleset, RepositorySettingsUpdate,
    UpdateWebhookParams, Webhook,
};
use std::sync::{Arc, Mutex};

/// Local mock `RepositoryClient` for `apply_repository_configuration` tests.
///
/// Only `set_actions_workflow_permissions` / `set_actions_access_level` are
/// tracked; every other method is a trivial always-succeeds stub, since the
/// scope here is "does `apply_repository_configuration` call `ActionsManager`
/// at the right time", not re-testing labels/webhooks/rulesets/custom
/// properties behavior (which have their own manager-level test suites).
struct MockActionsRepoClient {
    fail_workflow_permissions: bool,
    workflow_permissions_calls: Arc<Mutex<usize>>,
    access_level_calls: Arc<Mutex<usize>>,
}

impl MockActionsRepoClient {
    fn new() -> Self {
        Self {
            fail_workflow_permissions: false,
            workflow_permissions_calls: Arc::new(Mutex::new(0)),
            access_level_calls: Arc::new(Mutex::new(0)),
        }
    }

    fn failing_workflow_permissions() -> Self {
        Self {
            fail_workflow_permissions: true,
            ..Self::new()
        }
    }

    fn workflow_permissions_call_count(&self) -> usize {
        *self.workflow_permissions_calls.lock().unwrap()
    }

    fn access_level_call_count(&self) -> usize {
        *self.access_level_calls.lock().unwrap()
    }
}

#[async_trait]
impl RepositoryClient for MockActionsRepoClient {
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
        _owner: &str,
        _repo: &str,
        _payload: &ActionsWorkflowPermissionsUpdate,
    ) -> Result<(), GHError> {
        *self.workflow_permissions_calls.lock().unwrap() += 1;
        if self.fail_workflow_permissions {
            Err(GHError::AuthError("mock failure".to_string()))
        } else {
            Ok(())
        }
    }

    async fn set_actions_access_level(
        &self,
        _owner: &str,
        _repo: &str,
        _payload: &ActionsAccessLevelUpdate,
    ) -> Result<(), GHError> {
        *self.access_level_calls.lock().unwrap() += 1;
        Ok(())
    }
}

fn merged_config_with_actions(actions: ActionSettings) -> MergedConfiguration {
    let mut config = MergedConfiguration::new();
    config.actions = actions;
    config
}

#[tokio::test]
async fn test_apply_repository_configuration_invokes_actions_manager_when_default_workflow_permissions_set(
) {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();
    let merged_config = merged_config_with_actions(ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    });

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 1);
}

#[tokio::test]
async fn test_apply_repository_configuration_invokes_actions_manager_when_can_approve_pull_request_reviews_set(
) {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();
    let merged_config = merged_config_with_actions(ActionSettings {
        can_approve_pull_request_reviews: Some(OverridableValue::allowed(true)),
        ..Default::default()
    });

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 1);
}

#[tokio::test]
async fn test_apply_repository_configuration_invokes_actions_manager_when_access_level_set() {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();
    let merged_config = merged_config_with_actions(ActionSettings {
        access_level: Some(OverridableValue::allowed("organization".to_string())),
        ..Default::default()
    });

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.access_level_call_count(), 1);
}

#[tokio::test]
async fn test_apply_repository_configuration_does_not_invoke_actions_endpoints_when_actions_default(
) {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();
    let merged_config = MergedConfiguration::new(); // actions: ActionSettings::default()

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

/// Stub-killing: a `merged_config.actions` with only the 5 legacy
/// (out-of-scope) fields set must not trigger either new endpoint, even
/// though `apply_repository_configuration` must still return `Ok`.
#[tokio::test]
async fn test_apply_repository_configuration_does_not_invoke_actions_endpoints_for_legacy_fields_only(
) {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();
    let merged_config = merged_config_with_actions(ActionSettings {
        enabled: Some(OverridableValue::allowed(true)),
        allowed_actions: Some(OverridableValue::allowed("all".to_string())),
        ..Default::default()
    });

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.workflow_permissions_call_count(), 0);
    assert_eq!(mock.access_level_call_count(), 0);
}

/// Spec: errors from the ActionsManager path must propagate, NOT be
/// swallowed -- contrast with the non-fatal-warn convention used elsewhere
/// in `lib.rs` for team/collaborator permissions. This mirrors the existing
/// convention for labels/webhooks/rulesets/custom-properties in this same
/// function (all of which propagate via `?`).
#[tokio::test]
async fn test_apply_repository_configuration_propagates_actions_manager_error() {
    let mock = Arc::new(MockActionsRepoClient::failing_workflow_permissions());
    let client: Arc<dyn RepositoryClient> = mock;
    let merged_config = merged_config_with_actions(ActionSettings {
        default_workflow_permissions: Some(OverridableValue::allowed("write".to_string())),
        ..Default::default()
    });

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(
        result.is_err(),
        "a failing ActionsManager call must propagate as an error, not be swallowed"
    );
}

/// Stub-killing: the new `actions` block must be applied independently of
/// labels/webhooks/rulesets/custom_properties -- an implementation that only
/// checks `merged_config.actions` inside one of the pre-existing
/// `if !merged_config.X.is_empty()` blocks (instead of its own standalone
/// check) would skip it here, since all of those collections are empty.
#[tokio::test]
async fn test_apply_repository_configuration_applies_actions_even_when_everything_else_is_empty() {
    let mock = Arc::new(MockActionsRepoClient::new());
    let client: Arc<dyn RepositoryClient> = mock.clone();

    // Sanity: confirm the other collections really are empty on a fresh
    // MergedConfiguration, so this test actually isolates the `actions` path.
    let merged_config = merged_config_with_actions(ActionSettings {
        access_level: Some(OverridableValue::allowed("none".to_string())),
        ..Default::default()
    });
    assert!(merged_config.labels.is_empty());
    assert!(merged_config.webhooks.is_empty());
    assert!(merged_config.rulesets.is_empty());
    assert!(merged_config.custom_properties.is_empty());

    let result = apply_repository_configuration(&client, "acme", "widgets", &merged_config).await;

    assert!(result.is_ok(), "got: {result:?}");
    assert_eq!(mock.access_level_call_count(), 1);
}

// Integration-level tests (full `resolve_organization_configuration` pipeline,
// including the new auto-discovery wiring exercised against a live
// metadata/template repository) are in crates/integration_tests/tests/
// where we can use real, credentialed:
// - GitHubClient for repository operations
// - MetadataProvider for configuration access
// - Full configuration resolution workflow, including designated-template
//   auto-discovery
// - Configuration application with custom properties
