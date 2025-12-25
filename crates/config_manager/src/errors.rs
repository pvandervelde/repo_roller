//! Configuration system error types.
//!
//! Domain-specific errors for configuration loading, parsing,
//! and validation operations.
//!
//! See specs/interfaces/error-types.md#configurationerror

use thiserror::Error;

// Import ValidationError for the ValidationFailed variant
use crate::validator::ValidationError;

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

    #[error("Failed to access configuration file: {path} - {reason}")]
    FileAccessError { path: String, reason: String },

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

    #[error("Multiple metadata repositories found for organization '{org}' with topic '{topic}': {repositories:?}. Expected exactly one.")]
    AmbiguousMetadataRepository {
        org: String,
        topic: String,
        repositories: Vec<String>,
    },

    #[error("Configuration validation failed with {error_count} error(s)")]
    ValidationFailed {
        error_count: usize,
        errors: Vec<ValidationError>,
    },
}

/// Result type alias for configuration operations.
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
