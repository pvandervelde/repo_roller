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
            println!(
                "Would create repo '{}' for owner '{}' using template '{}' with variables: {:?}",
                name, owner, template, variables
            );
            // TODO: Call repo_roller_core logic here
        }
    }
}
