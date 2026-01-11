// GENERATED FROM: specs/interfaces/content-providers.md
// This file contains trait definitions and type stubs for content providers.
// Implementations will be added by the coder.

//! Content providers for repository initialization.
//!
//! This module provides the [`ContentProvider`] trait and implementations for
//! generating repository content during creation. Content providers follow the
//! Strategy pattern, allowing different content generation strategies without
//! modifying the repository creation workflow.
//!
//! # Providers
//!
//! - [`TemplateBasedContentProvider`]: Fetches and processes template files (current behavior)
//! - [`ZeroContentProvider`]: Creates no files (empty repository)
//! - [`CustomInitContentProvider`]: Creates selected initialization files
//!
//! # Examples
//!
//! ```no_run
//! use repo_roller_core::{TemplateBasedContentProvider, ContentProvider};
//! use template_engine::GitHubTemplateFetcher;
//!
//! # async fn example(
//! #     request: &repo_roller_core::RepositoryCreationRequest,
//! #     template: &config_manager::TemplateConfig,
//! #     merged_config: &config_manager::MergedConfiguration,
//! #     fetcher: &dyn template_engine::TemplateFetcher,
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Use template-based provider
//! let provider = TemplateBasedContentProvider::new(fetcher);
//! let temp_dir = provider.provide_content(
//!     request,
//!     Some(template),
//!     "org/template-repo",
//!     merged_config
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! See specs/interfaces/content-providers.md for complete specification.

use crate::errors::SystemError;
use crate::request::RepositoryCreationRequest;
use crate::{RepoRollerError, RepoRollerResult};
use temp_dir::TempDir;
use tracing::{error, info};

/// Provides content for repository initialization.
///
/// This trait defines the strategy for populating a repository with files
/// during creation. Different implementations support different content
/// strategies: template-based, empty, or custom initialization.
///
/// # Architecture
///
/// The ContentProvider follows the Strategy pattern, allowing the repository
/// creation workflow to remain unchanged while supporting multiple content
/// strategies. The workflow delegates content generation to the provider:
///
/// ```text
/// create_repository()
///     → ContentProvider.provide_content()
///     → Returns TempDir with content
///     → Git init & commit
///     → Push to GitHub
/// ```
///
/// # Implementations
///
/// - [`TemplateBasedContentProvider`]: Fetches and processes template files (current behavior)
/// - [`ZeroContentProvider`]: Creates no files (empty repository)
/// - [`CustomInitContentProvider`]: Creates selected initialization files (README, .gitignore)
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::{ContentProvider, TemplateBasedContentProvider};
/// use template_engine::GitHubTemplateFetcher;
///
/// # async fn example(
/// #     request: &repo_roller_core::RepositoryCreationRequest,
/// #     template: &config_manager::TemplateConfig,
/// #     merged_config: &config_manager::MergedConfiguration,
/// #     fetcher: &dyn template_engine::TemplateFetcher,
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// // Use template-based provider
/// let provider = TemplateBasedContentProvider::new(fetcher);
/// let temp_dir = provider.provide_content(
///     request,
///     Some(template),
///     "org/template-repo",
///     merged_config
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// See specs/interfaces/content-providers.md for complete contract specification.
#[async_trait::async_trait]
pub trait ContentProvider: Send + Sync {
    /// Provide content for a new repository.
    ///
    /// Creates and populates a temporary directory with repository content
    /// according to the provider's strategy.
    ///
    /// # Parameters
    ///
    /// * `request` - Repository creation request with name, owner, variables
    /// * `template_config` - Template configuration (may be None for non-template modes)
    /// * `template_source` - Template source identifier (e.g., "org/repo", may be empty for non-template)
    /// * `merged_config` - Merged organization configuration providing variables
    ///
    /// # Returns
    ///
    /// * `Ok(TempDir)` - Temporary directory with repository content
    /// * `Err(RepoRollerError)` - If content generation fails
    ///
    /// # Implementation Notes
    ///
    /// - The returned `TempDir` will be automatically cleaned up when dropped
    /// - Content should be ready for Git initialization (no `.git` directory)
    /// - Files should be in their final form (variables substituted, etc.)
    ///
    /// See specs/interfaces/content-providers.md#contentprovider-trait for complete specification.
    async fn provide_content(
        &self,
        request: &RepositoryCreationRequest,
        template_config: Option<&config_manager::TemplateConfig>,
        template_source: &str,
        merged_config: &config_manager::MergedConfiguration,
    ) -> RepoRollerResult<TempDir>;
}

/// Content provider that fetches and processes template files.
///
/// This is the current/default behavior: fetch template from GitHub,
/// copy files, substitute variables, create additional files.
///
/// # Behavior
///
/// 1. Fetches template files from source repository
/// 2. Copies template files to temporary directory
/// 3. Performs variable substitution in all files
/// 4. Creates additional files (README.md, .gitignore) if not provided by template
///
/// # Template Requirements
///
/// - Requires `template_config` to be Some (contains variable definitions)
/// - Requires valid `template_source` (GitHub org/repo identifier)
/// - Uses TemplateFetcher to retrieve files
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::{TemplateBasedContentProvider, ContentProvider};
/// use template_engine::GitHubTemplateFetcher;
/// use github_client::GitHubClient;
///
/// # async fn example(
/// #     request: &repo_roller_core::RepositoryCreationRequest,
/// #     template: &config_manager::TemplateConfig,
/// #     merged_config: &config_manager::MergedConfiguration,
/// #     github_client: GitHubClient,
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let fetcher = GitHubTemplateFetcher::new(github_client);
/// let provider = TemplateBasedContentProvider::new(&fetcher);
///
/// let temp_dir = provider.provide_content(
///     request,
///     Some(template),
///     "my-org/template-rust-library",
///     merged_config,
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// See specs/interfaces/content-providers.md#templatebasedcontentprovider for complete specification.
pub struct TemplateBasedContentProvider<'a> {
    /// Template fetcher for retrieving files from source repository
    fetcher: &'a dyn template_engine::TemplateFetcher,
}

impl<'a> TemplateBasedContentProvider<'a> {
    /// Create a new template content provider.
    ///
    /// # Parameters
    ///
    /// * `fetcher` - Template fetcher implementation for retrieving files
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use repo_roller_core::TemplateBasedContentProvider;
    /// use template_engine::GitHubTemplateFetcher;
    /// use github_client::GitHubClient;
    ///
    /// # fn example(github_client: GitHubClient) {
    /// let fetcher = GitHubTemplateFetcher::new(github_client);
    /// let provider = TemplateBasedContentProvider::new(&fetcher);
    /// # }
    /// ```
    pub fn new(fetcher: &'a dyn template_engine::TemplateFetcher) -> Self {
        Self { fetcher }
    }
}

#[async_trait::async_trait]
impl<'a> ContentProvider for TemplateBasedContentProvider<'a> {
    async fn provide_content(
        &self,
        request: &RepositoryCreationRequest,
        template_config: Option<&config_manager::TemplateConfig>,
        template_source: &str,
        merged_config: &config_manager::MergedConfiguration,
    ) -> RepoRollerResult<TempDir> {
        // Validate template_config is provided
        let template = template_config.ok_or_else(|| {
            RepoRollerError::System(SystemError::Internal {
                reason: "TemplateBasedContentProvider requires template configuration".to_string(),
            })
        })?;

        // Validate template_source is not empty
        if template_source.is_empty() {
            return Err(RepoRollerError::System(SystemError::Internal {
                reason: "Template source must be provided for template-based creation".to_string(),
            }));
        }

        // Call existing prepare_local_repository logic
        // This function already does:
        // 1. Create TempDir
        // 2. Fetch template files
        // 3. Copy files
        // 4. Replace variables
        // 5. Create additional files
        crate::template_processing::prepare_local_repository(
            request,
            template,
            template_source,
            self.fetcher,
            merged_config,
        )
        .await
    }
}

/// Content provider that creates no files (empty repository).
///
/// This provider creates a completely empty temporary directory,
/// resulting in a repository with no initial files or commit.
///
/// # Behavior
///
/// - Creates empty temporary directory
/// - No files are created
/// - When combined with Git initialization, creates an empty repository
///
/// # Use Cases
///
/// - Migration scenarios where content will be added separately
/// - Blank slate repositories for experimentation
/// - Using template configuration for settings only (no files)
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::{ZeroContentProvider, ContentProvider};
///
/// # async fn example(
/// #     request: &repo_roller_core::RepositoryCreationRequest,
/// #     template: Option<&config_manager::TemplateConfig>,
/// #     merged_config: &config_manager::MergedConfiguration,
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ZeroContentProvider::new();
/// let temp_dir = provider.provide_content(
///     request,
///     template,
///     "",
///     merged_config,
/// ).await?;
/// // temp_dir is empty, no files created
/// # Ok(())
/// # }
/// ```
///
/// See specs/interfaces/content-providers.md#zerocontentprovider for complete specification.
pub struct ZeroContentProvider;

impl ZeroContentProvider {
    /// Create a new empty content provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_roller_core::ZeroContentProvider;
    ///
    /// let provider = ZeroContentProvider::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZeroContentProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ContentProvider for ZeroContentProvider {
    async fn provide_content(
        &self,
        _request: &RepositoryCreationRequest,
        _template_config: Option<&config_manager::TemplateConfig>,
        _template_source: &str,
        _merged_config: &config_manager::MergedConfiguration,
    ) -> RepoRollerResult<TempDir> {
        info!("Creating empty repository (no files)");

        // Create empty temporary directory
        let local_repo_path = TempDir::new().map_err(|e| {
            error!("Failed to create temporary directory: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create temporary directory: {}", e),
            })
        })?;

        info!("Empty repository directory created");
        Ok(local_repo_path)
    }
}

/// Content provider that creates custom initialization files.
///
/// This provider creates a minimal set of initialization files based on
/// user preferences: README.md and/or .gitignore.
///
/// # Behavior
///
/// - Creates temporary directory
/// - Generates README.md if requested (with repository name, owner)
/// - Generates .gitignore if requested (with common patterns)
/// - Uses simple variable substitution (repo name, owner, template name)
///
/// # Configuration
///
/// Files are created based on the initialization options provided:
/// - `include_readme`: Generate basic README.md
/// - `include_gitignore`: Generate .gitignore with common patterns
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::{CustomInitContentProvider, CustomInitOptions, ContentProvider};
///
/// # async fn example(
/// #     request: &repo_roller_core::RepositoryCreationRequest,
/// #     template: Option<&config_manager::TemplateConfig>,
/// #     merged_config: &config_manager::MergedConfiguration,
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// let options = CustomInitOptions {
///     include_readme: true,
///     include_gitignore: true,
/// };
/// let provider = CustomInitContentProvider::new(options);
///
/// let temp_dir = provider.provide_content(
///     request,
///     template,
///     "",
///     merged_config,
/// ).await?;
/// // temp_dir contains README.md and .gitignore
/// # Ok(())
/// # }
/// ```
///
/// See specs/interfaces/content-providers.md#custominitcontentprovider for complete specification.
pub struct CustomInitContentProvider {
    /// Initialization options specifying which files to create
    options: CustomInitOptions,
}

/// Options for custom repository initialization.
///
/// Specifies which standard files should be created during repository
/// initialization.
///
/// # Examples
///
/// ```
/// use repo_roller_core::CustomInitOptions;
///
/// // Create both files
/// let options = CustomInitOptions {
///     include_readme: true,
///     include_gitignore: true,
/// };
///
/// // README only
/// let options = CustomInitOptions {
///     include_readme: true,
///     include_gitignore: false,
/// };
/// ```
///
/// See specs/interfaces/content-providers.md#custominitcontentprovider for complete specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomInitOptions {
    /// Create a basic README.md file with repository information
    pub include_readme: bool,

    /// Create a .gitignore file with common ignore patterns
    pub include_gitignore: bool,
}

impl CustomInitContentProvider {
    /// Create a new custom initialization content provider.
    ///
    /// # Parameters
    ///
    /// * `options` - Configuration specifying which files to create
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_roller_core::{CustomInitContentProvider, CustomInitOptions};
    ///
    /// let options = CustomInitOptions {
    ///     include_readme: true,
    ///     include_gitignore: true,
    /// };
    /// let provider = CustomInitContentProvider::new(options);
    /// ```
    pub fn new(options: CustomInitOptions) -> Self {
        Self { options }
    }
}

#[async_trait::async_trait]
impl ContentProvider for CustomInitContentProvider {
    async fn provide_content(
        &self,
        request: &RepositoryCreationRequest,
        template_config: Option<&config_manager::TemplateConfig>,
        _template_source: &str,
        _merged_config: &config_manager::MergedConfiguration,
    ) -> RepoRollerResult<TempDir> {
        info!("Creating repository with custom initialization files");

        // Create temporary directory
        let local_repo_path = TempDir::new().map_err(|e| {
            error!("Failed to create temporary directory: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create temporary directory: {}", e),
            })
        })?;

        // Create README.md if requested
        if self.options.include_readme {
            create_readme_file(&local_repo_path, request, template_config)?;
        }

        // Create .gitignore if requested
        if self.options.include_gitignore {
            create_gitignore_file(&local_repo_path)?;
        }

        info!(
            "Custom initialization files created: readme={}, gitignore={}",
            self.options.include_readme, self.options.include_gitignore
        );

        Ok(local_repo_path)
    }
}

/// Create a README.md file with repository information.
///
/// Generates a basic README with repository name, owner, and optional
/// template information.
///
/// See specs/interfaces/content-providers.md#file-generation for content format.
fn create_readme_file(
    local_repo_path: &TempDir,
    request: &RepositoryCreationRequest,
    template_config: Option<&config_manager::TemplateConfig>,
) -> RepoRollerResult<()> {
    let readme_path = local_repo_path.path().join("README.md");

    let readme_content = if let Some(_template) = template_config {
        // Template provided - include template reference
        if let Some(ref template_name) = request.template {
            format!(
                "# {}\n\nRepository created using RepoRoller with template '{}'.\n\n**Organization:** {}\n**Created:** {}\n",
                request.name.as_ref(),
                template_name.as_ref(),
                request.owner.as_ref(),
                chrono::Utc::now().format("%Y-%m-%d")
            )
        } else {
            // Template config exists but no template name (shouldn't happen, but handle gracefully)
            format!(
                "# {}\n\nRepository created using RepoRoller.\n\n**Organization:** {}\n**Created:** {}\n",
                request.name.as_ref(),
                request.owner.as_ref(),
                chrono::Utc::now().format("%Y-%m-%d")
            )
        }
    } else {
        // No template - basic README
        format!(
            "# {}\n\nRepository created using RepoRoller.\n\n**Organization:** {}\n**Created:** {}\n",
            request.name.as_ref(),
            request.owner.as_ref(),
            chrono::Utc::now().format("%Y-%m-%d")
        )
    };

    std::fs::write(&readme_path, readme_content).map_err(|e| {
        error!("Failed to create README.md: {}", e);
        RepoRollerError::System(SystemError::FileSystem {
            operation: "create README.md".to_string(),
            reason: e.to_string(),
        })
    })?;

    info!("README.md created at: {:?}", readme_path);
    Ok(())
}

/// Create a .gitignore file with common patterns.
///
/// Generates a .gitignore with commonly ignored files and directories.
///
/// See specs/interfaces/content-providers.md#file-generation for content format.
fn create_gitignore_file(local_repo_path: &TempDir) -> RepoRollerResult<()> {
    let gitignore_path = local_repo_path.path().join(".gitignore");

    let gitignore_content = "\
# Operating System Files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor and IDE
.vscode/
.idea/
*.swp
*.swo
*~

# Logs and Temporary Files
*.log
*.tmp
*.temp
*.bak

# Build Outputs
target/
dist/
build/
out/
*.o
*.so
*.exe

# Dependencies
node_modules/
vendor/
";

    std::fs::write(&gitignore_path, gitignore_content).map_err(|e| {
        error!("Failed to create .gitignore: {}", e);
        RepoRollerError::System(SystemError::FileSystem {
            operation: "create .gitignore".to_string(),
            reason: e.to_string(),
        })
    })?;

    info!(".gitignore created at: {:?}", gitignore_path);
    Ok(())
}

#[cfg(test)]
#[path = "content_providers_tests.rs"]
mod tests;
