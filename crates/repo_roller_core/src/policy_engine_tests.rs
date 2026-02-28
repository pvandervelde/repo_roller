//! Unit tests for [`PolicyEngine`].

use super::*;
use crate::permissions::{
    AccessLevel, OrganizationPermissionPolicies, PermissionGrant, PermissionHierarchy,
    PermissionRequest, PermissionScope, PermissionType, RepositoryContext,
    RepositoryTypePermissions, TemplatePermissions, UserPermissionRequests,
};
use crate::{OrganizationName, RepositoryName};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn make_context() -> RepositoryContext {
    RepositoryContext {
        organization: OrganizationName::new("test-org").unwrap(),
        repository: RepositoryName::new("test-repo").unwrap(),
    }
}

fn bare_request(perms: Vec<PermissionGrant>) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "test".to_string(),
        repository_context: make_context(),
        requested_permissions: perms,
        requestor: "tester".to_string(),
    }
}

fn empty_hierarchy() -> PermissionHierarchy {
    PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies::default(),
        repository_type_permissions: None,
        template_permissions: None,
        user_requested_permissions: UserPermissionRequests::default(),
    }
}

fn grant(
    permission_type: PermissionType,
    scope: PermissionScope,
    level: AccessLevel,
) -> PermissionGrant {
    PermissionGrant {
        conditions: vec![],
        expiration: None,
        level,
        permission_type,
        scope,
    }
}

fn engine() -> PolicyEngine {
    PolicyEngine::new()
}

// ── Empty / trivial cases ─────────────────────────────────────────────────────

#[test]
fn empty_request_against_empty_hierarchy_is_approved() {
    let e = engine();
    let req = bare_request(vec![]);
    let hierarchy = empty_hierarchy();

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    assert!(
        matches!(result, PermissionEvaluationResult::Approved { granted_permissions, .. }
            if granted_permissions.is_empty()
        ),
        "Expected Approved with no grants"
    );
}

#[test]
fn approved_result_carries_effective_duration_from_request() {
    use crate::permissions::PermissionDuration;

    let e = engine();
    let mut req = bare_request(vec![]);
    req.duration = Some(PermissionDuration::from_seconds(3600));
    let hierarchy = empty_hierarchy();

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            effective_duration, ..
        } => {
            assert_eq!(
                effective_duration,
                Some(PermissionDuration::from_seconds(3600))
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

// ── Organization baseline validation ─────────────────────────────────────────

#[test]
fn request_within_org_restriction_is_approved() {
    let e = engine();
    let restriction = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![restriction],
        },
        ..empty_hierarchy()
    };
    let req = bare_request(vec![grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    )]);

    let result = e.evaluate_permission_request(&req, &hierarchy);

    assert!(result.is_ok(), "Expected Ok: {result:?}");
    assert!(matches!(
        result.unwrap(),
        PermissionEvaluationResult::Approved { .. }
    ));
}

#[test]
fn request_exceeding_org_restriction_returns_exceeds_limits_error() {
    let e = engine();
    let restriction = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![restriction],
        },
        ..empty_hierarchy()
    };
    // Request Admin — above the Write ceiling
    let req = bare_request(vec![grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Admin,
    )]);

    let err = e.evaluate_permission_request(&req, &hierarchy).unwrap_err();

    assert!(
        matches!(
            err,
            PermissionError::ExceedsOrganizationLimits {
                permission_type: PermissionType::Push,
                maximum_allowed: AccessLevel::Write,
                ..
            }
        ),
        "Unexpected error: {err:?}"
    );
}

#[test]
fn restriction_only_applies_to_same_permission_type_and_scope() {
    let e = engine();
    // Restriction on Team Push; requesting User Admin is unrelated → OK
    let restriction = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![restriction],
        },
        ..empty_hierarchy()
    };
    let req = bare_request(vec![grant(
        PermissionType::Admin,
        PermissionScope::User,
        AccessLevel::Admin,
    )]);

    assert!(e.evaluate_permission_request(&req, &hierarchy).is_ok());
}

#[test]
fn request_below_org_baseline_returns_below_baseline_error() {
    let e = engine();
    let baseline = grant(
        PermissionType::Pull,
        PermissionScope::Team,
        AccessLevel::Read,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![baseline],
            restrictions: vec![],
        },
        ..empty_hierarchy()
    };
    // Request Triage for Pull/Team — but None is below the Read baseline
    let req = bare_request(vec![grant(
        PermissionType::Pull,
        PermissionScope::Team,
        AccessLevel::None,
    )]);

    let err = e.evaluate_permission_request(&req, &hierarchy).unwrap_err();

    assert!(
        matches!(
            err,
            PermissionError::BelowBaseline {
                permission_type: PermissionType::Pull,
                minimum_required: AccessLevel::Read,
                ..
            }
        ),
        "Unexpected error: {err:?}"
    );
}

// ── Repository type permission validation ────────────────────────────────────

#[test]
fn request_using_restricted_type_returns_exceeds_limits_error() {
    let e = engine();
    let hierarchy = PermissionHierarchy {
        repository_type_permissions: Some(RepositoryTypePermissions {
            required_permissions: vec![],
            restricted_types: vec![PermissionType::Admin],
        }),
        ..empty_hierarchy()
    };
    let req = bare_request(vec![grant(
        PermissionType::Admin,
        PermissionScope::User,
        AccessLevel::Admin,
    )]);

    let err = e.evaluate_permission_request(&req, &hierarchy).unwrap_err();

    assert!(
        matches!(
            err,
            PermissionError::ExceedsOrganizationLimits {
                permission_type: PermissionType::Admin,
                maximum_allowed: AccessLevel::None,
                ..
            }
        ),
        "Unexpected error: {err:?}"
    );
}

#[test]
fn request_not_using_restricted_type_passes_type_check() {
    let e = engine();
    let hierarchy = PermissionHierarchy {
        repository_type_permissions: Some(RepositoryTypePermissions {
            required_permissions: vec![],
            restricted_types: vec![PermissionType::Admin],
        }),
        ..empty_hierarchy()
    };
    // Push is not restricted
    let req = bare_request(vec![grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    )]);

    assert!(e.evaluate_permission_request(&req, &hierarchy).is_ok());
}

// ── Template permission validation ───────────────────────────────────────────

#[test]
fn template_requiring_permission_that_org_allows_passes() {
    let e = engine();
    let hierarchy = PermissionHierarchy {
        template_permissions: Some(TemplatePermissions {
            required_permissions: vec![grant(
                PermissionType::Push,
                PermissionScope::Team,
                AccessLevel::Write,
            )],
        }),
        ..empty_hierarchy()
    };
    let req = bare_request(vec![]);

    assert!(e.evaluate_permission_request(&req, &hierarchy).is_ok());
}

#[test]
fn template_requiring_permission_above_org_restriction_returns_template_conflict_error() {
    let e = engine();
    // Org restricts Admin Push to Write max; template requires Admin
    let restriction = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![],
            restrictions: vec![restriction],
        },
        template_permissions: Some(TemplatePermissions {
            required_permissions: vec![grant(
                PermissionType::Push,
                PermissionScope::Team,
                AccessLevel::Admin,
            )],
        }),
        ..empty_hierarchy()
    };
    let req = bare_request(vec![]);

    let err = e.evaluate_permission_request(&req, &hierarchy).unwrap_err();

    assert!(
        matches!(
            err,
            PermissionError::TemplateRequirementConflict {
                permission_type: PermissionType::Push,
                level: AccessLevel::Admin,
            }
        ),
        "Unexpected error: {err:?}"
    );
}

// ── Emergency access ──────────────────────────────────────────────────────────

#[test]
fn emergency_access_request_requires_approval() {
    let e = engine();
    let mut req = bare_request(vec![grant(
        PermissionType::Admin,
        PermissionScope::User,
        AccessLevel::Admin,
    )]);
    req.emergency_access = true;

    let result = e
        .evaluate_permission_request(&req, &empty_hierarchy())
        .unwrap();

    assert!(
        matches!(result, PermissionEvaluationResult::RequiresApproval { .. }),
        "Expected RequiresApproval for emergency access"
    );
}

#[test]
fn emergency_access_result_includes_requested_permissions() {
    let e = engine();
    let admin_grant = grant(
        PermissionType::Admin,
        PermissionScope::User,
        AccessLevel::Admin,
    );
    let mut req = bare_request(vec![admin_grant.clone()]);
    req.emergency_access = true;

    let result = e
        .evaluate_permission_request(&req, &empty_hierarchy())
        .unwrap();

    match result {
        PermissionEvaluationResult::RequiresApproval {
            restricted_permissions,
            ..
        } => {
            assert!(restricted_permissions.contains(&admin_grant));
        }
        other => panic!("Expected RequiresApproval, got {other:?}"),
    }
}

// ── Permission merging ────────────────────────────────────────────────────────

#[test]
fn approved_result_includes_org_baseline_grants() {
    let e = engine();
    let baseline = grant(
        PermissionType::Pull,
        PermissionScope::Team,
        AccessLevel::Read,
    );
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![baseline.clone()],
            restrictions: vec![],
        },
        ..empty_hierarchy()
    };
    let req = bare_request(vec![]);

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            assert!(
                granted_permissions.contains(&baseline),
                "Expected baseline grant in result; got {granted_permissions:?}"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

#[test]
fn approved_result_includes_template_required_grants() {
    let e = engine();
    let template_perm = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let hierarchy = PermissionHierarchy {
        template_permissions: Some(TemplatePermissions {
            required_permissions: vec![template_perm.clone()],
        }),
        ..empty_hierarchy()
    };
    let req = bare_request(vec![]);

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            assert!(
                granted_permissions.contains(&template_perm),
                "Expected template grant in result; got {granted_permissions:?}"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

#[test]
fn approved_result_includes_repo_type_required_grants() {
    let e = engine();
    let type_perm = grant(
        PermissionType::Pull,
        PermissionScope::Repository,
        AccessLevel::Read,
    );
    let hierarchy = PermissionHierarchy {
        repository_type_permissions: Some(RepositoryTypePermissions {
            required_permissions: vec![type_perm.clone()],
            restricted_types: vec![],
        }),
        ..empty_hierarchy()
    };
    let req = bare_request(vec![]);

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            assert!(
                granted_permissions.contains(&type_perm),
                "Expected repo-type required grant in result; got {granted_permissions:?}"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

#[test]
fn approved_result_includes_user_requested_grants() {
    let e = engine();
    let user_perm = grant(
        PermissionType::Maintain,
        PermissionScope::Team,
        AccessLevel::Maintain,
    );
    let req = bare_request(vec![user_perm.clone()]);

    let result = e
        .evaluate_permission_request(&req, &empty_hierarchy())
        .unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            assert!(
                granted_permissions.contains(&user_perm),
                "Expected user grant in result; got {granted_permissions:?}"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

#[test]
fn duplicate_grants_across_layers_are_deduplicated_in_result() {
    let e = engine();
    let perm = grant(
        PermissionType::Pull,
        PermissionScope::Team,
        AccessLevel::Read,
    );
    // Same permission appears in baseline AND user request
    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![perm.clone()],
            restrictions: vec![],
        },
        ..empty_hierarchy()
    };
    let req = bare_request(vec![perm.clone()]);

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            let occurrences = granted_permissions.iter().filter(|g| *g == &perm).count();
            assert_eq!(
                occurrences, 1,
                "Expected exactly one occurrence; got {occurrences}"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}

// ── Merged result from all four layers ───────────────────────────────────────

#[test]
fn approved_result_merges_all_four_hierarchy_layers() {
    let e = engine();
    let org_perm = grant(
        PermissionType::Pull,
        PermissionScope::Team,
        AccessLevel::Read,
    );
    let type_perm = grant(
        PermissionType::Push,
        PermissionScope::Team,
        AccessLevel::Write,
    );
    let tmpl_perm = grant(
        PermissionType::Triage,
        PermissionScope::User,
        AccessLevel::Triage,
    );
    let user_perm = grant(
        PermissionType::Maintain,
        PermissionScope::Team,
        AccessLevel::Maintain,
    );

    let hierarchy = PermissionHierarchy {
        organization_policies: OrganizationPermissionPolicies {
            baseline_requirements: vec![org_perm.clone()],
            restrictions: vec![],
        },
        repository_type_permissions: Some(RepositoryTypePermissions {
            required_permissions: vec![type_perm.clone()],
            restricted_types: vec![],
        }),
        template_permissions: Some(TemplatePermissions {
            required_permissions: vec![tmpl_perm.clone()],
        }),
        user_requested_permissions: UserPermissionRequests::default(),
    };
    let req = bare_request(vec![user_perm.clone()]);

    let result = e.evaluate_permission_request(&req, &hierarchy).unwrap();

    match result {
        PermissionEvaluationResult::Approved {
            granted_permissions,
            ..
        } => {
            assert!(
                granted_permissions.contains(&org_perm),
                "missing org baseline"
            );
            assert!(
                granted_permissions.contains(&type_perm),
                "missing type required"
            );
            assert!(
                granted_permissions.contains(&tmpl_perm),
                "missing template required"
            );
            assert!(
                granted_permissions.contains(&user_perm),
                "missing user requested"
            );
        }
        other => panic!("Expected Approved, got {other:?}"),
    }
}
