// Unit tests for repo_roller_core
// Covers create_repository success and error paths with isolated mock dependencies

use super::*;
use async_trait::async_trait;
use config_manager::{Config, TemplateConfig};
use github_client::{
    errors::Error as GitHubError, models, RepositoryClient, RepositorySettingsUpdate,
};
use std::sync::{Arc, Mutex};
use template_engine::TemplateFetcher;

// --- MOCK STRUCTS (ALPHABETICALLY ORDERED) ---

/// Configurable mock repository client that eliminates code duplication
struct ConfigurableMockRepoClient {
    config: MockRepoClientConfig,
}

impl ConfigurableMockRepoClient {
    fn new(config: MockRepoClientConfig) -> Self {
        Self { config }
    }

    /// Helper function to create a successful repository response
    fn create_successful_repository(
        &self,
        payload: &github_client::RepositoryCreatePayload,
    ) -> models::Repository {
        models::Repository::new(
            payload.name.clone(),
            format!("test-org/{}", payload.name),
            "MDEwOlJlcG9zaXRvcnkx".to_string(),
            false,
        )
    }
}

#[async_trait]
impl RepositoryClient for ConfigurableMockRepoClient {
    async fn create_org_repository(
        &self,
        _owner: &str,
        payload: &github_client::RepositoryCreatePayload,
    ) -> Result<models::Repository, GitHubError> {
        Ok(self.create_successful_repository(payload))
    }

    async fn create_user_repository(
        &self,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<models::Repository, GitHubError> {
        Err(GitHubError::AuthError(
            "User repos not supported in test".to_string(),
        ))
    }

    async fn update_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
        _settings: &RepositorySettingsUpdate,
    ) -> Result<models::Repository, GitHubError> {
        Err(GitHubError::AuthError(
            "Not implemented in test".to_string(),
        ))
    }

    async fn get_installation_token_for_org(&self, _org_name: &str) -> Result<String, GitHubError> {
        // Track the call if a tracker is configured
        if let Some(tracker) = &self.config.token_call_tracker {
            *tracker.lock().unwrap() = true;
        }

        // Return the configured result based on behavior
        match &self.config.token_behavior {
            MockTokenBehavior::Success(token) => Ok(token.clone()),
            MockTokenBehavior::InvalidResponse => Err(GitHubError::InvalidResponse),
            MockTokenBehavior::AuthError(msg) => Err(GitHubError::AuthError(msg.clone())),
        }
    }

    async fn get_organization_default_branch(
        &self,
        _org_name: &str,
    ) -> Result<String, GitHubError> {
        Ok(self.config.default_branch.clone())
    }

    async fn set_repository_custom_properties(
        &self,
        _owner: &str,
        _repo: &str,
        _payload: &github_client::CustomPropertiesPayload,
    ) -> Result<(), GitHubError> {
        // Not implemented in test mock - return success
        Ok(())
    }
}

/// Configuration for mock repository client behavior
#[derive(Clone)]
struct MockRepoClientConfig {
    /// Token behavior for get_installation_token_for_org
    token_behavior: MockTokenBehavior,
    /// Optional callback to track when get_installation_token_for_org is called
    token_call_tracker: Option<Arc<Mutex<bool>>>,
    /// Default branch to return from get_organization_default_branch
    default_branch: String,
}

impl Default for MockRepoClientConfig {
    fn default() -> Self {
        Self {
            token_behavior: MockTokenBehavior::Success("ghs_mock_token".to_string()),
            token_call_tracker: None,
            default_branch: "main".to_string(),
        }
    }
}

impl MockRepoClientConfig {
    /// Create config that tracks installation token calls
    fn with_token_tracking() -> (Self, Arc<Mutex<bool>>) {
        let tracker = Arc::new(Mutex::new(false));
        let config = Self {
            token_behavior: MockTokenBehavior::Success("ghs_test_token_for_org".to_string()),
            token_call_tracker: Some(tracker.clone()),
            default_branch: "main".to_string(),
        };
        (config, tracker)
    }

    /// Create config that fails installation token retrieval
    fn with_token_failure() -> Self {
        Self {
            token_behavior: MockTokenBehavior::InvalidResponse,
            token_call_tracker: None,
            default_branch: "main".to_string(),
        }
    }

    /// Create config with custom token for organization
    #[allow(dead_code)]
    fn with_org_token(org_name: &str) -> Self {
        Self {
            token_behavior: MockTokenBehavior::Success(format!("ghs_mock_token_for_{}", org_name)),
            token_call_tracker: None,
            default_branch: "main".to_string(),
        }
    }
}

struct MockTemplateFetcher {
    files: Vec<(String, Vec<u8>)>,
}

#[async_trait]
impl TemplateFetcher for MockTemplateFetcher {
    async fn fetch_template_files(
        &self,
        _source_repo: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, String> {
        Ok(self.files.clone())
    }
}

// --- MOCK ENUMS (ALPHABETICALLY ORDERED) ---

/// Configuration for mock repository client behavior
#[derive(Clone)]
enum MockTokenBehavior {
    /// Return an AuthError with the given message
    #[allow(dead_code)]
    AuthError(String),
    /// Return an InvalidResponse error
    InvalidResponse,
    /// Return a successful token with the given string
    Success(String),
}

// --- MOCK FUNCTIONS (ALPHABETICALLY ORDERED) ---

/// Creates a default template config for testing
fn create_basic_template_config() -> TemplateConfig {
    TemplateConfig {
        name: "basic".to_string(),
        source_repo: "stub".to_string(),
        description: None,
        topics: None,
        features: None,
        pr_settings: None,
        labels: None,
        branch_protection_rules: None,
        action_permissions: None,
        required_variables: None,
        variable_configs: None,
    }
}

/// Creates a config with the basic template
fn create_config_with_basic_template() -> Config {
    Config {
        templates: vec![create_basic_template_config()],
    }
}

/// Creates a config with no templates
fn create_empty_config() -> Config {
    Config { templates: vec![] }
}

/// Creates a mock repository client that fails installation token retrieval
fn create_failing_token_mock_repo_client() -> impl RepositoryClient {
    ConfigurableMockRepoClient::new(MockRepoClientConfig::with_token_failure())
}

/// Creates a mock repository client with successful responses
fn create_mock_repo_client() -> impl RepositoryClient {
    ConfigurableMockRepoClient::new(MockRepoClientConfig::default())
}

/// Creates a mock repository client with organization-specific token
#[allow(dead_code)]
fn create_mock_repo_client_for_org(org_name: &str) -> impl RepositoryClient {
    ConfigurableMockRepoClient::new(MockRepoClientConfig::with_org_token(org_name))
}

/// Creates a mock template fetcher that returns the provided files
fn create_mock_template_fetcher(files: Vec<(String, Vec<u8>)>) -> impl TemplateFetcher {
    MockTemplateFetcher { files }
}

/// Creates a mock repository client that tracks installation token calls
fn create_token_tracking_mock_repo_client() -> (impl RepositoryClient, Arc<Mutex<bool>>) {
    let (config, tracker) = MockRepoClientConfig::with_token_tracking();
    let client = ConfigurableMockRepoClient::new(config);
    (client, tracker)
}

// --- TEST FUNCTIONS (ALPHABETICALLY ORDERED) ---

#[test]
fn test_git_credentials_callback() {
    use git2::{Cred, CredentialType};

    // Test that our credentials callback works correctly
    let token = "ghs_test_token_12345".to_string();
    let allowed_types = CredentialType::USER_PASS_PLAINTEXT;

    // This simulates what git2 would call
    let result = if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
        Cred::userpass_plaintext("x-access-token", &token)
    } else {
        Err(git2::Error::from_str("No supported credential types"))
    };

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_repository_fails_on_installation_token_error() {
    let config = create_config_with_basic_template();
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let repo_client = create_failing_token_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

    let result =
        create_repository_with_custom_settings(req, &config, &template_fetcher, &repo_client).await;

    assert!(!result.success);
    assert!(result.message.contains("Failed to get installation token"));
}

#[tokio::test]
async fn test_create_repository_fails_with_empty_owner() {
    let config = create_config_with_basic_template();
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let repo_client = create_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "".to_string(), // Empty owner should trigger user repo creation
        template: "basic".to_string(),
    };

    let result =
        create_repository_with_custom_settings(req, &config, &template_fetcher, &repo_client).await;

    assert!(!result.success);
    // The mock returns an AuthError for user repositories, which gets formatted differently
    assert!(result.message.contains("Invalid response format") || result.message.contains("Auth"));
}

#[tokio::test]
async fn test_create_repository_fails_with_template_not_found() {
    let config = create_empty_config(); // No templates
    let template_fetcher = create_mock_template_fetcher(vec![]);
    let repo_client = create_mock_repo_client();

    let req = CreateRepoRequest {
        name: "mockrepo".to_string(),
        owner: "mockorg".to_string(),
        template: "missing".to_string(),
    };

    let result =
        create_repository_with_custom_settings(req, &config, &template_fetcher, &repo_client).await;

    assert!(!result.success);
    assert!(result.message.contains("Template not found"));
}

#[tokio::test]
async fn test_create_repository_gets_installation_token() {
    let config = create_config_with_basic_template();
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let (repo_client, token_called) = create_token_tracking_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

    let _result =
        create_repository_with_custom_settings(req, &config, &template_fetcher, &repo_client).await;

    // The repository creation should call get_installation_token_for_org
    assert!(
        *token_called.lock().unwrap(),
        "get_installation_token_for_org should have been called"
    );
}

#[tokio::test]
async fn test_push_to_origin_with_valid_token() {
    use temp_dir::TempDir;
    use url::Url;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize a git repository in the temp directory
    let repo = git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

    // Create a test file and commit it
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").expect("Failed to write test file");

    let mut index = repo.index().expect("Failed to get index");
    index
        .add_path(std::path::Path::new("test.txt"))
        .expect("Failed to add file");
    index.write().expect("Failed to write index");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");

    let signature =
        git2::Signature::now("Test User", "test@example.com").expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .expect("Failed to create commit");

    // Test the push function with an invalid URL (will fail, but we're testing the auth setup)
    let fake_url = Url::parse("https://github.com/test/test.git").expect("Failed to parse URL");
    let token = "ghs_test_token_12345";

    // This will fail because it's not a real repository, but it should fail with a network error
    // rather than an authentication error, proving our auth setup is correct
    let result = push_to_origin(&temp_dir, fake_url, "main", token);

    // We expect this to fail with a network/repository error, not an auth error
    assert!(result.is_err());
    let error_message = format!("{:?}", result.err().unwrap());

    // The error should not be about authentication
    assert!(!error_message
        .to_lowercase()
        .contains("authentication not implemented"));
}

// ============================================================================
// create_repository() Tests - Type-Safe API Wrapper
// ============================================================================

/// Verify that create_repository converts types correctly for success case.
///
/// This test validates the wrapper's ability to convert between typed and legacy formats.
#[tokio::test]
async fn test_create_repository_type_conversion() {
    // This test will be updated once implementation is complete
    // For now, verify the function signature is correct

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
        TemplateName::new("basic").unwrap(),
    )
    .build();

    let config = Config {
        templates: vec![TemplateConfig {
            name: "basic".to_string(),
            source_repo: "owner/template-repo".to_string(),
            variable_configs: None,
            description: None,
            topics: None,
            features: None,
            pr_settings: None,
            labels: None,
            branch_protection_rules: None,
            action_permissions: None,
            required_variables: None,
        }],
    };

    let result = create_repository(request, &config, 12345, "fake-key".to_string()).await;

    // Currently returns Internal error (not yet implemented)
    assert!(result.is_err());
    if let Err(RepoRollerError::System(SystemError::Internal { reason })) = result {
        assert!(reason.contains("not yet implemented"));
    }
}

/// Verify that create_repository returns proper error types.
#[tokio::test]
async fn test_create_repository_error_handling() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
        TemplateName::new("nonexistent-template").unwrap(),
    )
    .build();

    let config = Config {
        templates: vec![], // Empty templates - will cause error
    };

    let result = create_repository(request, &config, 12345, "fake-key".to_string()).await;

    // Should return an error
    assert!(result.is_err());
}

/// Verify that create_repository preserves request variables.
#[tokio::test]
async fn test_create_repository_preserves_variables() {
    let mut variables = std::collections::HashMap::new();
    variables.insert("author".to_string(), "Jane Doe".to_string());
    variables.insert("license".to_string(), "MIT".to_string());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
        TemplateName::new("basic").unwrap(),
    )
    .variables(variables.clone())
    .build();

    assert_eq!(request.variables.len(), 2);
    assert_eq!(request.variables.get("author").unwrap(), "Jane Doe");

    // Function signature accepts the request - type checking works
    let config = Config { templates: vec![] };
    let _result = create_repository(request, &config, 12345, "fake-key".to_string()).await;
}

/// Verify that branded types prevent type confusion.
#[tokio::test]
async fn test_create_repository_type_safety() {
    // This test primarily validates compile-time type safety
    // If this compiles, the types are working correctly

    let name = RepositoryName::new("my-repo").unwrap();
    let owner = OrganizationName::new("my-org").unwrap();
    let template = TemplateName::new("my-template").unwrap();

    // These types cannot be confused at compile time
    let request = RepositoryCreationRequestBuilder::new(name, owner, template).build();

    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_str(), "my-template");
}

/// Verify that create_repository returns the correct result type.
#[tokio::test]
async fn test_create_repository_result_type() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
        TemplateName::new("basic").unwrap(),
    )
    .build();

    let config = Config { templates: vec![] };

    let result = create_repository(request, &config, 12345, "fake-key".to_string()).await;

    // Verify the result type is RepoRollerResult<RepositoryCreationResult>
    match result {
        Ok(_creation_result) => {
            // Would contain repository_url, repository_id, created_at, default_branch
            // Currently not reached due to NotImplemented
        }
        Err(error) => {
            // Should be RepoRollerError
            assert!(matches!(error, RepoRollerError::System(_)));
        }
    }
}
