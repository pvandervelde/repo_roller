//! Template domain types
//!
//! Types representing template concepts in the business domain.
//! See specs/interfaces/shared-types.md for complete specifications.

use serde::{Deserialize, Serialize};

use crate::errors::ValidationError;

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;

/// Validated template identifier
///
/// Represents a template name in kebab-case format.
/// See specs/interfaces/shared-types.md#templatename
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateName(String);

impl TemplateName {
    /// Create a new template name with validation
    ///
    /// # Validation Rules
    /// - Length: 1-50 characters
    /// - Format: kebab-case (lowercase alphanumeric + hyphens)
    /// - Must not start or end with hyphen
    ///
    /// # Errors
    /// Returns `ValidationError` if validation fails
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ValidationError::empty_field("template_name"));
        }

        if name.len() > 50 {
            return Err(ValidationError::too_long("template_name", name.len(), 50));
        }

        if name.starts_with('-') || name.ends_with('-') {
            return Err(ValidationError::invalid_format(
                "template_name",
                "must not start or end with hyphen",
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(ValidationError::invalid_format(
                "template_name",
                "must be kebab-case (lowercase alphanumeric + hyphens)",
            ));
        }

        Ok(Self(name))
    }

    /// Get the template name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TemplateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TemplateName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
