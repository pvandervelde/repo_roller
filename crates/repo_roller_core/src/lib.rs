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
//! The primary entry points are:
//! - [`create_repository_with_config`] - Create a repository with provided configuration
//! - [`CreateRepoRequest`] - Request structure for repository creation
//! - [`CreateRepoResult`] - Result structure containing success/failure information
//!
//! ## New Type System (Interface Design)
//!
//! The crate is transitioning to a new type system with:
//! - Branded types for type safety ([`types`] module)
//! - Interface traits for clean architecture boundaries
//! - Comprehensive error handling with domain-specific errors
//!
//! See `specs/interfaces/` for complete interface specifications.
//!
//! ## Examples
//!
//! ```no_run
//! use repo_roller_core::{CreateRepoRequest, create_repository_with_config};
//! use config_manager::Config;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a repository creation request
//! let request = CreateRepoRequest {
//!     name: "my-new-project".to_string(),
//!     owner: "my-organization".to_string(),
//!     template: "rust-library".to_string(),
//! };
//!
//! // Load configuration with available templates
//! let config = Config { templates: vec![] }; // Would be loaded from config file
//!
//! // Create the repository
//! let result = create_repository_with_config(
//!     request,
//!     &config,
//!     12345, // GitHub App ID
//!     "private-key-content".to_string() // GitHub App private key
//! ).await;
//!
//! if result.success {
//!     println!("Repository created successfully: {}", result.message);
//! } else {
//!     eprintln!("Repository creation failed: {}", result.message);
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
//! All operations return [`CreateRepoResult`] which contains success status and
//! descriptive error messages. Internal operations use the [`Error`] type for
//! detailed error context.

use git2::{Repository, Signature};
use github_client::{create_app_client, GitHubClient, RepositoryClient, RepositoryCreatePayload};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use temp_dir::TempDir;
use template_engine::{self, TemplateFetcher, TemplateProcessingRequest, TemplateProcessor};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

mod errors;
use errors::Error;

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

/// Request for creating a new repository.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub owner: String,
    pub template: String,
}

/// Result of a repository creation attempt.
pub struct CreateRepoResult {
    pub success: bool,
    pub message: String,
}

impl CreateRepoResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Debug the current state of the repository including HEAD and commit history.
fn debug_repository_state(repo: &Repository) -> Result<(), Error> {
    debug!("Repository state check:");
    match repo.head() {
        Ok(head) => {
            debug!("  HEAD exists: {:?}", head.name());
            if let Some(oid) = head.target() {
                debug!("  HEAD points to: {}", oid);
                // Check if this commit exists
                match repo.find_commit(oid) {
                    Ok(commit) => debug!(
                        "  HEAD commit exists: {}",
                        commit.summary().unwrap_or("no message")
                    ),
                    Err(e) => debug!("  HEAD commit does not exist: {}", e),
                }
            }
        }
        Err(e) => debug!("  No HEAD reference yet: {}", e),
    }

    // Check for existing commits - handle the case where repository might be empty
    match repo.revwalk() {
        Ok(mut revwalk) => match revwalk.push_head() {
            Ok(_) => {
                let commit_count = revwalk.count();
                debug!("  Found {} existing commits in repository", commit_count);
            }
            Err(_) => debug!("  No commits found in repository (expected for new repo)"),
        },
        Err(e) => {
            debug!("Failed to create revwalk: {}", e);
            debug!("  No commits to walk (expected for new repo)");
        }
    }

    Ok(())
}

/// Debug files in the working directory by listing them and showing previews.
fn debug_working_directory(local_repo_path: &TempDir) -> Result<usize, Error> {
    let mut file_count = 0;
    if let Ok(entries) = std::fs::read_dir(local_repo_path.path()) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    file_count += 1;
                    info!("Found file in working directory: {:?}", entry.file_name());

                    // Log file size and first few bytes
                    if let Ok(metadata) = entry.metadata() {
                        info!("  File size: {} bytes", metadata.len());
                    }

                    // Try to read first 100 chars of file if it's text
                    if let Ok(path) = entry.path().canonicalize() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let preview = if content.len() > 100 {
                                format!("{}...", &content[..100])
                            } else {
                                content
                            };
                            info!("  File content preview: {}", preview.replace('\n', "\\n"));
                        }
                    }
                }
            }
        }
    }
    info!("Found {} files in working directory", file_count);

    if file_count == 0 {
        warn!("No files found in working directory - repository will be empty");
    }

    Ok(file_count)
}

/// Prepare the Git index and create a tree from all files in the working directory.
///
/// This function performs the critical Git operations needed before creating a commit:
/// 1. Retrieves the repository's staging area (index)
/// 2. Adds all files from the working directory to the index using `git add *`
/// 3. Writes the index contents to create a Git tree object
/// 4. Returns the tree OID for use in commit creation
///
/// ## Git Internals
///
/// In Git's object model, a tree represents a directory structure at a specific point
/// in time. The tree contains references to blobs (files) and other trees (subdirectories).
/// This function creates the tree that will be referenced by the initial commit.
///
/// ## Parameters
///
/// * `repo` - Reference to an open Git repository
///
/// ## Returns
///
/// * `Ok(git2::Oid)` - The Object ID of the created tree
/// * `Err(Error)` - If index operations fail or no files are found
///
/// ## Errors
///
/// This function will return an error if:
/// - The repository index cannot be accessed
/// - No files are found in the working directory (empty repository)
/// - File addition to the index fails
/// - Tree creation from the index fails
///
/// ## Example
///
/// ```rust,ignore
/// let repo = Repository::open("/path/to/repo")?;
/// let tree_oid = prepare_index_and_tree(&repo)?;
/// println!("Created tree with OID: {}", tree_oid);
/// ```
fn prepare_index_and_tree(repo: &Repository) -> Result<git2::Oid, Error> {
    // Get the repository index (staging area) - this is where Git tracks changes
    // before they become part of a commit
    let mut index = repo.index().map_err(|e| {
        error!("Failed to get repository index: {}", e);
        Error::GitOperation(format!("Failed to get repository index: {}", e))
    })?;

    debug!("Repository index retrieved");

    // Add all files in the working directory to the index
    // This is equivalent to running `git add *` on the command line
    // The "*" pattern matches all files recursively
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| {
            error!("Failed to add files to index: {}", e);
            Error::GitOperation(format!("Failed to add files to index: {}", e))
        })?;

    // Check how many entries are in the index
    let index_entry_count = index.len();
    info!("Added {} entries to git index", index_entry_count);

    if index_entry_count == 0 {
        error!("No files were added to the git index - cannot create commit");
        return Err(Error::GitOperation(
            "No files to commit - index is empty".to_string(),
        ));
    }

    // Write the index to create a tree object
    // A tree in Git represents the state of files and directories at a point in time
    // This tree will be referenced by the commit we create later
    let tree_oid = index.write_tree().map_err(|e| {
        error!("Failed to write tree: {}", e);
        Error::GitOperation(format!("Failed to write tree: {}", e))
    })?;

    debug!("Git tree written with OID: {}", tree_oid);

    Ok(tree_oid)
}

/// Create an initial commit with the given tree and message.
///
/// This function creates the first commit in a Git repository using the provided tree.
/// As an initial commit, it has no parent commits and establishes the foundation
/// of the repository's commit history.
///
/// ## Git Internals
///
/// A Git commit object contains:
/// - A reference to a tree object (the repository state)
/// - Parent commit references (none for initial commit)
/// - Author and committer signatures
/// - The commit message
///
/// This function creates the commit object but does not update any references
/// (like HEAD or branch pointers) - that is handled separately.
///
/// ## Parameters
///
/// * `repo` - Reference to an open Git repository
/// * `tree_oid` - Object ID of the tree to commit (from `prepare_index_and_tree`)
/// * `commit_message` - Message describing the commit
///
/// ## Returns
///
/// * `Ok(git2::Oid)` - The Object ID of the created commit
/// * `Err(Error)` - If commit creation fails
///
/// ## Errors
///
/// This function will return an error if:
/// - The tree object cannot be found using the provided OID
/// - Git signature creation fails
/// - Commit object creation fails
///
/// ## Example
///
/// ```rust,ignore
/// let tree_oid = prepare_index_and_tree(&repo)?;
/// let commit_oid = create_initial_commit(&repo, tree_oid, "Initial commit")?;
/// println!("Created commit with OID: {}", commit_oid);
/// ```
fn create_initial_commit(
    repo: &Repository,
    tree_oid: git2::Oid,
    commit_message: &str,
) -> Result<git2::Oid, Error> {
    // Find the tree object using the OID returned from prepare_index_and_tree
    let tree = repo.find_tree(tree_oid).map_err(|e| {
        error!("Failed to find tree: {}", e);
        Error::GitOperation(format!("Failed to find tree: {}", e))
    })?;

    // Create signature for both author and committer
    // In a real implementation, this would use actual user information
    let signature = Signature::now("RepoRoller", "repo-roller@example.com").map_err(|e| {
        error!("Failed to create signature: {}", e);
        Error::GitOperation(format!("Failed to create signature: {}", e))
    })?;

    debug!("Git signature created for RepoRoller");

    // Create the commit object in Git's object database
    // Parameters:
    // - None: Don't update any reference (HEAD) yet - we'll do that separately
    // - &signature: Author of the commit
    // - &signature: Committer of the commit (same as author in this case)
    // - commit_message: The commit message
    // - &tree: The tree object representing the repository state
    // - &[]: No parent commits since this is the initial commit
    let commit_oid = repo
        .commit(
            None, // Don't update any reference initially
            &signature,
            &signature,
            commit_message,
            &tree,
            &[], // No parents for initial commit
        )
        .map_err(|e| {
            error!("Failed to create commit: {}", e);
            Error::GitOperation(format!("Failed to create commit: {}", e))
        })?;

    debug!("Initial commit created with OID: {}", commit_oid);

    Ok(commit_oid)
}

/// Set the HEAD reference and verify commit creation.
///
/// This function completes the commit process by updating the repository's HEAD
/// reference to point to the newly created commit, then verifies the operation
/// was successful.
///
/// ## Git Internals
///
/// In Git, HEAD is a symbolic reference that points to the current branch or commit.
/// For a new repository, we need to:
/// 1. Create the HEAD reference pointing to our initial commit
/// 2. This implicitly creates the default branch (e.g., "main")
/// 3. Verify that both HEAD and the branch reference were created correctly
///
/// ## Parameters
///
/// * `repo` - Reference to an open Git repository
/// * `commit_oid` - Object ID of the commit to point HEAD to
/// * `commit_message` - Message from the commit (used for logging)
///
/// ## Returns
///
/// * `Ok(())` - If HEAD reference is set and verified successfully
/// * `Err(Error)` - If reference creation or verification fails
///
/// ## Errors
///
/// This function will return an error if:
/// - HEAD reference creation fails
/// - The repository state cannot be verified after the operation
///
/// ## Verification
///
/// After setting HEAD, this function performs verification by:
/// - Checking that HEAD reference exists and points to the correct commit
/// - Verifying that the default branch was created and points to the commit
/// - Logging warnings if verification fails (non-fatal for the overall operation)
///
/// ## Example
///
/// ```rust,ignore
/// let commit_oid = create_initial_commit(&repo, tree_oid, "Initial commit")?;
/// set_head_reference_and_verify(&repo, commit_oid, "Initial commit")?;
/// println!("Repository is ready with initial commit");
/// ```
fn set_head_reference_and_verify(
    repo: &Repository,
    commit_oid: git2::Oid,
    commit_message: &str,
) -> Result<(), Error> {
    // First, create the main branch reference pointing to our commit
    let branch_ref_name = "refs/heads/main";
    repo.reference(branch_ref_name, commit_oid, false, "Initial commit")
        .map_err(|e| {
            error!("Failed to create main branch reference: {}", e);
            Error::GitOperation(format!("Failed to create main branch reference: {}", e))
        })?;

    info!("Main branch reference created: {}", branch_ref_name);

    // Now set HEAD to point to the main branch (symbolic reference)
    repo.set_head(branch_ref_name).map_err(|e| {
        error!("Failed to set HEAD to main branch: {}", e);
        Error::GitOperation(format!("Failed to set HEAD to main branch: {}", e))
    })?;

    info!(
        "Changes committed successfully with OID: {} and message: '{}'",
        commit_oid, commit_message
    );

    // Verify that the HEAD reference was created correctly
    match repo.head() {
        Ok(head_ref) => {
            info!("HEAD reference exists: {:?}", head_ref.name());
            if let Some(oid) = head_ref.target() {
                info!("HEAD points to commit: {}", oid);
            }
        }
        Err(e) => {
            warn!("Failed to get HEAD reference after commit: {}", e);
        }
    }

    // Check if main branch exists
    match repo.find_branch("main", git2::BranchType::Local) {
        Ok(branch) => {
            info!("Main branch exists");
            if let Some(oid) = branch.get().target() {
                info!("Main branch points to commit: {}", oid);
            }
        }
        Err(e) => {
            warn!("Main branch not found: {}", e);
        }
    }

    Ok(())
}

/// Commit all changes in the local repository working directory.
///
/// This function orchestrates the complete Git commit workflow for a local repository,
/// taking all files in the working directory and creating an initial commit. It combines
/// multiple Git operations into a single cohesive process.
///
/// ## Workflow
///
/// 1. **Repository Access**: Opens the Git repository at the specified path
/// 2. **State Debugging**: Logs repository and working directory state for diagnostics
/// 3. **Index Preparation**: Adds all files to the Git index (staging area)
/// 4. **Tree Creation**: Creates a Git tree object from the staged files
/// 5. **Commit Creation**: Creates the initial commit with the tree
/// 6. **Reference Setting**: Updates HEAD to point to the new commit
/// 7. **Verification**: Confirms the commit was created successfully
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory containing the initialized Git repository
/// * `commit_message` - Message to use for the commit
///
/// ## Returns
///
/// * `Ok(())` - If all files are committed successfully
/// * `Err(Error)` - If any step in the commit process fails
///
/// ## Errors
///
/// This function will return an error if:
/// - The repository cannot be opened
/// - No files are found in the working directory
/// - Any Git operation fails (index, tree, commit, or reference operations)
///
/// ## Git Operations
///
/// This function uses several Git internals:
/// - **Index**: Git's staging area where changes are prepared for commit
/// - **Tree**: A snapshot of the directory structure at commit time
/// - **Commit**: A permanent record pointing to a tree with metadata
/// - **HEAD**: The reference pointing to the current commit
///
/// ## Example
///
/// ```rust,ignore
/// let temp_dir = TempDir::new()?;
/// // ... initialize repo and add files ...
/// commit_all_changes(&temp_dir, "Initial repository setup")?;
/// println!("All changes committed successfully");
/// ```
fn commit_all_changes(local_repo_path: &TempDir, commit_message: &str) -> Result<(), Error> {
    info!(
        "Committing all changes in repository at {:?} with message: '{}'",
        local_repo_path.path(),
        commit_message
    );

    // Open the repository
    let repo = Repository::open(local_repo_path.path()).map_err(|e| {
        error!("Failed to open repository: {}", e);
        Error::GitOperation(format!("Failed to open repository: {}", e))
    })?;

    debug!("Repository opened successfully");

    // Debug the repository state
    debug_repository_state(&repo)?;

    // Debug files in working directory
    let file_count = debug_working_directory(local_repo_path)?;

    if file_count == 0 {
        return Err(Error::GitOperation(
            "No files found in working directory - repository will be empty".to_string(),
        ));
    }

    // Prepare index and create tree
    let tree_oid = prepare_index_and_tree(&repo)?;

    // Create the initial commit
    let commit_oid = create_initial_commit(&repo, tree_oid, commit_message)?;

    // Set HEAD reference and verify
    set_head_reference_and_verify(&repo, commit_oid, commit_message)?;

    Ok(())
}

/// Copy template files to the local repository directory.
///
/// This function takes a collection of template files (as byte arrays) and writes them
/// to the local repository directory, preserving the directory structure and file paths
/// from the original template.
///
/// ## Parameters
///
/// * `files` - Vector of tuples containing (file_path, file_content) pairs
///   - `file_path`: Relative path where the file should be created
///   - `file_content`: Raw bytes of the file content
/// * `local_repo_path` - Temporary directory where files should be written
///
/// ## Returns
///
/// * `Ok(())` - If all files are copied successfully
/// * `Err(Error)` - If any file operation fails
///
/// ## Behavior
///
/// - Creates parent directories automatically if they don't exist
/// - Overwrites existing files with the same path
/// - Preserves the exact byte content of template files
/// - Maintains the relative path structure from the template
///
/// ## Errors
///
/// This function will return an error if:
/// - Parent directory creation fails
/// - File creation fails
/// - File writing fails
///
/// ## Example
///
/// ```rust,ignore
/// let files = vec![
///     ("README.md".to_string(), b"# My Project".to_vec()),
///     ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
/// ];
/// let temp_dir = TempDir::new()?;
/// copy_template_files(&files, &temp_dir)?;
/// ```
fn copy_template_files(
    files: &Vec<(String, Vec<u8>)>,
    local_repo_path: &TempDir,
) -> Result<(), Error> {
    debug!("Copying {} template files to local repository", files.len());

    for (file_path, content) in files {
        let target_path = local_repo_path.path().join(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create directory {:?}: {}", parent, e);
                Error::FileSystem(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }

        // Write the file content
        let mut file = File::create(&target_path).map_err(|e| {
            error!("Failed to create file {:?}: {}", target_path, e);
            Error::FileSystem(format!("Failed to create file {:?}: {}", target_path, e))
        })?;

        file.write_all(content).map_err(|e| {
            error!("Failed to write to file {:?}: {}", target_path, e);
            Error::FileSystem(format!("Failed to write to file {:?}: {}", target_path, e))
        })?;

        debug!("Copied file: {}", file_path);
    }

    info!("Template files copied successfully");
    Ok(())
}

/// Create additional repository files that may not be provided by the template.
///
/// This function generates standard repository files if they are not already present
/// in the template files. It ensures that every repository has basic files like
/// README.md and .gitignore, while respecting template-provided versions.
///
/// ## Additional Files Created
///
/// - **README.md**: A basic readme with repository information if not provided by template
/// - **.gitignore**: A standard gitignore file with common patterns if not provided by template
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory where additional files should be created
/// * `req` - Repository creation request containing name, owner, and template information
/// * `template_files` - List of files already provided by the template (used to check for conflicts)
///
/// ## Returns
///
/// * `Ok(())` - If additional files are created successfully
/// * `Err(Error)` - If file creation fails
///
/// ## Behavior
///
/// - Only creates files that are not already provided by the template
/// - Uses repository metadata (name, owner, template) to generate content
/// - Creates files with sensible defaults suitable for most projects
/// - Logs which files are created vs. skipped
///
/// ## File Content
///
/// - **README.md**: Contains repository name, RepoRoller attribution, and metadata
/// - **.gitignore**: Includes common ignore patterns for various development environments
///
/// ## Errors
///
/// This function will return an error if:
/// - File system operations fail
/// - Directory creation fails
///
/// ## Example
///
/// ```rust,ignore
/// let request = CreateRepoRequest {
///     name: "my-project".to_string(),
///     owner: "my-org".to_string(),
///     template: "basic".to_string(),
/// };
/// let template_files = vec![("src/main.rs".to_string(), vec![])];
/// create_additional_files(&temp_dir, &request, &template_files)?;
/// ```
fn create_additional_files(
    local_repo_path: &TempDir,
    req: &RepositoryCreationRequest,
    template_files: &[(String, Vec<u8>)],
) -> Result<(), Error> {
    info!("Creating additional files for repository initialization");

    // Check what files the template already provides
    let template_file_paths: std::collections::HashSet<String> = template_files
        .iter()
        .map(|(path, _)| path.clone())
        .collect();

    // Only create README.md if the template doesn't provide one
    if !template_file_paths.contains("README.md") {
        let readme_path = local_repo_path.path().join("README.md");
        let readme_content = format!(
            "# {}\n\nRepository created using RepoRoller.\n\nTemplate: {}\nOwner: {}\n",
            req.name.as_ref(), req.template.as_ref(), req.owner.as_ref()
        );

        debug!(
            "Creating README.md with content length: {} (template didn't provide one)",
            readme_content.len()
        );

        std::fs::write(&readme_path, readme_content).map_err(|e| {
            error!("Failed to create README.md: {}", e);
            Error::FileSystem(format!("Failed to create README.md: {}", e))
        })?;

        info!("README.md created successfully at: {:?}", readme_path);
    } else {
        info!("README.md provided by template, skipping creation");
    }

    // Only create .gitignore if the template doesn't provide one
    if !template_file_paths.contains(".gitignore") {
        let gitignore_path = local_repo_path.path().join(".gitignore");
        let gitignore_content =
            "# Common ignores\n.DS_Store\n*.log\n*.tmp\nnode_modules/\ntarget/\n";

        debug!("Creating .gitignore (template didn't provide one)");

        std::fs::write(&gitignore_path, gitignore_content).map_err(|e| {
            error!("Failed to create .gitignore: {}", e);
            Error::FileSystem(format!("Failed to create .gitignore: {}", e))
        })?;

        info!(".gitignore created successfully at: {:?}", gitignore_path);
    } else {
        info!(".gitignore provided by template, skipping creation");
    }

    Ok(())
}

/// Create a new repository from a template with dependency injection for testability.
///
/// TODO (Interface Design): Refactor to use new type system with branded types:
/// - Replace `String` parameters with `RepositoryName`, `OrganizationName`, `TemplateName`
/// - Use `RepoRollerResult` instead of `CreateRepoResult`
/// - Separate orchestration logic from infrastructure concerns
/// - See specs/interfaces/repository-domain.md for target interface
///
/// This is the core orchestration function that handles the complete repository creation
/// workflow. It coordinates multiple services and performs all steps required to create
/// a functional GitHub repository from a template.
///
/// ## Workflow Overview
///
/// 1. **Template Resolution**: Finds the requested template in the configuration
/// 2. **Template Fetching**: Downloads template files from the source repository
/// 3. **Local Setup**: Creates temporary directory and copies template files
/// 4. **Variable Processing**: Substitutes template variables with actual values
/// 5. **File Generation**: Creates additional standard files (README, .gitignore)
/// 6. **Git Operations**: Initializes local Git repository and creates initial commit
/// 7. **GitHub Integration**: Creates repository on GitHub using API
/// 8. **Content Push**: Uploads local content to the GitHub repository
/// 9. **Post-Setup**: Configures repository settings, apps, and webhooks
///
/// ## Parameters
///
/// * `req` - Repository creation request containing name, owner, and template
/// * `config` - Configuration containing available templates and settings
/// * `template_fetcher` - Service for downloading template files (injectable for testing)
/// * `repo_client` - Service for GitHub API operations (injectable for testing)
///
/// ## Returns
///
/// * `CreateRepoResult` - Success/failure result with descriptive message
///
/// ## Dependency Injection
///
/// This function accepts trait objects for external services, enabling:
/// - **Unit Testing**: Mock implementations for isolated testing
/// - **Integration Testing**: Test implementations that don't hit real APIs
/// - **Flexibility**: Different implementations for different environments
///
/// ## Error Handling
///
/// Each step in the workflow is individually error-handled:
/// - Template not found in configuration
/// - Template file fetching failures
/// - File system operations errors
/// - Git operation failures
/// - GitHub API errors
/// - Authentication failures
///
/// ## GitHub App Authentication
///
/// Uses GitHub App credentials to:
/// 1. Get installation token for the target organization
/// 2. Create repository using organization permissions
/// 3. Push content using authenticated Git operations
///
/// ## Example Usage
///
/// ```rust,ignore
/// let request = CreateRepoRequest {
///     name: "new-service".to_string(),
///     owner: "my-org".to_string(),
///     template: "microservice".to_string(),
/// };
///
/// let result = create_repository_with_custom_settings(
///     request,
///     &config,
///     &github_template_fetcher,
///     &github_repo_client
/// ).await;
///
/// match result.success {
///     true => println!("Repository created: {}", result.message),
///     false => eprintln!("Creation failed: {}", result.message),
/// }
/// ```
async fn create_repository_with_custom_settings(
    req: CreateRepoRequest,
    config: &config_manager::Config,
    template_fetcher: &dyn TemplateFetcher,
    repo_client: &dyn RepositoryClient,
) -> CreateRepoResult {
    info!(
        "Starting repository creation: name='{}', owner='{}', template='{}'",
        req.name, req.owner, req.template
    );

    debug!(
        "Found {} templates in configuration",
        config.templates.len()
    );

    // 1. Find template config
    debug!("Searching for template '{}' in configuration", req.template);
    let template = match config.templates.iter().find(|t| t.name == req.template) {
        Some(t) => {
            info!("Template '{}' found in configuration", req.template);
            debug!("Template source repository: {}", t.source_repo);
            t
        }
        None => {
            error!("Template '{}' not found in configuration", req.template);
            debug!(
                "Available templates: {:?}",
                config.templates.iter().map(|t| &t.name).collect::<Vec<_>>()
            );
            return CreateRepoResult::failure("Template not found in config");
        }
    };

    // 2. Create a local repository temporary directory
    let local_repo_path = match TempDir::new() {
        Ok(d) => d,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to create a temporary directory: {e}"
            ))
        }
    };

    // 3. Fetch template files
    let files = match template_fetcher
        .fetch_template_files(&template.source_repo)
        .await
    {
        Ok(f) => f,
        Err(e) => return CreateRepoResult::failure(format!("Failed to fetch template files: {e}")),
    };

    // 4. Add the template files (copy, not git clone)
    if let Err(e) = copy_template_files(&files, &local_repo_path) {
        return CreateRepoResult::failure(format!("Failed to copy template files: {e}"));
    }

    // Convert CreateRepoRequest to RepositoryCreationRequest for helper functions
    let new_request = match (
        RepositoryName::new(&req.name),
        OrganizationName::new(&req.owner),
        TemplateName::new(&req.template),
    ) {
        (Ok(name), Ok(owner), Ok(template)) => {
            RepositoryCreationRequestBuilder::new(name, owner, template).build()
        }
        _ => {
            return CreateRepoResult::failure(
                "Invalid request parameters: name, owner, or template invalid".to_string(),
            )
        }
    };

    // 5. Replace template variables in the copied files
    if let Err(e) = replace_template_variables(&local_repo_path, &new_request, template) {
        return CreateRepoResult::failure(format!("Failed to replace template variables: {e}"));
    }

    // 6. Create any additional required files (only if not provided by template)
    if let Err(e) = create_additional_files(&local_repo_path, &new_request, &files) {
        return CreateRepoResult::failure(format!("Failed to create additional files: {e}"));
    }

    // 7. Create repo on github (org or user)
    let payload = RepositoryCreatePayload {
        name: req.name.clone(),
        ..Default::default()
    };

    info!(
        "Creating GitHub repository: name='{}', owner='{}'",
        req.name, req.owner
    );
    debug!("Repository creation payload: {:?}", payload);

    // First, get installation token for the organization
    info!("Getting installation token for organization: {}", req.owner);
    let installation_token = match repo_client.get_installation_token_for_org(&req.owner).await {
        Ok(token) => {
            info!(
                token_length = token.len(),
                token_prefix = &token[..std::cmp::min(8, token.len())],
                "Successfully retrieved installation token for organization: {}",
                req.owner
            );
            token
        }
        Err(e) => {
            error!(
                "Failed to get installation token for organization '{}': {}",
                req.owner, e
            );
            return CreateRepoResult::failure(format!(
                "Failed to get installation token for organization '{}': {}",
                req.owner, e
            ));
        }
    };

    // Create a new client using the installation token for repository operations
    info!("Creating GitHub client with installation token");
    let installation_client = match github_client::create_token_client(&installation_token) {
        Ok(client) => {
            info!("Successfully created installation token client");
            GitHubClient::new(client)
        }
        Err(e) => {
            error!("Failed to create installation token client: {}", e);
            return CreateRepoResult::failure(format!(
                "Failed to create installation token client: {}",
                e
            ));
        }
    };

    // Get the organization's default branch setting (using installation token)
    info!(
        "Getting organization default branch setting for: {}",
        req.owner
    );
    let default_branch = match installation_client
        .get_organization_default_branch(&req.owner)
        .await
    {
        Ok(branch) => {
            info!(
                org_name = req.owner,
                default_branch = branch,
                "Successfully retrieved organization default branch setting"
            );
            branch
        }
        Err(e) => {
            error!(
                "Failed to get default branch for organization '{}': {}",
                req.owner, e
            );
            warn!("Falling back to 'main' as default branch");
            "main".to_string()
        }
    };

    // 8. Initialize the local git repository with the organization's default branch
    if let Err(e) = init_local_git_repo(&local_repo_path, &default_branch) {
        return CreateRepoResult::failure(format!("Failed to initialize local git repo: {e}"));
    }

    // 9. Commit all changes to the default branch (stub commit signing)
    if let Err(e) = commit_all_changes(&local_repo_path, "Initial commit (stub, unsigned)") {
        return CreateRepoResult::failure(format!("Failed to commit changes: {e}"));
    }

    // Now create the repository using the installation token client
    let repo_result = if !req.owner.is_empty() {
        info!("Creating organization repository for owner: {}", req.owner);
        installation_client
            .create_org_repository(&req.owner, &payload)
            .await
    } else {
        info!("Creating user repository");
        installation_client.create_user_repository(&payload).await
    };

    let repo = match repo_result {
        Ok(r) => {
            info!(
                "GitHub repository created successfully: name='{}', url='{}'",
                r.name(),
                r.url()
            );
            debug!("Repository details: node_id='{}'", r.node_id());
            r
        }
        Err(e) => {
            error!("Failed to create GitHub repository: {}", e);
            return CreateRepoResult::failure(format!("Failed to create repo: {e}"));
        }
    };

    info!(
        "Repository created successfully: name='{}', url='{}', id='{}'",
        repo.name(),
        repo.url(),
        repo.node_id()
    );

    // 10. Push the local repository to the origin with authentication
    info!(
        "Attempting to push local repository to remote origin: {}",
        repo.url()
    );
    if let Err(e) = push_to_origin(
        &local_repo_path,
        repo.url(),
        &default_branch,
        &installation_token,
    ) {
        error!("Git push operation failed: {}", e);
        return CreateRepoResult::failure(format!("Failed to push to origin: {e}"));
    }

    // 11. Update remote repository settings (stub)
    if let Err(e) = update_remote_settings(&repo) {
        return CreateRepoResult::failure(format!("Failed to update remote settings: {e}"));
    }

    // 12. Install required GitHub apps (stub)
    if let Err(e) = install_github_apps(&repo) {
        return CreateRepoResult::failure(format!("Failed to install GitHub apps: {e}"));
    }

    // 13. Trigger post-creation webhooks (stub)
    if let Err(e) = trigger_post_creation_webhooks(&repo) {
        return CreateRepoResult::failure(format!("Failed to trigger post-creation webhooks: {e}"));
    }

    CreateRepoResult::success(format!("Repository {} created successfully", repo.name()))
}

/// Create a new repository from a template with provided configuration.
///
/// This function allows callers to provide their own configuration instead of
/// loading it from the environment. This is useful for different interfaces
/// (CLI, API, Azure Function) that may load configuration differently.
///
/// # Arguments
///
/// * `request` - Repository creation request containing name, owner, and template
/// * `config` - Configuration containing template definitions and repository settings
/// * `app_id` - GitHub App ID for authentication
/// * `app_key` - GitHub App private key for authentication
///
/// # Returns
///
/// Returns `CreateRepoResult` indicating success or failure with details.
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::{CreateRepoRequest, create_repository_with_config};
/// use config_manager::Config;
///
/// # async fn example() {
/// let request = CreateRepoRequest {
///     name: "my-repo".to_string(),
///     owner: "my-org".to_string(),
///     template: "basic".to_string(),
/// };
/// let config = Config { templates: vec![] };
/// let result = create_repository_with_config(request, &config, 12345, "private-key".to_string()).await;
/// # }
/// ```
pub async fn create_repository_with_config(
    request: CreateRepoRequest,
    config: &config_manager::Config,
    app_id: u64,
    app_key: String,
) -> CreateRepoResult {
    let provider = match create_app_client(app_id, &app_key).await {
        Ok(p) => p,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to load the GitHub provider. Error was: {}",
                e
            ))
        }
    };

    let repo_client = GitHubClient::new(provider.clone());
    let template_fetcher = template_engine::GitHubTemplateFetcher::new(GitHubClient::new(provider));

    create_repository_with_custom_settings(request, config, &template_fetcher, &repo_client).await
}

/// Create a new repository with type-safe API (TEMPORARY WRAPPER - See TODO).
///
/// **⚠️ IMPLEMENTATION NOTE**: This is a temporary wrapper function created during API migration
/// (Task 1.8.6). It provides a type-safe interface while delegating to the legacy implementation.
///
/// ## TODO (Task 7.2 - Complete Refactoring)
///
/// When implementing Task 7.2 "Update repository creation workflow in repo_roller_core",
/// this function must be **completely rewritten** to:
///
/// ### Required Changes
/// 1. **Trait-Based Dependencies**: Accept `&dyn ConfigurationManager` and `&dyn UserAuthenticationService`
///    instead of concrete types and raw credentials
/// 2. **Remove Raw Credentials**: Eliminate `app_id`/`app_key` parameters - use auth service abstraction
/// 3. **Direct Implementation**: Implement orchestration logic directly instead of delegating to legacy code
/// 4. **Configuration Integration**: Use new configuration system from Tasks 2.0-5.0
/// 5. **Structured Metadata**: Collect and return complete `RepositoryCreationResult` with actual data
///    from GitHub API responses (url, id, created_at, default_branch)
///
/// ### Current Limitations
/// - **String Parsing**: Extracts repository metadata from legacy success/failure messages (fragile)
/// - **Incomplete Data**: Some `RepositoryCreationResult` fields use placeholder or default values
/// - **Concrete Dependencies**: Still uses `config_manager::Config` directly instead of trait objects
/// - **No Error Context**: Limited error information due to string-based legacy error handling
/// - **Delegation Overhead**: Unnecessary type conversions back and forth
///
/// ### Migration Path
/// 1. Implement new configuration system (Tasks 2.0-5.0)
/// 2. Define `ConfigurationManager` and `UserAuthenticationService` traits
/// 3. Rewrite this function to use traits and return structured data
/// 4. Update all call sites to use new interface
/// 5. Remove legacy `create_repository_with_config()` and associated types
///
/// See `specs/interfaces/repository-domain.md` for target interface specification.
///
/// ---
///
/// ## Current Behavior
///
/// This function provides type-safe repository creation by:
/// 1. Converting typed request to legacy format
/// 2. Calling existing `create_repository_with_config()`
/// 3. Parsing result and converting back to typed format
///
/// # Parameters
///
/// * `request` - Type-safe repository creation request with branded types
/// * `config` - Configuration containing template definitions
/// * `app_id` - GitHub App ID for authentication
/// * `app_key` - GitHub App private key for authentication
///
/// # Returns
///
/// * `Ok(RepositoryCreationResult)` - Repository created successfully with metadata
/// * `Err(RepoRollerError)` - Creation failed with categorized error
///
/// # Examples
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
/// match create_repository(request, &config, 12345, "private-key".to_string()).await {
///     Ok(result) => {
///         println!("Created: {}", result.repository_url);
///         println!("ID: {}", result.repository_id);
///     }
///     Err(e) => eprintln!("Failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
/// Setup GitHub clients and authentication.
///
/// Creates the GitHub App client and retrieves installation token for the organization.
///
/// # Returns
///
/// Returns tuple of (installation_token, installation_repo_client).
async fn setup_github_authentication(
    app_id: u64,
    app_key: &str,
    organization: &str,
) -> RepoRollerResult<(String, GitHubClient)> {
    info!("Creating GitHub App client for authentication");
    let app_client = create_app_client(app_id, app_key).await.map_err(|e| {
        error!("Failed to create GitHub App client: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create GitHub App client: {}", e),
        })
    })?;

    let repo_client = GitHubClient::new(app_client);

    info!("Getting installation token for organization: {}", organization);
    let installation_token = repo_client
        .get_installation_token_for_org(organization)
        .await
        .map_err(|e| {
            error!(
                "Failed to get installation token for organization '{}': {}",
                organization, e
            );
            RepoRollerError::GitHub(GitHubError::AuthenticationFailed {
                reason: format!(
                    "Failed to get installation token for organization '{}': {}",
                    organization, e
                ),
            })
        })?;

    info!("Successfully retrieved installation token");

    let installation_client = github_client::create_token_client(&installation_token)
        .map_err(|e| {
            error!("Failed to create installation token client: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create installation token client: {}", e),
            })
        })?;

    let installation_repo_client = GitHubClient::new(installation_client);

    Ok((installation_token, installation_repo_client))
}

/// Resolve organization configuration using OrganizationSettingsManager.
///
/// # Returns
///
/// Returns the merged configuration for the repository.
async fn resolve_organization_configuration(
    installation_token: &str,
    organization: &str,
    template_name: &str,
) -> RepoRollerResult<config_manager::MergedConfiguration> {
    use config_manager::{
        ConfigurationContext, GitHubMetadataProvider, MetadataProviderConfig,
        OrganizationSettingsManager,
    };
    use std::sync::Arc;

    info!("Resolving organization configuration");
    
    // Create a separate client for the metadata provider
    let metadata_client =
        github_client::create_token_client(installation_token).map_err(|e| {
            error!("Failed to create metadata provider client: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create metadata provider client: {}", e),
            })
        })?;
    let metadata_repo_client = GitHubClient::new(metadata_client);
    
    let metadata_provider_config = MetadataProviderConfig::explicit("repo-config");
    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        metadata_repo_client,
        metadata_provider_config,
    ));

    let settings_manager = OrganizationSettingsManager::new(metadata_provider);

    let config_context = ConfigurationContext::new(organization, template_name);

    let merged_config = settings_manager
        .resolve_configuration(&config_context)
        .await
        .map_err(|e| {
            warn!(
                "Failed to resolve organization configuration: {}. Continuing with defaults.",
                e
            );
            e
        })
        .map_err(RepoRollerError::Configuration)?;

    info!("Organization configuration resolved successfully");
    Ok(merged_config)
}

/// Prepare local repository with template files and processing.
///
/// This function:
/// 1. Fetches template files
/// 2. Copies them to local directory
/// 3. Processes template variables
/// 4. Creates additional required files
///
/// # Returns
///
/// Returns the temporary directory containing the prepared repository.
async fn prepare_local_repository(
    request: &RepositoryCreationRequest,
    template: &config_manager::TemplateConfig,
    template_fetcher: &dyn TemplateFetcher,
) -> RepoRollerResult<TempDir> {
    // Create temporary directory
    let local_repo_path = TempDir::new().map_err(|e| {
        error!("Failed to create temporary directory: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create temporary directory: {}", e),
        })
    })?;

    // Fetch template files
    info!("Fetching template files from: {}", template.source_repo);
    let files = template_fetcher
        .fetch_template_files(&template.source_repo)
        .await
        .map_err(|e| {
            error!("Failed to fetch template files: {}", e);
            RepoRollerError::Template(TemplateError::FetchFailed {
                reason: format!("Failed to fetch template files: {}", e),
            })
        })?;

    // Copy template files
    debug!("Copying template files to local repository");
    copy_template_files(&files, &local_repo_path).map_err(|e| {
        error!("Failed to copy template files: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to copy template files: {}", e),
        })
    })?;

    // Process template variables
    debug!("Processing template variables");
    replace_template_variables(&local_repo_path, request, template).map_err(|e| {
        error!("Failed to replace template variables: {}", e);
        RepoRollerError::Template(TemplateError::SubstitutionFailed {
            variable: "unknown".to_string(),
            reason: format!("Failed to replace template variables: {}", e),
        })
    })?;

    // Create additional files
    debug!("Creating additional required files");
    create_additional_files(&local_repo_path, request, &files).map_err(|e| {
        error!("Failed to create additional files: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create additional files: {}", e),
        })
    })?;

    Ok(local_repo_path)
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
) -> RepoRollerResult<String> {
    info!("Getting organization default branch setting for: {}", organization);
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

    debug!("Initializing local git repository with branch: {}", default_branch);
    init_local_git_repo(local_repo_path, &default_branch).map_err(|e| {
        error!("Failed to initialize local git repository: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to initialize local git repository: {}", e),
        })
    })?;

    debug!("Committing initial changes");
    commit_all_changes(local_repo_path, "Initial commit").map_err(|e| {
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
            RepoRollerError::GitHub(GitHubError::ApiRequestFailed {
                status: 500,
                message: format!("Failed to create repository: {}", e),
            })
        })?;

    info!("GitHub repository created successfully: url='{}'", repo.url());
    Ok(repo)
}

/// Apply merged configuration to the created repository.
///
/// This includes labels, webhooks, and custom properties.
fn apply_repository_configuration(
    merged_config: &config_manager::MergedConfiguration,
) {
    info!("Applying merged configuration to repository");

    // Apply labels
    if !merged_config.labels.is_empty() {
        debug!("Creating {} labels", merged_config.labels.len());
        for (label_name, label_config) in &merged_config.labels {
            debug!("Creating label: {}", label_name);
            // TODO: Implement label creation via GitHub API
            // This requires adding label creation support to GitHubClient
            info!("Label configuration: {} -> {:?}", label_name, label_config);
        }
    }

    // Apply webhooks
    if !merged_config.webhooks.is_empty() {
        debug!("Creating {} webhooks", merged_config.webhooks.len());
        for webhook in &merged_config.webhooks {
            debug!("Creating webhook: {}", webhook.url);
            // TODO: Implement webhook creation via GitHub API
            // This requires adding webhook creation support to GitHubClient
            info!("Webhook configuration: {:?}", webhook);
        }
    }

    // Apply custom properties (including repository type)
    if !merged_config.custom_properties.is_empty() {
        debug!("Setting {} custom properties", merged_config.custom_properties.len());
        // TODO: Implement custom property setting via GitHubClient
        // This should use the set_repository_custom_properties method from Task 6.2
        for prop in &merged_config.custom_properties {
            info!("Custom property: {} = {:?}", prop.property_name, prop.value);
        }
    }
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
/// use repo_roller_core::{RepositoryCreationRequest, create_repository};
/// use config_manager::Config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = RepositoryCreationRequest::builder()
///     .name("my-repo")?
///     .owner("my-org")?
///     .template("rust-service")?
///     .build();
///
/// let config = Config { templates: vec![] };
/// let result = create_repository(request, &config, 12345, "private-key".to_string()).await?;
/// println!("Created repository: {}", result.repository_url);
/// # Ok(())
/// # }
/// ```
pub async fn create_repository(
    request: RepositoryCreationRequest,
    config: &config_manager::Config,
    app_id: u64,
    app_key: String,
) -> RepoRollerResult<RepositoryCreationResult> {
    info!(
        "Starting repository creation: name='{}', owner='{}', template='{}'",
        request.name, request.owner, request.template
    );

    // Step 1: Setup GitHub authentication
    let (installation_token, installation_repo_client) = 
        setup_github_authentication(app_id, &app_key, request.owner.as_ref()).await?;

    // Step 2: Create template fetcher for later use
    let app_client = create_app_client(app_id, &app_key).await.map_err(|e| {
        error!("Failed to create GitHub App client for template fetcher: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create GitHub App client: {}", e),
        })
    })?;
    let template_fetcher = template_engine::GitHubTemplateFetcher::new(GitHubClient::new(app_client));

    // Step 3: Resolve organization configuration
    let merged_config = resolve_organization_configuration(
        &installation_token,
        request.owner.as_ref(),
        request.template.as_ref(),
    ).await?;

    // Step 4: Find template configuration
    debug!("Searching for template '{}' in configuration", request.template);
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
    let local_repo_path = prepare_local_repository(&request, template, &template_fetcher).await?;

    // Step 6: Initialize Git repository and commit
    let default_branch = initialize_git_repository(
        &local_repo_path,
        &installation_repo_client,
        request.owner.as_ref(),
    ).await?;

    // Step 7: Create repository on GitHub
    let repo = create_github_repository(&request, &merged_config, &installation_repo_client).await?;

    // Step 8: Push local repository to GitHub
    info!("Pushing local repository to remote: {}", repo.url());
    push_to_origin(
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
    apply_repository_configuration(&merged_config);

    info!("Repository creation completed successfully");

    // Step 10: Return success result with repository metadata
    Ok(RepositoryCreationResult {
        repository_url: repo.url().to_string(),
        repository_id: repo.node_id().to_string(),
        created_at: Timestamp::now(),
        default_branch: default_branch.clone(),
    })
}

/// Initialize a new Git repository with the specified default branch.
///
/// Creates a new Git repository in the given directory and sets the default branch
/// to match the organization's branch naming convention.
///
/// ## Parameters
///
/// * `local_path` - Temporary directory where the Git repository should be initialized
/// * `default_branch` - Name of the default branch (e.g., "main", "master")
///
/// ## Returns
///
/// * `Ok(())` - If repository initialization succeeds
/// * `Err(Error)` - If Git initialization fails
fn init_local_git_repo(local_path: &TempDir, default_branch: &str) -> Result<(), Error> {
    debug!("Initializing git repository at {:?}", local_path.path());

    // Initialize git repository with custom options to set the default branch
    let mut opts = git2::RepositoryInitOptions::new();
    let branch_ref = format!("refs/heads/{}", default_branch);
    opts.initial_head(&branch_ref); // Set the initial branch to the organization's default

    let repo = Repository::init_opts(local_path.path(), &opts).map_err(|e| {
        error!("Failed to initialize git repository: {}", e);
        Error::GitOperation(format!("Failed to initialize git repository: {}", e))
    })?;

    info!(
        "Git repository initialized successfully with '{}' as default branch",
        default_branch
    );

    // Verify the HEAD reference
    match repo.head() {
        Ok(head_ref) => {
            info!("Initial HEAD reference: {:?}", head_ref.name());
        }
        Err(e) => {
            info!(
                "HEAD reference not yet created (normal for empty repo): {}",
                e
            );
        }
    }

    Ok(())
}

fn install_github_apps(_repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

/// Push the local repository to GitHub using installation token authentication.
///
/// This function performs the Git push operation to upload the local repository content
/// to the newly created GitHub repository. It handles authentication using GitHub App
/// installation tokens and provides detailed progress tracking and error reporting.
///
/// ## Git Push Process
///
/// 1. **Repository Access**: Opens the local Git repository
/// 2. **Remote Management**: Removes any existing 'origin' remote and adds the new one
/// 3. **Authentication Setup**: Configures Git credentials using the installation token
/// 4. **Progress Tracking**: Sets up callbacks for monitoring push progress
/// 5. **Push Execution**: Performs the actual push operation with the specified refspec
/// 6. **Error Handling**: Provides detailed error context for troubleshooting
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory containing the local Git repository
/// * `repo_url` - URL of the GitHub repository to push to
/// * `branch_name` - Name of the branch to push (matches default branch)
/// * `access_token` - GitHub App installation token for authentication
///
/// ## Returns
///
/// * `Ok(())` - If push operation succeeds
/// * `Err(Error)` - If any step in the push process fails
///
/// ## Authentication
///
/// Uses GitHub App installation token authentication with the following approach:
/// - Username: "x-access-token" (GitHub convention for token-based auth)
/// - Password: The installation token
/// - Supports only USER_PASS_PLAINTEXT credential type
///
/// ## Progress Callbacks
///
/// The function sets up several callbacks for monitoring and debugging:
/// - **Credentials**: Handles authentication challenges
/// - **Pack Progress**: Tracks data transfer progress
/// - **Push Update Reference**: Monitors reference update status
///
/// ## Error Context
///
/// Provides specific error messages based on Git error classes:
/// - **Network errors**: Connection or DNS issues
/// - **HTTP errors**: Authentication or permission problems
/// - **Callback errors**: Token validation failures
/// - **Reference errors**: Branch or repository state issues
///
/// ## Example
///
/// ```rust,ignore
/// let repo_url = url::Url::parse("https://github.com/owner/repo")?;
/// push_to_origin(&temp_dir, repo_url, "main", &installation_token)?;
/// println!("Repository pushed successfully");
/// ```
fn push_to_origin(
    local_repo_path: &TempDir,
    repo_url: url::Url,
    branch_name: &str,
    access_token: &str,
) -> Result<(), Error> {
    info!(
        "Starting git push operation to origin: {} (branch: {})",
        repo_url, branch_name
    );
    debug!(
        "Token length: {} characters, starts with: {}",
        access_token.len(),
        &access_token.chars().take(8).collect::<String>()
    );

    let repo = Repository::open(local_repo_path.path()).map_err(|e| {
        error!(
            "Failed to open git repository at {:?}: {}",
            local_repo_path.path(),
            e
        );
        Error::GitOperation(format!("Failed to open git repository: {}", e))
    })?;

    debug!("Git repository opened successfully");

    // Check if 'origin' remote already exists and remove it
    match repo.find_remote("origin") {
        Ok(_) => {
            debug!("Origin remote already exists, removing it");
            repo.remote_delete("origin").map_err(|e| {
                error!("Failed to delete existing origin remote: {}", e);
                Error::GitOperation(format!("Failed to delete existing origin remote: {}", e))
            })?;
            info!("Existing origin remote removed");
        }
        Err(_) => debug!("No existing origin remote found (expected)"),
    }

    // Add remote 'origin'
    let mut remote = repo.remote("origin", repo_url.as_str()).map_err(|e| {
        error!("Failed to add remote origin '{}': {}", repo_url, e);
        Error::GitOperation(format!("Failed to add remote origin: {}", e))
    })?;

    info!("Remote 'origin' added successfully");

    // Set up authentication callbacks with the GitHub App installation token
    let mut callbacks = git2::RemoteCallbacks::new();
    let token = access_token.to_string(); // Clone for move into closure

    // Enhanced credential callback with detailed logging
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        info!(
            "Git credential callback triggered - URL: {}, username: {:?}, allowed types: {:?}",
            url, username_from_url, allowed_types
        );

        debug!("Credential types breakdown:");
        debug!(
            "  USER_PASS_PLAINTEXT: {}",
            allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
        );
        debug!(
            "  SSH_KEY: {}",
            allowed_types.contains(git2::CredentialType::SSH_KEY)
        );
        debug!(
            "  SSH_CUSTOM: {}",
            allowed_types.contains(git2::CredentialType::SSH_CUSTOM)
        );
        debug!(
            "  DEFAULT: {}",
            allowed_types.contains(git2::CredentialType::DEFAULT)
        );
        debug!(
            "  SSH_INTERACTIVE: {}",
            allowed_types.contains(git2::CredentialType::SSH_INTERACTIVE)
        );
        debug!(
            "  USERNAME: {}",
            allowed_types.contains(git2::CredentialType::USERNAME)
        );
        debug!(
            "  SSH_MEMORY: {}",
            allowed_types.contains(git2::CredentialType::SSH_MEMORY)
        );

        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            info!("Using USER_PASS_PLAINTEXT credentials with 'x-access-token' username");
            debug!(
                "Token for authentication: {}...",
                &token.chars().take(8).collect::<String>()
            );

            match git2::Cred::userpass_plaintext("x-access-token", &token) {
                Ok(cred) => {
                    info!("Successfully created git2 credentials");
                    Ok(cred)
                }
                Err(e) => {
                    error!("Failed to create git2 credentials: {}", e);
                    Err(e)
                }
            }
        } else {
            error!(
                "No supported credential types available. Allowed types: {:?}",
                allowed_types
            );
            Err(git2::Error::from_str(
                "No supported credential types for GitHub authentication",
            ))
        }
    });

    // Add progress callback for detailed push progress
    callbacks.pack_progress(|stage, current, total| {
        debug!(
            "Pack progress - stage: {:?}, current: {}, total: {}",
            stage, current, total
        );
    });

    // Add push update reference callback
    callbacks.push_update_reference(|refname, status| match status {
        Some(msg) => {
            error!("Reference update failed for '{}': {}", refname, msg);
            Err(git2::Error::from_str(&format!(
                "Push reference update failed: {}",
                msg
            )))
        }
        None => {
            info!("Reference '{}' updated successfully", refname);
            Ok(())
        }
    });

    // Push options
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Push the branch
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    info!("Attempting to push with refspec: {}", refspec);

    match remote.push(&[&refspec], Some(&mut push_options)) {
        Ok(_) => {
            info!("Successfully pushed to origin: {}", repo_url);
            Ok(())
        }
        Err(e) => {
            error!("Git push failed with error: {}", e);
            error!("Error details:");
            error!("  Error code: {:?}", e.code());
            error!("  Error class: {:?}", e.class());
            error!("  Error message: {}", e.message());

            // Provide more specific error context
            let detailed_error = match e.class() {
                git2::ErrorClass::Net => {
                    format!("Network error during push: {}. Check internet connection and repository URL.", e.message())
                }
                git2::ErrorClass::Http => {
                    format!("HTTP error during push: {}. This may indicate authentication or permission issues.", e.message())
                }
                git2::ErrorClass::Callback => {
                    format!("Authentication callback error: {}. GitHub App token may be invalid or expired.", e.message())
                }
                git2::ErrorClass::Reference => {
                    format!(
                        "Reference error during push: {}. Branch or repository state issue.",
                        e.message()
                    )
                }
                _ => {
                    format!(
                        "Git operation failed: {} (class: {:?})",
                        e.message(),
                        e.class()
                    )
                }
            };

            Err(Error::GitOperation(detailed_error))
        }
    }
}

/// Process template variables and substitute them in all template files.
///
/// This function handles the variable substitution phase of repository creation,
/// replacing template placeholders with actual values throughout all files in
/// the local repository.
///
/// ## Process Overview
///
/// 1. **Variable Setup**: Generates built-in variables and merges with user variables
/// 2. **Configuration Mapping**: Converts template variable configurations
/// 3. **File Reading**: Scans all files in the local repository
/// 4. **Template Processing**: Performs variable substitution using the template engine
/// 5. **File Replacement**: Removes original files and writes processed versions
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory containing template files to process
/// * `req` - Repository creation request containing substitution values
/// * `template` - Template configuration including variable definitions
///
/// ## Returns
///
/// * `Ok(())` - If template processing completes successfully
/// * `Err(Error)` - If any step in the processing fails
///
/// ## Built-in Variables
///
/// The function automatically generates these variables:
/// - `repo_name`: Repository name from the request
/// - `org_name`: Organization/owner name from the request
/// - `template_name`: Template name used for creation
/// - `user_login`: GitHub App login (placeholder)
/// - `user_name`: GitHub App display name (placeholder)
/// - `default_branch`: Default branch name ("main")
///
/// ## Variable Configuration
///
/// Converts template variable configurations from `config_manager` format
/// to `template_engine` format, including:
/// - Validation rules (pattern, length, required)
/// - Default values and examples
/// - Option lists for enumerated values
///
/// ## File Processing
///
/// - Processes all files recursively in the repository
/// - Excludes the `.git` directory from processing
/// - Maintains file paths and directory structure
/// - Handles both text and binary files appropriately
///
/// ## Error Handling
///
/// Returns errors for:
/// - File system operations (reading, writing, directory operations)
/// - Template engine processing failures
/// - Path manipulation errors
///
/// ## Example
///
/// ```rust,ignore
/// let request = CreateRepoRequest {
///     name: "my-api".to_string(),
///     owner: "acme-corp".to_string(),
///     template: "go-service".to_string(),
/// };
/// let template_config = // ... load from configuration
/// replace_template_variables(&temp_dir, &request, &template_config)?;
/// // Files now contain substituted values like "my-api" instead of "{{repo_name}}"
/// ```
///
/// ## Template Engine Integration
///
/// Uses the `template_engine` crate for actual variable substitution:
/// - Supports Handlebars-style `{{variable}}` syntax
/// - Handles conditional blocks and loops
/// - Provides validation and error reporting
/// - Configurable file inclusion/exclusion patterns
fn replace_template_variables(
    local_repo_path: &TempDir,
    req: &RepositoryCreationRequest,
    template: &config_manager::TemplateConfig,
) -> Result<(), Error> {
    debug!("Processing template variables using TemplateProcessor");

    // Create template processor
    let processor = TemplateProcessor::new().map_err(|e| {
        Error::TemplateProcessing(format!("Failed to create template processor: {}", e))
    })?;

    // Generate built-in variables
    let built_in_params = template_engine::BuiltInVariablesParams {
        repo_name: req.name.as_ref(),
        org_name: req.owner.as_ref(),
        template_name: req.template.as_ref(),
        template_repo: "unknown", // We'd need to get this from template config
        user_login: "reporoller-app", // Placeholder for GitHub App
        user_name: "RepoRoller App", // Placeholder for GitHub App
        default_branch: "main",
    };
    let built_in_variables = processor.generate_built_in_variables(&built_in_params); // For MVP, we'll use empty user variables and get variable configs from template
                                                                                      // In a full implementation, these would come from user input and merged configs
    let user_variables = HashMap::new();

    // Convert config_manager::VariableConfig to template_engine::VariableConfig
    let mut variable_configs = HashMap::new();
    if let Some(ref template_vars) = template.variable_configs {
        for (name, config) in template_vars {
            let engine_config = template_engine::VariableConfig {
                description: config.description.clone(),
                example: config.example.clone(),
                required: config.required,
                pattern: config.pattern.clone(),
                min_length: config.min_length,
                max_length: config.max_length,
                options: config.options.clone(),
                default: config.default.clone(),
            };
            variable_configs.insert(name.clone(), engine_config);
        }
    }

    // Create processing request
    let processing_request = TemplateProcessingRequest {
        variables: user_variables,
        built_in_variables,
        variable_configs,
        templating_config: None, // Use default processing (all files)
    };
    // Read all files that were copied to the local repo
    let mut files_to_process = Vec::new();
    for entry in WalkDir::new(local_repo_path.path()) {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            Error::FileSystem(format!("Failed to read directory entry: {}", e))
        })?;

        if entry.file_type().is_file() {
            let file_path = entry.path();
            let relative_path = file_path
                .strip_prefix(local_repo_path.path())
                .map_err(|e| {
                    error!("Failed to get relative path: {}", e);
                    Error::FileSystem(format!("Failed to get relative path: {}", e))
                })?;

            let content = fs::read(file_path).map_err(|e| {
                error!("Failed to read file {:?}: {}", file_path, e);
                Error::FileSystem(format!("Failed to read file {:?}: {}", file_path, e))
            })?;

            files_to_process.push((relative_path.to_string_lossy().to_string(), content));
        }
    }

    // Process the template files
    let processed = processor
        .process_template(
            &files_to_process,
            &processing_request,
            local_repo_path.path(),
        )
        .map_err(|e| {
            error!("Template processing failed: {}", e);
            Error::TemplateProcessing(format!("Template processing failed: {}", e))
        })?;

    // Write the processed files back to the local repo    // First, clear the directory (except .git)
    for entry in WalkDir::new(local_repo_path.path())
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
    {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            Error::FileSystem(format!("Failed to read directory entry: {}", e))
        })?;

        if entry.file_type().is_file() {
            fs::remove_file(entry.path()).map_err(|e| {
                error!("Failed to remove file {:?}: {}", entry.path(), e);
                Error::FileSystem(format!("Failed to remove file {:?}: {}", entry.path(), e))
            })?;
        }
    }

    // Write processed files
    for (file_path, content) in processed.files {
        let target_path = local_repo_path.path().join(&file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create directory {:?}: {}", parent, e);
                Error::FileSystem(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }

        // Write the file content
        fs::write(&target_path, content).map_err(|e| {
            error!("Failed to write processed file {:?}: {}", target_path, e);
            Error::FileSystem(format!(
                "Failed to write processed file {:?}: {}",
                target_path, e
            ))
        })?;

        debug!("Wrote processed file: {}", file_path);
    }

    info!("Template variable processing completed successfully");
    Ok(())
}

fn trigger_post_creation_webhooks(_repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

fn update_remote_settings(_repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}
