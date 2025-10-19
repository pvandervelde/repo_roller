//! Branch protection settings.
//!
//! Controls branch protection rules and policies.

use crate::OverridableValue;
use serde::{Deserialize, Serialize};

/// Branch protection settings with override controls.
///
/// Configures branch protection rules for the default branch and other protected branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BranchProtectionSettings {
    /// Default branch name
    pub default_branch: Option<OverridableValue<String>>,

    /// Require pull request reviews before merging
    pub require_pull_request_reviews: Option<OverridableValue<bool>>,

    /// Required number of approving reviews
    pub required_approving_review_count: Option<OverridableValue<i32>>,

    /// Dismiss stale reviews when new commits are pushed
    pub dismiss_stale_reviews: Option<OverridableValue<bool>>,

    /// Require review from code owners
    pub require_code_owner_reviews: Option<OverridableValue<bool>>,

    /// Require status checks to pass before merging
    pub require_status_checks: Option<OverridableValue<bool>>,

    /// Required status checks (list of check names)
    pub required_status_checks_list: Option<Vec<String>>,

    /// Require branches to be up to date before merging
    pub strict_required_status_checks: Option<OverridableValue<bool>>,

    /// Restrict who can push to matching branches
    pub restrict_pushes: Option<OverridableValue<bool>>,

    /// Allow force pushes
    pub allow_force_pushes: Option<OverridableValue<bool>>,

    /// Allow deletions
    pub allow_deletions: Option<OverridableValue<bool>>,

    /// Additional protected branch patterns
    pub additional_protected_patterns: Option<Vec<String>>,
}

#[cfg(test)]
#[path = "branch_protection_tests.rs"]
mod tests;
