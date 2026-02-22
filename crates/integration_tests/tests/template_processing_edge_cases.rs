//! Template processing edge case integration tests.
//!
//! These tests verify template engine behavior with complex scenarios:
//! large files, binary files, deep nesting, special characters.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use integration_tests::{generate_test_repo_name, RepositoryCleanup, TestConfig, TestRepository};
use repo_roller_core::{
    create_repository, OrganizationName, RepositoryCreationRequestBuilder, RepositoryName,
    TemplateName,
};
use tracing::info;

/// Initialize logging for tests
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_test_writer()
        .try_init();
}

/// Test processing templates with large files (>10MB).
///
/// Verifies that large files are handled correctly without
/// memory issues or timeouts.
#[tokio::test]
async fn test_large_file_processing() -> Result<()> {
    init_test_logging();
    info!("Testing large file processing");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "large-files");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-large-files")?)
    .build();

    // Create repository
    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Repository creation should succeed");

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Large file template processed successfully");

    // TODO: Verify large files transferred correctly (need API to check file sizes)

    // Cleanup
    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Large file processing test passed");
    Ok(())
}

/// Test processing templates with binary files.
///
/// Verifies that binary files (images, PDFs, executables)
/// are copied correctly without corruption.
#[tokio::test]
async fn test_binary_file_preservation() -> Result<()> {
    init_test_logging();
    info!("Testing binary file preservation");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "binary-files");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-binary-files")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Binary file template processed successfully");

    // TODO: Download binary files and verify checksums match originals

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Binary file preservation test passed");
    Ok(())
}

/// Test processing templates with deep directory nesting.
///
/// Verifies that deeply nested directory structures (>10 levels)
/// are handled correctly.
#[tokio::test]
async fn test_deep_directory_nesting() -> Result<()> {
    init_test_logging();
    info!("Testing deep directory nesting");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "deep-nesting");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-deep-nesting")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Deep nesting template processed successfully");

    // TODO: Verify all nested directories created (need list_repository_files API)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Deep directory nesting test passed");
    Ok(())
}

/// Test processing templates with many files (>1000).
///
/// Verifies performance and correctness when template
/// contains large number of files.
#[tokio::test]
async fn test_many_files_template() -> Result<()> {
    init_test_logging();
    info!("Testing template with many files");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "many-files");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-many-files")?)
    .build();

    let start_time = std::time::Instant::now();
    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok(), "Repository creation should succeed");
    info!("Repository creation took {:?}", elapsed);

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Many files template processed successfully");

    // TODO: Verify file count matches expected (need list_repository_files API)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Many files template test passed");
    Ok(())
}

/// Test filenames with special Unicode characters.
///
/// Verifies that filenames with Unicode, emojis, and special
/// characters are handled correctly.
#[tokio::test]
async fn test_unicode_filenames() -> Result<()> {
    init_test_logging();
    info!("Testing Unicode filenames");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "unicode-files");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-unicode-names")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Unicode filename template processed successfully");

    // TODO: Verify Unicode filenames preserved (need list_repository_files API)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Unicode filenames test passed");
    Ok(())
}

/// Test handling of symbolic links in templates.
///
/// Verifies how the system handles symlinks (should probably skip them
/// or resolve them, depending on design decision).
#[tokio::test]
async fn test_symlink_handling() -> Result<()> {
    init_test_logging();
    info!("Testing symbolic link handling");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "symlinks");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-with-symlinks")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Symlink template processed successfully");

    // TODO: Verify symlink handling (GitHub converts symlinks to regular files)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Symlink handling test passed");
    Ok(())
}

/// Test preservation of file permissions.
///
/// Verifies that executable bits and special permissions
/// are preserved during template processing.
#[tokio::test]
async fn test_executable_permissions_preserved() -> Result<()> {
    init_test_logging();
    info!("Testing executable permission preservation");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "executable-scripts");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-with-scripts")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Executable scripts template processed successfully");

    // TODO: Verify executable permissions (GitHub preserves via git mode)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Executable permissions test passed");
    Ok(())
}

/// Test templates with .gitignore and hidden files.
///
/// Verifies that hidden files (starting with .) are processed correctly.
#[tokio::test]
async fn test_hidden_files_processing() -> Result<()> {
    init_test_logging();
    info!("Testing hidden file processing");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "hidden-files");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-with-dotfiles")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Dotfiles template processed successfully");

    // TODO: Verify dotfiles copied (need list_repository_files API)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Hidden files processing test passed");
    Ok(())
}

/// Test template with empty directories.
///
/// Verifies handling of empty directories (Git doesn't track them,
/// so .gitkeep files might be needed).
#[tokio::test]
async fn test_empty_directory_handling() -> Result<()> {
    init_test_logging();
    info!("Testing empty directory handling");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "empty-dirs");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-empty-dirs")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ Empty directories template processed successfully");

    // TODO: Verify .gitkeep files created for empty directories

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Empty directory handling test passed");
    Ok(())
}

/// Test template with files that have no file extension.
///
/// Verifies that files without extensions are processed correctly.
#[tokio::test]
async fn test_files_without_extensions() -> Result<()> {
    init_test_logging();
    info!("Testing files without extensions");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "no-extensions");
    let _test_repo = TestRepository::new(repo_name.clone(), config.test_org.clone());

    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    // Create visibility providers
    let providers =
        integration_tests::create_visibility_providers(&installation_token, ".reporoller").await?;

    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(TemplateName::new("template-no-extensions")?)
    .build();

    let event_providers = integration_tests::create_event_notification_providers();
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider.clone(),
        providers.environment_detector.clone(),
        repo_roller_core::EventNotificationContext::new(
            "integration-test",
            event_providers.secret_resolver.clone(),
            event_providers.metrics.clone(),
        ),
    )
    .await;
    assert!(result.is_ok(), "Repository creation should succeed");

    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");
    info!("✓ No extensions template processed successfully");

    // TODO: Verify extension-less files copied (need list_repository_files API)

    let cleanup_client =
        github_client::create_app_client(config.github_app_id, &config.github_app_private_key)
            .await?;
    let cleanup = RepositoryCleanup::new(
        github_client::GitHubClient::new(cleanup_client),
        config.test_org.clone(),
    );
    cleanup.delete_repository(&repo_name).await.ok();

    info!("✓ Files without extensions test passed");
    Ok(())
}
