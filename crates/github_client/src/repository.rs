//! Repository domain types.
//!
//! This module contains types representing GitHub repositories and organizations.

use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;

/// Represents a GitHub organization.
///
/// This struct contains basic information about a GitHub organization,
/// primarily used for organization-related API operations and queries.
///
/// # Examples
///
/// ```rust
/// use github_client::Organization;
///
/// let org = Organization {
///     name: "my-organization".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// The name of the organization
    pub name: String,
}

/// Represents a GitHub repository.
///
/// This struct contains essential information about a GitHub repository,
/// including its name, visibility, and identifiers. It provides methods
/// for accessing repository properties and generating URLs.
///
/// # Examples
///
/// ```rust
/// use github_client::Repository;
///
/// let repo = Repository::new(
///     "my-repo".to_string(),
///     "owner/my-repo".to_string(),
///     "MDEwOlJlcG9zaXRvcnkx".to_string(),
///     false
/// );
///
/// println!("Repository: {}", repo.name());
/// println!("Is private: {}", repo.is_private());
/// println!("Clone URL: {}", repo.url());
/// ```
#[derive(Deserialize)]
pub struct Repository {
    /// The full name of the repository (owner/name)
    full_name: String,
    /// The name of the repository
    name: String,
    /// The GraphQL node ID of the repository
    node_id: String,
    /// Whether the repository is private
    private: bool,
    /// Whether issues are enabled for this repository
    has_issues: Option<bool>,
    /// Whether the wiki is enabled for this repository
    has_wiki: Option<bool>,
    /// Whether projects are enabled for this repository
    has_projects: Option<bool>,
    /// Whether discussions are enabled for this repository
    has_discussions: Option<bool>,
}

impl Repository {
    /// Returns whether the repository is private.
    ///
    /// # Returns
    ///
    /// `true` if the repository is private, `false` if it's public.
    pub fn is_private(&self) -> bool {
        self.private
    }

    /// Returns the name of the repository.
    ///
    /// # Returns
    ///
    /// A string slice containing the repository name (without owner).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Creates a new Repository instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the repository
    /// * `full_name` - The full name including owner (owner/repo)
    /// * `node_id` - The GraphQL node ID
    /// * `private` - Whether the repository is private
    ///
    /// # Returns
    ///
    /// A new `Repository` instance with the provided values.
    pub fn new(name: String, full_name: String, node_id: String, private: bool) -> Self {
        Self {
            full_name,
            name,
            node_id,
            private,
            has_issues: None,
            has_wiki: None,
            has_projects: None,
            has_discussions: None,
        }
    }

    /// Returns whether issues are enabled for this repository.
    ///
    /// # Returns
    ///
    /// `Some(true)` if issues are enabled, `Some(false)` if disabled, or `None` if unknown.
    pub fn has_issues(&self) -> Option<bool> {
        self.has_issues
    }

    /// Returns whether the wiki is enabled for this repository.
    ///
    /// # Returns
    ///
    /// `Some(true)` if wiki is enabled, `Some(false)` if disabled, or `None` if unknown.
    pub fn has_wiki(&self) -> Option<bool> {
        self.has_wiki
    }

    /// Returns whether projects are enabled for this repository.
    ///
    /// # Returns
    ///
    /// `Some(true)` if projects are enabled, `Some(false)` if disabled, or `None` if unknown.
    pub fn has_projects(&self) -> Option<bool> {
        self.has_projects
    }

    /// Returns whether discussions are enabled for this repository.
    ///
    /// # Returns
    ///
    /// `Some(true)` if discussions are enabled, `Some(false)` if disabled, or `None` if unknown.
    pub fn has_discussions(&self) -> Option<bool> {
        self.has_discussions
    }

    /// Returns the GraphQL node ID of the repository.
    ///
    /// # Returns
    ///
    /// A string slice containing the node ID used for GraphQL operations.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Returns the Git clone URL for the repository.
    ///
    /// # Returns
    ///
    /// A `Url` pointing to the Git clone endpoint for this repository.
    ///
    /// # Panics
    ///
    /// Panics if the repository full name cannot be formatted into a valid URL.
    /// This should not happen with valid GitHub repository names.
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
            has_issues: value.has_issues,
            has_wiki: value.has_wiki,
            has_projects: value.has_projects,
            // Note: has_discussions is not available in octocrab::models::Repository
            // This field may need to be fetched separately via the GitHub API
            has_discussions: None,
        }
    }
}
