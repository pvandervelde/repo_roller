//! Branch protection domain types.
//!
//! This module contains types representing GitHub branch protection rules.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "branch_protection_tests.rs"]
mod tests;

/// Represents branch protection rules for a repository branch.
///
/// This struct contains the essential branch protection settings that can be
/// verified during integration testing to ensure configuration was applied correctly.
///
/// # Examples
///
/// ```rust
/// use github_client::BranchProtection;
///
/// let protection = BranchProtection {
///     required_approving_review_count: Some(2),
///     require_code_owner_reviews: Some(true),
///     dismiss_stale_reviews: Some(true),
/// };
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BranchProtection {
    /// Required number of approving reviews before merging
    pub required_approving_review_count: Option<u32>,
    /// Whether code owner reviews are required
    pub require_code_owner_reviews: Option<bool>,
    /// Whether stale reviews are dismissed when new commits are pushed
    pub dismiss_stale_reviews: Option<bool>,
}
