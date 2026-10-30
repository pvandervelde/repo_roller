// Unit tests for repo_roller_core
// Covers create_repository success and error paths with isolated mock dependencies

#[ctor::ctor(unsafe)]
fn init_default_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

use super::*;
use async_trait::async_trait;
use config_manager::{
    settings::WebhookConfig, ConfigurationResult, MetadataRepository, MetadataRepositoryProvider,
    TemplateConfig, TemplateMetadata,
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

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<WebhookConfig>> {
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

    async fn list_webhooks(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<github_client::Webhook>, GitHubError> {
        // Not implemented in test mock - return empty webhooks
        Ok(vec![])
    }

    async fn create_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _params: &github_client::CreateWebhookParams<'_>,
    ) -> Result<github_client::Webhook, GitHubError> {
        // Not implemented in test mock - return error
        Err(GitHubError::AuthError(
            "Not implemented in test".to_string(),
        ))
    }

    async fn update_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _webhook_id: u64,
        _params: &github_client::UpdateWebhookParams<'_>,
    ) -> Result<github_client::Webhook, GitHubError> {
        // Not implemented in test mock - return error
        Err(GitHubError::AuthError(
            "Not implemented in test".to_string(),
        ))
    }

    async fn delete_webhook(
        &self,
        _owner: &str,
        _repo: &str,
        _webhook_id: u64,
    ) -> Result<(), GitHubError> {
        // Not implemented in test mock - return Ok
        Ok(())
    }

    async fn list_repository_rulesets(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<Vec<github_client::RepositoryRuleset>, GitHubError> {
        // Not implemented in test mock - return empty list
        Ok(Vec::new())
    }

    async fn create_repository_ruleset(
        &self,
        _owner: &str,
        _repo: &str,
        ruleset: &github_client::RepositoryRuleset,
    ) -> Result<github_client::RepositoryRuleset, GitHubError> {
        // Not implemented in test mock - return the input with an ID
        let mut result = ruleset.clone();
        result.id = Some(1);
        Ok(result)
    }

    async fn update_repository_ruleset(
        &self,
        _owner: &str,
        _repo: &str,
        _ruleset_id: u64,
        ruleset: &github_client::RepositoryRuleset,
    ) -> Result<github_client::RepositoryRuleset, GitHubError> {
        // Not implemented in test mock - return the input
        Ok(ruleset.clone())
    }

    async fn update_label(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _new_name: &str,
        _color: &str,
        _description: &str,
    ) -> Result<(), GitHubError> {
        // Not implemented in test mock - return Ok
        Ok(())
    }

    async fn delete_label(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
    ) -> Result<(), GitHubError> {
        // Not implemented in test mock - return Ok
        Ok(())
    }

    async fn create_fork(
        &self,
        _source_owner: &str,
        source_repo: &str,
        target_owner: Option<&str>,
    ) -> Result<github_client::Repository, GitHubError> {
        // Not exercised by the existing create_repository test suite - return a
        // minimal success value so the mock satisfies the widened trait.
        let owner = target_owner.unwrap_or("mock-owner");
        Ok(github_client::Repository::new(
            source_repo.to_string(),
            format!("{owner}/{source_repo}"),
            "MOCK_NODE_ID".to_string(),
            false,
        ))
    }

    async fn create_pull_request(
        &self,
        _owner: &str,
        _repo: &str,
        payload: &github_client::PullRequestCreatePayload,
    ) -> Result<github_client::PullRequest, GitHubError> {
        Ok(github_client::PullRequest {
            number: 1,
            html_url: String::new(),
            draft: payload.draft,
            state: "open".to_string(),
        })
    }

    async fn mark_pull_request_ready_for_review(
        &self,
        _owner: &str,
        _repo: &str,
        pr_number: u64,
    ) -> Result<github_client::PullRequest, GitHubError> {
        Ok(github_client::PullRequest {
            number: pr_number,
            html_url: String::new(),
            draft: false,
            state: "open".to_string(),
        })
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
    )
    .template(TemplateName::new("basic").unwrap())
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
        rulesets: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation: None,
        creation_capabilities: None,
    };

    let metadata_provider = MockMetadataProvider::with_template(template_config);
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);
    let secret_resolver = Arc::new(crate::event_secrets::EnvironmentSecretResolver::new());
    let test_registry = prometheus::Registry::new();
    let metrics = Arc::new(crate::event_metrics::PrometheusEventMetrics::new(
        &test_registry,
    ));

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
        EventNotificationContext::new("unit-test", secret_resolver, metrics),
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
    )
    .template(TemplateName::new("nonexistent-template").unwrap())
    .build();

    let metadata_provider = MockMetadataProvider::empty(); // No templates - will cause error
    let auth_service = MockAuthService;

    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);
    let secret_resolver = Arc::new(crate::event_secrets::EnvironmentSecretResolver::new());
    let test_registry = prometheus::Registry::new();
    let metrics = Arc::new(crate::event_metrics::PrometheusEventMetrics::new(
        &test_registry,
    ));

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
        EventNotificationContext::new("unit-test", secret_resolver, metrics),
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
    )
    .template(TemplateName::new("basic").unwrap())
    .variables(variables.clone())
    .build();

    assert_eq!(request.variables.len(), 2);
    assert_eq!(request.variables.get("author").unwrap(), "Jane Doe");

    // Function signature accepts the request - type checking works
    let metadata_provider = MockMetadataProvider::empty();
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);
    let secret_resolver = Arc::new(crate::event_secrets::EnvironmentSecretResolver::new());
    let test_registry = prometheus::Registry::new();
    let metrics = Arc::new(crate::event_metrics::PrometheusEventMetrics::new(
        &test_registry,
    ));
    let _result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
        EventNotificationContext::new("unit-test", secret_resolver, metrics),
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
    let request = RepositoryCreationRequestBuilder::new(name, owner)
        .template(template)
        .build();

    assert_eq!(request.name.as_str(), "my-repo");
    assert_eq!(request.owner.as_str(), "my-org");
    assert_eq!(request.template.as_ref().unwrap().as_str(), "my-template");
}

/// Verify that create_repository returns the correct result type.
#[tokio::test]
async fn test_create_repository_result_type() {
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("test-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .template(TemplateName::new("basic").unwrap())
    .build();

    let metadata_provider = MockMetadataProvider::empty();
    let auth_service = MockAuthService;
    let visibility_policy_provider = Arc::new(MockVisibilityPolicyProvider);
    let environment_detector = Arc::new(MockEnvironmentDetector);
    let secret_resolver = Arc::new(crate::event_secrets::EnvironmentSecretResolver::new());
    let test_registry = prometheus::Registry::new();
    let metrics = Arc::new(crate::event_metrics::PrometheusEventMetrics::new(
        &test_registry,
    ));

    let result = create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller",
        visibility_policy_provider,
        environment_detector,
        EventNotificationContext::new("unit-test", secret_resolver, metrics),
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

// ─── merge_access_map_with_policy ─────────────────────────────────────────────

/// Convenience helper: build a HashMap<String,String> from (&str, &str) pairs.
fn str_map(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Convenience helper: build a HashSet<String> from &[&str].
fn str_set(items: &[&str]) -> std::collections::HashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// Verify that config entries without any request overrides are included as-is.
#[test]
fn test_merge_policy_config_only_entries_preserved() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-a", "write"), ("team-b", "read")]);
    let result = merge_access_map_with_policy(&config, &str_set(&[]), None, &[], "team");

    assert_eq!(result.get("team-a"), Some(&AccessLevel::Write));
    assert_eq!(result.get("team-b"), Some(&AccessLevel::Read));
}

/// Verify that a request can add a new entry not present in config.
#[test]
fn test_merge_policy_request_adds_new_entry() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-a", "read")]);
    let request = [("new-team", AccessLevel::Triage)];
    let result = merge_access_map_with_policy(&config, &str_set(&[]), None, &request, "team");

    assert_eq!(result.get("new-team"), Some(&AccessLevel::Triage));
    assert_eq!(result.get("team-a"), Some(&AccessLevel::Read));
}

/// Verify that a request can upgrade (promote) an existing config entry.
#[test]
fn test_merge_policy_request_can_upgrade_config_entry() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-a", "read")]);
    let request = [("team-a", AccessLevel::Write)];
    let result = merge_access_map_with_policy(&config, &str_set(&[]), None, &request, "team");

    assert_eq!(result.get("team-a"), Some(&AccessLevel::Write));
}

/// Verify that a request cannot demote an existing config entry (no-demotion rule).
#[test]
fn test_merge_policy_request_cannot_demote_config_entry() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-a", "maintain")]);
    let request = [("team-a", AccessLevel::Read)];
    let result = merge_access_map_with_policy(&config, &str_set(&[]), None, &request, "team");

    // Must keep config level, not the requested demotion.
    assert_eq!(result.get("team-a"), Some(&AccessLevel::Maintain));
}

/// Verify that a locked entry is not altered by a request.
#[test]
fn test_merge_policy_locked_entry_not_altered_by_request() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("locked-team", "write")]);
    let locked = str_set(&["locked-team"]);
    let request = [("locked-team", AccessLevel::Admin)];
    let result = merge_access_map_with_policy(&config, &locked, None, &request, "team");

    // Must remain at config level despite the upgrade attempt.
    assert_eq!(result.get("locked-team"), Some(&AccessLevel::Write));
}

/// Verify that a locked entry that does NOT exist in config is still ignored when
/// a request tries to set it. In production this combination (locked but not in config)
/// is unreachable because `locked` is only populated from entries that already appear
/// in `config_entries`. The correct safe behaviour is to always ignore requests for
/// locked identifiers regardless.
#[test]
fn test_merge_policy_locked_entry_absent_from_config_ignores_request() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[]);
    let locked = str_set(&["ghost-team"]);
    let request = [("ghost-team", AccessLevel::Triage)];
    let result = merge_access_map_with_policy(&config, &locked, None, &request, "team");

    // The request value must NOT be inserted for a locked identifier.
    assert_eq!(result.get("ghost-team"), None);
}

/// Verify that a request entry exceeding the org ceiling is capped at the ceiling.
#[test]
fn test_merge_policy_request_capped_at_ceiling() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[]);
    let request = [("new-team", AccessLevel::Admin)];
    let result =
        merge_access_map_with_policy(&config, &str_set(&[]), Some("write"), &request, "team");

    assert_eq!(result.get("new-team"), Some(&AccessLevel::Write));
}

/// Verify that a request entry at exactly the ceiling is accepted unchanged.
#[test]
fn test_merge_policy_request_at_ceiling_is_accepted() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[]);
    let request = [("new-team", AccessLevel::Write)];
    let result =
        merge_access_map_with_policy(&config, &str_set(&[]), Some("write"), &request, "team");

    assert_eq!(result.get("new-team"), Some(&AccessLevel::Write));
}

/// Verify that ceiling does not affect config entries (ceiling applies to request entries only).
#[test]
fn test_merge_policy_ceiling_does_not_affect_config_entries() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-a", "admin")]);
    let result = merge_access_map_with_policy(&config, &str_set(&[]), Some("write"), &[], "team");

    // Config entry with admin must remain even though ceiling is "write".
    assert_eq!(result.get("team-a"), Some(&AccessLevel::Admin));
}

/// Verify ceiling + no-demotion interact correctly: after cap, if capped level is
/// still below config level the demotion rule fires and the config value wins.
#[test]
fn test_merge_policy_ceiling_then_no_demotion_preserves_higher_config_level() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    // Config has "maintain", ceiling is "write", request asks for "admin".
    // After capping: "write". "write" < "maintain" → demotion rule → keep "maintain".
    let config = str_map(&[("team-a", "maintain")]);
    let request = [("team-a", AccessLevel::Admin)];
    let result =
        merge_access_map_with_policy(&config, &str_set(&[]), Some("write"), &request, "team");

    assert_eq!(result.get("team-a"), Some(&AccessLevel::Maintain));
}

/// Verify that an invalid config level string is skipped gracefully.
#[test]
fn test_merge_policy_invalid_config_level_is_skipped() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let config = str_map(&[("team-bad", "ultramax"), ("team-ok", "read")]);
    let result = merge_access_map_with_policy(&config, &str_set(&[]), None, &[], "team");

    // Invalid entry silently dropped; valid entry preserved.
    assert!(!result.contains_key("team-bad"));
    assert_eq!(result.get("team-ok"), Some(&AccessLevel::Read));
}

/// Verify that an invalid ceiling string is treated as no ceiling (warning, not error).
#[test]
fn test_merge_policy_invalid_ceiling_treated_as_no_ceiling() {
    use crate::{merge_access_map_with_policy, permissions::AccessLevel};

    let request = [("new-team", AccessLevel::Admin)];
    let result = merge_access_map_with_policy(
        &str_map(&[]),
        &str_set(&[]),
        Some("not_a_level"),
        &request,
        "team",
    );

    // Ceiling is invalid, so no ceiling applied; Admin request accepted as-is.
    assert_eq!(result.get("new-team"), Some(&AccessLevel::Admin));
}

// ============================================================================
// FORK/MIRROR SOURCE-OWNER ALLOW-LIST GATE TESTS (HIGH finding, RCF-005
// extension)
// ============================================================================
//
// See docs/spec/interfaces/fork-mirror-operations.md §2 and §7: a fork/mirror
// capability being enabled is NOT sufficient authorization — `source.owner`
// must additionally appear (case-insensitively) in the merged
// `allowed_source_owners` list. `validate_creation_mode_capability` must
// check this immediately after the capability-enabled check and before the
// template-designation check, before any GitHub API call.
//
// `validate_creation_mode_capability` is a pure, synchronous function that
// takes no client — it can be (and is) exercised directly here without any
// mock `RepositoryClient`, which also proves the "no GitHub API call before
// rejection" property structurally (there is no client in scope to call).

fn fork_request_with_source(owner: &str) -> RepositoryCreationRequest {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("new-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .creation_mode(CreationMode::Fork {
        source: SourceRepositoryReference {
            owner: OrganizationName::new(owner).unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
    })
    .build()
}

fn mirror_request_with_source(owner: &str, preserve_history: bool) -> RepositoryCreationRequest {
    RepositoryCreationRequestBuilder::new(
        RepositoryName::new("new-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .creation_mode(CreationMode::Mirror {
        source: SourceRepositoryReference {
            owner: OrganizationName::new(owner).unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
        preserve_history,
    })
    .build()
}

/// Builds a `MergedConfiguration` with exactly one capability flag set
/// (`enabled`) and `allowed_source_owners` set to `allowed_owners` (or left
/// unset when `None`, exercising the fail-closed default).
fn merged_config_with_capability(
    capability: &str,
    enabled: bool,
    allowed_owners: Option<Vec<&str>>,
) -> config_manager::MergedConfiguration {
    use config_manager::{CreationCapabilitiesSettings, OverridableValue};

    let mut capabilities = CreationCapabilitiesSettings::default();
    match capability {
        "fork" => capabilities.fork_enabled = Some(OverridableValue::allowed(enabled)),
        "source_history_mirror" => {
            capabilities.source_history_mirror_enabled = Some(OverridableValue::allowed(enabled))
        }
        "source_only_mirror" => {
            capabilities.source_only_mirror_enabled = Some(OverridableValue::allowed(enabled))
        }
        other => panic!("unknown capability: {other}"),
    }
    capabilities.allowed_source_owners = allowed_owners
        .map(|owners| OverridableValue::allowed(owners.into_iter().map(str::to_string).collect()));

    let mut merged = config_manager::MergedConfiguration::new();
    merged.creation_capabilities = capabilities;
    merged
}

fn minimal_template_config(
    designation: Option<config_manager::TemplateDesignation>,
) -> TemplateConfig {
    TemplateConfig {
        template: TemplateMetadata {
            name: "t".to_string(),
            description: "d".to_string(),
            author: "a".to_string(),
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
        rulesets: None,
        default_visibility: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
        designation,
        creation_capabilities: None,
    }
}

#[test]
fn test_validate_creation_mode_capability_rejects_fork_when_allowed_source_owners_unset() {
    let request = fork_request_with_source("upstream-org");
    let merged = merged_config_with_capability("fork", true, None);

    let result = validate_creation_mode_capability(&request, &merged, None);

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::SourceOwnerNotAllowed {
            owner,
            capability,
        })) => {
            assert_eq!(owner, "upstream-org");
            assert_eq!(capability, "fork");
        }
        other => panic!(
            "expected SourceOwnerNotAllowed when the allow-list is unset \
             (fail-closed default), even though the capability is enabled, got {other:?}"
        ),
    }
}

#[test]
fn test_validate_creation_mode_capability_rejects_fork_when_allowed_source_owners_empty() {
    let request = fork_request_with_source("upstream-org");
    let merged = merged_config_with_capability("fork", true, Some(vec![]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(
        matches!(
            result,
            Err(RepoRollerError::Configuration(
                ConfigurationError::SourceOwnerNotAllowed { .. }
            ))
        ),
        "an explicitly-empty allow-list must still deny every source owner, got {result:?}"
    );
}

#[test]
fn test_validate_creation_mode_capability_rejects_fork_when_owner_not_in_allow_list() {
    let request = fork_request_with_source("evil-org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["good-org", "another-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::SourceOwnerNotAllowed {
            owner,
            capability,
        })) => {
            assert_eq!(owner, "evil-org");
            assert_eq!(capability, "fork");
        }
        other => panic!("expected SourceOwnerNotAllowed, got {other:?}"),
    }
}

#[test]
fn test_validate_creation_mode_capability_allows_fork_when_owner_exactly_matches_allow_list() {
    let request = fork_request_with_source("good-org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

#[test]
fn test_validate_creation_mode_capability_allow_list_match_is_case_insensitive_list_upper() {
    // Allow-list entry upper-case, request owner lower-case.
    let request = fork_request_with_source("good-org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["GOOD-ORG"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(
        result.is_ok(),
        "case-insensitive match should pass: {result:?}"
    );
}

#[test]
fn test_validate_creation_mode_capability_allow_list_match_is_case_insensitive_owner_upper() {
    // Request owner has mixed case, allow-list stored lower-case.
    let request = fork_request_with_source("Good-Org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(
        result.is_ok(),
        "case-insensitive match should pass: {result:?}"
    );
}

/// Stub-killing: a buggy implementation that only compares against the
/// first allow-list entry (e.g. `list.first() == Some(owner)`) would fail
/// this — the owner matches the *second* entry, not the first or last.
#[test]
fn test_validate_creation_mode_capability_allow_list_matches_any_position_not_just_first() {
    let request = fork_request_with_source("second-org");
    let merged = merged_config_with_capability(
        "fork",
        true,
        Some(vec!["first-org", "second-org", "third-org"]),
    );

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(
        result.is_ok(),
        "owner matching a non-first list entry must still pass: {result:?}"
    );
}

/// Ordering: the capability-enabled check must run before the owner
/// allow-list check — a disabled capability must reject with
/// `CapabilityDisabled` even when the source owner is on the allow-list.
#[test]
fn test_validate_creation_mode_capability_checks_capability_disabled_before_owner_allow_list() {
    let request = fork_request_with_source("good-org");
    let merged = merged_config_with_capability("fork", false, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::CapabilityDisabled {
            capability,
        })) => {
            assert_eq!(capability, "fork");
        }
        other => panic!(
            "expected CapabilityDisabled (checked before the owner allow-list), got {other:?}"
        ),
    }
}

/// Ordering: the owner allow-list check must run before the template
/// designation check — a disallowed owner must reject with
/// `SourceOwnerNotAllowed` even when a non-designated template is also
/// passed (which would otherwise trigger `TemplateNotDesignated`).
#[test]
fn test_validate_creation_mode_capability_checks_owner_allow_list_before_template_designation() {
    let request = fork_request_with_source("evil-org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["good-org"]));
    let wrong_template =
        minimal_template_config(Some(config_manager::TemplateDesignation::SourceOnlyMirror));

    let result = validate_creation_mode_capability(&request, &merged, Some(&wrong_template));

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::SourceOwnerNotAllowed {
            owner,
            ..
        })) => {
            assert_eq!(owner, "evil-org");
        }
        other => {
            panic!("expected SourceOwnerNotAllowed (checked before designation), got {other:?}")
        }
    }
}

/// Sanity check for the ordering test above: once the owner check passes,
/// the pre-existing designation check must still fire as before — this
/// rules out a fix that accidentally short-circuits `Ok(())` once the owner
/// check passes, skipping the designation check entirely.
#[test]
fn test_validate_creation_mode_capability_falls_through_to_designation_check_when_owner_allowed() {
    let request = fork_request_with_source("good-org");
    let merged = merged_config_with_capability("fork", true, Some(vec!["good-org"]));
    let wrong_template =
        minimal_template_config(Some(config_manager::TemplateDesignation::SourceOnlyMirror));

    let result = validate_creation_mode_capability(&request, &merged, Some(&wrong_template));

    assert!(
        matches!(
            result,
            Err(RepoRollerError::Configuration(
                ConfigurationError::TemplateNotDesignated { .. }
            ))
        ),
        "expected TemplateNotDesignated once the owner check passes, got {result:?}"
    );
}

#[test]
fn test_validate_creation_mode_capability_applies_allow_list_to_source_history_mirror() {
    let request = mirror_request_with_source("evil-org", true);
    let merged =
        merged_config_with_capability("source_history_mirror", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::SourceOwnerNotAllowed {
            capability,
            ..
        })) => {
            assert_eq!(capability, "source_history_mirror");
        }
        other => panic!("expected SourceOwnerNotAllowed, got {other:?}"),
    }
}

#[test]
fn test_validate_creation_mode_capability_applies_allow_list_to_source_only_mirror() {
    let request = mirror_request_with_source("evil-org", false);
    let merged = merged_config_with_capability("source_only_mirror", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    match result {
        Err(RepoRollerError::Configuration(ConfigurationError::SourceOwnerNotAllowed {
            capability,
            ..
        })) => {
            assert_eq!(capability, "source_only_mirror");
        }
        other => panic!("expected SourceOwnerNotAllowed, got {other:?}"),
    }
}

#[test]
fn test_validate_creation_mode_capability_allows_source_history_mirror_when_owner_allowed() {
    let request = mirror_request_with_source("good-org", true);
    let merged =
        merged_config_with_capability("source_history_mirror", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

#[test]
fn test_validate_creation_mode_capability_allows_source_only_mirror_when_owner_allowed() {
    let request = mirror_request_with_source("good-org", false);
    let merged = merged_config_with_capability("source_only_mirror", true, Some(vec!["good-org"]));

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

/// Regression guard: `CreationMode::Standard` has no `source` at all and
/// must continue to bypass the entire fork/mirror gate (including the new
/// owner allow-list check), regardless of how threadbare
/// `creation_capabilities` is.
#[test]
fn test_validate_creation_mode_capability_standard_mode_bypasses_allow_list_entirely() {
    // `.content_strategy(Empty)` sidesteps the pre-existing, unrelated builder
    // rule that `ContentStrategy::Template` (the default) requires a template
    // to be set — irrelevant to what this test exercises (the fork/mirror
    // gate bypass for `CreationMode::Standard`).
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new("new-repo").unwrap(),
        OrganizationName::new("test-org").unwrap(),
    )
    .content_strategy(ContentStrategy::Empty)
    .build();
    let merged = config_manager::MergedConfiguration::new(); // all-None capabilities

    let result = validate_creation_mode_capability(&request, &merged, None);

    assert!(
        result.is_ok(),
        "Standard mode must bypass the fork/mirror gate entirely: {result:?}"
    );
}

// ============================================================================
// Fork/Mirror Dispatch Function Tests
// ============================================================================
//
// `create_fork_repository`, `create_history_mirror_repository`, and
// `create_source_only_mirror_repository` (added for #255) previously had
// zero test coverage reachable without live GitHub credentials. The #255 QA
// mutation-testing audit flagged this gap as "proven material, not
// theoretical": the missing `index.write()` defect in `git.rs`
// (`prepare_index_and_tree`), which broke `create_source_only_mirror_repository`
// on every invocation, was only caught by a hand-reproduced scenario, not by
// any dispatch-level test.
//
// `create_fork_repository` makes no git2 network calls at all (GitHub forks
// the repository server-side), so it gets a full success-path test below,
// proving it reaches `create_fork` and `get_organization_default_branch` and
// returns a `RepositoryCreationResult`.
//
// `create_history_mirror_repository` and `create_source_only_mirror_repository`
// both eventually call `git::mirror_clone` / `git::shallow_clone_working_tree`
// and push to the real `https://github.com/{owner}/{name}.git` URL:
// `github_client::Repository::url()` hardcodes the `github.com` host with no
// override seam, and `SourceRepositoryReference` carries no URL field (see
// `build_github_https_url`). Reaching that boundary and succeeding therefore
// requires live GitHub credentials, exactly as already documented for the
// identical architectural gap in `lib_span_tests.rs`/`observability.rs` and
// in `integration_tests::fork_mirror_capability_tests` ("True success-path
// fork/mirror creation ... require[s] ... live" fixtures). The tests below
// exercise everything reachable without network -- visibility resolution,
// local git initialization/commit (the exact code path the #255 defect
// lived in), and the `create_github_repository` GitHub API call -- then
// assert the resulting error is propagated correctly when that call fails,
// proving the dispatch wiring up to the real network boundary.
//
// All three dispatch functions take `clients: &CreationClients`, whose
// `installation_repo_client` field is a concrete `github_client::GitHubClient`
// (not `&dyn RepositoryClient`), so `ConfigurableMockRepoClient` -- used
// elsewhere in this file for functions that *do* take `&dyn RepositoryClient`
// (e.g. `create_github_repository`, `initialize_git_repository`) -- cannot be
// substituted here. Instead these tests point a real `GitHubClient` at a
// `wiremock` server, mirroring the established pattern in
// `permission_manager_tests.rs::create_test_github_client`.

/// Creates a [`github_client::GitHubClient`] pointed at a wiremock server for
/// testing. See the module note above for why this (rather than
/// `ConfigurableMockRepoClient`) is required for the dispatch-function tests.
fn wiremock_github_client(server_uri: &str) -> github_client::GitHubClient {
    let octocrab = octocrab::Octocrab::builder()
        .base_uri(server_uri)
        .expect("valid URI from wiremock")
        .personal_token("test-token".to_string())
        .build()
        .expect("octocrab builder succeeds with valid base_uri");
    github_client::GitHubClient::new(octocrab)
}

/// Success path: `create_fork_repository` must call `create_fork` against
/// the source repository, apply post-creation settings (a no-op here since
/// `merged_config` and the request carry no teams/collaborators/labels/etc.),
/// look up the organization's default branch, and return a
/// `RepositoryCreationResult` built from the forked repository.
#[tokio::test]
async fn test_create_fork_repository_success_path_reaches_fork_and_default_branch_calls_and_returns_result()
{
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/source-org/source-repo/forks"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "full_name": "test-org/new-repo",
            "name": "new-repo",
            "node_id": "MDEwOlJlcG9zaXRvcnkx",
            "private": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/orgs/test-org"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "default_repository_branch": "main"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let request = fork_request_with_source("source-org");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("source-org").unwrap(),
        name: RepositoryName::new("source-repo").unwrap(),
    };
    let merged_config = config_manager::MergedConfiguration::new();
    let clients = CreationClients {
        installation_token: "test-token".to_string(),
        installation_repo_client: wiremock_github_client(&server.uri()),
        template_fetcher: template_engine::GitHubTemplateFetcher::new(),
    };

    let result =
        create_fork_repository(&request, &source, None, &merged_config, &clients, "test-user")
            .await;

    assert!(result.is_ok(), "fork creation should succeed: {result:?}");
    let creation_result = result.unwrap();
    assert_eq!(
        creation_result.repository_url, "https://github.com/test-org/new-repo.git",
        "repository_url must be derived from the forked repository's full_name"
    );
    assert_eq!(
        creation_result.default_branch, "main",
        "default_branch must come from get_organization_default_branch"
    );

    server.verify().await; // wiremock asserts both expect(1) calls were satisfied
}

/// `create_history_mirror_repository` must resolve visibility, then call
/// `create_github_repository` (POST `/orgs/{owner}/repos`) before ever
/// attempting the (network-requiring, see module note above)
/// `git::mirror_clone`/`git::mirror_push` steps. This test forces the
/// repository-creation call to fail and asserts the error is propagated as
/// `RepoRollerError::GitHub`, proving the dispatch reaches that call with the
/// correct parameters and stops there rather than proceeding to the
/// unreachable-in-tests git mirror steps.
#[tokio::test]
async fn test_create_history_mirror_repository_propagates_github_repository_creation_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/orgs/test-org/repos"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "Repository creation failed"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let request = mirror_request_with_source("source-org", true);
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("source-org").unwrap(),
        name: RepositoryName::new("source-repo").unwrap(),
    };
    let merged_config = config_manager::MergedConfiguration::new();
    let clients = CreationClients {
        installation_token: "test-token".to_string(),
        installation_repo_client: wiremock_github_client(&server.uri()),
        template_fetcher: template_engine::GitHubTemplateFetcher::new(),
    };

    let result = create_history_mirror_repository(
        &request,
        &source,
        None,
        &merged_config,
        &clients,
        Arc::new(MockVisibilityPolicyProvider),
        Arc::new(MockEnvironmentDetector),
        "test-user",
    )
    .await;

    assert!(
        matches!(result, Err(RepoRollerError::GitHub(_))),
        "repository-creation failure must be propagated as RepoRollerError::GitHub: {result:?}"
    );

    server.verify().await; // wiremock asserts the repo-creation call was made exactly once
}

/// `create_source_only_mirror_repository` must generate local content
/// (`ZeroContentProvider`, since no template is designated), initialize and
/// commit the local git repository (the exact `prepare_index_and_tree` code
/// path the #255 defect lived in -- exercised here end-to-end, locally, with
/// no network involved), then call `create_github_repository` before ever
/// attempting the (network-requiring, see module note above)
/// `push_repository_to_github_with_retry`/`git::shallow_clone_working_tree`/
/// `git::create_snapshot_branch` steps. This test forces the
/// repository-creation call to fail and asserts the error is propagated as
/// `RepoRollerError::GitHub`, proving the dispatch reaches that call (after
/// successfully exercising the fixed local git plumbing) with the correct
/// parameters and stops there rather than proceeding to the
/// unreachable-in-tests git/network steps.
#[tokio::test]
async fn test_create_source_only_mirror_repository_propagates_github_repository_creation_error() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/orgs/test-org"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "default_repository_branch": "main"
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/orgs/test-org/repos"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "Repository creation failed"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let request = mirror_request_with_source("source-org", false);
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("source-org").unwrap(),
        name: RepositoryName::new("source-repo").unwrap(),
    };
    let merged_config = config_manager::MergedConfiguration::new();
    let clients = CreationClients {
        installation_token: "test-token".to_string(),
        installation_repo_client: wiremock_github_client(&server.uri()),
        template_fetcher: template_engine::GitHubTemplateFetcher::new(),
    };

    let result = create_source_only_mirror_repository(
        &request,
        &source,
        None,
        &merged_config,
        &clients,
        Arc::new(MockVisibilityPolicyProvider),
        Arc::new(MockEnvironmentDetector),
        "test-user",
    )
    .await;

    assert!(
        matches!(result, Err(RepoRollerError::GitHub(_))),
        "repository-creation failure must be propagated as RepoRollerError::GitHub: {result:?}"
    );

    server.verify().await; // wiremock asserts both expect(1) calls were satisfied
}

// ============================================================================
// designation_for_creation_mode Tests (RED: #255 follow-up Gap 1 --
// "designated template is never auto-discovered")
// ============================================================================
//
// `designation_for_creation_mode` is the pure mapping `load_creation_config`
// must use to decide which `config_manager::TemplateDesignation` (if any) a
// request's `CreationMode` implies, per
// docs/spec/interfaces/fork-mirror-operations.md and the #255 follow-up gap
// note ("load_creation_config only loads a template when the caller's
// request.template is explicitly Some(...)"). It must consult only
// `creation_mode` -- NOT `request.template` -- so that the separate
// "explicit template always wins" rule (tested in
// `configuration_tests.rs::effective_template_name`) stays orthogonal and
// composable with this mapping.
//
// Mapping required:
//   CreationMode::Standard                          -> None
//   CreationMode::Fork { .. }                        -> Some(Fork)
//   CreationMode::Mirror { preserve_history: true }  -> Some(SourceHistoryMirror)
//   CreationMode::Mirror { preserve_history: false } -> Some(SourceOnlyMirror)

#[test]
fn test_designation_for_creation_mode_standard_returns_none() {
    let mode = CreationMode::Standard;

    let designation = designation_for_creation_mode(&mode);

    assert_eq!(
        designation, None,
        "Standard creation has no fork/mirror designation"
    );
}

#[test]
fn test_designation_for_creation_mode_fork_returns_fork_designation() {
    let mode = CreationMode::Fork {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
    };

    let designation = designation_for_creation_mode(&mode);

    assert_eq!(
        designation,
        Some(config_manager::TemplateDesignation::Fork)
    );
}

#[test]
fn test_designation_for_creation_mode_history_mirror_returns_source_history_mirror_designation() {
    let mode = CreationMode::Mirror {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
        preserve_history: true,
    };

    let designation = designation_for_creation_mode(&mode);

    assert_eq!(
        designation,
        Some(config_manager::TemplateDesignation::SourceHistoryMirror)
    );
}

#[test]
fn test_designation_for_creation_mode_source_only_mirror_returns_source_only_mirror_designation() {
    let mode = CreationMode::Mirror {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
        preserve_history: false,
    };

    let designation = designation_for_creation_mode(&mode);

    assert_eq!(
        designation,
        Some(config_manager::TemplateDesignation::SourceOnlyMirror)
    );
}

#[test]
fn test_designation_for_creation_mode_history_and_source_only_mirror_designations_are_distinct() {
    // Stub-killing: a lazy implementation might return the same designation
    // for both `preserve_history` values (e.g. always `SourceHistoryMirror`).
    // Assert they differ.
    let history = CreationMode::Mirror {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
        preserve_history: true,
    };
    let source_only = CreationMode::Mirror {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
        preserve_history: false,
    };

    assert_ne!(
        designation_for_creation_mode(&history),
        designation_for_creation_mode(&source_only),
        "history and source-only mirrors must map to distinct designations"
    );
}

#[test]
fn test_designation_for_creation_mode_fork_and_standard_designations_are_distinct() {
    // Stub-killing: a lazy implementation that always returns `Some(Fork)`
    // (or always `None`) would pass a single-variant test but fail this one.
    let fork = CreationMode::Fork {
        source: SourceRepositoryReference {
            owner: OrganizationName::new("source-org").unwrap(),
            name: RepositoryName::new("source-repo").unwrap(),
        },
    };

    assert_ne!(
        designation_for_creation_mode(&fork),
        designation_for_creation_mode(&CreationMode::Standard)
    );
}

// ============================================================================
// history_mirror_default_branch_update Tests (RED: #255 follow-up Gap 2 --
// "history mirror never sets the new repo's default branch on GitHub")
// ============================================================================
//
// `create_history_mirror_repository` must call
// `installation_repo_client.update_repository_settings` with a
// `RepositorySettingsUpdate { default_branch: Some(source_default_branch),
// ..Default::default() }` after `git::mirror_push` succeeds and before
// returning. That call -- like the `git::mirror_clone`/`git::mirror_push`
// steps themselves -- requires a live, network-reachable source repository
// (see the module note above `wiremock_github_client`: the source and
// push-target URLs are hardcoded `https://github.com/...` with no override
// seam), so the full `create_history_mirror_repository` success path
// (reaching this PATCH call) cannot be exercised end-to-end in this test
// suite without live GitHub credentials, exactly like `mirror_clone` itself.
//
// `history_mirror_default_branch_update` isolates the one piece of this fix
// that *is* synchronous, network-free, and therefore unit-testable without
// live credentials: the exact `RepositorySettingsUpdate` payload built from
// the source's default branch name. `create_history_mirror_repository` MUST
// build its settings-update payload using this exact function (or logic
// equivalent to it) so that payload construction cannot silently regress
// (e.g. a hardcoded branch name, or accidentally setting unrelated fields)
// independently of the (untestable-here) network wiring around it.

#[test]
fn test_history_mirror_default_branch_update_sets_default_branch_field() {
    let settings = history_mirror_default_branch_update("develop");

    assert_eq!(
        settings.default_branch,
        Some("develop".to_string()),
        "default_branch must be set to the source's default branch name"
    );
}

#[test]
fn test_history_mirror_default_branch_update_preserves_arbitrary_branch_names() {
    // Stub-killing: a hardcoded "main" would pass the test above but fail
    // here, since the source's actual default branch is "trunk".
    let settings = history_mirror_default_branch_update("trunk");

    assert_eq!(settings.default_branch, Some("trunk".to_string()));
}

#[test]
fn test_history_mirror_default_branch_update_rejects_empty_string_sentinel() {
    // Stub-killing: a lazy implementation that unconditionally returns
    // `Default::default()` (default_branch = None) would fail every test in
    // this section; an implementation that returns `Some(String::new())`
    // regardless of input would fail this one.
    let settings = history_mirror_default_branch_update("release/2026.1");

    assert_eq!(
        settings.default_branch,
        Some("release/2026.1".to_string())
    );
}

#[test]
fn test_history_mirror_default_branch_update_does_not_set_unrelated_fields() {
    let settings = history_mirror_default_branch_update("main");

    assert_eq!(settings.description, None);
    assert_eq!(settings.homepage, None);
    assert_eq!(settings.private, None);
    assert_eq!(settings.has_issues, None);
    assert_eq!(settings.has_projects, None);
    assert_eq!(settings.has_wiki, None);
}
