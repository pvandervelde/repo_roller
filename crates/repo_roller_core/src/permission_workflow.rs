//! Permission workflow helpers for the repository creation workflow.
//!
//! This module bridges the repository-creation orchestration in [`crate::create_repository`]
//! and the permission management system in [`crate::permission_manager`].
//! It assembles the domain objects that
//! [`crate::permission_manager::PermissionManager::apply_repository_permissions`]
//! requires from the data that is already available in the creation workflow.
//!
//! ## Responsibilities
//!
//! Two helper functions cover the two arguments that vary per creation request:
//!
//! - [`build_permission_hierarchy`] — assembles the four-layer policy hierarchy
//!   from configuration data available at creation time.  In this release only
//!   the template layer is populated; org-level and type-level policies will be
//!   wired in when [`config_manager::MergedConfiguration`] exposes them.
//!
//! - [`build_permission_request`] — constructs the [`PermissionRequest`] from
//!   the repository owner, name, and the identity of the person or service that
//!   initiated the creation.
//!
//! ## Future extensibility
//!
//! The two helpers contain all the logic for building these objects in one place,
//! so that future tasks (e.g. adding org-level policy threading or user-requested
//! permissions from the creation request) can be made by editing only this module.

use tracing::warn;

use crate::permissions::{
    OrganizationPermissionPolicies, PermissionHierarchy, PermissionRequest, RepositoryContext,
    TemplatePermissions, UserPermissionRequests,
};
use crate::{OrganizationName, RepositoryName};

#[cfg(test)]
#[path = "permission_workflow_tests.rs"]
mod tests;

// ── Public helpers ─────────────────────────────────────────────────────────────

/// Builds a [`PermissionHierarchy`] from creation-workflow context.
///
/// The hierarchy assembles the four policy layers that the
/// [`crate::policy_engine::PolicyEngine`] evaluates against a
/// [`PermissionRequest`]:
///
/// | Layer | Source | This release |
/// |---|---|---|
/// | Organization policies | `MergedConfiguration` (future) | `default()` (empty) |
/// | Repository-type permissions | `MergedConfiguration` (future) | `None` |
/// | Template permissions | `template.permissions` | populated when present |
/// | User-requested permissions | `RepositoryCreationRequest` (future) | empty |
///
/// A conversion error for the template permissions config is treated as a
/// non-fatal warning: the `template_permissions` layer is set to `None` and
/// creation continues.  This prevents a misconfigured template from blocking
/// every repository creation.
///
/// # Arguments
///
/// * `template` - Optional template configuration loaded during the creation
///   workflow.  Pass `None` for empty-repository or no-template creations.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permission_workflow::build_permission_hierarchy;
///
/// let hierarchy = build_permission_hierarchy(None);
/// assert!(hierarchy.template_permissions.is_none());
/// assert!(hierarchy.repository_type_permissions.is_none());
/// ```
pub fn build_permission_hierarchy(
    template: Option<&config_manager::TemplateConfig>,
) -> PermissionHierarchy {
    // Extract and convert template-level permission requirements.
    // Conversion errors are non-fatal: log a warning and proceed with no
    // template permissions rather than blocking repository creation.
    let template_permissions = template.and_then(|t| t.permissions.as_ref()).and_then(|p| {
        TemplatePermissions::try_from(p)
            .map_err(|e| {
                warn!(
                    target: "repo_roller_core::permission_workflow",
                    "Failed to convert template permissions config; \
                     proceeding with no template-layer constraints: {}",
                    e
                );
            })
            .ok()
    });

    PermissionHierarchy {
        // Org-level policies are not yet threaded through MergedConfiguration;
        // default() represents an empty policy (no baseline, no restrictions).
        organization_policies: OrganizationPermissionPolicies::default(),
        // Repository-type permissions are not yet wired; populated in a future task.
        repository_type_permissions: None,
        template_permissions,
        // User-requested permissions come from RepositoryCreationRequest; added in a future task.
        user_requested_permissions: UserPermissionRequests::default(),
    }
}

/// Builds a [`PermissionRequest`] from repository creation request data.
///
/// Constructs a standard, non-emergency permission request for the automated
/// repository-creation workflow.  The request carries no pre-filled permission
/// grants (those are accumulated by later tasks when the creation request gains
/// explicit permission fields) and is attributed to the given `requestor`.
///
/// # Arguments
///
/// * `owner`     - GitHub organization that owns the repository.
/// * `name`      - Name of the repository being created.
/// * `requestor` - GitHub username or service identity initiating the creation.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permission_workflow::build_permission_request;
/// use repo_roller_core::{OrganizationName, RepositoryName};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let owner = OrganizationName::new("my-org")?;
/// let name  = RepositoryName::new("my-repo")?;
/// let req   = build_permission_request(&owner, &name, "jsmith");
///
/// assert_eq!(req.requestor, "jsmith");
/// assert!(!req.emergency_access);
/// assert!(req.requested_permissions.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn build_permission_request(
    owner: &OrganizationName,
    name: &RepositoryName,
    requestor: &str,
) -> PermissionRequest {
    PermissionRequest {
        duration: None,
        emergency_access: false,
        justification: "Repository creation workflow".to_string(),
        repository_context: RepositoryContext {
            organization: owner.clone(),
            repository: name.clone(),
        },
        requested_permissions: vec![],
        requestor: requestor.to_string(),
    }
}
