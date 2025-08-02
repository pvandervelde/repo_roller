//! Unit tests for metadata repository provider functionality.

use super::*;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[cfg(test)]
mod metadata_repository_tests {
    use super::*;

    #[test]
    fn test_metadata_repository_creation() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let discovery_method = DiscoveryMethod::ConfigurationBased {
            repository_name: "acme-config".to_string(),
        };

        let repo = MetadataRepository::new(
            "acme-corp".to_string(),
            "acme-config".to_string(),
            discovery_method.clone(),
            timestamp,
        );

        assert_eq!(repo.organization, "acme-corp");
        assert_eq!(repo.repository_name, "acme-config");
        assert_eq!(repo.discovery_method, discovery_method);
        assert_eq!(repo.last_updated, timestamp);
    }

    #[test]
    fn test_metadata_repository_full_name() {
        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::TopicBased {
                topic: "template-metadata".to_string(),
            },
            Utc::now(),
        );

        assert_eq!(repo.full_name(), "test-org/test-repo");
    }

    #[test]
    fn test_metadata_repository_serialization() {
        let timestamp = Utc.with_ymd_and_hms(2024, 2, 20, 14, 45, 30).unwrap();
        let repo = MetadataRepository::new(
            "serialization-test".to_string(),
            "config-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "config-repo".to_string(),
            },
            timestamp,
        );

        // Test JSON serialization
        let json = serde_json::to_string(&repo).expect("Failed to serialize to JSON");
        let deserialized: MetadataRepository =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        assert_eq!(repo, deserialized);

        // Test that all required fields are present in serialized form
        assert!(json.contains("serialization-test"));
        assert!(json.contains("config-repo"));
        assert!(json.contains("ConfigurationBased"));
        assert!(json.contains("2024-02-20T14:45:30Z"));
    }

    #[test]
    fn test_metadata_repository_clone_and_equality() {
        let original = MetadataRepository::new(
            "clone-test".to_string(),
            "clone-repo".to_string(),
            DiscoveryMethod::TopicBased {
                topic: "org-metadata".to_string(),
            },
            Utc::now(),
        );

        let cloned = original.clone();
        assert_eq!(original, cloned);

        // Verify deep clone by checking nested data
        assert_eq!(original.discovery_method, cloned.discovery_method);
        assert_eq!(original.last_updated, cloned.last_updated);
    }
}

#[cfg(test)]
mod discovery_method_tests {
    use super::*;

    #[test]
    fn test_configuration_based_discovery_method() {
        let method = DiscoveryMethod::ConfigurationBased {
            repository_name: "explicit-config".to_string(),
        };

        assert!(!method.requires_search());
        assert_eq!(
            method.description(),
            "configuration-based (repository: explicit-config)"
        );
    }

    #[test]
    fn test_topic_based_discovery_method() {
        let method = DiscoveryMethod::TopicBased {
            topic: "template-metadata".to_string(),
        };

        assert!(method.requires_search());
        assert_eq!(
            method.description(),
            "topic-based (topic: template-metadata)"
        );
    }

    #[test]
    fn test_discovery_method_serialization() {
        // Test ConfigurationBased serialization
        let config_method = DiscoveryMethod::ConfigurationBased {
            repository_name: "test-config".to_string(),
        };
        let json = serde_json::to_string(&config_method).expect("Failed to serialize");
        let deserialized: DiscoveryMethod =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(config_method, deserialized);

        // Test TopicBased serialization
        let topic_method = DiscoveryMethod::TopicBased {
            topic: "metadata-topic".to_string(),
        };
        let json = serde_json::to_string(&topic_method).expect("Failed to serialize");
        let deserialized: DiscoveryMethod =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(topic_method, deserialized);
    }

    #[test]
    fn test_discovery_method_equality() {
        let method1 = DiscoveryMethod::ConfigurationBased {
            repository_name: "same-name".to_string(),
        };
        let method2 = DiscoveryMethod::ConfigurationBased {
            repository_name: "same-name".to_string(),
        };
        let method3 = DiscoveryMethod::ConfigurationBased {
            repository_name: "different-name".to_string(),
        };

        assert_eq!(method1, method2);
        assert_ne!(method1, method3);

        let topic_method = DiscoveryMethod::TopicBased {
            topic: "same-name".to_string(),
        };
        assert_ne!(method1, topic_method); // Different variants are not equal
    }

    #[test]
    fn test_discovery_method_clone() {
        let original = DiscoveryMethod::TopicBased {
            topic: "clone-test-topic".to_string(),
        };
        let cloned = original.clone();

        assert_eq!(original, cloned);

        // Verify it's a deep clone by modifying description and checking they're independent
        assert_eq!(original.description(), cloned.description());
    }
}

#[cfg(test)]
mod mock_provider_tests {
    use super::*;
    use crate::organization::{RepositoryTypeConfig, TeamConfig};
    use crate::settings::GlobalDefaults;
    use crate::LabelConfig;
    use async_trait::async_trait;

    /// Mock implementation for testing the MetadataRepositoryProvider trait.
    ///
    /// This implementation provides controlled responses for testing various scenarios
    /// including success cases, error conditions, and edge cases.
    struct MockMetadataProvider {
        should_fail_discovery: bool,
        should_fail_validation: bool,
        discovery_method: DiscoveryMethod,
        available_types: Vec<String>,
    }

    impl MockMetadataProvider {
        fn new() -> Self {
            Self {
                should_fail_discovery: false,
                should_fail_validation: false,
                discovery_method: DiscoveryMethod::ConfigurationBased {
                    repository_name: "mock-config".to_string(),
                },
                available_types: vec!["library".to_string(), "service".to_string()],
            }
        }

        fn with_discovery_failure() -> Self {
            Self {
                should_fail_discovery: true,
                ..Self::new()
            }
        }

        fn with_validation_failure() -> Self {
            Self {
                should_fail_validation: true,
                ..Self::new()
            }
        }

        fn with_topic_based_discovery() -> Self {
            Self {
                discovery_method: DiscoveryMethod::TopicBased {
                    topic: "template-metadata".to_string(),
                },
                ..Self::new()
            }
        }
    }

    #[async_trait]
    impl MetadataRepositoryProvider for MockMetadataProvider {
        async fn discover_metadata_repository(
            &self,
            org: &str,
        ) -> MetadataResult<MetadataRepository> {
            if self.should_fail_discovery {
                return Err(crate::ConfigurationError::RepositoryNotFound {
                    organization: org.to_string(),
                    search_method: "mock discovery".to_string(),
                });
            }

            Ok(MetadataRepository::new(
                org.to_string(),
                "mock-config-repo".to_string(),
                self.discovery_method.clone(),
                Utc::now(),
            ))
        }

        async fn validate_repository_structure(
            &self,
            _repo: &MetadataRepository,
        ) -> MetadataResult<()> {
            if self.should_fail_validation {
                return Err(crate::ConfigurationError::InvalidRepositoryStructure {
                    repository: "mock-config-repo".to_string(),
                    missing_items: vec!["global/defaults.toml".to_string()],
                });
            }
            Ok(())
        }

        async fn load_global_defaults(
            &self,
            _repo: &MetadataRepository,
        ) -> MetadataResult<GlobalDefaults> {
            // TODO: implement - return minimal valid GlobalDefaults
            Ok(GlobalDefaults::default())
        }

        async fn load_team_configuration(
            &self,
            _repo: &MetadataRepository,
            team: &str,
        ) -> MetadataResult<Option<TeamConfig>> {
            // Return None for unknown teams, Some(TeamConfig) for known teams
            if team == "platform-team" {
                Ok(Some(TeamConfig::default()))
            } else {
                Ok(None)
            }
        }

        async fn load_repository_type_configuration(
            &self,
            _repo: &MetadataRepository,
            repo_type: &str,
        ) -> MetadataResult<Option<RepositoryTypeConfig>> {
            // Return Some(RepositoryTypeConfig) for available types, None otherwise
            if self.available_types.contains(&repo_type.to_string()) {
                Ok(Some(RepositoryTypeConfig::default()))
            } else {
                Ok(None)
            }
        }

        async fn list_available_repository_types(
            &self,
            _repo: &MetadataRepository,
        ) -> MetadataResult<Vec<String>> {
            Ok(self.available_types.clone())
        }

        async fn load_standard_labels(
            &self,
            _repo: &MetadataRepository,
        ) -> MetadataResult<HashMap<String, LabelConfig>> {
            let mut labels = HashMap::new();
            labels.insert(
                "bug".to_string(),
                LabelConfig {
                    name: "bug".to_string(),
                    color: "d73a4a".to_string(),
                    description: Some("Something isn't working".to_string()),
                },
            );
            labels.insert(
                "enhancement".to_string(),
                LabelConfig {
                    name: "enhancement".to_string(),
                    color: "a2eeef".to_string(),
                    description: Some("New feature or request".to_string()),
                },
            );
            Ok(labels)
        }
    }

    #[tokio::test]
    async fn test_successful_repository_discovery() {
        let provider = MockMetadataProvider::new();
        let result = provider.discover_metadata_repository("test-org").await;

        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.organization, "test-org");
        assert_eq!(repo.repository_name, "mock-config-repo");
    }

    #[tokio::test]
    async fn test_repository_discovery_failure() {
        let provider = MockMetadataProvider::with_discovery_failure();
        let result = provider.discover_metadata_repository("test-org").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::ConfigurationError::RepositoryNotFound { organization, .. } => {
                assert_eq!(organization, "test-org");
            }
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_successful_repository_validation() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.validate_repository_structure(&repo).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repository_validation_failure() {
        let provider = MockMetadataProvider::with_validation_failure();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.validate_repository_structure(&repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::ConfigurationError::InvalidRepositoryStructure { missing_items, .. } => {
                assert!(missing_items.contains(&"global/defaults.toml".to_string()));
            }
            _ => panic!("Expected InvalidRepositoryStructure error"),
        }
    }

    #[tokio::test]
    async fn test_load_global_defaults() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.load_global_defaults(&repo).await;

        assert!(result.is_ok());
        // Verify we got a valid GlobalDefaults instance
        let _defaults = result.unwrap();
    }

    #[tokio::test]
    async fn test_load_team_configuration_exists() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider
            .load_team_configuration(&repo, "platform-team")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_load_team_configuration_not_exists() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider
            .load_team_configuration(&repo, "nonexistent-team")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_load_repository_type_configuration_exists() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider
            .load_repository_type_configuration(&repo, "library")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_load_repository_type_configuration_not_exists() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider
            .load_repository_type_configuration(&repo, "nonexistent-type")
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_available_repository_types() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.list_available_repository_types(&repo).await;

        assert!(result.is_ok());
        let types = result.unwrap();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"library".to_string()));
        assert!(types.contains(&"service".to_string()));
    }

    #[tokio::test]
    async fn test_load_standard_labels() {
        let provider = MockMetadataProvider::new();
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.load_standard_labels(&repo).await;

        assert!(result.is_ok());
        let labels = result.unwrap();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains_key("bug"));
        assert!(labels.contains_key("enhancement"));
    }

    #[tokio::test]
    async fn test_topic_based_discovery_method() {
        let provider = MockMetadataProvider::with_topic_based_discovery();
        let result = provider.discover_metadata_repository("test-org").await;

        assert!(result.is_ok());
        let repo = result.unwrap();
        match repo.discovery_method {
            DiscoveryMethod::TopicBased { topic } => {
                assert_eq!(topic, "template-metadata");
            }
            _ => panic!("Expected TopicBased discovery method"),
        }
    }

    #[tokio::test]
    async fn test_provider_trait_bounds() {
        // Test that MockMetadataProvider implements required trait bounds
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockMetadataProvider>();

        // Test that the trait object is Send + Sync
        fn assert_trait_object_bounds(_: Box<dyn MetadataRepositoryProvider>) {}
        let provider = MockMetadataProvider::new();
        assert_trait_object_bounds(Box::new(provider));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_workflow_success() {
        // Test a complete workflow: discover -> validate -> load configurations
        use async_trait::async_trait;
        use std::collections::HashMap;

        struct SuccessfulProvider;

        #[async_trait]
        impl MetadataRepositoryProvider for SuccessfulProvider {
            async fn discover_metadata_repository(
                &self,
                org: &str,
            ) -> MetadataResult<MetadataRepository> {
                Ok(MetadataRepository::new(
                    org.to_string(),
                    format!("{}-config", org),
                    DiscoveryMethod::ConfigurationBased {
                        repository_name: format!("{}-config", org),
                    },
                    Utc::now(),
                ))
            }

            async fn validate_repository_structure(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<()> {
                Ok(())
            }

            async fn load_global_defaults(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<GlobalDefaults> {
                Ok(GlobalDefaults::default())
            }

            async fn load_team_configuration(
                &self,
                _repo: &MetadataRepository,
                _team: &str,
            ) -> MetadataResult<Option<TeamConfig>> {
                Ok(Some(TeamConfig::default()))
            }

            async fn load_repository_type_configuration(
                &self,
                _repo: &MetadataRepository,
                _repo_type: &str,
            ) -> MetadataResult<Option<RepositoryTypeConfig>> {
                Ok(Some(RepositoryTypeConfig::default()))
            }

            async fn list_available_repository_types(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<Vec<String>> {
                Ok(vec!["library".to_string(), "service".to_string()])
            }

            async fn load_standard_labels(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<HashMap<String, crate::LabelConfig>> {
                Ok(HashMap::new())
            }
        }

        let provider = SuccessfulProvider;

        // Step 1: Discover repository
        let repo = provider
            .discover_metadata_repository("acme-corp")
            .await
            .unwrap();
        assert_eq!(repo.organization, "acme-corp");
        assert_eq!(repo.repository_name, "acme-corp-config");

        // Step 2: Validate structure
        provider.validate_repository_structure(&repo).await.unwrap();

        // Step 3: Load various configurations
        let _global_defaults = provider.load_global_defaults(&repo).await.unwrap();
        let team_config = provider
            .load_team_configuration(&repo, "platform-team")
            .await
            .unwrap();
        assert!(team_config.is_some());

        let type_config = provider
            .load_repository_type_configuration(&repo, "library")
            .await
            .unwrap();
        assert!(type_config.is_some());

        let types = provider
            .list_available_repository_types(&repo)
            .await
            .unwrap();
        assert_eq!(types.len(), 2);

        let labels = provider.load_standard_labels(&repo).await.unwrap();
        assert!(labels.is_empty());
    }

    #[tokio::test]
    async fn test_complete_workflow_with_failures() {
        // Test workflow with various failure points
        use async_trait::async_trait;
        use std::collections::HashMap;

        struct FailingProvider {
            fail_at_step: usize,
        }

        #[async_trait]
        impl MetadataRepositoryProvider for FailingProvider {
            async fn discover_metadata_repository(
                &self,
                org: &str,
            ) -> MetadataResult<MetadataRepository> {
                if self.fail_at_step == 1 {
                    return Err(crate::ConfigurationError::RepositoryNotFound {
                        organization: org.to_string(),
                        search_method: "test failure".to_string(),
                    });
                }

                Ok(MetadataRepository::new(
                    org.to_string(),
                    "test-config".to_string(),
                    DiscoveryMethod::ConfigurationBased {
                        repository_name: "test-config".to_string(),
                    },
                    Utc::now(),
                ))
            }

            async fn validate_repository_structure(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<()> {
                if self.fail_at_step == 2 {
                    return Err(crate::ConfigurationError::InvalidRepositoryStructure {
                        repository: "test-config".to_string(),
                        missing_items: vec!["global/defaults.toml".to_string()],
                    });
                }
                Ok(())
            }

            async fn load_global_defaults(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<GlobalDefaults> {
                if self.fail_at_step == 3 {
                    return Err(crate::ConfigurationError::FileNotFound {
                        file_path: "global/defaults.toml".to_string(),
                        repository: "test-config".to_string(),
                    });
                }
                Ok(GlobalDefaults::default())
            }

            async fn load_team_configuration(
                &self,
                _repo: &MetadataRepository,
                _team: &str,
            ) -> MetadataResult<Option<TeamConfig>> {
                Ok(None)
            }

            async fn load_repository_type_configuration(
                &self,
                _repo: &MetadataRepository,
                _repo_type: &str,
            ) -> MetadataResult<Option<RepositoryTypeConfig>> {
                Ok(None)
            }

            async fn list_available_repository_types(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<Vec<String>> {
                Ok(vec![])
            }

            async fn load_standard_labels(
                &self,
                _repo: &MetadataRepository,
            ) -> MetadataResult<HashMap<String, crate::LabelConfig>> {
                Ok(HashMap::new())
            }
        }

        // Test failure at discovery step
        let provider = FailingProvider { fail_at_step: 1 };
        let result = provider.discover_metadata_repository("test-org").await;
        assert!(result.is_err());

        // Test failure at validation step
        let provider = FailingProvider { fail_at_step: 2 };
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        let result = provider.validate_repository_structure(&repo).await;
        assert!(result.is_err());

        // Test failure at global defaults loading
        let provider = FailingProvider { fail_at_step: 3 };
        let repo = provider
            .discover_metadata_repository("test-org")
            .await
            .unwrap();
        provider.validate_repository_structure(&repo).await.unwrap();
        let result = provider.load_global_defaults(&repo).await;
        assert!(result.is_err());
    }
}
