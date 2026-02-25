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

pub struct TestEventNotificationProviders {
    pub secret_resolver: Arc<dyn repo_roller_core::event_secrets::SecretResolver>,
    pub metrics: Arc<dyn repo_roller_core::event_metrics::EventMetrics>,
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

/// Creates event notification providers for integration tests.
///
/// Returns a new set of providers with:
/// - `EnvironmentSecretResolver` for secret resolution
/// - `PrometheusEventMetrics` with a new registry
pub fn create_event_notification_providers() -> TestEventNotificationProviders {
    let secret_resolver =
        Arc::new(repo_roller_core::event_secrets::EnvironmentSecretResolver::new());
    let metrics_registry = prometheus::Registry::new();
    let metrics =
        Arc::new(repo_roller_core::event_metrics::PrometheusEventMetrics::new(&metrics_registry));

    TestEventNotificationProviders {
        secret_resolver,
        metrics,
    }
}
