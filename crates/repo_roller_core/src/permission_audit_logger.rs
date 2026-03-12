//! Structured audit logging for permission operations.
//!
//! This module provides [`PermissionAuditLogger`], a stateless component that
//! emits structured [`tracing`] events for every significant permission
//! decision. Events use the `"repo_roller_core::permission_audit"` target so they
//! can be independently routed or filtered by the application's tracing
//! subscriber (e.g. written to a separate audit log file in JSON format).
//!
//! ## Events emitted
//!
//! | Method | Level | Outcome field |
//! |--------|-------|---------------|
//! | [`log_policy_evaluation`] (Approved) | INFO | `"approved"` |
//! | [`log_policy_evaluation`] (RequiresApproval) | WARN | `"requires_approval"` |
//! | [`log_policy_denied`] | WARN | `"denied"` |
//! | [`log_permissions_applied`] | INFO | `"applied"` |
//!
//! Each event carries the following structured fields:
//! - `organization` — GitHub organization name
//! - `repository` — repository name
//! - `requestor` — GitHub username of the requester
//! - `emergency_access` — whether emergency-access bypass was requested
//! - Method-specific counters (e.g. `grant_count`, `teams_applied`)
//!
//! ## JSON structured logging
//!
//! Audit events are emitted via `tracing`. To capture them as JSON, configure
//! the application's tracing subscriber with a JSON formatter and filter on
//! target `"repo_roller_core::permission_audit"`:
//!
//! ```text
//! RUST_LOG="repo_roller_core::permission_audit=debug" ./repo_roller_api
//! ```
//!
//! See `docs/spec/design/multi-level-permissions.md` for the full specification.

use tracing::{info, warn};

use crate::permission_manager::ApplyPermissionsResult;
use crate::permissions::{PermissionError, PermissionRequest};
use crate::policy_engine::PermissionEvaluationResult;

#[cfg(test)]
#[path = "permission_audit_logger_tests.rs"]
mod tests;

// ── PermissionAuditLogger ─────────────────────────────────────────────────────

/// Stateless structured audit logger for permission operations.
///
/// Emits [`tracing`] events at each permission decision point so that audit
/// trails can be captured by the application's tracing subscriber without
/// requiring any persistent storage in this component.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permission_audit_logger::PermissionAuditLogger;
///
/// let logger = PermissionAuditLogger::new();
/// // logger is zero-sized; cheap to construct or embed as a field
/// assert!(std::mem::size_of::<PermissionAuditLogger>() == 0);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PermissionAuditLogger;

impl PermissionAuditLogger {
    /// Creates a new `PermissionAuditLogger`.
    ///
    /// The logger is stateless; this is equivalent to
    /// [`Default::default`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_audit_logger::PermissionAuditLogger;
    ///
    /// let logger = PermissionAuditLogger::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Logs the outcome of a [`crate::policy_engine::PolicyEngine`] evaluation.
    ///
    /// Emits an `INFO` event when the request was `Approved` and a `WARN`
    /// event when it `RequiresApproval`. Both events include the request
    /// context (organization, repository, requestor) and the number of
    /// grants produced or restricted.
    ///
    /// # Arguments
    ///
    /// * `request` — the permission request that was evaluated.
    /// * `result` — the evaluation result from the policy engine.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_audit_logger::PermissionAuditLogger;
    /// use repo_roller_core::permissions::{PermissionRequest, RepositoryContext};
    /// use repo_roller_core::policy_engine::PermissionEvaluationResult;
    /// use repo_roller_core::{OrganizationName, RepositoryName};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let logger = PermissionAuditLogger::new();
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
    /// let result = PermissionEvaluationResult::Approved {
    ///     granted_permissions: vec![],
    ///     effective_duration: None,
    /// };
    /// logger.log_policy_evaluation(&request, &result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn log_policy_evaluation(
        &self,
        request: &PermissionRequest,
        result: &PermissionEvaluationResult,
    ) {
        let org = request.repository_context.organization.as_str();
        let repo = request.repository_context.repository.as_str();
        match result {
            PermissionEvaluationResult::Approved {
                granted_permissions,
                ..
            } => {
                info!(
                    target: "repo_roller_core::permission_audit",
                    organization = %org,
                    repository = %repo,
                    requestor = %request.requestor,
                    emergency_access = request.emergency_access,
                    outcome = "approved",
                    grant_count = granted_permissions.len(),
                    "Permission request approved",
                );
            }
            PermissionEvaluationResult::RequiresApproval {
                reason,
                restricted_permissions,
            } => {
                warn!(
                    target: "repo_roller_core::permission_audit",
                    organization = %org,
                    repository = %repo,
                    requestor = %request.requestor,
                    emergency_access = request.emergency_access,
                    outcome = "requires_approval",
                    reason = %reason,
                    restricted_count = restricted_permissions.len(),
                    "Permission request requires approval",
                );
            }
        }
    }

    /// Logs a hard policy denial returned by the
    /// [`crate::policy_engine::PolicyEngine`].
    ///
    /// Emits a `WARN` event with `outcome = "denied"` and the error
    /// description. Call this when `evaluate_permission_request` returns
    /// `Err(PermissionError::*)`.
    ///
    /// # Arguments
    ///
    /// * `request` — the permission request that was denied.
    /// * `error` — the [`PermissionError`] that caused the denial.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_audit_logger::PermissionAuditLogger;
    /// use repo_roller_core::permissions::{
    ///     AccessLevel, PermissionError, PermissionRequest, PermissionType, RepositoryContext,
    /// };
    /// use repo_roller_core::{OrganizationName, RepositoryName};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let logger = PermissionAuditLogger::new();
    /// let request = PermissionRequest {
    ///     duration: None,
    ///     emergency_access: false,
    ///     justification: "Attempted admin".to_string(),
    ///     repository_context: RepositoryContext {
    ///         organization: OrganizationName::new("my-org")?,
    ///         repository: RepositoryName::new("my-repo")?,
    ///     },
    ///     requested_permissions: vec![],
    ///     requestor: "jsmith".to_string(),
    /// };
    /// let error = PermissionError::ExceedsOrganizationLimits {
    ///     permission_type: PermissionType::Admin,
    ///     level: AccessLevel::Admin,
    ///     maximum_allowed: AccessLevel::Maintain,
    /// };
    /// logger.log_policy_denied(&request, &error);
    /// # Ok(())
    /// # }
    /// ```
    pub fn log_policy_denied(&self, request: &PermissionRequest, error: &PermissionError) {
        let org = request.repository_context.organization.as_str();
        let repo = request.repository_context.repository.as_str();
        warn!(
            target: "repo_roller_core::permission_audit",
            organization = %org,
            repository = %repo,
            requestor = %request.requestor,
            emergency_access = request.emergency_access,
            outcome = "denied",
            error = %error,
            "Permission request denied by policy",
        );
    }

    /// Logs the successful completion of GitHub permission application.
    ///
    /// Emits an `INFO` event with `outcome = "applied"` and operation counters
    /// (teams applied/skipped, collaborators applied/removed/skipped, failures).
    /// Call this after [`crate::permission_manager::PermissionManager::apply_repository_permissions`]
    /// completes successfully.
    ///
    /// # Arguments
    ///
    /// * `request` — the permission request that was applied (provides context).
    /// * `result` — the operation result with counts from the manager.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_audit_logger::PermissionAuditLogger;
    /// use repo_roller_core::permission_manager::ApplyPermissionsResult;
    /// use repo_roller_core::permissions::{PermissionRequest, RepositoryContext};
    /// use repo_roller_core::{OrganizationName, RepositoryName};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let logger = PermissionAuditLogger::new();
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
    /// let mut result = ApplyPermissionsResult::new();
    /// result.teams_applied = 2;
    /// result.collaborators_applied = 1;
    /// logger.log_permissions_applied(&request, &result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn log_permissions_applied(
        &self,
        request: &PermissionRequest,
        result: &ApplyPermissionsResult,
    ) {
        let org = request.repository_context.organization.as_str();
        let repo = request.repository_context.repository.as_str();
        info!(
            target: "repo_roller_core::permission_audit",
            organization = %org,
            repository = %repo,
            requestor = %request.requestor,
            emergency_access = request.emergency_access,
            outcome = "applied",
            teams_applied = result.teams_applied,
            teams_skipped = result.teams_skipped,
            collaborators_applied = result.collaborators_applied,
            collaborators_removed = result.collaborators_removed,
            collaborators_skipped = result.collaborators_skipped,
            failed_teams = result.failed_teams.len(),
            failed_collaborators = result.failed_collaborators.len(),
            "Repository permissions applied",
        );
    }
}
