//! Naming rules integration tests.
//!
//! These tests verify the full naming rules pipeline:
//! configuration hierarchy → merger → `MergedConfiguration.naming_rules` →
//! `RepositoryNamingValidator`.
//!
//! The tests do **not** require GitHub credentials because naming validation
//! occurs before any GitHub API call.  They exercise the same code path that
//! runs inside `create_repository()` at Step 4b.

use anyhow::Result;
use config_manager::{
    ConfigurationMerger, GlobalDefaults, RepositoryNamingRulesConfig, RepositoryTypeConfig,
    TeamConfig, TemplateConfig, TemplateMetadata,
};
use repo_roller_core::RepositoryNamingValidator;
use tracing::info;

// ============================================================================
// Helpers
// ============================================================================

/// Build the smallest valid `TemplateConfig` with no naming rules.
fn minimal_template(name: &str) -> TemplateConfig {
    TemplateConfig {
        template: TemplateMetadata {
            name: name.to_string(),
            description: "Integration test template".to_string(),
            author: "test".to_string(),
            tags: vec![],
        },
        repository: None,
        repository_type: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        variables: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
    }
}

/// Convenience: merge global + template (no type / team) and run naming validation.
fn validate(
    global: &GlobalDefaults,
    template: &TemplateConfig,
    repo_name: &str,
) -> config_manager::ConfigurationResult<Result<(), repo_roller_core::ValidationError>> {
    let merged = ConfigurationMerger::new().merge_configurations(global, None, None, template)?;
    let result = RepositoryNamingValidator::new().validate(repo_name, &merged.naming_rules);
    Ok(result)
}

// ============================================================================
// Empty rules
// ============================================================================

/// No naming rules anywhere → every valid GitHub repo name is accepted.
#[tokio::test]
async fn test_no_naming_rules_accepts_any_name() -> Result<()> {
    info!("Testing that absent naming rules accept any name");

    let global = GlobalDefaults::default();
    let template = minimal_template("no-rules");

    let names = [
        "my-service",
        "acme-payments-svc",
        "x",
        "UPPERCASE-IS-OK-123",
    ];
    for name in names {
        let result = validate(&global, &template, name).expect("merge should succeed");
        assert!(
            result.is_ok(),
            "Name '{name}' should be accepted when no rules are set"
        );
    }

    info!("✓ No-rules pass-through test passed");
    Ok(())
}

// ============================================================================
// Rules from GlobalDefaults are enforced
// ============================================================================

/// A prefix rule in GlobalDefaults is enforced against every repository name.
#[tokio::test]
async fn test_global_prefix_rule_is_enforced() -> Result<()> {
    info!("Testing global-level required-prefix rule enforcement");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("All repos must start with 'acme-'".to_string()),
            required_prefix: Some("acme-".to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };
    let template = minimal_template("t");

    // Valid name
    let ok = validate(&global, &template, "acme-payments").expect("merge ok");
    assert!(ok.is_ok(), "Name with correct prefix should pass");

    // Invalid name
    let err = validate(&global, &template, "payments").expect("merge ok");
    assert!(err.is_err(), "Name without prefix should fail");
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("must start with 'acme-'"),
        "Error should describe the prefix violation: {msg}"
    );

    info!("✓ Global prefix rule enforcement test passed");
    Ok(())
}

/// A max-length rule in GlobalDefaults is enforced.
#[tokio::test]
async fn test_global_max_length_rule_is_enforced() -> Result<()> {
    info!("Testing global-level max-length rule enforcement");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Max 20 chars".to_string()),
            max_length: Some(20),
            ..Default::default()
        }]),
        ..Default::default()
    };
    let template = minimal_template("t");

    assert!(validate(&global, &template, "short-name")
        .expect("merge ok")
        .is_ok());

    let long_name = "a-very-long-repository-name";
    let err = validate(&global, &template, long_name).expect("merge ok");
    assert!(err.is_err(), "Long name should fail");
    assert!(
        err.unwrap_err().to_string().contains("too long"),
        "Error should mention 'too long'"
    );

    info!("✓ Global max-length rule enforcement test passed");
    Ok(())
}

// ============================================================================
// Rules from TemplateConfig are enforced
// ============================================================================

/// A reserved-words rule in TemplateConfig is enforced.
#[tokio::test]
async fn test_template_reserved_word_rule_is_enforced() -> Result<()> {
    info!("Testing template-level reserved-words rule enforcement");

    let global = GlobalDefaults::default();
    let template = TemplateConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("'demo' is reserved".to_string()),
            reserved_words: vec!["demo".to_string()],
            ..Default::default()
        }]),
        ..minimal_template("t")
    };

    assert!(validate(&global, &template, "my-service")
        .expect("merge ok")
        .is_ok());

    let err = validate(&global, &template, "demo").expect("merge ok");
    assert!(err.is_err(), "Reserved word 'demo' should be rejected");
    assert!(
        err.unwrap_err().to_string().contains("reserved word"),
        "Error should mention reserved word"
    );

    // Case-insensitive
    let err2 = validate(&global, &template, "DEMO").expect("merge ok");
    assert!(
        err2.is_err(),
        "Case-insensitive reserved word 'DEMO' should be rejected"
    );

    info!("✓ Template reserved-word rule enforcement test passed");
    Ok(())
}

/// A forbidden-pattern rule in TemplateConfig is enforced.
#[tokio::test]
async fn test_template_forbidden_pattern_rule_is_enforced() -> Result<()> {
    info!("Testing template-level forbidden-pattern rule enforcement");

    let global = GlobalDefaults::default();
    let template = TemplateConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Double hyphens are forbidden".to_string()),
            forbidden_patterns: vec![".*--.*".to_string()],
            ..Default::default()
        }]),
        ..minimal_template("t")
    };

    assert!(validate(&global, &template, "my-service")
        .expect("merge ok")
        .is_ok());

    let err = validate(&global, &template, "my--service").expect("merge ok");
    assert!(err.is_err(), "Double hyphen name should fail");
    assert!(
        err.unwrap_err()
            .to_string()
            .contains("matches forbidden pattern"),
        "Error should mention forbidden pattern"
    );

    info!("✓ Template forbidden-pattern rule enforcement test passed");
    Ok(())
}

// ============================================================================
// Additive merger semantics (rules from all levels are accumulated)
// ============================================================================

/// Rules from GlobalDefaults AND TemplateConfig are both applied — a name
/// must satisfy all of them.
#[tokio::test]
async fn test_global_and_template_rules_are_both_enforced() -> Result<()> {
    info!("Testing additive merger: global and template rules are both enforced");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Must start with 'acme-'".to_string()),
            required_prefix: Some("acme-".to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let template = TemplateConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Must end with '-svc'".to_string()),
            required_suffix: Some("-svc".to_string()),
            ..Default::default()
        }]),
        ..minimal_template("t")
    };

    // Satisfies both rules
    assert!(validate(&global, &template, "acme-payments-svc")
        .expect("merge ok")
        .is_ok());

    // Fails global rule (prefix)
    let err1 = validate(&global, &template, "payments-svc").expect("merge ok");
    assert!(err1.is_err(), "Missing prefix should fail");

    // Satisfies global rule but fails template rule (suffix)
    let err2 = validate(&global, &template, "acme-payments").expect("merge ok");
    assert!(err2.is_err(), "Missing suffix should fail");

    info!("✓ Additive rule enforcement test passed");
    Ok(())
}

/// Rules from all four hierarchy levels (global, repository type, team, template)
/// accumulate additively in the merged configuration.
#[tokio::test]
async fn test_all_config_levels_contribute_naming_rules() -> Result<()> {
    info!("Testing that all four config hierarchy levels contribute naming rules");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Min length 5".to_string()),
            min_length: Some(5),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let repo_type = RepositoryTypeConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Max length 30".to_string()),
            max_length: Some(30),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let team = TeamConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Must start with 'svc-'".to_string()),
            required_prefix: Some("svc-".to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let template = TemplateConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Must end with '-v1'".to_string()),
            required_suffix: Some("-v1".to_string()),
            ..Default::default()
        }]),
        ..minimal_template("t")
    };

    let merged = ConfigurationMerger::new()
        .merge_configurations(&global, Some(&repo_type), Some(&team), &template)
        .expect("merge should succeed");

    // All 4 rules should be present
    assert_eq!(
        merged.naming_rules.len(),
        4,
        "All four hierarchy levels should have contributed one rule each"
    );

    let validator = RepositoryNamingValidator::new();

    // Satisfies all rules: length 5-30, starts with "svc-", ends with "-v1"
    assert!(
        validator
            .validate("svc-auth-v1", &merged.naming_rules)
            .is_ok(),
        "Name satisfying all rules should pass"
    );

    // Fails min_length (global)
    assert!(
        validator.validate("s-v1", &merged.naming_rules).is_err(),
        "Too-short name should fail global rule"
    );

    // Fails prefix (team)
    assert!(
        validator
            .validate("auth-service-v1", &merged.naming_rules)
            .is_err(),
        "Name without 'svc-' prefix should fail team rule"
    );

    // Fails suffix (template)
    assert!(
        validator
            .validate("svc-auth", &merged.naming_rules)
            .is_err(),
        "Name without '-v1' suffix should fail template rule"
    );

    info!("✓ All-levels rule accumulation test passed");
    Ok(())
}

// ============================================================================
// Allowed pattern (regex)
// ============================================================================

/// An allowed-pattern rule from the global config is enforced.
#[tokio::test]
async fn test_allowed_pattern_from_config_hierarchy() -> Result<()> {
    info!("Testing allowed-pattern from config hierarchy");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Only lowercase letters and hyphens".to_string()),
            allowed_pattern: Some(r"[a-z][a-z0-9-]*".to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };
    let template = minimal_template("t");

    assert!(validate(&global, &template, "my-service-123")
        .expect("merge ok")
        .is_ok());

    let err = validate(&global, &template, "My_Service").expect("merge ok");
    assert!(err.is_err(), "Name with uppercase/underscore should fail");
    assert!(
        err.unwrap_err()
            .to_string()
            .contains("does not match required pattern"),
        "Error should mention pattern mismatch"
    );

    info!("✓ Allowed pattern from config hierarchy test passed");
    Ok(())
}

// ============================================================================
// Multiple rules: first violation is reported
// ============================================================================

/// When multiple rules are violated, the first one (in accumulation order)
/// is reported.
#[tokio::test]
async fn test_first_violated_rule_is_reported() -> Result<()> {
    info!("Testing that the first violated rule is reported");

    let global = GlobalDefaults {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Global: min length 5".to_string()),
            min_length: Some(5),
            ..Default::default()
        }]),
        ..Default::default()
    };

    let template = TemplateConfig {
        naming_rules: Some(vec![RepositoryNamingRulesConfig {
            description: Some("Template: prefix required".to_string()),
            required_prefix: Some("acme-".to_string()),
            ..Default::default()
        }]),
        ..minimal_template("t")
    };

    // "ab" violates both rules.  The global rule appears first in the
    // accumulated list, so its error should be reported.
    let err = validate(&global, &template, "ab").expect("merge ok");
    assert!(err.is_err());
    assert!(
        err.unwrap_err().to_string().contains("too short"),
        "First rule (min_length from global) should be reported"
    );

    info!("✓ First-violation ordering test passed");
    Ok(())
}
