//! Pull request configuration settings.
//!
//! Controls how pull requests work in repositories.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// Pull request settings with override controls.
///
/// Configures pull request behavior including merge strategies, review requirements,
/// and commit message formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PullRequestSettings {
    /// Allow merge commits
    pub allow_merge_commit: Option<OverridableValue<bool>>,

    /// Allow squash merging
    pub allow_squash_merge: Option<OverridableValue<bool>>,

    /// Allow rebase merging
    pub allow_rebase_merge: Option<OverridableValue<bool>>,

    /// Delete head branch after merge
    pub delete_branch_on_merge: Option<OverridableValue<bool>>,

    /// Number of required approving reviews
    pub required_approving_review_count: Option<OverridableValue<i32>>,

    /// Require code owner reviews
    pub require_code_owner_reviews: Option<OverridableValue<bool>>,

    /// Require conversation resolution before merging
    pub require_conversation_resolution: Option<OverridableValue<bool>>,

    /// Allow auto-merge
    pub allow_auto_merge: Option<OverridableValue<bool>>,

    /// Merge commit title template (PR_TITLE or MERGE_MESSAGE)
    pub merge_commit_title: Option<OverridableValue<String>>,

    /// Merge commit message template (PR_BODY, COMMIT_MESSAGES, or BLANK)
    pub merge_commit_message: Option<OverridableValue<String>>,

    /// Squash merge commit title template
    pub squash_merge_commit_title: Option<OverridableValue<String>>,

    /// Squash merge commit message template
    pub squash_merge_commit_message: Option<OverridableValue<String>>,
}

#[cfg(test)]
#[path = "pull_request_tests.rs"]
mod tests;
