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

/// Create a new repository from a template.
/// (Stub implementation for CLI integration)
pub fn create_repository(_req: CreateRepoRequest) -> CreateRepoResult {
    // TODO: Implement actual orchestration logic
    CreateRepoResult::success("Stub: repository creation not yet implemented")
}
