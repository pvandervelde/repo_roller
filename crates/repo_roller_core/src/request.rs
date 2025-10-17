//! Repository creation request types
//!
//! Types for requesting repository creation operations.
//! See specs/interfaces/repository-domain.md for complete specifications.

use std::collections::HashMap;

use crate::{OrganizationName, RepositoryName, TemplateName, Timestamp};

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;

/// Request for creating a new repository with validated types.
///
/// This is the new typed API for repository creation that uses
/// branded types for type safety and validation.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{RepositoryCreationRequest, RepositoryName, OrganizationName, TemplateName};
/// use std::collections::HashMap;
///
/// let request = RepositoryCreationRequest {
///     name: RepositoryName::new("my-new-repo").unwrap(),
///     owner: OrganizationName::new("my-org").unwrap(),
///     template: TemplateName::new("rust-library").unwrap(),
///     variables: HashMap::new(),
/// };
/// ```
///
/// See specs/interfaces/repository-domain.md#repositorycreationrequest
#[derive(Debug, Clone)]
pub struct RepositoryCreationRequest {
    /// The name of the repository to create
    pub name: RepositoryName,

    /// The organization or user that will own the repository
    pub owner: OrganizationName,

    /// The template to use for initializing the repository
    pub template: TemplateName,

    /// Template variables for variable substitution during processing
    pub variables: HashMap<String, String>,
}

/// Result of a successful repository creation operation.
///
/// Contains metadata about the newly created repository.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{RepositoryCreationResult, Timestamp};
///
/// let result = RepositoryCreationResult {
///     repository_url: "https://github.com/my-org/my-repo".to_string(),
///     repository_id: "R_kgDOABCDEF".to_string(),
///     created_at: Timestamp::now(),
///     default_branch: "main".to_string(),
/// };
/// ```
///
/// See specs/interfaces/repository-domain.md#repositorycreationresult
#[derive(Debug, Clone)]
pub struct RepositoryCreationResult {
    /// The URL of the created repository
    pub repository_url: String,

    /// The GitHub repository ID
    pub repository_id: String,

    /// When the repository was created
    pub created_at: Timestamp,

    /// The default branch name
    pub default_branch: String,
}


/// Builder for constructing RepositoryCreationRequest instances.
///
/// Provides an ergonomic API for building repository creation requests
/// with optional template variables.
///
/// # Examples
///
/// ```rust
/// use repo_roller_core::{RepositoryCreationRequestBuilder, RepositoryName, OrganizationName, TemplateName};
///
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo").unwrap(),
///     OrganizationName::new("my-org").unwrap(),
///     TemplateName::new("rust-library").unwrap(),
/// )
/// .variable("project_name", "MyProject")
/// .variable("author", "John Doe")
/// .build();
/// ```
#[derive(Debug, Clone)]
pub struct RepositoryCreationRequestBuilder {
    name: RepositoryName,
    owner: OrganizationName,
    template: TemplateName,
    variables: HashMap<String, String>,
}

impl RepositoryCreationRequestBuilder {
    /// Create a new builder with required fields.
    pub fn new(name: RepositoryName, owner: OrganizationName, template: TemplateName) -> Self {
        Self {
            name,
            owner,
            template,
            variables: HashMap::new(),
        }
    }

    /// Add a single template variable.
    ///
    /// If a variable with the same key already exists, it will be overwritten.
    pub fn variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple template variables at once.
    ///
    /// Existing variables with the same keys will be overwritten.
    pub fn variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Build the final RepositoryCreationRequest.
    pub fn build(self) -> RepositoryCreationRequest {
        RepositoryCreationRequest {
            name: self.name,
            owner: self.owner,
            template: self.template,
            variables: self.variables,
        }
    }
}
