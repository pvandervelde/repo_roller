//! Tests for ruleset configuration types.

use super::*;

// ============================================================================
// RulesetConfig Tests
// ============================================================================

/// Test minimal ruleset configuration deserialization.
#[test]
fn test_minimal_ruleset_config() {
    let toml = r#"
        name = "test-ruleset"
        rules = []
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.name, "test-ruleset");
    assert_eq!(config.target, "branch"); // default
    assert_eq!(config.enforcement, "active"); // default
    assert!(config.bypass_actors.is_empty());
    assert!(config.conditions.is_none());
    assert!(config.rules.is_empty());
}

/// Test complete ruleset configuration.
#[test]
fn test_complete_ruleset_config() {
    let toml = r#"
        name = "main-protection"
        target = "branch"
        enforcement = "active"

        [[bypass_actors]]
        actor_id = 123
        actor_type = "Team"
        bypass_mode = "pull_request"

        [conditions.ref_name]
        include = ["refs/heads/main", "refs/heads/release/*"]
        exclude = ["refs/heads/release/temp*"]

        [[rules]]
        type = "deletion"

        [[rules]]
        type = "required_linear_history"
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.name, "main-protection");
    assert_eq!(config.target, "branch");
    assert_eq!(config.enforcement, "active");
    assert_eq!(config.bypass_actors.len(), 1);
    assert!(config.conditions.is_some());
    assert_eq!(config.rules.len(), 2);
}

/// Test ruleset with default values.
#[test]
fn test_ruleset_defaults() {
    let toml = r#"
        name = "default-test"
        rules = []
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.target, "branch");
    assert_eq!(config.enforcement, "active");
}

/// Test ruleset with tag target.
#[test]
fn test_ruleset_tag_target() {
    let toml = r#"
        name = "tag-protection"
        target = "tag"
        rules = []
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.target, "tag");
}

/// Test ruleset with evaluate enforcement.
#[test]
fn test_ruleset_evaluate_enforcement() {
    let toml = r#"
        name = "evaluate-rules"
        enforcement = "evaluate"
        rules = []
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.enforcement, "evaluate");
}

// ============================================================================
// BypassActorConfig Tests
// ============================================================================

/// Test bypass actor configuration.
#[test]
fn test_bypass_actor_config() {
    let toml = r#"
        [[bypass_actors]]
        actor_id = 456
        actor_type = "OrganizationAdmin"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        bypass_actors: Vec<BypassActorConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.bypass_actors.len(), 1);
    assert_eq!(config.bypass_actors[0].actor_id, 456);
    assert_eq!(config.bypass_actors[0].actor_type, "OrganizationAdmin");
    assert_eq!(config.bypass_actors[0].bypass_mode, "always"); // default
}

/// Test bypass actor with pull_request mode.
#[test]
fn test_bypass_actor_pull_request_mode() {
    let toml = r#"
        [[bypass_actors]]
        actor_id = 789
        actor_type = "Team"
        bypass_mode = "pull_request"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        bypass_actors: Vec<BypassActorConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.bypass_actors[0].bypass_mode, "pull_request");
}

// ============================================================================
// RulesetConditionsConfig Tests
// ============================================================================

/// Test conditions with include and exclude patterns.
#[test]
fn test_conditions_with_patterns() {
    let toml = r#"
        [conditions.ref_name]
        include = ["refs/heads/main", "refs/heads/develop"]
        exclude = ["refs/heads/*/temp"]
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        conditions: RulesetConditionsConfig,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.conditions.ref_name.include.len(), 2);
    assert_eq!(config.conditions.ref_name.exclude.len(), 1);
    assert_eq!(config.conditions.ref_name.include[0], "refs/heads/main");
    assert_eq!(config.conditions.ref_name.exclude[0], "refs/heads/*/temp");
}

/// Test conditions with empty exclude (default).
#[test]
fn test_conditions_default_exclude() {
    let toml = r#"
        [conditions.ref_name]
        include = ["refs/heads/main"]
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        conditions: RulesetConditionsConfig,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(config.conditions.ref_name.exclude.is_empty());
}

// ============================================================================
// RuleConfig Tests
// ============================================================================

/// Test Creation rule.
#[test]
fn test_creation_rule_config() {
    let toml = r#"
        [[rules]]
        type = "creation"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.rules.len(), 1);
    assert!(matches!(config.rules[0], RuleConfig::Creation));
}

/// Test Update rule.
#[test]
fn test_update_rule_config() {
    let toml = r#"
        [[rules]]
        type = "update"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(matches!(config.rules[0], RuleConfig::Update));
}

/// Test Deletion rule.
#[test]
fn test_deletion_rule_config() {
    let toml = r#"
        [[rules]]
        type = "deletion"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(matches!(config.rules[0], RuleConfig::Deletion));
}

/// Test RequiredLinearHistory rule.
#[test]
fn test_required_linear_history_rule_config() {
    let toml = r#"
        [[rules]]
        type = "required_linear_history"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(matches!(config.rules[0], RuleConfig::RequiredLinearHistory));
}

/// Test RequiredSignatures rule.
#[test]
fn test_required_signatures_rule_config() {
    let toml = r#"
        [[rules]]
        type = "required_signatures"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(matches!(config.rules[0], RuleConfig::RequiredSignatures));
}

/// Test PullRequest rule with all parameters.
#[test]
fn test_pull_request_rule_config() {
    let toml = r#"
        [[rules]]
        type = "pull_request"
        dismiss_stale_reviews_on_push = true
        require_code_owner_review = true
        require_last_push_approval = false
        required_approving_review_count = 2
        required_review_thread_resolution = true
        allowed_merge_methods = ["squash", "rebase"]
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    match &config.rules[0] {
        RuleConfig::PullRequest {
            dismiss_stale_reviews_on_push,
            require_code_owner_review,
            require_last_push_approval,
            required_approving_review_count,
            required_review_thread_resolution,
            allowed_merge_methods,
        } => {
            assert_eq!(*dismiss_stale_reviews_on_push, Some(true));
            assert_eq!(*require_code_owner_review, Some(true));
            assert_eq!(*require_last_push_approval, Some(false));
            assert_eq!(*required_approving_review_count, Some(2));
            assert_eq!(*required_review_thread_resolution, Some(true));
            assert_eq!(allowed_merge_methods.as_ref().unwrap().len(), 2);
        }
        _ => panic!("Expected PullRequest rule"),
    }
}

/// Test PullRequest rule with minimal parameters.
#[test]
fn test_pull_request_rule_minimal() {
    let toml = r#"
        [[rules]]
        type = "pull_request"
        required_approving_review_count = 1
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    match &config.rules[0] {
        RuleConfig::PullRequest {
            required_approving_review_count,
            ..
        } => {
            assert_eq!(*required_approving_review_count, Some(1));
        }
        _ => panic!("Expected PullRequest rule"),
    }
}

/// Test RequiredStatusChecks rule.
#[test]
fn test_required_status_checks_rule_config() {
    let toml = r#"
        [[rules]]
        type = "required_status_checks"
        strict_required_status_checks_policy = true

        [[rules.required_status_checks]]
        context = "ci/test"

        [[rules.required_status_checks]]
        context = "ci/lint"
        integration_id = 123
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    match &config.rules[0] {
        RuleConfig::RequiredStatusChecks {
            required_status_checks,
            strict_required_status_checks_policy,
        } => {
            assert_eq!(required_status_checks.len(), 2);
            assert_eq!(required_status_checks[0].context, "ci/test");
            assert_eq!(required_status_checks[0].integration_id, None);
            assert_eq!(required_status_checks[1].context, "ci/lint");
            assert_eq!(required_status_checks[1].integration_id, Some(123));
            assert_eq!(*strict_required_status_checks_policy, Some(true));
        }
        _ => panic!("Expected RequiredStatusChecks rule"),
    }
}

/// Test NonFastForward rule.
#[test]
fn test_non_fast_forward_rule_config() {
    let toml = r#"
        [[rules]]
        type = "non_fast_forward"
    "#;

    #[derive(Deserialize)]
    struct TestConfig {
        rules: Vec<RuleConfig>,
    }

    let config: TestConfig = toml::from_str(toml).expect("Failed to parse");

    assert!(matches!(config.rules[0], RuleConfig::NonFastForward));
}

// ============================================================================
// Multiple Rules Tests
// ============================================================================

/// Test ruleset with multiple different rule types.
#[test]
fn test_multiple_rules() {
    let toml = r#"
        name = "multi-rule-test"

        [[rules]]
        type = "deletion"

        [[rules]]
        type = "required_linear_history"

        [[rules]]
        type = "pull_request"
        required_approving_review_count = 2
        allowed_merge_methods = ["squash"]

        [[rules]]
        type = "required_status_checks"
        [[rules.required_status_checks]]
        context = "ci/test"
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.rules.len(), 4);
    assert!(matches!(config.rules[0], RuleConfig::Deletion));
    assert!(matches!(config.rules[1], RuleConfig::RequiredLinearHistory));
    assert!(matches!(config.rules[2], RuleConfig::PullRequest { .. }));
    assert!(matches!(
        config.rules[3],
        RuleConfig::RequiredStatusChecks { .. }
    ));
}

/// Test ruleset with push target.
#[test]
fn test_push_target_ruleset() {
    let toml = r#"
        name = "push-rules"
        target = "push"
        rules = []
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.name, "push-rules");
    assert_eq!(config.target, "push");

    // Test conversion to domain type
    let domain = config.to_domain_ruleset();
    assert_eq!(domain.target, github_client::RulesetTarget::Push);
}

/// Test bypass actor with DeployKey type.
#[test]
fn test_deploy_key_bypass_actor() {
    let toml = r#"
        name = "with-deploy-key"
        rules = []

        [[bypass_actors]]
        actor_id = 789
        actor_type = "DeployKey"
    "#;

    let config: RulesetConfig = toml::from_str(toml).expect("Failed to parse");

    assert_eq!(config.bypass_actors.len(), 1);
    assert_eq!(config.bypass_actors[0].actor_id, 789);
    assert_eq!(config.bypass_actors[0].actor_type, "DeployKey");
    assert_eq!(config.bypass_actors[0].bypass_mode, "always"); // default

    // Test conversion to domain type
    let domain = config.to_domain_ruleset();
    assert_eq!(domain.bypass_actors.len(), 1);
    assert_eq!(
        domain.bypass_actors[0].actor_type,
        github_client::BypassActorType::DeployKey
    );
}

/// Test all target types convert correctly.
#[test]
fn test_target_conversion() {
    let targets = vec![
        ("branch", github_client::RulesetTarget::Branch),
        ("tag", github_client::RulesetTarget::Tag),
        ("push", github_client::RulesetTarget::Push),
    ];

    for (config_target, expected_domain) in targets {
        let toml = format!(
            r#"
            name = "test"
            target = "{}"
            rules = []
            "#,
            config_target
        );

        let config: RulesetConfig = toml::from_str(&toml).expect("Failed to parse");
        let domain = config.to_domain_ruleset();

        assert_eq!(domain.target, expected_domain);
    }
}

/// Test all bypass actor types convert correctly.
#[test]
fn test_bypass_actor_type_conversion() {
    use github_client::BypassActorType;

    let actor_types = vec![
        ("OrganizationAdmin", BypassActorType::OrganizationAdmin),
        ("RepositoryRole", BypassActorType::RepositoryRole),
        ("Team", BypassActorType::Team),
        ("Integration", BypassActorType::Integration),
        ("DeployKey", BypassActorType::DeployKey),
    ];

    for (config_type, expected_domain) in actor_types {
        let toml = format!(
            r#"
            name = "test"
            rules = []

            [[bypass_actors]]
            actor_id = 1
            actor_type = "{}"
            "#,
            config_type
        );

        let config: RulesetConfig = toml::from_str(&toml).expect("Failed to parse");
        let domain = config.to_domain_ruleset();

        assert_eq!(domain.bypass_actors.len(), 1);
        assert_eq!(domain.bypass_actors[0].actor_type, expected_domain);
    }
}
