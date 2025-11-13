//! HTTP request type definitions
//!
//! This module defines all HTTP request models for the REST API.
//! These types accept flexible input from HTTP clients and are translated
//! to domain types at the API boundary.
//!
//! # Architecture
//!
//! HTTP request types have:
//! - Optional fields for flexibility
//! - String types (not branded domain types)
//! - Relaxed validation (validated during translation)
//!
//! Translation to domain types happens via `TryFrom` implementations.
//!
//! See: specs/interfaces/api-request-types.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: Import domain types for translation when available
// use repo_roller_core::{RepositoryName, OrganizationName, TemplateName};

/// HTTP request to create a repository.
///
/// This type accepts flexible input from HTTP clients and is translated
/// to `RepositoryCreationRequest` (domain type) after validation.
///
/// # Example
///
/// ```json
/// {
///   "organization": "myorg",
///   "name": "my-new-repo",
///   "template": "rust-library",
///   "visibility": "private",
///   "team": "platform",
///   "repository_type": "library",
///   "variables": {
///     "project_name": "MyLib"
///   }
/// }
/// ```
///
/// See: specs/interfaces/api-request-types.md#createrepositoryrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CreateRepositoryRequest {
    /// Organization name (GitHub organization)
    pub organization: String,

    /// Repository name (must follow GitHub naming rules)
    pub name: String,

    /// Template name to use for repository creation
    pub template: String,

    /// Repository visibility (optional, defaults from configuration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>, // "public" or "private"

    /// Team name for team-specific configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Repository type override (optional, template may specify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,

    /// Template variables for substitution
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

// TODO: Implement TryFrom<CreateRepositoryRequest> for domain RepositoryCreationRequest
// when domain types are available

/// HTTP request to validate a repository name.
///
/// This endpoint checks both:
/// 1. Name format (GitHub naming rules)
/// 2. Availability (name not already taken in organization)
///
/// See: specs/interfaces/api-request-types.md#validaterepositorynamerequest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ValidateRepositoryNameRequest {
    /// Organization name
    pub organization: String,

    /// Repository name to validate
    pub name: String,
}

/// Type alias for complete request validation.
///
/// Uses the same structure as CreateRepositoryRequest since we validate
/// all the same fields, but returns validation results instead of creating.
///
/// See: specs/interfaces/api-request-types.md#validaterepositoryrequestrequest
pub type ValidateRepositoryRequestRequest = CreateRepositoryRequest;

/// Path parameters for listing templates.
///
/// See: specs/interfaces/api-request-types.md#listtemplatesrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTemplatesParams {
    /// Organization name
    pub org: String,
}

/// Path parameters for getting template details.
///
/// See: specs/interfaces/api-request-types.md#gettemplatedetailsrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTemplateDetailsParams {
    /// Organization name
    pub org: String,

    /// Template name
    pub template: String,
}

/// Path parameters for validating a template.
pub type ValidateTemplateParams = GetTemplateDetailsParams;

/// Path parameters for listing repository types.
///
/// See: specs/interfaces/api-request-types.md#listrepositorytypesrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoryTypesParams {
    /// Organization name
    pub org: String,
}

/// Path parameters for getting repository type configuration.
///
/// See: specs/interfaces/api-request-types.md#getrepositorytypeconfigrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryTypeConfigParams {
    /// Organization name
    pub org: String,

    /// Repository type name
    #[serde(rename = "type")]
    pub type_name: String,
}

/// Path parameters for getting global defaults.
pub type GetGlobalDefaultsParams = ListRepositoryTypesParams;

/// HTTP request to preview merged configuration.
///
/// This shows what configuration will be applied based on the
/// hierarchical merge of global → org → team → type → template.
///
/// # Example
///
/// ```json
/// {
///   "template": "rust-library",
///   "team": "platform",
///   "repository_type": "library"
/// }
/// ```
///
/// See: specs/interfaces/api-request-types.md#previewconfigurationrequest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PreviewConfigurationRequest {
    /// Template name (required)
    pub template: String,

    /// Team name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Repository type (optional - template may specify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,
}

/// Path parameters for validating organization settings.
pub type ValidateOrganizationParams = ListRepositoryTypesParams;

#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
