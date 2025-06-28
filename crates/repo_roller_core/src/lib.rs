//! Core logic for RepoRoller: orchestrates repository creation from templates.

// The core crate should be able to do the following
//
// - Define a structure with all the user provided information necessary to create a repository (CreateRepoRequest -> rename to CreateRepositoryRequest)
// - Provide a method to create a new repository (create_repository). This method takes the CreateRepositoryRequest structure,
//   validates that all the information is valid (provides errors for each invalid setting) and then creates the repository.
//
// Each GitHub org / personal space may define a set of default settings that apply to each repository being created in
// that space, e.g. naming guidelines or repository rulesets or .... The core crate will read these from a repository in
// that space.
//
// The create_repository method will take the following steps
// - Check that the information provided is valid, return errors for each invalid option
// - Create a new (empty) repository on GitHub with the provided org / personal space as owner and the provided repository
//   name
// - Locally clone the repository, add all the files to the local workspace.
// - Process all the template variables and replace them in the files in the local workspace
// - Create a single commit to the default branch (to be specified by the area settings). This commit should be
//   signed with the app key so that it is clear that the commit was made by the app.
// - Push the changes to the repository. In order to do so we may have to disable (for the app or the repository) rulesets
//   that block direct pushes to the default branch
// - Update the settings in the repository
// - Sign the repository up for the required GitHub apps that the area requires, e.g. all repositories in an area should
//   be signed up for a bot that checks pull requests
// - Trigger potential subsequent processes (defined by webhooks that can be provided). This allows other processes to
//   do work after the repository is created, e.g. creating infrastructure on a SaaS cloud etc.

use config_manager::ConfigLoader;
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

/// Org-specific rules for repository creation (stub)
#[derive(Debug, Clone, Default)]
pub struct OrgRules {
    pub repo_name_regex: Option<String>,
    // Add more rules as needed
}

impl OrgRules {
    /// Get organization-specific rules (stub implementation)
    pub fn new_from_text(org: &str) -> OrgRules {
        // In a real implementation, this would look up org-specific rules from config, a file, or a service.
        // For now, return a sample rule for demonstration.
        match org {
            "calvinverse" => OrgRules {
                repo_name_regex: Some(r"^[a-z][a-z0-9\-]{2,30}$".to_string()),
            },
            _ => OrgRules {
                repo_name_regex: Some(r"^[a-zA-Z0-9_\-]{1,50}$".to_string()),
            },
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

/// Prepare the git index and create a tree from all files in the working directory.
fn prepare_index_and_tree(repo: &Repository) -> Result<git2::Oid, Error> {
    // Get the repository index and add all files
    let mut index = repo.index().map_err(|e| {
        error!("Failed to get repository index: {}", e);
        Error::GitOperation(format!("Failed to get repository index: {}", e))
    })?;

    debug!("Repository index retrieved");

    // Add all files in the working directory to the index
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

    // Write the index
    let tree_oid = index.write_tree().map_err(|e| {
        error!("Failed to write tree: {}", e);
        Error::GitOperation(format!("Failed to write tree: {}", e))
    })?;

    debug!("Git tree written with OID: {}", tree_oid);

    Ok(tree_oid)
}

/// Create an initial commit with the given tree and message.
fn create_initial_commit(
    repo: &Repository,
    tree_oid: git2::Oid,
    commit_message: &str,
) -> Result<git2::Oid, Error> {
    let tree = repo.find_tree(tree_oid).map_err(|e| {
        error!("Failed to find tree: {}", e);
        Error::GitOperation(format!("Failed to find tree: {}", e))
    })?;

    // Create signature (using placeholder values for MVP)
    let signature = Signature::now("RepoRoller", "repo-roller@example.com").map_err(|e| {
        error!("Failed to create signature: {}", e);
        Error::GitOperation(format!("Failed to create signature: {}", e))
    })?;

    debug!("Git signature created for RepoRoller");

    // Create the commit (this is the initial commit, so no parent)
    // For initial commit, don't reference HEAD until after creation
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
fn set_head_reference_and_verify(
    repo: &Repository,
    commit_oid: git2::Oid,
    commit_message: &str,
) -> Result<(), Error> {
    // Now set the HEAD reference to point to our new commit
    let reference_name = "HEAD";
    repo.reference(reference_name, commit_oid, true, "Initial commit")
        .map_err(|e| {
            error!("Failed to set HEAD reference: {}", e);
            Error::GitOperation(format!("Failed to set HEAD reference: {}", e))
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

fn create_additional_files(
    local_repo_path: &TempDir,
    req: &CreateRepoRequest,
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
            req.name, req.template, req.owner
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

// --- Update create_repository to be generic over RepoName ---
/// Create a new repository from a template, with dependency injection for testability.
async fn create_repository_with_custom_settings(
    req: CreateRepoRequest,
    config_loader: &dyn ConfigLoader,
    template_fetcher: &dyn TemplateFetcher,
    repo_client: &dyn RepositoryClient,
) -> CreateRepoResult {
    info!(
        "Starting repository creation: name='{}', owner='{}', template='{}'",
        req.name, req.owner, req.template
    );

    // 1. Load config
    // TODO: Move the loading of the config to the endpoints because the endpoints need the config to determine
    //       how to connect to github etc.
    let config_path =
        std::env::var("REPOROLLER_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    debug!("Loading configuration from: {}", config_path);

    let config = match config_loader.load_config(&config_path) {
        Ok(cfg) => {
            info!("Configuration loaded successfully from: {}", config_path);
            debug!("Found {} templates in configuration", cfg.templates.len());
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration from '{}': {}", config_path, e);
            return CreateRepoResult::failure(format!("Failed to load config: {e}"));
        }
    };

    // 2. Find template config
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

    // 3. Create a local repository temporary directory
    let local_repo_path = match TempDir::new() {
        Ok(d) => d,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to create a temporary directory: {e}"
            ))
        }
    };

    // 4. Fetch template files
    let files = match template_fetcher
        .fetch_template_files(&template.source_repo)
        .await
    {
        Ok(f) => f,
        Err(e) => return CreateRepoResult::failure(format!("Failed to fetch template files: {e}")),
    };

    // 5. Add the template files (copy, not git clone)
    if let Err(e) = copy_template_files(&files, &local_repo_path) {
        return CreateRepoResult::failure(format!("Failed to copy template files: {e}"));
    }

    // 6. Replace template variables in the copied files
    if let Err(e) = replace_template_variables(&local_repo_path, &req, template) {
        return CreateRepoResult::failure(format!("Failed to replace template variables: {e}"));
    }

    // 7. Create any additional required files (only if not provided by template)
    if let Err(e) = create_additional_files(&local_repo_path, &req, &files) {
        return CreateRepoResult::failure(format!("Failed to create additional files: {e}"));
    }

    // 8. Create repo on github (org or user)
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

    // 9. Initialize the local git repository with the organization's default branch
    if let Err(e) = init_local_git_repo(&local_repo_path, &default_branch) {
        return CreateRepoResult::failure(format!("Failed to initialize local git repo: {e}"));
    }

    // 10. Commit all changes to the default branch (stub commit signing)
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

pub async fn create_repository(
    request: CreateRepoRequest,
    app_id: u64,
    app_key: String,
) -> CreateRepoResult {
    let config_loader = config_manager::FileConfigLoader;

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

    create_repository_with_custom_settings(request, &config_loader, &template_fetcher, &repo_client)
        .await
}

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

fn replace_template_variables(
    local_repo_path: &TempDir,
    req: &CreateRepoRequest,
    template: &config_manager::TemplateConfig,
) -> Result<(), Error> {
    debug!("Processing template variables using TemplateProcessor");

    // Create template processor
    let processor = TemplateProcessor::new();

    // Generate built-in variables
    let built_in_params = template_engine::BuiltInVariablesParams {
        repo_name: &req.name,
        org_name: &req.owner,
        template_name: &req.template,
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
