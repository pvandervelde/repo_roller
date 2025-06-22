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
use github_client::{create_app_client, GitHubClient, RepositoryClient, RepositoryCreatePayload};
use temp_dir::TempDir;
use template_engine::{self, TemplateFetcher};
use tracing::error;

mod errors;
use errors::Error;
use url::Url;

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

fn commit_all_changes(local_repo_path: &TempDir, arg: &str) -> Result<(), Error> {
    Ok(())
}

fn copy_template_files(
    files: &Vec<(String, Vec<u8>)>,
    local_repo_path: &TempDir,
) -> Result<(), Error> {
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
    }

    // 6. Replace template variables in the copied files
    if let Err(e) = replace_template_variables(&local_repo_path, &req) {
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

    // 10. Push the local repository to the origin (stub branch protection)
    if let Err(e) = push_to_origin(&local_repo_path, repo.url(), "main") {
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
    Ok(())
}

fn install_github_apps(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

fn push_to_origin(local_repo_path: &TempDir, url: Url, arg: &str) -> Result<(), Error> {
    Ok(())
}

fn replace_template_variables(
    local_repo_path: &TempDir,
    req: &CreateRepoRequest,
) -> Result<(), Error> {
    Ok(())
}

fn trigger_post_creation_webhooks(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}

fn update_remote_settings(repo: &github_client::models::Repository) -> Result<(), Error> {
    Ok(())
}
