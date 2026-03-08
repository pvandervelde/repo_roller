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

    #[error("Template repository not found: {org}/{template}")]
    TemplateNotFound { org: String, template: String },

    #[error("Template configuration file missing in {org}/{template}: expected .reporoller/template.toml")]
    TemplateConfigurationMissing { org: String, template: String },

    #[error("Configuration validation failed with {error_count} error(s)")]
    ValidationFailed {
        error_count: usize,
        errors: Vec<ValidationError>,
    },

    /// A lower-precedence configuration level attempted to reduce the access
    /// level of a team or collaborator that was set at a higher-precedence level.
    ///
    /// The `context` field identifies the configuration level being protected
    /// (e.g. `"org"` or `"template"`); `identifier` is the team slug or username.
    #[error(
        "Cannot demote {context} permission for '{identifier}': \
         current level '{current_level}' is higher than attempted '{attempted_level}'"
    )]
    PermissionDemotionNotAllowed {
        /// The team slug or collaborator username being protected.
        identifier: String,
        /// Human-readable label for the level being protected (e.g. `"org"`).
        context: String,
        /// The effective level already established at the higher-precedence level.
        current_level: String,
        /// The lower level the lower-precedence config tried to set.
        attempted_level: String,
    },

    /// A lower-precedence configuration level attempted to change a team or
    /// collaborator entry that was marked `locked` at a higher-precedence level.
    ///
    /// `context` identifies the level that locked the entry; `identifier` is the
    /// team slug or username.
    #[error("Cannot alter locked {context} permission for '{identifier}'")]
    PermissionLockedNotAllowed {
        /// The team slug or collaborator username that is locked.
        identifier: String,
        /// Human-readable label for the level that locked the entry (e.g. `"org"`).
        context: String,
    },
}

/// Result type alias for configuration operations.
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
