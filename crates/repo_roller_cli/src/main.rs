use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

/// RepoRoller CLI: Create new GitHub repositories from templates
#[derive(Parser)]
#[command(name = "repo-roller")]
#[command(about = "Create new GitHub repositories from templates", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new repository from a template
    Create {
        /// Path to a TOML config file with repository settings
        #[arg(long)]
        config: Option<String>,

        /// Name of the new repository
        #[arg(long)]
        name: String,

        /// Owner (user or org) for the new repository
        #[arg(long)]
        owner: String,

        /// Template type (e.g., library, service, action)
        #[arg(long)]
        template: String,

        /// Key-value pairs for template variables (e.g., key=value)
        #[arg(long, value_parser = parse_key_val, num_args = 0..)]
        variables: Vec<(String, String)>,
    },
    /// Show the CLI version
    Version,
    /// Show default settings
    ShowDefaults,
    /// List recognized template variables
    ListVariables,
    /// Show the status of the last operation
    Status,
    /// Initialize a repository config file
    Init,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[derive(serde::Deserialize)]
struct ConfigFile {
    name: Option<String>,
    owner: Option<String>,
    template: Option<String>,
    variables: Option<Vec<(String, String)>>,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Create {
            config,
            name,
            owner,
            template,
            variables,
        } => {
            // If a config file is provided, load it and override missing fields
            let mut name = name.clone();
            let mut owner = owner.clone();
            let mut template = template.clone();
            let mut variables = variables.clone();

            if let Some(config_path) = config {
                match fs::read_to_string(config_path) {
                    Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                        Ok(cfg) => {
                            if name.trim().is_empty() {
                                name = cfg.name.unwrap_or_default();
                            }
                            if owner.trim().is_empty() {
                                owner = cfg.owner.unwrap_or_default();
                            }
                            if template.trim().is_empty() {
                                template = cfg.template.unwrap_or_default();
                            }
                            if variables.is_empty() {
                                variables = cfg.variables.unwrap_or_default();
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse config file: {e}");
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read config file: {e}");
                        std::process::exit(1);
                    }
                }
            }

            if name.trim().is_empty() || owner.trim().is_empty() || template.trim().is_empty() {
                println!("Some required information is missing. Enter values interactively:");
                // Ask for owner/org first
                if owner.trim().is_empty() {
                    loop {
                        print!("Owner (user or org, required): ");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut owner).unwrap();
                        owner = owner.trim().to_string();
                        if !owner.is_empty() {
                            break;
                        }
                        println!("  Error: Owner cannot be empty.");
                        owner.clear();
                    }
                }
                // Fetch org-specific rules
                let org_rules = repo_roller_core::get_org_rules(&owner);
                // Now ask for name, applying org-specific rules if present
                if name.trim().is_empty() {
                    match org_rules.repo_name_regex {
                        Some(ref regex) => {
                            let re = regex::Regex::new(regex).unwrap();
                            loop {
                                print!(
                                    "Repository name (required, must match org rules {:?}): ",
                                    regex
                                );
                                io::stdout().flush().unwrap();
                                io::stdin().read_line(&mut name).unwrap();
                                name = name.trim().to_string();
                                if re.is_match(&name) {
                                    break;
                                }
                                println!(
                                    "  Error: Name does not match org-specific naming rules: {:?}",
                                    regex
                                );
                                name.clear();
                            }
                        }
                        None => {
                            loop {
                                print!("Repository name (required, must be a valid GitHub repo name): ");
                                io::stdout().flush().unwrap();
                                io::stdin().read_line(&mut name).unwrap();
                                name = name.trim().to_string();
                                if !name.is_empty() {
                                    break;
                                }
                                println!("  Error: Name cannot be empty.");
                                name.clear();
                            }
                        }
                    }
                }
                // (Removed duplicate name prompt block)
                if owner.trim().is_empty() {
                    loop {
                        print!("Owner (user or org, required): ");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut owner).unwrap();
                        owner = owner.trim().to_string();
                        if !owner.is_empty() {
                            break;
                        }
                        println!("  Error: Owner cannot be empty.");
                        owner.clear();
                    }
                }
                if template.trim().is_empty() {
                    loop {
                        print!("Template type (required, e.g., library, service, action): ");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut template).unwrap();
                        template = template.trim().to_string();
                        if !template.is_empty() {
                            break;
                        }
                        println!("  Error: Template type cannot be empty.");
                        template.clear();
                    }
                }
                // Prompt for variables if empty
                if variables.is_empty() {
                    println!("Enter template variables as key=value (empty line to finish):");
                    loop {
                        print!("Variable: ");
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim();
                        if input.is_empty() {
                            break;
                        }
                        match parse_key_val(input) {
                            Ok(kv) => variables.push(kv),
                            Err(e) => println!("  Error: {e}"),
                        }
                    }
                }
            }

            // Call core logic
            let req = repo_roller_core::CreateRepoRequest {
                name,
                owner,
                template,
                variables,
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
        Commands::Version => {
            // Print version info from baked-in value
            println!(
                "repo-roller version {}",
                option_env!("REPO_ROLLER_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
            );
            std::process::exit(0);
        }
        Commands::ShowDefaults => {
            println!("Default settings: (not yet implemented)");
            std::process::exit(0);
        }
        Commands::ListVariables => {
            println!("Recognized template variables: (not yet implemented)");
            std::process::exit(0);
        }
        Commands::Status => {
            println!("Status: (not yet implemented)");
            std::process::exit(0);
        }
        Commands::Init => {
            println!("Repository config initialization: (not yet implemented)");
            std::process::exit(0);
        }
    }
}
