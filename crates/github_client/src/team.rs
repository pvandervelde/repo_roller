//! GitHub team domain types.
//!
//! This module contains types representing GitHub organization teams and their
//! members, used for managing repository access permissions via the GitHub Teams API.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "team_tests.rs"]
mod tests;

/// Represents a GitHub organization team.
///
/// Teams are groups of organization members that can be assigned permissions
/// to repositories. Permission assignments apply to all team members.
///
/// # Examples
///
/// ```rust
/// use github_client::Team;
///
/// let team = Team {
///     id: 12345,
///     slug: "backend-engineers".to_string(),
///     name: "Backend Engineers".to_string(),
///     description: Some("The backend engineering team".to_string()),
/// };
///
/// assert_eq!(team.slug, "backend-engineers");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    /// The GitHub-assigned numeric team ID.
    pub id: u64,
    /// The URL-safe team slug used in API requests.
    pub slug: String,
    /// The human-readable team name.
    pub name: String,
    /// Optional description of the team's purpose.
    pub description: Option<String>,
}

/// Represents a member of a GitHub organization team.
///
/// Members are individual GitHub user accounts that belong to a team.
///
/// # Examples
///
/// ```rust
/// use github_client::TeamMember;
///
/// let member = TeamMember {
///     id: 98765,
///     login: "jsmith".to_string(),
/// };
///
/// assert_eq!(member.login, "jsmith");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamMember {
    /// The GitHub-assigned numeric user ID.
    pub id: u64,
    /// The GitHub username (login handle).
    pub login: String,
}
