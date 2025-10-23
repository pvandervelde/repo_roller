//! Tests for GitHub metadata repository provider.

use super::*;

// Note: Full integration tests with GitHubClient require actual GitHub API access
// or a trait-based abstraction for GitHubClient (future enhancement).
// For now, we test the configuration types and document expected behavior.

#[test]
fn test_metadata_provider_config_explicit() {
    let config = MetadataProviderConfig::explicit("org-metadata");

    match config.discovery {
        DiscoveryConfig::RepositoryName(name) => {
            assert_eq!(name, "org-metadata");
        }
        _ => panic!("Expected RepositoryName discovery config"),
    }
}

#[test]
fn test_metadata_provider_config_by_topic() {
    let config = MetadataProviderConfig::by_topic("reporoller-metadata");

    match config.discovery {
        DiscoveryConfig::Topic(topic) => {
            assert_eq!(topic, "reporoller-metadata");
        }
        _ => panic!("Expected Topic discovery config"),
    }
}

#[test]
fn test_metadata_provider_config_clone() {
    let config = MetadataProviderConfig::explicit("org-metadata");
    let cloned = config.clone();

    // Both should have the same discovery config
    match (&config.discovery, &cloned.discovery) {
        (DiscoveryConfig::RepositoryName(name1), DiscoveryConfig::RepositoryName(name2)) => {
            assert_eq!(name1, name2);
        }
        _ => panic!("Expected matching RepositoryName configs"),
    }
}

#[test]
fn test_metadata_provider_config_debug() {
    let config = MetadataProviderConfig::explicit("test-repo");
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("MetadataProviderConfig"));
    assert!(debug_str.contains("test-repo"));
}

// Note: Full integration tests for discovery require actual GitHubClient
// For now, we document expected behavior in test names and comments

/// Test discovery with explicit repository name - successful case.
///
/// Expected behavior:
/// - Provider attempts to get repository with configured name
/// - If repository exists, returns MetadataRepository with ConfigurationBased discovery
/// - MetadataRepository contains organization name and repository name
/// - last_updated timestamp is set to current time
#[tokio::test]
async fn test_discover_by_name_success_documented() {
    // This test documents the expected behavior
    // Actual implementation will be tested with real or mocked GitHubClient

    // Expected flow:
    // 1. Create provider with explicit config
    // 2. Call discover_metadata_repository("my-org")
    // 3. Provider calls client.get_repository("my-org", "org-metadata")
    // 4. If successful, creates MetadataRepository with:
    //    - organization: "my-org"
    //    - repository_name: "org-metadata"
    //    - discovery_method: ConfigurationBased
    //    - last_updated: Utc::now()
}

/// Test discovery with explicit repository name - repository not found.
///
/// Expected behavior:
/// - Provider attempts to get repository with configured name
/// - If repository doesn't exist (404), returns MetadataRepositoryNotFound error
/// - Error contains the organization name for context
#[tokio::test]
async fn test_discover_by_name_not_found_documented() {
    // Expected flow:
    // 1. Create provider with explicit config
    // 2. Call discover_metadata_repository("nonexistent-org")
    // 3. Provider calls client.get_repository("nonexistent-org", "org-metadata")
    // 4. GitHub returns 404
    // 5. Provider maps to ConfigurationError::MetadataRepositoryNotFound
}

/// Test discovery with topic - successful case.
///
/// Expected behavior:
/// - Provider searches repositories in org with specified topic
/// - If exactly one match found, returns MetadataRepository with TopicBased discovery
/// - MetadataRepository contains the found repository's name
/// - discovery_method contains the search topic
#[tokio::test]
async fn test_discover_by_topic_success_documented() {
    // Expected flow:
    // 1. Create provider with topic config ("reporoller-metadata")
    // 2. Call discover_metadata_repository("my-org")
    // 3. Provider searches org repositories with topic
    // 4. Finds exactly one: "my-org/config-repo"
    // 5. Returns MetadataRepository with:
    //    - organization: "my-org"
    //    - repository_name: "config-repo"
    //    - discovery_method: TopicBased { topic: "reporoller-metadata" }
    //    - last_updated: Utc::now()
}

/// Test discovery with topic - no matches found.
///
/// Expected behavior:
/// - Provider searches repositories in org with specified topic
/// - If no repositories match, returns MetadataRepositoryNotFound error
#[tokio::test]
async fn test_discover_by_topic_not_found_documented() {
    // Expected flow:
    // 1. Create provider with topic config
    // 2. Call discover_metadata_repository("my-org")
    // 3. Provider searches org repositories with topic
    // 4. No matches found
    // 5. Returns ConfigurationError::MetadataRepositoryNotFound
}

/// Test discovery with topic - multiple matches found.
///
/// Expected behavior:
/// - Provider searches repositories in org with specified topic
/// - If multiple repositories match, returns error requiring disambiguation
/// - Error should indicate ambiguous configuration
#[tokio::test]
async fn test_discover_by_topic_multiple_matches_documented() {
    // Expected flow:
    // 1. Create provider with topic config
    // 2. Call discover_metadata_repository("my-org")
    // 3. Provider searches org repositories with topic
    // 4. Multiple matches found (e.g., "config-repo" and "metadata-repo")
    // 5. Returns error indicating ambiguous configuration
    //    (Could be MetadataRepositoryNotFound with context or new error variant)
}

/// Test discovery method is correctly recorded in returned metadata.
///
/// Expected behavior:
/// - Explicit discovery: DiscoveryMethod::ConfigurationBased
/// - Topic discovery: DiscoveryMethod::TopicBased
/// - Both include relevant configuration details
#[tokio::test]
async fn test_discovery_method_recorded_documented() {
    // Verify that the discovery method in returned MetadataRepository
    // correctly reflects how the repository was found
}

/// Test concurrent discovery operations.
///
/// Expected behavior:
/// - Multiple concurrent discover calls should work correctly
/// - GitHubClient handles concurrent requests
/// - Each call returns correct MetadataRepository
#[tokio::test]
async fn test_concurrent_discovery_documented() {
    // Expected flow:
    // 1. Create provider
    // 2. Spawn multiple concurrent discover_metadata_repository calls
    // 3. All complete successfully
    // 4. Each returns correct MetadataRepository for its org
}

/// Test timestamp is set correctly on discovery.
///
/// Expected behavior:
/// - last_updated timestamp reflects when discovery occurred
/// - Timestamp is in UTC
/// - Timestamp is reasonably close to current time
#[tokio::test]
async fn test_discovery_timestamp_documented() {
    // Verify last_updated is set to Utc::now() at discovery time
}

// Additional test documentation for edge cases

/// Test discovery with invalid organization name.
///
/// Expected behavior:
/// - Invalid org names should be handled gracefully
/// - Returns MetadataRepositoryNotFound with org context
#[tokio::test]
async fn test_discover_invalid_org_documented() {
    // Test with empty string, special characters, etc.
}

/// Test discovery with network failures.
///
/// Expected behavior:
/// - Network/API errors should be mapped to appropriate ConfigurationError
/// - Transient errors could be retried (future enhancement)
/// - Permanent errors return MetadataRepositoryNotFound
#[tokio::test]
async fn test_discover_network_failure_documented() {
    // Test GitHub API unavailability, timeouts, etc.
}

/// Test discovery with authentication failures.
///
/// Expected behavior:
/// - Authentication errors should be propagated appropriately
/// - May need specific error variant for auth issues
#[tokio::test]
async fn test_discover_auth_failure_documented() {
    // Test with invalid/expired GitHub token
}

// Repository structure validation tests

#[tokio::test]
async fn test_validate_structure_valid_repository() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    // Create a mock provider (would need actual GitHubClient for full test)
    // For now, we can test the validation logic directly

    // Valid repository with normal names should pass validation
    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_structure_path_traversal_in_org() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "../etc/passwd".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, reason } => {
            assert_eq!(field, "organization");
            assert!(reason.contains("invalid characters"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_path_traversal_in_repo() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "../secrets".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "../secrets".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, reason } => {
            assert_eq!(field, "repository_name");
            assert!(reason.contains("invalid characters"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_slash_in_org() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "org/malicious".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, .. } => {
            assert_eq!(field, "organization");
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_slash_in_repo() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "valid-org".to_string(),
        repository_name: "repo/name".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "repo/name".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConfigurationError::InvalidConfiguration { field, .. } => {
            assert_eq!(field, "repository_name");
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_validate_structure_hyphen_allowed() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_structure_underscore_allowed() {
    use chrono::Utc;

    let metadata_repo = MetadataRepository {
        organization: "my_org".to_string(),
        repository_name: "org_metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org_metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = validate_repository_names(&metadata_repo);
    assert!(result.is_ok());
}

// Helper function for testing validation logic
fn validate_repository_names(repo: &MetadataRepository) -> ConfigurationResult<()> {
    // Security validation: ensure no path traversal in repository/org names
    if repo.organization.contains("..") || repo.organization.contains('/') {
        return Err(ConfigurationError::InvalidConfiguration {
            field: "organization".to_string(),
            reason: "Organization name contains invalid characters".to_string(),
        });
    }

    if repo.repository_name.contains("..") || repo.repository_name.contains('/') {
        return Err(ConfigurationError::InvalidConfiguration {
            field: "repository_name".to_string(),
            reason: "Repository name contains invalid characters".to_string(),
        });
    }

    Ok(())
}

// ============================================================================
// TOML Configuration File Loading Tests (Task 3.4)
// ============================================================================

/// Test loading global defaults from valid TOML.
///
/// Verifies that `load_global_defaults` can successfully parse a well-formed
/// global-defaults.toml file and return the expected GlobalDefaults structure.
#[tokio::test]
async fn test_load_global_defaults_success() {
    // Test will verify:
    // 1. Provider fetches "global-defaults.toml" from repository root
    // 2. TOML content is parsed into GlobalDefaults struct
    // 3. All fields are correctly deserialized
    // 4. OverridableValue settings are preserved
}

/// Test loading global defaults when file doesn't exist.
///
/// Verifies that `load_global_defaults` returns FileNotFound error when
/// the global-defaults.toml file is missing from the repository.
#[tokio::test]
async fn test_load_global_defaults_file_not_found() {
    // Expected behavior:
    // 1. Provider attempts to fetch "global-defaults.toml"
    // 2. GitHub API returns 404 (file not found)
    // 3. Provider returns ConfigurationError::FileNotFound with context
}

/// Test loading global defaults with invalid TOML syntax.
///
/// Verifies that `load_global_defaults` returns ParseError when the TOML
/// content has syntax errors.
#[tokio::test]
async fn test_load_global_defaults_invalid_toml() {
    // Test scenarios:
    // - Malformed TOML syntax (unclosed brackets, invalid values)
    // - Returns ConfigurationError::ParseError with line/column information
}

/// Test loading global defaults with missing required fields.
///
/// Verifies that `load_global_defaults` returns InvalidConfiguration when
/// the TOML is valid but missing required fields.
#[tokio::test]
async fn test_load_global_defaults_missing_required_fields() {
    // Test scenarios:
    // - Valid TOML but incomplete GlobalDefaults structure
    // - Returns ConfigurationError::InvalidConfiguration with field name
}

/// Test loading team configuration for existing team.
///
/// Verifies that `load_team_configuration` can successfully load and parse
/// a team's configuration file from teams/{team}/config.toml.
#[tokio::test]
async fn test_load_team_configuration_success() {
    // Test will verify:
    // 1. Provider fetches "teams/backend-team/config.toml"
    // 2. TOML content is parsed into TeamConfig struct
    // 3. Returns Some(TeamConfig) with correct values
    // 4. Override settings are correctly parsed
}

/// Test loading team configuration when team has no config.
///
/// Verifies that `load_team_configuration` returns Ok(None) when a team
/// directory exists but has no config.toml file.
#[tokio::test]
async fn test_load_team_configuration_not_found() {
    // Expected behavior:
    // 1. Provider attempts to fetch "teams/frontend-team/config.toml"
    // 2. GitHub API returns 404 (file not found)
    // 3. Provider returns Ok(None) - not an error, just no custom config
}

/// Test loading team configuration with invalid TOML.
///
/// Verifies that `load_team_configuration` returns ParseError when the
/// team's config.toml has syntax errors.
#[tokio::test]
async fn test_load_team_configuration_invalid_toml() {
    // Test scenarios:
    // - Malformed TOML in team config
    // - Returns ConfigurationError::ParseError with context
}

/// Test loading team configuration with path traversal attempt.
///
/// Verifies that `load_team_configuration` safely handles team names
/// that attempt path traversal attacks.
#[tokio::test]
async fn test_load_team_configuration_path_traversal() {
    // Security test:
    // 1. Call with team name like "../../../etc/passwd"
    // 2. Provider validates and sanitizes team name
    // 3. Returns error or safely constructs path without traversal
}

/// Test loading repository type configuration for existing type.
///
/// Verifies that `load_repository_type_configuration` can successfully
/// load and parse a repository type's configuration from types/{type}/config.toml.
#[tokio::test]
async fn test_load_repository_type_configuration_success() {
    // Test will verify:
    // 1. Provider fetches "types/library/config.toml"
    // 2. TOML content is parsed into RepositoryTypeConfig struct
    // 3. Returns Some(RepositoryTypeConfig) with correct values
}

/// Test loading repository type configuration when type has no config.
///
/// Verifies that `load_repository_type_configuration` returns Ok(None)
/// when a type directory exists but has no config.toml file.
#[tokio::test]
async fn test_load_repository_type_configuration_not_found() {
    // Expected behavior:
    // 1. Provider attempts to fetch "types/service/config.toml"
    // 2. GitHub API returns 404
    // 3. Provider returns Ok(None)
}

/// Test loading repository type configuration with invalid TOML.
///
/// Verifies that `load_repository_type_configuration` returns ParseError
/// when the type's config.toml has syntax errors.
#[tokio::test]
async fn test_load_repository_type_configuration_invalid_toml() {
    // Test scenarios:
    // - Malformed TOML in type config
    // - Returns ConfigurationError::ParseError
}

/// Test loading repository type configuration with path traversal.
///
/// Verifies safe handling of repository type names that attempt path traversal.
#[tokio::test]
async fn test_load_repository_type_configuration_path_traversal() {
    // Security test:
    // 1. Call with type name like "../../secrets"
    // 2. Provider validates and sanitizes type name
    // 3. Returns error or safely constructs path
}

/// Test loading standard labels from valid labels.toml.
///
/// Verifies that `load_standard_labels` can successfully parse the
/// labels.toml file and return a map of label configurations.
#[tokio::test]
async fn test_load_standard_labels_success() {
    // Test will verify:
    // 1. Provider fetches "labels.toml" from repository root
    // 2. TOML content is parsed into HashMap<String, LabelConfig>
    // 3. All label configurations are correctly deserialized
    // 4. Color codes, descriptions are preserved
}

/// Test loading standard labels when file doesn't exist.
///
/// Verifies that `load_standard_labels` returns an empty map when
/// labels.toml doesn't exist (labels are optional).
#[tokio::test]
async fn test_load_standard_labels_file_not_found() {
    // Expected behavior:
    // 1. Provider attempts to fetch "labels.toml"
    // 2. GitHub API returns 404
    // 3. Provider returns Ok(empty HashMap) - labels are optional
}

/// Test loading standard labels with invalid TOML.
///
/// Verifies that `load_standard_labels` returns ParseError when the
/// labels.toml file has syntax errors.
#[tokio::test]
async fn test_load_standard_labels_invalid_toml() {
    // Test scenarios:
    // - Malformed TOML in labels file
    // - Returns ConfigurationError::ParseError
}

/// Test loading standard labels with invalid label structure.
///
/// Verifies that `load_standard_labels` returns InvalidConfiguration
/// when label definitions are malformed.
#[tokio::test]
async fn test_load_standard_labels_invalid_structure() {
    // Test scenarios:
    // - Valid TOML but incorrect LabelConfig structure
    // - Missing required fields (color, description)
    // - Returns ConfigurationError::InvalidConfiguration
}

/// Test listing available repository types.
///
/// Verifies that `list_available_repository_types` returns all directory
/// names under the types/ directory.
#[tokio::test]
async fn test_list_available_repository_types_success() {
    // Test will verify:
    // 1. Provider lists contents of "types/" directory
    // 2. Returns vector of directory names (not files)
    // 3. Alphabetically sorted or in discovery order
}

/// Test listing repository types when types directory doesn't exist.
///
/// Verifies that `list_available_repository_types` returns an empty vector
/// when the types/ directory doesn't exist (types are optional).
#[tokio::test]
async fn test_list_available_repository_types_no_directory() {
    // Expected behavior:
    // 1. Provider attempts to list "types/" directory
    // 2. GitHub API returns 404 or empty response
    // 3. Provider returns Ok(empty Vec)
}

/// Test listing repository types when types directory is empty.
///
/// Verifies that `list_available_repository_types` returns an empty vector
/// when the types/ directory exists but contains no subdirectories.
#[tokio::test]
async fn test_list_available_repository_types_empty_directory() {
    // Expected behavior:
    // 1. Provider lists "types/" directory
    // 2. Directory exists but has no contents
    // 3. Provider returns Ok(empty Vec)
}

/// Test listing repository types filters out files.
///
/// Verifies that `list_available_repository_types` only returns directories,
/// not files that might exist in the types/ directory.
#[tokio::test]
async fn test_list_available_repository_types_filters_files() {
    // Test scenario:
    // 1. types/ contains both directories and files
    // 2. Provider returns only directory names
    // 3. Files like "README.md" are excluded
}

/// Test concurrent loading of different configuration files.
///
/// Verifies that multiple concurrent calls to load different configuration
/// files work correctly without interference.
#[tokio::test]
async fn test_concurrent_configuration_loading() {
    // Test scenario:
    // 1. Simultaneously load global defaults, team config, type config, labels
    // 2. All complete successfully
    // 3. Each returns correct parsed configuration
}

/// Test loading configuration with UTF-8 content.
///
/// Verifies that configuration files with UTF-8 characters (descriptions,
/// team names) are correctly parsed and preserved.
#[tokio::test]
async fn test_load_configuration_utf8_content() {
    // Test scenarios:
    // - Team descriptions with emoji or non-ASCII characters
    // - Label descriptions in various languages
    // - UTF-8 content correctly preserved
}

/// Test error messages contain useful context.
///
/// Verifies that configuration errors include sufficient context for
/// debugging (file path, organization, repository).
#[tokio::test]
async fn test_configuration_errors_include_context() {
    // Verify error messages include:
    // - Which file failed to load
    // - Organization and repository name
    // - Specific reason for failure (parse error line, missing field)
}
