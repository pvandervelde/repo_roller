//! GitHub repository collaborator domain types.
//!
//! This module contains types representing repository collaborators — individual
//! GitHub users granted direct access to a repository outside of team membership.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "collaborator_tests.rs"]
mod tests;

/// Represents a repository collaborator — a GitHub user with direct repository access.
///
/// Collaborators are individuals granted explicit access to a repository,
/// distinct from team-based access. The permission level is managed separately
/// via [`GitHubClient::add_repository_collaborator`] /
/// [`GitHubClient::set_collaborator_permission`].
///
/// # Examples
///
/// ```rust
/// use github_client::Collaborator;
///
/// let collaborator = Collaborator {
///     id: 98765,
///     login: "jsmith".to_string(),
/// };
///
/// assert_eq!(collaborator.login, "jsmith");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collaborator {
    /// The GitHub-assigned numeric user ID.
    pub id: u64,
    /// The GitHub username (login handle).
    pub login: String,
}
