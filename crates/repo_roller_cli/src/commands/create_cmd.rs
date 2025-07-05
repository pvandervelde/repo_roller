//! Repository creation command module.
//!
//! This module handles the creation of new GitHub repositories from templates.
//! It supports loading configuration from TOML files, prompting users for missing
//! values, applying organization-specific naming rules, and delegating to the
//! core repository creation logic.
//!
//! ## Features
//!
//! - Configuration file support (TOML format)
//! - Interactive prompting for missing required values
//! - Organization-specific repository naming rules
//! - GitHub App and Personal Access Token authentication
//! - Template-based repository creation

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

#[cfg(test)]
#[path = "create_cmd_tests.rs"]
mod create_cmd_tests;

/// Configuration structure for repository creation loaded from TOML files.
///
/// This structure represents the optional configuration that can be loaded
/// from a TOML file to provide default values for repository creation.
/// All fields are optional and can be overridden by CLI arguments.
///
/// # Example TOML
///
/// ```toml
/// name = "my-new-repo"
/// owner = "my-organization"
/// template = "rust-library"
/// ```
#[derive(serde::Deserialize)]
pub struct ConfigFile {
    /// The name of the repository to create.
    /// Can be overridden by the `--name` CLI argument.
    pub name: Option<String>,

    /// The owner (user or organization) for the repository.
    /// Can be overridden by the `--owner` CLI argument.
    pub owner: Option<String>,

    /// The template type to use for repository creation.
    /// Can be overridden by the `--template` CLI argument.
    pub template: Option<String>,
}

/// Temporary compatibility layer for loading configuration using AppConfig.
///
/// This function loads configuration using the full AppConfig structure but
/// extracts only the fields needed for repository creation. This provides
/// a compatibility layer while we refactor away from the ConfigFile struct.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file to load
///
/// # Returns
///
/// Returns a Result containing a tuple of (name, owner, template) extracted
/// from the configuration file, or an Error if loading fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be read
/// - The configuration file contains invalid TOML
/// - The AppConfig structure cannot be deserialized
fn load_config_with_app_config(config_path: &str) -> Result<(String, String, String), Error> {
    let config_file_path = std::path::Path::new(config_path);
    let app_config = AppConfig::load(config_file_path)?;

    // For now, we'll use empty strings as defaults since the original ConfigFile
    // approach would have resulted in empty strings for missing fields anyway.
    // In the future, we can implement more sophisticated template matching logic.
    let name = String::new();
    let owner = String::new();
    let template = String::new();

    // TODO: In the future, we could implement logic to:
    // 1. Check if there's a default template in the config
    // 2. Extract default owner/name from template configuration
    // 3. Use template-specific defaults

    Ok((name, owner, template))
}

/// Command-line arguments for the create command.
///
/// This structure defines all the command-line options available for
/// the repository creation command. Arguments provided via CLI will
/// override any corresponding values from configuration files.
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Path to a TOML configuration file containing repository settings.
    ///
    /// The configuration file can specify default values for name, owner,
    /// and template. CLI arguments will override these defaults.
    #[arg(long)]
    pub config: Option<String>,

    /// Name of the new repository to create.
    ///
    /// Must follow GitHub repository naming conventions and any
    /// organization-specific naming rules.
    #[arg(long)]
    pub name: Option<String>,

    /// Owner (user or organization) for the new repository.
    ///
    /// Must be a valid GitHub username or organization name that
    /// the authenticated user has permission to create repositories for.
    #[arg(long)]
    pub owner: Option<String>,

    /// Template type to use for repository creation.
    ///
    /// Specifies which template should be used as the basis for the
    /// new repository (e.g., "library", "service", "action").
    #[arg(long)]
    pub template: Option<String>,
}

/// Creates a repository using the default application configuration and authentication.
///
/// This function loads the application configuration from the default path,
/// retrieves authentication credentials from the system keyring, and delegates
/// to the core repository creation logic.
///
/// # Arguments
///
/// * `request` - The repository creation request containing name, owner, and template
///
/// # Returns
///
/// Returns a `CreateRepoResult` indicating success or failure with a descriptive message.
///
/// # Errors
///
/// This function returns a failure result if:
/// - The application configuration cannot be loaded
/// - Authentication credentials cannot be retrieved from the keyring
/// - The core repository creation process fails
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

    // Use the new function that accepts config directly
    repo_roller_core::create_repository_with_config(request, &config.core, app_id, app_key).await
}

/// Retrieves GitHub authentication tokens from the system keyring.
///
/// Based on the authentication method configured in the app config, this function
/// retrieves the appropriate credentials from the system keyring. For GitHub App
/// authentication, it returns the App ID and private key. For token authentication,
/// it currently returns an error as this method is not yet implemented.
///
/// # Arguments
///
/// * `config` - The application configuration containing the auth method
///
/// # Returns
///
/// Returns a tuple of (app_id, private_key) for successful GitHub App authentication,
/// or an Error for unsupported methods or credential retrieval failures.
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

/// Handles the complete repository creation workflow.
///
/// This function orchestrates the entire repository creation process by:
/// 1. Loading configuration from TOML files if specified
/// 2. Merging CLI arguments with configuration values
/// 3. Prompting users for any missing required values
/// 4. Applying organization-specific naming rules and validation
/// 5. Delegating to the actual repository creation function
///
/// The function is designed to be testable through dependency injection
/// of the user input, rule retrieval, and repository creation functions.
///
/// # Arguments
///
/// * `config` - Optional path to a TOML configuration file
/// * `name` - Repository name from CLI (overrides config file)
/// * `owner` - Repository owner from CLI (overrides config file)
/// * `template` - Template type from CLI (overrides config file)
/// * `ask_user_for_value` - Function to prompt user for missing values
/// * `get_org_rules` - Function to retrieve organization-specific rules
/// * `create_repository_fn` - Function to perform actual repository creation
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(CreateRepoResult)` - The result of the repository creation attempt
/// - `Err(Error)` - If configuration loading or validation fails
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be read or parsed
/// - Required values cannot be obtained from user input
/// - Organization rules validation fails
pub async fn handle_create_command<F, Fut, AskFn, RulesFn>(
    options: CreateCommandOptions<'_>,
    ask_user_for_value: AskFn,
    get_org_rules: RulesFn,
    create_repository_fn: F,
) -> Result<CreateRepoResult, Error>
where
    F: Fn(CreateRepoRequest) -> Fut + Send + Sync,
    Fut: Future<Output = CreateRepoResult> + Send,
    AskFn: Fn(&str) -> Result<String, Error>,
    RulesFn: for<'a> Fn(&'a str) -> repo_roller_core::OrgRules,
{
    // Load config file if provided, otherwise start with empty values.
    let (mut final_name, mut final_owner, mut final_template) =
        if let Some(cfg_path) = options.config {
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
    if let Some(n) = options.name {
        final_name = n.clone();
    }
    if let Some(o) = options.owner {
        final_owner = o.clone();
    }
    if let Some(t) = options.template {
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

/// Options for the create command, grouping CLI arguments and configuration.
///
/// This struct reduces the parameter count of handle_create_command by grouping
/// related CLI arguments together into a single parameter.
#[derive(Debug)]
pub struct CreateCommandOptions<'a> {
    /// Path to a TOML configuration file containing repository settings.
    pub config: &'a Option<String>,
    /// Name of the new repository to create.
    pub name: &'a Option<String>,
    /// Owner (user or organization) for the new repository.
    pub owner: &'a Option<String>,
    /// Template type to use for repository creation.
    pub template: &'a Option<String>,
}

impl<'a> CreateCommandOptions<'a> {
    /// Creates new CreateCommandOptions from individual CLI arguments.
    pub fn new(
        config: &'a Option<String>,
        name: &'a Option<String>,
        owner: &'a Option<String>,
        template: &'a Option<String>,
    ) -> Self {
        Self {
            config,
            name,
            owner,
            template,
        }
    }
}
