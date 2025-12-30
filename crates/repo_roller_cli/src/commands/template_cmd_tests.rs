//! Tests for template commands.
//!
//! These tests verify the template inspection and validation command implementations.

use super::*;
use async_trait::async_trait;
use config_manager::{
    settings::{BranchProtectionSettings, PullRequestSettings, RepositorySettings},
    ConfigurationError, ConfigurationResult, GlobalDefaults, LabelConfig, MetadataRepository,
    RepositoryTypeConfig, TeamConfig, TemplateConfig, TemplateMetadata, TemplateVariable,
};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Mock MetadataRepositoryProvider for Testing
// ============================================================================

/// Mock provider for testing template operations.
struct MockMetadataProvider {
    templates: Vec<String>,
    template_configs: HashMap<String, Result<TemplateConfig, ConfigurationError>>,
    available_types: Vec<String>,
}

impl MockMetadataProvider {
    fn new() -> Self {
        Self {
            templates: Vec::new(),
            template_configs: HashMap::new(),
            available_types: Vec::new(),
        }
    }

    fn with_templates(mut self, templates: Vec<String>) -> Self {
        self.templates = templates;
        self
    }

    fn with_template_config(mut self, name: String, config: TemplateConfig) -> Self {
        self.template_configs.insert(name, Ok(config));
        self
    }

    fn with_template_error(mut self, name: String, error: ConfigurationError) -> Self {
        self.template_configs.insert(name, Err(error));
        self
    }

    #[allow(dead_code)] // Will be used for validation tests in task 4.3
    fn with_available_types(mut self, types: Vec<String>) -> Self {
        self.available_types = types;
        self
    }
}

#[async_trait]
impl MetadataRepositoryProvider for MockMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        _org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        unimplemented!("Not needed for these tests")
    }

    async fn load_global_defaults(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<GlobalDefaults> {
        unimplemented!("Not needed for these tests")
    }

    async fn load_team_configuration(
        &self,
        _repo: &MetadataRepository,
        _team: &str,
    ) -> ConfigurationResult<Option<TeamConfig>> {
        unimplemented!("Not needed for these tests")
    }

    async fn load_repository_type_configuration(
        &self,
        _repo: &MetadataRepository,
        _repo_type: &str,
    ) -> ConfigurationResult<Option<RepositoryTypeConfig>> {
        unimplemented!("Not needed for these tests")
    }

    async fn load_standard_labels(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<HashMap<String, LabelConfig>> {
        unimplemented!("Not needed for these tests")
    }

    async fn list_available_repository_types(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<String>> {
        Ok(self.available_types.clone())
    }

    async fn validate_repository_structure(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<()> {
        unimplemented!("Not needed for these tests")
    }

    async fn list_templates(&self, _org: &str) -> ConfigurationResult<Vec<String>> {
        Ok(self.templates.clone())
    }

    async fn load_template_configuration(
        &self,
        _org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        self.template_configs
            .get(template_name)
            .cloned()
            .unwrap_or_else(|| {
                Err(ConfigurationError::TemplateNotFound {
                    org: "test-org".to_string(),
                    template: template_name.to_string(),
                })
            })
    }
}

// ============================================================================
// Helper Functions for Test Data
// ============================================================================

fn create_minimal_template_config(name: &str) -> TemplateConfig {
    TemplateConfig {
        template: TemplateMetadata {
            name: name.to_string(),
            description: format!("{} template", name),
            author: "Test Author".to_string(),
            tags: vec![],
        },
        repository_type: None,
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
    }
}

fn create_full_template_config(name: &str) -> TemplateConfig {
    let mut variables = HashMap::new();
    variables.insert(
        "project_name".to_string(),
        TemplateVariable {
            description: "Project name".to_string(),
            required: Some(true),
            default: None,
            example: Some("my-project".to_string()),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
        },
    );
    variables.insert(
        "service_port".to_string(),
        TemplateVariable {
            description: "Service port".to_string(),
            required: Some(false),
            default: Some("8080".to_string()),
            example: Some("3000".to_string()),
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
        },
    );

    TemplateConfig {
        template: TemplateMetadata {
            name: name.to_string(),
            description: format!("{} template with full config", name),
            author: "Platform Team".to_string(),
            tags: vec!["rust".to_string(), "service".to_string()],
        },
        repository_type: Some(config_manager::RepositoryTypeSpec {
            repository_type: "service".to_string(),
            policy: config_manager::RepositoryTypePolicy::Fixed,
        }),
        variables: Some(variables),
        repository: Some(RepositorySettings::default()),
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
    }
}

// ============================================================================
// Translation Function Tests
// ============================================================================

#[test]
fn test_template_config_to_info_minimal() {
    let config = create_minimal_template_config("minimal-template");

    let info = template_config_to_info(config);

    assert_eq!(info.name, "minimal-template");
    assert_eq!(info.description, "minimal-template template");
    assert_eq!(info.author, "Test Author");
    assert_eq!(info.tags.len(), 0);
    assert!(info.repository_type.is_none());
    assert_eq!(info.variables.len(), 0);
    assert_eq!(info.configuration_sections, 0);
}

#[test]
fn test_template_config_to_info_full() {
    let config = create_full_template_config("full-template");

    let info = template_config_to_info(config);

    assert_eq!(info.name, "full-template");
    assert_eq!(info.description, "full-template template with full config");
    assert_eq!(info.author, "Platform Team");
    assert_eq!(info.tags, vec!["rust", "service"]);

    // Check repository type
    assert!(info.repository_type.is_some());
    let repo_type = info.repository_type.unwrap();
    assert_eq!(repo_type.type_name, "service");
    assert_eq!(repo_type.policy, "fixed");

    // Check variables
    assert_eq!(info.variables.len(), 2);
    let project_var = info.variables.iter().find(|v| v.name == "project_name");
    assert!(project_var.is_some());
    let project_var = project_var.unwrap();
    assert_eq!(project_var.description, Some("Project name".to_string()));
    assert!(project_var.required);
    assert_eq!(project_var.default_value, None);
    assert_eq!(project_var.example, Some("my-project".to_string()));

    // Check configuration sections count (repository is present)
    assert_eq!(info.configuration_sections, 1);
}

#[test]
fn test_template_config_to_info_repository_type_preferable() {
    let mut config = create_minimal_template_config("preferable-template");
    config.repository_type = Some(config_manager::RepositoryTypeSpec {
        repository_type: "library".to_string(),
        policy: config_manager::RepositoryTypePolicy::Preferable,
    });

    let info = template_config_to_info(config);

    assert!(info.repository_type.is_some());
    let repo_type = info.repository_type.unwrap();
    assert_eq!(repo_type.type_name, "library");
    assert_eq!(repo_type.policy, "preferable");
}

#[test]
fn test_template_config_to_info_configuration_sections_count() {
    let mut config = create_minimal_template_config("multi-section");
    config.repository = Some(RepositorySettings::default());
    config.pull_requests = Some(PullRequestSettings::default());
    config.branch_protection = Some(BranchProtectionSettings::default());
    config.labels = Some(vec![]);
    config.webhooks = Some(vec![]);
    config.environments = Some(vec![]);
    config.github_apps = Some(vec![]);

    let info = template_config_to_info(config);

    // All 7 sections present
    assert_eq!(info.configuration_sections, 7);
}

// ============================================================================
// list_templates() Tests
// ============================================================================

#[tokio::test]
async fn test_list_templates_empty() {
    let provider = Arc::new(MockMetadataProvider::new().with_templates(vec![]));

    let result = list_templates("test-org", provider).await;

    assert!(result.is_ok());
    let templates = result.unwrap();
    assert_eq!(templates.len(), 0);
}

#[tokio::test]
async fn test_list_templates_single() {
    let config = create_minimal_template_config("template1");
    let provider = Arc::new(
        MockMetadataProvider::new()
            .with_templates(vec!["template1".to_string()])
            .with_template_config("template1".to_string(), config),
    );

    let result = list_templates("test-org", provider).await;

    assert!(result.is_ok());
    let templates = result.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "template1");
}

#[tokio::test]
async fn test_list_templates_multiple() {
    let config1 = create_minimal_template_config("template1");
    let config2 = create_minimal_template_config("template2");
    let config3 = create_full_template_config("template3");

    let provider = Arc::new(
        MockMetadataProvider::new()
            .with_templates(vec![
                "template1".to_string(),
                "template2".to_string(),
                "template3".to_string(),
            ])
            .with_template_config("template1".to_string(), config1)
            .with_template_config("template2".to_string(), config2)
            .with_template_config("template3".to_string(), config3),
    );

    let result = list_templates("test-org", provider).await;

    assert!(result.is_ok());
    let templates = result.unwrap();
    assert_eq!(templates.len(), 3);
    assert_eq!(templates[0].name, "template1");
    assert_eq!(templates[1].name, "template2");
    assert_eq!(templates[2].name, "template3");
}

#[tokio::test]
async fn test_list_templates_with_load_error_skips_template() {
    // Template1 loads successfully, template2 fails, template3 loads successfully
    let config1 = create_minimal_template_config("template1");
    let config3 = create_minimal_template_config("template3");

    let provider = Arc::new(
        MockMetadataProvider::new()
            .with_templates(vec![
                "template1".to_string(),
                "template2".to_string(),
                "template3".to_string(),
            ])
            .with_template_config("template1".to_string(), config1)
            .with_template_error(
                "template2".to_string(),
                ConfigurationError::TemplateConfigurationMissing {
                    org: "test-org".to_string(),
                    template: "template2".to_string(),
                },
            )
            .with_template_config("template3".to_string(), config3),
    );

    let result = list_templates("test-org", provider).await;

    // Should succeed but only include template1 and template3
    assert!(result.is_ok());
    let templates = result.unwrap();
    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].name, "template1");
    assert_eq!(templates[1].name, "template3");
}

// ============================================================================
// get_template_info() Tests
// ============================================================================

#[tokio::test]
async fn test_get_template_info_success() {
    let config = create_minimal_template_config("rust-library");
    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("rust-library".to_string(), config),
    );

    let result = get_template_info("test-org", "rust-library", provider).await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.name, "rust-library");
    assert_eq!(info.description, "rust-library template");
    assert_eq!(info.author, "Test Author");
}

#[tokio::test]
async fn test_get_template_info_full_config() {
    let config = create_full_template_config("rust-service");
    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("rust-service".to_string(), config),
    );

    let result = get_template_info("test-org", "rust-service", provider).await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.name, "rust-service");
    assert_eq!(info.tags, vec!["rust", "service"]);
    assert!(info.repository_type.is_some());
    assert_eq!(info.variables.len(), 2);
    assert_eq!(info.configuration_sections, 1);
}

#[tokio::test]
async fn test_get_template_info_template_not_found() {
    let provider = Arc::new(MockMetadataProvider::new());

    let result = get_template_info("test-org", "nonexistent", provider).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        Error::Config(msg) => {
            // Accept either the formatted message or the original error text
            assert!(
                msg.contains("Template")
                    || msg.contains("not found")
                    || msg.contains("'nonexistent'"),
                "Expected template not found error, got: {}",
                msg
            );
        }
        _ => panic!("Expected Config error, got {:?}", err),
    }
}

#[tokio::test]
async fn test_get_template_info_configuration_missing() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "incomplete-template".to_string(),
        ConfigurationError::TemplateConfigurationMissing {
            org: "test-org".to_string(),
            template: "incomplete-template".to_string(),
        },
    ));

    let result = get_template_info("test-org", "incomplete-template", provider).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        Error::Config(msg) => {
            // Accept either the formatted message or the original error text
            assert!(
                msg.contains("configuration")
                    || msg.contains("missing")
                    || msg.contains(".reporoller/template.toml"),
                "Expected configuration missing error, got: {}",
                msg
            );
        }
        _ => panic!("Expected Config error, got {:?}", err),
    }
}

#[tokio::test]
async fn test_get_template_info_parse_error() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "invalid-template".to_string(),
        ConfigurationError::ParseError {
            reason: "invalid TOML syntax in .reporoller/template.toml".to_string(),
        },
    ));

    let result = get_template_info("test-org", "invalid-template", provider).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        Error::Config(msg) => {
            assert!(msg.contains("Parse") || msg.contains("invalid"));
        }
        _ => panic!("Expected Config error, got {:?}", err),
    }
}

// ============================================================================
// validate_template() Tests
// ============================================================================

#[tokio::test]
async fn test_validate_template_success_minimal() {
    let config = create_minimal_template_config("valid-template");
    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("valid-template".to_string(), config),
    );

    let result = validate_template("test-org", "valid-template", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert_eq!(validation.template_name, "valid-template");
    assert!(validation.valid);
    assert_eq!(validation.issues.len(), 0);
}

#[tokio::test]
async fn test_validate_template_success_full() {
    let config = create_full_template_config("full-template");
    let provider = Arc::new(
        MockMetadataProvider::new()
            .with_template_config("full-template".to_string(), config)
            .with_available_types(vec!["service".to_string()]),
    );

    let result = validate_template("test-org", "full-template", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert_eq!(validation.template_name, "full-template");
    assert!(validation.valid);
    assert_eq!(validation.issues.len(), 0);
}

#[tokio::test]
async fn test_validate_template_empty_description_warning() {
    let mut config = create_minimal_template_config("warning-template");
    config.template.description = "".to_string();

    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("warning-template".to_string(), config),
    );

    let result = validate_template("test-org", "warning-template", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    // Empty description should be a warning, not an error
    assert!(validation.valid);
    assert!(validation.warnings.len() > 0);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.message.contains("description")));
}

#[tokio::test]
async fn test_validate_template_no_tags_warning() {
    let config = create_minimal_template_config("no-tags");
    // minimal config already has no tags

    let provider =
        Arc::new(MockMetadataProvider::new().with_template_config("no-tags".to_string(), config));

    let result = validate_template("test-org", "no-tags", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(validation.valid);
    assert!(validation.warnings.len() > 0);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.message.contains("tags") || w.message.contains("categorization")));
}

#[tokio::test]
async fn test_validate_template_no_variables_warning() {
    let config = create_minimal_template_config("no-vars");
    // minimal config has no variables

    let provider =
        Arc::new(MockMetadataProvider::new().with_template_config("no-vars".to_string(), config));

    let result = validate_template("test-org", "no-vars", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(validation.valid);
    assert!(validation.warnings.len() > 0);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.message.contains("variable")));
}

#[tokio::test]
async fn test_validate_template_invalid_variable_name() {
    let mut config = create_minimal_template_config("invalid-vars");
    let mut variables = HashMap::new();
    variables.insert(
        "invalid name".to_string(), // Space in name - invalid
        TemplateVariable {
            description: "Invalid variable name".to_string(),
            required: Some(true),
            default: None,
            example: None,
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
        },
    );
    config.variables = Some(variables);

    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("invalid-vars".to_string(), config),
    );

    let result = validate_template("test-org", "invalid-vars", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.valid); // Should be invalid
    assert!(validation.issues.len() > 0);
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("variable name") || i.message.contains("identifier")));
}

#[tokio::test]
async fn test_validate_template_required_variable_with_default() {
    let mut config = create_minimal_template_config("contradictory-vars");
    let mut variables = HashMap::new();
    variables.insert(
        "project_name".to_string(),
        TemplateVariable {
            description: "Project name".to_string(),
            required: Some(true),
            default: Some("default-value".to_string()), // Contradiction!
            example: None,
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
        },
    );
    config.variables = Some(variables);

    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("contradictory-vars".to_string(), config),
    );

    let result = validate_template("test-org", "contradictory-vars", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.valid);
    assert!(validation.issues.len() > 0);
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("required") && i.message.contains("default")));
}

#[tokio::test]
async fn test_validate_template_required_variable_without_example_warning() {
    let mut config = create_minimal_template_config("no-example");
    let mut variables = HashMap::new();
    variables.insert(
        "service_name".to_string(),
        TemplateVariable {
            description: "Service name".to_string(),
            required: Some(true),
            default: None,
            example: None, // No example - warning
            pattern: None,
            min_length: None,
            max_length: None,
            options: None,
        },
    );
    config.variables = Some(variables);

    let provider = Arc::new(
        MockMetadataProvider::new().with_template_config("no-example".to_string(), config),
    );

    let result = validate_template("test-org", "no-example", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(validation.valid);
    assert!(validation.warnings.len() > 0);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.message.contains("example")));
}

#[tokio::test]
async fn test_validate_template_invalid_repository_type() {
    let mut config = create_minimal_template_config("invalid-type");
    config.repository_type = Some(config_manager::RepositoryTypeSpec {
        repository_type: "nonexistent-type".to_string(),
        policy: config_manager::RepositoryTypePolicy::Fixed,
    });

    let provider = Arc::new(
        MockMetadataProvider::new()
            .with_template_config("invalid-type".to_string(), config)
            .with_available_types(vec!["library".to_string(), "service".to_string()]),
    );

    let result = validate_template("test-org", "invalid-type", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.valid);
    assert!(validation.issues.len() > 0);
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("repository type") || i.message.contains("nonexistent-type")));
}

#[tokio::test]
async fn test_validate_template_not_found() {
    let provider = Arc::new(MockMetadataProvider::new());

    let result = validate_template("test-org", "missing", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.valid);
    assert_eq!(validation.template_name, "missing");
    assert!(validation.issues.len() > 0);
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("not found") || i.message.contains("Template")));
}

#[tokio::test]
async fn test_validate_template_configuration_missing() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "incomplete".to_string(),
        ConfigurationError::TemplateConfigurationMissing {
            org: "test-org".to_string(),
            template: "incomplete".to_string(),
        },
    ));

    let result = validate_template("test-org", "incomplete", provider).await;

    assert!(result.is_ok());
    let validation = result.unwrap();
    assert!(!validation.valid);
    assert!(validation.issues.len() > 0);
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("configuration") || i.message.contains("template.toml")));
}
