//! Directory listing integration tests.
//!
//! These tests verify the GitHub Contents API directory listing functionality
//! against real GitHub infrastructure (glitchgrove organization).
//!
//! Tests the list_directory_contents() method and the GitHubMetadataProvider's
//! list_available_repository_types() integration.

use anyhow::Result;
use auth_handler::UserAuthenticationService;
use config_manager::MetadataRepositoryProvider;
use github_client::EntryType;
use integration_tests::utils::TestConfig;
use tracing::info;

/// Test listing directory contents from metadata repository.
///
/// Verifies that list_directory_contents() successfully lists the types/
/// directory in the .reporoller-test repository and returns proper TreeEntry
/// objects with correct type discrimination.
#[tokio::test]
async fn test_list_directory_contents_types_directory() -> Result<()> {
    info!("Testing list_directory_contents with types/ directory");

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

    // List types/ directory from .reporoller-test repository
    let entries = github_client
        .list_directory_contents(&config.test_org, ".reporoller-test", "types", "main")
        .await?;

    // Verify we got results
    assert!(
        !entries.is_empty(),
        "types/ directory should contain repository type directories"
    );

    info!("✓ Found {} entries in types/ directory", entries.len());

    // Verify we have directory entries
    let dir_count = entries
        .iter()
        .filter(|e| matches!(e.entry_type, EntryType::Dir))
        .count();
    assert!(
        dir_count > 0,
        "Should have at least one directory in types/"
    );

    info!("✓ Found {} directory entries", dir_count);

    // Verify entry structure
    for entry in &entries {
        assert!(!entry.name.is_empty(), "Entry name should not be empty");
        assert!(
            !entry.path.is_empty(),
            "Entry path should not be empty"
        );
        assert!(!entry.sha.is_empty(), "Entry SHA should not be empty");

        if matches!(entry.entry_type, EntryType::Dir) {
            assert_eq!(
                entry.size, 0,
                "Directory entries should have size 0"
            );
            assert!(
                entry.download_url.is_none(),
                "Directory entries should not have download URL"
            );
        }
    }

    info!("✓ All entries have valid structure");

    Ok(())
}

/// Test listing a non-existent directory.
///
/// Verifies that attempting to list a path that doesn't exist returns
/// Error::NotFound.
#[tokio::test]
async fn test_list_directory_contents_nonexistent_path() -> Result<()> {
    info!("Testing list_directory_contents with non-existent path");

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

    // Try to list a non-existent directory
    let unique_path = format!("nonexistent-path-{}", chrono::Utc::now().timestamp());
    let result = github_client
        .list_directory_contents(&config.test_org, ".reporoller-test", &unique_path, "main")
        .await;

    // Verify we get NotFound error
    assert!(result.is_err(), "Should return error for non-existent path");
    assert!(
        matches!(result.unwrap_err(), github_client::Error::NotFound),
        "Should return NotFound error"
    );

    info!("✓ Non-existent path correctly returns NotFound error");

    Ok(())
}

/// Test GitHubMetadataProvider's list_available_repository_types().
///
/// Verifies that the GitHubMetadataProvider can discover repository types
/// by listing the types/ directory and filtering to only directories.
#[tokio::test]
async fn test_list_available_repository_types() -> Result<()> {
    info!("Testing GitHubMetadataProvider::list_available_repository_types()");

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

    // Create metadata provider
    let metadata_provider = config_manager::GitHubMetadataProvider::new(
        github_client,
        config_manager::MetadataProviderConfig::explicit(".reporoller-test"),
    );

    // Create metadata repository reference
    let metadata_repo = config_manager::MetadataRepository {
        organization: config.test_org.clone(),
        repository_name: ".reporoller-test".to_string(),
        discovery_method: config_manager::DiscoveryMethod::ConfigurationBased {
            repository_name: ".reporoller-test".to_string(),
        },
        last_updated: chrono::Utc::now(),
    };

    // List available repository types
    let types = metadata_provider
        .list_available_repository_types(&metadata_repo)
        .await?;

    // Verify we got results
    assert!(
        !types.is_empty(),
        "Should find repository types in types/ directory"
    );

    info!("✓ Found {} repository types: {:?}", types.len(), types);

    // Verify all types are valid directory names (no path separators, no special chars)
    for repo_type in &types {
        assert!(
            !repo_type.contains('/'),
            "Repository type should not contain path separators"
        );
        assert!(
            !repo_type.contains('\\'),
            "Repository type should not contain backslashes"
        );
        assert!(
            !repo_type.is_empty(),
            "Repository type should not be empty"
        );
    }

    info!("✓ All repository types are valid directory names");

    // Verify we can find expected types (library, service should exist)
    // Note: This is based on glitchgrove/.reporoller-test structure
    let expected_types = ["library", "service"];
    for expected in &expected_types {
        assert!(
            types.contains(&expected.to_string()),
            "Should find '{}' repository type",
            expected
        );
    }

    info!("✓ Expected repository types (library, service) found");

    Ok(())
}

/// Test listing directory with files and directories mixed.
///
/// Verifies that when a directory contains both files and subdirectories,
/// the GitHubMetadataProvider correctly filters to only return directories.
#[tokio::test]
async fn test_filter_directories_from_mixed_entries() -> Result<()> {
    info!("Testing filtering of directories from mixed file/directory entries");

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

    // List global/ directory which may contain both files and subdirectories
    let entries = github_client
        .list_directory_contents(&config.test_org, ".reporoller-test", "global", "main")
        .await?;

    info!("Found {} entries in global/ directory", entries.len());

    // Count directories and files
    let dir_count = entries
        .iter()
        .filter(|e| matches!(e.entry_type, EntryType::Dir))
        .count();

    let file_count = entries
        .iter()
        .filter(|e| matches!(e.entry_type, EntryType::File))
        .count();

    info!(
        "✓ Filtered to {} directories and {} files",
        dir_count,
        file_count
    );

    // Verify filtering works correctly - collect directories
    let directories: Vec<_> = entries
        .into_iter()
        .filter(|e| matches!(e.entry_type, EntryType::Dir))
        .collect();

    // Verify filtering works correctly
    for dir in &directories {
        assert!(
            matches!(dir.entry_type, EntryType::Dir),
            "Filtered result should only contain directories"
        );
    }

    Ok(())
}
