use clap::{Parser, Subcommand};

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
            // Argument validation
            if name.trim().is_empty() {
                eprintln!("Error: --name cannot be empty");
                std::process::exit(1);
            }
            if owner.trim().is_empty() {
                eprintln!("Error: --owner cannot be empty");
                std::process::exit(1);
            }
            if template.trim().is_empty() {
                eprintln!("Error: --template cannot be empty");
                std::process::exit(1);
            }

            // Call core logic
            let req = repo_roller_core::CreateRepoRequest {
                name: name.clone(),
                owner: owner.clone(),
                template: template.clone(),
                variables: variables.clone(),
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
            // Print version info
            println!("repo-roller version {}", env!("CARGO_PKG_VERSION"));
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
