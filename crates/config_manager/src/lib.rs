//! Configuration management for RepoRoller
//!
//! This crate provides configuration management for RepoRoller, including:
//! - Template discovery and loading from GitHub repositories
//! - Organization-level configuration management
//! - Configuration merging and validation
//!
//! See specs/interfaces/configuration-interfaces.md for detailed specifications.

// Configuration system types
pub mod errors;
pub mod global_defaults;
pub mod merged_config;
pub mod overridable;
pub mod repository_type_config;
pub mod settings;
pub mod team_config;
pub mod template_config;

// Metadata repository provider
pub mod github_metadata_provider;
pub mod metadata_provider;

// Configuration merger
pub mod merger;

// Organization settings manager
pub mod configuration_context;
pub mod organization_settings_manager;

// Repository type management
pub mod repository_type_name;
pub mod repository_type_validator;

// Configuration validation
pub mod basic_validator;
pub mod validator;

// Integration tests
#[cfg(test)]
mod integration_tests;

// Re-export for convenient access
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
    RepositoryTypePolicy, RepositoryTypeSpec, TemplateConfig, TemplateMetadata, TemplateVariable,
};
pub use validator::{
    ConfigurationValidator, ValidationError, ValidationErrorType, ValidationResult,
    ValidationWarning,
};

// Re-export BasicConfigurationValidator for convenience
pub use basic_validator::BasicConfigurationValidator;
