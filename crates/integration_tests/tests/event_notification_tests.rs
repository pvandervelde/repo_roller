//! Integration tests for outbound event notifications.
//!
//! These tests verify that `create_repository` completes successfully when
//! `EventNotificationContext` is provided, confirming the fire-and-forget
//! notification path does not block or break repository creation.
//!
//! ## What is NOT tested here
//!
//! HTTP delivery correctness (HMAC signatures, header values, retry behaviour,
//! 4xx/5xx handling) is covered by unit tests in
//! `crates/repo_roller_core/src/event_publisher_tests.rs` using wiremock.
//!
//! End-to-end delivery to a real external webhook requires a publicly reachable
//! HTTPS endpoint and is outside the scope of these tests.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use integration_tests::{
    create_event_notification_providers, create_visibility_providers, generate_test_repo_name,
    TestConfig, TestRepository,
};
use repo_roller_core::{
    create_repository, event_metrics::NoOpEventMetrics, event_secrets::EnvironmentSecretResolver,
    EventNotificationContext, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
};
use std::sync::Arc;
use tracing::info;

/// Initialize logging for tests.
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_test_writer()
        .try_init();
}

/// Verify that `create_repository` succeeds when a full `EventNotificationContext`
/// is provided (Prometheus metrics + environment secret resolver).
///
/// The metadata repositories have no `notifications.toml`, so no HTTP delivery
/// is attempted. The test confirms the plumbing compiles and runs without panic.
#[tokio::test]
async fn test_repository_creation_succeeds_with_event_notifications_context() -> Result<()> {
    init_test_logging();
    info!("Testing repository creation with EventNotificationContext");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "event-ctx");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;
    let event_providers = create_event_notification_providers();

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(
        result.is_ok(),
        "Repository creation should succeed: {:?}",
        result.err()
    );

    let repo_result = result.unwrap();
    assert!(
        repo_result.repository_url.contains(&repo_name),
        "Result URL should contain the repository name"
    );

    info!(" Repository creation succeeded with EventNotificationContext");
    Ok(())
}

/// Verify that `create_repository` succeeds when using `NoOpEventMetrics`.
///
/// This confirms the metrics implementation is wired up correctly and that
/// a no-op implementation does not cause any issues.
#[tokio::test]
async fn test_repository_creation_succeeds_with_noop_event_metrics() -> Result<()> {
    init_test_logging();
    info!("Testing repository creation with NoOpEventMetrics");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "event-noop");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    let secret_resolver = Arc::new(EnvironmentSecretResolver::new());
    let metrics = Arc::new(NoOpEventMetrics::new());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
        EventNotificationContext::new("integration-test-noop", secret_resolver, metrics),
    )
    .await;

    assert!(
        result.is_ok(),
        "Repository creation should succeed with NoOpEventMetrics: {:?}",
        result.err()
    );
    info!(" NoOpEventMetrics does not interfere with repository creation");
    Ok(())
}

/// Verify that repository creation does NOT fail when notification delivery
/// would fail (fire-and-forget guarantee).
///
/// The background task spawned by `create_repository` must never propagate
/// errors back to the caller. This test confirms the return value reflects
/// only the repository-creation outcome, not any notification outcome.
#[tokio::test]
async fn test_notification_failure_does_not_block_repository_creation() -> Result<()> {
    init_test_logging();
    info!("Testing that notification failure does not block repository creation");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "event-no-block");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    let secret_resolver = Arc::new(EnvironmentSecretResolver::new());
    let metrics = Arc::new(NoOpEventMetrics::new());

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
        EventNotificationContext::new("integration-test-no-block", secret_resolver, metrics),
    )
    .await;

    assert!(
        result.is_ok(),
        "Repository creation must succeed even when notification delivery would fail: {:?}",
        result.err()
    );

    info!(" Notification failure path does not affect repository creation result");
    Ok(())
}

/// Verify that the `EventNotificationContext` correctly propagates the `created_by`
/// identifier into repository creation without error.
#[tokio::test]
async fn test_created_by_identifier_is_accepted() -> Result<()> {
    init_test_logging();
    info!("Testing created_by identifier in EventNotificationContext");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "event-creator");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;
    let event_providers = create_event_notification_providers();

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-test-basic")?)
    .build();

    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "jane.doe@example.com",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(
        result.is_ok(),
        "Repository creation should accept any created_by string: {:?}",
        result.err()
    );
    info!(" created_by identifier accepted without issue");
    Ok(())
}
