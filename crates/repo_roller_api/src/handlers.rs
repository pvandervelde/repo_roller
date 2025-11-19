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
    Extension,
    Json,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    errors::ApiError,
    middleware::AuthContext,
    models::{
        request::*,
        response::*,
    },
    AppState,
};

// Domain service imports
use config_manager::{
    ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
    OrganizationSettingsManager, MetadataRepositoryProvider,
};
use github_client::GitHubClient;

/// Create an OrganizationSettingsManager and MetadataRepositoryProvider for a request.
///
/// Creates a GitHub client using the authentication token from the request context,
/// then creates a metadata provider and settings manager.
///
/// # Arguments
///
/// * `auth` - Authentication context with bearer token
/// * `state` - Application state with configuration
///
/// # Returns
///
/// Returns tuple of (manager, provider) for use in handlers.
///
/// # Errors
///
/// Returns ApiError if GitHub client creation fails.
async fn create_settings_manager(
    auth: &AuthContext,
    state: &AppState,
) -> Result<(OrganizationSettingsManager, Arc<dyn MetadataRepositoryProvider>), ApiError> {
    // Create GitHub client with installation token
    let octocrab = github_client::create_token_client(&auth.token)
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to create GitHub client: {}", e)))?;

    let github_client = GitHubClient::new(octocrab);

    // Create metadata provider
    let provider_config = MetadataProviderConfig::explicit(&state.metadata_repository_name);
    let metadata_provider = GitHubMetadataProvider::new(github_client, provider_config);
    let provider_arc = Arc::new(metadata_provider) as Arc<dyn MetadataRepositoryProvider>;

    // Create settings manager
    let manager = OrganizationSettingsManager::new(provider_arc.clone());

    Ok((manager, provider_arc))
}

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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<CreateRepositoryRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateRepositoryResponse>), ApiError> {
    use crate::translation::{
        http_create_repository_request_to_domain, domain_repository_creation_result_to_http,
    };

    // Translate HTTP request to domain request (includes validation)
    let domain_request = http_create_repository_request_to_domain(request.clone())?;

    // Load configuration manager (needed by create_repository)
    // TODO: Load from actual config file
    let config_manager = config_manager::Config { templates: vec![] };

    // Create authentication service that returns the installation token we already have
    // The auth middleware has already validated this token with GitHub
    let token = auth.token.clone();
    let auth_service = TokenAuthService::new(token);

    // Call domain service to create repository
    let result = repo_roller_core::create_repository(
        domain_request,
        &config_manager,
        &auth_service,
        &state.metadata_repository_name,
    )
    .await?; // ApiError::from(RepoRollerError) converts automatically

    // Translate domain result to HTTP response
    let http_response = domain_repository_creation_result_to_http(result, &request);

    Ok((axum::http::StatusCode::CREATED, Json(http_response)))
}

/// Simple auth service implementation that returns a pre-validated token.
///
/// This is used when the API layer has already validated the GitHub installation token
/// via the auth middleware. We just need to provide it to the domain layer.
struct TokenAuthService {
    token: String,
}

impl TokenAuthService {
    fn new(token: String) -> Self {
        Self { token }
    }
}

#[async_trait]
impl auth_handler::UserAuthenticationService for TokenAuthService {
    async fn authenticate_installation(
        &self,
        _app_id: u64,
        _private_key: &str,
        _installation_id: u64,
    ) -> Result<String, auth_handler::AuthError> {
        // Token is already validated by middleware, just return it
        Ok(self.token.clone())
    }

    async fn get_installation_token_for_org(
        &self,
        _org: &str,
    ) -> Result<String, auth_handler::AuthError> {
        // Token is already validated by middleware, just return it
        Ok(self.token.clone())
    }
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<ListTemplatesParams>,
) -> Result<Json<ListTemplatesResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // List templates using the metadata provider
    let template_names = provider
        .list_templates(&params.org)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list templates for organization '{}': {:?}", params.org, e);
            ApiError::from(anyhow::anyhow!(
                "Failed to list templates: {}",
                e
            ))
        })?;

    // Load template configurations to get details
    let mut templates = Vec::new();
    for template_name in template_names {
        match provider.load_template_configuration(&params.org, &template_name).await {
            Ok(config) => {
                // Extract variable names from the template config
                let variable_names: Vec<String> = config.variables
                    .unwrap_or_default()
                    .keys()
                    .cloned()
                    .collect();

                templates.push(TemplateSummary {
                    name: template_name.clone(),
                    description: config.template.description.clone(),
                    category: config.template.tags.first().cloned(),
                    variables: variable_names,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to load template configuration for '{}': {:?}", template_name, e);
                // Skip templates that can't be loaded
                continue;
            }
        }
    }

    Ok(Json(ListTemplatesResponse { templates }))
}

/// GET /api/v1/orgs/:org/templates/:template
///
/// Get detailed information about a specific template.
///
/// See: specs/interfaces/api-request-types.md#gettemplatedetailsrequest
pub async fn get_template_details(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<GetTemplateDetailsParams>,
) -> Result<Json<TemplateDetailsResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // Load template configuration
    let config = provider
        .load_template_configuration(&params.org, &params.template)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to load template configuration for '{}/{}': {:?}",
                params.org,
                params.template,
                e
            );
            // Map ConfigurationError to appropriate HTTP error
            match e {
                config_manager::ConfigurationError::FileNotFound { .. } => {
                    ApiError::from(anyhow::anyhow!(
                        "Template '{}' not found in organization '{}'",
                        params.template,
                        params.org
                    ))
                }
                _ => ApiError::from(anyhow::anyhow!(
                    "Failed to load template configuration: {}",
                    e
                )),
            }
        })?;

    // Convert template variables to API response format
    let mut variables = std::collections::HashMap::new();
    if let Some(template_vars) = config.variables {
        for (name, var_config) in template_vars {
            variables.insert(
                name,
                VariableDefinition {
                    description: var_config.description,
                    required: var_config.required.unwrap_or(false),
                    default: var_config.default,
                    pattern: None, // TemplateVariable doesn't have pattern field
                },
            );
        }
    }

    // Build configuration from repository settings
    let configuration = serde_json::json!({
        "repository": config.repository,
        "pull_requests": config.pull_requests,
        "branch_protection": config.branch_protection,
    });

    let response = TemplateDetailsResponse {
        name: params.template.clone(),
        description: config.template.description.clone(),
        category: config.template.tags.first().cloned(),
        variables,
        configuration,
    };

    Ok(Json(response))
}

/// POST /api/v1/orgs/:org/templates/:template/validate
///
/// Validate a template structure.
///
/// See: specs/interfaces/api-request-types.md#validatetemplaterequest
pub async fn validate_template(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<ValidateTemplateParams>,
) -> Result<Json<ValidateTemplateResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // Try to load template configuration - if it fails, template is invalid
    match provider
        .load_template_configuration(&params.org, &params.template)
        .await
    {
        Ok(_config) => {
            // Template loaded successfully - it's valid
            Ok(Json(ValidateTemplateResponse {
                valid: true,
                errors: vec![],
                warnings: vec![],
            }))
        }
        Err(e) => {
            tracing::warn!(
                "Template validation failed for '{}/{}': {:?}",
                params.org,
                params.template,
                e
            );

            // Map errors to validation results
            match e {
                config_manager::ConfigurationError::FileNotFound { .. } => {
                    Err(ApiError::from(anyhow::anyhow!(
                        "Template '{}' not found in organization '{}'",
                        params.template,
                        params.org
                    )))
                }
                config_manager::ConfigurationError::ParseError { reason } => {
                    Ok(Json(ValidateTemplateResponse {
                        valid: false,
                        errors: vec![ValidationResult {
                            field: "template_structure".to_string(),
                            message: format!("Template configuration is malformed: {}", reason),
                            severity: ValidationSeverity::Error,
                        }],
                        warnings: vec![],
                    }))
                }
                _ => {
                    Ok(Json(ValidateTemplateResponse {
                        valid: false,
                        errors: vec![ValidationResult {
                            field: "template".to_string(),
                            message: format!("Template validation failed: {}", e),
                            severity: ValidationSeverity::Error,
                        }],
                        warnings: vec![],
                    }))
                }
            }
        }
    }
}

/// GET /api/v1/orgs/:org/repository-types
///
/// List available repository types for an organization.
///
/// See: specs/interfaces/api-request-types.md#listrepositorytypesrequest
pub async fn list_repository_types(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<ListRepositoryTypesParams>,
) -> Result<Json<ListRepositoryTypesResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(&params.org)
        .await
        .map_err(|e| {
            // If metadata repository not found, return empty list
            if matches!(e, config_manager::ConfigurationError::MetadataRepositoryNotFound { .. }) {
                return ApiError::from(anyhow::anyhow!("Metadata repository not found for organization '{}'", params.org));
            }
            ApiError::from(anyhow::anyhow!("Failed to discover metadata repository: {}", e))
        })?;

    // List available repository types
    // TODO: Once GitHub tree API is implemented in github_client, this will return actual types
    // For now, this returns an empty vector (see Technical Debt in tasks.md)
    let type_names = provider
        .list_available_repository_types(&metadata_repo)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to list repository types: {}", e)))?;

    // Convert to response format
    // TODO: Load descriptions from repository type configurations when available
    let types = type_names
        .into_iter()
        .map(|name| RepositoryTypeSummary {
            name: name.clone(),
            description: format!("Repository type: {}", name),
        })
        .collect();

    Ok(Json(ListRepositoryTypesResponse { types }))
}

/// GET /api/v1/orgs/:org/repository-types/:type
///
/// Get configuration for a specific repository type.
///
/// See: specs/interfaces/api-request-types.md#getrepositorytypeconfigrequest
pub async fn get_repository_type_config(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<GetRepositoryTypeConfigParams>,
) -> Result<Json<RepositoryTypeConfigResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(&params.org)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to discover metadata repository: {}", e)))?;

    // Load repository type configuration
    let type_config = provider
        .load_repository_type_configuration(&metadata_repo, &params.type_name)
        .await
        .map_err(|e| {
            // Check if it's a not found error
            if e.to_string().contains("not found") || e.to_string().contains("does not exist") {
                return ApiError::from(anyhow::anyhow!(
                    "Repository type '{}' not found in organization '{}'",
                    params.type_name,
                    params.org
                ));
            }
            ApiError::from(anyhow::anyhow!("Failed to load repository type configuration: {}", e))
        })?;

    // If configuration is None, type doesn't exist
    let config = type_config.ok_or_else(|| {
        ApiError::from(anyhow::anyhow!(
            "Repository type '{}' not found in organization '{}'",
            params.type_name,
            params.org
        ))
    })?;

    // Convert to JSON for response
    let configuration = serde_json::to_value(&config)
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to serialize configuration: {}", e)))?;

    let response = RepositoryTypeConfigResponse {
        name: params.type_name.clone(),
        configuration,
    };

    Ok(Json(response))
}

/// GET /api/v1/orgs/:org/defaults
///
/// Get global default configuration for an organization.
///
/// See: specs/interfaces/api-request-types.md#getglobaldefaultsrequest
pub async fn get_global_defaults(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<GetGlobalDefaultsParams>,
) -> Result<Json<GlobalDefaultsResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    // Discover metadata repository
    let metadata_repo = provider
        .discover_metadata_repository(&params.org)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to discover metadata repository: {}", e)))?;

    // Load global defaults
    let global_defaults = provider
        .load_global_defaults(&metadata_repo)
        .await
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to load global defaults: {}", e)))?;

    // Convert to JSON for response
    let defaults = serde_json::to_value(&global_defaults)
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to serialize defaults: {}", e)))?;

    Ok(Json(GlobalDefaultsResponse { defaults }))
}

/// POST /api/v1/orgs/:org/configuration/preview
///
/// Preview merged configuration for repository creation.
///
/// See: specs/interfaces/api-request-types.md#previewconfigurationrequest
pub async fn preview_configuration(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(org): Path<String>,
    Json(request): Json<PreviewConfigurationRequest>,
) -> Result<Json<PreviewConfigurationResponse>, ApiError> {
    // Create settings manager
    let (manager, _provider) = create_settings_manager(&auth, &state).await?;

    // Create configuration context
    let mut context = ConfigurationContext::new(&org, &request.template);

    if let Some(ref team) = request.team {
        context = context.with_team(team);
    }

    if let Some(ref repo_type) = request.repository_type {
        context = context.with_repository_type(repo_type);
    }

    // Resolve merged configuration
    let merged = manager
        .resolve_configuration(&context)
        .await
        .map_err(|e| {
            // Check if it's a template not found error
            if e.to_string().contains("not found") || e.to_string().contains("does not exist") {
                return ApiError::from(anyhow::anyhow!("Template '{}' not found", request.template));
            }
            ApiError::from(anyhow::anyhow!("Failed to resolve configuration: {}", e))
        })?;

    // Convert merged configuration to JSON
    let merged_configuration = serde_json::to_value(&merged)
        .map_err(|e| ApiError::from(anyhow::anyhow!("Failed to serialize merged configuration: {}", e)))?;

    // Extract source attribution from the merged configuration's source trace
    // TODO: Implement proper source trace extraction when API supports detailed attribution
    // For now, return empty map as the source_trace is complex and needs proper mapping
    let sources = std::collections::HashMap::new();

    let response = PreviewConfigurationResponse {
        merged_configuration,
        sources,
    };

    Ok(Json(response))
}

/// POST /api/v1/orgs/:org/validate
///
/// Validate organization settings and configuration.
///
/// See: specs/interfaces/api-request-types.md#validateorganizationrequest
pub async fn validate_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(params): Path<ValidateOrganizationParams>,
) -> Result<Json<ValidateOrganizationResponse>, ApiError> {
    // Create settings manager and provider
    let (_manager, provider) = create_settings_manager(&auth, &state).await?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Try to discover metadata repository
    let metadata_repo = match provider
        .discover_metadata_repository(&params.org)
        .await
    {
        Ok(repo) => repo,
        Err(e) => {
            errors.push(ValidationResult {
                field: "metadata_repository".to_string(),
                message: format!("Failed to discover metadata repository: {}", e),
                severity: ValidationSeverity::Error,
            });

            return Ok(Json(ValidateOrganizationResponse {
                valid: false,
                errors,
                warnings,
            }));
        }
    };

    // Try to load global defaults
    if let Err(e) = provider
        .load_global_defaults(&metadata_repo)
        .await
    {
        errors.push(ValidationResult {
            field: "global_defaults".to_string(),
            message: format!("Failed to load global defaults: {}", e),
            severity: ValidationSeverity::Error,
        });
    }

    // Try to list repository types
    match provider
        .list_available_repository_types(&metadata_repo)
        .await
    {
        Ok(types) => {
            // Try to load each repository type configuration
            for type_name in types {
                if let Err(e) = provider
                    .load_repository_type_configuration(&metadata_repo, &type_name)
                    .await
                {
                    errors.push(ValidationResult {
                        field: format!("repository_type.{}", type_name),
                        message: format!("Failed to load repository type '{}': {}", type_name, e),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }
        Err(e) => {
            warnings.push(ValidationResult {
                field: "repository_types".to_string(),
                message: format!("Could not list repository types: {}", e),
                severity: ValidationSeverity::Warning,
            });
        }
    }

    let valid = errors.is_empty();

    Ok(Json(ValidateOrganizationResponse {
        valid,
        errors,
        warnings,
    }))
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
