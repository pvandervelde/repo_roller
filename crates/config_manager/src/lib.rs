//! Configuration management for RepoRoller
//!
//! TODO (Interface Design): This crate will be completely redesigned based on interface specifications.
//! The types below are temporary stubs to maintain compatibility with existing code.
//!
//! See specs/interfaces/configuration-interfaces.md for the new interface design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Configuration manager trait
pub mod config_manager_trait;

// New configuration system types (Task 2.0)
pub mod errors;
pub mod global_defaults;
pub mod merged_config;
pub mod overridable;
pub mod repository_type_config;
pub mod settings;
pub mod team_config;
pub mod template_config;

// Metadata repository provider (Task 3.0)
pub mod github_metadata_provider;
pub mod metadata_provider;

// Configuration merger (Task 4.0)
pub mod merger;

// Organization settings manager (Task 5.0)
pub mod configuration_context;
pub mod organization_settings_manager;

// Repository type management (Task 6.0)
pub mod repository_type_name;
pub mod repository_type_validator;

// Configuration validation (Task 8.0)
pub mod basic_validator;
pub mod validator;

// Integration tests (Task 2.7)
#[cfg(test)]
mod integration_tests;

// Re-export for convenient access
pub use config_manager_trait::ConfigurationManager;
pub use configuration_context::ConfigurationContext;
pub use errors::{ConfigurationError, ConfigurationResult};
pub use github_metadata_provider::{GitHubMetadataProvider, MetadataProviderConfig};
pub use global_defaults::GlobalDefaults;
pub use merged_config::{ConfigurationSource, ConfigurationSourceTrace, MergedConfiguration};
pub use merger::ConfigurationMerger;
pub use metadata_provider::{DiscoveryMethod, MetadataRepository, MetadataRepositoryProvider};
pub use organization_settings_manager::OrganizationSettingsManager;
pub use overridable::OverridableValue;
pub use repository_type_config::RepositoryTypeConfig;
pub use repository_type_name::RepositoryTypeName;
pub use repository_type_validator::RepositoryTypeValidator;
pub use settings::LabelConfig;
pub use team_config::TeamConfig;
pub use template_config::{
    RepositoryTypePolicy, RepositoryTypeSpec, TemplateMetadata, TemplateVariable,
};
pub use validator::{
    ConfigurationValidator, ValidationError, ValidationErrorType, ValidationResult,
    ValidationWarning,
};

// Re-export BasicConfigurationValidator for convenience
pub use basic_validator::BasicConfigurationValidator;

// Re-export new TemplateConfig with different name to avoid conflict with legacy type
pub use template_config::TemplateConfig as NewTemplateConfig;

// ================================================================================================
// TEMPORARY COMPATIBILITY TYPES
// These types maintain compatibility with existing code while interface design is in progress
// TODO: Replace with new types from specs/interfaces/configuration-interfaces.md
// ================================================================================================

/// Temporary Config structure for existing code compatibility
///
/// TODO: Replace with new ConfigurationManager trait and types
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub templates: Vec<TemplateConfig>,
}

// Implement ConfigurationManager trait for the legacy Config struct
#[async_trait::async_trait]
impl ConfigurationManager for Config {
    async fn get_template(&self, name: &str) -> ConfigurationResult<TemplateConfig> {
        self.templates
            .iter()
            .find(|t| t.name == name)
            .cloned()
            .ok_or_else(|| ConfigurationError::InvalidConfiguration {
                field: "template".to_string(),
                reason: format!("Template '{}' not found", name),
            })
    }

    async fn list_templates(&self) -> ConfigurationResult<Vec<String>> {
        Ok(self.templates.iter().map(|t| t.name.clone()).collect())
    }
}

/// Temporary TemplateConfig structure for existing code compatibility
///
/// TODO: Replace with new template configuration types
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub name: String,
    pub source_repo: String,
    pub variable_configs: Option<HashMap<String, VariableConfig>>,

    // Additional fields for integration tests compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_protection_rules: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_permissions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_variables: Option<Vec<String>>,
}

/// Temporary VariableConfig structure for existing code compatibility
///
/// TODO: Replace with new variable configuration types
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariableConfig {
    pub description: String,
    pub example: Option<String>,
    pub required: Option<bool>,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
}
