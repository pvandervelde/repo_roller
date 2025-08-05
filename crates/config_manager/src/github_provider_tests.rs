//! Unit tests for GitHub metadata provider functionality.

use super::*;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod discovery_config_tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();

        assert_eq!(
            config.repository_name_pattern,
            Some("{org}-config".to_string())
        );
        assert_eq!(config.metadata_topic, Some("template-metadata".to_string()));
        assert_eq!(config.max_search_results, 100);
        assert_eq!(config.api_timeout_seconds, 30);
    }

    #[test]
    fn test_discovery_config_builder() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-repo-settings")
            .metadata_topic("org-config")
            .max_search_results(50)
            .api_timeout_seconds(60)
            .build();

        assert_eq!(
            config.repository_name_pattern,
            Some("{org}-repo-settings".to_string())
        );
        assert_eq!(config.metadata_topic, Some("org-config".to_string()));
        assert_eq!(config.max_search_results, 50);
        assert_eq!(config.api_timeout_seconds, 60);
    }

    #[test]
    fn test_discovery_config_builder_with_defaults() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-custom")
            .build();

        assert_eq!(
            config.repository_name_pattern,
            Some("{org}-custom".to_string())
        );
        assert_eq!(config.metadata_topic, Some("template-metadata".to_string())); // default
        assert_eq!(config.max_search_results, 100); // default
        assert_eq!(config.api_timeout_seconds, 30); // default
    }

    #[test]
    #[should_panic(expected = "max_search_results must be greater than 0")]
    fn test_discovery_config_builder_invalid_max_results() {
        DiscoveryConfig::builder().max_search_results(0);
    }

    #[test]
    #[should_panic(expected = "api_timeout_seconds must be greater than 0")]
    fn test_discovery_config_builder_invalid_timeout() {
        DiscoveryConfig::builder().api_timeout_seconds(0);
    }

    #[test]
    fn test_generate_repository_name() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-config")
            .build();

        assert_eq!(
            config.generate_repository_name("acme-corp"),
            Some("acme-corp-config".to_string())
        );
    }

    #[test]
    fn test_generate_repository_name_with_case_placeholders() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org_upper}-{org_lower}-CONFIG")
            .build();

        assert_eq!(
            config.generate_repository_name("Acme-Corp"),
            Some("ACME-CORP-acme-corp-CONFIG".to_string())
        );
    }

    #[test]
    fn test_generate_repository_name_no_pattern() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("")
            .build();

        // Empty pattern should still return Some("")
        assert_eq!(
            config.generate_repository_name("acme-corp"),
            Some("".to_string())
        );
    }

    #[test]
    fn test_generate_repository_name_none() {
        let mut config = DiscoveryConfig::default();
        config.repository_name_pattern = None;

        assert_eq!(config.generate_repository_name("acme-corp"), None);
    }

    #[test]
    fn test_has_configuration_based_discovery() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-config")
            .build();

        assert!(config.has_configuration_based_discovery());

        let mut config_none = config.clone();
        config_none.repository_name_pattern = None;
        assert!(!config_none.has_configuration_based_discovery());
    }

    #[test]
    fn test_has_topic_based_discovery() {
        let config = DiscoveryConfig::builder()
            .metadata_topic("template-metadata")
            .build();

        assert!(config.has_topic_based_discovery());

        let mut config_none = config.clone();
        config_none.metadata_topic = None;
        assert!(!config_none.has_topic_based_discovery());
    }

    #[test]
    fn test_discovery_config_serialization() {
        let config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-test")
            .metadata_topic("test-topic")
            .max_search_results(25)
            .api_timeout_seconds(45)
            .build();

        // Test JSON serialization/deserialization
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: DiscoveryConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(config, deserialized);

        // Verify content
        assert!(json.contains("{org}-test"));
        assert!(json.contains("test-topic"));
        assert!(json.contains("25"));
        assert!(json.contains("45"));
    }

    #[test]
    fn test_discovery_config_clone_and_equality() {
        let config1 = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-config")
            .metadata_topic("metadata")
            .build();

        let config2 = config1.clone();
        assert_eq!(config1, config2);

        let config3 = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-different")
            .metadata_topic("metadata")
            .build();

        assert_ne!(config1, config3);
    }
}

#[cfg(test)]
mod github_client_trait_tests {
    use super::*;

    /// Mock implementation of GitHubClientTrait for testing.
    #[derive(Debug)]
    struct MockGitHubClient {
        repositories: HashMap<String, DateTime<Utc>>,
        repository_topics: HashMap<String, Vec<String>>,
        file_contents: HashMap<String, String>,
        directory_contents: HashMap<String, Vec<String>>,
        should_fail: bool,
        failure_message: String,
    }

    impl MockGitHubClient {
        fn new() -> Self {
            Self {
                repositories: HashMap::new(),
                repository_topics: HashMap::new(),
                file_contents: HashMap::new(),
                directory_contents: HashMap::new(),
                should_fail: false,
                failure_message: "Mock failure".to_string(),
            }
        }

        fn with_repository(mut self, org: &str, repo: &str, last_updated: DateTime<Utc>) -> Self {
            self.repositories
                .insert(format!("{}/{}", org, repo), last_updated);
            self
        }

        fn with_repository_topic(mut self, org: &str, repo: &str, topic: &str) -> Self {
            let key = format!("{}/{}", org, repo);
            self.repository_topics
                .entry(key)
                .or_insert_with(Vec::new)
                .push(topic.to_string());
            self
        }

        fn with_file_content(mut self, org: &str, repo: &str, path: &str, content: &str) -> Self {
            let key = format!("{}/{}/{}", org, repo, path);
            self.file_contents.insert(key, content.to_string());
            self
        }

        fn with_directory_content(
            mut self,
            org: &str,
            repo: &str,
            path: &str,
            contents: Vec<&str>,
        ) -> Self {
            let key = format!("{}/{}/{}", org, repo, path);
            self.directory_contents
                .insert(key, contents.into_iter().map(String::from).collect());
            self
        }

        fn with_failure(mut self, message: &str) -> Self {
            self.should_fail = true;
            self.failure_message = message.to_string();
            self
        }
    }

    #[async_trait]
    impl GitHubClientTrait for MockGitHubClient {
        async fn get_repository_info(
            &self,
            org: &str,
            repo: &str,
        ) -> Result<Option<DateTime<Utc>>, String> {
            if self.should_fail {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}", org, repo);
            Ok(self.repositories.get(&key).copied())
        }

        async fn search_repositories_by_topic(
            &self,
            org: &str,
            topic: &str,
            max_results: usize,
        ) -> Result<Vec<String>, String> {
            if self.should_fail {
                return Err(self.failure_message.clone());
            }

            let mut results = Vec::new();
            for (repo_key, topics) in &self.repository_topics {
                if repo_key.starts_with(&format!("{}/", org)) && topics.contains(&topic.to_string())
                {
                    let repo_name = repo_key.split('/').nth(1).unwrap().to_string();
                    results.push(repo_name);
                    if results.len() >= max_results {
                        break;
                    }
                }
            }
            Ok(results)
        }

        async fn get_file_content(
            &self,
            org: &str,
            repo: &str,
            path: &str,
        ) -> Result<Option<String>, String> {
            if self.should_fail {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}/{}", org, repo, path);
            Ok(self.file_contents.get(&key).cloned())
        }

        async fn list_directory(
            &self,
            org: &str,
            repo: &str,
            path: &str,
        ) -> Result<Option<Vec<String>>, String> {
            if self.should_fail {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}/{}", org, repo, path);
            Ok(self.directory_contents.get(&key).cloned())
        }
    }

    #[tokio::test]
    async fn test_mock_github_client_repository_info() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let client = MockGitHubClient::new().with_repository("acme-corp", "acme-config", timestamp);

        let result = client.get_repository_info("acme-corp", "acme-config").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(timestamp));

        let result = client.get_repository_info("acme-corp", "nonexistent").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_mock_github_client_search_by_topic() {
        let client = MockGitHubClient::new()
            .with_repository_topic("acme-corp", "acme-config", "template-metadata")
            .with_repository_topic("acme-corp", "other-repo", "different-topic")
            .with_repository_topic("acme-corp", "another-config", "template-metadata");

        let result = client
            .search_repositories_by_topic("acme-corp", "template-metadata", 10)
            .await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert_eq!(repos.len(), 2);
        assert!(repos.contains(&"acme-config".to_string()));
        assert!(repos.contains(&"another-config".to_string()));
    }

    #[tokio::test]
    async fn test_mock_github_client_search_max_results() {
        let client = MockGitHubClient::new()
            .with_repository_topic("acme-corp", "repo1", "template-metadata")
            .with_repository_topic("acme-corp", "repo2", "template-metadata")
            .with_repository_topic("acme-corp", "repo3", "template-metadata");

        let result = client
            .search_repositories_by_topic("acme-corp", "template-metadata", 2)
            .await;
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert_eq!(repos.len(), 2); // Limited by max_results
    }

    #[tokio::test]
    async fn test_mock_github_client_file_content() {
        let client = MockGitHubClient::new().with_file_content(
            "acme-corp",
            "acme-config",
            "global/defaults.toml",
            "# Global defaults\n[repository]\nenabled = true",
        );

        let result = client
            .get_file_content("acme-corp", "acme-config", "global/defaults.toml")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = client
            .get_file_content("acme-corp", "acme-config", "nonexistent.toml")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_mock_github_client_directory_listing() {
        let client = MockGitHubClient::new().with_directory_content(
            "acme-corp",
            "acme-config",
            "types",
            vec!["library", "service", "documentation"],
        );

        let result = client
            .list_directory("acme-corp", "acme-config", "types")
            .await;
        assert!(result.is_ok());
        let contents = result.unwrap().unwrap();
        assert_eq!(contents.len(), 3);
        assert!(contents.contains(&"library".to_string()));
        assert!(contents.contains(&"service".to_string()));
        assert!(contents.contains(&"documentation".to_string()));
    }

    #[tokio::test]
    async fn test_mock_github_client_failures() {
        let client = MockGitHubClient::new().with_failure("Network timeout");

        let result = client.get_repository_info("acme-corp", "acme-config").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Network timeout");

        let result = client
            .search_repositories_by_topic("acme-corp", "template-metadata", 10)
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Network timeout");

        let result = client
            .get_file_content("acme-corp", "acme-config", "file.toml")
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Network timeout");

        let result = client
            .list_directory("acme-corp", "acme-config", "dir")
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Network timeout");
    }
}

#[cfg(test)]
mod github_metadata_provider_tests {
    use super::*;
    use crate::metadata::MetadataRepositoryProvider;

    #[derive(Debug)]
    struct MockGitHubClient {
        repositories: HashMap<String, DateTime<Utc>>,
        repository_topics: HashMap<String, Vec<String>>,
        file_contents: HashMap<String, String>,
        directory_contents: HashMap<String, Vec<String>>,
        should_fail_operation: Option<String>,
        failure_message: String,
    }

    impl MockGitHubClient {
        fn new() -> Self {
            Self {
                repositories: HashMap::new(),
                repository_topics: HashMap::new(),
                file_contents: HashMap::new(),
                directory_contents: HashMap::new(),
                should_fail_operation: None,
                failure_message: "Mock failure".to_string(),
            }
        }

        fn with_repository(mut self, org: &str, repo: &str, last_updated: DateTime<Utc>) -> Self {
            self.repositories
                .insert(format!("{}/{}", org, repo), last_updated);
            self
        }

        fn with_repository_topic(mut self, org: &str, repo: &str, topic: &str) -> Self {
            let key = format!("{}/{}", org, repo);
            self.repository_topics
                .entry(key)
                .or_insert_with(Vec::new)
                .push(topic.to_string());
            self
        }

        fn with_failure_for_operation(mut self, operation: &str, message: &str) -> Self {
            self.should_fail_operation = Some(operation.to_string());
            self.failure_message = message.to_string();
            self
        }
    }

    #[async_trait]
    impl GitHubClientTrait for MockGitHubClient {
        async fn get_repository_info(
            &self,
            org: &str,
            repo: &str,
        ) -> Result<Option<DateTime<Utc>>, String> {
            if self.should_fail_operation.as_ref() == Some(&"get_repository_info".to_string()) {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}", org, repo);
            Ok(self.repositories.get(&key).copied())
        }

        async fn search_repositories_by_topic(
            &self,
            org: &str,
            topic: &str,
            max_results: usize,
        ) -> Result<Vec<String>, String> {
            if self.should_fail_operation.as_ref()
                == Some(&"search_repositories_by_topic".to_string())
            {
                return Err(self.failure_message.clone());
            }

            let mut results = Vec::new();
            for (repo_key, topics) in &self.repository_topics {
                if repo_key.starts_with(&format!("{}/", org)) && topics.contains(&topic.to_string())
                {
                    let repo_name = repo_key.split('/').nth(1).unwrap().to_string();
                    results.push(repo_name);
                    if results.len() >= max_results {
                        break;
                    }
                }
            }
            Ok(results)
        }

        async fn get_file_content(
            &self,
            org: &str,
            repo: &str,
            path: &str,
        ) -> Result<Option<String>, String> {
            if self.should_fail_operation.as_ref() == Some(&"get_file_content".to_string()) {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}/{}", org, repo, path);
            Ok(self.file_contents.get(&key).cloned())
        }

        async fn list_directory(
            &self,
            org: &str,
            repo: &str,
            path: &str,
        ) -> Result<Option<Vec<String>>, String> {
            if self.should_fail_operation.as_ref() == Some(&"list_directory".to_string()) {
                return Err(self.failure_message.clone());
            }

            let key = format!("{}/{}/{}", org, repo, path);
            Ok(self.directory_contents.get(&key).cloned())
        }
    }

    #[test]
    fn test_github_metadata_provider_creation() {
        let github_client = Arc::new(MockGitHubClient::new());
        let discovery_config = DiscoveryConfig::default();
        let provider = GitHubMetadataProvider::new(github_client.clone(), discovery_config.clone());

        assert_eq!(provider.discovery_config(), &discovery_config);
    }

    #[tokio::test]
    async fn test_discover_metadata_repository_not_found() {
        let github_client = Arc::new(MockGitHubClient::new());
        let discovery_config = DiscoveryConfig::builder()
            .repository_name_pattern("{org}-config")
            .metadata_topic("template-metadata")
            .build();
        let provider = GitHubMetadataProvider::new(github_client, discovery_config);

        let result = provider
            .discover_metadata_repository("nonexistent-org")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::ConfigurationError::RepositoryNotFound {
                organization,
                search_method,
            } => {
                assert_eq!(organization, "nonexistent-org");
                assert!(search_method.contains("configuration-based and topic-based"));
            }
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_trait_bounds_and_thread_safety() {
        // Test that GitHubMetadataProvider implements required trait bounds
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GitHubMetadataProvider>();

        // Test that the trait object is Send + Sync
        fn assert_trait_object_bounds(_: Box<dyn MetadataRepositoryProvider>) {}
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());
        assert_trait_object_bounds(Box::new(provider));
    }

    #[tokio::test]
    async fn test_validate_repository_structure_with_missing_items() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider.validate_repository_structure(&repo).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::ConfigurationError::InvalidRepositoryStructure {
                repository,
                missing_items,
            } => {
                assert_eq!(repository, "test-org/test-repo");
                assert!(missing_items.contains(&"global/ directory".to_string()));
                assert!(missing_items.contains(&"global/defaults.toml file".to_string()));
                assert!(missing_items.contains(&"teams/ directory".to_string()));
                assert!(missing_items.contains(&"types/ directory".to_string()));
            }
            _ => panic!("Expected InvalidRepositoryStructure error"),
        }
    }

    #[tokio::test]
    async fn test_load_global_defaults_file_not_found() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider.load_global_defaults(&repo).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::ConfigurationError::FileNotFound {
                file_path,
                repository,
            } => {
                assert_eq!(file_path, "global/defaults.toml");
                assert_eq!(repository, "test-org/test-repo");
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_load_team_configuration_none_when_file_not_found() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider
            .load_team_configuration(&repo, "platform-team")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_load_repository_type_configuration_none_when_file_not_found() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider
            .load_repository_type_configuration(&repo, "library")
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_available_repository_types_not_implemented() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider.list_available_repository_types(&repo).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<String>::new()); // Empty list when types/ directory doesn't exist
    }

    #[tokio::test]
    async fn test_load_standard_labels_empty_when_file_not_found() {
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        let result = provider.load_standard_labels(&repo).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty()); // Empty map when labels file doesn't exist
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::metadata::MetadataRepositoryProvider;

    #[tokio::test]
    async fn test_discovery_config_integration() {
        // Test that discovery config patterns work correctly with provider
        let github_client = Arc::new(MockGitHubClient::new());

        // Test different pattern styles
        let configs = vec![
            ("{org}-config", "acme-corp", "acme-corp-config"),
            ("{org_lower}-settings", "ACME-CORP", "acme-corp-settings"),
            ("{org_upper}-CONFIG", "acme-corp", "ACME-CORP-CONFIG"),
            ("config-{org}", "test-org", "config-test-org"),
        ];

        for (pattern, org, expected) in configs {
            let config = DiscoveryConfig::builder()
                .repository_name_pattern(pattern)
                .build();

            assert_eq!(
                config.generate_repository_name(org),
                Some(expected.to_string())
            );

            let provider = GitHubMetadataProvider::new(github_client.clone(), config);
            assert!(provider
                .discovery_config()
                .has_configuration_based_discovery());
        }
    }

    #[tokio::test]
    async fn test_provider_configuration_validation() {
        let github_client = Arc::new(MockGitHubClient::new());

        // Test provider with no discovery methods enabled
        let config = DiscoveryConfig {
            repository_name_pattern: None,
            metadata_topic: None,
            max_search_results: 100,
            api_timeout_seconds: 30,
        };

        let provider = GitHubMetadataProvider::new(github_client.clone(), config);
        assert!(!provider
            .discovery_config()
            .has_configuration_based_discovery());
        assert!(!provider.discovery_config().has_topic_based_discovery());

        // Test provider with both methods enabled
        let config = DiscoveryConfig::default();
        let provider = GitHubMetadataProvider::new(github_client, config);
        assert!(provider
            .discovery_config()
            .has_configuration_based_discovery());
        assert!(provider.discovery_config().has_topic_based_discovery());
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
        // Test that methods return appropriate errors when files/directories are missing
        let github_client = Arc::new(MockGitHubClient::new());
        let provider = GitHubMetadataProvider::new(github_client, DiscoveryConfig::default());

        let repo = MetadataRepository::new(
            "test-org".to_string(),
            "test-repo".to_string(),
            DiscoveryMethod::ConfigurationBased {
                repository_name: "test-repo".to_string(),
            },
            Utc::now(),
        );

        // validate_repository_structure should return InvalidRepositoryStructure
        let validate_result = provider.validate_repository_structure(&repo).await;
        assert!(validate_result.is_err());
        assert!(matches!(
            validate_result.unwrap_err(),
            crate::ConfigurationError::InvalidRepositoryStructure { .. }
        ));

        // load_global_defaults should return FileNotFound
        let global_defaults_result = provider.load_global_defaults(&repo).await;
        assert!(global_defaults_result.is_err());
        assert!(matches!(
            global_defaults_result.unwrap_err(),
            crate::ConfigurationError::FileNotFound { .. }
        ));

        // load_team_configuration should return None (not an error)
        let team_result = provider.load_team_configuration(&repo, "team").await;
        assert!(team_result.is_ok());
        assert!(team_result.unwrap().is_none());

        // load_repository_type_configuration should return None (not an error)
        let repo_type_result = provider
            .load_repository_type_configuration(&repo, "type")
            .await;
        assert!(repo_type_result.is_ok());
        assert!(repo_type_result.unwrap().is_none());

        // list_available_repository_types should return empty list (not an error)
        let list_result = provider.list_available_repository_types(&repo).await;
        assert!(list_result.is_ok());
        assert!(list_result.unwrap().is_empty());

        // load_standard_labels should return empty map (not an error)
        let labels_result = provider.load_standard_labels(&repo).await;
        assert!(labels_result.is_ok());
        assert!(labels_result.unwrap().is_empty());
    }

    /// Mock GitHub client for testing different scenarios
    #[derive(Debug)]
    struct MockGitHubClient;

    impl MockGitHubClient {
        /// Create a new mock GitHub client
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl GitHubClientTrait for MockGitHubClient {
        async fn get_repository_info(
            &self,
            _org: &str,
            _repo: &str,
        ) -> Result<Option<DateTime<Utc>>, String> {
            Ok(None) // Repository not found
        }

        async fn search_repositories_by_topic(
            &self,
            _org: &str,
            _topic: &str,
            _max_results: usize,
        ) -> Result<Vec<String>, String> {
            Ok(vec![]) // No repositories found
        }

        async fn get_file_content(
            &self,
            _org: &str,
            _repo: &str,
            _path: &str,
        ) -> Result<Option<String>, String> {
            Ok(None) // File not found
        }

        async fn list_directory(
            &self,
            _org: &str,
            _repo: &str,
            _path: &str,
        ) -> Result<Option<Vec<String>>, String> {
            Ok(None) // Directory not found
        }
    }
}
