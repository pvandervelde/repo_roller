use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create {
            name,
            owner,
            template,
            variables,
        } => {
            // Interactive/config-driven create logic
            use std::fs;
            use std::io::{self, Write};

            // Check for config file argument (future: add --config option)
            // For now, if all fields are empty, prompt interactively
            let mut name = name.clone();
            let mut owner = owner.clone();
            let mut template = template.clone();
            let mut variables = variables.clone();

            if name.trim().is_empty() || owner.trim().is_empty() || template.trim().is_empty() {
                println!("Some required information is missing. Enter values interactively:");
                if name.trim().is_empty() {
                    print!("Repository name: ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut name).unwrap();
                    name = name.trim().to_string();
                }
                if owner.trim().is_empty() {
                    print!("Owner (user or org): ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut owner).unwrap();
                    owner = owner.trim().to_string();
                }
                if template.trim().is_empty() {
                    print!("Template type: ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut template).unwrap();
                    template = template.trim().to_string();
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
                println!("Repository created successfully: {}", result.message);
                std::process::exit(0);
            } else {
                eprintln!("Repository creation failed: {}", result.message);
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
