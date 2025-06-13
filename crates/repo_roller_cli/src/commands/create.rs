use crate::errors::Error;
use async_trait::async_trait;
use std::fs;

/// Function type for prompting the user for a value interactively.
/// Used for required fields that may be missing from config or CLI args.
type AskUserForValue = dyn Fn(&str) -> Result<String, Error>;

/// Function type for retrieving organization-specific repository rules.
/// Allows for dependency injection and testability.
pub type GetOrgRulesFn = dyn Fn(&str) -> repo_roller_core::OrgRules;

#[cfg(test)]
#[path = "create_tests.rs"]
mod tests;

/// Structure representing the configuration file for repository creation.
/// All fields are optional and may be overridden by CLI arguments.
#[derive(serde::Deserialize)]
pub struct ConfigFile {
    /// Optional repository name.
    pub name: Option<String>,
    /// Optional repository owner (user or org).
    pub owner: Option<String>,
    /// Optional template type (e.g., library, service, action).
    pub template: Option<String>,
}

#[async_trait]
pub trait RepositoryCreator {
    async fn create_repository(
        &self,
        request: repo_roller_core::CreateRepoRequest,
    ) -> repo_roller_core::CreateRepoResult;
}

/// Handles the create command logic for the CLI. sch
///
/// This function merges configuration from a TOML file and CLI arguments,
/// prompts the user for any missing required values, applies organization-specific
/// naming rules, and delegates repository creation to the provided function reference.
///
/// # Arguments
/// * `config` - Optional path to a TOML config file.
/// * `name` - Name of the new repository (overrides config).
/// * `owner` - Owner (user or org) for the new repository (overrides config).
/// * `template` - Template type (overrides config).
/// * `ask_user_for_value` - Function to prompt the user for missing values.
/// * `get_org_rules` - Function to retrieve org-specific rules for validation.
/// * `create_repository` - Function to perform the actual repository creation.
///
/// # Returns
/// * `Result<CreateRepoResult, Error>` - The result of the repository creation attempt or an error.
pub async fn handle_create_command(
    config: &Option<String>,
    name: &Option<String>,
    owner: &Option<String>,
    template: &Option<String>,
    ask_user_for_value: &AskUserForValue,
    get_org_rules: &GetOrgRulesFn,
    repository_creator: &impl RepositoryCreator,
) -> Result<repo_roller_core::CreateRepoResult, Error> {
    // Load config file if provided, otherwise start with empty values.
    let (mut final_name, mut final_owner, mut final_template) = if let Some(config_path) = config {
        match fs::read_to_string(config_path) {
            Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                Ok(cfg) => (
                    cfg.name.unwrap_or_default(),
                    cfg.owner.unwrap_or_default(),
                    cfg.template.unwrap_or_default(),
                ),
                Err(e) => {
                    // Return error if TOML parsing fails.
                    return Err(Error::ParseTomlFile(e));
                }
            },
            Err(e) => {
                // Return error if file cannot be loaded.
                return Err(Error::LoadFile(e));
            }
        }
    } else {
        (String::new(), String::new(), String::new())
    };

    // Overwrite config values with CLI arguments if provided.
    if name.is_some() {
        final_name = name.clone().unwrap_or_default();
    }
    if owner.is_some() {
        final_owner = owner.clone().unwrap_or_default();
    }
    if template.is_some() {
        final_template = template.clone().unwrap_or_default();
    }

    // Prompt for owner/org if still missing.
    if final_owner.trim().is_empty() {
        loop {
            final_owner = ask_user_for_value("Owner (user or org, required): ").unwrap_or_default();
            if !final_owner.is_empty() {
                break;
            }
            println!("  Error: Owner cannot be empty.");
        }
    }

    // Retrieve org-specific rules for validation (e.g., naming regex).
    let org_rules = get_org_rules(&final_owner);

    // Prompt for repository name if missing or invalid per org rules.
    match org_rules.repo_name_regex {
        Some(ref regex) => {
            let re = regex::Regex::new(regex).unwrap();
            let request = format!(
                "Repository name (required, must match org rules {:?}): ",
                regex
            );
            if !re.is_match(&final_name) {
                loop {
                    final_name = ask_user_for_value(request.as_str()).unwrap_or_default();
                    if re.is_match(&final_name) {
                        break;
                    }
                    println!(
                        "  Error: Name does not match org-specific naming rules: {:?}",
                        regex
                    );
                }
            }
        }
        None => {
            // If no org rules, prompt for name if still missing.
            if final_owner.trim().is_empty() {
                loop {
                    final_name = ask_user_for_value(
                        "Repository name (required, must be a valid GitHub repo name): ",
                    )
                    .unwrap_or_default();
                    if !final_name.is_empty() {
                        break;
                    }
                    println!("  Error: Name cannot be empty.");
                }
            }
        }
    }

    // Prompt for template type if missing.
    if final_template.trim().is_empty() {
        loop {
            final_template =
                ask_user_for_value("Template type (required, e.g., library, service, action): ")
                    .unwrap_or_default();
            if !final_template.is_empty() {
                break;
            }
            println!("  Error: Template type cannot be empty.");
        }
    }

    // Construct the repository creation request and delegate to the provided function.
    let req = repo_roller_core::CreateRepoRequest {
        name: final_name,
        owner: final_owner,
        template: final_template,
    };
    let result = repository_creator.create_repository(req).await;

    Ok(result)
}
