use clap::CommandFactory;
use repo_roller_cli::Cli;

#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}
