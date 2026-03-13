//! Tests for the permission_workflow module.
//!
//! Tests are organised into two groups:
//!
//! 1. **`build_permission_hierarchy` tests** — verify hierarchy construction
//!    from various combinations of template config.
//! 2. **`build_permission_request` tests** — verify request construction from
//!    owner, name and requestor inputs.

use config_manager::settings::{PermissionGrantConfig, TemplatePermissionsConfig};
use config_manager::template_config::TemplateMetadata;
use config_manager::TemplateConfig;

use super::*;
use crate::permissions::{
    AccessLevel, OrganizationPermissionPolicies, PermissionScope, PermissionType,
};
use crate::{OrganizationName, RepositoryName};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Returns a minimal valid [`TemplateConfig`] with no optional fields set.
fn minimal_template() -> TemplateConfig {
    TemplateConfig {
        template: TemplateMetadata {
            name: "test-template".to_string(),
            description: "A test template".to_string(),
            author: "test-author".to_string(),
            tags: vec![],
        },
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        rulesets: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
    }
}

/// Returns a [`TemplateConfig`] with a single required push/write/team permission.
fn template_with_one_required_permission() -> TemplateConfig {
    let mut t = minimal_template();
    t.permissions = Some(TemplatePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "push".to_string(),
            level: "write".to_string(),
            scope: "team".to_string(),
        }]),
    });
    t
}

/// Returns a [`TemplateConfig`] whose `permissions.required` list contains an
/// invalid permission-type string, triggering a conversion error.
fn template_with_invalid_permission() -> TemplateConfig {
    let mut t = minimal_template();
    t.permissions = Some(TemplatePermissionsConfig {
        required: Some(vec![PermissionGrantConfig {
            permission_type: "not-a-valid-type".to_string(),
            level: "write".to_string(),
            scope: "team".to_string(),
        }]),
    });
    t
}

// ─── build_permission_hierarchy tests ─────────────────────────────────────────

#[test]
fn build_permission_hierarchy_with_no_template_returns_no_template_permissions() {
    let hierarchy = build_permission_hierarchy(None);
    assert!(
        hierarchy.template_permissions.is_none(),
        "Expected no template_permissions when no template is provided"
    );
}

#[test]
fn build_permission_hierarchy_with_no_template_returns_no_type_permissions() {
    let hierarchy = build_permission_hierarchy(None);
    assert!(
        hierarchy.repository_type_permissions.is_none(),
        "Expected no repository_type_permissions in this release"
    );
}

#[test]
fn build_permission_hierarchy_with_no_template_uses_default_org_policies() {
    let hierarchy = build_permission_hierarchy(None);
    assert_eq!(
        hierarchy.organization_policies,
        OrganizationPermissionPolicies::default(),
        "Expected default (empty) org policies when no template is provided"
    );
}

#[test]
fn build_permission_hierarchy_with_no_template_has_empty_user_requests() {
    let hierarchy = build_permission_hierarchy(None);
    assert!(
        hierarchy.user_requested_permissions.permissions.is_empty(),
        "Expected empty user_requested_permissions when no template is provided"
    );
}

#[test]
fn build_permission_hierarchy_with_template_but_no_permissions_returns_no_template_permissions() {
    let template = minimal_template(); // permissions: None
    let hierarchy = build_permission_hierarchy(Some(&template));
    assert!(
        hierarchy.template_permissions.is_none(),
        "Expected no template_permissions when template has no permissions config"
    );
}

#[test]
fn build_permission_hierarchy_with_template_permissions_populates_template_layer() {
    let template = template_with_one_required_permission();
    let hierarchy = build_permission_hierarchy(Some(&template));
    let tp = hierarchy
        .template_permissions
        .expect("Expected template_permissions to be populated");
    assert_eq!(
        tp.required_permissions.len(),
        1,
        "Expected one required permission from template config"
    );
    let grant = &tp.required_permissions[0];
    assert_eq!(grant.permission_type, PermissionType::Push);
    assert_eq!(grant.level, AccessLevel::Write);
    assert_eq!(grant.scope, PermissionScope::Team);
}

#[test]
fn build_permission_hierarchy_with_template_permissions_preserves_default_org_policies() {
    let template = template_with_one_required_permission();
    let hierarchy = build_permission_hierarchy(Some(&template));
    assert_eq!(
        hierarchy.organization_policies,
        OrganizationPermissionPolicies::default(),
        "Expected default org policies even when template permissions are present"
    );
}

#[test]
fn build_permission_hierarchy_with_template_permissions_has_no_type_permissions() {
    let template = template_with_one_required_permission();
    let hierarchy = build_permission_hierarchy(Some(&template));
    assert!(
        hierarchy.repository_type_permissions.is_none(),
        "Expected no repository_type_permissions in this release"
    );
}

#[test]
fn build_permission_hierarchy_with_invalid_template_permissions_returns_no_template_layer() {
    // A conversion error must be treated as a non-fatal warning and produce None,
    // not panic or propagate an error type.
    let template = template_with_invalid_permission();
    let hierarchy = build_permission_hierarchy(Some(&template));
    assert!(
        hierarchy.template_permissions.is_none(),
        "Expected template_permissions to be None when config conversion fails"
    );
}

// ─── build_permission_request tests ───────────────────────────────────────────

fn make_org() -> OrganizationName {
    OrganizationName::new("test-org").expect("valid org name")
}

fn make_repo() -> RepositoryName {
    RepositoryName::new("test-repo").expect("valid repo name")
}

#[test]
fn build_permission_request_sets_organization_in_repository_context() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert_eq!(
        req.repository_context.organization,
        make_org(),
        "Expected organization set correctly in repository context"
    );
}

#[test]
fn build_permission_request_sets_repository_in_repository_context() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert_eq!(
        req.repository_context.repository,
        make_repo(),
        "Expected repository name set correctly in repository context"
    );
}

#[test]
fn build_permission_request_sets_requestor() {
    let req = build_permission_request(&make_org(), &make_repo(), "jsmith");
    assert_eq!(req.requestor, "jsmith", "Expected requestor to match input");
}

#[test]
fn build_permission_request_has_non_emergency_access() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert!(
        !req.emergency_access,
        "Expected emergency_access to be false for automated workflows"
    );
}

#[test]
fn build_permission_request_has_empty_requested_permissions() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert!(
        req.requested_permissions.is_empty(),
        "Expected requested_permissions to be empty in this release"
    );
}

#[test]
fn build_permission_request_has_no_duration() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert!(
        req.duration.is_none(),
        "Expected no duration for standard creation workflow"
    );
}

#[test]
fn build_permission_request_has_non_empty_justification() {
    let req = build_permission_request(&make_org(), &make_repo(), "user1");
    assert!(
        !req.justification.is_empty(),
        "Expected a non-empty justification string"
    );
}
