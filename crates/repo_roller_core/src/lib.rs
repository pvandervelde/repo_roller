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
//! // Create the repository
//! match create_repository(
//!     request,
//!     &metadata_provider,
//!     &auth_service,
//!     ".reporoller", // Metadata repository name
//!     visibility_policy_provider,
//!     environment_detector,
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

/// Create a new repository from a template with type-safe API.
///
/// This function orchestrates the complete repository creation workflow with
/// type-safe branded types and comprehensive error handling.
///
/// ## Workflow Overview
///
/// 1. **Authentication**: Set up GitHub App authentication and get installation token
/// 2. **Configuration Resolution**: Use OrganizationSettingsManager to resolve hierarchical configuration
/// 3. **Local Repository Preparation**: Create temp directory, fetch template, process variables
/// 4. **Git Initialization**: Initialize local Git repository with correct default branch
/// 5. **GitHub Repository Creation**: Create repository via GitHub API
/// 6. **Configuration Application**: Apply resolved settings to GitHub repository
///
/// ## Parameters
///
/// * `request` - Type-safe repository creation request with branded types
/// * `config` - Configuration containing template definitions
/// * `app_id` - GitHub App ID for authentication
/// * `app_key` - GitHub App private key for authentication
///
/// ## Returns
///
/// * `Ok(RepositoryCreationResult)` - Repository created successfully with metadata (url, id, created_at, default_branch)
/// * `Err(RepoRollerError)` - Creation failed with categorized error
///
/// ## Error Types
///
/// - `ValidationError` - Invalid input parameters or missing required data
/// - `AuthenticationError` - GitHub App authentication failures
/// - `ConfigurationError` - Template or configuration resolution failures
/// - `TemplateError` - Template fetching or processing errors
/// - `GitHubError` - GitHub API operation failures
/// - `RepositoryError` - Git operations or repository setup failures
/// - `SystemError` - File system or internal errors
///
/// ## Examples
///
/// ```no_run
/// use repo_roller_core::{
///     RepositoryCreationRequestBuilder, RepositoryName,
///     OrganizationName, TemplateName, create_repository
/// };
/// use config_manager::{GitHubMetadataProvider, MetadataProviderConfig};
/// use auth_handler::{GitHubAuthService, UserAuthenticationService};
/// use github_client;
///
/// # async fn example(
/// #     visibility_policy_provider: std::sync::Arc<dyn config_manager::VisibilityPolicyProvider>,
/// #     environment_detector: std::sync::Arc<dyn github_client::GitHubEnvironmentDetector>
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-service")?,
///     OrganizationName::new("my-org")?,
/// )
/// .template(TemplateName::new("rust-service")?)
/// .variable("author", "Jane Doe")
/// .build();
///
/// let auth_service = GitHubAuthService::new(12345, "private-key".to_string());
/// let token = auth_service.get_installation_token_for_org("my-org").await?;
/// let github_client = github_client::create_token_client(&token)?;
/// let github_client = github_client::GitHubClient::new(github_client);
///
/// let metadata_provider = GitHubMetadataProvider::new(
///     github_client,
///     MetadataProviderConfig::explicit(".reporoller")
/// );
///
/// match create_repository(request, &metadata_provider, &auth_service, ".reporoller", visibility_policy_provider, environment_detector).await {
///     Ok(result) => {
///         println!("Created: {}", result.repository_url);
///         println!("ID: {}", result.repository_id);
///         println!("Branch: {}", result.default_branch);
///     }
///     Err(e) => eprintln!("Failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
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
/// let result = create_repository(
///     request,
///     &metadata_provider,
///     &auth_service,
///     ".reporoller",
///     visibility_policy_provider,
///     environment_detector,
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
) -> RepoRollerResult<RepositoryCreationResult> {
    info!(
        "Starting repository creation: name='{}', owner='{}', template={:?}, strategy={:?}",
        request.name, request.owner, request.template, request.content_strategy
    );

    // Step 1: Setup GitHub authentication and get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(request.owner.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to authenticate: {}", e);
            RepoRollerError::GitHub(GitHubError::AuthenticationFailed {
                reason: format!("Failed to get installation token: {}", e),
            })
        })?;

    // Create GitHub client with installation token
    let installation_client =
        github_client::create_token_client(&installation_token).map_err(|e| {
            error!("Failed to create installation token client: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create installation token client: {}", e),
            })
        })?;
    let installation_repo_client = GitHubClient::new(installation_client);

    // Step 2: Create template fetcher
    // For now, reuse installation token - in the future this could be separate
    let template_fetcher_client =
        github_client::create_token_client(&installation_token).map_err(|e| {
            error!("Failed to create GitHub client for template fetcher: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create GitHub client: {}", e),
            })
        })?;
    let template_fetcher =
        template_engine::GitHubTemplateFetcher::new(GitHubClient::new(template_fetcher_client));

    // Step 3: Resolve organization configuration
    // Template name is optional - use empty string if not provided
    let template_name_for_config = request.template.as_ref().map(|t| t.as_ref()).unwrap_or("");
    let merged_config = configuration::resolve_organization_configuration(
        &installation_token,
        request.owner.as_ref(),
        template_name_for_config,
        metadata_repository_name,
    )
    .await?;

    // Step 4: Load template configuration from GitHub (if template provided)
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

    // Step 5: Resolve repository visibility
    info!(
        "Resolving repository visibility for organization '{}'",
        request.owner
    );
    let visibility_request = visibility::VisibilityRequest {
        organization: request.owner.clone(),
        user_preference: request.visibility,
        template_default: template.as_ref().and_then(|t| t.default_visibility),
    };

    let visibility_resolver =
        visibility::VisibilityResolver::new(visibility_policy_provider, environment_detector);

    let visibility_decision = visibility_resolver
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
                    // For PolicyNotFound, we should use unrestricted policy
                    // This error shouldn't reach here if resolver handles it correctly
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
        visibility_decision.visibility, visibility_decision.source
    );
    debug!(
        "Applied constraints: {:?}",
        visibility_decision.constraints_applied
    );

    // Construct source repository identifier for template fetcher (if template exists)
    // Format: "org/repo" (e.g., "my-org/template-rust-library")
    let template_source = request
        .template
        .as_ref()
        .map(|t| format!("{}/{}", request.owner.as_ref(), t.as_ref()))
        .unwrap_or_default();

    // Step 6: Generate repository content based on strategy
    let content_provider: Box<dyn crate::ContentProvider> = match request.content_strategy {
        crate::ContentStrategy::Template => {
            Box::new(crate::TemplateBasedContentProvider::new(&template_fetcher))
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

    let local_repo_path = content_provider
        .provide_content(
            &request,
            template.as_ref(),
            &template_source,
            &merged_config,
        )
        .await?;

    // Step 7: Initialize Git repository and commit
    // For empty repositories, we allow empty commits
    let allow_empty_commit = matches!(request.content_strategy, ContentStrategy::Empty);
    let default_branch = initialize_git_repository(
        &local_repo_path,
        &installation_repo_client,
        request.owner.as_ref(),
        allow_empty_commit,
    )
    .await?;

    // Step 8: Create repository on GitHub
    let repo = create_github_repository(
        &request,
        &merged_config,
        &installation_repo_client,
        visibility_decision.visibility,
    )
    .await?;

    // Step 9: Push local repository to GitHub
    info!("Pushing local repository to remote: {}", repo.url());
    git::push_to_origin(
        &local_repo_path,
        repo.url(),
        &default_branch,
        &installation_token,
    )
    .map_err(|e| {
        error!("Failed to push to origin: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to push to origin: {}", e),
        })
    })?;

    info!("Repository successfully pushed to GitHub");

    // Step 10: Apply merged configuration to repository
    configuration::apply_repository_configuration(
        &installation_repo_client,
        request.owner.as_ref(),
        request.name.as_ref(),
        &merged_config,
    )
    .await?;

    info!("Repository creation completed successfully");

    // Step 11: Return success result with repository metadata
    Ok(RepositoryCreationResult {
        repository_url: repo.url().to_string(),
        repository_id: repo.node_id().to_string(),
        created_at: Timestamp::now(),
        default_branch: default_branch.clone(),
    })
}
