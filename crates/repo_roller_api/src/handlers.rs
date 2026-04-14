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
//! 5. Returns `Result<Json<Response>, ApiError>`
//!
//! See: .llm/rest-api-implementation-guide.md

use async_trait::async_trait;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    errors::ApiError,
    middleware::AuthContext,
    models::{request::*, response::*},
    AppState,
};

// Domain service imports
use config_manager::{
    ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
    MetadataRepositoryProvider, OrganizationSettingsManager,
};
use github_client::GitHubClient;
use repo_roller_core::{RepoRollerError, RepositoryNamingValidator};

/// Build an `OrganizationSettingsManager` and `MetadataRepositoryProvider` from
/// an already-constructed `GitHubClient`.
///
/// The caller is responsible for constructing the client; this helper wires it
/// into the metadata provider, template loader, and settings manager.
///
/// `client` is cloned once (cheap: shares the underlying connection pool) to
/// satisfy both the metadata provider and the template repository.
async fn create_settings_manager_from_client(
    client: GitHubClient,
    state: &AppState,
) -> Result<
    (
        OrganizationSettingsManager,
        Arc<dyn MetadataRepositoryProvider>,
    ),
    ApiError,
> {
    let provider_config = MetadataProviderConfig::explicit(&state.metadata_repository_name);
    let metadata_provider = GitHubMetadataProvider::new(client.clone(), provider_config);
    let provider_arc = Arc::new(metadata_provider) as Arc<dyn MetadataRepositoryProvider>;

    let template_repo = Arc::new(config_manager::GitHubTemplateRepository::new(Arc::new(
        client,
    )));
    let template_loader = Arc::new(config_manager::TemplateLoader::new(template_repo));

    let manager = OrganizationSettingsManager::new(provider_arc.clone(), template_loader);
    Ok((manager, provider_arc))
}

/// Create an OrganizationSettingsManager and MetadataRepositoryProvider for a request.
///
/// Creates a GitHub client using the authentication token from the request context,
/// then delegates to [`create_settings_manager_from_client`].
///
/// # Errors
///
/// Returns `ApiError` if GitHub client creation fails.
async fn create_settings_manager(
    auth: &AuthContext,
    state: &AppState,
) -> Result<
    (
        OrganizationSettingsManager,
        Arc<dyn MetadataRepositoryProvider>,
    ),
    ApiError,
> {
    let client =
        github_client::create_github_client(&auth.token, state.github_api_base_url.as_deref())
            .map_err(|e| {
                ApiError::from(anyhow::anyhow!("Failed to create GitHub client: {}", e))
            })?;
    create_settings_manager_from_client(client, state).await
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

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.')
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
        domain_repository_creation_result_to_http, http_create_repository_request_to_domain,
    };

    // Set the actor identity from the authenticated user, falling back to the
    // service-level identity when the token is an installation token (no user login).
    let actor_login = auth
        .user_login
        .as_deref()
        .unwrap_or("reporoller-api")
        .to_string();

    // Translate HTTP request to domain request (includes validation).
    // actor_login is passed directly so the builder sets it, keeping all
    // construction through a single code path.
    let domain_request =
        http_create_repository_request_to_domain(request.clone(), actor_login.clone())?;

    // Create GitHub client for template operations
    let github_octocrab = std::sync::Arc::new(
        github_client::create_token_client(&auth.token)
            .map_err(|e| ApiError::internal(format!("Failed to create GitHub client: {}", e)))?,
    );
    let github_client = github_client::GitHubClient::new(github_octocrab.as_ref().clone());

    // Create metadata provider for template discovery and loading
    let metadata_provider = std::sync::Arc::new(config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(&state.metadata_repository_name),
    ));

    // Create authentication service that returns the installation token we already have
    // The auth middleware has already validated this token with GitHub
    let token = auth.token.clone();
    let auth_service = TokenAuthService::new(token);

    // Create visibility providers
    let visibility_policy_provider = std::sync::Arc::new(
        config_manager::ConfigBasedPolicyProvider::new(metadata_provider.clone()),
    );
    let environment_detector = std::sync::Arc::new(
        github_client::GitHubApiEnvironmentDetector::new(github_octocrab),
    );

    // Create event notification dependencies
    let secret_resolver =
        std::sync::Arc::new(repo_roller_core::event_secrets::EnvironmentSecretResolver::new());
    // Reuse the shared metrics instance from AppState (Arc clone, not a new
    // PrometheusEventMetrics allocation).  Creating a new instance per request
    // would panic on the second call because Prometheus rejects duplicate metric
    // registrations against the same registry.
    let metrics = state.event_metrics.clone();
    let event_context =
        repo_roller_core::EventNotificationContext::new(&actor_login, secret_resolver, metrics);

    // Call domain service to create repository
    let result = repo_roller_core::create_repository(
        domain_request,
        metadata_provider.as_ref(),
        &auth_service,
        &state.metadata_repository_name,
        visibility_policy_provider,
        environment_detector,
        event_context,
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
    async fn get_installation_token_for_org(
        &self,
        _org: &str,
    ) -> Result<String, auth_handler::AuthError> {
        // Token is already validated by middleware, just return it
        Ok(self.token.clone())
    }
}

/// Load and apply organisation-level naming rules to `name`.
///
/// Discovers the metadata repository for `org`, loads its global defaults,
/// extracts any `naming_rules`, and validates `name` against them using
/// `RepositoryNamingValidator`.
///
/// # Returns
///
/// A list of human-readable error messages for each rule violation, or an
/// empty list when the name passes all rules.  Failures to reach the metadata
/// repository or load defaults are treated as soft degradation — the validation
/// still succeeds with only the format check applied.
pub(crate) async fn check_org_naming_rules(
    name: &str,
    org: &str,
    provider: &dyn MetadataRepositoryProvider,
) -> Vec<String> {
    // Discover the metadata repository for the org.
    let metadata_repo = match provider.discover_metadata_repository(org).await {
        Ok(repo) => repo,
        Err(e) => {
            tracing::warn!(
                org = org,
                error = %e,
                "Failed to discover metadata repository; skipping org naming rules"
            );
            return vec![];
        }
    };

    // Load global defaults to obtain org-level naming rules.
    let global_defaults = match provider.load_global_defaults(&metadata_repo).await {
        Ok(defaults) => defaults,
        Err(e) => {
            tracing::warn!(
                org = org,
                error = %e,
                "Failed to load global defaults; skipping org naming rules"
            );
            return vec![];
        }
    };

    // Extract naming rules — nothing to validate when none are configured.
    let naming_rules = match global_defaults.naming_rules {
        Some(rules) if !rules.is_empty() => rules,
        _ => return vec![],
    };

    // Validate the name against every configured naming rule.
    let validator = RepositoryNamingValidator::new();
    match validator.validate(name, &naming_rules) {
        Ok(()) => vec![],
        Err(e) => vec![e.to_string()],
    }
}

/// Check whether a repository name is available in the given organisation by
/// querying the GitHub API.
///
/// This is a best-effort check: API failures degrade gracefully — the caller
/// still learns `available = true` (warn-only, no 5xx).
///
/// ## Degraded path
///
/// Any non-404 error — including `403 Forbidden` (token lacks read access to a
/// private repository that actually exists) — is treated as a transient failure
/// and mapped to `available = true` with a warning message. This is intentional
/// per UX-ASSERT-028: the user may still proceed, but is cautioned that the
/// check could not be completed. Callers should not infer that a `403` means
/// the repository is free.
///
/// # Returns
///
/// A tuple of `(available, optional_warning_message)`.  When the repository
/// exists the first element is `false`; when the check could not be completed
/// due to an API error the first element is `true` and the second contains a
/// human-readable warning.
pub(crate) async fn check_repository_availability(
    client: &GitHubClient,
    org: &str,
    name: &str,
) -> (bool, Option<String>) {
    match client.get_repository(org, name).await {
        Ok(_) => (
            false,
            Some(format!(
                "Repository '{name}' already exists in organisation '{org}'."
            )),
        ),
        Err(github_client::Error::NotFound) => (true, None),
        Err(e) => {
            tracing::warn!(
                org = org,
                name = name,
                error = ?e,
                "GitHub API error while checking repository availability; treating as available"
            );
            (
                true,
                Some(
                    "Could not verify name availability with GitHub. \
                     The name may already exist."
                        .to_string(),
                ),
            )
        }
    }
}

/// POST /api/v1/repositories/validate-name
///
/// Validate a repository name for availability and format.
///
/// Performs three levels of validation:
/// 1. **Format check** (hard gate, no network): GitHub character-set rules and
///    empty-value guards for both `name` and `organization`.
/// 2. **Org naming rules** (soft-fail): organisation-level constraints loaded
///    from the metadata repository via `RepositoryNamingValidator`.
/// 3. **GitHub availability check** (soft-fail): calls the GitHub API to verify
///    the name is not already taken in the organisation.
///
/// A single `GitHubClient` is constructed per request and shared between steps
/// 2 and 3 to avoid redundant HTTP connection-pool creation. The base URL can
/// be overridden via `AppState::github_api_base_url` (e.g. for GitHub
/// Enterprise or mock servers in integration tests).
///
/// Both network checks degrade gracefully — failures produce warn-only
/// messages in the response rather than a 5xx.
///
/// See: specs/interfaces/api-request-types.md#validaterepositorynamerequest
pub async fn validate_repository_name(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<ValidateRepositoryNameRequest>,
) -> Result<Json<ValidateRepositoryNameResponse>, ApiError> {
    let mut messages = Vec::new();
    let mut valid = true;

    // ── Format validation (hard gate: no network calls) ──────────────────────
    if request.name.is_empty() {
        messages.push("Repository name cannot be empty".to_string());
        valid = false;
    }
    if !is_valid_repository_name(&request.name) {
        messages.push(
            "Repository name can only contain lowercase letters, numbers, hyphens, and underscores"
                .to_string(),
        );
        valid = false;
        if request.name.chars().any(|c| c.is_uppercase()) {
            messages.push("Repository name cannot contain uppercase letters".to_string());
        }
    }
    if !valid {
        return Ok(Json(ValidateRepositoryNameResponse {
            valid: false,
            available: false,
            messages: Some(messages),
        }));
    }

    // ── Organisation validation (hard gate: no network calls) ────────────────
    if request.organization.is_empty() {
        return Ok(Json(ValidateRepositoryNameResponse {
            valid: false,
            available: false,
            messages: Some(vec!["Organization name cannot be empty".to_string()]),
        }));
    }

    // ── Build a single GitHub client for all subsequent API calls ─────────────
    // One client is reused for both the org naming-rules check and the
    // availability check, avoiding redundant HTTP connection-pool creation.
    // The base URL can be overridden via AppState::github_api_base_url (for
    // GitHub Enterprise deployments or mock servers in integration tests).
    let github_client = match github_client::create_github_client(
        &auth.token,
        state.github_api_base_url.as_deref(),
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                org = %request.organization,
                error = ?e,
                "Failed to create GitHub client; skipping naming rules and availability check"
            );
            return Ok(Json(ValidateRepositoryNameResponse {
                valid: true,
                available: true,
                messages: Some(vec!["Could not verify name availability with GitHub. \
                     The name may already exist."
                    .to_string()]),
            }));
        }
    };

    // ── Org naming rules (soft-fail) ──────────────────────────────────────────
    match create_settings_manager_from_client(github_client.clone(), &state).await {
        Ok((_manager, provider)) => {
            let naming_messages =
                check_org_naming_rules(&request.name, &request.organization, provider.as_ref())
                    .await;
            if !naming_messages.is_empty() {
                valid = false;
                messages.extend(naming_messages);
            }
        }
        Err(e) => {
            tracing::warn!(
                org = %request.organization,
                error = ?e,
                "Failed to create settings manager for naming rule check; skipping org rules"
            );
        }
    }

    // ── Real-time name availability check (soft-fail) ─────────────────────────
    let available = if valid {
        let (is_available, availability_message) =
            check_repository_availability(&github_client, &request.organization, &request.name)
                .await;
        if let Some(msg) = availability_message {
            messages.push(msg);
        }
        is_available
    } else {
        false
    };

    Ok(Json(ValidateRepositoryNameResponse {
        valid,
        available,
        messages: if messages.is_empty() {
            None
        } else {
            Some(messages)
        },
    }))
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

    // Validate template (optional unless using Template strategy)
    if let Some(ref template_str) = request.template {
        if template_str.is_empty() {
            errors.push(ValidationResult {
                field: "template".to_string(),
                message: "Template name cannot be empty".to_string(),
                severity: ValidationSeverity::Error,
            });
        } else {
            // Note: This endpoint performs structural format validation only.
            // Template existence is validated during actual repository creation via
            // OrganizationSettingsManager, which has GitHub API access.
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

    // Note: Variable content validation (required fields, patterns) requires the
    // template configuration to be loaded from GitHub, so it is deferred to the
    // creation step via the template engine.

    // Note: Team existence validation requires GitHub API calls and is deferred
    // to the creation step.

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
    let template_names = provider.list_templates(&params.org).await.map_err(|e| {
        tracing::error!(
            "Failed to list templates for organization '{}': {:?}",
            params.org,
            e
        );
        ApiError::from(anyhow::anyhow!("Failed to list templates: {}", e))
    })?;

    // Load template configurations to get details
    let mut templates = Vec::new();
    for template_name in template_names {
        match provider
            .load_template_configuration(&params.org, &template_name)
            .await
        {
            Ok(config) => {
                // Extract variable names from the template config
                let variable_names: Vec<String> = config
                    .variables
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
                tracing::warn!(
                    "Failed to load template configuration for '{}': {:?}",
                    template_name,
                    e
                );
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
                _ => Ok(Json(ValidateTemplateResponse {
                    valid: false,
                    errors: vec![ValidationResult {
                        field: "template".to_string(),
                        message: format!("Template validation failed: {}", e),
                        severity: ValidationSeverity::Error,
                    }],
                    warnings: vec![],
                })),
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
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

    // List available repository types
    // Note: GitHub tree API for listing directory contents is documented in Technical Debt
    // Currently returns empty vector - types must be specified explicitly in requests
    let type_names = provider
        .list_available_repository_types(&metadata_repo)
        .await
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

    // Convert to response format
    // Descriptions are not available without an extra API call per type;
    // returning the name is the most accurate data we have at this point.
    // A dedicated endpoint (get_repository_type_config) provides full details.
    let types = type_names
        .into_iter()
        .map(|name| RepositoryTypeSummary {
            name: name.clone(),
            description: name.clone(),
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
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

    // Load repository type configuration
    let type_config = provider
        .load_repository_type_configuration(&metadata_repo, &params.type_name)
        .await
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

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
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

    // Load global defaults
    let global_defaults = provider
        .load_global_defaults(&metadata_repo)
        .await
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

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
    let (manager, provider) = create_settings_manager(&auth, &state).await?;

    // Validate repository type exists before merging
    if let Some(ref repo_type) = request.repository_type {
        let meta = provider
            .discover_metadata_repository(&org)
            .await
            .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;
        let type_cfg = provider
            .load_repository_type_configuration(&meta, repo_type)
            .await
            .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;
        if type_cfg.is_none() {
            return Err(ApiError::from(RepoRollerError::Configuration(
                config_manager::ConfigurationError::InvalidConfiguration {
                    field: "repository_type".to_string(),
                    reason: format!("Repository type '{}' is not defined", repo_type),
                },
            )));
        }
    }

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
        .map_err(|e| ApiError::from(RepoRollerError::Configuration(e)))?;

    // Convert merged configuration to JSON
    let merged_configuration = serde_json::to_value(&merged).map_err(|e| {
        ApiError::from(anyhow::anyhow!(
            "Failed to serialize merged configuration: {}",
            e
        ))
    })?;

    // Extract source attribution from the merged configuration's source trace
    let sources: std::collections::HashMap<String, String> = merged
        .source_trace
        .configured_fields()
        .into_iter()
        .filter_map(|field| {
            merged.source_trace.get_source(field).map(|src| {
                let level = match src {
                    config_manager::ConfigurationSource::Global => "global",
                    config_manager::ConfigurationSource::RepositoryType => "repository_type",
                    config_manager::ConfigurationSource::Team => "team",
                    config_manager::ConfigurationSource::Template => "template",
                };
                (field.to_string(), level.to_string())
            })
        })
        .collect();

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
    let metadata_repo = match provider.discover_metadata_repository(&params.org).await {
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
    if let Err(e) = provider.load_global_defaults(&metadata_repo).await {
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

/// GET /api/v1/orgs/:org/teams
///
/// List all GitHub organization teams.
///
/// Returns the slug and human-readable name for every team in the org.
/// Used by the frontend wizard Step 2 to populate the team dropdown.
///
/// See: specs/interfaces/api-response-types.md#listteamsresponse
pub async fn list_organization_teams(
    State(_state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(org): Path<String>,
) -> Result<Json<ListTeamsResponse>, ApiError> {
    // Create a GitHub client with the request's installation token.
    let octocrab = github_client::create_token_client(&auth.token)
        .map_err(|e| ApiError::internal(format!("Failed to create GitHub client: {}", e)))?;
    let github_client = GitHubClient::new(octocrab);

    let teams = github_client
        .list_organization_teams(&org)
        .await
        .map_err(|e| {
            tracing::error!(org = %org, error = %e, "Failed to list organization teams");
            ApiError::from(anyhow::anyhow!("Failed to list organization teams: {}", e))
        })?;

    let team_summaries = teams
        .into_iter()
        .map(|t| TeamSummary {
            slug: t.slug,
            name: t.name,
        })
        .collect();

    Ok(Json(ListTeamsResponse {
        teams: team_summaries,
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
