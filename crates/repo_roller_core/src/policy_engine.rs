//! Permission policy evaluation engine.
//!
//! This module provides the [`PolicyEngine`], which evaluates
//! [`PermissionRequest`]s against a [`PermissionHierarchy`] to produce an
//! [`PermissionEvaluationResult`].
//!
//! ## Evaluation Flow
//!
//! ```text
//! PermissionRequest
//!   1. Organization baseline validation  (floor & ceiling)
//!   2. Repository type permission validation (restricted types)
//!   3. Template permission validation  (template ↔ org conflict check)
//!   4. User permission validation  (user request vs. org restrictions)
//!   5. Emergency-access gate  → RequiresApproval
//!   6. Merge all layers → Approved
//! ```
//!
//! All evaluation steps are pure (no I/O). The caller is responsible for
//! loading the [`PermissionHierarchy`] from configuration.
//!
//! See `docs/spec/design/multi-level-permissions.md` for the full spec.

use tracing::instrument;

use crate::permissions::{
    AccessLevel, OrganizationPermissionPolicies, PermissionDuration, PermissionGrant,
    PermissionHierarchy, PermissionRequest, PermissionType, RepositoryTypePermissions,
    TemplatePermissions,
};

pub use crate::permissions::PermissionError;

#[cfg(test)]
#[path = "policy_engine_tests.rs"]
mod tests;

// ── Evaluation result ─────────────────────────────────────────────────────────

/// The outcome of evaluating a [`PermissionRequest`] against a
/// [`PermissionHierarchy`].
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::policy_engine::{PolicyEngine, PermissionEvaluationResult};
/// use repo_roller_core::permissions::{
///     AccessLevel, PermissionGrant, PermissionHierarchy, PermissionRequest,
///     PermissionScope, PermissionType, RepositoryContext,
///     OrganizationPermissionPolicies, UserPermissionRequests,
/// };
/// use repo_roller_core::{OrganizationName, RepositoryName};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let engine = PolicyEngine::new();
///
/// let request = PermissionRequest {
///     duration: None,
///     emergency_access: false,
///     justification: "Standard setup".to_string(),
///     repository_context: RepositoryContext {
///         organization: OrganizationName::new("my-org")?,
///         repository: RepositoryName::new("my-repo")?,
///     },
///     requested_permissions: vec![],
///     requestor: "jsmith".to_string(),
/// };
///
/// let hierarchy = PermissionHierarchy {
///     organization_policies: OrganizationPermissionPolicies::default(),
///     repository_type_permissions: None,
///     template_permissions: None,
///     user_requested_permissions: UserPermissionRequests::default(),
/// };
///
/// let result = engine.evaluate_permission_request(&request, &hierarchy)?;
/// assert!(matches!(result, PermissionEvaluationResult::Approved { .. }));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionEvaluationResult {
    /// All validations passed; the listed permissions are granted.
    Approved {
        /// The effective set of permissions to apply, drawn from all
        /// hierarchy layers (baseline + type + template + user-requested).
        granted_permissions: Vec<PermissionGrant>,
        /// Optional effective duration, sourced from the request.
        effective_duration: Option<PermissionDuration>,
    },
    /// The request requires explicit human approval before permissions can
    /// be applied (e.g. emergency access).
    RequiresApproval {
        /// Human-readable reason for requiring approval.
        reason: String,
        /// The permissions that triggered the approval requirement.
        restricted_permissions: Vec<PermissionGrant>,
    },
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Evaluates permission requests against the configured hierarchy.
///
/// `PolicyEngine` is a pure-computation component: it takes a
/// [`PermissionRequest`] and a [`PermissionHierarchy`] and returns a
/// [`PermissionEvaluationResult`] without performing any I/O. Loading the
/// hierarchy from configuration is the caller's responsibility.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::policy_engine::PolicyEngine;
///
/// let engine = PolicyEngine::new();
/// // engine.evaluate_permission_request(&request, &hierarchy)?;
/// ```
#[derive(Debug, Default)]
pub struct PolicyEngine;

impl PolicyEngine {
    /// Creates a new `PolicyEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates a [`PermissionRequest`] against the provided
    /// [`PermissionHierarchy`].
    ///
    /// ### Evaluation order
    ///
    /// 1. Organization baseline — enforce floor (baseline) and ceiling
    ///    (restrictions).
    /// 2. Repository type — reject permissions using a restricted type.
    /// 3. Template — detect conflicts between template requirements and org
    ///    restrictions.
    /// 4. User permissions — check user requests against org restrictions.
    /// 5. Emergency access gate — route to [`PermissionEvaluationResult::RequiresApproval`].
    /// 6. Merge layers — assemble the final granted-permission list and
    ///    return [`PermissionEvaluationResult::Approved`].
    ///
    /// # Errors
    ///
    /// Returns [`PermissionError`] when a validation step fails.
    ///
    /// - [`PermissionError::BelowBaseline`] — a requested level is below the
    ///   org baseline minimum.
    /// - [`PermissionError::ExceedsOrganizationLimits`] — a requested level
    ///   exceeds the org maximum, or uses a type restricted by the repository
    ///   type config.
    /// - [`PermissionError::TemplateRequirementConflict`] — the template
    ///   requires a permission that org policy disallows.
    #[instrument(skip(self, hierarchy), fields(
        requestor = %request.requestor,
        emergency_access = request.emergency_access,
        repository = %request.repository_context.repository,
        organization = %request.repository_context.organization
    ))]
    pub fn evaluate_permission_request(
        &self,
        request: &PermissionRequest,
        hierarchy: &PermissionHierarchy,
    ) -> Result<PermissionEvaluationResult, PermissionError> {
        // Step 1: Organization baseline — enforce floor (baseline) and ceiling (restrictions).
        self.validate_organization_baseline(
            &request.requested_permissions,
            &hierarchy.organization_policies,
        )?;

        // Step 2: Repository type — reject any permission using a restricted type.
        if let Some(type_perms) = &hierarchy.repository_type_permissions {
            self.validate_repository_type_permissions(&request.requested_permissions, type_perms)?;
        }

        // Step 3: Template — detect conflicts between template requirements and org restrictions.
        if let Some(template) = &hierarchy.template_permissions {
            self.validate_template_permissions(template, &hierarchy.organization_policies)?;
        }

        // Step 4: User permission validation — user requests vs. org restrictions (explicit step
        // for traceability; structurally overlaps with step 1 but is a distinct evaluation pass).
        self.validate_user_permissions(
            &request.requested_permissions,
            &hierarchy.organization_policies,
        )?;

        // Step 5: Emergency access gate — route to RequiresApproval before granting anything.
        if request.emergency_access {
            return Ok(PermissionEvaluationResult::RequiresApproval {
                reason: "Emergency access requests require explicit approval before \
                         permissions can be applied"
                    .to_string(),
                restricted_permissions: request.requested_permissions.clone(),
            });
        }

        // Step 6: Merge all hierarchy layers and approve.
        let granted_permissions = self.merge_permissions(hierarchy, &request.requested_permissions);

        Ok(PermissionEvaluationResult::Approved {
            granted_permissions,
            effective_duration: request.duration,
        })
    }

    /// Validates that requested permissions respect the organization's
    /// baseline floor and ceiling.
    ///
    /// - For each **restriction** in `policies.restrictions`: any requested
    ///   permission with the same `permission_type` and `scope` must not
    ///   exceed the restriction's `level` → [`PermissionError::ExceedsOrganizationLimits`].
    /// - For each **baseline requirement** in `policies.baseline_requirements`:
    ///   any requested permission with the same `permission_type` and `scope`
    ///   must be at or above the requirement's `level` →
    ///   [`PermissionError::BelowBaseline`].
    fn validate_organization_baseline(
        &self,
        requested: &[PermissionGrant],
        policies: &OrganizationPermissionPolicies,
    ) -> Result<(), PermissionError> {
        for req in requested {
            // Ceiling check: must not exceed the restriction maximum for this type+scope.
            for restriction in &policies.restrictions {
                if restriction.permission_type == req.permission_type
                    && restriction.scope == req.scope
                    && req.level > restriction.level
                {
                    return Err(PermissionError::ExceedsOrganizationLimits {
                        permission_type: req.permission_type,
                        level: req.level,
                        maximum_allowed: restriction.level,
                    });
                }
            }

            // Floor check: must not fall below the baseline level for this type+scope.
            for baseline in &policies.baseline_requirements {
                if baseline.permission_type == req.permission_type
                    && baseline.scope == req.scope
                    && req.level < baseline.level
                {
                    return Err(PermissionError::BelowBaseline {
                        permission_type: req.permission_type,
                        level: req.level,
                        minimum_required: baseline.level,
                    });
                }
            }
        }
        Ok(())
    }

    /// Validates that no requested permission uses a permission type that the
    /// repository type configuration marks as restricted.
    ///
    /// A restricted type is completely disallowed for this repository type.
    /// Any request using one returns
    /// [`PermissionError::ExceedsOrganizationLimits`] with
    /// `maximum_allowed: AccessLevel::None`.
    fn validate_repository_type_permissions(
        &self,
        requested: &[PermissionGrant],
        type_perms: &RepositoryTypePermissions,
    ) -> Result<(), PermissionError> {
        for req in requested {
            if self.is_restricted_type(req.permission_type, type_perms) {
                return Err(PermissionError::ExceedsOrganizationLimits {
                    permission_type: req.permission_type,
                    level: req.level,
                    maximum_allowed: AccessLevel::None,
                });
            }
        }
        Ok(())
    }

    /// Validates that template-required permissions do not conflict with
    /// organization policy.
    ///
    /// For each permission required by the template, this method checks
    /// whether the organization's restrictions would forbid it.  If a
    /// template requires a permission that org policy disallows, it returns
    /// [`PermissionError::TemplateRequirementConflict`].
    fn validate_template_permissions(
        &self,
        template: &TemplatePermissions,
        policies: &OrganizationPermissionPolicies,
    ) -> Result<(), PermissionError> {
        for required in &template.required_permissions {
            for restriction in &policies.restrictions {
                if restriction.permission_type == required.permission_type
                    && restriction.scope == required.scope
                    && required.level > restriction.level
                {
                    return Err(PermissionError::TemplateRequirementConflict {
                        permission_type: required.permission_type,
                        level: required.level,
                    });
                }
            }
        }
        Ok(())
    }

    /// Validates user-requested permissions against organization restrictions.
    ///
    /// This is semantically equivalent to the ceiling half of
    /// [`Self::validate_organization_baseline`] but is a distinct step to
    /// make the evaluation flow explicit.
    fn validate_user_permissions(
        &self,
        requested: &[PermissionGrant],
        policies: &OrganizationPermissionPolicies,
    ) -> Result<(), PermissionError> {
        // Check user-requested permissions against org restrictions (ceiling only).
        // This is an explicit step to make the evaluation flow traceable; it overlaps
        // structurally with the ceiling half of validate_organization_baseline.
        for req in requested {
            for restriction in &policies.restrictions {
                if restriction.permission_type == req.permission_type
                    && restriction.scope == req.scope
                    && req.level > restriction.level
                {
                    return Err(PermissionError::ExceedsOrganizationLimits {
                        permission_type: req.permission_type,
                        level: req.level,
                        maximum_allowed: restriction.level,
                    });
                }
            }
        }
        Ok(())
    }

    /// Assembles the final effective permission list by merging all hierarchy
    /// layers in precedence order.
    ///
    /// Layer merge order (later entries win on duplicate):
    /// 1. Organization baseline requirements (guaranteed minimums)
    /// 2. Repository type required permissions
    /// 3. Template required permissions
    /// 4. User-requested permissions
    ///
    /// Deduplication is by equality (`PartialEq`): the first occurrence of an
    /// identical [`PermissionGrant`] wins; subsequent duplicates are skipped.
    fn merge_permissions(
        &self,
        hierarchy: &PermissionHierarchy,
        requested: &[PermissionGrant],
    ) -> Vec<PermissionGrant> {
        let mut merged: Vec<PermissionGrant> = Vec::new();

        // Layer 1: Organization baseline requirements (guaranteed minimums).
        for perm in &hierarchy.organization_policies.baseline_requirements {
            if !merged.contains(perm) {
                merged.push(perm.clone());
            }
        }

        // Layer 2: Repository type required permissions.
        if let Some(type_perms) = &hierarchy.repository_type_permissions {
            for perm in &type_perms.required_permissions {
                if !merged.contains(perm) {
                    merged.push(perm.clone());
                }
            }
        }

        // Layer 3: Template required permissions.
        if let Some(template) = &hierarchy.template_permissions {
            for perm in &template.required_permissions {
                if !merged.contains(perm) {
                    merged.push(perm.clone());
                }
            }
        }

        // Layer 4: User-requested permissions.
        for perm in requested {
            if !merged.contains(perm) {
                merged.push(perm.clone());
            }
        }

        merged
    }

    /// Checks whether a `permission_type` appears in the list of restricted
    /// types for a repository type configuration.
    fn is_restricted_type(
        &self,
        permission_type: PermissionType,
        type_perms: &RepositoryTypePermissions,
    ) -> bool {
        type_perms.restricted_types.contains(&permission_type)
    }
}
