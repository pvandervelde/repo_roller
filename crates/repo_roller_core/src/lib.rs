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
use tracing::{debug, error, info};
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

fn commit_all_changes(local_repo_path: &TempDir, commit_message: &str) -> Result<(), Error> {
    debug!("Committing all changes to git repository");

    let repo = Repository::open(local_repo_path.path()).map_err(|e| {
        error!("Failed to open git repository: {}", e);
        Error::GitOperation(format!("Failed to open git repository: {}", e))
    })?;

    // Get the repository index and add all files
    let mut index = repo.index().map_err(|e| {
        error!("Failed to get repository index: {}", e);
        Error::GitOperation(format!("Failed to get repository index: {}", e))
    })?;

    // Add all files in the working directory to the index
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| {
            error!("Failed to add files to index: {}", e);
            Error::GitOperation(format!("Failed to add files to index: {}", e))
        })?;

    // Write the index
    let tree_oid = index.write_tree().map_err(|e| {
        error!("Failed to write tree: {}", e);
        Error::GitOperation(format!("Failed to write tree: {}", e))
    })?;

    let tree = repo.find_tree(tree_oid).map_err(|e| {
        error!("Failed to find tree: {}", e);
        Error::GitOperation(format!("Failed to find tree: {}", e))
    })?;

    // Create signature (using placeholder values for MVP)
    let signature = Signature::now("RepoRoller", "repo-roller@example.com").map_err(|e| {
        error!("Failed to create signature: {}", e);
        Error::GitOperation(format!("Failed to create signature: {}", e))
    })?;

    // Create the commit (this is the initial commit, so no parent)
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[],
    )
    .map_err(|e| {
        error!("Failed to create commit: {}", e);
        Error::GitOperation(format!("Failed to create commit: {}", e))
    })?;

    info!(
        "Changes committed successfully with message: {}",
        commit_message
    );
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
) -> Result<(), Error> {
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
    // 1. Load config
    let config_path =
        std::env::var("REPOROLLER_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    let config = match config_loader.load_config(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => return CreateRepoResult::failure(format!("Failed to load config: {e}")),
    };

    // 2. Find template config
    let template = match config.templates.iter().find(|t| t.name == req.template) {
        Some(t) => t,
        None => return CreateRepoResult::failure("Template not found in config"),
    };

    // 3. Create a local repository
    let local_repo_path = match TempDir::new() {
        Ok(d) => d,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to create a temporary directory: {e}"
            ))
        }
    };

    if let Err(e) = init_local_git_repo(&local_repo_path) {
        return CreateRepoResult::failure(format!("Failed to initialize local git repo: {e}"));
    }

    // 4. Fetch template files
    let files = match template_fetcher.fetch_template_files(&template.source_repo) {
        Ok(f) => f,
        Err(e) => return CreateRepoResult::failure(format!("Failed to fetch template files: {e}")),
    };

    // 5. Add the template files (copy, not git clone)
    if let Err(e) = copy_template_files(&files, &local_repo_path) {
        return CreateRepoResult::failure(format!("Failed to copy template files: {e}"));
    } // 6. Replace template variables in the copied files
    if let Err(e) = replace_template_variables(&local_repo_path, &req, template) {
        return CreateRepoResult::failure(format!("Failed to replace template variables: {e}"));
    }

    // 7. Create any additional required files (stub)
    if let Err(e) = create_additional_files(&local_repo_path, &req) {
        return CreateRepoResult::failure(format!("Failed to create additional files: {e}"));
    }

    // 8. Commit all changes to the default branch (stub commit signing)
    if let Err(e) = commit_all_changes(&local_repo_path, "Initial commit (stub, unsigned)") {
        return CreateRepoResult::failure(format!("Failed to commit changes: {e}"));
    }

    // 9. Create repo on github (org or user)
    let payload = RepositoryCreatePayload {
        name: req.name.clone(),
        ..Default::default()
    };
    let repo_result = if !req.owner.is_empty() {
        repo_client
            .create_org_repository(&req.owner, &payload)
            .await
    } else {
        repo_client.create_user_repository(&payload).await
    };
    let repo = match repo_result {
        Ok(r) => r,
        Err(e) => return CreateRepoResult::failure(format!("Failed to create repo: {e}")),
    };

    // 9.5. Get installation access token for the organization/user
    let owner = if !req.owner.is_empty() {
        &req.owner
    } else {
        // For user repositories, we still need to get the token for the authenticated user
        // This will require the user's login name - for now we'll assume the owner field
        // is always provided. TODO: Handle user repositories properly.
        return CreateRepoResult::failure(
            "User repositories not yet supported - please specify an organization owner"
                .to_string(),
        );
    };

    let installation_token = match repo_client.get_installation_token_for_org(owner).await {
        Ok(token) => token,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to get installation token for organization '{}': {}",
                owner, e
            ))
        }
    };

    // 10. Push the local repository to the origin with authentication
    if let Err(e) = push_to_origin(&local_repo_path, repo.url(), "main", &installation_token) {
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
    let template_fetcher = template_engine::DefaultTemplateFetcher;

    let provider = match create_app_client(app_id, &app_key).await {
        Ok(p) => p,
        Err(e) => {
            return CreateRepoResult::failure(format!(
                "Failed to load the GitHub provider. Error was: {}",
                e
            ))
        }
    };

    let repo_client = GitHubClient::new(provider);

    create_repository_with_custom_settings(request, &config_loader, &template_fetcher, &repo_client)
        .await
}

fn init_local_git_repo(local_path: &TempDir) -> Result<(), Error> {
    debug!("Initializing git repository at {:?}", local_path.path());

    // Initialize git repository
    let repo = Repository::init(local_path.path()).map_err(|e| {
        error!("Failed to initialize git repository: {}", e);
        Error::GitOperation(format!("Failed to initialize git repository: {}", e))
    })?;

    info!("Git repository initialized successfully");
    Ok(())
}

fn install_github_apps(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

fn push_to_origin(
    local_repo_path: &TempDir,
    repo_url: url::Url,
    branch_name: &str,
    access_token: &str,
) -> Result<(), Error> {
    debug!("Pushing changes to origin: {}", repo_url);

    let repo = Repository::open(local_repo_path.path()).map_err(|e| {
        error!("Failed to open git repository: {}", e);
        Error::GitOperation(format!("Failed to open git repository: {}", e))
    })?;

    // Add remote 'origin'
    let mut remote = repo.remote("origin", repo_url.as_str()).map_err(|e| {
        error!("Failed to add remote origin: {}", e);
        Error::GitOperation(format!("Failed to add remote origin: {}", e))
    })?;

    // Set up authentication callbacks with the GitHub App installation token
    let mut callbacks = git2::RemoteCallbacks::new();
    let token = access_token.to_string(); // Clone for move into closure
    callbacks.credentials(move |_url, _username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            // For GitHub, use 'x-access-token' as username and the installation token as password
            git2::Cred::userpass_plaintext("x-access-token", &token)
        } else {
            Err(git2::Error::from_str(
                "No supported credential types for GitHub authentication",
            ))
        }
    });

    // Push options
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks); // Push the branch
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    match remote.push(&[&refspec], Some(&mut push_options)) {
        Ok(_) => {
            info!("Successfully pushed to origin");
            Ok(())
        }
        Err(e) => {
            error!("Failed to push to remote: {}", e);
            Err(Error::GitOperation(format!(
                "Failed to push to remote: {}",
                e
            )))
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
    let built_in_variables = processor.generate_built_in_variables(
        &req.name,        // repo_name
        &req.owner,       // org_name
        &req.template,    // template_name
        "unknown",        // template_repo (we'd need to get this from template config)
        "reporoller-app", // user_login (placeholder for GitHub App)
        "RepoRoller App", // user_name (placeholder for GitHub App)
        "main",           // default_branch
    ); // For MVP, we'll use empty user variables and get variable configs from template
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

fn trigger_post_creation_webhooks(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

fn update_remote_settings(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}
