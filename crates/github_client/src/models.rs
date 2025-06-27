//! # Models
//!
//! This module contains the data models used throughout the Merge Warden core.
//!
//! These models represent the core entities that Merge Warden works with, such as
//! pull requests, comments, and labels. They are designed to be serializable and
//! deserializable to facilitate integration with Git provider APIs.

use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

/// Represents a GitHub account (user or organization).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    /// The unique ID of the account
    pub id: u64,
    /// The login name of the account
    pub login: String,
    /// The type of account (User or Organization)
    #[serde(rename = "type")]
    pub account_type: String,
    /// The node ID for GraphQL operations
    pub node_id: String,
}

// Remove the Account conversion since octocrab::models::Account might not be available
// We'll construct Account directly in the Installation conversion above

/// Represents a GitHub App installation.
///
/// This struct contains information about where a GitHub App is installed,
/// such as an organization or user account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Installation {
    /// The unique ID of the installation
    pub id: u64,
    /// The account (user or organization) where the app is installed
    pub account: Account,
    /// Optional repository selection details
    pub repository_selection: Option<String>,
    /// The node ID for GraphQL operations
    pub node_id: String,
}

impl From<octocrab::models::Installation> for Installation {
    fn from(value: octocrab::models::Installation) -> Self {
        let account_node_id = value.account.node_id.clone();
        Self {
            id: *value.id,
            account: Account {
                id: *value.account.id,
                login: value.account.login,
                account_type: value.account.r#type,
                node_id: value.account.node_id,
            },
            repository_selection: value.repository_selection,
            node_id: account_node_id, // Use account's node_id since installation doesn't have one
        }
    }
}

/// Represents a label on a pull request.
///
/// This struct contains the essential information about a label
/// that is needed for categorization and filtering.
///
/// # Fields
///
/// * `name` - The name of the label
///
/// # Examples
///
/// ```
/// use merge_warden_developer_platforms::models::Label;
///
/// let label = Label {
///     name: "bug".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The name of the label
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// The name of the organization
    pub name: String,
}

#[derive(Deserialize)]
pub struct Repository {
    full_name: String,
    name: String,
    node_id: String,
    private: bool,
}
impl Repository {
    pub fn is_private(&self) -> bool {
        self.private
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new(name: String, full_name: String, node_id: String, private: bool) -> Self {
        Self {
            full_name,
            name,
            node_id,
            private,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    pub fn url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}.git", self.full_name))
            .expect("Valid GitHub repository URL")
    }
}

impl From<octocrab::models::Repository> for Repository {
    fn from(value: octocrab::models::Repository) -> Self {
        Self {
            name: value.name.clone(),
            full_name: value.full_name.unwrap_or(value.name.clone()),
            node_id: value.node_id.unwrap_or_default(),
            private: value.private.unwrap_or(false),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub login: String,
}
