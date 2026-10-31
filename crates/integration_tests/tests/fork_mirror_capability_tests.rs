//! Integration tests for fork/mirror capability gating (RCF-005).
//!
//! See docs/spec/interfaces/fork-mirror-operations.md §2 and §7
//! ("Testing Requirements" → Integration tests: "Each capability disabled →
//! `CapabilityDisabled` error, no GitHub API calls made").
//!
//! # Fixture dependency
//!
//! These tests rely on the **default-disabled, fail-closed** behaviour
//! specified in fork-mirror-operations.md §2: "treat a fully-`None`
//! `CreationCapabilitiesSettings` at Global level as 'disabled'". Because
//! fork/mirror creation is a brand-new capability, the live `.reporoller-test`
//! metadata repository used by this test suite does not (yet) configure
//! `creation_capabilities` anywhere in its hierarchy — so these tests should
//! pass against the *unmodified* fixture, with no new fixture content
//! required. If a future change to `.reporoller-test` explicitly enables one
//! of these capabilities at any level, the corresponding test below will
//! need updating.
//!
//! True success-path fork/mirror creation (RCA-004/005/006) and the
//! non-designated-template rejection (RCF-006) additionally require a
//! designated/non-designated template fixture and an explicitly *enabled*
//! capability in `.reporoller-test`, which this test file does not assume —
//! see the Tester's report for this gap.
//!
//! # `allowed_source_owners` (RCF-005 extension) gap
//!
//! The same gap applies to the HIGH finding that `allowed_source_owners`
//! must additionally permit `source.owner` even when a capability is
//! enabled (docs/spec/interfaces/fork-mirror-operations.md §2): testing
//! "capability enabled + owner not on allow-list → `SourceOwnerNotAllowed`"
//! and "capability enabled + owner on allow-list → gate passes" both
//! require the capability to be explicitly enabled in `.reporoller-test`,
//! which it is not. Those scenarios are covered at the unit level instead,
//! directly against `validate_creation_mode_capability`, in
//! `crates/repo_roller_core/src/lib_tests.rs` (search for
//! `allowed_source_owners`), which does not require a live GitHub call.
//!
//! What *is* testable against the live, unmodified fixture is the ordering
//! guarantee that the capability-disabled check still wins over the
//! owner-allow-list check even for a source owner that could never
//! plausibly be on an allow-list — see the three
//! `*_disabled_by_default_takes_precedence_over_source_owner_check` tests
//! below.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use config_manager::{ConfigBasedPolicyProvider, GitHubMetadataProvider, MetadataProviderConfig};
use github_client::{create_token_client, GitHubApiEnvironmentDetector, GitHubClient};
use integration_tests::{generate_test_repo_name, TestConfig};
use repo_roller_core::{
    CreationMode, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    SourceRepositoryReference,
};
use std::sync::Arc;
use tracing::info;

/// Shared setup mirroring `error_handling_tests.rs`: real GitHub App auth
/// against the configured test organization and its `.reporoller-test`
/// metadata repository.
async fn build_test_harness() -> Result<(
    auth_handler::GitHubAuthService,
    Arc<GitHubMetadataProvider>,
    Arc<ConfigBasedPolicyProvider>,
    Arc<GitHubApiEnvironmentDetector>,
    TestConfig,
)> {
    let config = TestConfig::from_env()?;

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab = Arc::new(create_token_client(&installation_token)?);
    let github_client = GitHubClient::new(octocrab.as_ref().clone());

    let metadata_provider = Arc::new(GitHubMetadataProvider::new(
        github_client,
        MetadataProviderConfig::explicit(".reporoller-test"),
    ));

    let visibility_policy_provider =
        Arc::new(ConfigBasedPolicyProvider::new(metadata_provider.clone()));
    let environment_detector = Arc::new(GitHubApiEnvironmentDetector::new(octocrab));

    Ok((
        auth_service,
        metadata_provider,
        visibility_policy_provider,
        environment_detector,
        config,
    ))
}

fn assert_is_capability_disabled_error(
    error: &repo_roller_core::RepoRollerError,
    capability_hint: &str,
) {
    let message = error.to_string();
    assert!(
        message.to_lowercase().contains("disabled")
            || message.to_lowercase().contains("capability"),
        "expected a CapabilityDisabled-style error for '{capability_hint}', got: {message}"
    );
}

/// RCF-005: fork creation must be rejected with a capability-disabled error
/// when the organization has not explicitly enabled `fork_enabled` anywhere
/// in the configuration hierarchy (the fail-closed default).
#[tokio::test]
async fn test_fork_creation_disabled_by_default_returns_capability_disabled_error() -> Result<()> {
    info!("Testing fork creation capability-disabled-by-default rejection");

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "fork-disabled");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new(&config.test_org)?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Fork { source })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(
        result.is_err(),
        "fork creation must fail when fork_enabled is not explicitly set anywhere (fail-closed default)"
    );
    assert_is_capability_disabled_error(&result.unwrap_err(), "fork");

    info!("✓ Fork capability-disabled-by-default test passed");
    Ok(())
}

/// RCF-005: source+history mirror creation must be rejected by default.
#[tokio::test]
async fn test_source_history_mirror_disabled_by_default_returns_capability_disabled_error(
) -> Result<()> {
    info!("Testing source+history mirror capability-disabled-by-default rejection");

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "history-mirror-disabled");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new(&config.test_org)?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Mirror {
        source,
        preserve_history: true,
    })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(result.is_err());
    assert_is_capability_disabled_error(&result.unwrap_err(), "source_history_mirror");

    info!("✓ Source+history mirror capability-disabled-by-default test passed");
    Ok(())
}

/// RCF-005: source-only mirror creation must be rejected by default.
#[tokio::test]
async fn test_source_only_mirror_disabled_by_default_returns_capability_disabled_error(
) -> Result<()> {
    info!("Testing source-only mirror capability-disabled-by-default rejection");

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "source-only-mirror-disabled");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new(&config.test_org)?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Mirror {
        source,
        preserve_history: false,
    })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(result.is_err());
    assert_is_capability_disabled_error(&result.unwrap_err(), "source_only_mirror");

    info!("✓ Source-only mirror capability-disabled-by-default test passed");
    Ok(())
}

/// RCF-005 extension: the capability-disabled check must take precedence
/// over the `allowed_source_owners` check (docs/spec/interfaces/
/// fork-mirror-operations.md §2: "checks `allowed_source_owners`
/// immediately after the capability-enabled check"). Uses a source owner
/// name that could never plausibly appear on any real allow-list, so a
/// future fixture change that enables the capability for `config.test_org`
/// specifically cannot make this test coincidentally pass for the wrong
/// reason — it must fail with `CapabilityDisabled`, not
/// `SourceOwnerNotAllowed`, purely because the capability itself is off.
#[tokio::test]
async fn test_fork_creation_disabled_by_default_takes_precedence_over_source_owner_check(
) -> Result<()> {
    info!("Testing fork capability-disabled check precedence over source-owner allow-list check");

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "fork-disabled-owner-order");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("definitely-not-a-configured-allowlist-owner")?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Fork { source })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(result.is_err());
    // Discriminates against `SourceOwnerNotAllowed`: that error's message
    // template is "Source owner '...' is not in the configured allow-list
    // for '...'", which does not contain the literal substrings "disabled"
    // or "capability", so `assert_is_capability_disabled_error` would fail
    // if the gate checked owner-allow-list before capability-enabled.
    assert_is_capability_disabled_error(&result.unwrap_err(), "fork");

    info!("✓ Fork capability-disabled-takes-precedence-over-owner-check test passed");
    Ok(())
}

/// Symmetric to the fork test above, for source+history mirror.
#[tokio::test]
async fn test_source_history_mirror_disabled_by_default_takes_precedence_over_source_owner_check(
) -> Result<()> {
    info!(
        "Testing source+history mirror capability-disabled check precedence over \
         source-owner allow-list check"
    );

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "history-mirror-disabled-owner-order");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("definitely-not-a-configured-allowlist-owner")?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Mirror {
        source,
        preserve_history: true,
    })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(result.is_err());
    assert_is_capability_disabled_error(&result.unwrap_err(), "source_history_mirror");

    info!(
        "✓ Source+history mirror capability-disabled-takes-precedence-over-owner-check test passed"
    );
    Ok(())
}

/// Symmetric to the fork test above, for source-only mirror.
#[tokio::test]
async fn test_source_only_mirror_disabled_by_default_takes_precedence_over_source_owner_check(
) -> Result<()> {
    info!(
        "Testing source-only mirror capability-disabled check precedence over \
         source-owner allow-list check"
    );

    let (auth_service, metadata_provider, visibility_policy_provider, environment_detector, config) =
        build_test_harness().await?;

    let repo_name = generate_test_repo_name("test", "source-only-mirror-disabled-owner-order");
    let source = SourceRepositoryReference {
        owner: OrganizationName::new("definitely-not-a-configured-allowlist-owner")?,
        name: RepositoryName::new("template-test-basic")?,
    };

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .creation_mode(CreationMode::Mirror {
        source,
        preserve_history: false,
    })
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = repo_roller_core::create_repository(
        request,
        metadata_provider.as_ref(),
        &auth_service,
        ".reporoller-test",
        visibility_policy_provider,
        environment_detector,
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    assert!(result.is_err());
    assert_is_capability_disabled_error(&result.unwrap_err(), "source_only_mirror");

    info!("✓ Source-only mirror capability-disabled-takes-precedence-over-owner-check test passed");
    Ok(())
}
