use super::Cli;
use clap::CommandFactory;
use std::collections::HashSet;
use template_engine::{BuiltInVariablesParams, TemplateProcessor};

#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}

/// Verifies that the variable names printed by `list-variables` match the keys
/// actually produced by `template_engine::generate_built_in_variables()`.
///
/// If a variable is added, renamed, or removed in the template engine this test
/// will fail, preventing the CLI output from silently drifting out of sync.
#[test]
fn list_variables_matches_template_engine_built_ins() {
    let processor = TemplateProcessor::new().expect("TemplateProcessor::new");
    let params = BuiltInVariablesParams {
        repo_name: "test-repo",
        org_name: "test-org",
        template_name: "test-template",
        template_repo: "test-org/test-template",
        user_login: "test-user",
        user_name: "Test User",
        default_branch: "main",
    };
    let engine_keys: HashSet<String> = processor
        .generate_built_in_variables(&params)
        .into_keys()
        .collect();

    // These must stay in sync with the println! calls in Commands::ListVariables.
    let documented_keys: HashSet<String> = [
        "repo_name",
        "org_name",
        "user_login",
        "user_name",
        "template_name",
        "template_repo",
        "default_branch",
        "timestamp",
        "timestamp_unix",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let missing_from_docs: Vec<&String> = engine_keys.difference(&documented_keys).collect();
    let missing_from_engine: Vec<&String> = documented_keys.difference(&engine_keys).collect();

    assert!(
        missing_from_docs.is_empty(),
        "template_engine produces variables not documented in list-variables: {:?}",
        missing_from_docs
    );
    assert!(
        missing_from_engine.is_empty(),
        "list-variables documents variables not produced by template_engine: {:?}",
        missing_from_engine
    );
}
