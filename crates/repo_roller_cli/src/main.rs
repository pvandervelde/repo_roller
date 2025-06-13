use std::io;
use std::io::Write;

use async_trait::async_trait;
use clap::{Parser, Subcommand};
use tokio;

mod commands;

mod errors;
use commands::create::{handle_create_command, RepositoryCreator};
use errors::Error;

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
        name: Option<String>,

        /// Owner (user or org) for the new repository
        #[arg(long)]
        owner: Option<String>,

        /// Template type (e.g., library, service, action)
        #[arg(long)]
        template: Option<String>,
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

struct RepositoryCreatorBridge {}

#[async_trait]
impl RepositoryCreator for RepositoryCreatorBridge {
    async fn create_repository(
        &self,
        request: repo_roller_core::CreateRepoRequest,
    ) -> repo_roller_core::CreateRepoResult {
        repo_roller_core::create_repository_from_request(request).await
    }
}

fn ask_user_for_value(request: &str) -> Result<String, Error> {
    print!("{}", request);

    io::stdout().flush().map_err(|_| Error::StdOutFlushFailed)?;

    let mut temp = String::new();
    io::stdin().read_line(&mut temp).unwrap();
    Ok(temp.trim().to_string())
}

pub fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Create {
            config,
            name,
            owner,
            template,
        } => {
            // Use handle_create_command to merge config, prompt, and apply org rules
            let creator = RepositoryCreatorBridge {};
            let result = handle_create_command(
                config,
                name,
                owner,
                template,
                &ask_user_for_value,
                &|org| repo_roller_core::OrgRules::new_from_text(org),
                &creator,
            )
            .await;

            match result {
                Ok(res) => {
                    if res.success {
                        println!("Repository created");
                        std::process::exit(0);
                    } else {
                        println!("Failed to create repository: {}", res.message);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    println!("Error: {e}");
                    std::process::exit(2);
                }
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
