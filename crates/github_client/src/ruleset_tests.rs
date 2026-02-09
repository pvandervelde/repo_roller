//! Tests for repository ruleset types.

use super::*;
use serde_json::{from_str, to_string};

// ============================================================================
// RepositoryRuleset Tests
// ============================================================================

/// Test basic ruleset deserialization from GitHub API response.
#[test]
fn test_ruleset_deserialization() {
    let json = r#"{
        "id": 123,
        "name": "main-protection",
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [],
        "rules": []
    }"#;

    let ruleset: RepositoryRuleset = from_str(json).expect("Failed to deserialize");

    assert_eq!(ruleset.id, Some(123));
    assert_eq!(ruleset.name, "main-protection");
    assert_eq!(ruleset.target, RulesetTarget::Branch);
    assert_eq!(ruleset.enforcement, RulesetEnforcement::Active);
    assert!(ruleset.bypass_actors.is_empty());
    assert!(ruleset.rules.is_empty());
}

/// Test ruleset serialization for API request.
#[test]
fn test_ruleset_serialization() {
    let ruleset = RepositoryRuleset {
        id: None, // Omit ID for creation
        name: "test-ruleset".to_string(),
        target: RulesetTarget::Tag,
        enforcement: RulesetEnforcement::Evaluate,
        bypass_actors: vec![],
        conditions: None,
        rules: vec![],
        node_id: None,
        source: None,
        source_type: None,
        created_at: None,
        updated_at: None,
        _links: None,
    };

    let json = to_string(&ruleset).expect("Failed to serialize");

    // ID should not be present when None
    assert!(!json.contains("\"id\""));
    assert!(json.contains("\"test-ruleset\""));
    assert!(json.contains("\"tag\""));
    assert!(json.contains("\"evaluate\""));
}

/// Test ruleset with conditions.
#[test]
fn test_ruleset_with_conditions() {
    let json = r#"{
        "id": 456,
        "name": "release-protection",
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {
            "ref_name": {
                "include": ["refs/heads/release/*", "refs/heads/main"],
                "exclude": ["refs/heads/release/temp*"]
            }
        },
        "rules": []
    }"#;

    let ruleset: RepositoryRuleset = from_str(json).expect("Failed to deserialize");

    assert!(ruleset.conditions.is_some());
    let conditions = ruleset.conditions.unwrap();
    assert_eq!(conditions.ref_name.include.len(), 2);
    assert_eq!(conditions.ref_name.exclude.len(), 1);
    assert_eq!(conditions.ref_name.include[0], "refs/heads/release/*");
    assert_eq!(conditions.ref_name.exclude[0], "refs/heads/release/temp*");
}

// ============================================================================
// RulesetTarget Tests
// ============================================================================

/// Test RulesetTarget enum serialization.
#[test]
fn test_ruleset_target_serialization() {
    assert_eq!(to_string(&RulesetTarget::Branch).unwrap(), "\"branch\"");
    assert_eq!(to_string(&RulesetTarget::Tag).unwrap(), "\"tag\"");
}

/// Test RulesetTarget enum deserialization.
#[test]
fn test_ruleset_target_deserialization() {
    assert_eq!(
        from_str::<RulesetTarget>("\"branch\"").unwrap(),
        RulesetTarget::Branch
    );
    assert_eq!(
        from_str::<RulesetTarget>("\"tag\"").unwrap(),
        RulesetTarget::Tag
    );
}

// ============================================================================
// RulesetEnforcement Tests
// ============================================================================

/// Test RulesetEnforcement enum values.
#[test]
fn test_ruleset_enforcement_values() {
    let active = RulesetEnforcement::Active;
    let disabled = RulesetEnforcement::Disabled;
    let evaluate = RulesetEnforcement::Evaluate;

    assert_eq!(to_string(&active).unwrap(), "\"active\"");
    assert_eq!(to_string(&disabled).unwrap(), "\"disabled\"");
    assert_eq!(to_string(&evaluate).unwrap(), "\"evaluate\"");
}

/// Test RulesetEnforcement deserialization.
#[test]
fn test_ruleset_enforcement_deserialization() {
    assert_eq!(
        from_str::<RulesetEnforcement>("\"active\"").unwrap(),
        RulesetEnforcement::Active
    );
    assert_eq!(
        from_str::<RulesetEnforcement>("\"disabled\"").unwrap(),
        RulesetEnforcement::Disabled
    );
    assert_eq!(
        from_str::<RulesetEnforcement>("\"evaluate\"").unwrap(),
        RulesetEnforcement::Evaluate
    );
}

// ============================================================================
// BypassActor Tests
// ============================================================================

/// Test BypassActor deserialization.
#[test]
fn test_bypass_actor_deserialization() {
    let json = r#"{
        "actor_id": 789,
        "actor_type": "Team",
        "bypass_mode": "always"
    }"#;

    let actor: BypassActor = from_str(json).expect("Failed to deserialize");

    assert_eq!(actor.actor_id, 789);
    assert_eq!(actor.actor_type, BypassActorType::Team);
    assert_eq!(actor.bypass_mode, BypassMode::Always);
}

/// Test BypassActorType variants.
#[test]
fn test_bypass_actor_type_variants() {
    assert_eq!(
        from_str::<BypassActorType>("\"OrganizationAdmin\"").unwrap(),
        BypassActorType::OrganizationAdmin
    );
    assert_eq!(
        from_str::<BypassActorType>("\"RepositoryRole\"").unwrap(),
        BypassActorType::RepositoryRole
    );
    assert_eq!(
        from_str::<BypassActorType>("\"Team\"").unwrap(),
        BypassActorType::Team
    );
    assert_eq!(
        from_str::<BypassActorType>("\"Integration\"").unwrap(),
        BypassActorType::Integration
    );
}

/// Test BypassMode variants.
#[test]
fn test_bypass_mode_variants() {
    assert_eq!(
        from_str::<BypassMode>("\"always\"").unwrap(),
        BypassMode::Always
    );
    assert_eq!(
        from_str::<BypassMode>("\"pull_request\"").unwrap(),
        BypassMode::PullRequest
    );
}

// ============================================================================
// Rule Tests
// ============================================================================

/// Test Creation rule.
#[test]
fn test_creation_rule() {
    let json = r#"{"type": "creation"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::Creation));
}

/// Test Update rule.
#[test]
fn test_update_rule() {
    let json = r#"{"type": "update"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::Update));
}

/// Test Deletion rule.
#[test]
fn test_deletion_rule() {
    let json = r#"{"type": "deletion"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::Deletion));
}

/// Test RequiredLinearHistory rule.
#[test]
fn test_required_linear_history_rule() {
    let json = r#"{"type": "required_linear_history"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::RequiredLinearHistory));
}

/// Test RequiredSignatures rule.
#[test]
fn test_required_signatures_rule() {
    let json = r#"{"type": "required_signatures"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::RequiredSignatures));
}

/// Test PullRequest rule with parameters.
#[test]
fn test_pull_request_rule() {
    let json = r#"{
        "type": "pull_request",
        "parameters": {
            "dismiss_stale_reviews_on_push": true,
            "require_code_owner_review": true,
            "require_last_push_approval": false,
            "required_approving_review_count": 2,
            "required_review_thread_resolution": true,
            "allowed_merge_methods": ["squash", "rebase"]
        }
    }"#;

    let rule: Rule = from_str(json).expect("Failed to deserialize");

    match rule {
        Rule::PullRequest { parameters } => {
            assert_eq!(parameters.dismiss_stale_reviews_on_push, Some(true));
            assert_eq!(parameters.require_code_owner_review, Some(true));
            assert_eq!(parameters.require_last_push_approval, Some(false));
            assert_eq!(parameters.required_approving_review_count, Some(2));
            assert_eq!(parameters.required_review_thread_resolution, Some(true));
            assert_eq!(parameters.allowed_merge_methods.as_ref().unwrap().len(), 2);
        }
        _ => panic!("Expected PullRequest rule"),
    }
}

/// Test RequiredStatusChecks rule.
#[test]
fn test_required_status_checks_rule() {
    let json = r#"{
        "type": "required_status_checks",
        "parameters": {
            "required_status_checks": [
                {"context": "ci/test"},
                {"context": "ci/lint", "integration_id": 123}
            ],
            "strict_required_status_checks_policy": true
        }
    }"#;

    let rule: Rule = from_str(json).expect("Failed to deserialize");

    match rule {
        Rule::RequiredStatusChecks { parameters } => {
            assert_eq!(parameters.required_status_checks.len(), 2);
            assert_eq!(parameters.required_status_checks[0].context, "ci/test");
            assert_eq!(parameters.required_status_checks[0].integration_id, None);
            assert_eq!(parameters.required_status_checks[1].context, "ci/lint");
            assert_eq!(
                parameters.required_status_checks[1].integration_id,
                Some(123)
            );
            assert_eq!(parameters.strict_required_status_checks_policy, Some(true));
        }
        _ => panic!("Expected RequiredStatusChecks rule"),
    }
}

/// Test NonFastForward rule.
#[test]
fn test_non_fast_forward_rule() {
    let json = r#"{"type": "non_fast_forward"}"#;
    let rule: Rule = from_str(json).expect("Failed to deserialize");

    assert!(matches!(rule, Rule::NonFastForward));
}

// ============================================================================
// MergeMethod Tests
// ============================================================================

/// Test MergeMethod enum serialization.
#[test]
fn test_merge_method_serialization() {
    assert_eq!(to_string(&MergeMethod::Merge).unwrap(), "\"merge\"");
    assert_eq!(to_string(&MergeMethod::Squash).unwrap(), "\"squash\"");
    assert_eq!(to_string(&MergeMethod::Rebase).unwrap(), "\"rebase\"");
}

/// Test MergeMethod enum deserialization.
#[test]
fn test_merge_method_deserialization() {
    assert_eq!(
        from_str::<MergeMethod>("\"merge\"").unwrap(),
        MergeMethod::Merge
    );
    assert_eq!(
        from_str::<MergeMethod>("\"squash\"").unwrap(),
        MergeMethod::Squash
    );
    assert_eq!(
        from_str::<MergeMethod>("\"rebase\"").unwrap(),
        MergeMethod::Rebase
    );
}

// ============================================================================
// Complex Ruleset Tests
// ============================================================================

/// Test complete ruleset with all fields.
#[test]
fn test_complete_ruleset() {
    let json = r#"{
        "id": 999,
        "name": "comprehensive-protection",
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [
            {
                "actor_id": 100,
                "actor_type": "Team",
                "bypass_mode": "pull_request"
            }
        ],
        "conditions": {
            "ref_name": {
                "include": ["refs/heads/main"],
                "exclude": []
            }
        },
        "rules": [
            {"type": "deletion"},
            {"type": "required_linear_history"},
            {
                "type": "pull_request",
                "parameters": {
                    "required_approving_review_count": 3,
                    "require_code_owner_review": true,
                    "allowed_merge_methods": ["squash"]
                }
            }
        ]
    }"#;

    let ruleset: RepositoryRuleset = from_str(json).expect("Failed to deserialize");

    assert_eq!(ruleset.id, Some(999));
    assert_eq!(ruleset.name, "comprehensive-protection");
    assert_eq!(ruleset.bypass_actors.len(), 1);
    assert_eq!(ruleset.rules.len(), 3);

    // Verify rules
    assert!(matches!(ruleset.rules[0], Rule::Deletion));
    assert!(matches!(ruleset.rules[1], Rule::RequiredLinearHistory));

    match &ruleset.rules[2] {
        Rule::PullRequest { parameters } => {
            assert_eq!(parameters.required_approving_review_count, Some(3));
            assert_eq!(parameters.require_code_owner_review, Some(true));
            let methods = parameters.allowed_merge_methods.as_ref().unwrap();
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0], MergeMethod::Squash);
        }
        _ => panic!("Expected PullRequest rule"),
    }
}

/// Test ruleset serialization omits optional fields when None.
#[test]
fn test_ruleset_optional_fields_omitted() {
    let ruleset = RepositoryRuleset {
        id: None,
        name: "minimal".to_string(),
        target: RulesetTarget::Branch,
        enforcement: RulesetEnforcement::Active,
        bypass_actors: vec![],
        conditions: None,
        rules: vec![Rule::Creation],
        node_id: None,
        source: None,
        source_type: None,
        created_at: None,
        updated_at: None,
        _links: None,
    };

    let json = to_string(&ruleset).expect("Failed to serialize");

    // Optional fields should not be present
    assert!(!json.contains("\"id\""));
    assert!(!json.contains("\"conditions\""));
    assert!(!json.contains("\"node_id\""));
    assert!(!json.contains("\"_links\""));
}

/// Test RulesetTarget with Push variant.
#[test]
fn test_ruleset_target_push() {
    assert_eq!(to_string(&RulesetTarget::Push).unwrap(), "\"push\"");
    assert_eq!(
        from_str::<RulesetTarget>("\"push\"").unwrap(),
        RulesetTarget::Push
    );
}

/// Test BypassActorType with DeployKey variant.
#[test]
fn test_bypass_actor_type_deploy_key() {
    assert_eq!(
        to_string(&BypassActorType::DeployKey).unwrap(),
        "\"DeployKey\""
    );
    assert_eq!(
        from_str::<BypassActorType>("\"DeployKey\"").unwrap(),
        BypassActorType::DeployKey
    );
}

/// Test complete ruleset with push target and deploy key bypass.
#[test]
fn test_ruleset_with_push_target_and_deploy_key() {
    let json = r#"{
        "id": 456,
        "name": "push-protection",
        "target": "push",
        "enforcement": "active",
        "bypass_actors": [{
            "actor_id": 999,
            "actor_type": "DeployKey",
            "bypass_mode": "always"
        }],
        "rules": [{"type": "creation"}]
    }"#;

    let ruleset: RepositoryRuleset = from_str(json).expect("Failed to deserialize");

    assert_eq!(ruleset.id, Some(456));
    assert_eq!(ruleset.name, "push-protection");
    assert_eq!(ruleset.target, RulesetTarget::Push);
    assert_eq!(ruleset.enforcement, RulesetEnforcement::Active);
    assert_eq!(ruleset.bypass_actors.len(), 1);
    assert_eq!(ruleset.bypass_actors[0].actor_id, 999);
    assert_eq!(
        ruleset.bypass_actors[0].actor_type,
        BypassActorType::DeployKey
    );
    assert_eq!(ruleset.bypass_actors[0].bypass_mode, BypassMode::Always);
    assert_eq!(ruleset.rules.len(), 1);
}

// ============================================================================
// GitHub API Response Shape Tests
// ============================================================================

/// Test LIST rulesets response (no rules field).
///
/// GitHub's LIST endpoint returns minimal metadata without the rules array.
/// This caused deserialization failures until we added #[serde(default)].
#[test]
fn test_list_rulesets_response_without_rules() {
    // Real response shape from GitHub LIST /repos/{owner}/{repo}/rules
    let json = r#"{
        "id": 12555584,
        "name": "branch-protection",
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": []
    }"#;

    let ruleset: RepositoryRuleset = from_str(json).expect("Should handle missing rules field");

    assert_eq!(ruleset.id, Some(12555584));
    assert_eq!(ruleset.name, "branch-protection");
    assert_eq!(ruleset.target, RulesetTarget::Branch);
    assert_eq!(ruleset.enforcement, RulesetEnforcement::Active);
    assert!(ruleset.bypass_actors.is_empty());
    assert!(
        ruleset.rules.is_empty(),
        "Missing rules field should default to empty vec"
    );
}

/// Test GET ruleset response (with rules field).
///
/// GitHub's GET endpoint returns full details including rules.
#[test]
fn test_get_ruleset_response_with_rules() {
    // Real response shape from GitHub GET /repos/{owner}/{repo}/rules/{id}
    let json = r#"{
        "id": 12555584,
        "name": "branch-protection",
        "target": "branch",
        "enforcement": "active",
        "bypass_actors": [],
        "rules": [
            {"type": "creation"},
            {"type": "deletion"}
        ]
    }"#;

    let ruleset: RepositoryRuleset =
        from_str(json).expect("Should handle rules field when present");

    assert_eq!(ruleset.id, Some(12555584));
    assert_eq!(ruleset.rules.len(), 2);
    assert!(matches!(ruleset.rules[0], Rule::Creation));
    assert!(matches!(ruleset.rules[1], Rule::Deletion));
}

/// Test LIST response with conditions but no rules.
#[test]
fn test_list_response_with_conditions_no_rules() {
    let json = r#"{
        "id": 12555585,
        "name": "tag-protection",
        "target": "tag",
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {
            "ref_name": {
                "include": ["refs/tags/v*"],
                "exclude": []
            }
        }
    }"#;

    let ruleset: RepositoryRuleset =
        from_str(json).expect("Should handle conditions without rules");

    assert_eq!(ruleset.id, Some(12555585));
    assert!(ruleset.conditions.is_some());
    assert!(
        ruleset.rules.is_empty(),
        "Missing rules should default to empty"
    );
}
