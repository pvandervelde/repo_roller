//! Tests for template configuration structures.

use crate::settings::*;
use crate::templates::*;
use crate::types::*;
use std::collections::HashMap;

#[cfg(test)]
mod template_metadata_tests {
    use super::*;

    #[test]
    fn template_metadata_new() {
        let metadata = TemplateMetadata::new("rust-service".to_string());
        assert_eq!(metadata.name, "rust-service");
        assert!(metadata.description.is_none());
        assert!(metadata.version.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.tags.is_none());
    }

    #[test]
    fn template_metadata_builder_pattern() {
        let metadata = TemplateMetadata::new("rust-service".to_string())
            .with_description("Rust microservice template".to_string())
            .with_version("1.0.0".to_string())
            .with_author("Platform Team".to_string())
            .with_tags(vec!["rust".to_string(), "service".to_string()]);

        assert_eq!(metadata.name, "rust-service");
        assert_eq!(
            metadata.description,
            Some("Rust microservice template".to_string())
        );
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.author, Some("Platform Team".to_string()));
        assert_eq!(
            metadata.tags,
            Some(vec!["rust".to_string(), "service".to_string()])
        );
    }
}

#[cfg(test)]
mod template_variable_tests {
    use super::*;

    #[test]
    fn template_variable_new() {
        let var = TemplateVariable::new(
            Some("Service name".to_string()),
            Some("my-service".to_string()),
            true,
        );

        assert_eq!(var.description, Some("Service name".to_string()));
        assert_eq!(var.default_value, Some("my-service".to_string()));
        assert!(var.required);
    }

    #[test]
    fn template_variable_required() {
        let var = TemplateVariable::required(Some("Required setting".to_string()));
        assert_eq!(var.description, Some("Required setting".to_string()));
        assert!(var.default_value.is_none());
        assert!(var.required);
    }

    #[test]
    fn template_variable_optional() {
        let var = TemplateVariable::optional(
            Some("Optional setting".to_string()),
            "default_value".to_string(),
        );
        assert_eq!(var.description, Some("Optional setting".to_string()));
        assert_eq!(var.default_value, Some("default_value".to_string()));
        assert!(!var.required);
    }
}

#[cfg(test)]
mod repository_type_spec_tests {
    use super::*;

    #[test]
    fn repository_type_spec_new() {
        let spec = RepositoryTypeSpec::new(
            "service".to_string(),
            Some("Microservice repository".to_string()),
        );

        assert_eq!(spec.required_type, "service");
        assert_eq!(
            spec.description,
            Some("Microservice repository".to_string())
        );
    }

    #[test]
    fn repository_type_spec_without_description() {
        let spec = RepositoryTypeSpec::new("library".to_string(), None);
        assert_eq!(spec.required_type, "library");
        assert!(spec.description.is_none());
    }
}

#[cfg(test)]
mod template_config_tests {
    use super::*;

    #[test]
    fn template_config_new() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let config = TemplateConfig::new(metadata);

        assert_eq!(config.template().name, "test-template");
        assert!(config.repository_type().is_none());
        assert!(config.repository().is_none());
        assert!(config.pull_requests().is_none());
        assert!(config.branch_protection().is_none());
        assert!(config.labels().is_none());
        assert!(config.webhooks().is_none());
        assert!(config.github_apps().is_none());
        assert!(config.environments().is_none());
        assert!(config.variables().is_none());
    }

    #[test]
    fn template_config_getters_and_setters() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let mut config = TemplateConfig::new(metadata);

        // Test repository type
        let repo_type = RepositoryTypeSpec::new("service".to_string(), None);
        config.set_repository_type(Some(repo_type));
        assert!(config.repository_type().is_some());
        assert_eq!(config.repository_type().unwrap().required_type, "service");

        // Test repository settings
        let repo_settings = RepositorySettings::new();
        config.set_repository(Some(repo_settings));
        assert!(config.repository().is_some());

        // Test labels
        let labels = vec![LabelConfig::new("bug".to_string(), None, "red".to_string())];
        config.set_labels(Some(labels));
        assert!(config.labels().is_some());
        assert_eq!(config.labels().unwrap().len(), 1);
        assert_eq!(config.labels().unwrap()[0].name, "bug");
    }

    #[test]
    fn template_config_add_variable() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let mut config = TemplateConfig::new(metadata);

        assert!(config.variables().is_none());

        config.add_variable(
            "service_name".to_string(),
            TemplateVariable::required(Some("Service name".to_string())),
        );

        assert!(config.variables().is_some());
        assert_eq!(config.variables().unwrap().len(), 1);
        assert!(config.variables().unwrap().contains_key("service_name"));
    }

    #[test]
    fn template_config_has_configuration() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let mut config = TemplateConfig::new(metadata);

        assert!(!config.has_configuration());
        assert_eq!(config.configuration_count(), 0);

        config.set_repository_type(Some(RepositoryTypeSpec::new("service".to_string(), None)));
        assert!(config.has_configuration());
        assert_eq!(config.configuration_count(), 1);

        config.set_repository(Some(RepositorySettings::new()));
        assert_eq!(config.configuration_count(), 2);

        let mut variables = HashMap::new();
        variables.insert("test".to_string(), TemplateVariable::required(None));
        config.set_variables(Some(variables));
        assert_eq!(config.configuration_count(), 3);
    }

    #[test]
    fn template_config_webhooks_and_apps() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let mut config = TemplateConfig::new(metadata);

        // Test webhooks
        let webhooks = vec![WebhookConfig::new(
            "https://example.com/webhook".to_string(),
            vec![WebhookEvent::Push],
            true,
            None,
        )];
        config.set_webhooks(Some(webhooks));
        assert!(config.webhooks().is_some());
        assert_eq!(config.webhooks().unwrap().len(), 1);

        // Test GitHub apps
        let apps = vec![GitHubAppConfig::new("dependabot".to_string())];
        config.set_github_apps(Some(apps));
        assert!(config.github_apps().is_some());
        assert_eq!(config.github_apps().unwrap().len(), 1);
    }

    #[test]
    fn template_config_environments() {
        let metadata = TemplateMetadata::new("test-template".to_string());
        let mut config = TemplateConfig::new(metadata);

        let environments = vec![
            EnvironmentConfig::new("production".to_string(), None, None, None),
            EnvironmentConfig::new("staging".to_string(), None, None, None),
        ];
        config.set_environments(Some(environments));

        assert!(config.environments().is_some());
        assert_eq!(config.environments().unwrap().len(), 2);
        assert_eq!(config.environments().unwrap()[0].name, "production");
        assert_eq!(config.environments().unwrap()[1].name, "staging");
    }
}
