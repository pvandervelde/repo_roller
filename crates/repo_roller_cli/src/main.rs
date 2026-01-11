//! # RepoRoller CLI
//!
//! A command-line interface for creating GitHub repositories from templates.
//!
//! This crate provides the main CLI application that allows users to:
//! - Create new repositories from predefined templates
//! - Configure authentication settings
//! - Manage repository configuration files
//! - List available template variables
//!
//! ## Usage
//!
//! ```bash
//! repo-roller create --name my-repo --owner my-org --template rust-library
//! ```

use std::io;
use std::io::Write;

use clap::{Parser, Subcommand};

use tracing::error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod commands;
use commands::create_cmd::{create_repository, handle_create_command, CreateCommandOptions};

mod config;

mod errors;
use errors::Error;

use crate::commands::{
    auth_cmd::AuthCommands, config_cmd::ConfigCommands, create_cmd::CreateArgs,
    org_settings_cmd::OrgSettingsCommands, template_cmd::TemplateCommands,
};

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

/// Available CLI commands for the RepoRoller application.
///
/// Each command provides different functionality for managing repositories,
/// authentication, and configuration.
#[derive(Subcommand)]
enum Commands {
    /// Authentication-related commands for managing GitHub credentials
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Configuration management commands for templates and settings
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Create a new repository from a template
    #[command()]
    Create(CreateArgs),

    /// Initialize a repository config file in the current directory
    Init,

    /// List recognized template variables and their descriptions
    ListVariables,

    /// Organization settings inspection commands
    #[command(subcommand)]
    OrgSettings(OrgSettingsCommands),

    /// Template inspection and validation commands
    #[command(subcommand)]
    Template(TemplateCommands),

    /// Show the CLI version information
    Version,
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
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_env("REPO_ROLLER_LOG"))
        .init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Auth(cmd) => {
            if let Err(e) = crate::commands::auth_cmd::execute(cmd).await {
                error!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Config(cmd) => {
            if let Err(e) = crate::commands::config_cmd::execute(cmd).await {
                error!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Create(args) => {
            // Use handle_create_command to merge config, prompt, and validate
            let options = CreateCommandOptions::new(
                &args.config,
                &args.name,
                &args.owner,
                &args.template,
                args.empty,
                args.init_readme,
                args.init_gitignore,
            );
            let result =
                handle_create_command(options, &ask_user_for_value, create_repository).await;

            match result {
                Ok(creation_result) => {
                    println!("Repository created successfully!");
                    println!("  URL: {}", creation_result.repository_url);
                    println!("  ID: {}", creation_result.repository_id);
                    println!("  Default branch: {}", creation_result.default_branch);
                    println!("  Created at: {}", creation_result.created_at);
                    std::process::exit(0);
                }
                Err(e) => {
                    println!("Failed to create repository: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Init => {
            println!("Repository config initialization: (not yet implemented)");
            std::process::exit(0);
        }
        Commands::ListVariables => {
            println!("Recognized template variables: (not yet implemented)");
            std::process::exit(0);
        }
        Commands::OrgSettings(cmd) => {
            if let Err(e) = crate::commands::org_settings_cmd::execute(cmd).await {
                error!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Template(cmd) => {
            if let Err(e) = crate::commands::template_cmd::execute(cmd).await {
                error!("Error: {e}");
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
    }
}
