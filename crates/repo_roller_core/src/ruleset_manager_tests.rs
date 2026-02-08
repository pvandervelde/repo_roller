//! Tests for ruleset_manager module.

use super::*;
use config_manager::settings::{
    RefNameConditionConfig, RuleConfig, RulesetConditionsConfig, RulesetConfig,
};
use github_client::{RepositoryRuleset, RulesetEnforcement, RulesetTarget};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock GitHub client for testing
// Note: Currently unused as tests require trait-based client for proper mocking
// Kept for future integration tests when GitHubClient becomes trait-based
#[allow(dead_code)]
#[derive(Clone)]
struct MockGitHubClient {
    list_rulesets_result: Arc<Mutex<Result<Vec<RepositoryRuleset>, GitHubError>>>,
    create_ruleset_result: Arc<Mutex<Result<RepositoryRuleset, GitHubError>>>,
    update_ruleset_result: Arc<Mutex<Result<RepositoryRuleset, GitHubError>>>,
    list_calls: Arc<Mutex<Vec<(String, String)>>>,
    create_calls: Arc<Mutex<Vec<(String, String, RepositoryRuleset)>>>,
    #[allow(clippy::type_complexity)]
    update_calls: Arc<Mutex<Vec<(String, String, u64, RepositoryRuleset)>>>,
}

#[allow(dead_code)]
impl MockGitHubClient {
    fn new() -> Self {
        Self {
            list_rulesets_result: Arc::new(Mutex::new(Ok(Vec::new()))),
            create_ruleset_result: Arc::new(Mutex::new(Ok(RepositoryRuleset {
                id: Some(1),
                name: "test".to_string(),
                target: RulesetTarget::Branch,
                enforcement: RulesetEnforcement::Active,
                bypass_actors: Vec::new(),
                conditions: None,
                rules: Vec::new(),
            }))),
            update_ruleset_result: Arc::new(Mutex::new(Ok(RepositoryRuleset {
                id: Some(1),
                name: "test".to_string(),
                target: RulesetTarget::Branch,
                enforcement: RulesetEnforcement::Active,
                bypass_actors: Vec::new(),
                conditions: None,
                rules: Vec::new(),
            }))),
            list_calls: Arc::new(Mutex::new(Vec::new())),
            create_calls: Arc::new(Mutex::new(Vec::new())),
            update_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_list_result(&self, result: Result<Vec<RepositoryRuleset>, GitHubError>) {
        *self.list_rulesets_result.blocking_lock() = result;
    }

    fn set_create_result(&self, result: Result<RepositoryRuleset, GitHubError>) {
        *self.create_ruleset_result.blocking_lock() = result;
    }

    fn set_update_result(&self, result: Result<RepositoryRuleset, GitHubError>) {
        *self.update_ruleset_result.blocking_lock() = result;
    }

    fn get_list_calls(&self) -> Vec<(String, String)> {
        self.list_calls.blocking_lock().clone()
    }

    fn get_create_calls(&self) -> Vec<(String, String, RepositoryRuleset)> {
        self.create_calls.blocking_lock().clone()
    }

    fn get_update_calls(&self) -> Vec<(String, String, u64, RepositoryRuleset)> {
        self.update_calls.blocking_lock().clone()
    }
}

// Helper to create minimal ruleset config
#[allow(dead_code)]
fn create_test_ruleset_config(name: &str) -> RulesetConfig {
    RulesetConfig {
        name: name.to_string(),
        target: "branch".to_string(),
        enforcement: "active".to_string(),
        bypass_actors: vec![],
        conditions: Some(RulesetConditionsConfig {
            ref_name: RefNameConditionConfig {
                include: vec!["refs/heads/main".to_string()],
                exclude: vec![],
            },
        }),
        rules: vec![RuleConfig::Deletion],
    }
}

// ============================================================================
// ApplyRulesetsResult Tests
// ============================================================================

#[test]
fn test_apply_rulesets_result_new() {
    let result = ApplyRulesetsResult::new();

    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.failed, 0);
    assert!(result.failed_rulesets.is_empty());
}

#[test]
fn test_apply_rulesets_result_default() {
    let result = ApplyRulesetsResult::default();

    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.failed, 0);
}

#[test]
fn test_apply_rulesets_result_is_success_when_no_failures() {
    let result = ApplyRulesetsResult {
        created: 3,
        updated: 2,
        failed: 0,
        failed_rulesets: vec![],
    };

    assert!(result.is_success());
}

#[test]
fn test_apply_rulesets_result_is_not_success_when_failures() {
    let result = ApplyRulesetsResult {
        created: 3,
        updated: 2,
        failed: 1,
        failed_rulesets: vec!["failed-ruleset".to_string()],
    };

    assert!(!result.is_success());
}

#[test]
fn test_apply_rulesets_result_has_changes_when_created() {
    let result = ApplyRulesetsResult {
        created: 1,
        updated: 0,
        failed: 0,
        failed_rulesets: vec![],
    };

    assert!(result.has_changes());
}

#[test]
fn test_apply_rulesets_result_has_changes_when_updated() {
    let result = ApplyRulesetsResult {
        created: 0,
        updated: 1,
        failed: 0,
        failed_rulesets: vec![],
    };

    assert!(result.has_changes());
}

#[test]
fn test_apply_rulesets_result_no_changes_when_none() {
    let result = ApplyRulesetsResult {
        created: 0,
        updated: 0,
        failed: 0,
        failed_rulesets: vec![],
    };

    assert!(!result.has_changes());
}

// ============================================================================
// RulesetManager Tests - Require GitHub Client mock
// ============================================================================

// Note: These tests demonstrate the expected behavior but won't compile
// without either:
// 1. A trait-based GitHubClient (for easy mocking)
// 2. Integration tests with wiremock
// 3. A test-specific constructor on RulesetManager

// For now, documenting expected test cases:

/*
#[tokio::test]
async fn test_ruleset_manager_new() {
    // Test: RulesetManager::new creates instance with client
    // Expected: Manager stores client reference
}

#[tokio::test]
async fn test_apply_rulesets_creates_new_rulesets() {
    // Test: apply_rulesets with empty existing rulesets creates all
    // Given: Repository with no existing rulesets
    // When: apply_rulesets called with 3 rulesets
    // Then: 3 create calls made, result shows created=3
}

#[tokio::test]
async fn test_apply_rulesets_updates_existing_rulesets() {
    // Test: apply_rulesets updates rulesets that already exist
    // Given: Repository with 2 existing rulesets (matching names)
    // When: apply_rulesets called with same 2 rulesets
    // Then: 2 update calls made, result shows updated=2
}

#[tokio::test]
async fn test_apply_rulesets_mixed_create_and_update() {
    // Test: apply_rulesets handles mix of new and existing
    // Given: Repository with 1 existing ruleset
    // When: apply_rulesets called with 3 rulesets (1 matching, 2 new)
    // Then: 1 update call + 2 create calls, result shows created=2 updated=1
}

#[tokio::test]
async fn test_apply_rulesets_handles_create_failure() {
    // Test: apply_rulesets continues on individual failures
    // Given: GitHub API returns error for one ruleset creation
    // When: apply_rulesets called with 3 rulesets
    // Then: Other 2 succeed, result shows created=2 failed=1
}

#[tokio::test]
async fn test_apply_rulesets_handles_update_failure() {
    // Test: apply_rulesets continues when update fails
    // Given: GitHub API returns error for one ruleset update
    // When: apply_rulesets called
    // Then: Operation continues, result shows failed=1
}

#[tokio::test]
async fn test_apply_rulesets_handles_list_failure() {
    // Test: apply_rulesets assumes all new when list fails
    // Given: list_rulesets returns error
    // When: apply_rulesets called with 3 rulesets
    // Then: Attempts to create all 3
}

#[tokio::test]
async fn test_apply_rulesets_is_idempotent() {
    // Test: Calling apply_rulesets twice with same config is safe
    // Given: First call creates 3 rulesets
    // When: Second call with same rulesets
    // Then: Second call updates 3 (not recreates)
}

#[tokio::test]
async fn test_apply_rulesets_empty_config() {
    // Test: apply_rulesets with empty config succeeds immediately
    // Given: Empty ruleset HashMap
    // When: apply_rulesets called
    // Then: No API calls made, result shows all zeros
}

#[tokio::test]
async fn test_list_rulesets_success() {
    // Test: list_rulesets returns rulesets from GitHub
    // Given: Repository with 3 rulesets
    // When: list_rulesets called
    // Then: Returns Vec with 3 rulesets
}

#[tokio::test]
async fn test_list_rulesets_empty_repository() {
    // Test: list_rulesets returns empty vec for new repo
    // Given: Repository with no rulesets
    // When: list_rulesets called
    // Then: Returns empty Vec
}

#[tokio::test]
async fn test_list_rulesets_handles_api_error() {
    // Test: list_rulesets propagates GitHub errors
    // Given: GitHub API returns error
    // When: list_rulesets called
    // Then: Returns Err(RepoRollerError::System)
}
*/

// ============================================================================
// Documentation Tests
// ============================================================================

/// Verify example in ApplyRulesetsResult documentation compiles.
#[test]
fn test_apply_rulesets_result_example() {
    let result = ApplyRulesetsResult {
        created: 2,
        updated: 1,
        failed: 0,
        failed_rulesets: vec![],
    };

    assert!(result.is_success());
    assert!(result.has_changes());
}
