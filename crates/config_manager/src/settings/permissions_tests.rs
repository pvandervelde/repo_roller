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
