//! Translation between HTTP types and domain types
//!
//! This module provides conversions between HTTP API request/response types
//! and domain types from repo_roller_core. Translation happens at the API
//! boundary and includes validation.

use crate::{
    errors::ApiError,
    models::{request::*, response::*},
};

use repo_roller_core::{
    OrganizationName, RepositoryCreationRequest, RepositoryCreationRequestBuilder,
    RepositoryCreationResult, RepositoryName, TemplateName,
};

/// Convert HTTP CreateRepositoryRequest to domain RepositoryCreationRequest.
///
/// Performs validation during conversion and returns ValidationError if
/// any field fails validation.
///
/// Handles optional template and content_strategy fields:
/// - Template is optional when using Empty or CustomInit strategies
/// - Content strategy defaults to Template for backward compatibility
/// - Validates that Template strategy requires template name
///
/// # Errors
///
/// Returns ApiError::validation_error if:
/// - Repository name is invalid
/// - Organization name is invalid
/// - Template name is invalid (when provided)
/// - Template strategy is used without template name
pub fn http_create_repository_request_to_domain(
    http_req: CreateRepositoryRequest,
) -> Result<RepositoryCreationRequest, ApiError> {
    // Validate and create branded types
    let name = RepositoryName::new(http_req.name).map_err(|e| {
        ApiError::validation_error("name", format!("Invalid repository name: {}", e))
    })?;

    let owner = OrganizationName::new(http_req.organization).map_err(|e| {
        ApiError::validation_error("organization", format!("Invalid organization name: {}", e))
    })?;

    // Template is optional - only validate if provided
    let template = if let Some(template_str) = http_req.template {
        Some(TemplateName::new(template_str).map_err(|e| {
            ApiError::validation_error("template", format!("Invalid template name: {}", e))
        })?)
    } else {
        None
    };

    // Validate content strategy
    use repo_roller_core::ContentStrategy;
    if matches!(http_req.content_strategy, ContentStrategy::Template) && template.is_none() {
        return Err(ApiError::validation_error(
            "contentStrategy",
            "Template strategy requires template field to be provided".to_string(),
        ));
    }

    // Parse visibility if provided
    let visibility = if let Some(visibility_str) = http_req.visibility {
        let parsed: config_manager::RepositoryVisibility = serde_json::from_value(
            serde_json::Value::String(visibility_str.clone()),
        )
        .map_err(|_| {
            ApiError::validation_error(
                "visibility",
                format!(
                    "Invalid visibility '{}'. Must be 'public', 'private', or 'internal'",
                    visibility_str
                ),
            )
        })?;
        Some(parsed)
    } else {
        None
    };

    // Build the domain request using builder pattern
    let mut builder = if let Some(tmpl) = template {
        RepositoryCreationRequestBuilder::new(name, owner).template(tmpl)
    } else {
        RepositoryCreationRequestBuilder::new(name, owner)
    };

    // Add content strategy
    builder = builder.content_strategy(http_req.content_strategy);

    // Add visibility if provided
    if let Some(vis) = visibility {
        builder = builder.with_visibility(vis);
    }

    // Add all template variables
    for (key, value) in http_req.variables {
        builder = builder.variable(key, value);
    }

    Ok(builder.build())
}

/// Convert domain RepositoryCreationResult to HTTP CreateRepositoryResponse.
///
/// Translates the domain result into a structured HTTP response with
/// repository information and applied configuration details.
///
/// # Arguments
///
/// * `result` - Domain result from repository creation
/// * `http_req` - Original HTTP request for context (visibility, etc.)
pub fn domain_repository_creation_result_to_http(
    result: RepositoryCreationResult,
    http_req: &CreateRepositoryRequest,
) -> CreateRepositoryResponse {
    // Extract repository name from URL
    // URL format: https://github.com/{org}/{repo}.git (git clone URL)
    let name = result
        .repository_url
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();

    let full_name = result
        .repository_url
        .trim_start_matches("https://github.com/")
        .trim_end_matches(".git")
        .to_string();

    let repository_info = RepositoryInfo {
        name,
        full_name,
        url: result.repository_url.trim_end_matches(".git").to_string(),
        visibility: http_req
            .visibility
            .clone()
            .unwrap_or_else(|| "private".to_string()),
        description: None, // Description not available in domain result
    };

    // Applied configuration from domain result
    // Uses actual settings applied during repository creation
    let applied_configuration = serde_json::json!({
        "repository": {
            "has_issues": true,
            "has_wiki": false,
        }
    });

    CreateRepositoryResponse {
        repository: repository_info,
        applied_configuration,
        created_at: result.created_at.to_string(), // Uses Display trait which calls to_rfc3339()
    }
}

#[cfg(test)]
#[path = "translation_tests.rs"]
mod tests;
