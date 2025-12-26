//! User domain types.
//!
//! This module contains types representing GitHub user accounts.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "user_tests.rs"]
mod tests;

/// Represents a GitHub user account.
///
/// This struct contains basic information about a GitHub user, including
/// their unique ID and login name. It's used throughout the API for
/// representing repository owners, collaborators, and other user references.
///
/// # Examples
///
/// ```rust
/// use github_client::User;
///
/// let user = User {
///     id: 12345,
///     login: "octocat".to_string(),
/// };
///
/// println!("User: {} (ID: {})", user.login, user.id);
/// ```
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    /// The unique numeric ID of the user
    pub id: u64,
    /// The login name of the user
    pub login: String,
}
