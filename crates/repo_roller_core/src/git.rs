//! Local Git repository operations.
//!
//! This module provides functions for working with local Git repositories using git2.
//! These operations handle:
//! - Repository initialization
//! - File staging and committing
//! - Remote repository configuration
//! - Push operations with authentication
//!
//! For GitHub API operations (creating repositories, managing settings), see the
//! `github_client` crate.

use crate::errors::Error;
use git2::{Repository, Signature};
use temp_dir::TempDir;
use tracing::{debug, error, info, warn};

/// Debug the current state of the repository including HEAD and commit history.
///
/// Logs information about the repository's current state for diagnostic purposes.
pub(crate) fn debug_repository_state(repo: &Repository) -> Result<(), Error> {
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
///
/// Returns the number of files found in the working directory.
pub(crate) fn debug_working_directory(local_repo_path: &TempDir) -> Result<usize, Error> {
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
pub(crate) fn prepare_index_and_tree(repo: &Repository) -> Result<git2::Oid, Error> {
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
pub(crate) fn create_initial_commit(
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
pub(crate) fn set_head_reference_and_verify(
    repo: &Repository,
    commit_oid: git2::Oid,
    commit_message: &str,
) -> Result<(), Error> {
    // Determine the branch name from the HEAD symbolic reference
    // This respects the default branch configured during init_local_git_repo
    let branch_name = if repo.head_detached().unwrap_or(false) {
        // Detached HEAD - shouldn't happen in our workflow
        warn!("Repository has detached HEAD, defaulting to 'main'");
        "main".to_string()
    } else if let Ok(head_ref) = repo.head() {
        // HEAD exists and points to a reference (e.g., "refs/heads/main")
        if let Some(name) = head_ref.name() {
            // Extract the branch name from the full reference (e.g., "refs/heads/main" -> "main")
            if let Some(branch) = name.strip_prefix("refs/heads/") {
                debug!("Found existing HEAD pointing to branch: {}", branch);
                branch.to_string()
            } else {
                // Fallback if the reference format is unexpected
                warn!("Unexpected HEAD reference format: {}", name);
                "main".to_string()
            }
        } else {
            // HEAD exists but doesn't have a name (shouldn't happen)
            warn!("HEAD reference exists but has no name");
            "main".to_string()
        }
    } else {
        // HEAD doesn't exist yet (expected for new repos) - check what it's configured to point to
        // when it does get created (the "unborn" HEAD state)
        match repo.find_reference("HEAD") {
            Ok(head_ref) => {
                if let Some(target) = head_ref.symbolic_target() {
                    // HEAD is configured to point to a branch when created
                    if let Some(branch) = target.strip_prefix("refs/heads/") {
                        debug!("Found unborn HEAD configured for branch: {}", branch);
                        branch.to_string()
                    } else {
                        warn!("Unexpected HEAD symbolic target: {}", target);
                        "main".to_string()
                    }
                } else {
                    debug!("HEAD reference exists but has no symbolic target, defaulting to 'main'");
                    "main".to_string()
                }
            }
            Err(_) => {
                // No HEAD reference at all - use default
                debug!("No HEAD reference found, defaulting to 'main'");
                "main".to_string()
            }
        }
    };

    info!("Creating branch '{}' for initial commit", branch_name);

    // Create the branch reference pointing to our commit
    let branch_ref_name = format!("refs/heads/{}", branch_name);
    repo.reference(&branch_ref_name, commit_oid, false, "Initial commit")
        .map_err(|e| {
            error!("Failed to create {} branch reference: {}", branch_name, e);
            Error::GitOperation(format!(
                "Failed to create {} branch reference: {}",
                branch_name, e
            ))
        })?;

    info!("Branch reference created: {}", branch_ref_name);

    // Now set HEAD to point to the branch (symbolic reference)
    repo.set_head(&branch_ref_name).map_err(|e| {
        error!("Failed to set HEAD to {} branch: {}", branch_name, e);
        Error::GitOperation(format!("Failed to set HEAD to {} branch: {}", branch_name, e))
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

    // Check if the branch exists
    match repo.find_branch(&branch_name, git2::BranchType::Local) {
        Ok(branch) => {
            info!("{} branch exists", branch_name);
            if let Some(oid) = branch.get().target() {
                info!("{} branch points to commit: {}", branch_name, oid);
            }
        }
        Err(e) => {
            warn!("{} branch not found: {}", branch_name, e);
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
pub fn commit_all_changes(local_repo_path: &TempDir, commit_message: &str) -> Result<(), Error> {
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
pub fn init_local_git_repo(local_path: &TempDir, default_branch: &str) -> Result<(), Error> {
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
pub fn push_to_origin(
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

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
