//! Tests for template commands.
//!
//! These tests verify the template inspection and validation command implementations.

use super::*;
use async_trait::async_trait;
use chrono::Utc;
use config_manager::{
    settings::{BranchProtectionSettings, PullRequestSettings, RepositorySettings, WebhookConfig},
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

    fn with_available_types(mut self, types: Vec<String>) -> Self {
        self.available_types = types;
        self
    }
}

#[async_trait]
impl MetadataRepositoryProvider for MockMetadataProvider {
    async fn discover_metadata_repository(
        &self,
        org: &str,
    ) -> ConfigurationResult<MetadataRepository> {
        Ok(MetadataRepository {
            organization: org.to_string(),
            repository_name: ".reporoller-test".to_string(),
            discovery_method: config_manager::DiscoveryMethod::ConfigurationBased {
                repository_name: ".reporoller-test".to_string(),
            },
            last_updated: Utc::now(),
        })
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

    async fn load_global_webhooks(
        &self,
        _repo: &MetadataRepository,
    ) -> ConfigurationResult<Vec<WebhookConfig>> {
        unimplemented!("Not needed for these tests")
    }
}

// ============================================================================
// Helper Functions for Test Data
// ============================================================================

fn create_minimal_template_config(name: &str) -> TemplateConfig {
    TemplateConfig {
        default_visibility: None,
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
        rulesets: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
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
        default_visibility: None,
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
        rulesets: None,
        templating: None,
        notifications: None,
        permissions: None,
        teams: None,
        collaborators: None,
        naming_rules: None,
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
    assert!(!validation.warnings.is_empty());
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
    assert!(!validation.warnings.is_empty());
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
    assert!(!validation.warnings.is_empty());
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
    assert!(!validation.issues.is_empty());
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("Variable")
            && i.message.contains("invalid")
            && i.message.contains("characters")));
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
    assert!(!validation.issues.is_empty());
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
    assert!(!validation.warnings.is_empty());
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
    assert!(!validation.issues.is_empty());
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
    assert!(!validation.issues.is_empty());
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
    assert!(!validation.issues.is_empty());
    assert!(validation
        .issues
        .iter()
        .any(|i| i.message.contains("configuration") || i.message.contains("template.toml")));
}
// ============================================================================
// Output Formatting Tests
// ============================================================================

/// Test formatting TemplateInfo as JSON.
#[test]
fn test_format_template_info_json() {
    let info = TemplateInfo {
        name: "rust-library".to_string(),
        description: "A Rust library template".to_string(),
        author: "Platform Team".to_string(),
        tags: vec!["rust".to_string(), "library".to_string()],
        repository_type: Some(RepositoryTypeInfo {
            type_name: "library".to_string(),
            policy: "fixed".to_string(),
        }),
        variables: vec![TemplateVariableInfo {
            name: "project_name".to_string(),
            description: Some("Project name".to_string()),
            required: true,
            default_value: None,
            example: Some("my-lib".to_string()),
        }],
        configuration_sections: 3,
    };

    let result = format_template_info(&info, "json");
    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["name"], "rust-library");
    assert_eq!(parsed["author"], "Platform Team");
    assert!(json_str.contains("rust-library"));
}

/// Test formatting TemplateInfo as pretty output.
#[test]
fn test_format_template_info_pretty() {
    let info = TemplateInfo {
        name: "rust-library".to_string(),
        description: "A Rust library template".to_string(),
        author: "Platform Team".to_string(),
        tags: vec!["rust".to_string(), "library".to_string()],
        repository_type: Some(RepositoryTypeInfo {
            type_name: "library".to_string(),
            policy: "fixed".to_string(),
        }),
        variables: vec![TemplateVariableInfo {
            name: "project_name".to_string(),
            description: Some("Project name".to_string()),
            required: true,
            default_value: None,
            example: Some("my-lib".to_string()),
        }],
        configuration_sections: 3,
    };

    let result = format_template_info(&info, "pretty");
    assert!(result.is_ok());
    let output = result.unwrap();

    // Verify key information is present
    assert!(output.contains("rust-library"));
    assert!(output.contains("Platform Team"));
    assert!(output.contains("A Rust library template"));
    assert!(output.contains("rust"));
    assert!(output.contains("library"));
    assert!(output.contains("project_name"));
}

/// Test formatting ValidationResult as JSON.
#[test]
fn test_format_validation_result_json() {
    let result_data = TemplateValidationResult {
        template_name: "test-template".to_string(),
        valid: true,
        issues: vec![],
        warnings: vec![ValidationWarning {
            category: "best_practice".to_string(),
            message: "Consider adding more tags".to_string(),
        }],
    };

    let result = format_validation_result(&result_data, "json");
    assert!(result.is_ok());
    let json_str = result.unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["template_name"], "test-template");
    assert_eq!(parsed["warnings"].as_array().unwrap().len(), 1);
}

/// Test formatting ValidationResult with errors as pretty output.
#[test]
fn test_format_validation_result_with_errors_pretty() {
    let result_data = TemplateValidationResult {
        template_name: "invalid-template".to_string(),
        valid: false,
        issues: vec![
            ValidationIssue {
                severity: "error".to_string(),
                location: "template.toml".to_string(),
                message: "Missing required field 'name'".to_string(),
            },
            ValidationIssue {
                severity: "error".to_string(),
                location: "variables.service_name".to_string(),
                message: "Invalid variable name format".to_string(),
            },
        ],
        warnings: vec![],
    };

    let result = format_validation_result(&result_data, "pretty");
    assert!(result.is_ok());
    let output = result.unwrap();

    // Verify issues are displayed
    assert!(output.contains("invalid-template"));
    assert!(output.contains("name") || output.contains("field"));
    assert!(output.contains("variable") || output.contains("service_name"));
}

/// Test formatting ValidationResult with warnings as pretty output.
#[test]
fn test_format_validation_result_with_warnings_pretty() {
    let result_data = TemplateValidationResult {
        template_name: "warning-template".to_string(),
        valid: true,
        issues: vec![],
        warnings: vec![
            ValidationWarning {
                category: "best_practice".to_string(),
                message: "No description provided".to_string(),
            },
            ValidationWarning {
                category: "completeness".to_string(),
                message: "No variables defined".to_string(),
            },
        ],
    };

    let result = format_validation_result(&result_data, "pretty");
    assert!(result.is_ok());
    let output = result.unwrap();

    // Verify warnings are displayed
    assert!(output.contains("warning-template"));
    assert!(output.contains("description") || output.contains("variables"));
}

/// Test invalid format string returns error.
#[test]
fn test_format_output_invalid_format() {
    let info = TemplateInfo {
        name: "test".to_string(),
        description: "Test".to_string(),
        author: "Author".to_string(),
        tags: vec![],
        repository_type: None,
        variables: vec![],
        configuration_sections: 0,
    };

    let result = format_template_info(&info, "invalid");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::InvalidArguments(_)));
}
// ============================================================================
// Error Message Enhancement Tests
// ============================================================================

/// Test that template not found errors include helpful suggestions.
#[tokio::test]
async fn test_error_template_not_found_includes_suggestions() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "nonexistent".to_string(),
        ConfigurationError::TemplateNotFound {
            org: "test-org".to_string(),
            template: "nonexistent".to_string(),
        },
    ));

    let result = get_template_info("test-org", "nonexistent", provider).await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());

    // Error should mention the template name and org
    assert!(err_msg.contains("nonexistent"));
    assert!(err_msg.contains("test-org"));
    // Should be a Config error
    assert!(err_msg.contains("Configuration error"));
}

/// Test that parse errors include details about the issue.
#[tokio::test]
async fn test_error_parse_includes_details() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "malformed".to_string(),
        ConfigurationError::ParseError {
            reason: "missing field name at line 5".to_string(),
        },
    ));

    let result = get_template_info("test-org", "malformed", provider).await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());

    // Should include parse error details
    assert!(err_msg.contains("Failed to parse") || err_msg.contains("parse"));
    assert!(err_msg.contains("malformed"));
    // Should include the specific reason
    assert!(err_msg.contains("missing field") || err_msg.contains("line 5"));
}

/// Test that missing configuration file errors are clear.
#[tokio::test]
async fn test_error_missing_configuration_file_clear() {
    let provider = Arc::new(MockMetadataProvider::new().with_template_error(
        "no-config".to_string(),
        ConfigurationError::TemplateConfigurationMissing {
            org: "test-org".to_string(),
            template: "no-config".to_string(),
        },
    ));

    let result = get_template_info("test-org", "no-config", provider).await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());

    // Should mention the missing configuration file
    assert!(
        err_msg.contains(".reporoller/template.toml") || err_msg.contains("configuration file")
    );
    assert!(err_msg.contains("missing") || err_msg.contains("exists but"));
}

// ============================================================================
// Keyring Constant Tests
// ============================================================================

/// Verify that template_cmd uses the canonical keyring constants from auth_cmd.
///
/// Credentials are stored by `auth github` using auth_cmd's service name and key names.
/// If template_cmd uses different values, keyring lookups silently fail at runtime.
#[test]
fn test_keyring_constants_match_auth_cmd_canonical_values() {
    assert_eq!(
        KEY_RING_SERVICE_NAME, "repo_roller_cli",
        "service name must match auth_cmd so credentials saved by 'auth github' are found"
    );
    assert_eq!(
        KEY_RING_APP_ID, "github_app_id",
        "app ID key must match auth_cmd"
    );
    assert_eq!(
        KEY_RING_APP_PRIVATE_KEY_PATH, "github_private_key_path",
        "private key path key must match auth_cmd so the correct keyring entry is read"
    );
}

// ============================================================================
// load_template_config_from_path() Tests (task 2.2)
// ============================================================================

#[test]
fn test_load_template_config_from_path_valid() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".reporoller")).unwrap();
    std::fs::write(
        tmp.path().join(".reporoller/template.toml"),
        "[template]\nname = \"local-template\"\ndescription = \"A local template\"\nauthor = \"Platform Team\"\ntags = [\"rust\"]\n",
    )
    .unwrap();

    let result = load_template_config_from_path(tmp.path());

    assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
    let config = result.unwrap();
    assert_eq!(config.template.name, "local-template");
    assert_eq!(config.template.author, "Platform Team");
}

#[test]
fn test_load_template_config_from_path_missing_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    // No .reporoller/template.toml created

    let result = load_template_config_from_path(tmp.path());

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Config(msg) => {
            assert!(
                msg.contains("template.toml")
                    || msg.contains("not found")
                    || msg.contains("No such"),
                "Unexpected error message: {}",
                msg
            );
        }
        e => panic!("Expected Config error, got {:?}", e),
    }
}

#[test]
fn test_load_template_config_from_path_invalid_toml() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".reporoller")).unwrap();
    std::fs::write(
        tmp.path().join(".reporoller/template.toml"),
        "this is not valid toml @@@ !!!",
    )
    .unwrap();

    let result = load_template_config_from_path(tmp.path());

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::Config(msg) => {
            assert!(
                msg.contains("parse") || msg.contains("TOML") || msg.contains("invalid"),
                "Unexpected error message: {}",
                msg
            );
        }
        e => panic!("Expected Config error, got {:?}", e),
    }
}

// ============================================================================
// detect_github_remote() Tests (task 3.1 / 3.3)
// ============================================================================

#[test]
fn test_detect_github_remote_https_url() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(
        tmp.path().join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = https://github.com/myorg/my-repo.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n",
    )
    .unwrap();

    let result = detect_github_remote(tmp.path());

    assert!(result.is_some(), "Expected Some but got None");
    let (org, repo) = result.unwrap();
    assert_eq!(org, "myorg");
    assert_eq!(repo, "my-repo");
}

#[test]
fn test_detect_github_remote_ssh_url() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(
        tmp.path().join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = git@github.com:acme/cool-service.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n",
    )
    .unwrap();

    let result = detect_github_remote(tmp.path());

    assert!(result.is_some(), "Expected Some but got None");
    let (org, repo) = result.unwrap();
    assert_eq!(org, "acme");
    assert_eq!(repo, "cool-service");
}

#[test]
fn test_detect_github_remote_non_github_url_returns_none() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(
        tmp.path().join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = https://gitlab.com/someorg/some-repo.git\n",
    )
    .unwrap();

    let result = detect_github_remote(tmp.path());

    assert!(result.is_none());
}

#[test]
fn test_detect_github_remote_no_git_config_returns_none() {
    let tmp = tempfile::TempDir::new().unwrap();
    // No .git directory created

    let result = detect_github_remote(tmp.path());

    assert!(result.is_none());
}

#[test]
fn test_detect_github_remote_empty_git_config_returns_none() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(tmp.path().join(".git/config"), "").unwrap();

    let result = detect_github_remote(tmp.path());

    assert!(result.is_none());
}

#[test]
fn test_detect_github_remote_https_without_dot_git_suffix() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(
        tmp.path().join(".git/config"),
        "[remote \"origin\"]\n\turl = https://github.com/owner/repo-name\n",
    )
    .unwrap();

    let result = detect_github_remote(tmp.path());

    assert!(result.is_some());
    let (org, repo) = result.unwrap();
    assert_eq!(org, "owner");
    assert_eq!(repo, "repo-name");
}

// ============================================================================
// run_git_clone() Tests (task 4.1 / 4.3)
// ============================================================================

/// Initialise a minimal git repository with one commit in `dir`.
/// Returns a `file://` URL that `git clone` accepts without network access.
fn create_local_git_repo(dir: &std::path::Path) -> String {
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .expect("git init failed");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("git config email failed");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .expect("git config name failed");
    std::fs::write(dir.join("README.md"), "# Test").expect("write README failed");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .expect("git add failed");
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir)
        .output()
        .expect("git commit failed");
    format!("file://{}", dir.display())
}

#[test]
fn test_run_git_clone_success_local_repo() {
    let source_dir = tempfile::TempDir::new().unwrap();
    let url = create_local_git_repo(source_dir.path());
    let dest_dir = tempfile::TempDir::new().unwrap();
    let dest = dest_dir.path().join("cloned");

    let result = run_git_clone(&url, &dest);

    assert!(
        result.is_ok(),
        "clone should succeed; got: {:?}",
        result.err()
    );
    assert!(
        dest.join("README.md").exists(),
        "README.md should exist in clone"
    );
}

#[test]
fn test_run_git_clone_nonexistent_url_returns_github_error() {
    let dest_dir = tempfile::TempDir::new().unwrap();
    let dest = dest_dir.path().join("cloned");

    let result = run_git_clone("file:///nonexistent/path/that/does/not/exist/99999", &dest);

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::GitHub(msg) => {
            assert!(!msg.is_empty(), "error message should not be empty");
        }
        e => panic!("Expected GitHub error, got {:?}", e),
    }
}

// ============================================================================
// template_validate() Routing Tests (tasks 2.3, 3.2, 4.2)
// ============================================================================

/// Minimal valid template.toml TOML content used across routing tests.
const VALID_TEMPLATE_TOML: &str =
    "[template]\nname = \"routing-test-template\"\ndescription = \"A template for routing tests\"\nauthor = \"Test Author\"\ntags = [\"test\"]\n";

#[tokio::test]
async fn test_template_validate_routing_neither_path_nor_org_returns_invalid_arguments() {
    let result = template_validate(None, None, None, "pretty").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidArguments(msg) => {
            assert!(
                msg.contains("--path") || msg.contains("--org") || msg.contains("path"),
                "Unexpected message: {}",
                msg
            );
        }
        e => panic!("Expected InvalidArguments, got {:?}", e),
    }
}

#[tokio::test]
async fn test_template_validate_routing_only_org_no_template_returns_invalid_arguments() {
    let result = template_validate(Some("myorg"), None, None, "pretty").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidArguments(_) => {}
        e => panic!("Expected InvalidArguments, got {:?}", e),
    }
}

#[tokio::test]
async fn test_template_validate_routing_local_path_with_valid_config_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".reporoller")).unwrap();
    std::fs::write(
        tmp.path().join(".reporoller/template.toml"),
        VALID_TEMPLATE_TOML,
    )
    .unwrap();
    // No .git/config → no remote detection → no provider needed; no --org given
    let path_str = tmp.path().to_str().unwrap().to_string();

    let result = template_validate(None, None, Some(&path_str), "json").await;

    assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
}

#[tokio::test]
async fn test_template_validate_routing_local_path_missing_config_returns_error() {
    let tmp = tempfile::TempDir::new().unwrap();
    // Existing directory but no .reporoller/template.toml
    let path_str = tmp.path().to_str().unwrap().to_string();

    let result = template_validate(None, None, Some(&path_str), "pretty").await;

    // load_template_config_from_path fails → propagates as Err
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_validate_routing_nonexistent_path_no_org_returns_invalid_arguments() {
    // Path does not exist AND no org+template provided → InvalidArguments
    let result = template_validate(
        None,
        None,
        Some("/nonexistent/path/that/does/not/exist/xyz"),
        "pretty",
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        Error::InvalidArguments(_) => {}
        e => panic!("Expected InvalidArguments, got {:?}", e),
    }
}
