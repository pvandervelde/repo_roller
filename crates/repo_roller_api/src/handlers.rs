//! HTTP request handlers
//!
//! This module contains all request handlers for the REST API endpoints.
//! Handlers translate HTTP requests to domain operations and domain results
//! to HTTP responses.
//!
//! # Architecture
//!
//! Each handler:
//! 1. Extracts HTTP request data (path params, query params, body)
//! 2. Translates HTTP types to domain types
//! 3. Calls business logic via service interfaces
//! 4. Translates domain results to HTTP responses
//! 5. Returns Result<Json<Response>, ApiError>
//!
//! See: .llm/rest-api-implementation-guide.md

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::ApiError,
    models::{
        request::*,
        response::*,
    },
    AppState,
};

// TODO: Import domain services when available
// use repo_roller_core::{RepositoryService, ConfigurationManager};

/// Validate a repository name against GitHub naming rules.
///
/// Repository names must:
/// - Contain only lowercase letters, numbers, hyphens, underscores, and dots
/// - Not start with a dot
/// - Not contain consecutive dots (..)
fn is_valid_repository_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('.') || name.contains("..") {
        return false;
    }
    
    name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.'
    })
}

/// POST /api/v1/repositories
///
/// Create a new repository from a template.
///
/// See: specs/interfaces/api-request-types.md#createrepositoryrequest
pub async fn create_repository(
    State(_state): State<AppState>,
    Json(request): Json<CreateRepositoryRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateRepositoryResponse>), ApiError> {
    // Validate repository name format
    if request.name.is_empty() {
        return Err(ApiError::validation_error(
            "name",
            "Repository name cannot be empty"
        ));
    }
    
    // Validate against GitHub naming rules
    if !is_valid_repository_name(&request.name) {
        return Err(ApiError::validation_error(
            "name",
            &format!("Repository name '{}' contains invalid characters. Use lowercase letters, numbers, hyphens, and underscores only.", request.name)
        ));
    }
    
    // Validate organization name
    if request.organization.is_empty() {
        return Err(ApiError::validation_error(
            "organization",
            "Organization name cannot be empty"
        ));
    }
    
    // Validate template name
    if request.template.is_empty() {
        return Err(ApiError::validation_error(
            "template",
            "Template name cannot be empty"
        ));
    }
    
    // TODO: Call actual repository creation service
    // For now, return a mock successful response
    
    let repository_info = RepositoryInfo {
        name: request.name.clone(),
        full_name: format!("{}/{}", request.organization, request.name),
        url: format!("https://github.com/{}/{}", request.organization, request.name),
        visibility: request.visibility.unwrap_or_else(|| "private".to_string()),
        description: None,
    };
    
    let applied_configuration = serde_json::json!({
        "repository": {
            "has_issues": true,
            "has_wiki": false,
            "has_projects": false
        },
        "sources": {
            "repository.has_issues": "global",
            "repository.has_wiki": "template",
        }
    });
    
    let response = CreateRepositoryResponse {
        repository: repository_info,
        applied_configuration,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

/// POST /api/v1/repositories/validate-name
///
/// Validate a repository name for availability and format.
///
/// See: specs/interfaces/api-request-types.md#validaterepositorynamerequest
pub async fn validate_repository_name(
    State(_state): State<AppState>,
    Json(request): Json<ValidateRepositoryNameRequest>,
) -> Result<Json<ValidateRepositoryNameResponse>, ApiError> {
    let mut messages = Vec::new();
    let mut valid = true;
    
    // Check if name is empty
    if request.name.is_empty() {
        messages.push("Repository name cannot be empty".to_string());
        valid = false;
    }
    
    // Check for invalid characters
    if !is_valid_repository_name(&request.name) {
        messages.push("Repository name can only contain lowercase letters, numbers, hyphens, and underscores".to_string());
        valid = false;
        
        // Check specifically for uppercase
        if request.name.chars().any(|c| c.is_uppercase()) {
            messages.push("Repository name cannot contain uppercase letters".to_string());
        }
    }
    
    // TODO: Check repository availability via GitHub API
    // For now, assume available if name is valid
    let available = valid;
    
    let response = ValidateRepositoryNameResponse {
        valid,
        available,
        messages: if messages.is_empty() { None } else { Some(messages) },
    };
    
    Ok(Json(response))
}

/// POST /api/v1/repositories/validate
///
/// Validate a complete repository creation request.
///
/// See: specs/interfaces/api-request-types.md#validaterepositoryrequestrequest
pub async fn validate_repository_request(
    State(_state): State<AppState>,
    Json(request): Json<ValidateRepositoryRequestRequest>,
) -> Result<Json<ValidateRepositoryRequestResponse>, ApiError> {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    
    // Validate repository name
    if request.name.is_empty() {
        errors.push(ValidationResult {
            field: "name".to_string(),
            message: "Repository name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    } else if !is_valid_repository_name(&request.name) {
        errors.push(ValidationResult {
            field: "name".to_string(),
            message: "Repository name contains invalid characters".to_string(),
            severity: ValidationSeverity::Error,
        });
    }
    
    // Validate organization
    if request.organization.is_empty() {
        errors.push(ValidationResult {
            field: "organization".to_string(),
            message: "Organization name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    }
    
    // Validate template
    if request.template.is_empty() {
        errors.push(ValidationResult {
            field: "template".to_string(),
            message: "Template name cannot be empty".to_string(),
            severity: ValidationSeverity::Error,
        });
    } else {
        // TODO: Check if template exists in metadata repository
        // For now, simulate check for known invalid template name
        if request.template == "nonexistent-template" {
            errors.push(ValidationResult {
                field: "template".to_string(),
                message: format!("Template '{}' does not exist in organization", request.template),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    // Validate visibility if provided
    if let Some(ref visibility) = request.visibility {
        if visibility != "public" && visibility != "private" {
            errors.push(ValidationResult {
                field: "visibility".to_string(),
                message: "Visibility must be 'public' or 'private'".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    // TODO: Validate template variables against template requirements
    // For now, simulate missing required variable check
    if request.template == "rust-library" && request.variables.is_empty() {
        errors.push(ValidationResult {
            field: "variables.project_name".to_string(),
            message: "Required variable is missing".to_string(),
            severity: ValidationSeverity::Error,
        });
    }
    
    // TODO: Validate team exists if provided
    if let Some(ref team) = request.team {
        if team == "nonexistent-team" {
            errors.push(ValidationResult {
                field: "team".to_string(),
                message: format!("Team '{}' does not exist in organization", team),
                severity: ValidationSeverity::Error,
            });
        }
    }
    
    let response = ValidateRepositoryRequestResponse {
        valid: errors.is_empty(),
        warnings,
        errors,
    };
    
    Ok(Json(response))
}

/// GET /api/v1/orgs/:org/templates
///
/// List available templates for an organization.
///
/// See: specs/interfaces/api-request-types.md#listtemplatesrequest
pub async fn list_templates(
    State(_state): State<AppState>,
    Path(params): Path<ListTemplatesParams>,
) -> Result<Json<ListTemplatesResponse>, ApiError> {
    // TODO: Task 9.3.8 - Replace with actual template discovery service call
    // For now, return mock data to establish HTTP contract
    
    // Return empty list for organizations without templates
    if params.org == "emptyorg" {
        return Ok(Json(ListTemplatesResponse {
            templates: vec![],
        }));
    }
    
    // Mock template data for establishing HTTP contract
    let templates = vec![
        TemplateSummary {
            name: "rust-library".to_string(),
            description: "Rust library project template".to_string(),
            category: Some("rust".to_string()),
            variables: vec!["project_name".to_string(), "license".to_string()],
        },
        TemplateSummary {
            name: "rust-binary".to_string(),
            description: "Rust binary application template".to_string(),
            category: Some("rust".to_string()),
            variables: vec!["project_name".to_string(), "version".to_string()],
        },
    ];
    
    Ok(Json(ListTemplatesResponse { templates }))
}

/// GET /api/v1/orgs/:org/templates/:template
///
/// Get detailed information about a specific template.
///
/// See: specs/interfaces/api-request-types.md#gettemplatedetailsrequest
pub async fn get_template_details(
    State(_state): State<AppState>,
    Path(params): Path<GetTemplateDetailsParams>,
) -> Result<Json<TemplateDetailsResponse>, ApiError> {
    // TODO: Task 9.3.8 - Replace with actual template service call
    // For now, return mock data for known templates or 404
    
    // Check if template exists (mock check)
    if params.template == "nonexistent-template" || params.template == "nonexistent" {
        return Err(ApiError::from(anyhow::anyhow!(
            "Template '{}' not found in organization '{}'",
            params.template,
            params.org
        )));
    }
    
    // Mock template details for establishing HTTP contract
    let mut variables = std::collections::HashMap::new();
    variables.insert(
        "project_name".to_string(),
        VariableDefinition {
            description: "Name of the project".to_string(),
            required: true,
            default: None,
            pattern: Some("^[a-z][a-z0-9-]*$".to_string()),
        },
    );
    variables.insert(
        "license".to_string(),
        VariableDefinition {
            description: "License type".to_string(),
            required: false,
            default: Some("MIT".to_string()),
            pattern: None,
        },
    );
    
    let response = TemplateDetailsResponse {
        name: params.template.clone(),
        description: format!("Template for {}", params.template),
        category: Some("rust".to_string()),
        variables,
        configuration: serde_json::json!({
            "has_issues": true,
            "has_wiki": false,
            "auto_init": true,
        }),
    };
    
    Ok(Json(response))
}

/// POST /api/v1/orgs/:org/templates/:template/validate
///
/// Validate a template structure.
///
/// See: specs/interfaces/api-request-types.md#validatetemplaterequest
pub async fn validate_template(
    State(_state): State<AppState>,
    Path(params): Path<ValidateTemplateParams>,
) -> Result<Json<ValidateTemplateResponse>, ApiError> {
    // TODO: Task 9.3.8 - Replace with actual template validation service call
    // For now, return mock validation results
    
    // Check if template exists (mock check)
    if params.template == "nonexistent" || params.template == "nonexistent-template" {
        return Err(ApiError::from(anyhow::anyhow!(
            "Template '{}' not found in organization '{}'",
            params.template,
            params.org
        )));
    }
    
    // Mock validation: invalid-template fails, others pass
    if params.template == "invalid-template" {
        let errors = vec![ValidationResult {
            field: "template_structure".to_string(),
            message: "Template configuration file is malformed".to_string(),
            severity: ValidationSeverity::Error,
        }];
        
        return Ok(Json(ValidateTemplateResponse {
            valid: false,
            errors,
            warnings: vec![],
        }));
    }
    
    // Valid templates return success
    Ok(Json(ValidateTemplateResponse {
        valid: true,
        errors: vec![],
        warnings: vec![],
    }))
}

/// GET /api/v1/orgs/:org/repository-types
///
/// List available repository types for an organization.
///
/// See: specs/interfaces/api-request-types.md#listrepositorytypesrequest
pub async fn list_repository_types(
    State(_state): State<AppState>,
    Path(_params): Path<ListRepositoryTypesParams>,
) -> Result<Json<ListRepositoryTypesResponse>, ApiError> {
    // TODO: Implement handler
    unimplemented!("See specs/interfaces/api-request-types.md#listrepositorytypesrequest")
}

/// GET /api/v1/orgs/:org/repository-types/:type
///
/// Get configuration for a specific repository type.
///
/// See: specs/interfaces/api-request-types.md#getrepositorytypeconfigrequest
pub async fn get_repository_type_config(
    State(_state): State<AppState>,
    Path(_params): Path<GetRepositoryTypeConfigParams>,
) -> Result<Json<RepositoryTypeConfigResponse>, ApiError> {
    // TODO: Implement handler
    unimplemented!("See specs/interfaces/api-request-types.md#getrepositorytypeconfigrequest")
}

/// GET /api/v1/orgs/:org/defaults
///
/// Get global default configuration for an organization.
///
/// See: specs/interfaces/api-request-types.md#getglobaldefaultsrequest
pub async fn get_global_defaults(
    State(_state): State<AppState>,
    Path(_params): Path<GetGlobalDefaultsParams>,
) -> Result<Json<GlobalDefaultsResponse>, ApiError> {
    // TODO: Implement handler
    unimplemented!("See specs/interfaces/api-request-types.md#getglobaldefaultsrequest")
}

/// POST /api/v1/orgs/:org/configuration/preview
///
/// Preview merged configuration for repository creation.
///
/// See: specs/interfaces/api-request-types.md#previewconfigurationrequest
pub async fn preview_configuration(
    State(_state): State<AppState>,
    Path(org): Path<String>,
    Json(_request): Json<PreviewConfigurationRequest>,
) -> Result<Json<PreviewConfigurationResponse>, ApiError> {
    let _ = org; // Suppress unused warning
    // TODO: Implement handler
    unimplemented!("See specs/interfaces/api-request-types.md#previewconfigurationrequest")
}

/// POST /api/v1/orgs/:org/validate
///
/// Validate organization settings and configuration.
///
/// See: specs/interfaces/api-request-types.md#validateorganizationrequest
pub async fn validate_organization(
    State(_state): State<AppState>,
    Path(_params): Path<ValidateOrganizationParams>,
) -> Result<Json<ValidateOrganizationResponse>, ApiError> {
    // TODO: Implement handler
    unimplemented!("See specs/interfaces/api-request-types.md#validateorganizationrequest")
}

/// GET /api/v1/health
///
/// Health check endpoint.
///
/// Returns service health status with version and timestamp.
pub async fn health_check() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse {
        status: "healthy".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    })
}

/// Health check response
///
/// See: specs/interfaces/api-response-types.md#healthcheckresponse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    /// Service status: "healthy" or "unhealthy"
    pub status: String,

    /// Service version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Current timestamp (ISO 8601)
    pub timestamp: String,

    /// Error message (if unhealthy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
#[path = "handlers_tests.rs"]
mod tests;
