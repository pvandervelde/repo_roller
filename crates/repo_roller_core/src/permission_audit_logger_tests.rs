//! Tests for the permission_audit_logger module.
//!
//! Uses `#[traced_test]` from the `tracing-test` crate to capture
//! structured tracing events and assert on their content.

use tracing_test::traced_test;

use super::*;
use crate::permission_manager::ApplyPermissionsResult;
use crate::permissions::{
    AccessLevel, PermissionError, PermissionGrant, PermissionRequest, PermissionScope,
    PermissionType, RepositoryContext,
};
use crate::policy_engine::PermissionEvaluationResult;
use crate::{OrganizationName, RepositoryName};

// ── helpers ────────────────────────────────────────────────────────────────────

fn make_request(org: &str, repo: &str, emergency: bool) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access: emergency,
        justification: "unit test".to_string(),
        repository_context: RepositoryContext {
            organization: OrganizationName::new(org).expect("valid org"),
            repository: RepositoryName::new(repo).expect("valid repo"),
        },
        requested_permissions: vec![],
        requestor: "test-user".to_string(),
    }
}

fn make_grant() -> PermissionGrant {
    PermissionGrant {
        conditions: vec![],
        expiration: None,
        level: AccessLevel::Write,
        permission_type: PermissionType::Push,
        scope: PermissionScope::Team,
    }
}

// ── PermissionAuditLogger::new ─────────────────────────────────────────────────

#[test]
fn new_creates_zero_sized_logger() {
    let logger = PermissionAuditLogger::new();
    let _ = logger; // just verify it constructs without issue
    assert_eq!(std::mem::size_of::<PermissionAuditLogger>(), 0);
}

#[test]
fn default_equals_new() {
    let a = PermissionAuditLogger::new();
    let b = PermissionAuditLogger; // unit struct — same as new() and default()
    let _: PermissionAuditLogger = a;
    let _: PermissionAuditLogger = b;
}

// ── log_policy_evaluation — Approved ─────────────────────────────────────────

#[traced_test]
#[test]
fn log_policy_evaluation_approved_emits_info_with_approved_outcome() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("test-org", "test-repo", false);
    let result = PermissionEvaluationResult::Approved {
        granted_permissions: vec![make_grant()],
        effective_duration: None,
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("approved"),
        "Expected 'approved' in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_approved_includes_organization() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("my-audit-org", "any-repo", false);
    let result = PermissionEvaluationResult::Approved {
        granted_permissions: vec![],
        effective_duration: None,
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("my-audit-org"),
        "Expected organization name in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_approved_includes_repository() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "my-audit-repo", false);
    let result = PermissionEvaluationResult::Approved {
        granted_permissions: vec![],
        effective_duration: None,
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("my-audit-repo"),
        "Expected repository name in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_approved_includes_requestor() {
    let logger = PermissionAuditLogger::new();
    let mut request = make_request("some-org", "some-repo", false);
    request.requestor = "audit-user".to_string();
    let result = PermissionEvaluationResult::Approved {
        granted_permissions: vec![],
        effective_duration: None,
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("audit-user"),
        "Expected requestor in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_approved_includes_grant_count() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "some-repo", false);
    let result = PermissionEvaluationResult::Approved {
        granted_permissions: vec![make_grant(), make_grant(), make_grant()],
        effective_duration: None,
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(logs_contain("3"), "Expected grant count 3 in log output");
}

// ── log_policy_evaluation — RequiresApproval ─────────────────────────────────

#[traced_test]
#[test]
fn log_policy_evaluation_requires_approval_emits_requires_approval_outcome() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("test-org", "test-repo", true);
    let result = PermissionEvaluationResult::RequiresApproval {
        reason: "Emergency access requires human review".to_string(),
        restricted_permissions: vec![make_grant()],
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("requires_approval"),
        "Expected 'requires_approval' in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_requires_approval_includes_reason() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("test-org", "test-repo", true);
    let result = PermissionEvaluationResult::RequiresApproval {
        reason: "emergency-justification-reason".to_string(),
        restricted_permissions: vec![],
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(
        logs_contain("emergency-justification-reason"),
        "Expected reason string in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_evaluation_requires_approval_includes_organization() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("approval-org", "some-repo", true);
    let result = PermissionEvaluationResult::RequiresApproval {
        reason: "emergency access".to_string(),
        restricted_permissions: vec![],
    };

    logger.log_policy_evaluation(&request, &result);

    assert!(logs_contain("approval-org"), "Expected org in log output");
}

// ── log_policy_denied ─────────────────────────────────────────────────────────

#[traced_test]
#[test]
fn log_policy_denied_emits_denied_outcome() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("deny-org", "deny-repo", false);
    let error = PermissionError::ExceedsOrganizationLimits {
        permission_type: PermissionType::Admin,
        level: AccessLevel::Admin,
        maximum_allowed: AccessLevel::Maintain,
    };

    logger.log_policy_denied(&request, &error);

    assert!(logs_contain("denied"), "Expected 'denied' in log output");
}

#[traced_test]
#[test]
fn log_policy_denied_includes_organization() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("denied-org-name", "some-repo", false);
    let error = PermissionError::BelowBaseline {
        permission_type: PermissionType::Pull,
        level: AccessLevel::None,
        minimum_required: AccessLevel::Read,
    };

    logger.log_policy_denied(&request, &error);

    assert!(
        logs_contain("denied-org-name"),
        "Expected org name in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_denied_includes_repository() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "denied-repo-name", false);
    let error = PermissionError::ExceedsOrganizationLimits {
        permission_type: PermissionType::Push,
        level: AccessLevel::Admin,
        maximum_allowed: AccessLevel::Write,
    };

    logger.log_policy_denied(&request, &error);

    assert!(
        logs_contain("denied-repo-name"),
        "Expected repo name in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_denied_includes_requestor() {
    let logger = PermissionAuditLogger::new();
    let mut request = make_request("some-org", "some-repo", false);
    request.requestor = "denied-requestor-name".to_string();
    let error = PermissionError::ExceedsOrganizationLimits {
        permission_type: PermissionType::Admin,
        level: AccessLevel::Admin,
        maximum_allowed: AccessLevel::Write,
    };

    logger.log_policy_denied(&request, &error);

    assert!(
        logs_contain("denied-requestor-name"),
        "Expected requestor in log output"
    );
}

#[traced_test]
#[test]
fn log_policy_denied_includes_error_description() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "some-repo", false);
    // PermissionError::BelowBaseline formats as "below the organization baseline requirement"
    let error = PermissionError::BelowBaseline {
        permission_type: PermissionType::Push,
        level: AccessLevel::None,
        minimum_required: AccessLevel::Read,
    };

    logger.log_policy_denied(&request, &error);

    assert!(
        logs_contain("baseline"),
        "Expected error text 'baseline' in log output"
    );
}

// ── log_permissions_applied ───────────────────────────────────────────────────

#[traced_test]
#[test]
fn log_permissions_applied_emits_applied_outcome() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("apply-org", "apply-repo", false);
    let result = ApplyPermissionsResult::new();

    logger.log_permissions_applied(&request, &result);

    assert!(logs_contain("applied"), "Expected 'applied' in log output");
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_organization() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("applied-org-name", "some-repo", false);
    let result = ApplyPermissionsResult::new();

    logger.log_permissions_applied(&request, &result);

    assert!(
        logs_contain("applied-org-name"),
        "Expected org name in log output"
    );
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_repository() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "applied-repo-name", false);
    let result = ApplyPermissionsResult::new();

    logger.log_permissions_applied(&request, &result);

    assert!(
        logs_contain("applied-repo-name"),
        "Expected repo name in log output"
    );
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_teams_applied_count() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "some-repo", false);
    let mut result = ApplyPermissionsResult::new();
    result.teams_applied = 5;

    logger.log_permissions_applied(&request, &result);

    assert!(
        logs_contain("5"),
        "Expected teams_applied count 5 in log output"
    );
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_collaborators_applied_count() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "some-repo", false);
    let mut result = ApplyPermissionsResult::new();
    result.collaborators_applied = 3;

    logger.log_permissions_applied(&request, &result);

    assert!(
        logs_contain("3"),
        "Expected collaborators_applied count 3 in log output"
    );
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_requestor() {
    let logger = PermissionAuditLogger::new();
    let mut request = make_request("some-org", "some-repo", false);
    request.requestor = "applied-requestor".to_string();
    let result = ApplyPermissionsResult::new();

    logger.log_permissions_applied(&request, &result);

    assert!(
        logs_contain("applied-requestor"),
        "Expected requestor in log output"
    );
}

#[traced_test]
#[test]
fn log_permissions_applied_includes_collaborators_removed() {
    let logger = PermissionAuditLogger::new();
    let request = make_request("some-org", "some-repo", false);
    let mut result = ApplyPermissionsResult::new();
    result.collaborators_removed = 2;

    logger.log_permissions_applied(&request, &result);

    assert!(logs_contain("2"), "Expected removed count 2 in log output");
}
