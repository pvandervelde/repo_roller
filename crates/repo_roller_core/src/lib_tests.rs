// Unit tests for repo_roller_core
// Covers create_repository success and error paths

use super::*;
use async_trait::async_trait;
use config_manager::ConfigError;
use std::env;
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
    ) -> Result<github_client::models::Repository, github_client::Error> {
        Err(github_client::Error::AuthError(
            "MockRepoClient cannot construct Repository; this is expected in tests".to_string(),
        ))
    }
    async fn create_user_repository(
        &self,
        _payload: &github_client::RepositoryCreatePayload,
    ) -> Result<github_client::models::Repository, github_client::Error> {
        Err(github_client::Error::AuthError(
            "MockRepoClient cannot construct Repository; this is expected in tests".to_string(),
        ))
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
    let result = create_repository(req, &config_loader, &template_fetcher, &repo_client).await;
    assert!(!result.success);
    assert!(result.message.contains("Template not found"));
}
