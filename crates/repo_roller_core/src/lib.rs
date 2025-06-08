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
use github_client::{RepositoryClient, RepositoryCreatePayload};
use template_engine::{self, TemplateFetcher};

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

/// Org-specific rules for repository creation (stub)
#[derive(Debug, Clone, Default)]
pub struct OrgRules {
    pub repo_name_regex: Option<String>,
    // Add more rules as needed
}

/// Get organization-specific rules (stub implementation)
pub fn get_org_rules(org: &str) -> OrgRules {
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

// --- Update create_repository to be generic over RepoName ---
/// Create a new repository from a template, with dependency injection for testability.
pub fn create_repository(
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

    // 3. Fetch template files
    let files = match template_fetcher.fetch_template_files(&template.source_repo) {
        Ok(f) => f,
        Err(e) => return CreateRepoResult::failure(format!("Failed to fetch template files: {e}")),
    };

    // 4. Create a local repository

    // 5. Add the template files

    // 6. Commit all changes to the default branch

    // 7. Create repo on github (org or user)
    let payload = RepositoryCreatePayload {
        name: &req.name,
        ..Default::default()
    };
    let repo_result = if !req.owner.is_empty() {
        repo_client.create_org_repository(&req.owner, &payload)
    } else {
        repo_client.create_user_repository(&payload)
    };
    let repo = match repo_result {
        Ok(r) => r,
        Err(e) => return CreateRepoResult::failure(format!("Failed to create repo: {e}")),
    };

    // 8. Set the origin for the local repository to the new repository
    foobar();

    // 9. Push the local repository to the origin

    // 10. Update remote repository settings

    CreateRepoResult::success(format!("Repository {} created successfully", repo.name()))
}

/// Production entrypoint: uses real implementations.
pub async fn create_repository_from_request(req: CreateRepoRequest) -> CreateRepoResult {
    let config_loader = config_manager::FileConfigLoader;
    let template_fetcher = template_engine::DefaultTemplateFetcher;

    // GitHubClient requires async construction, so we need to use a runtime
    let app_id = match std::env::var("GITHUB_APP_ID") {
        Ok(val) => match val.parse::<u64>() {
            Ok(id) => id,
            Err(e) => return CreateRepoResult::failure(format!("Invalid GITHUB_APP_ID: {e}")),
        },
        Err(_) => return CreateRepoResult::failure("GITHUB_APP_ID environment variable not set"),
    };

    let private_key = match std::env::var("GITHUB_APP_PRIVATE_KEY") {
        Ok(val) => val,
        Err(_) => {
            return CreateRepoResult::failure("GITHUB_APP_PRIVATE_KEY environment variable not set")
        }
    };

    let repo_client = match github_client::GitHubClient::new(app_id, private_key).await {
        Ok(client) => client,
        Err(e) => return CreateRepoResult::failure(format!("Failed to create GitHub client: {e}")),
    };

    create_repository(req, &config_loader, &template_fetcher, &repo_client)
}
