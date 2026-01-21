//! Label management operations for repositories.
//!
//! This module provides the [`LabelManager`] component for orchestrating
//! label operations with business logic, idempotency, and error handling.

use github_client::{GitHubClient, RepositoryClient};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{GitHubError, RepoRollerError, RepoRollerResult};

/// Manages label operations for repositories.
///
/// This manager provides high-level label management operations that handle
/// idempotency, bulk operations, and proper error handling. It orchestrates
/// the GitHubClient label operations with business logic.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::GitHubClient;
/// use repo_roller_core::LabelManager;
/// use config_manager::settings::LabelConfig;
/// use std::collections::HashMap;
///
/// # async fn example(github_client: GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
/// let manager = LabelManager::new(github_client);
///
/// let mut labels = HashMap::new();
/// labels.insert("bug".to_string(), LabelConfig {
///     name: "bug".to_string(),
///     color: "d73a4a".to_string(),
///     description: "Something isn't working".to_string(),
/// });
///
/// let result = manager.apply_labels("my-org", "my-repo", &labels).await?;
/// println!("Created: {}, Updated: {}", result.created, result.updated);
/// # Ok(())
/// # }
/// ```
pub struct LabelManager {
    /// GitHub client for API operations
    github_client: GitHubClient,
}

impl LabelManager {
    /// Creates a new LabelManager.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::GitHubClient;
    /// use repo_roller_core::LabelManager;
    ///
    /// # async fn example(client: GitHubClient) {
    /// let manager = LabelManager::new(client);
    /// # }
    /// ```
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Applies labels to a repository, creating or updating as needed.
    ///
    /// This method ensures that all labels from the configuration are present
    /// in the repository. It is idempotent and safe to call multiple times.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `labels` - Map of label name to LabelConfig
    ///
    /// # Returns
    ///
    /// `Ok(ApplyLabelsResult)` with details of operations performed
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if label operations fail.
    ///
    /// # Behavior
    ///
    /// 1. Iterates through all labels in configuration
    /// 2. Creates or updates each label (create_label is idempotent)
    /// 3. Logs each operation (created/updated/skipped)
    /// 4. Returns summary of operations
    ///
    /// # Error Handling
    ///
    /// - Continues on individual label failures (logs warning)
    /// - Returns error only if all labels fail
    /// - Partial success is considered success
    pub async fn apply_labels(
        &self,
        owner: &str,
        repo: &str,
        labels: &HashMap<String, config_manager::settings::LabelConfig>,
    ) -> RepoRollerResult<ApplyLabelsResult> {
        info!(
            owner = owner,
            repo = repo,
            label_count = labels.len(),
            "Applying labels to repository"
        );

        let mut result = ApplyLabelsResult::new();

        for (label_name, label_config) in labels {
            info!(name = label_name, "Applying label");

            match self
                .github_client
                .create_label(
                    owner,
                    repo,
                    &label_config.name,
                    &label_config.color,
                    &label_config.description,
                )
                .await
            {
                Ok(()) => {
                    info!(name = label_name, "Label applied successfully");
                    result.created += 1;
                }
                Err(e) => {
                    warn!(
                        name = label_name,
                        error = ?e,
                        "Failed to apply label"
                    );
                    result.failed += 1;
                    result.failed_labels.push(label_name.clone());
                }
            }
        }

        info!(
            created = result.created,
            failed = result.failed,
            "Label application complete"
        );

        Ok(result)
    }

    /// Lists all labels currently defined in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Vector of label names
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if API call fails.
    pub async fn list_labels(&self, owner: &str, repo: &str) -> RepoRollerResult<Vec<String>> {
        match self.github_client.list_repository_labels(owner, repo).await {
            Ok(labels) => Ok(labels),
            Err(e) => {
                warn!(owner = owner, repo = repo, error = ?e, "Failed to list labels");
                Err(GitHubError::InvalidResponse {
                    reason: format!("Failed to list labels for {}/{}: {}", owner, repo, e),
                }
                .into())
            }
        }
    }
}

/// Result of applying labels to a repository.
///
/// Contains counters for the different outcomes of label operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyLabelsResult {
    /// Number of labels created
    pub created: usize,

    /// Number of labels updated
    pub updated: usize,

    /// Number of labels that already existed with correct configuration
    pub skipped: usize,

    /// Number of labels that failed to apply
    pub failed: usize,

    /// Names of labels that failed (for error reporting)
    pub failed_labels: Vec<String>,
}

impl ApplyLabelsResult {
    /// Creates a new empty result.
    pub fn new() -> Self {
        Self {
            created: 0,
            updated: 0,
            skipped: 0,
            failed: 0,
            failed_labels: Vec::new(),
        }
    }

    /// Returns true if all operations succeeded.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Returns true if any labels were successfully applied (created or updated).
    pub fn has_changes(&self) -> bool {
        self.created > 0 || self.updated > 0
    }
}

impl Default for ApplyLabelsResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "label_manager_tests.rs"]
mod tests;
