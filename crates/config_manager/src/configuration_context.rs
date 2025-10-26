//! Configuration resolution context.
//!
//! This module provides the `ConfigurationContext` type which carries metadata
//! about a configuration resolution request. The context includes the organization,
//! template, and optional team and repository type information.
//!
//! # Examples
//!
//! ```
//! use config_manager::ConfigurationContext;
//!
//! // Create context with only required fields
//! let context = ConfigurationContext::new("my-org", "rust-service");
//!
//! // Create context with optional fields using builder pattern
//! let context = ConfigurationContext::new("my-org", "rust-service")
//!     .with_team("backend-team")
//!     .with_repository_type("library");
//! ```

use chrono::{DateTime, Utc};

/// Configuration resolution context.
///
/// Carries metadata about a configuration resolution request including
/// the organization, template, team (optional), and repository type (optional).
///
/// This context is used during the configuration resolution workflow to
/// determine which configuration files to load and merge.
///
/// # Examples
///
/// ```
/// use config_manager::ConfigurationContext;
///
/// // Minimal context
/// let context = ConfigurationContext::new("my-org", "rust-service");
/// assert_eq!(context.organization(), "my-org");
/// assert_eq!(context.template(), "rust-service");
/// assert_eq!(context.team(), None);
/// assert_eq!(context.repository_type(), None);
///
/// // Context with team
/// let context = ConfigurationContext::new("my-org", "rust-service")
///     .with_team("backend-team");
/// assert_eq!(context.team(), Some("backend-team"));
///
/// // Context with repository type
/// let context = ConfigurationContext::new("my-org", "rust-service")
///     .with_repository_type("library");
/// assert_eq!(context.repository_type(), Some("library"));
///
/// // Context with both
/// let context = ConfigurationContext::new("my-org", "rust-service")
///     .with_team("backend-team")
///     .with_repository_type("library");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationContext {
    /// The GitHub organization name.
    organization: String,

    /// The template name/identifier.
    template: String,

    /// Optional team name for team-specific configuration.
    team: Option<String>,

    /// Optional repository type for type-specific configuration.
    repository_type: Option<String>,

    /// Timestamp when this context was created.
    created_at: DateTime<Utc>,
}

impl ConfigurationContext {
    /// Creates a new configuration context with required fields.
    ///
    /// # Arguments
    ///
    /// * `organization` - The GitHub organization name
    /// * `template` - The template name/identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// assert_eq!(context.organization(), "my-org");
    /// assert_eq!(context.template(), "rust-service");
    /// ```
    pub fn new(organization: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            organization: organization.into(),
            template: template.into(),
            team: None,
            repository_type: None,
            created_at: Utc::now(),
        }
    }

    /// Adds team information to the context.
    ///
    /// Uses builder pattern for ergonomic chaining.
    ///
    /// # Arguments
    ///
    /// * `team` - The team name
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service")
    ///     .with_team("backend-team");
    /// assert_eq!(context.team(), Some("backend-team"));
    /// ```
    pub fn with_team(mut self, team: impl Into<String>) -> Self {
        self.team = Some(team.into());
        self
    }

    /// Adds repository type information to the context.
    ///
    /// Uses builder pattern for ergonomic chaining.
    ///
    /// # Arguments
    ///
    /// * `repository_type` - The repository type name
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service")
    ///     .with_repository_type("library");
    /// assert_eq!(context.repository_type(), Some("library"));
    /// ```
    pub fn with_repository_type(mut self, repository_type: impl Into<String>) -> Self {
        self.repository_type = Some(repository_type.into());
        self
    }

    /// Gets the organization name.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// assert_eq!(context.organization(), "my-org");
    /// ```
    pub fn organization(&self) -> &str {
        &self.organization
    }

    /// Gets the template name.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// assert_eq!(context.template(), "rust-service");
    /// ```
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Gets the team name if specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// assert_eq!(context.team(), None);
    ///
    /// let context = context.with_team("backend-team");
    /// assert_eq!(context.team(), Some("backend-team"));
    /// ```
    pub fn team(&self) -> Option<&str> {
        self.team.as_deref()
    }

    /// Gets the repository type if specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// assert_eq!(context.repository_type(), None);
    ///
    /// let context = context.with_repository_type("library");
    /// assert_eq!(context.repository_type(), Some("library"));
    /// ```
    pub fn repository_type(&self) -> Option<&str> {
        self.repository_type.as_deref()
    }

    /// Gets the timestamp when this context was created.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::ConfigurationContext;
    ///
    /// let context = ConfigurationContext::new("my-org", "rust-service");
    /// let created_at = context.created_at();
    /// assert!(created_at <= chrono::Utc::now());
    /// ```
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
#[path = "configuration_context_tests.rs"]
mod tests;
