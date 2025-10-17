//! Repository domain types
//!
//! Types representing repository concepts in the business domain.
//! See specs/interfaces/shared-types.md for complete specifications.

use serde::{Deserialize, Serialize};

use crate::errors::ValidationError;

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;

/// Validated GitHub repository name
///
/// Represents a repository name that conforms to GitHub's naming rules.
/// See specs/interfaces/shared-types.md#repositoryname
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryName(String);

impl RepositoryName {
    /// Create a new repository name with validation
    ///
    /// # Validation Rules
    /// - Length: 1-100 characters
    /// - Characters: alphanumeric, hyphens, underscores, periods
    /// - Must not start with `.` or `-`
    ///
    /// # Errors
    /// Returns `ValidationError` if validation fails
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValidationError::empty_field("repository_name"));
        }

        if name.len() > 100 {
            return Err(ValidationError::too_long(
                "repository_name",
                name.len(),
                100,
            ));
        }

        if name.starts_with('.') || name.starts_with('-') {
            return Err(ValidationError::invalid_format(
                "repository_name",
                "must not start with '.' or '-'",
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(ValidationError::invalid_format(
                "repository_name",
                "must contain only alphanumeric characters, hyphens, underscores, or periods",
            ));
        }

        Ok(Self(name))
    }

    /// Get the repository name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RepositoryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Validated GitHub organization or user name
///
/// Represents an organization or user account name on GitHub.
/// See specs/interfaces/shared-types.md#organizationname
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationName(String);

impl OrganizationName {
    /// Create a new organization name with validation
    ///
    /// # Validation Rules
    /// - Length: 1-39 characters
    /// - Characters: alphanumeric and hyphens only
    /// - Must not start or end with hyphen
    /// - No consecutive hyphens
    ///
    /// # Errors
    /// Returns `ValidationError` if validation fails
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValidationError::empty_field("organization_name"));
        }

        if name.len() > 39 {
            return Err(ValidationError::too_long(
                "organization_name",
                name.len(),
                39,
            ));
        }

        if name.starts_with('-') || name.ends_with('-') {
            return Err(ValidationError::invalid_format(
                "organization_name",
                "must not start or end with hyphen",
            ));
        }

        if name.contains("--") {
            return Err(ValidationError::invalid_format(
                "organization_name",
                "must not contain consecutive hyphens",
            ));
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::invalid_format(
                "organization_name",
                "must contain only alphanumeric characters and hyphens",
            ));
        }

        Ok(Self(name))
    }

    /// Get the organization name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OrganizationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for OrganizationName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
