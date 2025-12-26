//! GitHub App installation domain types.
//!
//! This module contains types related to GitHub App installations,
//! including installation details and account information.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "installation_tests.rs"]
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
