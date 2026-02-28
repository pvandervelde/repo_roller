use super::*;
use chrono::Duration;

// ── AccessLevel ordering ───────────────────────────────────────────────────────

#[test]
fn access_level_admin_is_greater_than_write() {
    assert!(AccessLevel::Admin > AccessLevel::Write);
}

#[test]
fn access_level_admin_is_greater_than_all_others() {
    for level in [
        AccessLevel::None,
        AccessLevel::Read,
        AccessLevel::Triage,
        AccessLevel::Write,
        AccessLevel::Maintain,
    ] {
        assert!(
            AccessLevel::Admin > level,
            "Admin should be greater than {level:?}"
        );
    }
}

#[test]
fn access_level_none_is_less_than_read() {
    assert!(AccessLevel::None < AccessLevel::Read);
}

#[test]
fn access_level_none_is_less_than_all_others() {
    for level in [
        AccessLevel::Read,
        AccessLevel::Triage,
        AccessLevel::Write,
        AccessLevel::Maintain,
        AccessLevel::Admin,
    ] {
        assert!(
            AccessLevel::None < level,
            "None should be less than {level:?}"
        );
    }
}

#[test]
fn access_level_ordering_is_none_read_triage_write_maintain_admin() {
    let mut levels = vec![
        AccessLevel::Admin,
        AccessLevel::Write,
        AccessLevel::None,
        AccessLevel::Maintain,
        AccessLevel::Read,
        AccessLevel::Triage,
    ];
    levels.sort();
    assert_eq!(
        levels,
        vec![
            AccessLevel::None,
            AccessLevel::Read,
            AccessLevel::Triage,
            AccessLevel::Write,
            AccessLevel::Maintain,
            AccessLevel::Admin,
        ]
    );
}

// ── AccessLevel serde ─────────────────────────────────────────────────────────

#[test]
fn access_level_serializes_to_snake_case_string() {
    assert_eq!(
        serde_json::to_string(&AccessLevel::Read).unwrap(),
        "\"read\""
    );
    assert_eq!(
        serde_json::to_string(&AccessLevel::Write).unwrap(),
        "\"write\""
    );
    assert_eq!(
        serde_json::to_string(&AccessLevel::Admin).unwrap(),
        "\"admin\""
    );
    assert_eq!(
        serde_json::to_string(&AccessLevel::None).unwrap(),
        "\"none\""
    );
}

#[test]
fn access_level_round_trips_through_serde() {
    for level in [
        AccessLevel::None,
        AccessLevel::Read,
        AccessLevel::Triage,
        AccessLevel::Write,
        AccessLevel::Maintain,
        AccessLevel::Admin,
    ] {
        let serialized = serde_json::to_string(&level).unwrap();
        let deserialized: AccessLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, level, "Round-trip failed for {level:?}");
    }
}

// ── PermissionScope serde ─────────────────────────────────────────────────────

#[test]
fn permission_scope_round_trips_through_serde() {
    for scope in [
        PermissionScope::Repository,
        PermissionScope::Team,
        PermissionScope::User,
        PermissionScope::GitHubApp,
    ] {
        let serialized = serde_json::to_string(&scope).unwrap();
        let deserialized: PermissionScope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, scope, "Round-trip failed for {scope:?}");
    }
}

// ── PermissionType serde ──────────────────────────────────────────────────────

#[test]
fn permission_type_round_trips_through_serde() {
    for ptype in [
        PermissionType::Pull,
        PermissionType::Triage,
        PermissionType::Push,
        PermissionType::Maintain,
        PermissionType::Admin,
    ] {
        let serialized = serde_json::to_string(&ptype).unwrap();
        let deserialized: PermissionType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, ptype, "Round-trip failed for {ptype:?}");
    }
}

// ── GitHubPermissionLevel::as_str ─────────────────────────────────────────────

#[test]
fn github_permission_level_as_str_returns_correct_api_strings() {
    assert_eq!(GitHubPermissionLevel::Pull.as_str(), "pull");
    assert_eq!(GitHubPermissionLevel::Triage.as_str(), "triage");
    assert_eq!(GitHubPermissionLevel::Push.as_str(), "push");
    assert_eq!(GitHubPermissionLevel::Maintain.as_str(), "maintain");
    assert_eq!(GitHubPermissionLevel::Admin.as_str(), "admin");
}

// ── From<AccessLevel> for GitHubPermissionLevel ───────────────────────────────

#[test]
fn access_level_maps_to_github_permission_level_correctly() {
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::None),
        GitHubPermissionLevel::Pull,
        "AccessLevel::None should map to Pull (minimum GitHub level)"
    );
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::Read),
        GitHubPermissionLevel::Pull
    );
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::Triage),
        GitHubPermissionLevel::Triage
    );
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::Write),
        GitHubPermissionLevel::Push
    );
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::Maintain),
        GitHubPermissionLevel::Maintain
    );
    assert_eq!(
        GitHubPermissionLevel::from(AccessLevel::Admin),
        GitHubPermissionLevel::Admin
    );
}

// ── PermissionGrant::is_expired ───────────────────────────────────────────────

fn make_grant(expiration: Option<DateTime<Utc>>) -> PermissionGrant {
    PermissionGrant {
        conditions: vec![],
        expiration,
        level: AccessLevel::Read,
        permission_type: PermissionType::Pull,
        scope: PermissionScope::User,
    }
}

#[test]
fn permission_grant_without_expiration_is_not_expired() {
    let grant = make_grant(None);
    assert!(!grant.is_expired());
}

#[test]
fn permission_grant_with_future_expiration_is_not_expired() {
    let grant = make_grant(Some(Utc::now() + Duration::hours(24)));
    assert!(!grant.is_expired());
}

#[test]
fn permission_grant_with_past_expiration_is_expired() {
    let grant = make_grant(Some(Utc::now() - Duration::hours(1)));
    assert!(grant.is_expired());
}

#[test]
fn permission_grant_with_exactly_current_time_expiration_is_expired() {
    // A grant that expires at a timestamp in the past (even by a millisecond)
    // is considered expired.
    let grant = make_grant(Some(Utc::now() - Duration::milliseconds(1)));
    assert!(grant.is_expired());
}

// ── PermissionGrant serde ─────────────────────────────────────────────────────

#[test]
fn permission_grant_with_no_conditions_and_no_expiry_round_trips_through_serde() {
    let grant = PermissionGrant {
        conditions: vec![],
        expiration: None,
        level: AccessLevel::Write,
        permission_type: PermissionType::Push,
        scope: PermissionScope::Team,
    };
    let serialized = serde_json::to_string(&grant).unwrap();
    let deserialized: PermissionGrant = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, grant);
}

#[test]
fn permission_grant_with_conditions_round_trips_through_serde() {
    let grant = PermissionGrant {
        conditions: vec![PermissionCondition {
            description: "Only for private repos".to_string(),
            expression: Some("visibility == private".to_string()),
        }],
        expiration: None,
        level: AccessLevel::Admin,
        permission_type: PermissionType::Admin,
        scope: PermissionScope::Repository,
    };
    let serialized = serde_json::to_string(&grant).unwrap();
    let deserialized: PermissionGrant = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, grant);
}

// ── PermissionHierarchy construction ─────────────────────────────────────────

#[test]
fn permission_hierarchy_with_org_only_builds_successfully() {
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies::default(),
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    };
    assert!(hierarchy.repository_type_permissions.is_none());
    assert!(hierarchy.template_permissions.is_none());
}

#[test]
fn permission_hierarchy_with_all_levels_builds_successfully() {
    let read_grant = make_grant(None);

    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![read_grant.clone()],
            restrictions: vec![],
        },
        repository_type_permissions: Some(RepositoryTypePermissions {
            required_permissions: vec![read_grant.clone()],
            restricted_types: vec![PermissionType::Admin],
        }),
        template_permissions: Some(TemplatePermissions {
            required_permissions: vec![read_grant.clone()],
        }),
        user_requested_permissions: UserPermissionRequests {
            permissions: vec![read_grant],
        },
    };

    assert!(hierarchy.organization_policies.baseline_requirements.len() == 1);
    assert!(hierarchy.repository_type_permissions.is_some());
    assert!(hierarchy.template_permissions.is_some());
    assert!(hierarchy.user_requested_permissions.permissions.len() == 1);
}

// ── PermissionRequest construction ───────────────────────────────────────────

#[test]
fn permission_request_without_emergency_access_and_no_duration_builds() {
    let request = PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "Standard developer access".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new("my-org").unwrap(),
            repository: RepositoryName::new("my-repo").unwrap(),
        },
        requested_permissions: vec![],
        requestor: "jsmith".to_string(),
    };

    assert!(!request.emergency_access);
    assert!(request.duration.is_none());
    assert_eq!(request.requestor, "jsmith");
}

#[test]
fn permission_request_with_duration_records_seconds() {
    let request = PermissionRequest {
        duration: Some(PermissionDuration::from_seconds(3600)),
        emergency_access: false,
        justification: "Temporary access".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new("my-org").unwrap(),
            repository: RepositoryName::new("my-repo").unwrap(),
        },
        requested_permissions: vec![],
        requestor: "jsmith".to_string(),
    };

    assert_eq!(request.duration.unwrap().seconds, 3600);
}

// ── PermissionError variants ──────────────────────────────────────────────────

#[test]
fn permission_error_below_baseline_carries_full_context() {
    let err = PermissionError::BelowBaseline {
        permission_type: PermissionType::Push,
        level: AccessLevel::None,
        minimum_required: AccessLevel::Read,
    };

    match &err {
        PermissionError::BelowBaseline {
            permission_type,
            level,
            minimum_required,
        } => {
            assert_eq!(*permission_type, PermissionType::Push);
            assert_eq!(*level, AccessLevel::None);
            assert_eq!(*minimum_required, AccessLevel::Read);
        }
        _ => panic!("Wrong variant"),
    }

    // Verify Display includes meaningful info
    let msg = err.to_string();
    assert!(
        msg.contains("baseline"),
        "Error message should mention 'baseline': {msg}"
    );
}

#[test]
fn permission_error_exceeds_organization_limits_carries_full_context() {
    let err = PermissionError::ExceedsOrganizationLimits {
        permission_type: PermissionType::Admin,
        level: AccessLevel::Admin,
        maximum_allowed: AccessLevel::Maintain,
    };

    match &err {
        PermissionError::ExceedsOrganizationLimits {
            permission_type,
            level,
            maximum_allowed,
        } => {
            assert_eq!(*permission_type, PermissionType::Admin);
            assert_eq!(*level, AccessLevel::Admin);
            assert_eq!(*maximum_allowed, AccessLevel::Maintain);
        }
        _ => panic!("Wrong variant"),
    }

    let msg = err.to_string();
    assert!(
        msg.contains("maximum") || msg.contains("exceeds"),
        "Error message should mention limits: {msg}"
    );
}

#[test]
fn permission_error_template_requirement_conflict_carries_context() {
    let err = PermissionError::TemplateRequirementConflict {
        permission_type: PermissionType::Admin,
        level: AccessLevel::Admin,
    };

    let msg = err.to_string();
    assert!(
        msg.contains("Template") || msg.contains("template"),
        "Error message should mention template: {msg}"
    );
}

//  TryFrom config conversion tests

#[cfg(test)]
mod tryfrom_tests {
    use super::*;
    use config_manager::settings::{
        OrganizationPermissionPoliciesConfig, PermissionGrantConfig,
        RepositoryTypePermissionsConfig, TemplatePermissionsConfig,
    };

    fn make_grant(permission_type: &str, level: &str, scope: &str) -> PermissionGrantConfig {
        PermissionGrantConfig {
            permission_type: permission_type.to_string(),
            level: level.to_string(),
            scope: scope.to_string(),
        }
    }

    #[test]
    fn permission_grant_converts_push_write_team() {
        let cfg = make_grant("push", "write", "team");
        let _grant = PermissionGrant::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn permission_grant_converts_pull_read_user() {
        let cfg = make_grant("pull", "read", "user");
        let _grant = PermissionGrant::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn permission_grant_converts_admin_admin_repository() {
        let cfg = make_grant("admin", "admin", "repository");
        let _grant = PermissionGrant::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn org_policies_converts_baseline_and_restrictions() {
        let cfg = OrganizationPermissionPoliciesConfig {
            baseline: Some(vec![make_grant("push", "write", "team")]),
            restrictions: Some(vec![make_grant("admin", "admin", "repository")]),
        };
        let _policies =
            OrganizationPermissionPolicies::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn org_policies_converts_none_baseline_and_restrictions() {
        let cfg = OrganizationPermissionPoliciesConfig {
            baseline: None,
            restrictions: None,
        };
        let _policies =
            OrganizationPermissionPolicies::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn repo_type_perms_converts_required_and_restricted() {
        let cfg = RepositoryTypePermissionsConfig {
            required: Some(vec![make_grant("push", "write", "team")]),
            restricted_types: Some(vec!["admin".to_string()]),
        };
        let _perms = RepositoryTypePermissions::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn repo_type_perms_converts_empty() {
        let cfg = RepositoryTypePermissionsConfig {
            required: None,
            restricted_types: None,
        };
        let _perms = RepositoryTypePermissions::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn template_perms_converts_required() {
        let cfg = TemplatePermissionsConfig {
            required: Some(vec![make_grant("push", "write", "team")]),
        };
        let _perms = TemplatePermissions::try_from(&cfg).expect("conversion succeeds");
    }

    #[test]
    fn template_perms_converts_none_required() {
        let cfg = TemplatePermissionsConfig { required: None };
        let _perms = TemplatePermissions::try_from(&cfg).expect("conversion succeeds");
    }
}
