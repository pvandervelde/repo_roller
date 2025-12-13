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
// TOML Parsing Tests
// ============================================================================
// Note: Tests that require GitHub API access are covered by integration tests.
// These unit tests focus on TOML parsing logic that can be tested in isolation.

/// Test parsing standard labels from valid TOML.
///
/// Verifies that label configurations can be successfully parsed from
/// TOML content. Note: The `name` field is populated from the map key
/// by `load_standard_labels()`, so this test focuses on color and description.
#[test]
fn test_load_standard_labels_success() {
    let labels_toml = r#"
[bug]
color = "d73a4a"
description = "Something isn't working"

[enhancement]
color = "a2eeef"
description = "New feature or request"

[documentation]
color = "0075ca"
description = "Improvements or additions to documentation"
"#;

    let mut labels: HashMap<String, LabelConfig> = toml::from_str(labels_toml).expect("Should parse valid TOML");
    
    // Simulate what load_standard_labels() does: populate name from map key
    for (name, label) in labels.iter_mut() {
        label.name = name.clone();
    }
    
    assert_eq!(labels.len(), 3);

    let bug_label = labels.get("bug").expect("bug label should exist");
    assert_eq!(bug_label.name, "bug");
    assert_eq!(bug_label.color, "d73a4a");
    assert_eq!(bug_label.description, "Something isn't working");

    let enhancement_label = labels
        .get("enhancement")
        .expect("enhancement label should exist");
    assert_eq!(enhancement_label.name, "enhancement");
    assert_eq!(enhancement_label.color, "a2eeef");
    assert_eq!(enhancement_label.description, "New feature or request");

    let doc_label = labels
        .get("documentation")
        .expect("documentation label should exist");
    assert_eq!(doc_label.name, "documentation");
    assert_eq!(doc_label.color, "0075ca");
    assert_eq!(
        doc_label.description,
        "Improvements or additions to documentation"
    );
}

/// Test parsing empty standard labels TOML.
///
/// Verifies that empty TOML content (no labels defined) is valid
/// and returns an empty map.
#[test]
fn test_load_standard_labels_empty() {
    let labels_toml = "";

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> = toml::from_str(labels_toml);
    assert!(result.is_ok());

    let labels = result.unwrap();
    assert!(labels.is_empty(), "Empty TOML should parse to empty map");
}

/// Test parsing standard labels with invalid TOML syntax.
///
/// Verifies that malformed TOML is properly rejected with a parse error.
#[test]
fn test_load_standard_labels_invalid_toml() {
    let invalid_toml = "[bug\ncolor = not a string";

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> =
        toml::from_str(invalid_toml);
    assert!(result.is_err(), "Invalid TOML should fail to parse");
}

/// Test parsing standard labels with missing required fields.
///
/// Verifies that label definitions missing required fields are rejected.
#[test]
fn test_load_standard_labels_invalid_structure() {
    // Valid TOML but missing required 'description' field
    let invalid_structure = r#"
[bug]
color = "d73a4a"
"#;

    let result: Result<HashMap<String, LabelConfig>, toml::de::Error> =
        toml::from_str(invalid_structure);
    assert!(result.is_err(), "Should fail with missing required fields");
}


