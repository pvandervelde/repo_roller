use crate::{
    commands::auth_cmd::{KEY_RING_APP_ID, KEY_RING_APP_PRIVATE_KEY_PATH, KEY_RING_SERVICE_NAME},
    config::{get_config_path, AppConfig},
    errors::Error,
};
use clap::{arg, Args};
use keyring::Entry;
use repo_roller_core::{CreateRepoRequest, CreateRepoResult};
use std::{fs, future::Future};
use tracing::{debug, error, info};

/// Function type for prompting the user for a value interactively.
/// Used for required fields that may be missing from config or CLI args.
type AskUserForValue = dyn Fn(&str) -> Result<String, Error>;

/// Function type for retrieving organization-specific repository rules.
/// Allows for dependency injection and testability.
pub type GetOrgRulesFn = dyn Fn(&str) -> repo_roller_core::OrgRules;

#[cfg(test)]
#[path = "create_cmd_tests.rs"]
mod create_cmd_tests;

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

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Path to a TOML config file with repository settings
    #[arg(long)]
    pub config: Option<String>,

    /// Name of the new repository
    #[arg(long)]
    pub name: Option<String>,

    /// Owner (user or org) for the new repository
    #[arg(long)]
    pub owner: Option<String>,

    /// Template type (e.g., library, service, action)
    #[arg(long)]
    pub template: Option<String>,
}

/// Default repository creation logic using application config and GitHub App auth.
pub async fn create_repository(request: CreateRepoRequest) -> CreateRepoResult {
    let path = get_config_path(None);
    let config = match AppConfig::load(&path) {
        Ok(c) => c,
        Err(_) => {
            return CreateRepoResult::failure(
                "Failed to load the app config from the default path.",
            )
        }
    };

    let (app_id, app_key) = match get_authentication_tokens(&config).await {
        Ok(p) => p,
        Err(_) => {
            return CreateRepoResult::failure("Could not get the GitHub App ID and token.");
        }
    };

    repo_roller_core::create_repository(request, app_id, app_key).await
}

async fn get_authentication_tokens(config: &AppConfig) -> Result<(u64, String), Error> {
    debug!("Creating GitHub app client");
    let provider = match config.authentication.auth_method.as_str() {
        "token" => {
            let err = Error::InvalidArguments(format!(
                "Unsupported authentication method: {}",
                config.authentication.auth_method
            ));
            error!(message = "Failed to create GitHub app client", error = ?err);
            return Err(err);
        }
        "app" => {
            info!(message = "Using GitHub App authentication");
            let app_id = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
                .map_err(|e| {
                    Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    Error::Auth(format!("Failed to get app ID from the keyring: {}", e))
                })?;

            let app_key_path = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
                .map_err(|e| {
                    Error::Auth(format!("Failed to create an entry in the keyring: {}", e))
                })?
                .get_password()
                .map_err(|e| {
                    Error::Auth(format!(
                        "Failed to get app key location from the keyring: {}",
                        e
                    ))
                })?;

            let app_key = fs::read_to_string(app_key_path).map_err(|e| {
                Error::Config(format!(
                    "Failed to load the app key from the provided file: {}",
                    e
                ))
            })?;

            let app_id_number = app_id.parse::<u64>().map_err(|e| {
                Error::InvalidArguments(format!(
                    "Failed to parse the app ID. Expected a number, got {}. Error was: {}.",
                    app_id, e
                ))
            })?;

            (app_id_number, app_key)
        }
        _ => {
            let err = Error::InvalidArguments(format!(
                "Unsupported authentication method: {}",
                config.authentication.auth_method
            ));
            error!(message = "Failed to create GitHub app client", error = ?err);
            return Err(err);
        }
    };

    Ok(provider)
}

/// Handles the create command logic.
///
/// This function merges configuration from a TOML file and CLI arguments,
/// prompts the user for any missing required values, applies organization-specific
/// naming rules, and delegates repository creation to the provided function.
///
/// # Arguments
/// * `config` - Optional path to a TOML config file.
/// * `name` - Name of the new repository (overrides config).
/// * `owner` - Owner (user or org) for the new repository (overrides config).
/// * `template` - Template type (overrides config).
/// * `ask_user_for_value` - Function to prompt the user for missing values.
/// * `get_org_rules` - Function to retrieve org-specific rules for validation.
/// * `create_repository_fn` - Function to perform the actual repository creation.
///
/// # Returns
/// * `Result<CreateRepoResult, Error>` - The result of the repository creation attempt or an error.
pub async fn handle_create_command<F, Fut>(
    config: &Option<String>,
    name: &Option<String>,
    owner: &Option<String>,
    template: &Option<String>,
    ask_user_for_value: &AskUserForValue,
    get_org_rules: &GetOrgRulesFn,
    create_repository_fn: F,
) -> Result<CreateRepoResult, Error>
where
    F: Fn(CreateRepoRequest) -> Fut + Send + Sync,
    Fut: Future<Output = CreateRepoResult> + Send,
{
    // Load config file if provided, otherwise start with empty values.
    let (mut final_name, mut final_owner, mut final_template) = if let Some(cfg_path) = config {
        match fs::read_to_string(cfg_path) {
            Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                Ok(cfg) => (
                    cfg.name.unwrap_or_default(),
                    cfg.owner.unwrap_or_default(),
                    cfg.template.unwrap_or_default(),
                ),
                Err(e) => return Err(Error::ParseTomlFile(e)),
            },
            Err(e) => return Err(Error::LoadFile(e)),
        }
    } else {
        (String::new(), String::new(), String::new())
    };

    // Override with CLI args if provided.
    if let Some(n) = name {
        final_name = n.clone();
    }
    if let Some(o) = owner {
        final_owner = o.clone();
    }
    if let Some(t) = template {
        final_template = t.clone();
    }

    // Prompt for owner if missing.
    if final_owner.trim().is_empty() {
        loop {
            final_owner = ask_user_for_value("Owner (user or org, required): ").unwrap_or_default();
            if !final_owner.is_empty() {
                break;
            }
            println!("  Error: Owner cannot be empty.");
        }
    }

    // Apply org-specific naming rules.
    let org_rules = get_org_rules(&final_owner);
    if let Some(ref regex) = org_rules.repo_name_regex {
        let re = regex::Regex::new(regex).unwrap();
        let msg = format!("Repository name (must match org rules {:?}): ", regex);
        if !re.is_match(&final_name) {
            loop {
                final_name = ask_user_for_value(&msg).unwrap_or_default();
                if re.is_match(&final_name) {
                    break;
                }
                println!(
                    "  Error: Name does not match org-specific naming rules: {:?}",
                    regex
                );
            }
        }
    } else if final_name.trim().is_empty() {
        loop {
            final_name = ask_user_for_value("Repository name (required): ").unwrap_or_default();
            if !final_name.is_empty() {
                break;
            }
            println!("  Error: Name cannot be empty.");
        }
    }

    // Prompt for template if missing.
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

    // Build request and call provided function.
    let req = CreateRepoRequest {
        name: final_name,
        owner: final_owner,
        template: final_template,
    };
    Ok(create_repository_fn(req).await)
}
