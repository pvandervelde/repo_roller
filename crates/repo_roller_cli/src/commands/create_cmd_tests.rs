use super::*;
use crate::errors::Error;
use repo_roller_core::{
    RepoRollerError, RepoRollerResult, RepositoryCreationRequest, RepositoryCreationResult,
    SystemError, Timestamp,
};
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
    create_repository_args: Vec<RepositoryCreationRequest>,
}

impl CallLog {
    fn new() -> Self {
        Self {
            create_repository_args: Vec::new(),
        }
    }
}

/// Creates a mock create_repository function that logs calls and returns success.
/// Returns a closure that can be used in tests to track repository creation requests.
fn make_logged_create_repo_success(
    log: Arc<Mutex<CallLog>>,
) -> impl Fn(
    RepositoryCreationRequest,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = RepoRollerResult<RepositoryCreationResult>> + Send>,
> + Send
       + Sync {
    move |req: RepositoryCreationRequest| {
        let log = log.clone();
        Box::pin(async move {
            log.lock().unwrap().create_repository_args.push(req.clone());
            Ok(RepositoryCreationResult {
                repository_url: "https://github.com/test/test-repo".to_string(),
                repository_id: "test-id-123".to_string(),
                created_at: Timestamp::now(),
                default_branch: "main".to_string(),
            })
        })
    }
}

/// Creates a mock create_repository function that logs calls and returns failure.
/// Returns a closure that can be used in tests to track repository creation requests.
fn make_logged_create_repo_failure(
    log: Arc<Mutex<CallLog>>,
    failure_message: &'static str,
) -> impl Fn(
    RepositoryCreationRequest,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = RepoRollerResult<RepositoryCreationResult>> + Send>,
> + Send
       + Sync {
    move |req: RepositoryCreationRequest| {
        let log = log.clone();
        let msg = failure_message.to_string();
        Box::pin(async move {
            log.lock().unwrap().create_repository_args.push(req.clone());
            Err(RepoRollerError::System(SystemError::Internal {
                reason: msg,
            }))
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

    let _log = Arc::new(Mutex::new(CallLog::new()));
    let options = CreateCommandOptions::new(&path, &None, &None, &None, false, false, false);
    let result = handle_create_command(options, ask, create_repository).await;
    assert!(matches!(result, Err(Error::ParseTomlFile(_))));
}

#[tokio::test]
async fn test_cli_config_missing() {
    let ask = make_ask_user_for_value;

    let _log = Arc::new(Mutex::new(CallLog::new()));
    let config_file = Some("nonexistent.toml".to_string());
    let options = CreateCommandOptions::new(&config_file, &None, &None, &None, false, false, false);
    let result = handle_create_command(options, ask, create_repository).await;
    assert!(matches!(result, Err(Error::LoadFile(_))));
}

#[tokio::test]
async fn test_cli_config_missing_fields() {
    // CLI config missing template, should prompt for it
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "name = \"repo6\"\nowner = \"calvinverse\"").unwrap();
    let path = file.path().to_str().map(|s| s.to_string());
    // Return a valid template name (lowercase only)
    let ask = |_prompt: &str| Ok("library".to_string());
    let log = Arc::new(Mutex::new(CallLog::new()));

    let create_repo = make_logged_create_repo_success(log.clone());

    let options = CreateCommandOptions::new(&path, &None, &None, &None, false, false, false);
    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert_eq!(res.repository_url, "https://github.com/test/test-repo");

    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "repo6");
    assert_eq!(req.owner.as_str(), "calvinverse");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "library");
}

#[tokio::test]
async fn test_create_repository_failure() {
    // Simulate create_repository returning failure
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let create_repo = make_logged_create_repo_failure(log.clone(), "creation failed");

    let repo_name = Some("repo5".to_string());
    let org_name = Some("calvinverse".to_string());
    let repo_type = Some("library".to_string());
    let options = CreateCommandOptions::new(
        &None, &repo_name, &org_name, &repo_type, false, false, false,
    );
    let result = handle_create_command(options, ask, create_repo).await;

    // Now returns Error instead of Ok(CreateRepoResult::failure)
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("creation failed"));

    // Verify the logged calls
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "repo5");
    assert_eq!(req.owner.as_str(), "calvinverse");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "library");
}

#[tokio::test]
async fn test_happy_path_with_all_args() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));

    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("repo1".to_string());
    let org_name = Some("calvinverse".to_string());
    let repo_type = Some("library".to_string());
    let options = CreateCommandOptions::new(
        &None, &repo_name, &org_name, &repo_type, false, false, false,
    );
    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert_eq!(res.repository_url, "https://github.com/test/test-repo");

    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "repo1");
    assert_eq!(req.owner.as_str(), "calvinverse");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "library");
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

    let create_repo = make_logged_create_repo_success(log.clone());

    let options = CreateCommandOptions::new(&path, &None, &None, &None, false, false, false);
    let result = handle_create_command(options, ask, create_repo).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    assert_eq!(res.repository_url, "https://github.com/test/test-repo");

    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "repo2");
    assert_eq!(req.owner.as_str(), "calvinverse");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "service");
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

// =============================================================================
// Empty Repository and Custom Init Tests
// =============================================================================

/// Verify that --empty flag creates repository with Empty content strategy.
#[tokio::test]
async fn test_empty_flag_creates_empty_repository() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("empty-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, true, false, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "empty-repo");
    assert_eq!(req.owner.as_str(), "test-org");
    // TODO (Task 6.7): After implementation, verify content_strategy is Empty
    // assert!(matches!(req.content_strategy, ContentStrategy::Empty));
}

/// Verify that --empty with --template uses template settings but no content.
#[tokio::test]
async fn test_empty_with_template_uses_template_settings() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("empty-templated-repo".to_string());
    let org_name = Some("test-org".to_string());
    let template = Some("rust-service".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &template, true, false, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "empty-templated-repo");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "rust-service");
    // TODO (Task 6.7): Verify content_strategy is Empty despite having template
    // assert!(matches!(req.content_strategy, ContentStrategy::Empty));
}

/// Verify that --init-readme flag creates repository with CustomInit strategy.
#[tokio::test]
async fn test_init_readme_flag_creates_custom_init_repository() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("readme-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, false, true, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "readme-repo");
    // TODO (Task 6.7): Verify content_strategy is CustomInit with include_readme=true
    // assert!(matches!(req.content_strategy, ContentStrategy::CustomInit(_)));
}

/// Verify that --init-gitignore flag creates repository with CustomInit strategy.
#[tokio::test]
async fn test_init_gitignore_flag_creates_custom_init_repository() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("gitignore-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, false, false, true);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "gitignore-repo");
    // TODO (Task 6.7): Verify content_strategy is CustomInit with include_gitignore=true
    // assert!(matches!(req.content_strategy, ContentStrategy::CustomInit(_)));
}

/// Verify that --init-readme --init-gitignore creates repository with both files.
#[tokio::test]
async fn test_init_readme_and_gitignore_creates_both_files() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("both-init-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options = CreateCommandOptions::new(&None, &repo_name, &org_name, &None, false, true, true);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "both-init-repo");
    // TODO (Task 6.7): Verify content_strategy is CustomInit with both flags set
    // assert!(matches!(req.content_strategy, ContentStrategy::CustomInit(opts) if opts.include_readme && opts.include_gitignore));
}

/// Verify that init flags with template use template settings.
#[tokio::test]
async fn test_init_flags_with_template_uses_template_settings() {
    let ask = make_ask_user_for_value;
    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("init-with-template-repo".to_string());
    let org_name = Some("test-org".to_string());
    let template = Some("rust-library".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &template, false, true, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    assert_eq!(log.create_repository_args.len(), 1);
    let req = &log.create_repository_args[0];
    assert_eq!(req.name.as_str(), "init-with-template-repo");
    assert_eq!(req.template.as_ref().unwrap().as_str(), "rust-library");
    // TODO (Task 6.7): Verify content_strategy is CustomInit with include_readme=true
    // assert!(matches!(req.content_strategy, ContentStrategy::CustomInit(_)));
}

/// Verify that template is not required when using --empty flag.
#[tokio::test]
async fn test_empty_flag_does_not_require_template() {
    // This test verifies that the prompt for template is skipped when --empty is set
    let ask_count = Arc::new(Mutex::new(0));
    let ask_count_clone = ask_count.clone();
    let ask = move |prompt: &str| {
        *ask_count_clone.lock().unwrap() += 1;
        Ok(format!("unexpected-prompt: {}", prompt))
    };

    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("no-template-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, true, false, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    // Should not have prompted for template
    let ask_count = ask_count.lock().unwrap();
    assert_eq!(
        *ask_count, 0,
        "Should not prompt for template when --empty is used"
    );
}

/// Verify that template is not required when using init flags.
#[tokio::test]
async fn test_init_flags_do_not_require_template() {
    let ask_count = Arc::new(Mutex::new(0));
    let ask_count_clone = ask_count.clone();
    let ask = move |prompt: &str| {
        *ask_count_clone.lock().unwrap() += 1;
        Ok(format!("unexpected-prompt: {}", prompt))
    };

    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("init-no-template-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, false, true, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    // Should not have prompted for template
    let ask_count = ask_count.lock().unwrap();
    assert_eq!(
        *ask_count, 0,
        "Should not prompt for template when init flags are used"
    );
}

/// Verify that template is still required when no special flags are set.
#[tokio::test]
async fn test_template_required_without_special_flags() {
    let ask = |_prompt: &str| Ok("required-template".to_string());

    let log = Arc::new(Mutex::new(CallLog::new()));
    let create_repo = make_logged_create_repo_success(log.clone());

    let repo_name = Some("normal-repo".to_string());
    let org_name = Some("test-org".to_string());
    let options =
        CreateCommandOptions::new(&None, &repo_name, &org_name, &None, false, false, false);

    let result = handle_create_command(options, ask, create_repo).await;

    assert!(result.is_ok());
    let log = log.lock().unwrap();
    let req = &log.create_repository_args[0];
    // Template should have been prompted and set
    assert_eq!(req.template.as_ref().unwrap().as_str(), "required-template");
}
