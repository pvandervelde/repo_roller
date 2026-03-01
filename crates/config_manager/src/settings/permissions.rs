//! Permission policy configuration types for TOML deserialization.
//!
//! These types represent the TOML-deserializable form of permission policies
//! used in the multi-level permissions system. They are intentionally kept
//! separate from the domain types in `repo_roller_core::permissions` so that
//! `config_manager` does not depend on `repo_roller_core`.
//!
//! ## TOML Format
//!
//! ### `GlobalDefaults` permissions section
//!
//! ```toml
//! [[permissions.baseline]]
//! permission_type = "push"
//! level = "write"
//! scope = "team"
//!
//! [[permissions.restrictions]]
//! permission_type = "admin"
//! level = "admin"
//! scope = "user"
//! ```
//!
//! ### `RepositoryTypeConfig` permissions section
//!
//! ```toml
//! [[permissions.required]]
//! permission_type = "push"
//! level = "write"
//! scope = "repository"
//!
//! [permissions]
//! restricted_types = ["admin"]
//! ```
//!
//! ### `TemplateConfig` permissions section
//!
//! ```toml
//! [[permissions.required]]
//! permission_type = "push"
//! level = "write"
//! scope = "team"
//! ```
//!
//! ## Validation
//!
//! Call `.validate()` on each config type after loading to ensure all string
//! values are valid before converting to domain types.
//!
//! See `docs/spec/design/multi-level-permissions.md` for the complete specification.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod tests;

// ── Validation error ──────────────────────────────────────────────────────────

/// Error returned when a permission configuration value is invalid.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PermissionConfigError {
    /// An unrecognised permission type string was encountered.
    ///
    /// Valid values: `"pull"`, `"triage"`, `"push"`, `"maintain"`, `"admin"`.
    #[error("Invalid permission_type '{0}'; expected one of: pull, triage, push, maintain, admin")]
    InvalidPermissionType(String),

    /// An unrecognised access level string was encountered.
    ///
    /// Valid values: `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, `"admin"`.
    #[error("Invalid level '{0}'; expected one of: none, read, triage, write, maintain, admin")]
    InvalidLevel(String),

    /// An unrecognised permission scope string was encountered.
    ///
    /// Valid values: `"repository"`, `"team"`, `"user"`, `"github_app"`.
    #[error("Invalid scope '{0}'; expected one of: repository, team, user, github_app")]
    InvalidScope(String),
}

// ── Accepted string constants ─────────────────────────────────────────────────

const VALID_PERMISSION_TYPES: &[&str] = &["pull", "triage", "push", "maintain", "admin"];
const VALID_LEVELS: &[&str] = &["none", "read", "triage", "write", "maintain", "admin"];
const VALID_SCOPES: &[&str] = &["repository", "team", "user", "github_app"];

// ── PermissionGrantConfig ─────────────────────────────────────────────────────

/// TOML-deserializable representation of a single permission grant.
///
/// Each field is a lowercase string matching the corresponding domain enum
/// variant. Call [`validate`] after deserialization to ensure all values are
/// legal before converting to the domain form via
/// `TryFrom<PermissionGrantConfig> for repo_roller_core::permissions::PermissionGrant`.
///
/// # TOML Example
///
/// ```toml
/// [[permissions.baseline]]
/// permission_type = "push"
/// level = "write"
/// scope = "team"
/// ```
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::permissions::PermissionGrantConfig;
///
/// let grant = PermissionGrantConfig {
///     permission_type: "push".to_string(),
///     level: "write".to_string(),
///     scope: "team".to_string(),
/// };
/// assert!(grant.validate().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionGrantConfig {
    /// GitHub permission type string.
    ///
    /// Valid values: `"pull"`, `"triage"`, `"push"`, `"maintain"`, `"admin"`.
    pub permission_type: String,

    /// Access level string.
    ///
    /// Valid values: `"none"`, `"read"`, `"triage"`, `"write"`, `"maintain"`, `"admin"`.
    pub level: String,

    /// Scope that this grant applies to.
    ///
    /// Valid values: `"repository"`, `"team"`, `"user"`, `"github_app"`.
    pub scope: String,
}

impl PermissionGrantConfig {
    /// Validates that all string fields hold accepted values.
    ///
    /// # Errors
    ///
    /// - [`PermissionConfigError::InvalidPermissionType`] – unrecognised `permission_type`.
    /// - [`PermissionConfigError::InvalidLevel`] – unrecognised `level`.
    /// - [`PermissionConfigError::InvalidScope`] – unrecognised `scope`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::permissions::PermissionGrantConfig;
    ///
    /// let valid = PermissionGrantConfig {
    ///     permission_type: "push".to_string(),
    ///     level: "write".to_string(),
    ///     scope: "team".to_string(),
    /// };
    /// assert!(valid.validate().is_ok());
    ///
    /// let bad_type = PermissionGrantConfig {
    ///     permission_type: "unknown".to_string(),
    ///     level: "write".to_string(),
    ///     scope: "team".to_string(),
    /// };
    /// assert!(bad_type.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), PermissionConfigError> {
        if !VALID_PERMISSION_TYPES.contains(&self.permission_type.as_str()) {
            return Err(PermissionConfigError::InvalidPermissionType(
                self.permission_type.clone(),
            ));
        }
        if !VALID_LEVELS.contains(&self.level.as_str()) {
            return Err(PermissionConfigError::InvalidLevel(self.level.clone()));
        }
        if !VALID_SCOPES.contains(&self.scope.as_str()) {
            return Err(PermissionConfigError::InvalidScope(self.scope.clone()));
        }
        Ok(())
    }
}

// ── OrganizationPermissionPoliciesConfig ──────────────────────────────────────

/// TOML-deserializable organization-wide permission policy configuration.
///
/// Added to [`GlobalDefaults`](crate::GlobalDefaults) under the `[permissions]` key.
/// Defines the floor (`baseline`) and ceiling (`restrictions`) for all
/// repositories in the organization.
///
/// # TOML Example
///
/// ```toml
/// [[permissions.baseline]]
/// permission_type = "pull"
/// level = "read"
/// scope = "team"
///
/// [[permissions.restrictions]]
/// permission_type = "admin"
/// level = "maintain"
/// scope = "user"
/// ```
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::permissions::OrganizationPermissionPoliciesConfig;
///
/// let toml = r#"
///     [[baseline]]
///     permission_type = "pull"
///     level = "read"
///     scope = "team"
/// "#;
///
/// let config: OrganizationPermissionPoliciesConfig = toml::from_str(toml).unwrap();
/// assert_eq!(config.baseline.unwrap().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OrganizationPermissionPoliciesConfig {
    /// Minimum permissions that all repositories must have (floor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Vec<PermissionGrantConfig>>,

    /// Maximum permissions allowed; requests exceeding these are denied (ceiling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Vec<PermissionGrantConfig>>,
}

impl OrganizationPermissionPoliciesConfig {
    /// Validates all nested [`PermissionGrantConfig`] entries.
    ///
    /// # Errors
    ///
    /// Returns the first [`PermissionConfigError`] encountered.
    pub fn validate(&self) -> Result<(), PermissionConfigError> {
        if let Some(baseline) = &self.baseline {
            for grant in baseline {
                grant.validate()?;
            }
        }
        if let Some(restrictions) = &self.restrictions {
            for grant in restrictions {
                grant.validate()?;
            }
        }
        Ok(())
    }
}

// ── RepositoryTypePermissionsConfig ──────────────────────────────────────────

/// TOML-deserializable repository type permission configuration.
///
/// Added to [`RepositoryTypeConfig`](crate::RepositoryTypeConfig) under the
/// `[permissions]` key. Defines permissions required for all repositories of
/// this type and permission types that are forbidden.
///
/// # TOML Example
///
/// ```toml
/// [[permissions.required]]
/// permission_type = "push"
/// level = "write"
/// scope = "repository"
///
/// [permissions]
/// restricted_types = ["admin"]
/// ```
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::permissions::RepositoryTypePermissionsConfig;
///
/// let toml = r#"
///     restricted_types = ["admin"]
///
///     [[required]]
///     permission_type = "push"
///     level = "write"
///     scope = "repository"
/// "#;
///
/// let config: RepositoryTypePermissionsConfig = toml::from_str(toml).unwrap();
/// assert!(config.restricted_types.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RepositoryTypePermissionsConfig {
    /// Permissions that all repositories of this type must have.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<PermissionGrantConfig>>,

    /// Permission types completely disallowed for this repository type.
    ///
    /// Valid values: `"pull"`, `"triage"`, `"push"`, `"maintain"`, `"admin"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restricted_types: Option<Vec<String>>,
}

impl RepositoryTypePermissionsConfig {
    /// Validates all nested grants and restricted-type strings.
    ///
    /// # Errors
    ///
    /// - [`PermissionConfigError::InvalidPermissionType`] – an entry in
    ///   `restricted_types` is unrecognised.
    /// - Any error from [`PermissionGrantConfig::validate`] applied to
    ///   entries in `required`.
    pub fn validate(&self) -> Result<(), PermissionConfigError> {
        if let Some(required) = &self.required {
            for grant in required {
                grant.validate()?;
            }
        }
        if let Some(restricted_types) = &self.restricted_types {
            for t in restricted_types {
                if !VALID_PERMISSION_TYPES.contains(&t.as_str()) {
                    return Err(PermissionConfigError::InvalidPermissionType(t.clone()));
                }
            }
        }
        Ok(())
    }
}

// ── TemplatePermissionsConfig ─────────────────────────────────────────────────

/// TOML-deserializable template permission configuration.
///
/// Added to [`TemplateConfig`](crate::template_config::TemplateConfig) under the
/// `[permissions]` key. Defines permissions required for the template to
/// function correctly.
///
/// # TOML Example
///
/// ```toml
/// [[permissions.required]]
/// permission_type = "push"
/// level = "write"
/// scope = "team"
/// ```
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::permissions::TemplatePermissionsConfig;
///
/// let toml = r#"
///     [[required]]
///     permission_type = "push"
///     level = "write"
///     scope = "team"
/// "#;
///
/// let config: TemplatePermissionsConfig = toml::from_str(toml).unwrap();
/// assert_eq!(config.required.unwrap().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TemplatePermissionsConfig {
    /// Permissions required for the template to function correctly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<PermissionGrantConfig>>,
}

impl TemplatePermissionsConfig {
    /// Validates all nested [`PermissionGrantConfig`] entries.
    ///
    /// # Errors
    ///
    /// Returns the first [`PermissionConfigError`] encountered.
    pub fn validate(&self) -> Result<(), PermissionConfigError> {
        if let Some(required) = &self.required {
            for grant in required {
                grant.validate()?;
            }
        }
        Ok(())
    }
}
