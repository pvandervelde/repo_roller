//! # RepoRoller Core
//!
//! This crate provides the core orchestration logic for RepoRoller, a tool that creates
//! GitHub repositories from templates with variable substitution and automated setup.
//!
//! ## Overview
//!
//! RepoRoller Core handles the complete workflow of repository creation:
//! 1. Template fetching from source repositories
//! 2. Variable substitution in template files
//! 3. Local Git repository initialization and commit creation
//! 4. GitHub repository creation via API
//! 5. Repository content push with authentication
//! 6. Post-creation setup (apps, webhooks, settings)
//!
//! ## Main Functions
//!
//! The primary entry point is:
//! - [`create_repository`] - Create a repository with type-safe branded types
//!
//! ## Type System
//!
//! The crate uses a type-safe design with:
//! - Branded types for domain values ([`RepositoryName`], [`OrganizationName`], [`TemplateName`])
//! - [`RepositoryCreationRequest`] and [`RepositoryCreationRequestBuilder`] for type-safe requests
//! - [`RepositoryCreationResult`] with structured repository metadata
//! - [`RepoRollerResult<T>`] for comprehensive error handling with domain-specific errors
//!
//! See `specs/interfaces/` for complete interface specifications.
//!
//! ## Examples
//!
//! ```no_run
//! use repo_roller_core::{
//!     create_repository, RepositoryCreationRequestBuilder,
//!     RepositoryName, OrganizationName, TemplateName
//! };
//! use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
//! use auth_handler::GitHubAuthService;
//! use github_client::GitHubClient;
//!
//! # async fn example(
//! #     visibility_policy_provider: std::sync::Arc<dyn config_manager::VisibilityPolicyProvider>,
//! #     environment_detector: std::sync::Arc<dyn github_client::GitHubEnvironmentDetector>
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Create a type-safe repository creation request
//! let request = RepositoryCreationRequestBuilder::new(
//!     RepositoryName::new("my-new-project")?,
//!     OrganizationName::new("my-organization")?,
//! )
//! .template(TemplateName::new("rust-library")?)
//! .variable("author", "Jane Doe")
//! .build();
//!
//! // Create GitHub client and metadata provider
//! let github_client = github_client::create_token_client("installation-token")?;
//! let metadata_provider = GitHubMetadataProvider::new(
//!     GitHubClient::new(github_client),
//!     MetadataProviderConfig::explicit(".reporoller")
//! );
//!
//! // Create authentication service
//! let auth_service = GitHubAuthService::new(12345, "private-key-content".to_string());
//!
//! // Create event notification providers
//! let secret_resolver = std::sync::Arc::new(
//!     repo_roller_core::event_secrets::EnvironmentSecretResolver::new()
//! );
//! let metrics_registry = prometheus::Registry::new();
//! let metrics = std::sync::Arc::new(
//!     repo_roller_core::event_metrics::PrometheusEventMetrics::new(&metrics_registry)
//! );
//!
//! // Bundle event notification parameters
//! let event_context = repo_roller_core::event_publisher::EventNotificationContext::new(
//!     "user@example.com",
//!     secret_resolver,
//!     metrics,
//! );
//!
//! // Create the repository
//! match create_repository(
//!     request,
//!     &metadata_provider,
//!     &auth_service,
//!     ".reporoller", // Metadata repository name
//!     visibility_policy_provider,
//!     environment_detector,
//!     event_context,
//! ).await {
//!     Ok(result) => {
//!         println!("Repository created successfully:");
//!         println!("  URL: {}", result.repository_url);
//!         println!("  ID: {}", result.repository_id);
//!         println!("  Default branch: {}", result.default_branch);
//!     }
//!     Err(e) => eprintln!("Repository creation failed: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`RepoRollerResult<T>`] which provides structured error
//! information with domain-specific error types.

use github_client::{GitHubClient, RepositoryClient, RepositoryCreatePayload};
use temp_dir::TempDir;
use tracing::{debug, error, info, warn};

mod errors;

// Git operations module
mod git;

// Configuration resolution and application module
mod configuration;

// Template processing operations module
mod template_processing;
// Re-export for testing
pub use template_processing::extract_config_variables;

// Content providers for repository initialization
mod content_providers;

// Label management operations
mod label_manager;

// Webhook management operations
mod webhook_manager;

// Ruleset management operations
mod ruleset_manager;

// Permission types and domain model
pub mod permissions;

// Policy engine for permission evaluation
pub mod policy_engine;

// Permission manager for orchestrating permission application
pub mod permission_manager;

// Structured audit logging for permission decisions
pub mod permission_audit_logger;

// Permission workflow helpers for the repository creation pipeline
pub mod permission_workflow;

// Event publishing operations
pub mod event_publisher;

// Event secret resolution
pub mod event_secrets;

// Event metrics collection
pub mod event_metrics;

// Re-export error types for public API
pub use errors::{
    AuthenticationError, ConfigurationError, GitHubError, RepoRollerError, RepoRollerResult,
    RepositoryError, SystemError, TemplateError, ValidationError,
};

// Domain-specific types organized by business area
// See specs/interfaces/shared-types.md for complete specifications

/// Repository domain types (RepositoryName, OrganizationName)
pub mod repository;

/// Template domain types (TemplateName)
pub mod template;

/// Repository creation request types
pub mod request;

/// GitHub integration types (InstallationId, GitHubToken)
pub mod github;

/// Authentication domain types (UserId, SessionId)
pub mod authentication;

/// Repository visibility types and resolution
///
/// This module provides types and logic for determining repository visibility
/// based on hierarchical policies and GitHub environment constraints.
///
/// # Example
///
/// ```no_run
/// use repo_roller_core::{VisibilityResolver, VisibilityRequest, OrganizationName};
/// use config_manager::RepositoryVisibility;
/// use std::sync::Arc;
///
/// # async fn example(
/// #     policy_provider: Arc<dyn config_manager::VisibilityPolicyProvider>,
/// #     environment_detector: Arc<dyn github_client::GitHubEnvironmentDetector>
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let resolver = VisibilityResolver::new(policy_provider, environment_detector);
///
/// let request = VisibilityRequest {
///     organization: OrganizationName::new("my-org")?,
///     user_preference: Some(RepositoryVisibility::Private),
///     template_default: None,
/// };
///
/// let decision = resolver.resolve_visibility(request).await?;
/// println!("Visibility: {:?}", decision.visibility);
/// # Ok(())
/// # }
/// ```
pub mod visibility;

// Re-export commonly used types
pub use authentication::{SessionId, UserId};
pub use github::{GitHubToken, InstallationId};
pub use repository::{OrganizationName, RepositoryName};
pub use request::{
    ContentStrategy, RepositoryCreationRequest, RepositoryCreationRequestBuilder,
    RepositoryCreationResult,
};
pub use template::TemplateName;
// Re-exported from visibility module - see module docs for examples
pub use visibility::{
    DecisionSource, GitHubEnvironmentDetector, PlanLimitations, PolicyConstraint,
    RepositoryVisibility, VisibilityDecision, VisibilityError, VisibilityPolicy,
    VisibilityPolicyProvider, VisibilityRequest, VisibilityResolver,
};
// Re-exported from content_providers module
pub use content_providers::{
    ContentProvider, CustomInitContentProvider, CustomInitOptions, TemplateBasedContentProvider,
    ZeroContentProvider,
};
// Re-exported from label_manager module
pub use label_manager::{ApplyLabelsResult, LabelManager};
// Re-exported from webhook_manager module
pub use webhook_manager::{ApplyWebhooksResult, WebhookManager};
// Re-exported from ruleset_manager module
pub use ruleset_manager::{ApplyRulesetsResult, RulesetManager};
// Re-exported from permissions module
pub use permissions::{
    AccessLevel, GitHubPermissionLevel, OrganizationPermissionPolicies, PermissionCondition,
    PermissionDuration, PermissionError, PermissionGrant, PermissionHierarchy, PermissionRequest,
    PermissionScope, PermissionType, RepositoryContext, RepositoryTypePermissions,
    TemplatePermissions, UserPermissionRequests,
};
// Re-exported from event_publisher module
pub use event_publisher::{
    collect_notification_endpoints, compute_hmac_sha256, publish_repository_created,
    sign_webhook_request, AppliedSettings, DeliveryResult, EventNotificationContext,
    RepositoryCreatedEvent,
};
// Re-exported from config_manager
pub use config_manager::{NotificationEndpoint, NotificationsConfig};
// Re-exported from event_secrets module
pub use event_secrets::{
    EnvironmentSecretResolver, FilesystemSecretResolver, SecretResolutionError, SecretResolver,
};
// Re-exported from event_metrics module
pub use event_metrics::{EventMetrics, NoOpEventMetrics, PrometheusEventMetrics};

// Cross-cutting types used across all domains
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// UTC timestamp wrapper
///
/// Represents a point in time in UTC timezone.
/// See specs/interfaces/shared-types.md#timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Create a timestamp for the current moment
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Create a timestamp from a `DateTime<Utc>`
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Get the inner `DateTime<Utc>`
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::from_datetime(dt)
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "lib_integration_tests.rs"]
mod integration_tests;

// ── Private helpers ───────────────────────────────────────────────────────────

/// GitHub clients created during the authentication step.
///
/// Groups the three GitHub-facing objects produced in Steps 1–2 so they can
/// be passed around as a unit without a long parameter list.
struct CreationClients {
    /// Raw installation token used for git push authentication.
    installation_token: String,
    /// GitHub API client authenticated as the installation.
    installation_repo_client: GitHubClient,
    /// Template content fetcher backed by the same installation token.
    template_fetcher: template_engine::GitHubTemplateFetcher,
}

/// Authenticates as the GitHub App installation and creates the GitHub clients
/// needed for the rest of the creation workflow.
///
/// # Errors
///
/// Returns `AuthenticationFailed` when the installation token cannot be obtained,
/// or `SystemError::Internal` when a client cannot be constructed.
async fn setup_github_clients(
    auth_service: &dyn auth_handler::UserAuthenticationService,
    owner: &str,
) -> RepoRollerResult<CreationClients> {
    let installation_token = auth_service
        .get_installation_token_for_org(owner)
        .await
        .map_err(|e| {
            error!("Failed to authenticate: {}", e);
            RepoRollerError::GitHub(GitHubError::AuthenticationFailed {
                reason: format!("Failed to get installation token: {}", e),
            })
        })?;

    let installation_client =
        github_client::create_token_client(&installation_token).map_err(|e| {
            error!("Failed to create installation token client: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create installation token client: {}", e),
            })
        })?;
    let installation_repo_client = GitHubClient::new(installation_client);

    let template_fetcher_client =
        github_client::create_token_client(&installation_token).map_err(|e| {
            error!("Failed to create GitHub client for template fetcher: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create GitHub client: {}", e),
            })
        })?;
    let template_fetcher =
        template_engine::GitHubTemplateFetcher::new(GitHubClient::new(template_fetcher_client));

    Ok(CreationClients {
        installation_token,
        installation_repo_client,
        template_fetcher,
    })
}

/// Resolves the merged organization configuration and loads the template
/// configuration from GitHub (when a template is specified).
///
/// Returns `(merged_config, template)` where `template` is `None` for
/// empty-repository or no-template creations.
///
/// # Errors
///
/// Returns `ConfigurationError` when the merged config cannot be resolved, or
/// `TemplateError::TemplateNotFound` when the requested template does not exist.
async fn load_creation_config(
    installation_token: &str,
    request: &RepositoryCreationRequest,
    metadata_provider: &dyn config_manager::MetadataRepositoryProvider,
    metadata_repository_name: &str,
) -> RepoRollerResult<(
    config_manager::MergedConfiguration,
    Option<config_manager::TemplateConfig>,
)> {
    let template_name_for_config = request.template.as_ref().map(|t| t.as_ref()).unwrap_or("");
    let merged_config = configuration::resolve_organization_configuration(
        installation_token,
        request.owner.as_ref(),
        template_name_for_config,
        metadata_repository_name,
    )
    .await?;

    let template = if let Some(ref template_name) = request.template {
        debug!(
            "Loading template '{}' from organization '{}'",
            template_name, request.owner
        );
        let loaded = metadata_provider
            .load_template_configuration(request.owner.as_ref(), template_name.as_ref())
            .await
            .map_err(|e| {
                error!("Template '{}' not found: {}", template_name, e);
                RepoRollerError::Template(TemplateError::TemplateNotFound {
                    name: template_name.to_string(),
                })
            })?;
        info!("Template '{}' loaded successfully", template_name);
        Some(loaded)
    } else {
        info!("No template specified - using organization defaults");
        None
    };

    Ok((merged_config, template))
}

/// Resolves the final repository visibility by evaluating organization policies,
/// GitHub environment constraints, and the user's preference.
///
/// # Errors
///
/// Maps all [`visibility::VisibilityError`] variants to [`RepoRollerError`].
async fn resolve_repository_visibility(
    request: &RepositoryCreationRequest,
    template: Option<&config_manager::TemplateConfig>,
    visibility_policy_provider: std::sync::Arc<dyn visibility::VisibilityPolicyProvider>,
    environment_detector: std::sync::Arc<dyn visibility::GitHubEnvironmentDetector>,
) -> RepoRollerResult<visibility::VisibilityDecision> {
    info!(
        "Resolving repository visibility for organization '{}'",
        request.owner
    );

    let visibility_request = visibility::VisibilityRequest {
        organization: request.owner.clone(),
        user_preference: request.visibility,
        template_default: template.and_then(|t| t.default_visibility),
    };

    let resolver =
        visibility::VisibilityResolver::new(visibility_policy_provider, environment_detector);

    let decision = resolver
        .resolve_visibility(visibility_request)
        .await
        .map_err(|e| {
            error!("Failed to resolve visibility: {}", e);
            match e {
                visibility::VisibilityError::PolicyViolation { requested, policy } => {
                    RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
                        field: "visibility".to_string(),
                        reason: format!(
                            "Visibility {:?} violates organization policy: {}",
                            requested, policy
                        ),
                    })
                }
                visibility::VisibilityError::GitHubConstraint { requested, reason } => {
                    RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
                        field: "visibility".to_string(),
                        reason: format!("Visibility {:?} not available: {}", requested, reason),
                    })
                }
                visibility::VisibilityError::PolicyNotFound { organization } => {
                    warn!(
                        "No visibility policy configured for organization '{}', using default",
                        organization
                    );
                    RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
                        field: "visibility_policy".to_string(),
                        reason: format!(
                            "No visibility policy configured for organization '{}'",
                            organization
                        ),
                    })
                }
                visibility::VisibilityError::ConfigurationError { message } => {
                    RepoRollerError::Configuration(ConfigurationError::InvalidConfiguration {
                        field: "visibility".to_string(),
                        reason: message,
                    })
                }
                visibility::VisibilityError::EnvironmentDetectionFailed {
                    organization,
                    reason,
                } => RepoRollerError::GitHub(GitHubError::NetworkError {
                    reason: format!(
                        "Failed to detect GitHub environment for organization '{}': {}",
                        organization, reason
                    ),
                }),
                visibility::VisibilityError::GitHubApiError(source) => {
                    RepoRollerError::GitHub(GitHubError::NetworkError {
                        reason: format!("Failed to detect GitHub environment: {}", source),
                    })
                }
            }
        })?;

    info!(
        "Visibility resolved: {:?}, source: {:?}",
        decision.visibility, decision.source
    );
    debug!("Applied constraints: {:?}", decision.constraints_applied);

    Ok(decision)
}

/// Generates the local repository content by selecting a [`ContentProvider`]
/// based on `request.content_strategy` and calling `provide_content`.
///
/// Returns the temporary directory path containing the generated content.
///
/// # Errors
///
/// Propagates errors from the content provider.
async fn generate_repository_content(
    request: &RepositoryCreationRequest,
    template: Option<&config_manager::TemplateConfig>,
    merged_config: &config_manager::MergedConfiguration,
    template_fetcher: &template_engine::GitHubTemplateFetcher,
) -> RepoRollerResult<TempDir> {
    let template_source = request
        .template
        .as_ref()
        .map(|t| format!("{}/{}", request.owner.as_ref(), t.as_ref()))
        .unwrap_or_default();

    let content_provider: Box<dyn crate::ContentProvider> = match request.content_strategy {
        crate::ContentStrategy::Template => {
            Box::new(crate::TemplateBasedContentProvider::new(template_fetcher))
        }
        crate::ContentStrategy::Empty => Box::new(crate::ZeroContentProvider::new()),
        crate::ContentStrategy::CustomInit {
            include_readme,
            include_gitignore,
        } => Box::new(crate::CustomInitContentProvider::new(
            crate::CustomInitOptions {
                include_readme,
                include_gitignore,
            },
        )),
    };

    content_provider
        .provide_content(request, template, &template_source, merged_config)
        .await
}

/// Pushes the local repository to the newly created GitHub remote.
///
/// # Errors
///
/// Returns `SystemError::Internal` if the push fails.
fn push_repository_to_github(
    local_repo_path: &TempDir,
    repo_url: url::Url,
    default_branch: &str,
    installation_token: &str,
) -> RepoRollerResult<()> {
    info!("Pushing local repository to remote: {}", repo_url);
    git::push_to_origin(
        local_repo_path,
        repo_url,
        default_branch,
        installation_token,
    )
    .map_err(|e| {
        error!("Failed to push to origin: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to push to origin: {}", e),
        })
    })?;
    info!("Repository successfully pushed to GitHub");
    Ok(())
}

/// Applies the merged configuration and repository permissions after the
/// repository has been created and populated on GitHub.
///
/// Permission errors are treated as non-fatal warnings: configuration will
/// still be applied and the repository is fully usable.
///
/// # Errors
///
/// Returns errors from configuration application; permission errors are logged
/// and suppressed.
async fn apply_post_creation_settings(
    installation_repo_client: &GitHubClient,
    request: &RepositoryCreationRequest,
    merged_config: &config_manager::MergedConfiguration,
    template: Option<&config_manager::TemplateConfig>,
    requestor: &str,
) -> RepoRollerResult<()> {
    configuration::apply_repository_configuration(
        installation_repo_client,
        request.owner.as_ref(),
        request.name.as_ref(),
        merged_config,
    )
    .await?;

    // Permission errors are non-fatal: the repository already exists and is
    // usable; log a warning rather than failing the entire creation.
    {
        let permission_manager = crate::permission_manager::PermissionManager::new(
            installation_repo_client.clone(),
            crate::policy_engine::PolicyEngine::new(),
        );
        let hierarchy = crate::permission_workflow::build_permission_hierarchy(template);
        let perm_request = crate::permission_workflow::build_permission_request(
            &request.owner,
            &request.name,
            requestor,
        );
        // NOTE: `hierarchy` carries empty `organization_policies` and
        // `user_requested_permissions` because org-level policies are not yet
        // threaded through `MergedConfiguration`. The `PolicyEngine::evaluate_permission_request`
        // call below therefore evaluates an empty request against an empty policy and
        // is effectively a no-op today. The real permission-merge enforcement lives in
        // `merge_access_map_with_policy` (called immediately below). The PolicyEngine
        // path is future infrastructure intended to replace that merge once org policies
        // are properly wired up. Until then, do not add policy logic inside the engine.
        // Teams and collaborators are merged from two sources:
        //   1. Org/template config (merged_config.teams / .collaborators) – baseline
        //   2. Explicit request fields (request.teams / .collaborators)    – highest precedence
        //
        // Policy rules applied during merge (request overrides are subject to):
        //   • Locked entries (merged_config.locked_teams) cannot be altered by the request.
        //   • Config-established entries cannot be demoted (only upgraded) by the request.
        //   • No request entry may exceed merged_config.max_team_access_level (capped).
        let teams: std::collections::HashMap<String, crate::permissions::AccessLevel> =
            merge_access_map_with_policy(
                &merged_config.teams,
                &merged_config.locked_teams,
                merged_config.max_team_access_level.as_deref(),
                request
                    .teams
                    .iter()
                    .map(|(k, v)| (k.as_str(), *v))
                    .collect::<Vec<_>>()
                    .as_slice(),
                "team",
            );
        let collaborators: std::collections::HashMap<String, crate::permissions::AccessLevel> =
            merge_access_map_with_policy(
                &merged_config.collaborators,
                &merged_config.locked_collaborators,
                merged_config.max_collaborator_access_level.as_deref(),
                request
                    .collaborators
                    .iter()
                    .map(|(k, v)| (k.as_str(), *v))
                    .collect::<Vec<_>>()
                    .as_slice(),
                "collaborator",
            );

        match permission_manager
            .apply_repository_permissions(
                request.owner.as_ref(),
                request.name.as_ref(),
                &perm_request,
                &hierarchy,
                &teams,
                &collaborators,
            )
            .await
        {
            Ok(result) => {
                info!(
                    teams_applied = result.teams_applied,
                    collaborators_applied = result.collaborators_applied,
                    "Repository permissions applied successfully"
                );
            }
            Err(e) => {
                warn!(
                    "Permission application failed (non-fatal); repository created \
                     but permissions may not be fully configured: {}",
                    e
                );
            }
        }
    }

    Ok(())
}

/// Merges a config-derived access map with request-supplied overrides, enforcing
/// three protection rules:
///
/// 1. **Locked entries** — identifiers in `locked` cannot be altered by any request entry.
///    The config value is preserved and a warning is logged.
/// 2. **No demotion** — request entries that would *reduce* the access level of a
///    config-established identifier are silently skipped (with a warning).
/// 3. **Max-level ceiling** — if `max_level_str` is `Some`, request entries that exceed
///    that ceiling are capped at the ceiling value (with a warning).
///
/// # Arguments
///
/// * `config_entries`  – Config-derived identifiers (`HashMap<String, String>`).
/// * `locked`          – Set of identifiers that must not be changed by requests.
/// * `max_level_str`   – Optional ceiling level string; entries exceeding it are capped.
/// * `request_entries` – `(&str, AccessLevel)` pairs from the creation request.
/// * `kind`            – Human-readable label used in warning messages (`"team"` or
///   `"collaborator"`).
///
/// # Returns
///
/// A `HashMap<String, AccessLevel>` with the fully resolved access map.
pub fn merge_access_map_with_policy(
    config_entries: &std::collections::HashMap<String, String>,
    locked: &std::collections::HashSet<String>,
    max_level_str: Option<&str>,
    request_entries: &[(&str, crate::permissions::AccessLevel)],
    kind: &str,
) -> std::collections::HashMap<String, crate::permissions::AccessLevel> {
    // Parse and collect config entries; skip entries with unrecognised level strings.
    let mut map: std::collections::HashMap<String, crate::permissions::AccessLevel> =
        config_entries
            .iter()
            .filter_map(|(id, level_str)| {
                crate::permissions::AccessLevel::try_from(level_str.as_str())
                    .map(|level| (id.clone(), level))
                    .map_err(|e| {
                        warn!(
                            kind,
                            identifier = id.as_str(),
                            level = level_str.as_str(),
                            "Ignoring invalid access level for default {} from config: {}",
                            kind,
                            e
                        );
                    })
                    .ok()
            })
            .collect();

    // Parse the ceiling once; if the string is invalid, ignore the ceiling with a warning.
    let ceiling: Option<crate::permissions::AccessLevel> = max_level_str.and_then(|s| {
        crate::permissions::AccessLevel::try_from(s)
            .map_err(|e| {
                warn!(
                    kind,
                    ceiling = s,
                    "Ignoring invalid max_{}_access_level from org config: {}",
                    kind,
                    e
                );
            })
            .ok()
    });

    for (id, mut requested_level) in request_entries.iter().copied() {
        let id_owned = id.to_string();

        // Rule 1 – locked entry must not be changed by a request.
        // Note: `locked` is populated only from entries already present in
        // `config_entries`, so the map entry is always Occupied here.
        // A locked entry must never accept an arbitrary request value.
        if locked.contains(&id_owned) {
            warn!(
                kind,
                identifier = id,
                "Request tried to alter locked {} '{}'; ignoring request entry",
                kind,
                id
            );
            continue;
        }

        // Apply ceiling (Rule 3 in the spec) before the no-demotion check so that
        // we compare the already-capped level against the config-established level.
        if let Some(cap) = ceiling {
            if requested_level > cap {
                warn!(
                    kind,
                    identifier = id,
                    requested = ?requested_level,
                    ceiling = ?cap,
                    "Request {} level for '{}' exceeds org ceiling; capping at {:?}",
                    kind,
                    id,
                    cap
                );
                requested_level = cap;
            }
        }

        // No-demotion (Rule 2 in the spec): a request cannot lower a
        // config-established level (checked against the already-capped value).
        if let Some(&existing_level) = map.get(&id_owned) {
            if requested_level < existing_level {
                warn!(
                    kind,
                    identifier = id,
                    existing = ?existing_level,
                    requested = ?requested_level,
                    "Request tried to demote {} '{}' from {:?} to {:?}; ignoring",
                    kind,
                    id,
                    existing_level,
                    requested_level
                );
                continue;
            }
        }

        map.insert(id_owned, requested_level);
    }

    map
}

/// Spawns a background task that delivers `RepositoryCreatedEvent` notifications
/// to all configured webhook endpoints.
///
/// Uses a fire-and-forget pattern: the caller does not wait for delivery.
/// Delivery results are summarised in the tracing log.
fn spawn_event_notification(
    result: &RepositoryCreationResult,
    request: RepositoryCreationRequest,
    merged_config: config_manager::MergedConfiguration,
    event_context: event_publisher::EventNotificationContext,
) {
    let result_clone = result.clone();
    let created_by_str = event_context.created_by;
    let secret_resolver = event_context.secret_resolver;
    let metrics = event_context.metrics;

    tokio::spawn(async move {
        info!(
            repository = %request.name,
            "Spawning background task for event notifications"
        );

        let delivery_results = publish_repository_created(
            &result_clone,
            &request,
            &merged_config,
            &created_by_str,
            secret_resolver.as_ref(),
            metrics.as_ref(),
        )
        .await;

        let success_count = delivery_results.iter().filter(|r| r.success).count();
        let failure_count = delivery_results.len() - success_count;

        if failure_count > 0 {
            warn!(
                repository = %request.name,
                success_count = success_count,
                failure_count = failure_count,
                "Event notification delivery completed with failures"
            );
        } else if success_count > 0 {
            info!(
                repository = %request.name,
                success_count = success_count,
                "Event notification delivery completed successfully"
            );
        }
    });
}

/// Initialize and commit local Git repository.
///
/// # Returns
///
/// Returns the default branch name used.
async fn initialize_git_repository(
    local_repo_path: &TempDir,
    installation_repo_client: &GitHubClient,
    organization: &str,
    allow_empty_commit: bool,
) -> RepoRollerResult<String> {
    info!(
        "Getting organization default branch setting for: {}",
        organization
    );
    let default_branch = installation_repo_client
        .get_organization_default_branch(organization)
        .await
        .unwrap_or_else(|e| {
            warn!(
                "Failed to get default branch for organization '{}': {}. Using 'main' as default.",
                organization, e
            );
            "main".to_string()
        });

    info!("Using default branch: {}", default_branch);

    debug!(
        "Initializing local git repository with branch: {}",
        default_branch
    );
    git::init_local_git_repo(local_repo_path, &default_branch).map_err(|e| {
        error!("Failed to initialize local git repository: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to initialize local git repository: {}", e),
        })
    })?;

    debug!(
        "Committing initial changes (allow_empty: {})",
        allow_empty_commit
    );
    git::commit_all_changes(local_repo_path, "Initial commit", allow_empty_commit).map_err(
        |e| {
            error!("Failed to commit changes: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to commit changes: {}", e),
            })
        },
    )?;

    Ok(default_branch)
}

/// Create repository on GitHub with merged configuration settings.
///
/// # Returns
///
/// Returns the created GitHub repository.
async fn create_github_repository(
    request: &RepositoryCreationRequest,
    merged_config: &config_manager::MergedConfiguration,
    installation_repo_client: &GitHubClient,
    visibility: visibility::RepositoryVisibility,
) -> RepoRollerResult<github_client::Repository> {
    let payload = RepositoryCreatePayload {
        name: request.name.as_ref().to_string(),
        private: Some(visibility.is_private()),
        has_issues: merged_config.repository.issues.as_ref().map(|v| v.value),
        has_projects: merged_config.repository.projects.as_ref().map(|v| v.value),
        has_wiki: merged_config.repository.wiki.as_ref().map(|v| v.value),
        ..Default::default()
    };

    info!(
        "Creating GitHub repository: name='{}', visibility={:?}",
        request.name, visibility
    );
    let repo = installation_repo_client
        .create_org_repository(request.owner.as_ref(), &payload)
        .await
        .map_err(|e| {
            error!("Failed to create GitHub repository: {}", e);
            RepoRollerError::GitHub(GitHubError::NetworkError {
                reason: format!("Failed to create repository: {}", e),
            })
        })?;

    info!(
        "GitHub repository created successfully: url='{}'",
        repo.url()
    );
    Ok(repo)
}

/// Create a new repository with type-safe API and organization settings integration.
///
/// This is the main repository creation orchestration function that coordinates:
/// - Configuration resolution via OrganizationSettingsManager
/// - Template fetching and processing
/// - Local Git repository initialization
/// - GitHub repository creation
/// - Configuration application (settings, labels, webhooks, branch protection)
/// - Repository type assignment via custom properties
///
/// # Arguments
///
/// * `request` - Type-safe repository creation request with branded types
/// * `metadata_provider` - Provider for loading template configurations from GitHub
/// * `auth_service` - Authentication service for GitHub operations
/// * `metadata_repository_name` - Name of the repository containing organization configuration (e.g., ".reporoller")
///
/// # Returns
///
/// Returns `RepoRollerResult<RepositoryCreationResult>` with repository metadata on success.
///
/// # Errors
///
/// Returns `RepoRollerError` for various failure conditions:
/// - `ValidationError` - Invalid input or configuration
/// - `TemplateError` - Template not found or processing failed
/// - `ConfigurationError` - Configuration resolution failed
/// - `GitHubError` - GitHub API operations failed
/// - `SystemError` - Git operations or file system errors
///
/// # Example
///
/// ```no_run
/// use repo_roller_core::{
///     RepositoryCreationRequestBuilder, RepositoryName,
///     OrganizationName, TemplateName, create_repository
/// };
/// use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
/// use auth_handler::GitHubAuthService;
/// use github_client::GitHubClient;
/// use repo_roller_core::{event_secrets, event_metrics};
///
/// # async fn example(
/// #     visibility_policy_provider: std::sync::Arc<dyn config_manager::VisibilityPolicyProvider>,
/// #     environment_detector: std::sync::Arc<dyn github_client::GitHubEnvironmentDetector>
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo")?,
///     OrganizationName::new("my-org")?,
/// )
/// .template(TemplateName::new("rust-service")?)
/// .build();
///
/// let github_client = github_client::create_token_client("token")?;
/// let metadata_provider = GitHubMetadataProvider::new(
///     GitHubClient::new(github_client),
///     MetadataProviderConfig::explicit(".reporoller")
/// );
/// let auth_service = GitHubAuthService::new(12345, "private-key".to_string());
///
/// let secret_resolver = std::sync::Arc::new(event_secrets::EnvironmentSecretResolver::new());
/// let metrics_registry = prometheus::Registry::new();
/// let metrics = std::sync::Arc::new(event_metrics::PrometheusEventMetrics::new(&metrics_registry));
///
/// let event_context = repo_roller_core::event_publisher::EventNotificationContext::new(
///     "user@example.com",
///     secret_resolver,
///     metrics,
/// );
///
/// let result = create_repository(
///     request,
///     &metadata_provider,
///     &auth_service,
///     ".reporoller",
///     visibility_policy_provider,
///     environment_detector,
///     event_context,
/// ).await?;
/// println!("Created repository: {}", result.repository_url);
/// # Ok(())
/// # }
/// ```
pub async fn create_repository(
    request: RepositoryCreationRequest,
    metadata_provider: &dyn config_manager::MetadataRepositoryProvider,
    auth_service: &dyn auth_handler::UserAuthenticationService,
    metadata_repository_name: &str,
    visibility_policy_provider: std::sync::Arc<dyn visibility::VisibilityPolicyProvider>,
    environment_detector: std::sync::Arc<dyn visibility::GitHubEnvironmentDetector>,
    event_context: event_publisher::EventNotificationContext,
) -> RepoRollerResult<RepositoryCreationResult> {
    info!(
        "Starting repository creation: name='{}', owner='{}', template={:?}, strategy={:?}",
        request.name, request.owner, request.template, request.content_strategy
    );

    // Steps 1–2: Authenticate and create GitHub clients.
    let clients = setup_github_clients(auth_service, request.owner.as_ref()).await?;

    // Steps 3–4: Resolve merged configuration and load the template config.
    let (merged_config, template) = load_creation_config(
        &clients.installation_token,
        &request,
        metadata_provider,
        metadata_repository_name,
    )
    .await?;

    // Step 5: Resolve repository visibility.
    let visibility_decision = resolve_repository_visibility(
        &request,
        template.as_ref(),
        visibility_policy_provider,
        environment_detector,
    )
    .await?;

    // Step 6: Generate local repository content.
    let local_repo_path = generate_repository_content(
        &request,
        template.as_ref(),
        &merged_config,
        &clients.template_fetcher,
    )
    .await?;

    // Step 7: Initialize the local Git repository and create the initial commit.
    let allow_empty_commit = matches!(request.content_strategy, ContentStrategy::Empty);
    let default_branch = initialize_git_repository(
        &local_repo_path,
        &clients.installation_repo_client,
        request.owner.as_ref(),
        allow_empty_commit,
    )
    .await?;

    // Step 8: Create the repository on GitHub.
    let repo = create_github_repository(
        &request,
        &merged_config,
        &clients.installation_repo_client,
        visibility_decision.visibility,
    )
    .await?;

    // Step 9: Push local content to the GitHub remote.
    push_repository_to_github(
        &local_repo_path,
        repo.url(),
        &default_branch,
        &clients.installation_token,
    )?;

    // Steps 10–11: Apply merged configuration and repository permissions.
    apply_post_creation_settings(
        &clients.installation_repo_client,
        &request,
        &merged_config,
        template.as_ref(),
        &event_context.created_by,
    )
    .await?;

    info!("Repository creation completed successfully");

    // Step 12: Build the result.
    let result = RepositoryCreationResult {
        repository_url: repo.url().to_string(),
        repository_id: repo.node_id().to_string(),
        created_at: Timestamp::now(),
        default_branch: default_branch.clone(),
    };

    // Step 13: Fire-and-forget event notification.
    spawn_event_notification(&result, request, merged_config, event_context);

    Ok(result)
}
