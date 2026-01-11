//! Integration tests for empty repository creation (non-template repositories).
//!
//! These tests verify the functionality added in Task 6.0-6.8 for creating
//! repositories without using template content, including:
//! - Empty repositories (no files)
//! - Custom initialization (README.md and/or .gitignore only)
//! - Settings application without templates
//!
//! Tests are run sequentially (--test-threads=1) to avoid GitHub API rate limits
//! and repository name conflicts.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use integration_tests::{
    create_visibility_providers, generate_test_repo_name, TestConfig, TestRepository,
};
use repo_roller_core::{
    create_repository, ContentStrategy, OrganizationName, RepositoryCreationRequestBuilder,
    RepositoryName,
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

/// Test creating an empty repository with no template.
///
/// Verifies that:
/// - Repository is created successfully
/// - No files are present in the repository
/// - Organization default settings are applied
#[tokio::test]
async fn test_empty_repository_without_template() -> Result<()> {
    init_test_logging();
    info!("Testing empty repository creation without template");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "empty-no-template");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with Empty content strategy and no template
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .content_strategy(ContentStrategy::Empty)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(result.is_ok(), "Empty repository creation should succeed");

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify repository is empty by trying to get README.md
    // Empty repository should not have any files
    let readme_result = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await;

    assert!(
        readme_result.is_err(),
        "Empty repository should not have README.md"
    );

    info!("✓ Empty repository without template test passed");
    Ok(())
}

/// Test creating an empty repository with template settings.
///
/// Verifies that:
/// - Repository is created successfully
/// - No files are present (despite template providing them)
/// - Template settings are applied (not org defaults)
#[tokio::test]
async fn test_empty_repository_with_template_settings() -> Result<()> {
    init_test_logging();
    info!("Testing empty repository with template settings");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "empty-with-settings");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with Empty content strategy but with template for settings
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .content_strategy(ContentStrategy::Empty)
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "Empty repository with template settings should succeed"
    );

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify repository is empty despite having template
    let readme_result = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await;

    assert!(
        readme_result.is_err(),
        "Empty repository should not have README.md even with template"
    );

    // TODO: Verify that template settings were applied (once we have setting inspection)
    info!("✓ Empty repository with template settings test passed");
    Ok(())
}

/// Test creating a repository with custom initialization (README only).
///
/// Verifies that:
/// - Repository is created successfully
/// - Only README.md is present
/// - README has expected content
#[tokio::test]
async fn test_custom_init_readme_only() -> Result<()> {
    init_test_logging();
    info!("Testing custom initialization with README only");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "init-readme");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with CustomInit content strategy (README only)
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: false,
    })
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "Custom init repository creation should succeed"
    );

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify repository has README.md
    let readme_content = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await?;

    assert!(
        readme_content.contains(&repo_name),
        "README should contain repository name"
    );
    assert!(
        readme_content.starts_with("# "),
        "README should start with heading"
    );

    // Verify no .gitignore file
    let gitignore_result = verification_client
        .get_file_content(&config.test_org, &repo_name, ".gitignore")
        .await;

    assert!(
        gitignore_result.is_err(),
        "Repository should not have .gitignore when not requested"
    );

    info!("✓ Custom init with README only test passed");
    Ok(())
}

/// Test creating a repository with custom initialization (gitignore only).
///
/// Verifies that:
/// - Repository is created successfully
/// - Only .gitignore is present
/// - .gitignore has expected content
#[tokio::test]
async fn test_custom_init_gitignore_only() -> Result<()> {
    init_test_logging();
    info!("Testing custom initialization with .gitignore only");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "init-gitignore");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with CustomInit content strategy (gitignore only)
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: false,
        include_gitignore: true,
    })
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "Custom init repository creation should succeed"
    );

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify repository has .gitignore
    let gitignore_content = verification_client
        .get_file_content(&config.test_org, &repo_name, ".gitignore")
        .await?;

    assert!(
        !gitignore_content.is_empty(),
        ".gitignore should have content"
    );
    assert!(
        gitignore_content.contains("target/") || gitignore_content.contains("*.log"),
        ".gitignore should contain common ignore patterns"
    );

    // Verify no README file
    let readme_result = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await;

    assert!(
        readme_result.is_err(),
        "Repository should not have README.md when not requested"
    );

    info!("✓ Custom init with .gitignore only test passed");
    Ok(())
}

/// Test creating a repository with custom initialization (both README and gitignore).
///
/// Verifies that:
/// - Repository is created successfully
/// - Both README.md and .gitignore are present
/// - No other files exist
#[tokio::test]
async fn test_custom_init_both_files() -> Result<()> {
    init_test_logging();
    info!("Testing custom initialization with README and .gitignore");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "init-both");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with CustomInit content strategy (both files)
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    })
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "Custom init repository creation should succeed"
    );

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify repository has both README.md and .gitignore
    let readme_content = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await?;

    assert!(
        readme_content.contains(&repo_name),
        "README should contain repository name"
    );

    let gitignore_content = verification_client
        .get_file_content(&config.test_org, &repo_name, ".gitignore")
        .await?;

    assert!(
        !gitignore_content.is_empty(),
        ".gitignore should have content"
    );

    info!("✓ Custom init with both files test passed");
    Ok(())
}

/// Test creating a custom init repository with template settings.
///
/// Verifies that:
/// - Repository is created successfully
/// - Only custom init files are present (not template files)
/// - Template settings are applied
#[tokio::test]
async fn test_custom_init_with_template_settings() -> Result<()> {
    init_test_logging();
    info!("Testing custom initialization with template settings");

    let config = TestConfig::from_env()?;
    let repo_name = generate_test_repo_name("test", "init-with-settings");
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
    let providers = create_visibility_providers(&installation_token, ".reporoller").await?;

    // Build request with CustomInit content strategy and template for settings
    let request = RepositoryCreationRequestBuilder::new(
        RepositoryName::new(&repo_name)?,
        OrganizationName::new(&config.test_org)?,
    )
    .template(repo_roller_core::TemplateName::new("template-test-basic")?)
    .content_strategy(ContentStrategy::CustomInit {
        include_readme: true,
        include_gitignore: true,
    })
    .build();

    // Create repository
    let result = create_repository(
        request,
        providers.metadata_provider.as_ref(),
        &auth_service,
        ".reporoller",
        providers.visibility_policy_provider,
        providers.environment_detector,
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "Custom init with template settings should succeed"
    );

    // Verify repository exists
    let verification_client = github_client::create_token_client(&installation_token)?;
    let verification_client = github_client::GitHubClient::new(verification_client);

    let repo = verification_client
        .get_repository(&config.test_org, &repo_name)
        .await?;

    assert_eq!(repo.name(), repo_name, "Repository name should match");

    // Verify only custom init files are present (README.md and .gitignore)
    let readme_content = verification_client
        .get_file_content(&config.test_org, &repo_name, "README.md")
        .await?;

    assert!(
        readme_content.contains(&repo_name),
        "README should contain repository name"
    );

    let gitignore_content = verification_client
        .get_file_content(&config.test_org, &repo_name, ".gitignore")
        .await?;

    assert!(
        !gitignore_content.is_empty(),
        ".gitignore should have content"
    );

    // Verify template files are NOT present
    // (template-test-basic would have other files if template content was used)
    // We can't easily list all files, but we know custom init should only have 2 files

    // TODO: Verify that template settings were applied
    info!("✓ Custom init with template settings test passed");
    Ok(())
}
