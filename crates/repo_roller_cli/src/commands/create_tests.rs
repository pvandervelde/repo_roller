use super::*;
use crate::errors::Error;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

// Helper to simulate user input
fn make_ask_user_for_value(input_text: &str) -> Result<String, Error> {
    Ok(input_text.to_string())
}

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

#[test]
fn test_happy_path_with_all_args() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        repo_roller_core::get_org_rules(org)
    };

    let log_clone = log.clone();
    let create_repository = move |req: repo_roller_core::CreateRepoRequest| {
        log_clone
            .lock()
            .unwrap()
            .create_repository_args
            .push(req.clone());
        repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        }
    };

    let result = handle_create_command(
        &None,
        &Some("repo1".to_string()),
        &Some("calvinverse".to_string()),
        &Some("library".to_string()),
        &ask,
        &get_org_rules,
        &create_repository,
    );
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

#[test]
fn test_happy_path_with_config_file() {
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
        repo_roller_core::get_org_rules(org)
    };

    let log_clone = log.clone();
    let create_repository = move |req: repo_roller_core::CreateRepoRequest| {
        log_clone
            .lock()
            .unwrap()
            .create_repository_args
            .push(req.clone());
        repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        }
    };

    let result = handle_create_command(
        &path,
        &None,
        &None,
        &None,
        &ask,
        &get_org_rules,
        &create_repository,
    );
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

#[test]
fn test_config_file_missing() {
    let ask = make_ask_user_for_value;
    let get_org_rules = |_org: &str| repo_roller_core::get_org_rules(_org);
    let create_repository =
        |_req: repo_roller_core::CreateRepoRequest| repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        };
    let result = handle_create_command(
        &Some("nonexistent.toml".to_string()),
        &None,
        &None,
        &None,
        &ask,
        &get_org_rules,
        &create_repository,
    );
    assert!(matches!(result, Err(Error::LoadFile(_))));
}

#[test]
fn test_config_file_invalid_toml() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "not valid toml").unwrap();
    let path = file.path().to_str().map(|s| s.to_string());
    let ask = make_ask_user_for_value;
    let get_org_rules = |_org: &str| repo_roller_core::get_org_rules(_org);
    let create_repository =
        |_req: repo_roller_core::CreateRepoRequest| repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        };
    let result = handle_create_command(
        &path,
        &None,
        &None,
        &None,
        &ask,
        &get_org_rules,
        &create_repository,
    );
    assert!(matches!(result, Err(Error::ParseTomlFile(_))));
}

#[test]
fn test_partial_args_prompt_for_owner() {
    // Only name and template provided, owner missing, should prompt
    let ask = |_prompt: &str| Ok("prompted_owner".to_string());
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        repo_roller_core::get_org_rules(org)
    };

    let log_clone = log.clone();
    let create_repository = move |req: repo_roller_core::CreateRepoRequest| {
        log_clone
            .lock()
            .unwrap()
            .create_repository_args
            .push(req.clone());
        repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        }
    };

    let result = handle_create_command(
        &None,
        &Some("repo3".to_string()),
        &None,
        &Some("library".to_string()),
        &ask,
        &get_org_rules,
        &create_repository,
    );
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.message, "stubbed");

    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["prompted_owner"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo3");
    assert_eq!(req.owner, "prompted_owner");
    assert_eq!(req.template, "library");
}

#[test]
fn test_partial_args_prompt_for_template() {
    // Only name and owner provided, template missing, should prompt
    let ask = |_prompt: &str| Ok("prompted_template".to_string());
    let log = Arc::new(Mutex::new(CallLog::new()));

    let log_clone = log.clone();
    let get_org_rules = move |org: &str| {
        log_clone
            .lock()
            .unwrap()
            .get_org_rules_args
            .push(org.to_string());
        repo_roller_core::get_org_rules(org)
    };

    let log_clone = log.clone();
    let create_repository = move |req: repo_roller_core::CreateRepoRequest| {
        log_clone
            .lock()
            .unwrap()
            .create_repository_args
            .push(req.clone());
        repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        }
    };

    let result = handle_create_command(
        &None,
        &Some("repo4".to_string()),
        &Some("calvinverse".to_string()),
        &None,
        &ask,
        &get_org_rules,
        &create_repository,
    );
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.message, "stubbed");

    let log = log.lock().unwrap();
    assert_eq!(log.get_org_rules_args, vec!["calvinverse"]);
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name, "repo4");
    assert_eq!(req.owner, "calvinverse");
    assert_eq!(req.template, "prompted_template");
}

#[test]
fn test_create_repository_failure() {
    // Simulate create_repository returning failure
    let ask = make_ask_user_for_value;
    let get_org_rules = |_org: &str| repo_roller_core::get_org_rules(_org);
    let create_repository =
        |_req: repo_roller_core::CreateRepoRequest| repo_roller_core::CreateRepoResult {
            success: false,
            message: "creation failed".to_string(),
        };

    let result = handle_create_command(
        &None,
        &Some("repo5".to_string()),
        &Some("calvinverse".to_string()),
        &Some("library".to_string()),
        &ask,
        &get_org_rules,
        &create_repository,
    );
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(!res.success);
    assert_eq!(res.message, "creation failed");
}

#[test]
fn test_config_file_missing_fields() {
    // Config file missing template, should prompt for it
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
        repo_roller_core::get_org_rules(org)
    };

    let log_clone = log.clone();
    let create_repository = move |req: repo_roller_core::CreateRepoRequest| {
        log_clone
            .lock()
            .unwrap()
            .create_repository_args
            .push(req.clone());
        repo_roller_core::CreateRepoResult {
            success: true,
            message: "stubbed".to_string(),
        }
    };

    let result = handle_create_command(
        &path,
        &None,
        &None,
        &None,
        &ask,
        &get_org_rules,
        &create_repository,
    );
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
