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

// The CLI should have the following commands
// - help --> returns the help text, exits with success code
// - version --> returns the version, exits with success code
// - create --> Creates a new repository
// - init --> Creates a repository config file that the user can update with their data
//
// The create mode allows 3 different ways of functioning:
// - The user provides a complete config file. It is assumed that this config file contains
//   all the information required. The CLI extracts the information from the config file, puts it
//   in a structure (defined in the core crate) and sends it to a method in the core crate for processing
//   The core crate method checks the structure for validity and returns errors (one for every invalid setting)
//   or creates the repository
// - The CLI asks the user questions and creates the structure from the answers to these questions. For some questions
//   the CLI should be able to provide guidelines or suggestions. For instance each org/personal area may have naming
//   guidelines for repository names. The CLI will ask the user for the name and provides the naming guidelines. It can
//   then check the name against the naming guidelines and suggest improvements if the name does not abide by the naming
//   guidelines. This also applies to other settings
// - The user provides a partial config file and the CLI asks the user follow up questions to obtain the rest of the
//   information.
//
// The CLI should be able to tell the user what the default settings are. These will also be provided in the default
// repository config. This may be done through a command.
//
// The CLI should be able to tell the user which template variables it recognises. This may be done through a command

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
