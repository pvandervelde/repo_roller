//! Tests for metadata repository provider interface and types.

use super::*;
use chrono::Utc;

#[test]
fn test_discovery_method_configuration_based() {
    let method = DiscoveryMethod::ConfigurationBased {
        repository_name: "org-metadata".to_string(),
    };

    match method {
        DiscoveryMethod::ConfigurationBased { repository_name } => {
            assert_eq!(repository_name, "org-metadata");
        }
        _ => panic!("Expected ConfigurationBased variant"),
    }
}

#[test]
fn test_discovery_method_topic_based() {
    let method = DiscoveryMethod::TopicBased {
        topic: "reporoller-metadata".to_string(),
    };

    match method {
        DiscoveryMethod::TopicBased { topic } => {
            assert_eq!(topic, "reporoller-metadata");
        }
        _ => panic!("Expected TopicBased variant"),
    }
}

#[test]
fn test_discovery_method_equality() {
    let method1 = DiscoveryMethod::ConfigurationBased {
        repository_name: "org-metadata".to_string(),
    };
    let method2 = DiscoveryMethod::ConfigurationBased {
        repository_name: "org-metadata".to_string(),
    };
    let method3 = DiscoveryMethod::ConfigurationBased {
        repository_name: "different-repo".to_string(),
    };

    assert_eq!(method1, method2);
    assert_ne!(method1, method3);
}

#[test]
fn test_discovery_method_clone() {
    let method = DiscoveryMethod::TopicBased {
        topic: "reporoller-config".to_string(),
    };
    let cloned = method.clone();

    assert_eq!(method, cloned);
}

#[test]
fn test_metadata_repository_creation() {
    let now = Utc::now();
    let metadata_repo = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: now,
    };

    assert_eq!(metadata_repo.organization, "my-org");
    assert_eq!(metadata_repo.repository_name, "org-metadata");
    assert_eq!(metadata_repo.last_updated, now);
}

#[test]
fn test_metadata_repository_with_topic_discovery() {
    let metadata_repo = MetadataRepository {
        organization: "test-org".to_string(),
        repository_name: "config-repo".to_string(),
        discovery_method: DiscoveryMethod::TopicBased {
            topic: "reporoller-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    match metadata_repo.discovery_method {
        DiscoveryMethod::TopicBased { topic } => {
            assert_eq!(topic, "reporoller-metadata");
        }
        _ => panic!("Expected TopicBased discovery method"),
    }
}

#[test]
fn test_metadata_repository_equality() {
    let now = Utc::now();

    let repo1 = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: now,
    };

    let repo2 = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: now,
    };

    assert_eq!(repo1, repo2);
}

#[test]
fn test_metadata_repository_clone() {
    let original = MetadataRepository {
        organization: "my-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.organization, cloned.organization);
    assert_eq!(original.repository_name, cloned.repository_name);
}

#[test]
fn test_metadata_repository_debug_format() {
    let metadata_repo = MetadataRepository {
        organization: "debug-org".to_string(),
        repository_name: "debug-repo".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "debug-repo".to_string(),
        },
        last_updated: Utc::now(),
    };

    let debug_string = format!("{:?}", metadata_repo);
    assert!(debug_string.contains("MetadataRepository"));
    assert!(debug_string.contains("debug-org"));
    assert!(debug_string.contains("debug-repo"));
}

// Mock implementation for contract testing
#[cfg(test)]
mod mock_provider {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock metadata repository provider for testing.
    ///
    /// This implementation allows tests to verify the contract without
    /// requiring actual GitHub API access.
    pub struct MockMetadataProvider {
        pub discovered_repos: Arc<Mutex<Vec<MetadataRepository>>>,
        pub should_fail_discovery: bool,
        pub should_fail_load: bool,
    }

    impl MockMetadataProvider {
        pub fn new() -> Self {
            Self {
                discovered_repos: Arc::new(Mutex::new(Vec::new())),
                should_fail_discovery: false,
                should_fail_load: false,
            }
        }

        pub fn with_failure(mut self, fail_discovery: bool, fail_load: bool) -> Self {
            self.should_fail_discovery = fail_discovery;
            self.should_fail_load = fail_load;
            self
        }
    }

    #[async_trait]
    impl MetadataRepositoryProvider for MockMetadataProvider {
        async fn discover_metadata_repository(
            &self,
            org: &str,
        ) -> ConfigurationResult<MetadataRepository> {
            if self.should_fail_discovery {
                return Err(crate::ConfigurationError::MetadataRepositoryNotFound {
                    org: org.to_string(),
                });
            }

            let repo = MetadataRepository {
                organization: org.to_string(),
                repository_name: "org-metadata".to_string(),
                discovery_method: DiscoveryMethod::ConfigurationBased {
                    repository_name: "org-metadata".to_string(),
                },
                last_updated: Utc::now(),
            };

            self.discovered_repos.lock().await.push(repo.clone());
            Ok(repo)
        }

        async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
            Ok(vec![])
        }

        async fn load_template_configuration(
            &self,
            _org: &str,
            _template_name: &str,
        ) -> ConfigurationResult<crate::template_config::TemplateConfig> {
            Err(crate::ConfigurationError::FileNotFound {
                path: "template.toml".to_string(),
            })
        }

        async fn load_global_defaults(
            &self,
            _repo: &MetadataRepository,
        ) -> ConfigurationResult<GlobalDefaults> {
            if self.should_fail_load {
                return Err(crate::ConfigurationError::FileNotFound {
                    path: "global-defaults.toml".to_string(),
                });
            }

            Ok(GlobalDefaults {
                repository: None,
                pull_requests: None,
                branch_protection: None,
                actions: None,
                push: None,
                webhooks: None,
                github_apps: None,
                environments: None,
                custom_properties: None,
                repository_visibility: None,
            })
        }

        async fn load_team_configuration(
            &self,
            _repo: &MetadataRepository,
            _team: &str,
        ) -> ConfigurationResult<Option<TeamConfig>> {
            if self.should_fail_load {
                return Err(crate::ConfigurationError::ParseError {
                    reason: "Invalid TOML".to_string(),
                });
            }

            Ok(None)
        }

        async fn load_repository_type_configuration(
            &self,
            _repo: &MetadataRepository,
            _repo_type: &str,
        ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
            if self.should_fail_load {
                return Err(crate::ConfigurationError::ParseError {
                    reason: "Invalid TOML".to_string(),
                });
            }

            Ok(None)
        }

        async fn load_standard_labels(
            &self,
            _repo: &MetadataRepository,
        ) -> ConfigurationResult<HashMap<String, LabelConfig>> {
            if self.should_fail_load {
                return Err(crate::ConfigurationError::ParseError {
                    reason: "Invalid TOML".to_string(),
                });
            }

            Ok(HashMap::new())
        }

        async fn list_available_repository_types(
            &self,
            _repo: &MetadataRepository,
        ) -> ConfigurationResult<Vec<String>> {
            Ok(vec!["library".to_string(), "service".to_string()])
        }

        async fn validate_repository_structure(
            &self,
            _repo: &MetadataRepository,
        ) -> ConfigurationResult<()> {
            if self.should_fail_load {
                return Err(crate::ConfigurationError::InvalidConfiguration {
                    field: "structure".to_string(),
                    reason: "Missing global-defaults.toml".to_string(),
                });
            }

            Ok(())
        }
    }
}

// Contract tests for trait implementation
#[tokio::test]
async fn test_provider_discover_success() {
    let provider = mock_provider::MockMetadataProvider::new();

    let result = provider.discover_metadata_repository("test-org").await;

    assert!(result.is_ok());
    let metadata_repo = result.unwrap();
    assert_eq!(metadata_repo.organization, "test-org");
    assert_eq!(metadata_repo.repository_name, "org-metadata");
}

#[tokio::test]
async fn test_provider_discover_failure() {
    let provider = mock_provider::MockMetadataProvider::new().with_failure(true, false);

    let result = provider
        .discover_metadata_repository("nonexistent-org")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::ConfigurationError::MetadataRepositoryNotFound { org } => {
            assert_eq!(org, "nonexistent-org");
        }
        _ => panic!("Expected MetadataRepositoryNotFound error"),
    }
}

#[tokio::test]
async fn test_provider_load_global_defaults_success() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider.load_global_defaults(&metadata_repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_provider_load_global_defaults_failure() {
    let provider = mock_provider::MockMetadataProvider::new().with_failure(false, true);
    let metadata_repo = MetadataRepository {
        organization: "test-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = provider.load_global_defaults(&metadata_repo).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::ConfigurationError::FileNotFound { path } => {
            assert_eq!(path, "global-defaults.toml");
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[tokio::test]
async fn test_provider_load_team_configuration_none() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider
        .load_team_configuration(&metadata_repo, "nonexistent-team")
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_provider_load_team_configuration_parse_error() {
    let provider = mock_provider::MockMetadataProvider::new().with_failure(false, true);
    let metadata_repo = MetadataRepository {
        organization: "test-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = provider
        .load_team_configuration(&metadata_repo, "backend-team")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_provider_load_repository_type_configuration_none() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider
        .load_repository_type_configuration(&metadata_repo, "custom-type")
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_provider_load_standard_labels_empty() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider.load_standard_labels(&metadata_repo).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_provider_list_repository_types() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider
        .list_available_repository_types(&metadata_repo)
        .await;

    assert!(result.is_ok());
    let types = result.unwrap();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"library".to_string()));
    assert!(types.contains(&"service".to_string()));
}

#[tokio::test]
async fn test_provider_validate_repository_structure_success() {
    let provider = mock_provider::MockMetadataProvider::new();
    let metadata_repo = provider
        .discover_metadata_repository("test-org")
        .await
        .unwrap();

    let result = provider.validate_repository_structure(&metadata_repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_provider_validate_repository_structure_failure() {
    let provider = mock_provider::MockMetadataProvider::new().with_failure(false, true);
    let metadata_repo = MetadataRepository {
        organization: "test-org".to_string(),
        repository_name: "org-metadata".to_string(),
        discovery_method: DiscoveryMethod::ConfigurationBased {
            repository_name: "org-metadata".to_string(),
        },
        last_updated: Utc::now(),
    };

    let result = provider.validate_repository_structure(&metadata_repo).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::ConfigurationError::InvalidConfiguration { field, reason } => {
            assert_eq!(field, "structure");
            assert!(reason.contains("global-defaults.toml"));
        }
        _ => panic!("Expected InvalidConfiguration error"),
    }
}

#[tokio::test]
async fn test_provider_concurrent_access() {
    use std::sync::Arc;

    let provider = Arc::new(mock_provider::MockMetadataProvider::new());

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for i in 0..5 {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            let org = format!("org-{}", i);
            provider_clone.discover_metadata_repository(&org).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all discoveries were recorded
    let discovered = provider.discovered_repos.lock().await;
    assert_eq!(discovered.len(), 5);
}
