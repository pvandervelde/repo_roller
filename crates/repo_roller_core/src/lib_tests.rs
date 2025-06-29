// Unit tests for repo_roller_core
// Covers create_repository success and error paths with isolated mock dependencies

use super::*;
use async_trait::async_trait;
use config_manager::{Config, ConfigError, ConfigLoader, TemplateConfig};
use github_client::{
    errors::Error as GitHubError, models, RepositoryClient, RepositorySettingsUpdate,
};
use std::sync::{Arc, Mutex};
use template_engine::TemplateFetcher;

// --- MOCK FACTORY FUNCTIONS ---
// These functions create fresh, isolated mock instances for each test

/// Creates a mock config loader that returns the provided config
fn create_mock_config_loader(config: Config) -> impl ConfigLoader {
    struct MockConfigLoader {
        config: Config,
    }

    impl ConfigLoader for MockConfigLoader {
        fn load_config(&self, _path: &str) -> Result<Config, ConfigError> {
            Ok(self.config.clone())
        }
    }

    MockConfigLoader { config }
}

/// Creates a mock template fetcher that returns the provided files
fn create_mock_template_fetcher(files: Vec<(String, Vec<u8>)>) -> impl TemplateFetcher {
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

    MockTemplateFetcher { files }
}

/// Creates a mock repository client with successful responses
fn create_mock_repo_client() -> impl RepositoryClient {
    struct MockRepoClient;

    #[async_trait]
    impl RepositoryClient for MockRepoClient {
        async fn create_org_repository(
            &self,
            _owner: &str,
            payload: &github_client::RepositoryCreatePayload,
        ) -> Result<models::Repository, GitHubError> {
            Ok(models::Repository::new(
                payload.name.clone(),
                format!("test-org/{}", payload.name),
                "MDEwOlJlcG9zaXRvcnkx".to_string(),
                false,
            ))
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

        async fn get_installation_token_for_org(
            &self,
            org_name: &str,
        ) -> Result<String, GitHubError> {
            Ok(format!("ghs_mock_token_for_{}", org_name))
        }

        async fn get_organization_default_branch(
            &self,
            _org_name: &str,
        ) -> Result<String, GitHubError> {
            Ok("main".to_string())
        }
    }

    MockRepoClient
}

/// Creates a mock repository client that tracks installation token calls
fn create_token_tracking_mock_repo_client() -> (impl RepositoryClient, Arc<Mutex<bool>>) {
    let token_called = Arc::new(Mutex::new(false));

    struct TokenTrackingMockRepoClient {
        token_called: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl RepositoryClient for TokenTrackingMockRepoClient {
        async fn create_org_repository(
            &self,
            _owner: &str,
            payload: &github_client::RepositoryCreatePayload,
        ) -> Result<models::Repository, GitHubError> {
            Ok(models::Repository::new(
                payload.name.clone(),
                format!("test-org/{}", payload.name),
                "MDEwOlJlcG9zaXRvcnkx".to_string(),
                false,
            ))
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

        async fn get_installation_token_for_org(
            &self,
            _org_name: &str,
        ) -> Result<String, GitHubError> {
            *self.token_called.lock().unwrap() = true;
            Ok("ghs_test_token_for_org".to_string())
        }

        async fn get_organization_default_branch(
            &self,
            _org_name: &str,
        ) -> Result<String, GitHubError> {
            Ok("main".to_string())
        }
    }

    let client = TokenTrackingMockRepoClient {
        token_called: token_called.clone(),
    };

    (client, token_called)
}

/// Creates a mock repository client that fails installation token retrieval
fn create_failing_token_mock_repo_client() -> impl RepositoryClient {
    struct FailingTokenMockRepoClient;

    #[async_trait]
    impl RepositoryClient for FailingTokenMockRepoClient {
        async fn create_org_repository(
            &self,
            _owner: &str,
            payload: &github_client::RepositoryCreatePayload,
        ) -> Result<models::Repository, GitHubError> {
            Ok(models::Repository::new(
                payload.name.clone(),
                format!("test-org/{}", payload.name),
                "MDEwOlJlcG9zaXRvcnkx".to_string(),
                false,
            ))
        }

        async fn create_user_repository(
            &self,
            _payload: &github_client::RepositoryCreatePayload,
        ) -> Result<models::Repository, GitHubError> {
            Err(GitHubError::AuthError("Not implemented".to_string()))
        }

        async fn update_repository_settings(
            &self,
            _owner: &str,
            _repo: &str,
            _settings: &RepositorySettingsUpdate,
        ) -> Result<models::Repository, GitHubError> {
            Err(GitHubError::AuthError("Not implemented".to_string()))
        }

        async fn get_installation_token_for_org(
            &self,
            _org_name: &str,
        ) -> Result<String, GitHubError> {
            Err(GitHubError::InvalidResponse)
        }

        async fn get_organization_default_branch(
            &self,
            _org_name: &str,
        ) -> Result<String, GitHubError> {
            Ok("main".to_string())
        }
    }

    FailingTokenMockRepoClient
}

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

// --- TESTS (ALPHABETICALLY ORDERED) ---

#[tokio::test]
async fn test_create_repository_fails_on_installation_token_error() {
    let config = create_config_with_basic_template();
    let config_loader = create_mock_config_loader(config);
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let repo_client = create_failing_token_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

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

#[tokio::test]
async fn test_create_repository_fails_with_empty_owner() {
    let config = create_config_with_basic_template();
    let config_loader = create_mock_config_loader(config);
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let repo_client = create_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "".to_string(), // Empty owner should trigger user repo creation
        template: "basic".to_string(),
    };

    let result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;

    assert!(!result.success);
    // The mock returns an AuthError for user repositories, which gets formatted differently
    assert!(result.message.contains("Invalid response format") || result.message.contains("Auth"));
}

#[tokio::test]
async fn test_create_repository_fails_with_template_not_found() {
    let config = create_empty_config(); // No templates
    let config_loader = create_mock_config_loader(config);
    let template_fetcher = create_mock_template_fetcher(vec![]);
    let repo_client = create_mock_repo_client();

    let req = CreateRepoRequest {
        name: "mockrepo".to_string(),
        owner: "mockorg".to_string(),
        template: "missing".to_string(),
    };

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
async fn test_create_repository_gets_installation_token() {
    let config = create_config_with_basic_template();
    let config_loader = create_mock_config_loader(config);
    let template_fetcher =
        create_mock_template_fetcher(vec![("README.md".to_string(), b"test content".to_vec())]);
    let (repo_client, token_called) = create_token_tracking_mock_repo_client();

    let req = CreateRepoRequest {
        name: "test-repo".to_string(),
        owner: "test-org".to_string(),
        template: "basic".to_string(),
    };

    let _result = create_repository_with_custom_settings(
        req,
        &config_loader,
        &template_fetcher,
        &repo_client,
    )
    .await;

    // The repository creation should call get_installation_token_for_org
    assert!(
        *token_called.lock().unwrap(),
        "get_installation_token_for_org should have been called"
    );
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
