//! Configuration Manager trait definition
//!
//! Defines the abstract interface for configuration management following
//! dependency injection principles.

use crate::{ConfigurationResult, TemplateConfig};
use async_trait::async_trait;

/// Configuration management service interface
///
/// Provides access to template configurations and organizational settings.
/// This trait enables dependency injection and testability.
///
/// # Examples
///
/// ```no_run
/// use config_manager::ConfigurationManager;
/// use async_trait::async_trait;
///
/// struct MyConfigManager {
///     templates: Vec<config_manager::TemplateConfig>,
/// }
///
/// #[async_trait]
/// impl ConfigurationManager for MyConfigManager {
///     async fn get_template(&self, name: &str)
///         -> config_manager::ConfigurationResult<config_manager::TemplateConfig>
///     {
///         self.templates
///             .iter()
///             .find(|t| t.name == name)
///             .cloned()
///             .ok_or_else(|| {
///                 config_manager::ConfigurationError::InvalidConfiguration {
///                     field: "template".to_string(),
///                     reason: format!("Template '{}' not found", name),
///                 }
///             })
///     }
///
///     async fn list_templates(&self) -> config_manager::ConfigurationResult<Vec<String>> {
///         Ok(self.templates.iter().map(|t| t.name.clone()).collect())
///     }
/// }
/// ```
#[async_trait]
pub trait ConfigurationManager: Send + Sync {
    /// Get template configuration by name
    ///
    /// # Parameters
    /// - `name`: Template name to retrieve
    ///
    /// # Returns
    /// Template configuration if found
    ///
    /// # Errors
    /// Returns `ConfigurationError::InvalidConfiguration` if template not found
    async fn get_template(&self, name: &str) -> ConfigurationResult<TemplateConfig>;

    /// List all available template names
    ///
    /// # Returns
    /// Vector of template names
    ///
    /// # Errors
    /// Returns `ConfigurationError` if listing fails
    async fn list_templates(&self) -> ConfigurationResult<Vec<String>>;
}
