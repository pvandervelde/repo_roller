//! Configuration system error types.
//!
//! Domain-specific errors for configuration loading, parsing,
//! and validation operations.
//!
//! See specs/interfaces/error-types.md#configurationerror

use thiserror::Error;

/// Configuration system errors.
///
/// These errors occur when loading, parsing, or validating
/// configuration from various sources (metadata repositories,
/// TOML files, hierarchical merging, etc.).
///
/// See specs/interfaces/error-types.md#configurationerror
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConfigurationError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to parse configuration: {reason}")]
    ParseError { reason: String },

    #[error("Invalid configuration: {field} - {reason}")]
    InvalidConfiguration { field: String, reason: String },

    #[error("Configuration override not permitted: {setting} - {reason}")]
    OverrideNotPermitted { setting: String, reason: String },

    #[error("Required configuration missing: {key}")]
    RequiredConfigMissing { key: String },

    #[error("Configuration hierarchy resolution failed: {reason}")]
    HierarchyResolutionFailed { reason: String },

    #[error("Metadata repository not found for organization: {org}")]
    MetadataRepositoryNotFound { org: String },
}

/// Result type alias for configuration operations.
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
