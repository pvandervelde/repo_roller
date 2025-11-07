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
//! use config_manager::Config;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a type-safe repository creation request
//! let request = RepositoryCreationRequestBuilder::new(
//!     RepositoryName::new("my-new-project")?,
//!     OrganizationName::new("my-organization")?,
//!     TemplateName::new("rust-library")?,
//! )
//! .variable("author", "Jane Doe")
//! .build();
//!
//! // Load configuration with available templates
//! let config = Config { templates: vec![] }; // Would be loaded from config file
//!
//! // Create the repository
//! match create_repository(
//!     request,
//!     &config,
//!     12345, // GitHub App ID
//!     "private-key-content".to_string(), // GitHub App private key
//!     ".reporoller" // Metadata repository name
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
//! ## Architecture
//!
//! The crate follows a dependency injection pattern for testability:
//! - [`TemplateFetcher`] trait for retrieving template files
//! - [`RepositoryClient`] trait for GitHub API operations
//! - Configuration-driven template processing via [`template_engine`]
//!
//! ## Error Handling
//!
//! All operations return [`RepoRollerResult<T>`] which provides structured error
//! information with domain-specific error types. Internal operations use the [`Error`] type for
//! detailed error context.

use github_client::{create_app_client, GitHubClient, RepositoryClient, RepositoryCreatePayload};
use temp_dir::TempDir;
use tracing::{debug, error, info, warn};

mod errors;

// Git operations module
mod git;

// Configuration resolution and application module
mod configuration;

// GitHub App authentication module
mod github_auth;

// Template processing operations module
mod template_processing;
// Re-export for testing
pub use template_processing::extract_config_variables;

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

// Re-export commonly used types
pub use authentication::{SessionId, UserId};
pub use github::{GitHubToken, InstallationId};
pub use repository::{OrganizationName, RepositoryName};
pub use request::{
    RepositoryCreationRequest, RepositoryCreationRequestBuilder, RepositoryCreationResult,
};
pub use template::TemplateName;

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

    /// Create a timestamp from a DateTime<Utc>
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Get the inner DateTime<Utc>
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
/// use config_manager::Config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-service")?,
///     OrganizationName::new("my-org")?,
///     TemplateName::new("rust-service")?,
/// )
/// .variable("author", "Jane Doe")
/// .build();
///
/// let config = Config { templates: vec![] };
///
/// match create_repository(request, &config, 12345, "private-key".to_string(), ".reporoller").await {
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

    debug!("Committing initial changes");
    git::commit_all_changes(local_repo_path, "Initial commit").map_err(|e| {
        error!("Failed to commit changes: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to commit changes: {}", e),
        })
    })?;

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
) -> RepoRollerResult<github_client::models::Repository> {
    let payload = RepositoryCreatePayload {
        name: request.name.as_ref().to_string(),
        has_issues: merged_config.repository.issues.as_ref().map(|v| v.value),
        has_projects: merged_config.repository.projects.as_ref().map(|v| v.value),
        has_wiki: merged_config.repository.wiki.as_ref().map(|v| v.value),
        ..Default::default()
    };

    info!("Creating GitHub repository: name='{}'", request.name);
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
/// * `config` - Application configuration containing template definitions
/// * `app_id` - GitHub App ID for authentication
/// * `app_key` - GitHub App private key for authentication
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
/// use config_manager::Config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequestBuilder::new(
///     RepositoryName::new("my-repo")?,
///     OrganizationName::new("my-org")?,
///     TemplateName::new("rust-service")?,
/// )
/// .build();
///
/// let config = Config { templates: vec![] };
/// let result = create_repository(
///     request,
///     &config,
///     12345,
///     "private-key".to_string(),
///     ".reporoller"
/// ).await?;
/// println!("Created repository: {}", result.repository_url);
/// # Ok(())
/// # }
/// ```
pub async fn create_repository(
    request: RepositoryCreationRequest,
    config: &config_manager::Config,
    app_id: u64,
    app_key: String,
    metadata_repository_name: &str,
) -> RepoRollerResult<RepositoryCreationResult> {
    info!(
        "Starting repository creation: name='{}', owner='{}', template='{}'",
        request.name, request.owner, request.template
    );

    // Step 1: Setup GitHub authentication
    let (installation_token, installation_repo_client) =
        github_auth::setup_github_authentication(app_id, &app_key, request.owner.as_ref()).await?;

    // Step 2: Create template fetcher for later use
    let app_client = create_app_client(app_id, &app_key).await.map_err(|e| {
        error!(
            "Failed to create GitHub App client for template fetcher: {}",
            e
        );
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create GitHub App client: {}", e),
        })
    })?;
    let template_fetcher =
        template_engine::GitHubTemplateFetcher::new(GitHubClient::new(app_client));

    // Step 3: Resolve organization configuration
    let merged_config = configuration::resolve_organization_configuration(
        &installation_token,
        request.owner.as_ref(),
        request.template.as_ref(),
        metadata_repository_name,
    )
    .await?;

    // Step 4: Find template configuration
    debug!(
        "Searching for template '{}' in configuration",
        request.template
    );
    let template = config
        .templates
        .iter()
        .find(|t| t.name == request.template.as_ref())
        .ok_or_else(|| {
            error!("Template '{}' not found in configuration", request.template);
            RepoRollerError::Template(TemplateError::TemplateNotFound {
                name: request.template.as_ref().to_string(),
            })
        })?;

    info!("Template '{}' found in configuration", request.template);

    // Step 5: Prepare local repository with template files
    let local_repo_path = template_processing::prepare_local_repository(
        &request,
        template,
        &template_fetcher,
        &merged_config,
    )
    .await?;

    // Step 6: Initialize Git repository and commit
    let default_branch = initialize_git_repository(
        &local_repo_path,
        &installation_repo_client,
        request.owner.as_ref(),
    )
    .await?;

    // Step 7: Create repository on GitHub
    let repo =
        create_github_repository(&request, &merged_config, &installation_repo_client).await?;

    // Step 8: Push local repository to GitHub
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

    // Step 9: Apply merged configuration to repository
    configuration::apply_repository_configuration(
        &installation_repo_client,
        request.owner.as_ref(),
        request.name.as_ref(),
        &merged_config,
    )
    .await?;

    info!("Repository creation completed successfully");

    // Step 10: Return success result with repository metadata
    Ok(RepositoryCreationResult {
        repository_url: repo.url().to_string(),
        repository_id: repo.node_id().to_string(),
        created_at: Timestamp::now(),
        default_branch: default_branch.clone(),
    })
}
