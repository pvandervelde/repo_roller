//! Repository creation request types
//!
//! Types for requesting repository creation operations.
//! See specs/interfaces/repository-domain.md for complete specifications.

use std::collections::HashMap;

use crate::{OrganizationName, RepositoryName, TemplateName};

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
    // TODO: Implement builder fields
}

impl RepositoryCreationRequestBuilder {
    /// Create a new builder with required fields.
    pub fn new(name: RepositoryName, owner: OrganizationName, template: TemplateName) -> Self {
        todo!()
    }

    /// Add a single template variable.
    pub fn variable(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        todo!()
    }

    /// Add multiple template variables at once.
    pub fn variables(self, vars: HashMap<String, String>) -> Self {
        todo!()
    }

    /// Build the final RepositoryCreationRequest.
    pub fn build(self) -> RepositoryCreationRequest {
        todo!()
    }
}
