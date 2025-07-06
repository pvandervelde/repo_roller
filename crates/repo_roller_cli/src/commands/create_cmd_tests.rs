use super::*;
use crate::errors::Error;
use repo_roller_core::OrgRules;
use repo_roller_core::{CreateRepoRequest, CreateRepoResult};
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio;

// =============================================================================
// Test Helper Functions and Types
// =============================================================================

/// Helper function to simulate user input for testing.
/// Returns the provided input text as if the user had entered it.
fn make_ask_user_for_value(input_text: &str) -> Result<String, Error> {
    Ok(input_text.to_string())
}

/// Test helper struct to track function calls during testing.
/// Records arguments passed to mocked functions for verification.
#[derive(Debug, Clone)]
struct CallLog {
    get_org_rules_args: Vec<String>,
    create_repository_args: Vec<repo_roller_core::CreateRepoRequest>,
}

impl CallLog {
    fn new() -> Self {
        Self {
            get_org_rules_args: Vec::new(),
            create_repository_args: Vec::new(),
        }
    }
}

/// Creates a mock get_org_rules function that logs calls.
/// Returns a closure that can be used in tests to track org rule lookups.
fn make_logged_get_org_rules(log: Arc<Mutex<CallLog>>) -> impl Fn(&str) -> OrgRules + Send + Sync {
    move |org: &str| {
        log.lock().unwrap().get_org_rules_args.push(org.to_string());
        OrgRules::new_from_text(org)
    }
}

/// Creates a mock create_repository function that logs calls and returns failure.
/// Returns a closure that can be used in tests to track repository creation requests.
fn make_logged_create_repo_failure(
    log: Arc<Mutex<CallLog>>,
    failure_message: &'static str,
) -> impl Fn(
    CreateRepoRequest,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = CreateRepoResult> + Send>>
       + Send
       + Sync {
    move |req: CreateRepoRequest| {
        let log = log.clone();
        Box::pin(async move {
            log.lock().unwrap().create_repository_args.push(req.clone());
            CreateRepoResult::failure(failure_message)
        })
    }
}

// =============================================================================
// CLI Configuration Loading Tests
// =============================================================================

// =============================================================================
// Handle Create Command Integration Tests
// =============================================================================

#[tokio::test]
async fn test_cli_config_invalid_toml() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "not valid toml").unwrap();
    let path = file.path().to_str().map(|s| s.to_string());
    let ask = make_ask_user_for_value;
    let get_org_rules = |_org: &str| OrgRules::new_from_text(_org);

    let _log = Arc::new(Mutex::new(CallLog::new()));
    let options = CreateCommandOptions::new(&path, &None, &None, &None);
    let result =
        handle_create_command(options, ask, get_org_rules, |req| create_repository(req)).await;
    assert!(matches!(result, Err(Error::ParseTomlFile(_))));
}

#[tokio::test]
async fn test_cli_config_missing() {
    let ask = make_ask_user_for_value;
    let get_org_rules = |_org: &str| OrgRules::new_from_text(_org);

    let _log = Arc::new(Mutex::new(CallLog::new()));
    let config_file = Some("nonexistent.toml".to_string());
    let options = CreateCommandOptions::new(&config_file, &None, &None, &None);
    let result =
        handle_create_command(options, ask, get_org_rules, |req| create_repository(req)).await;
    assert!(matches!(result, Err(Error::LoadFile(_))));
}

#[tokio::test]
async fn test_cli_config_missing_fields() {
    // CLI config missing template, should prompt for it
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "name = \"repo6\"\nowner = \"calvinverse\"").unwrap();
    let path = file.path().to_str().map(|s| s.to_string());
    let ask = |_prompt: &str| Ok("prompted_template".to_string());
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        OrgRules::new_from_text(org)
    };

    let create_repo = {
        let log = log.clone();
        move |req: CreateRepoRequest| {
            let log = log.clone();
            async move {
                log.lock().unwrap().create_repository_args.push(req.clone());
                CreateRepoResult::success("stubbed")
            }
        }
    };

    let options = CreateCommandOptions::new(&path, &None, &None, &None);
    let result = handle_create_command(options, ask, get_org_rules, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.message, "stubbed");

    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["calvinverse"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo6");
    assert_eq!(req.owner, "calvinverse");
    assert_eq!(req.template, "prompted_template");
}

#[tokio::test]
async fn test_create_repository_failure() {
    // Simulate create_repository returning failure
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let get_org_rules = make_logged_get_org_rules(log.clone());
    let create_repo = make_logged_create_repo_failure(log.clone(), "creation failed");

    let repo_name = Some("repo5".to_string());
    let org_name = Some("calvinverse".to_string());
    let repo_type = Some("library".to_string());
    let options = CreateCommandOptions::new(&None, &repo_name, &org_name, &repo_type);
    let result = handle_create_command(options, ask, get_org_rules, create_repo).await;

    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(!res.success);
    assert_eq!(res.message, "creation failed");

    // Verify the logged calls
    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["calvinverse"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo5");
    assert_eq!(req.owner, "calvinverse");
    assert_eq!(req.template, "library");
}

#[tokio::test]
async fn test_happy_path_with_all_args() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        OrgRules::new_from_text(org)
    };

    let create_repo = {
        let log = log.clone();
        move |req: CreateRepoRequest| {
            let log = log.clone();
            async move {
                log.lock().unwrap().create_repository_args.push(req.clone());
                CreateRepoResult::success("stubbed")
            }
        }
    };

    let repo_name = Some("repo1".to_string());
    let org_name = Some("calvinverse".to_string());
    let repo_type = Some("library".to_string());
    let options = CreateCommandOptions::new(&None, &repo_name, &org_name, &repo_type);
    let result = handle_create_command(options, ask, get_org_rules, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.message, "stubbed");

    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["calvinverse"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo1");
    assert_eq!(req.owner, "calvinverse");
    assert_eq!(req.template, "library");
}

#[tokio::test]
async fn test_happy_path_with_cli_config() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        "name = \"repo2\"\nowner = \"calvinverse\"\ntemplate = \"service\""
    )
    .unwrap();
    let path = file.path().to_str().map(|s| s.to_string());
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        OrgRules::new_from_text(org)
    };

    let create_repo = {
        let log = log.clone();
        move |req: CreateRepoRequest| {
            let log = log.clone();
            async move {
                log.lock().unwrap().create_repository_args.push(req.clone());
                CreateRepoResult::success("stubbed")
            }
        }
    };

    let options = CreateCommandOptions::new(&path, &None, &None, &None);
    let result = handle_create_command(options, ask, get_org_rules, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.message, "stubbed");

    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["calvinverse"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo2");
    assert_eq!(req.owner, "calvinverse");
    assert_eq!(req.template, "service");
}

#[tokio::test]
async fn test_load_cli_config_invalid_file() {
    // Create an invalid TOML file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "invalid toml content [").unwrap();

    let path = file.path().to_str().unwrap();
    let result = load_cli_config(path);

    assert!(result.is_err());
    // The error should be a ParseTomlFile error from TOML parsing
    assert!(matches!(result.unwrap_err(), Error::ParseTomlFile(_)));
}

#[tokio::test]
async fn test_load_cli_config_missing_file() {
    let result = load_cli_config("nonexistent_file.toml");

    assert!(result.is_err());
    // The error should be a LoadFile error
    assert!(matches!(result.unwrap_err(), Error::LoadFile(_)));
}

#[tokio::test]
async fn test_load_cli_config_valid_file() {
    // Create a valid CLI config TOML file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "name = \"test-repo\"").unwrap();
    writeln!(file, "owner = \"test-owner\"").unwrap();
    writeln!(file, "template = \"test-template\"").unwrap();

    let path = file.path().to_str().unwrap();
    let result = load_cli_config(path);

    assert!(result.is_ok());
    let (name, owner, template) = result.unwrap();
    assert_eq!(name, "test-repo");
    assert_eq!(owner, "test-owner");
    assert_eq!(template, "test-template");
}

// =============================================================================
// Enhanced CLI Configuration Tests
// =============================================================================

#[tokio::test]
async fn test_load_cli_config_partial_fields() {
    // Test CLI config with only some fields present
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "name = \"partial-repo\"").unwrap();
    writeln!(file, "# owner field intentionally missing").unwrap();
    writeln!(file, "template = \"partial-template\"").unwrap();

    let path = file.path().to_str().unwrap();
    let result = load_cli_config(path);

    assert!(result.is_ok());
    let (name, owner, template) = result.unwrap();
    assert_eq!(name, "partial-repo");
    assert_eq!(owner, ""); // Should be empty string for missing field
    assert_eq!(template, "partial-template");
}

#[tokio::test]
async fn test_load_cli_config_empty_file() {
    // Test CLI config with empty TOML file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Empty config file with just comments").unwrap();

    let path = file.path().to_str().unwrap();
    let result = load_cli_config(path);

    assert!(result.is_ok());
    let (name, owner, template) = result.unwrap();
    assert_eq!(name, ""); // All fields should be empty
    assert_eq!(owner, "");
    assert_eq!(template, "");
}
