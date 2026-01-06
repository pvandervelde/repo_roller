/// Helper function to create visibility providers for integration tests
use anyhow::Result;
use config_manager::{ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig};
use github_client::{create_token_client, GitHubApiEnvironmentDetector, GitHubClient};
use std::sync::Arc;

pub struct TestVisibilityProviders {
    pub metadata_provider: Arc<GitHubMetadataProvider>,
    pub visibility_policy_provider: Arc<ConfigBasedPolicyProvider>,
    pub environment_detector: Arc<GitHubApiEnvironmentDetector>,
}

pub async fn create_visibility_providers(
    installation_token: &str,
    metadata_repo: &str,
) -> Result<TestVisibilityProviders> {
    let octocrab = Arc::new(create_token_client(installation_token)?);
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client,
        MetadataProviderConfig::explicit(metadata_repo),
    ));

    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    Ok(TestVisibilityProviders {
        metadata_provider,
        visibility_policy_provider,
        environment_detector,
    })
}
