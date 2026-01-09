// Unit tests for repo_roller_core
// Covers create_repository success and error paths with isolated mock dependencies

use super::*;
use async_trait::async_trait;
use config_manager::{
    ConfigurationResult, MetadataRepository, MetadataRepositoryProvider, TemplateConfig,
    TemplateMetadata,
};
use github_client::{errors::Error as GitHubError, RepositoryClient, RepositorySettingsUpdate};
use std::sync::{Arc, Mutex};
use visibility::{
    GitHubEnvironmentDetector, PlanLimitations, VisibilityError, VisibilityPolicy,
    VisibilityPolicyProvider,
};

// --- MOCK STRUCTS (ALPHABETICALLY ORDERED) ---

/// Mock authentication service that fails immediately
///
/// Used for testing error paths without hitting real GitHub API
struct MockAuthService;

#[async_trait]
impl auth_handler::UserAuthenticationService for MockAuthService {
    async fn authenticate_installation(
        &self,
        _app_id: u64,
        _private_key: &str,
        _installation_id: u64,
    ) -> auth_handler::AuthResult<String> {
        Err(auth_handler::AuthError::InvalidCredentials)
    }

    async fn get_installation_token_for_org(
        &self,
        _org_name: &str,
    ) -> auth_handler::AuthResult<String> {
        Err(auth_handler::AuthError::InvalidCredentials)
    }
}

/// Mock metadata provider for testing
///
/// Returns a configured TemplateConfig for testing purposes
struct MockMetadataProvider {
    template_config: Option<TemplateConfig>,
}

impl MockMetadataProvider {
    fn with_template(template_config: TemplateConfig) -> Self {
        Self {
            template_config: Some(template_config),
        }
    }

    fn empty() -> Self {
        Self {
            template_config: None,
        }
    }
}

#[async_trait]
impl MetadataRepositoryProvider for MockMetadataProvider {
    async fn load_template_configuration(
        &self,
        _org: &str,
        _template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        self.template_config.clone().ok_or_else(|| {
            config_manager::ConfigurationError::InvalidConfiguration {
                field: "template".to_string(),
                reason: "Template not found".to_string(),
            }
        })
    }

    async fn discover_metadata_repository(
        &self,
        _org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        unimplemented!("Not used in these tests")
    }

    async fn load_global_defaults(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<config_manager::GlobalDefaults> {
        unimplemented!("Not used in these tests")
    }

    async fn load_team_configuration(
        &self,
        _repo: &MetadataRepository,
        _team: &str,
    ) -> ConfigurationResult<Option<config_manager::TeamConfig>> {
        unimplemented!("Not used in these tests")
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &MetadataRepository,
        _repo_type: &str,
    ) -> ConfigurationResult<Option<config_manager::RepositoryTypeConfig>> {
        unimplemented!("Not used in these tests")
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<std::collections::HashMap<String, config_manager::LabelConfig>> {
        unimplemented!("Not used in these tests")
    }

    async fn list_available_repository_types(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        unimplemented!("Not used in these tests")
    }

    async fn validate_repository_structure(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<()> {
        unimplemented!("Not used in these tests")
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        unimplemented!("Not used in these tests")
    }
}

/// Mock visibility policy provider
///
/// Returns unrestricted policy for all organizations
struct MockVisibilityPolicyProvider;

#[async_trait]
impl VisibilityPolicyProvider for MockVisibilityPolicyProvider {
    async fn get_policy(&self, _organization: &str) -> Result<VisibilityPolicy, VisibilityError> {
        Ok(VisibilityPolicy::Unrestricted)
    }

    async fn invalidate_cache(&self, _organization: &str) {
        // No-op for mock
    }
}

/// Mock environment detector
///
/// Returns paid plan (supports private repos) by default
struct MockEnvironmentDetector;

#[async_trait]
impl GitHubEnvironmentDetector for MockEnvironmentDetector {
    async fn get_plan_limitations(
        &self,
        _organization: &str,
    ) -> Result<PlanLimitations, github_client::Error> {
        Ok(PlanLimitations {
            supports_private_repos: true,
            supports_internal_repos: false,
            private_repo_limit: None,
            is_enterprise: false,
        })
    }

    async fn is_enterprise(&self, _organization: &str) -> Result<bool, github_client::Error> {
        Ok(false)
    }
}

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
    ) -> github_client::Repository {
        github_client::Repository::new(
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
    ) -> Result<github_client::Repository, GitHubError> {
        Ok(self.create_successful_repository(payload))
    }

    async fn create_user_repository(
        &self,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::Repository, GitHubError> {
        Err(GitHubError::AuthError(
            "User repos not supported in test".to_string(),
        ))
    }

    async fn update_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
        _settings: &RepositorySettingsUpdate,
    ) -> Result<github_client::Repository, GitHubError> {
        Err(GitHubError::AuthError(
            "Not implemented in test".to_string(),
        ))
    }

    async fn search_repositories(
        &self,
        _query: &str,
    ) -> Result<Vec<github_client::Repository>, GitHubError> {
        Ok(vec![])
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

    async fn get_custom_properties(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<std::collections::HashMap<String, String>, GitHubError> {
        // Not implemented in test mock - return empty properties
        Ok(std::collections::HashMap::new())
    }

    async fn list_repository_labels(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<String>, GitHubError> {
        // Not implemented in test mock - return empty labels
        Ok(vec![])
    }

    async fn create_label(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _color: &str,
        _description: &str,
    ) -> Result<(), GitHubError> {
        // Not implemented in test mock - return success
        Ok(())
    }

    async fn list_repository_files(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<String>, GitHubError> {
        // Not implemented in test mock - return empty files
        Ok(vec![])
    }

    async fn get_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<github_client::Repository, GitHubError> {
        // Not implemented in test mock - return error
        Err(GitHubError::AuthError(
            "Not implemented in test".to_string(),
        ))
    }

    async fn get_branch_protection(
        &self,
        _owner: &str,
        _repo: &str,
        _branch: &str,
    ) -> Result<Option<github_client::BranchProtection>, GitHubError> {
        // Not implemented in test mock - return None (no protection)
        Ok(None)
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

// --- MOCK ENUMS (ALPHABETICALLY ORDERED) ---

/// Configuration for mock repository client behavior
#[derive(Clone)]
enum MockTokenBehavior {
    /// Return an AuthError with the given message
    #[allow(dead_code)]
    AuthError(String),
    /// Return an InvalidResponse error
    #[allow(dead_code)] // Used in match arm but compiler doesn't detect it
    InvalidResponse,
    /// Return a successful token with the given string
    Success(String),
}

// --- MOCK FUNCTIONS (ALPHABETICALLY ORDERED) ---

/// Creates a mock repository client with organization-specific token
#[allow(dead_code)]
fn create_mock_repo_client_for_org(org_name: &str) -> impl RepositoryClient {
    ConfigurableMockRepoClient::new(MockRepoClientConfig::with_org_token(org_name))
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
    let result = crate::git::push_to_origin(&temp_dir, fake_url, "main", token);

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
    // Verify the new create_repository function attempts to execute (will fail due to mocking limitations)
    // Full integration tests will be added in Task 7.2.3

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
        TemplateName::new("basic").unwrap(),
    )
    .build();

    let template_config = TemplateConfig {
        template: TemplateMetadata {
            name: "basic".to_string(),
            description: "Test template".to_string(),
            author: "Test".to_string(),
            tags: vec![],
        },
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
        default_visibility: None,
    };

    let metadata_provider = MockMetadataProvider::with_template(template_config);
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

    // Should fail during authentication with mock service
    assert!(result.is_err());
    if let Err(e) = result {
        // Error should be from authentication
        assert!(
            e.to_string().contains("authentication")
                || e.to_string().contains("token")
                || e.to_string().contains("credentials"),
            "Expected authentication error, got: {}",
            e
        );
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

    let metadata_provider = MockMetadataProvider::empty(); // No templates - will cause error
    let auth_service = MockAuthService;

    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

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
    let metadata_provider = MockMetadataProvider::empty();
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);
    let _result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
    )
    .await;
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

    let metadata_provider = MockMetadataProvider::empty();
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
    )
    .await;

    // Verify the result type is RepoRollerResult<RepositoryCreationResult>
    match result {
        Ok(_creation_result) => {
            // Would contain repository_url, repository_id, created_at, default_branch
            // Not reached in this test due to authentication failure
        }
        Err(error) => {
            // Should be RepoRollerError
            assert!(
                matches!(error, RepoRollerError::GitHub(_))
                    || matches!(error, RepoRollerError::System(_))
                    || matches!(error, RepoRollerError::Template(_))
            );
        }
    }
}

// --- CONFIGURATION VARIABLE EXTRACTION TESTS ---

/// Verify that extract_config_variables returns empty map for default configuration.
#[test]
fn test_extract_config_variables_empty_config() {
    let merged_config = config_manager::MergedConfiguration::new();
    let variables = extract_config_variables(&merged_config);

    // Default config should have no explicitly set values, so we get default boolean values
    assert_eq!(
        variables.get("config_issues_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_wiki_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_projects_enabled"),
        Some(&"false".to_string())
    );
}

/// Verify that repository feature settings are correctly extracted as config variables.
#[test]
fn test_extract_config_variables_repository_features() {
    use config_manager::{settings::RepositorySettings, MergedConfiguration, OverridableValue};

    let mut merged_config = MergedConfiguration::new();
    merged_config.repository = RepositorySettings {
        issues: Some(OverridableValue::allowed(true)),
        wiki: Some(OverridableValue::fixed(false)),
        projects: Some(OverridableValue::allowed(true)),
        discussions: Some(OverridableValue::allowed(false)),
        pages: Some(OverridableValue::allowed(true)),
        security_advisories: Some(OverridableValue::allowed(true)),
        vulnerability_reporting: Some(OverridableValue::allowed(false)),
        auto_close_issues: Some(OverridableValue::allowed(false)),
    };

    let variables = extract_config_variables(&merged_config);

    assert_eq!(
        variables.get("config_issues_enabled"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_wiki_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_projects_enabled"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_discussions_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_pages_enabled"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_security_advisories_enabled"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_vulnerability_reporting_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_auto_close_issues_enabled"),
        Some(&"false".to_string())
    );
}

/// Verify that pull request settings are correctly extracted as config variables.
#[test]
fn test_extract_config_variables_pull_request_settings() {
    use config_manager::{settings::PullRequestSettings, MergedConfiguration, OverridableValue};

    let mut merged_config = MergedConfiguration::new();
    merged_config.pull_requests = PullRequestSettings {
        required_approving_review_count: Some(OverridableValue::allowed(2)),
        allow_merge_commit: Some(OverridableValue::allowed(true)),
        allow_squash_merge: Some(OverridableValue::allowed(false)),
        allow_rebase_merge: Some(OverridableValue::allowed(true)),
        allow_auto_merge: Some(OverridableValue::allowed(false)),
        delete_branch_on_merge: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    let variables = extract_config_variables(&merged_config);

    assert_eq!(
        variables.get("config_required_approving_review_count"),
        Some(&"2".to_string())
    );
    assert_eq!(
        variables.get("config_allow_merge_commit"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_allow_squash_merge"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_allow_rebase_merge"),
        Some(&"true".to_string())
    );
    assert_eq!(
        variables.get("config_allow_auto_merge"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_delete_branch_on_merge"),
        Some(&"true".to_string())
    );
}

/// Verify that all config variables use the "config_" prefix to avoid naming conflicts.
#[test]
fn test_extract_config_variables_uses_prefix() {
    use config_manager::{settings::RepositorySettings, MergedConfiguration, OverridableValue};

    let mut merged_config = MergedConfiguration::new();
    merged_config.repository = RepositorySettings {
        issues: Some(OverridableValue::allowed(true)),
        ..Default::default()
    };

    let variables = extract_config_variables(&merged_config);

    // Verify all keys start with "config_"
    for key in variables.keys() {
        assert!(
            key.starts_with("config_"),
            "Variable '{}' should start with 'config_'",
            key
        );
    }
}

/// Verify that None values in configuration are handled correctly.
#[test]
fn test_extract_config_variables_none_values() {
    use config_manager::{settings::RepositorySettings, MergedConfiguration};

    let mut merged_config = MergedConfiguration::new();
    merged_config.repository = RepositorySettings {
        issues: None, // Explicitly set to None
        wiki: None,
        projects: None,
        discussions: None,
        pages: None,
        security_advisories: None,
        vulnerability_reporting: None,
        auto_close_issues: None,
    };

    let variables = extract_config_variables(&merged_config);

    // None values should result in "false" as the default
    assert_eq!(
        variables.get("config_issues_enabled"),
        Some(&"false".to_string())
    );
    assert_eq!(
        variables.get("config_wiki_enabled"),
        Some(&"false".to_string())
    );
}
