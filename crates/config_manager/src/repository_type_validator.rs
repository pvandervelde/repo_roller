//! Repository type validation logic.
//!
//! Provides validation for repository types including existence checks,
//! policy enforcement, and GitHub custom property creation.

use crate::{
    repository_type_name::RepositoryTypeName,
    settings::custom_property::{CustomProperty, CustomPropertyValue},
    template_config::{RepositoryTypePolicy, RepositoryTypeSpec},
    ConfigurationError, ConfigurationResult,
};

/// Validator for repository type operations.
///
/// Provides validation logic for:
/// - Checking repository type existence in organization configuration
/// - Enforcing repository type policies (fixed vs preferable)
/// - Creating GitHub custom properties for repository types
///
/// # Examples
///
/// ```
/// use config_manager::{RepositoryTypeValidator, RepositoryTypeName};
///
/// let validator = RepositoryTypeValidator::new();
///
/// // Validate type exists
/// let available = vec!["library".to_string(), "service".to_string()];
/// let type_name = RepositoryTypeName::try_new("library")?;
/// validator.validate_type_exists(&type_name, &available)?;
/// # Ok::<(), config_manager::ConfigurationError>(())
/// ```
pub struct RepositoryTypeValidator;

impl RepositoryTypeValidator {
    /// Create a new repository type validator.
    pub fn new() -> Self {
        Self
    }

    /// Validate that a repository type exists in the available types.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The repository type name to validate
    /// * `available_types` - List of available repository types in the organization
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::InvalidConfiguration` if:
    /// - The type is not in the available types list
    /// - The available types list is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::{RepositoryTypeValidator, RepositoryTypeName};
    ///
    /// let validator = RepositoryTypeValidator::new();
    /// let available = vec!["library".to_string(), "service".to_string()];
    /// let type_name = RepositoryTypeName::try_new("library")?;
    ///
    /// // This succeeds
    /// validator.validate_type_exists(&type_name, &available)?;
    ///
    /// // This fails
    /// let unknown = RepositoryTypeName::try_new("unknown")?;
    /// assert!(validator.validate_type_exists(&unknown, &available).is_err());
    /// # Ok::<(), config_manager::ConfigurationError>(())
    /// ```
    pub fn validate_type_exists(
        &self,
        type_name: &RepositoryTypeName,
        available_types: &[String],
    ) -> ConfigurationResult<()> {
        // Check available_types is not empty
        if available_types.is_empty() {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: "No repository types are defined in the organization configuration"
                    .to_string(),
            });
        }

        // Check type_name is in available_types (case-insensitive)
        let type_name_lower = type_name.as_str().to_lowercase();
        let found = available_types
            .iter()
            .any(|t| t.to_lowercase() == type_name_lower);

        if !found {
            return Err(ConfigurationError::InvalidConfiguration {
                field: "repository_type".to_string(),
                reason: format!(
                    "Repository type '{}' is not defined in organization configuration. Available types: {}",
                    type_name,
                    available_types.join(", ")
                ),
            });
        }

        Ok(())
    }

    /// Validate repository type policy and determine final type to use.
    ///
    /// Enforces the repository type policy:
    /// - **Fixed**: Template type cannot be overridden
    /// - **Preferable**: Template type is default, but user can override
    ///
    /// # Arguments
    ///
    /// * `spec` - The repository type specification from the template
    /// * `user_override` - Optional user-specified repository type override
    ///
    /// # Returns
    ///
    /// The repository type to use (either template type or validated user override).
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::InvalidConfiguration` if:
    /// - Policy is Fixed and user provided an override
    /// - User override is provided but doesn't match validation rules
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::{RepositoryTypeValidator, RepositoryTypeName};
    /// use config_manager::template_config::{RepositoryTypeSpec, RepositoryTypePolicy};
    ///
    /// let validator = RepositoryTypeValidator::new();
    ///
    /// // Fixed policy - no override allowed
    /// let spec = RepositoryTypeSpec {
    ///     repository_type: "service".to_string(),
    ///     policy: RepositoryTypePolicy::Fixed,
    /// };
    /// let result = validator.validate_type_policy(&spec, None)?;
    /// assert_eq!(result.as_str(), "service");
    ///
    /// // Preferable policy - override allowed
    /// let spec = RepositoryTypeSpec {
    ///     repository_type: "library".to_string(),
    ///     policy: RepositoryTypePolicy::Preferable,
    /// };
    /// let override_type = RepositoryTypeName::try_new("documentation")?;
    /// let result = validator.validate_type_policy(&spec, Some(&override_type))?;
    /// assert_eq!(result.as_str(), "documentation");
    /// # Ok::<(), config_manager::ConfigurationError>(())
    /// ```
    pub fn validate_type_policy(
        &self,
        spec: &RepositoryTypeSpec,
        user_override: Option<&RepositoryTypeName>,
    ) -> ConfigurationResult<RepositoryTypeName> {
        match spec.policy {
            RepositoryTypePolicy::Fixed => {
                // Fixed policy: user cannot override
                if user_override.is_some() {
                    return Err(ConfigurationError::InvalidConfiguration {
                        field: "repository_type".to_string(),
                        reason: format!(
                            "Repository type '{}' is fixed by template and cannot be overridden",
                            spec.repository_type
                        ),
                    });
                }
                // Return template type
                RepositoryTypeName::try_new(&spec.repository_type)
            }
            RepositoryTypePolicy::Preferable => {
                // Preferable policy: user can override, but template type is default
                if let Some(user_type) = user_override {
                    // Return user override
                    Ok(user_type.clone())
                } else {
                    // Return template type
                    RepositoryTypeName::try_new(&spec.repository_type)
                }
            }
        }
    }

    /// Create a GitHub custom property for the repository type.
    ///
    /// Creates a `CustomProperty` with:
    /// - Property name: "repository_type"
    /// - Value: SingleSelect variant with the type name
    ///
    /// This property can be applied to the repository via GitHub API.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The repository type name
    ///
    /// # Returns
    ///
    /// A `CustomProperty` ready to be applied via GitHub API.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::{RepositoryTypeValidator, RepositoryTypeName};
    ///
    /// let validator = RepositoryTypeValidator::new();
    /// let type_name = RepositoryTypeName::try_new("library")?;
    /// let property = validator.create_type_custom_property(&type_name);
    ///
    /// assert_eq!(property.property_name, "repository_type");
    /// # Ok::<(), config_manager::ConfigurationError>(())
    /// ```
    pub fn create_type_custom_property(&self, type_name: &RepositoryTypeName) -> CustomProperty {
        // TODO: Implement custom property creation
        // - Create CustomProperty with property_name = "repository_type"
        // - Use CustomPropertyValue::SingleSelect with type_name

        CustomProperty {
            property_name: "repository_type".to_string(),
            value: CustomPropertyValue::SingleSelect(type_name.as_str().to_string()),
        }
    }
}

impl Default for RepositoryTypeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "repository_type_validator_tests.rs"]
mod tests;
