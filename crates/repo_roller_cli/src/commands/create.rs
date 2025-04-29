//! Implementation of the `create` command for the RepoRoller CLI.
//!
//! Handles argument parsing, config loading, interactive prompts, and
//! calls into the core orchestration logic to create a new repository.

use std::fs;

use crate::errors::Error;

type AskUserForValue = dyn Fn(&str) -> Result<String, Error>;

/// Configuration file structure for repository creation.
#[derive(serde::Deserialize)]
pub struct ConfigFile {
    pub name: Option<String>,
    pub owner: Option<String>,
    pub template: Option<String>,
}

/// Handles the create command logic.
///
/// # Arguments
/// * `config` - Optional path to a TOML config file.
/// * `name` - Name of the new repository.
/// * `owner` - Owner (user or org) for the new repository.
/// * `template` - Template type (e.g., library, service, action).
/// * `variables` - Key-value pairs for template variables.
///
/// # Returns
/// * `()` - Exits the process with appropriate status code.
pub fn handle_create_command(
    config: &Option<String>,
    name: &Option<String>,
    owner: &Option<String>,
    template: &Option<String>,
    ask_user_for_value: &AskUserForValue,
) -> Result<(), Error> {
    // If a config file is provided, load it
    let (mut final_name, mut final_owner, mut final_template) = if let Some(config_path) = config {
        match fs::read_to_string(config_path) {
            Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                Ok(cfg) => (
                    cfg.name.unwrap_or_default(),
                    cfg.owner.unwrap_or_default(),
                    cfg.template.unwrap_or_default(),
                ),
                Err(e) => {
                    return Err(Error::ParseTomlFile(e));
                }
            },
            Err(e) => {
                return Err(Error::LoadFile(e));
            }
        }
    } else {
        (String::new(), String::new(), String::new())
    };

    // Overwrite the values from the config file with the values provided on the command line.
    if name.is_some() {
        final_name = name.clone().unwrap_or_default();
    }

    if owner.is_some() {
        final_owner = owner.clone().unwrap_or_default();
    }

    if template.is_some() {
        final_template = template.clone().unwrap_or_default();
    }

    // Ask for owner/org first
    if final_owner.trim().is_empty() {
        loop {
            final_owner = ask_user_for_value("Owner (user or org, required): ").unwrap_or_default();
            if !final_owner.is_empty() {
                break;
            }
            println!("  Error: Owner cannot be empty.");
        }
    }

    // Fetch org-specific rules
    let org_rules = repo_roller_core::get_org_rules(&final_owner);

    // Now ask for name, applying org-specific rules if present
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

    // Call core logic
    let req = repo_roller_core::CreateRepoRequest {
        name: final_name,
        owner: final_owner,
        template: final_template,
    };
    let result = repo_roller_core::create_repository(req);

    if result.success {
        println!();
        println!("✅ Repository created successfully!");
        if !result.message.trim().is_empty() {
            println!("  Details: {}", result.message);
        }
        println!();
        println!("You can now navigate to your new repository on GitHub.");
        println!("If you provided template variables, review the generated files for correctness.");
        std::process::exit(0);
    } else {
        println!();
        eprintln!("❌ Repository creation failed.");
        if !result.message.trim().is_empty() {
            eprintln!("  Reason: {}", result.message);
        }
        eprintln!("Please check your input and try again, or consult the documentation for troubleshooting tips.");
        std::process::exit(1);
    }
}
