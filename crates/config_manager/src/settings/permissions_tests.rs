//! Tests for config_manager settings::permissions types.

use super::*;

// ─── PermissionGrantConfig deserialization ───────────────────────────────────

#[test]
fn test_grant_deserializes_valid_push_write_team() {
    let toml = r#"
        permission_type = "push"
        level = "write"
        scope = "team"
    "#;

    let grant: PermissionGrantConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(grant.permission_type, "push");
    assert_eq!(grant.level, "write");
    assert_eq!(grant.scope, "team");
}

#[test]
fn test_grant_deserializes_all_valid_permission_types() {
    for pt in ["pull", "triage", "push", "maintain", "admin"] {
        let toml = format!("permission_type = \"{pt}\"\nlevel = \"read\"\nscope = \"repository\"");
        let grant: PermissionGrantConfig = toml::from_str(&toml).expect("valid TOML");
        assert_eq!(grant.permission_type, pt);
    }
}

#[test]
fn test_grant_deserializes_all_valid_levels() {
    for level in ["none", "read", "triage", "write", "maintain", "admin"] {
        let toml =
            format!("permission_type = \"push\"\nlevel = \"{level}\"\nscope = \"repository\"");
        let grant: PermissionGrantConfig = toml::from_str(&toml).expect("valid TOML");
        assert_eq!(grant.level, level);
    }
}

#[test]
fn test_grant_deserializes_all_valid_scopes() {
    for scope in ["repository", "team", "user", "github_app"] {
        let toml = format!("permission_type = \"push\"\nlevel = \"write\"\nscope = \"{scope}\"");
        let grant: PermissionGrantConfig = toml::from_str(&toml).expect("valid TOML");
        assert_eq!(grant.scope, scope);
    }
}

// ─── PermissionGrantConfig validation ───────────────────────────────────────

#[test]
fn test_grant_validate_ok_for_valid_grant() {
    let grant = PermissionGrantConfig {
        permission_type: "push".to_string(),
        level: "write".to_string(),
        scope: "team".to_string(),
    };
    assert!(grant.validate().is_ok());
}

#[test]
fn test_grant_validate_error_for_unknown_permission_type() {
    let grant = PermissionGrantConfig {
        permission_type: "deploy".to_string(),
        level: "write".to_string(),
        scope: "team".to_string(),
    };
    let err = grant.validate().unwrap_err();
    assert!(
        matches!(err, PermissionConfigError::InvalidPermissionType(_)),
        "Expected InvalidPermissionType, got: {err}"
    );
}

#[test]
fn test_grant_validate_error_for_unknown_level() {
    let grant = PermissionGrantConfig {
        permission_type: "push".to_string(),
        level: "superadmin".to_string(),
        scope: "team".to_string(),
    };
    let err = grant.validate().unwrap_err();
    assert!(
        matches!(err, PermissionConfigError::InvalidLevel(_)),
        "Expected InvalidLevel, got: {err}"
    );
}

#[test]
fn test_grant_validate_error_for_unknown_scope() {
    let grant = PermissionGrantConfig {
        permission_type: "push".to_string(),
        level: "write".to_string(),
        scope: "organization".to_string(),
    };
    let err = grant.validate().unwrap_err();
    assert!(
        matches!(err, PermissionConfigError::InvalidScope(_)),
        "Expected InvalidScope, got: {err}"
    );
}

#[test]
fn test_grant_validate_error_for_empty_permission_type() {
    let grant = PermissionGrantConfig {
        permission_type: "".to_string(),
        level: "write".to_string(),
        scope: "team".to_string(),
    };
    assert!(grant.validate().is_err());
}

#[test]
fn test_grant_validate_error_for_mixed_case_permission_type() {
    // Validation must be case-sensitive (lowercase only).
    let grant = PermissionGrantConfig {
        permission_type: "Push".to_string(),
        level: "write".to_string(),
        scope: "team".to_string(),
    };
    assert!(grant.validate().is_err());
}

// ─── OrganizationPermissionPoliciesConfig ────────────────────────────────────

#[test]
fn test_org_policies_deserializes_with_baseline_and_restrictions() {
    let toml = r#"
        [[baseline]]
        permission_type = "pull"
        level = "read"
        scope = "repository"

        [[restrictions]]
        permission_type = "admin"
        level = "maintain"
        scope = "user"
    "#;

    let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).expect("valid TOML");
    let baseline = config.baseline.expect("baseline present");
    assert_eq!(baseline.len(), 1);
    assert_eq!(baseline[0].permission_type, "pull");

    let restrictions = config.restrictions.expect("restrictions present");
    assert_eq!(restrictions.len(), 1);
    assert_eq!(restrictions[0].permission_type, "admin");
}

#[test]
fn test_org_policies_deserializes_with_empty_config() {
    let toml = "";
    let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).expect("valid TOML");
    assert!(config.baseline.is_none());
    assert!(config.restrictions.is_none());
}

#[test]
fn test_org_policies_default_is_empty() {
    let config = OrganizationPermissionPoliciesConfig::default();
    assert!(config.baseline.is_none());
    assert!(config.restrictions.is_none());
}

#[test]
fn test_org_policies_validate_ok_when_all_grants_are_valid() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: Some(vec![PermissionGrantConfig {
            permission_type: "pull".to_string(),
            level: "read".to_string(),
            scope: "repository".to_string(),
        }]),
        restrictions: Some(vec![PermissionGrantConfig {
            permission_type: "admin".to_string(),
            level: "admin".to_string(),
            scope: "user".to_string(),
        }]),
        max_team_access_level: None,
        max_collaborator_access_level: None,
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_org_policies_validate_error_on_invalid_baseline_grant() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: Some(vec![PermissionGrantConfig {
            permission_type: "invalid_type".to_string(),
            level: "read".to_string(),
            scope: "repository".to_string(),
        }]),
        restrictions: None,
        max_team_access_level: None,
        max_collaborator_access_level: None,
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_org_policies_validate_error_on_invalid_restriction_grant() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: None,
        restrictions: Some(vec![PermissionGrantConfig {
            permission_type: "admin".to_string(),
            level: "ultramax".to_string(),
            scope: "user".to_string(),
        }]),
        max_team_access_level: None,
        max_collaborator_access_level: None,
    };
    assert!(config.validate().is_err());
}

// ─── RepositoryTypePermissionsConfig ─────────────────────────────────────────

#[test]
fn test_repo_type_perms_deserializes_with_required_and_restricted() {
    let toml = r#"
        restricted_types = ["admin"]

        [[required]]
        permission_type = "push"
        level = "write"
        scope = "repository"
    "#;

    let config: RepositoryTypePermissionsConfig = toml::from_str(toml).expect("valid TOML");
    let required = config.required.expect("required present");
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].permission_type, "push");

    let restricted = config.restricted_types.expect("restricted_types present");
    assert_eq!(restricted, vec!["admin"]);
}

#[test]
fn test_repo_type_perms_deserializes_empty() {
    let config: RepositoryTypePermissionsConfig = toml::from_str("").expect("valid TOML");
    assert!(config.required.is_none());
    assert!(config.restricted_types.is_none());
}

#[test]
fn test_repo_type_perms_validate_ok_for_valid_config() {
    let config = RepositoryTypePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "push".to_string(),
            level: "write".to_string(),
            scope: "repository".to_string(),
        }]),
        restricted_types: Some(vec!["admin".to_string()]),
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_repo_type_perms_validate_error_for_invalid_required_grant() {
    let config = RepositoryTypePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "write".to_string(), // "write" is a level, not a type
            level: "write".to_string(),
            scope: "repository".to_string(),
        }]),
        restricted_types: None,
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_repo_type_perms_validate_error_for_invalid_restricted_type() {
    let config = RepositoryTypePermissionsConfig {
        required: None,
        restricted_types: Some(vec!["deploy".to_string()]),
    };
    assert!(config.validate().is_err());
}

// ─── TemplatePermissionsConfig ────────────────────────────────────────────────

#[test]
fn test_template_perms_deserializes_with_required() {
    let toml = r#"
        [[required]]
        permission_type = "push"
        level = "write"
        scope = "team"
    "#;

    let config: TemplatePermissionsConfig = toml::from_str(toml).expect("valid TOML");
    let required = config.required.expect("required present");
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].scope, "team");
}

#[test]
fn test_template_perms_deserializes_empty() {
    let config: TemplatePermissionsConfig = toml::from_str("").expect("valid TOML");
    assert!(config.required.is_none());
}

#[test]
fn test_template_perms_validate_ok_for_valid_config() {
    let config = TemplatePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "push".to_string(),
            level: "write".to_string(),
            scope: "team".to_string(),
        }]),
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_template_perms_validate_error_for_invalid_grant() {
    let config = TemplatePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "push".to_string(),
            level: "write".to_string(),
            scope: "org".to_string(), // invalid
        }]),
    };
    assert!(config.validate().is_err());
}

// ─── DefaultTeamConfig ───────────────────────────────────────────────────────

#[test]
fn test_default_team_deserializes_from_toml() {
    let toml = r#"
        slug = "platform-team"
        access_level = "write"
    "#;
    let team: DefaultTeamConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(team.slug, "platform-team");
    assert_eq!(team.access_level, "write");
}

#[test]
fn test_default_team_validates_all_valid_levels() {
    for level in ["read", "triage", "write", "maintain", "admin", "none"] {
        let team = DefaultTeamConfig {
            slug: "some-team".to_string(),
            access_level: level.to_string(),
            locked: false,
        };
        assert!(team.validate().is_ok(), "level '{level}' should be valid");
    }
}

#[test]
fn test_default_team_validate_error_for_invalid_level() {
    let team = DefaultTeamConfig {
        slug: "some-team".to_string(),
        access_level: "superadmin".to_string(),
        locked: false,
    };
    assert!(matches!(
        team.validate(),
        Err(PermissionConfigError::InvalidLevel(_))
    ));
}

#[test]
fn test_default_team_validate_error_for_empty_slug() {
    let team = DefaultTeamConfig {
        slug: "".to_string(),
        access_level: "write".to_string(),
        locked: false,
    };
    assert!(matches!(
        team.validate(),
        Err(PermissionConfigError::EmptyIdentifier(_))
    ));
}

#[test]
fn test_default_team_validate_error_for_blank_slug() {
    let team = DefaultTeamConfig {
        slug: "   ".to_string(),
        access_level: "write".to_string(),
        locked: false,
    };
    assert!(matches!(
        team.validate(),
        Err(PermissionConfigError::EmptyIdentifier(_))
    ));
}

#[test]
fn test_default_team_deserialized_in_array() {
    let toml = r#"
        [[teams]]
        slug = "security-team"
        access_level = "triage"

        [[teams]]
        slug = "ops-team"
        access_level = "maintain"
    "#;
    #[derive(serde::Deserialize)]
    struct Wrapper {
        teams: Vec<DefaultTeamConfig>,
    }
    let wrapper: Wrapper = toml::from_str(toml).expect("valid TOML");
    assert_eq!(wrapper.teams.len(), 2);
    assert_eq!(wrapper.teams[0].slug, "security-team");
    assert_eq!(wrapper.teams[1].access_level, "maintain");
}

// ─── DefaultCollaboratorConfig ───────────────────────────────────────────────

#[test]
fn test_default_collaborator_deserializes_from_toml() {
    let toml = r#"
        username = "monitoring-bot"
        access_level = "read"
    "#;
    let collab: DefaultCollaboratorConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(collab.username, "monitoring-bot");
    assert_eq!(collab.access_level, "read");
}

#[test]
fn test_default_collaborator_validates_all_valid_levels() {
    for level in ["read", "triage", "write", "maintain", "admin", "none"] {
        let collab = DefaultCollaboratorConfig {
            username: "some-user".to_string(),
            access_level: level.to_string(),
            locked: false,
        };
        assert!(collab.validate().is_ok(), "level '{level}' should be valid");
    }
}

#[test]
fn test_default_collaborator_validate_error_for_invalid_level() {
    let collab = DefaultCollaboratorConfig {
        username: "some-user".to_string(),
        access_level: "viewer".to_string(),
        locked: false,
    };
    assert!(matches!(
        collab.validate(),
        Err(PermissionConfigError::InvalidLevel(_))
    ));
}

#[test]
fn test_default_collaborator_validate_error_for_empty_username() {
    let collab = DefaultCollaboratorConfig {
        username: "".to_string(),
        access_level: "read".to_string(),
        locked: false,
    };
    assert!(matches!(
        collab.validate(),
        Err(PermissionConfigError::EmptyIdentifier(_))
    ));
}

// ─── DefaultTeamConfig locked field ─────────────────────────────────────────

#[test]
fn test_default_team_locked_defaults_to_false_when_absent() {
    let toml = r#"
        slug = "platform-team"
        access_level = "write"
    "#;
    let team: DefaultTeamConfig = toml::from_str(toml).expect("valid TOML");
    assert!(!team.locked, "locked should default to false");
}

#[test]
fn test_default_team_locked_true_deserializes() {
    let toml = r#"
        slug = "security-team"
        access_level = "triage"
        locked = true
    "#;
    let team: DefaultTeamConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(team.slug, "security-team");
    assert!(team.locked);
}

#[test]
fn test_default_team_locked_false_deserializes() {
    let toml = r#"
        slug = "dev-team"
        access_level = "write"
        locked = false
    "#;
    let team: DefaultTeamConfig = toml::from_str(toml).expect("valid TOML");
    assert!(!team.locked);
}

#[test]
fn test_default_team_locked_round_trips() {
    let original = DefaultTeamConfig {
        slug: "ops-team".to_string(),
        access_level: "maintain".to_string(),
        locked: true,
    };
    let serialized = toml::to_string(&original).expect("serialization failed");
    let deserialized: DefaultTeamConfig =
        toml::from_str(&serialized).expect("deserialization failed");
    assert_eq!(original, deserialized);
}

#[test]
fn test_default_team_validate_ignores_locked_field() {
    // validate() should pass regardless of locked value
    for locked in [true, false] {
        let team = DefaultTeamConfig {
            slug: "some-team".to_string(),
            access_level: "write".to_string(),
            locked,
        };
        assert!(
            team.validate().is_ok(),
            "validate should pass when locked={}",
            locked
        );
    }
}

// ─── DefaultCollaboratorConfig locked field ──────────────────────────────────

#[test]
fn test_default_collaborator_locked_defaults_to_false_when_absent() {
    let toml = r#"
        username = "bot-user"
        access_level = "read"
    "#;
    let collab: DefaultCollaboratorConfig = toml::from_str(toml).expect("valid TOML");
    assert!(!collab.locked, "locked should default to false");
}

#[test]
fn test_default_collaborator_locked_true_deserializes() {
    let toml = r#"
        username = "security-bot"
        access_level = "read"
        locked = true
    "#;
    let collab: DefaultCollaboratorConfig = toml::from_str(toml).expect("valid TOML");
    assert!(collab.locked);
}

#[test]
fn test_default_collaborator_locked_round_trips() {
    let original = DefaultCollaboratorConfig {
        username: "audit-bot".to_string(),
        access_level: "triage".to_string(),
        locked: true,
    };
    let serialized = toml::to_string(&original).expect("serialization failed");
    let deserialized: DefaultCollaboratorConfig =
        toml::from_str(&serialized).expect("deserialization failed");
    assert_eq!(original, deserialized);
}

// ─── OrganizationPermissionPoliciesConfig – max access levels ────────────────

#[test]
fn test_org_perm_config_max_team_access_level_deserializes() {
    let toml = r#"
        max_team_access_level = "maintain"
    "#;
    let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(config.max_team_access_level.as_deref(), Some("maintain"));
}

#[test]
fn test_org_perm_config_max_collaborator_access_level_deserializes() {
    let toml = r#"
        max_collaborator_access_level = "write"
    "#;
    let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(
        config.max_collaborator_access_level.as_deref(),
        Some("write")
    );
}

#[test]
fn test_org_perm_config_max_levels_absent_defaults_to_none() {
    let toml = r#""#;
    let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).expect("valid TOML");
    assert!(config.max_team_access_level.is_none());
    assert!(config.max_collaborator_access_level.is_none());
}

#[test]
fn test_org_perm_config_validate_valid_max_levels() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: None,
        restrictions: None,
        max_team_access_level: Some("maintain".to_string()),
        max_collaborator_access_level: Some("write".to_string()),
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_org_perm_config_validate_invalid_max_team_level() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: None,
        restrictions: None,
        max_team_access_level: Some("superuser".to_string()),
        max_collaborator_access_level: None,
    };
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, PermissionConfigError::InvalidLevel(ref s) if s == "superuser"),
        "Expected InvalidLevel(\"superuser\"), got: {err}"
    );
}

#[test]
fn test_org_perm_config_validate_invalid_max_collab_level() {
    let config = OrganizationPermissionPoliciesConfig {
        baseline: None,
        restrictions: None,
        max_team_access_level: None,
        max_collaborator_access_level: Some("owner".to_string()),
    };
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, PermissionConfigError::InvalidLevel(ref s) if s == "owner"),
        "Expected InvalidLevel(\"owner\"), got: {err}"
    );
}

// ─── access_level_order ──────────────────────────────────────────────────────

#[test]
fn test_access_level_order_returns_none_for_unknown() {
    assert_eq!(access_level_order("superuser"), None);
    assert_eq!(access_level_order(""), None);
    assert_eq!(access_level_order("WRITE"), None); // case-sensitive
}

#[test]
fn test_access_level_order_all_valid_levels_ordered() {
    let levels = ["none", "read", "triage", "write", "maintain", "admin"];
    let orders: Vec<u8> = levels
        .iter()
        .map(|l| access_level_order(l).unwrap())
        .collect();
    for window in orders.windows(2) {
        assert!(
            window[0] < window[1],
            "Expected strict ordering but got {:?}",
            orders
        );
    }
}

#[test]
fn test_access_level_order_admin_is_highest() {
    assert_eq!(access_level_order("admin"), Some(5));
}

#[test]
fn test_access_level_order_none_is_lowest() {
    assert_eq!(access_level_order("none"), Some(0));
}

#[test]
fn test_access_level_order_comparisons() {
    assert!(access_level_order("write") > access_level_order("read"));
    assert!(access_level_order("maintain") > access_level_order("triage"));
    assert!(access_level_order("admin") > access_level_order("maintain"));
}
