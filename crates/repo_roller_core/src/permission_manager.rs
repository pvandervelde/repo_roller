//! Permission management orchestration for repositories.
//!
//! This module provides the [`PermissionManager`] component for orchestrating
//! repository permission operations: evaluating permission requests through the
//! [`PolicyEngine`] and applying approved permissions to GitHub via the
//! [`github_client::GitHubClient`].
//!
//! ## Workflow
//!
//! ```text
//! apply_repository_permissions()
//!   1. Evaluate request via PolicyEngine (pure, no I/O)
//!   2. If RequiresApproval → return PermissionManagerError::RequiresApproval
//!   3. Apply team permissions via GitHubClient  (idempotent upsert)
//!   4. Apply collaborator permissions via GitHubClient (idempotent PUT for
//!      non-None; membership check + remove for AccessLevel::None)
//! ```
//!
//! ## Idempotency
//!
//! - **Teams**: `add_team_to_repository` uses a PUT endpoint on GitHub — safe to call
//!   multiple times. `AccessLevel::None` on teams is logged as a warning and skipped
//!   (a remove-team API endpoint is not yet exposed by `GitHubClient`).
//! - **Collaborators**: The GitHub `PUT /repos/{owner}/{repo}/collaborators/{username}`
//!   endpoint is idempotent — it adds the collaborator when they are new and updates
//!   their permission level when they already exist. The manager always calls this
//!   endpoint (for any non-`None` level) so that permission changes are applied on
//!   subsequent idempotent runs. When `AccessLevel::None` is requested the existing
//!   collaborator list is fetched (one API call) to decide whether to remove or skip;
//!   when no `None` entries are present the list fetch is skipped entirely.
//!
//! See `docs/spec/design/multi-level-permissions.md` for the full specification.

use std::collections::HashMap;

use github_client::GitHubClient;
use tracing::{info, instrument, warn};

use crate::permission_audit_logger::PermissionAuditLogger;
use crate::permissions::{
    AccessLevel, GitHubPermissionLevel, PermissionGrant, PermissionHierarchy, PermissionRequest,
};
use crate::policy_engine::{PermissionEvaluationResult, PolicyEngine};

pub use crate::permissions::PermissionError;

#[cfg(test)]
#[path = "permission_manager_tests.rs"]
mod tests;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur during permission management operations.
#[derive(thiserror::Error, Debug)]
pub enum PermissionManagerError {
    /// An unexpected GitHub API error occurred while applying permissions.
    #[error("GitHub API error: {0}")]
    GitHubError(String),

    /// The [`PolicyEngine`] rejected the permission request.
    ///
    /// See [`PermissionError`] variants for specific rejection reasons.
    #[error("Permission policy denied: {0}")]
    PolicyDenied(#[from] PermissionError),

    /// The permission request requires human approval before it can be applied.
    ///
    /// This occurs when `PermissionRequest::emergency_access` is `true` or when
    /// the [`PolicyEngine`] determines manual approval is required.
    #[error("Permission request requires approval: {reason}")]
    RequiresApproval {
        /// Human-readable reason for requiring approval.
        reason: String,
        /// The permissions that triggered the approval requirement.
        restricted_permissions: Vec<PermissionGrant>,
    },
}

// ── Result type ───────────────────────────────────────────────────────────────

/// Summary of the outcome of an `apply_repository_permissions` operation.
///
/// Counters describe what was done for teams and collaborators separately.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::permission_manager::ApplyPermissionsResult;
///
/// let mut result = ApplyPermissionsResult::new();
/// result.teams_applied = 2;
/// result.collaborators_applied = 1;
///
/// assert!(result.is_success());
/// assert!(result.has_changes());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPermissionsResult {
    /// Number of team permissions successfully applied.
    pub teams_applied: u32,
    /// Number of team permissions skipped (e.g. already present or `None` access).
    pub teams_skipped: u32,
    /// Number of collaborator permissions successfully applied (added).
    pub collaborators_applied: u32,
    /// Number of collaborators successfully removed.
    pub collaborators_removed: u32,
    /// Number of collaborator operations skipped (`None` requested for a non-member, or
    /// `None` team access level).
    pub collaborators_skipped: u32,
    /// Team slugs whose permission application failed.
    pub failed_teams: Vec<String>,
    /// Usernames whose permission application or removal failed.
    pub failed_collaborators: Vec<String>,
}

impl ApplyPermissionsResult {
    /// Creates an empty result (all counters zero, no failures).
    pub fn new() -> Self {
        Self {
            teams_applied: 0,
            teams_skipped: 0,
            collaborators_applied: 0,
            collaborators_removed: 0,
            collaborators_skipped: 0,
            failed_teams: Vec::new(),
            failed_collaborators: Vec::new(),
        }
    }

    /// Returns `true` when no operations failed.
    ///
    /// Note: skipped operations are not considered failures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_manager::ApplyPermissionsResult;
    ///
    /// let result = ApplyPermissionsResult::new();
    /// assert!(result.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.failed_teams.is_empty() && self.failed_collaborators.is_empty()
    }

    /// Returns `true` when at least one permission was applied or removed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use repo_roller_core::permission_manager::ApplyPermissionsResult;
    ///
    /// let mut result = ApplyPermissionsResult::new();
    /// result.teams_applied = 1;
    /// assert!(result.has_changes());
    /// ```
    pub fn has_changes(&self) -> bool {
        self.teams_applied > 0 || self.collaborators_applied > 0 || self.collaborators_removed > 0
    }
}

impl Default for ApplyPermissionsResult {
    fn default() -> Self {
        Self::new()
    }
}

// ── Manager ───────────────────────────────────────────────────────────────────

/// Orchestrates repository permission operations.
///
/// `PermissionManager` evaluates permission requests through the [`PolicyEngine`]
/// and applies approved permissions to GitHub repositories via [`GitHubClient`].
///
/// ## Idempotency
///
/// - **Teams**: Applied via PUT (idempotent at the GitHub API level).
///   `AccessLevel::None` is skipped with a warning.
/// - **Collaborators**: Applied via idempotent PUT for non-`None` levels.
///   `AccessLevel::None` triggers removal when the collaborator is present;
///   non-present entries are skipped. The existing-collaborator list is fetched
///   only when at least one `None` entry is present.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::GitHubClient;
/// use repo_roller_core::permission_manager::PermissionManager;
/// use repo_roller_core::policy_engine::PolicyEngine;
///
/// # async fn example(client: GitHubClient) {
/// let manager = PermissionManager::new(client, PolicyEngine::new());
/// # }
/// ```
pub struct PermissionManager {
    /// Structured audit logger for recording permission decisions.
    audit_logger: PermissionAuditLogger,
    /// GitHub client for applying permissions via the GitHub API.
    github_client: GitHubClient,
    /// Policy engine for validating permission requests against the hierarchy.
    policy_engine: PolicyEngine,
}

impl PermissionManager {
    /// Creates a new `PermissionManager`.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API operations.
    /// * `policy_engine` - Policy engine for permission evaluation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::GitHubClient;
    /// use repo_roller_core::permission_manager::PermissionManager;
    /// use repo_roller_core::policy_engine::PolicyEngine;
    ///
    /// # async fn example(client: GitHubClient) {
    /// let manager = PermissionManager::new(client, PolicyEngine::new());
    /// # }
    /// ```
    pub fn new(github_client: GitHubClient, policy_engine: PolicyEngine) -> Self {
        Self {
            audit_logger: PermissionAuditLogger::new(),
            github_client,
            policy_engine,
        }
    }

    /// Applies approved permissions to a repository.
    ///
    /// Evaluates the `request` against the `hierarchy` via the [`PolicyEngine`].
    /// When the evaluation is [`PermissionEvaluationResult::Approved`], applies
    /// team and collaborator permissions to the repository via the GitHub API.
    ///
    /// ## Evaluation
    ///
    /// The `request.requested_permissions` are evaluated against the `hierarchy`
    /// for policy compliance. The `teams` and `collaborators` maps specify the
    /// concrete entities and desired access levels to apply after approval.
    ///
    /// ## Idempotency
    ///
    /// Team permissions use a PUT-based upsert — calling the endpoint
    /// multiple times is safe. Collaborator permissions also use PUT (idempotent),
    /// so existing collaborators are always updated to the requested level rather
    /// than skipped. The existing-member list (`list_repository_collaborators`) is
    /// fetched only when at least one collaborator entry carries
    /// [`AccessLevel::None`], since that is the only case where membership needs
    /// to be checked before deciding to remove or skip.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (GitHub organization or user login).
    /// * `repo` - Repository name.
    /// * `request` - Permission request to evaluate and apply.
    /// * `hierarchy` - Fully assembled permission hierarchy for evaluation.
    /// * `teams` - Map of team slug → desired access level.
    /// * `collaborators` - Map of GitHub username → desired access level.
    ///   Use [`AccessLevel::None`] to remove a collaborator.
    ///
    /// # Returns
    ///
    /// `Ok(ApplyPermissionsResult)` with operation counts on success.
    ///
    /// # Errors
    ///
    /// - [`PermissionManagerError::PolicyDenied`] — policy engine rejected the request.
    /// - [`PermissionManagerError::RequiresApproval`] — request requires human approval.
    /// - [`PermissionManagerError::GitHubError`] — GitHub API error while listing
    ///   collaborators for `AccessLevel::None` checks (team and per-collaborator
    ///   failures are counted in the result, not returned as errors).
    #[instrument(skip(self, request, hierarchy, teams, collaborators), fields(owner = %owner, repo = %repo))]
    pub async fn apply_repository_permissions(
        &self,
        owner: &str,
        repo: &str,
        request: &PermissionRequest,
        hierarchy: &PermissionHierarchy,
        teams: &HashMap<String, AccessLevel>,
        collaborators: &HashMap<String, AccessLevel>,
    ) -> Result<ApplyPermissionsResult, PermissionManagerError> {
        info!(
            owner = owner,
            repo = repo,
            team_count = teams.len(),
            collaborator_count = collaborators.len(),
            "Applying repository permissions"
        );

        // ── Step 1: Policy evaluation ─────────────────────────────────────────
        let eval_result = match self
            .policy_engine
            .evaluate_permission_request(request, hierarchy)
        {
            Ok(r) => r,
            Err(e) => {
                self.audit_logger.log_policy_denied(request, &e);
                return Err(PermissionManagerError::PolicyDenied(e));
            }
        };

        self.audit_logger
            .log_policy_evaluation(request, &eval_result);

        if let PermissionEvaluationResult::RequiresApproval {
            reason,
            restricted_permissions,
        } = eval_result
        {
            warn!(
                owner = owner,
                repo = repo,
                reason = %reason,
                "Permission request requires approval; aborting apply"
            );
            return Err(PermissionManagerError::RequiresApproval {
                reason,
                restricted_permissions,
            });
        }

        info!(owner = owner, repo = repo, "PolicyEngine approved request");

        let mut result = ApplyPermissionsResult::new();

        // ── Step 2: Apply team permissions ────────────────────────────────────
        for (team_slug, access_level) in teams {
            if *access_level == AccessLevel::None {
                warn!(
                    owner = owner,
                    repo = repo,
                    team = team_slug.as_str(),
                    "Team AccessLevel::None requested but team removal is not yet supported; skipping"
                );
                result.teams_skipped += 1;
                continue;
            }

            let permission_str = GitHubPermissionLevel::from(*access_level)
                .as_str()
                .to_string();

            info!(
                owner = owner,
                repo = repo,
                team = team_slug.as_str(),
                permission = permission_str.as_str(),
                "Applying team permission"
            );

            match self
                .github_client
                .add_team_to_repository(owner, team_slug, repo, &permission_str)
                .await
            {
                Ok(()) => {
                    info!(
                        owner = owner,
                        repo = repo,
                        team = team_slug.as_str(),
                        permission = permission_str.as_str(),
                        "Team permission applied"
                    );
                    result.teams_applied += 1;
                }
                Err(e) => {
                    warn!(
                        owner = owner,
                        repo = repo,
                        team = team_slug.as_str(),
                        error = ?e,
                        "Failed to apply team permission"
                    );
                    result.failed_teams.push(team_slug.clone());
                }
            }
        }

        // ── Step 3: Apply collaborator permissions ────────────────────────────
        // The existing-collaborator list is only needed to decide whether an
        // `AccessLevel::None` request should trigger a removal (collaborator is
        // present) or be silently skipped (collaborator is absent).
        // The PUT endpoint used for non-None levels is idempotent and does not
        // require a membership check, so we skip the API call entirely when no
        // collaborator entry has `AccessLevel::None`.
        let has_none_entry = collaborators.values().any(|l| *l == AccessLevel::None);
        let existing_logins: Vec<String> = if has_none_entry {
            self.github_client
                .list_repository_collaborators(owner, repo)
                .await
                .map_err(|e| PermissionManagerError::GitHubError(e.to_string()))?
                .into_iter()
                .map(|c| c.login)
                .collect()
        } else {
            Vec::new()
        };

        for (username, access_level) in collaborators {
            let is_existing = existing_logins.contains(username);

            if *access_level == AccessLevel::None {
                if is_existing {
                    info!(
                        owner = owner,
                        repo = repo,
                        username = username.as_str(),
                        "Removing collaborator (AccessLevel::None)"
                    );
                    match self
                        .github_client
                        .remove_repository_collaborator(owner, repo, username)
                        .await
                    {
                        Ok(()) => {
                            info!(
                                owner = owner,
                                repo = repo,
                                username = username.as_str(),
                                "Collaborator removed"
                            );
                            result.collaborators_removed += 1;
                        }
                        Err(e) => {
                            warn!(
                                owner = owner,
                                repo = repo,
                                username = username.as_str(),
                                error = ?e,
                                "Failed to remove collaborator"
                            );
                            result.failed_collaborators.push(username.clone());
                        }
                    }
                } else {
                    info!(
                        owner = owner,
                        repo = repo,
                        username = username.as_str(),
                        "Collaborator not present, skipping removal"
                    );
                    result.collaborators_skipped += 1;
                }
                continue;
            }

            let permission_str = GitHubPermissionLevel::from(*access_level)
                .as_str()
                .to_string();

            // The GitHub PUT endpoint is idempotent: it adds the collaborator when
            // they are new and updates their permission level when they already
            // exist. Always call it so that permission upgrades are applied on
            // subsequent idempotent runs (e.g. re-applying config to a repo that
            // already has collaborators at a lower level).
            let action = if is_existing { "Updating" } else { "Adding" };
            info!(
                owner = owner,
                repo = repo,
                username = username.as_str(),
                permission = permission_str.as_str(),
                "{} collaborator permission",
                action
            );

            match self
                .github_client
                .add_repository_collaborator(owner, repo, username, &permission_str)
                .await
            {
                Ok(()) => {
                    info!(
                        owner = owner,
                        repo = repo,
                        username = username.as_str(),
                        permission = permission_str.as_str(),
                        "Collaborator permission applied"
                    );
                    result.collaborators_applied += 1;
                }
                Err(e) => {
                    warn!(
                        owner = owner,
                        repo = repo,
                        username = username.as_str(),
                        error = ?e,
                        "Failed to apply collaborator permission"
                    );
                    result.failed_collaborators.push(username.clone());
                }
            }
        }

        info!(
            owner = owner,
            repo = repo,
            teams_applied = result.teams_applied,
            teams_skipped = result.teams_skipped,
            collaborators_applied = result.collaborators_applied,
            collaborators_removed = result.collaborators_removed,
            collaborators_skipped = result.collaborators_skipped,
            failed_teams = result.failed_teams.len(),
            failed_collaborators = result.failed_collaborators.len(),
            "Permission application complete"
        );

        self.audit_logger.log_permissions_applied(request, &result);

        Ok(result)
    }
}
