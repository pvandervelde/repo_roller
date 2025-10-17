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
