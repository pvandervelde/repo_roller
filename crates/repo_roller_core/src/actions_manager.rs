//! GitHub Actions settings application for repositories.
//!
//! This module provides the [`ActionsManager`] component for applying the
//! subset of [`config_manager::settings::ActionSettings`] that maps onto
//! GitHub's Actions permissions REST endpoints
//! (`set_actions_workflow_permissions`, `set_actions_access_level`).
//!
//! Only the 3 fields introduced by #276/#277
//! (`default_workflow_permissions`, `can_approve_pull_request_reviews`,
//! `access_level`) are mapped to an API call here. The 5 pre-existing
//! `ActionSettings` fields (`enabled`, `allowed_actions`,
//! `github_owned_allowed`, `verified_allowed`, `patterns_allowed`) are
//! explicitly out of scope for this manager.

use config_manager::settings::ActionSettings;
use github_client::{ActionsAccessLevelUpdate, ActionsWorkflowPermissionsUpdate, RepositoryClient};
use std::sync::Arc;

use crate::{GitHubError, RepoRollerError, RepoRollerResult};

/// Applies GitHub Actions settings to a repository.
///
/// This manager orchestrates the two GitHub Actions permissions REST calls
/// (`set_actions_workflow_permissions` and `set_actions_access_level`),
/// building the appropriate request payload from the merged
/// [`ActionSettings`] and skipping any call whose underlying fields are
/// unset.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::{GitHubClient, RepositoryClient};
/// use repo_roller_core::ActionsManager;
/// use config_manager::settings::ActionSettings;
/// use std::sync::Arc;
///
/// # async fn example(github_client: GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
/// let manager = ActionsManager::new(Arc::new(github_client) as Arc<dyn RepositoryClient>);
///
/// let actions = ActionSettings::default();
/// manager.apply_action_settings("my-org", "my-repo", &actions).await?;
/// # Ok(())
/// # }
/// ```
pub struct ActionsManager {
    /// GitHub client for API operations
    github_client: Arc<dyn RepositoryClient>,
}

impl ActionsManager {
    /// Creates a new ActionsManager.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API operations
    pub fn new(github_client: Arc<dyn RepositoryClient>) -> Self {
        Self { github_client }
    }

    /// Applies the mapped subset of `ActionSettings` to a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `actions` - The merged `ActionSettings` to apply
    ///
    /// # Returns
    ///
    /// `Ok(())` once every applicable call has succeeded (or no call was
    /// applicable at all).
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::GitHub(GitHubError::NetworkError)` if
    /// either underlying GitHub API call fails. The calls are made
    /// sequentially and abort on the first failure: if
    /// `default_workflow_permissions` and/or
    /// `can_approve_pull_request_reviews` is set and that call fails, the
    /// `access_level` call (even if also set) is never attempted.
    ///
    /// # Behavior
    ///
    /// 1. If `default_workflow_permissions` or
    ///    `can_approve_pull_request_reviews` is set, calls
    ///    `set_actions_workflow_permissions` with an
    ///    [`ActionsWorkflowPermissionsUpdate`] built from whichever of the
    ///    two fields are present.
    /// 2. If `access_level` is set, calls `set_actions_access_level` with
    ///    an [`ActionsAccessLevelUpdate`] carrying that value.
    /// 3. The 5 pre-existing `ActionSettings` fields (`enabled`,
    ///    `allowed_actions`, `github_owned_allowed`, `verified_allowed`,
    ///    `patterns_allowed`) never trigger any API call.
    pub async fn apply_action_settings(
        &self,
        owner: &str,
        repo: &str,
        actions: &ActionSettings,
    ) -> RepoRollerResult<()> {
        if actions.default_workflow_permissions.is_some()
            || actions.can_approve_pull_request_reviews.is_some()
        {
            let payload = ActionsWorkflowPermissionsUpdate {
                default_workflow_permissions: actions
                    .default_workflow_permissions
                    .as_ref()
                    .map(|v| v.value.clone()),
                can_approve_pull_request_reviews: actions
                    .can_approve_pull_request_reviews
                    .as_ref()
                    .map(|v| v.value),
            };

            self.github_client
                .set_actions_workflow_permissions(owner, repo, &payload)
                .await
                .map_err(|e| {
                    RepoRollerError::GitHub(GitHubError::NetworkError {
                        reason: format!(
                            "Failed to set Actions workflow permissions for {owner}/{repo}: {e}"
                        ),
                    })
                })?;
        }

        if let Some(access_level) = &actions.access_level {
            let payload = ActionsAccessLevelUpdate {
                access_level: access_level.value.clone(),
            };

            self.github_client
                .set_actions_access_level(owner, repo, &payload)
                .await
                .map_err(|e| {
                    RepoRollerError::GitHub(GitHubError::NetworkError {
                        reason: format!(
                            "Failed to set Actions access level for {owner}/{repo}: {e}"
                        ),
                    })
                })?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "actions_manager_tests.rs"]
mod tests;
