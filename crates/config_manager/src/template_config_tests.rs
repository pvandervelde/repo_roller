//! Tests for template configuration.

use super::*;

#[test]
fn test_template_metadata_creation() {
    let metadata = TemplateMetadata {
        name: "rust-library".to_string(),
        description: "Rust library template".to_string(),
        author: "Platform Team".to_string(),
        tags: vec!["rust".to_string(), "library".to_string()],
    };

    assert_eq!(metadata.name, "rust-library");
    assert_eq!(metadata.description, "Rust library template");
    assert_eq!(metadata.author, "Platform Team");
    assert_eq!(metadata.tags.len(), 2);
}

#[test]
fn test_repository_type_spec_fixed_policy() {
    let spec = RepositoryTypeSpec {
        repository_type: "service".to_string(),
        policy: RepositoryTypePolicy::Fixed,
    };

    assert_eq!(spec.repository_type, "service");
    assert_eq!(spec.policy, RepositoryTypePolicy::Fixed);
}

#[test]
fn test_repository_type_spec_preferable_policy() {
    let spec = RepositoryTypeSpec {
        repository_type: "library".to_string(),
        policy: RepositoryTypePolicy::Preferable,
    };

    assert_eq!(spec.repository_type, "library");
    assert_eq!(spec.policy, RepositoryTypePolicy::Preferable);
}

#[test]
fn test_repository_type_policy_serialization() {
    // Test serialization within a struct (not standalone enum)
    let spec = RepositoryTypeSpec {
        repository_type: "service".to_string(),
        policy: RepositoryTypePolicy::Fixed,
    };
    let toml = toml::to_string(&spec).expect("Failed to serialize");
    assert!(toml.contains("policy = \"fixed\""));

    let spec2 = RepositoryTypeSpec {
        repository_type: "library".to_string(),
        policy: RepositoryTypePolicy::Preferable,
    };
    let toml2 = toml::to_string(&spec2).expect("Failed to serialize");
    assert!(toml2.contains("policy = \"preferable\""));
}

#[test]
fn test_repository_type_policy_deserialization() {
    // Test deserialization within a struct
    let toml = r#"
        type = "service"
        policy = "fixed"
    "#;
    let spec: RepositoryTypeSpec = toml::from_str(toml).expect("Failed to parse");
    assert_eq!(spec.policy, RepositoryTypePolicy::Fixed);

    let toml2 = r#"
        type = "library"
        policy = "preferable"
    "#;
    let spec2: RepositoryTypeSpec = toml::from_str(toml2).expect("Failed to parse");
    assert_eq!(spec2.policy, RepositoryTypePolicy::Preferable);
}

#[test]
fn test_template_variable_with_all_fields() {
    let var = TemplateVariable {
        description: "Service name".to_string(),
        example: Some("user-service".to_string()),
        required: Some(true),
        default: Some("my-service".to_string()),
    };

    assert_eq!(var.description, "Service name");
    assert_eq!(var.example, Some("user-service".to_string()));
    assert_eq!(var.required, Some(true));
    assert_eq!(var.default, Some("my-service".to_string()));
}

#[test]
fn test_template_variable_minimal() {
    let var = TemplateVariable {
        description: "Port number".to_string(),
        example: None,
        required: None,
        default: None,
    };

    assert_eq!(var.description, "Port number");
    assert!(var.example.is_none());
    assert!(var.required.is_none());
    assert!(var.default.is_none());
}

#[test]
fn test_minimal_template_config() {
    let toml = r#"
        [template]
        name = "minimal-template"
        description = "Minimal template"
        author = "Test Author"
        tags = []
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert_eq!(config.template.name, "minimal-template");
    assert_eq!(config.template.description, "Minimal template");
    assert_eq!(config.template.author, "Test Author");
    assert_eq!(config.template.tags.len(), 0);
    assert!(config.repository_type.is_none());
    assert!(config.variables.is_none());
    assert!(config.repository.is_none());
}

#[test]
fn test_template_config_with_repository_type_fixed() {
    let toml = r#"
        [template]
        name = "service-template"
        description = "Microservice template"
        author = "Platform Team"
        tags = ["service"]

        [repository_type]
        type = "service"
        policy = "fixed"
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository_type.is_some());

    let repo_type = config.repository_type.unwrap();
    assert_eq!(repo_type.repository_type, "service");
    assert_eq!(repo_type.policy, RepositoryTypePolicy::Fixed);
}

#[test]
fn test_template_config_with_repository_type_preferable() {
    let toml = r#"
        [template]
        name = "library-template"
        description = "Library template"
        author = "Dev Team"
        tags = ["library"]

        [repository_type]
        type = "library"
        policy = "preferable"
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository_type.is_some());

    let repo_type = config.repository_type.unwrap();
    assert_eq!(repo_type.repository_type, "library");
    assert_eq!(repo_type.policy, RepositoryTypePolicy::Preferable);
}

#[test]
fn test_template_config_with_variables() {
    let toml = r#"
        [template]
        name = "app-template"
        description = "Application template"
        author = "Team"
        tags = ["app"]

        [variables.service_name]
        description = "Name of the service"
        example = "user-service"
        required = true

        [variables.port]
        description = "Port number"
        default = "8080"
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.variables.is_some());

    let vars = config.variables.unwrap();
    assert_eq!(vars.len(), 2);

    let service_name = vars.get("service_name").unwrap();
    assert_eq!(service_name.description, "Name of the service");
    assert_eq!(service_name.example, Some("user-service".to_string()));
    assert_eq!(service_name.required, Some(true));

    let port = vars.get("port").unwrap();
    assert_eq!(port.description, "Port number");
    assert_eq!(port.default, Some("8080".to_string()));
}

#[test]
fn test_template_config_with_repository_settings() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [repository]
        wiki = false
        issues = true
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.repository.is_some());

    let repo = config.repository.unwrap();
    assert!(repo.wiki.is_some());
    assert!(!repo.wiki.as_ref().unwrap().value);
    assert!(repo.issues.is_some());
    assert!(repo.issues.as_ref().unwrap().value);
}

#[test]
fn test_template_config_with_pull_request_settings() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.pull_requests.is_some());

    let pr = config.pull_requests.unwrap();
    assert_eq!(
        pr.required_approving_review_count.as_ref().unwrap().value,
        2
    );
    assert!(pr.require_code_owner_reviews.as_ref().unwrap().value);
}

#[test]
fn test_template_config_with_labels() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [[labels]]
        name = "template-specific"
        color = "ff0000"
        description = "Template-specific label"
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.labels.is_some());

    let labels = config.labels.unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "template-specific");
    assert_eq!(labels[0].color, "ff0000");
}

#[test]
fn test_template_config_with_webhooks() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [[webhooks]]
        url = "https://template.example.com/webhook"
        content_type = "json"
        events = ["push"]
        active = true
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.webhooks.is_some());

    let webhooks = config.webhooks.unwrap();
    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0].url, "https://template.example.com/webhook");
}

#[test]
fn test_template_config_with_environments() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [[environments]]
        name = "production"
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.environments.is_some());

    let envs = config.environments.unwrap();
    assert_eq!(envs.len(), 1);
    assert_eq!(envs[0].name, "production");
}

#[test]
fn test_template_config_with_github_apps() {
    let toml = r#"
        [template]
        name = "test-template"
        description = "Test"
        author = "Author"
        tags = []

        [[github_apps]]
        app_id = 99999
        permissions = { deployments = "write" }
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
    assert!(config.github_apps.is_some());

    let apps = config.github_apps.unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].app_id, 99999);
}

#[test]
fn test_complete_template_config() {
    let toml = r#"
        [template]
        name = "rust-microservice"
        description = "Production-ready Rust microservice"
        author = "Platform Team"
        tags = ["rust", "microservice", "backend"]

        [repository_type]
        type = "service"
        policy = "fixed"

        [variables.service_name]
        description = "Name of the microservice"
        example = "user-service"
        required = true

        [variables.service_port]
        description = "Port the service runs on"
        default = "8080"

        [repository]
        wiki = false
        security_advisories = true

        [pull_requests]
        required_approving_review_count = 2
        require_code_owner_reviews = true

        [branch_protection]
        require_pull_request_reviews = true

        [[labels]]
        name = "microservice"
        color = "0052cc"
        description = "Microservice-related"

        [[webhooks]]
        url = "https://ci.example.com/webhook"
        content_type = "json"
        events = ["push", "pull_request"]
        active = true

        [[environments]]
        name = "staging"

        [[github_apps]]
        app_id = 55555
        permissions = { actions = "write", deployments = "write" }
    "#;

    let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");

    // Verify template metadata
    assert_eq!(config.template.name, "rust-microservice");
    assert_eq!(config.template.tags.len(), 3);

    // Verify repository type
    assert!(config.repository_type.is_some());
    let repo_type = config.repository_type.unwrap();
    assert_eq!(repo_type.repository_type, "service");
    assert_eq!(repo_type.policy, RepositoryTypePolicy::Fixed);

    // Verify variables
    assert!(config.variables.is_some());
    let vars = config.variables.unwrap();
    assert_eq!(vars.len(), 2);

    // Verify all configuration sections
    assert!(config.repository.is_some());
    assert!(config.pull_requests.is_some());
    assert!(config.branch_protection.is_some());
    assert!(config.labels.is_some());
    assert!(config.webhooks.is_some());
    assert!(config.environments.is_some());
    assert!(config.github_apps.is_some());
}

#[test]
fn test_serialize_round_trip() {
    let config = TemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "Test template".to_string(),
            author: "Author".to_string(),
            tags: vec!["test".to_string()],
        },
        repository_type: Some(RepositoryTypeSpec {
            repository_type: "library".to_string(),
            policy: RepositoryTypePolicy::Preferable,
        }),
        variables: None,
        repository: None,
        pull_requests: None,
        branch_protection: None,
        labels: None,
        webhooks: None,
        environments: None,
        github_apps: None,
    };

    let toml = toml::to_string(&config).expect("Failed to serialize");
    let parsed: TemplateConfig = toml::from_str(&toml).expect("Failed to parse");

    assert_eq!(config, parsed);
}

#[test]
fn test_clone_creates_independent_copy() {
    let config = TemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
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
    };

    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_debug_format() {
    let config = TemplateConfig {
        template: TemplateMetadata {
            name: "test".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
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
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("TemplateConfig"));
    assert!(debug_str.contains("template"));
}
