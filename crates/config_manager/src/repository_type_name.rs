//! Repository type name validation.
//!
//! Provides a branded type for repository type names with validation rules
//! to ensure consistency across the organization.

use crate::{ConfigurationError, ConfigurationResult};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

/// A validated repository type name.
///
/// Repository type names must:
/// - Be 1-50 characters long
/// - Contain only lowercase alphanumeric characters, hyphens, and underscores
/// - Not start or end with a hyphen
/// - Follow naming conventions for GitHub custom property values
///
/// # Examples
///
/// ```
/// use config_manager::RepositoryTypeName;
///
/// // Valid repository type names
/// let library = RepositoryTypeName::try_new("library").unwrap();
/// let service = RepositoryTypeName::try_new("microservice").unwrap();
/// let docs = RepositoryTypeName::try_new("documentation").unwrap();
///
/// // Invalid names will return an error
/// assert!(RepositoryTypeName::try_new("").is_err());
/// assert!(RepositoryTypeName::try_new("UPPERCASE").is_err());
/// assert!(RepositoryTypeName::try_new("-starts-with-hyphen").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepositoryTypeName(String);

impl RepositoryTypeName {
    /// Create a new RepositoryTypeName from a string.
    ///
    /// # Arguments
    ///
    /// * `name` - The repository type name to validate
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::InvalidConfiguration` if:
    /// - Name is empty or longer than 50 characters
    /// - Name contains uppercase letters
    /// - Name contains characters other than lowercase letters, digits, hyphens, underscores
    /// - Name starts or ends with a hyphen
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::RepositoryTypeName;
    ///
    /// let name = RepositoryTypeName::try_new("library")?;
    /// assert_eq!(name.as_str(), "library");
    /// # Ok::<(), config_manager::ConfigurationError>(())
    /// ```
    pub fn try_new(name: impl Into<String>) -> ConfigurationResult<Self> {
        let name = name.into();

        // Validate length
        if name.is_empty() {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: "Repository type name cannot be empty".to_string(),
            });
        }

        if name.len() > 50 {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: format!(
                    "Repository type name too long ({} characters, maximum 50)",
                    name.len()
                ),
            });
        }

        // Validate character set: lowercase alphanumeric, hyphen, underscore only
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: format!(
                    "Repository type name '{}' contains invalid characters (only lowercase letters, digits, hyphens, and underscores allowed)",
                    name
                ),
            });
        }

        // Validate doesn't start or end with hyphen
        if name.starts_with('-') {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: format!("Repository type name '{}' cannot start with a hyphen", name),
            });
        }

        if name.ends_with('-') {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: format!("Repository type name '{}' cannot end with a hyphen", name),
            });
        }

        Ok(Self(name))
    }

    /// Get the underlying string value.
    ///
    /// # Returns
    ///
    /// A string slice containing the repository type name.
    ///
    /// # Note
    ///
    /// This type also implements `Deref<Target = str>` and `AsRef<str>`,
    /// so you can use it directly where `&str` is expected.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the underlying String.
    ///
    /// # Returns
    ///
    /// The repository type name as an owned String.
    ///
    /// # Note
    ///
    /// This type also implements `From<RepositoryTypeName> for String`,
    /// so you can use `.into()` for the same effect.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for RepositoryTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RepositoryTypeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for RepositoryTypeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<str> for RepositoryTypeName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<RepositoryTypeName> for String {
    fn from(name: RepositoryTypeName) -> String {
        name.0
    }
}

#[cfg(test)]
#[path = "repository_type_name_tests.rs"]
mod tests;
