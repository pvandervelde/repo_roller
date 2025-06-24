//! # Models
//!
//! This module contains the data models used throughout the Merge Warden core.
//!
//! These models represent the core entities that Merge Warden works with, such as
//! pull requests, comments, and labels. They are designed to be serializable and
//! deserializable to facilitate integration with Git provider APIs.

use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

#[derive(Deserialize)]
pub struct Installation {
    pub id: u64,
    pub slug: Option<String>,
    pub client_id: Option<String>,
    pub node_id: String,
    pub name: Option<String>,
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
