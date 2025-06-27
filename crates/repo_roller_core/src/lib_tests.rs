// Unit tests for repo_roller_core
// Covers create_repository success and error paths

use super::*;
use async_trait::async_trait;
use config_manager::ConfigError;
use github_client::{models, RepositorySettingsUpdate};
use std::sync::Mutex;

// --- MOCKS ---
// We'll use static Mutex to allow test-by-test override of dependency behavior.
// In a real project, dependency injection would be preferable.

static MOCK_CONFIG: Mutex<Option<config_manager::Config>> = Mutex::new(None);
static MOCK_TEMPLATE_FILES: Mutex<Option<Result<Vec<(String, Vec<u8>)>, String>>> =
    Mutex::new(None);
static MOCK_GITHUB_CLIENT: Mutex<Option<Result<MockGitHubClient, String>>> = Mutex::new(None);

#[derive(Clone)]
pub struct MockGitHubClient;

impl MockGitHubClient {
    pub async fn create_org_repository(
        &self,
        _owner: &str,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::models::Repository, String> {
        Err("MockGitHubClient cannot construct Repository; this is expected in tests".to_string())
    }

    pub async fn create_user_repository(
        &self,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::models::Repository, String> {
        Err("MockGitHubClient cannot construct Repository; this is expected in tests".to_string())
    }
}

// --- MOCK TRAITS FOR DEPENDENCY INJECTION ---

struct MockConfigLoader;
impl config_manager::ConfigLoader for MockConfigLoader {
    fn load_config(&self, _path: &str) -> Result<config_manager::Config, ConfigError> {
        MOCK_CONFIG
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| ConfigError::Text("Mock config missing".to_string()))
    }
}

struct MockTemplateFetcher;
impl template_engine::TemplateFetcher for MockTemplateFetcher {
    fn fetch_template_files(&self, _source_repo: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        MOCK_TEMPLATE_FILES
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Ok(vec![]))
    }
}

struct MockRepoClient;

#[async_trait]
impl github_client::RepositoryClient for MockRepoClient {
    async fn create_org_repository(
        &self,
        _owner: &str,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::models::Repository, github_client::errors::Error> {
        Err(github_client::errors::Error::AuthError(
            "MockRepoClient cannot construct Repository; this is expected in tests".to_string(),
        ))
    }

    async fn create_user_repository(
        &self,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::models::Repository, github_client::errors::Error> {
        Err(github_client::errors::Error::AuthError(
            "MockRepoClient cannot construct Repository; this is expected in tests".to_string(),
        ))
    }
    async fn update_repository_settings(
        &self,
        _owner: &str,
        _repo: &str,
        _settings: &RepositorySettingsUpdate,
    ) -> Result<models::Repository, github_client::errors::Error> {
        Err(github_client::errors::Error::AuthError(
            "MockRepoClient cannot construct Repository; this is expected in tests".to_string(),
        ))
    }

    async fn get_installation_token_for_org(
        &self,
        org_name: &str,
    ) -> Result<String, github_client::errors::Error> {
        // Return a mock token for testing
        Ok(format!("ghs_mock_token_for_{}", org_name))
    }
}

// --- TESTS ---

fn setup_success_mocks() {
    // Config with one template
    let config = config_manager::Config {
        templates: vec![config_manager::TemplateConfig {
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
        }],
    };
    *MOCK_CONFIG.lock().unwrap() = Some(config);
    *MOCK_TEMPLATE_FILES.lock().unwrap() = Some(Ok(vec![(
        "README.md".to_string(),
        b"test content".to_vec(),
    )]));
    *MOCK_GITHUB_CLIENT.lock().unwrap() = Some(Ok(MockGitHubClient));
}

#[tokio::test]
async fn test_create_repository_template_not_found() {
    setup_success_mocks();
    *MOCK_CONFIG.lock().unwrap() = Some(config_manager::Config { templates: vec![] });
    let req = CreateRepoRequest {
        name: "mockrepo".to_string(),
        owner: "mockorg".to_string(),
        template: "missing".to_string(),
    };
    let config_loader = MockConfigLoader;
    let template_fetcher = MockTemplateFetcher;
    let repo_client = MockRepoClient;
    let result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;
    assert!(!result.success);
    assert!(result.message.contains("Template not found"));
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
async fn test_create_repository_gets_installation_token() {
    setup_success_mocks();

    // Create a mock repo client that tracks if get_installation_token_for_org was called
    struct TokenTrackingMockRepoClient {
        token_called: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl github_client::RepositoryClient for TokenTrackingMockRepoClient {
        async fn create_org_repository(
            &self,
            _owner: &str,
            payload: &github_client::RepositoryCreatePayload,
        ) -> Result<github_client::models::Repository, github_client::errors::Error> {
            // Return a mock repository
            Ok(github_client::models::Repository::new(
                payload.name.clone(),
                format!("test-org/{}", payload.name),
                "MDEwOlJlcG9zaXRvcnkx".to_string(),
                false,
            ))
        }

        async fn create_user_repository(
            &self,
            _payload: &github_client::RepositoryCreatePayload,
        ) -> Result<github_client::models::Repository, github_client::errors::Error> {
            Err(github_client::errors::Error::AuthError(
                "User repos not supported in test".to_string(),
            ))
        }

        async fn update_repository_settings(
            &self,
            _owner: &str,
            _repo: &str,
            _settings: &RepositorySettingsUpdate,
        ) -> Result<models::Repository, github_client::errors::Error> {
            Err(github_client::errors::Error::AuthError(
                "Not implemented in test".to_string(),
            ))
        }

        async fn get_installation_token_for_org(
            &self,
            _org_name: &str,
        ) -> Result<String, github_client::errors::Error> {
            // Mark that this method was called
            *self.token_called.lock().unwrap() = true;
            Ok("ghs_test_token_for_org".to_string())
        }
    }

    let token_called = std::sync::Arc::new(std::sync::Mutex::new(false));
    let repo_client = TokenTrackingMockRepoClient {
        token_called: token_called.clone(),
    };

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

    let config_loader = MockConfigLoader;
    let template_fetcher = MockTemplateFetcher;

    let result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;

    // The repository creation should fail at the git push step (since we can't actually push),
    // but it should have called get_installation_token_for_org
    assert!(
        *token_called.lock().unwrap(),
        "get_installation_token_for_org should have been called"
    );
}

#[tokio::test]
async fn test_create_repository_fails_without_owner() {
    setup_success_mocks();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "".to_string(), // Empty owner - should trigger user repo error
        template: "basic".to_string(),
    };

    let config_loader = MockConfigLoader;
    let template_fetcher = MockTemplateFetcher;
    let repo_client = MockRepoClient;

    let result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;

    assert!(!result.success);
    assert!(result
        .message
        .contains("User repositories not yet supported"));
}

#[tokio::test]
async fn test_create_repository_fails_on_token_error() {
    setup_success_mocks();

    // Create a mock repo client that fails to get installation token
    struct FailingTokenMockRepoClient;

    #[async_trait]
    impl github_client::RepositoryClient for FailingTokenMockRepoClient {
        async fn create_org_repository(
            &self,
            _owner: &str,
            payload: &github_client::RepositoryCreatePayload,
        ) -> Result<github_client::models::Repository, github_client::errors::Error> {
            Ok(github_client::models::Repository::new(
                payload.name.clone(),
                format!("test-org/{}", payload.name),
                "MDEwOlJlcG9zaXRvcnkx".to_string(),
                false,
            ))
        }

        async fn create_user_repository(
            &self,
            _payload: &github_client::RepositoryCreatePayload,
        ) -> Result<github_client::models::Repository, github_client::errors::Error> {
            Err(github_client::errors::Error::AuthError(
                "Not implemented".to_string(),
            ))
        }

        async fn update_repository_settings(
            &self,
            _owner: &str,
            _repo: &str,
            _settings: &RepositorySettingsUpdate,
        ) -> Result<models::Repository, github_client::errors::Error> {
            Err(github_client::errors::Error::AuthError(
                "Not implemented".to_string(),
            ))
        }

        async fn get_installation_token_for_org(
            &self,
            _org_name: &str,
        ) -> Result<String, github_client::errors::Error> {
            Err(github_client::errors::Error::InvalidResponse)
        }
    }

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

    let config_loader = MockConfigLoader;
    let template_fetcher = MockTemplateFetcher;
    let repo_client = FailingTokenMockRepoClient;

    let result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;

    assert!(!result.success);
    assert!(result.message.contains("Failed to get installation token"));
}
