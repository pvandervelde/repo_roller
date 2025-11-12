//! HTTP response type definitions
//!
//! This module defines all HTTP response models for the REST API.
//! These types are created from domain results and sent to HTTP clients.
//!
//! # Architecture
//!
//! HTTP response types:
//! - Convert from domain types via `From` trait
//! - Include HTTP-specific metadata
//! - Use camelCase for JSON serialization
//!
//! See: specs/interfaces/api-response-types.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: Import domain types for conversion when available
// use repo_roller_core::{Repository, Template, RepositoryType};

/// HTTP response for successful repository creation.
///
/// Returns repository details and metadata about the creation operation.
///
/// # Example
///
/// ```json
/// {
///   "repository": {
///     "name": "my-new-repo",
///     "fullName": "myorg/my-new-repo",
///     "url": "https://github.com/myorg/my-new-repo",
///     "visibility": "private"
///   },
///   "appliedConfiguration": {
///     "teamPermissions": {...},
///     "branchProtection": {...}
///   },
///   "createdAt": "2025-11-12T10:30:00Z"
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#createrepositoryresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepositoryResponse {
    /// Created repository details
    pub repository: RepositoryInfo,

    /// Configuration that was applied to the repository
    pub applied_configuration: serde_json::Value,

    /// Timestamp when repository was created
    pub created_at: String, // ISO 8601 format
}

/// Repository information included in responses.
///
/// This is a simplified view of repository details for HTTP responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInfo {
    /// Repository name (without organization)
    pub name: String,

    /// Full name including organization (e.g., "myorg/my-repo")
    pub full_name: String,

    /// GitHub URL
    pub url: String,

    /// Repository visibility ("public" or "private")
    pub visibility: String,

    /// Repository description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// TODO: Implement From<Repository> for RepositoryInfo when domain type is available

/// HTTP response for repository name validation.
///
/// # Example
///
/// ```json
/// {
///   "valid": true,
///   "available": true
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#validaterepositorynameresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRepositoryNameResponse {
    /// Whether the name is valid (format check)
    pub valid: bool,

    /// Whether the name is available (not taken in organization)
    pub available: bool,

    /// Validation messages if invalid or unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<String>>,
}

/// HTTP response for complete request validation.
///
/// Validates all aspects of a repository creation request.
///
/// # Example
///
/// ```json
/// {
///   "valid": false,
///   "errors": [
///     {
///       "field": "name",
///       "message": "Repository name must be lowercase",
///       "severity": "error"
///     },
///     {
///       "field": "variables.project_name",
///       "message": "Required template variable is missing",
///       "severity": "error"
///     }
///   ],
///   "warnings": [
///     {
///       "field": "visibility",
///       "message": "Public visibility is not recommended for this template",
///       "severity": "warning"
///     }
///   ]
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#validaterepositoryrequestresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRepositoryRequestResponse {
    /// Whether the entire request is valid
    pub valid: bool,

    /// Validation errors (prevent creation)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationResult>,

    /// Validation warnings (don't prevent creation)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationResult>,
}

/// Individual validation result for a field or constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Field name (using dot notation for nested fields)
    pub field: String,

    /// Validation message
    pub message: String,

    /// Severity level
    pub severity: ValidationSeverity,
}

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    /// Error - prevents operation
    Error,

    /// Warning - operation can proceed
    Warning,

    /// Info - informational message
    Info,
}

/// HTTP response for listing available templates.
///
/// # Example
///
/// ```json
/// {
///   "templates": [
///     {
///       "name": "rust-library",
///       "description": "Rust library template with CI/CD",
///       "category": "rust",
///       "variables": ["project_name", "author"]
///     }
///   ]
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#listtemplatesresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTemplatesResponse {
    /// Available templates
    pub templates: Vec<TemplateSummary>,
}

/// Template summary information for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    /// Template name (identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Template category (e.g., "rust", "python", "documentation")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Required variable names
    pub variables: Vec<String>,
}

// TODO: Implement From<Template> for TemplateSummary when domain type is available

/// HTTP response for template details.
///
/// # Example
///
/// ```json
/// {
///   "name": "rust-library",
///   "description": "Rust library template with CI/CD",
///   "category": "rust",
///   "variables": {
///     "project_name": {
///       "description": "Name of the project",
///       "required": true,
///       "default": null
///     }
///   },
///   "configuration": {
///     "visibility": "public",
///     "features": ["issues", "projects"]
///   }
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#templatedetailsresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetailsResponse {
    /// Template name
    pub name: String,

    /// Description
    pub description: String,

    /// Category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Variable definitions
    pub variables: HashMap<String, VariableDefinition>,

    /// Template-specific configuration
    pub configuration: serde_json::Value,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    /// Variable description
    pub description: String,

    /// Whether variable is required
    pub required: bool,

    /// Default value (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Pattern for validation (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// HTTP response for template validation.
///
/// See: specs/interfaces/api-response-types.md#validatetemplateresponse
pub type ValidateTemplateResponse = ValidateRepositoryRequestResponse;

/// HTTP response for listing repository types.
///
/// # Example
///
/// ```json
/// {
///   "types": [
///     {
///       "name": "library",
///       "description": "Reusable library component"
///     }
///   ]
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#listrepositorytypesresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoryTypesResponse {
    /// Available repository types
    pub types: Vec<RepositoryTypeSummary>,
}

/// Repository type summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTypeSummary {
    /// Type name (identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,
}

/// HTTP response for repository type configuration.
///
/// See: specs/interfaces/api-response-types.md#repositorytypeconfigresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTypeConfigResponse {
    /// Type name
    pub name: String,

    /// Configuration for this type
    pub configuration: serde_json::Value,
}

/// HTTP response for global defaults.
///
/// See: specs/interfaces/api-response-types.md#globaldefaultsresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalDefaultsResponse {
    /// Global default configuration
    pub defaults: serde_json::Value,
}

/// HTTP response for configuration preview.
///
/// Shows the merged configuration that will be applied.
///
/// # Example
///
/// ```json
/// {
///   "mergedConfiguration": {
///     "visibility": "private",
///     "features": ["issues"],
///     "teamPermissions": {...}
///   },
///   "sources": {
///     "visibility": "team",
///     "features": "template",
///     "teamPermissions": "type"
///   }
/// }
/// ```
///
/// See: specs/interfaces/api-response-types.md#previewconfigurationresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewConfigurationResponse {
    /// Merged configuration result
    pub merged_configuration: serde_json::Value,

    /// Source of each configuration value (for traceability)
    pub sources: HashMap<String, String>, // key -> source (e.g., "visibility" -> "team")
}

/// HTTP response for organization settings validation.
///
/// See: specs/interfaces/api-response-types.md#validateorganizationresponse
pub type ValidateOrganizationResponse = ValidateRepositoryRequestResponse;

#[cfg(test)]
#[path = "response_tests.rs"]
mod tests;
