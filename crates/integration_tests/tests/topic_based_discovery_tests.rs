//! Topic-based metadata repository discovery integration tests.
//!
//! These tests verify the topic-based discovery mechanism for finding
//! metadata repositories using GitHub repository topics.
//!
//! Tests against real GitHub infrastructure (glitchgrove organization).

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use config_manager::MetadataRepositoryProvider;
use integration_tests::utils::TestConfig;
use tracing::info;

/// Test search_repositories_by_topic with existing metadata repository.
///
/// Verifies that searching for repositories with the 'reporoller-metadata' topic
/// in the glitchgrove organization successfully finds the .reporoller-test repository.
#[tokio::test]
async fn test_search_repositories_by_topic_finds_metadata_repo() -> Result<()> {
    info!("Testing search_repositories_by_topic with reporoller-metadata topic");

    let config = TestConfig::from_env()?;

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(octocrab_client);

    // Search for repositories with reporoller-metadata topic
    let repos = github_client
        .search_repositories_by_topic(&config.test_org, "reporoller-metadata")
        .await?;

    // Verify we found at least one repository
    assert!(
        !repos.is_empty(),
        "Should find at least one repository with reporoller-metadata topic"
    );

    info!(
        "✓ Found {} repository(ies) with reporoller-metadata topic",
        repos.len()
    );

    // Verify the .reporoller-test repository is in the results
    let found_metadata_repo = repos.iter().any(|repo| repo.name() == ".reporoller-test");
    assert!(
        found_metadata_repo,
        ".reporoller-test repository should be found with reporoller-metadata topic"
    );

    info!("✓ Verified .reporoller-test repository has reporoller-metadata topic");

    Ok(())
}

/// Test search_repositories_by_topic with non-existent topic.
///
/// Verifies that searching for a topic that doesn't exist returns an empty result
/// (not an error).
#[tokio::test]
async fn test_search_repositories_by_topic_nonexistent_topic() -> Result<()> {
    info!("Testing search_repositories_by_topic with non-existent topic");

    let config = TestConfig::from_env()?;

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(octocrab_client);

    // Search for a topic that definitely doesn't exist
    let unique_topic = format!("nonexistent-topic-{}", chrono::Utc::now().timestamp());
    let repos = github_client
        .search_repositories_by_topic(&config.test_org, &unique_topic)
        .await?;

    // Verify empty result (not an error)
    assert!(
        repos.is_empty(),
        "Should return empty vector for non-existent topic"
    );

    info!("✓ Non-existent topic correctly returns empty result");

    Ok(())
}

/// Test discover_by_topic successfully finds single metadata repository.
///
/// Verifies that the GitHubMetadataProvider can discover the metadata repository
/// using topic-based search when exactly one repository matches.
#[tokio::test]
async fn test_discover_by_topic_single_match() -> Result<()> {
    info!("Testing discover_by_topic with single matching repository");

    let config = TestConfig::from_env()?;

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(octocrab_client);

    // Create metadata provider with topic-based configuration
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::by_topic("reporoller-metadata"),
    );

    // Discover metadata repository
    let metadata_repo = metadata_provider
        .discover_metadata_repository(&config.test_org)
        .await?;

    // Verify discovery was successful
    assert_eq!(
        metadata_repo.repository_name, ".reporoller-test",
        "Should discover .reporoller-test as metadata repository"
    );

    // Verify discovery method is recorded correctly
    match metadata_repo.discovery_method {
        config_manager::DiscoveryMethod::TopicBased { ref topic } => {
            assert_eq!(topic, "reporoller-metadata", "Should record correct topic");
        }
        _ => panic!("Expected TopicBased discovery method"),
    }

    info!("✓ Topic-based discovery successfully found metadata repository");

    Ok(())
}

/// Test discover_by_topic with non-existent topic returns error.
///
/// Verifies that when no repositories match the topic, discover_by_topic
/// returns MetadataRepositoryNotFound error.
#[tokio::test]
async fn test_discover_by_topic_no_matches() -> Result<()> {
    info!("Testing discover_by_topic with no matching repositories");

    let config = TestConfig::from_env()?;

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(octocrab_client);

    // Create metadata provider with non-existent topic
    let unique_topic = format!("nonexistent-topic-{}", chrono::Utc::now().timestamp());
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::by_topic(&unique_topic),
    );

    // Attempt to discover metadata repository
    let result = metadata_provider
        .discover_metadata_repository(&config.test_org)
        .await;

    // Verify returns MetadataRepositoryNotFound error
    assert!(
        result.is_err(),
        "Should return error when no repositories match topic"
    );

    match result {
        Err(config_manager::ConfigurationError::MetadataRepositoryNotFound { org }) => {
            assert_eq!(
                org, config.test_org,
                "Error should contain organization name"
            );
            info!("✓ Correctly returns MetadataRepositoryNotFound error");
        }
        Err(e) => panic!("Expected MetadataRepositoryNotFound error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }

    Ok(())
}

/// Test end-to-end repository creation with topic-based metadata discovery.
///
/// Verifies that the entire repository creation workflow works correctly when
/// using topic-based metadata repository discovery.
#[tokio::test]
async fn test_end_to_end_creation_with_topic_discovery() -> Result<()> {
    info!("Testing end-to-end repository creation with topic-based discovery");

    let config = TestConfig::from_env()?;
    let org_name = repo_roller_core::OrganizationName::new(&config.test_org)?;
    let repo_name = integration_tests::generate_test_repo_name("test", "topic-discovery-e2e");

    let _test_repo =
        integration_tests::TestRepository::new(repo_name.clone(), config.test_org.clone());

    // Create authentication service
    let auth_service = auth_handler::GitHubAuthService::new(
        config.github_app_id,
        config.github_app_private_key.clone(),
    );

    // Get installation token
    let installation_token = auth_service
        .get_installation_token_for_org(&config.test_org)
        .await?;

    let octocrab_client = github_client::create_token_client(&installation_token)?;
    let github_client = github_client::GitHubClient::new(octocrab_client);

    // Create metadata provider with topic-based configuration
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::by_topic("reporoller-metadata"),
    );

    // Build request for basic template
    let request = repo_roller_core::RepositoryCreationRequestBuilder::new(
        repo_roller_core::RepositoryName::new(&repo_name)?,
        org_name,
        repo_roller_core::TemplateName::new("template-test-basic")?,
    )
    .build();

    // Execute repository creation
    let result = repo_roller_core::create_repository(
        request,
        &metadata_provider,
        &auth_service,
        ".reporoller-test", // Will be discovered via topic, not used directly
    )
    .await;

    // Verify result
    match result {
        Ok(creation_result) => {
            info!(
                "✓ Repository created successfully with topic-based discovery: {}",
                creation_result.repository_url
            );

            // Verify repository exists
            let verification_client = github_client::create_token_client(&installation_token)?;
            let verification_client = github_client::GitHubClient::new(verification_client);

            let repo = verification_client
                .get_repository(&config.test_org, &repo_name)
                .await?;

            info!("✓ Verified repository exists: {}", repo.name());

            Ok(())
        }
        Err(e) => {
            panic!(
                "Repository creation with topic-based discovery failed: {}",
                e
            );
        }
    }
}
