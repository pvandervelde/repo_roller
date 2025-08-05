//! Tests for configuration file parsers.

use super::*;
use crate::{
    organization::{
        GlobalDefaults, GlobalDefaultsEnhanced, OverridableValue, RepositoryVisibility,
    },
    types::{LabelConfig, MergeConfig, WebhookConfig, WebhookEvent},
};

#[cfg(test)]
mod global_defaults_parser_tests {
    use super::*;

    #[test]
    fn new_creates_parser_with_default_settings() {
        let parser = GlobalDefaultsParser::new();

        // Parser should be created successfully
        assert!(parser.strict_security_validation);
        assert!(!parser.allow_deprecated_syntax);
        assert!(parser.custom_validators.is_empty());
    }

    #[test]
    fn with_options_creates_parser_with_custom_settings() {
        let parser = GlobalDefaultsParser::with_options(false, true);

        assert!(!parser.strict_security_validation);
        assert!(parser.allow_deprecated_syntax);
        assert!(parser.custom_validators.is_empty());
    }

    #[test]
    fn default_creates_parser_same_as_new() {
        let parser1 = GlobalDefaultsParser::new();
        let parser2 = GlobalDefaultsParser::default();

        assert_eq!(
            parser1.strict_security_validation,
            parser2.strict_security_validation
        );
        assert_eq!(
            parser1.allow_deprecated_syntax,
            parser2.allow_deprecated_syntax
        );
        assert_eq!(
            parser1.custom_validators.len(),
            parser2.custom_validators.len()
        );
    }

    #[test]
    fn parse_valid_empty_toml_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = "";

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        assert_eq!(result.metadata.file_path, "global/defaults.toml");
        assert_eq!(result.metadata.repository_context, "org/config");
        assert_eq!(result.metadata.fields_parsed, 0);
        assert_eq!(result.metadata.defaults_applied, 0);
        assert!(!result.metadata.has_deprecated_syntax);
    }

    #[test]
    fn parse_valid_repository_settings_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        [[default_labels]]
        name = "bug"
        description = "Something isn't working"
        color = "d73a4a"

        [[default_labels]]
        name = "enhancement"
        description = "New feature or request"
        color = "a2eeef"
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());

        let config = result.config.unwrap();
        // Default labels should be properly parsed
        assert!(config.default_labels.is_some());

        let default_labels = config.default_labels.as_ref().unwrap();
        // We should have the overridable value structure
        assert!(default_labels.value().len() > 0);

        assert_eq!(result.metadata.fields_parsed, 2);
        assert!(!result.metadata.has_deprecated_syntax);
    }

    #[test]
    fn parse_valid_branch_protection_settings_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        branch_protection_enabled = { value = true, override_allowed = false }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        assert_eq!(result.metadata.fields_parsed, 1);

        let config = result.config.unwrap();
        assert!(config.branch_protection_enabled.is_some());
    }

    #[test]
    fn parse_valid_webhooks_configuration_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        organization_webhooks = {
            value = [
                {
                    url = "https://security.company.com/webhook",
                    events = ["push", "pull_request"],
                    active = true,
                    secret = "webhook_secret_key"
                }
            ],
            override_allowed = false
        }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());

        let config = result.config.unwrap();
        assert!(config.organization_webhooks.is_some());
        let webhooks = config.organization_webhooks.as_ref().unwrap();
        assert_eq!(webhooks.value().len(), 1);
        assert_eq!(
            webhooks.value()[0].url,
            "https://security.company.com/webhook"
        );
        assert!(webhooks.value()[0].active);
    }

    #[test]
    fn parse_invalid_toml_syntax_returns_error() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        [repository
        wiki = false  # Missing closing bracket
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert!(error.reason.contains("TOML syntax"));
        assert_eq!(error.field_path, "global/defaults.toml");
    }

    #[test]
    fn parse_unknown_field_returns_error() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        unknown_field = true
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert_eq!(error.field_path, "unknown_field");
        assert!(error.reason.contains("unknown field"));
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn parse_invalid_override_policy_structure_returns_error() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        branch_protection_enabled = { value = false }  # Missing override_allowed
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert_eq!(error.field_path, "branch_protection_enabled");
        assert!(error.reason.contains("override_allowed"));
    }

    #[test]
    fn parse_insecure_webhook_url_returns_error_with_strict_security() {
        let parser = GlobalDefaultsParser::with_options(true, false); // Strict security
        let toml_content = r#"
        organization_webhooks = {
            value = [
                {
                    url = "http://insecure.example.com/webhook",
                    events = ["push"],
                    active = true
                }
            ],
            override_allowed = false
        }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert_eq!(error.field_path, "organization_webhooks.value[0].url");
        assert!(error.reason.contains("secure protocol"));
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.as_ref().unwrap().contains("https://"));
    }

    #[test]
    fn parse_insecure_webhook_url_returns_warning_without_strict_security() {
        let parser = GlobalDefaultsParser::with_options(false, false); // Not strict security
        let toml_content = r#"
        organization_webhooks = {
            value = [
                {
                    url = "http://insecure.example.com/webhook",
                    events = ["push"],
                    active = true
                }
            ],
            override_allowed = false
        }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        assert!(!result.warnings.is_empty());

        let warning = &result.warnings[0];
        assert_eq!(warning.field_path, "organization_webhooks.value[0].url");
        assert!(warning.message.contains("secure protocol"));
    }

    #[test]
    fn parse_enhanced_global_defaults_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        [actions]
        enabled = { value = true, override_allowed = false }
        default_workflow_permissions = { value = "read", override_allowed = true }

        [branch_protection]
        enabled = { value = true, override_allowed = false }
        required_reviews = { value = 2, override_allowed = true }
        "#;

        let result = parser.parse_enhanced(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());

        let config = result.config.unwrap();
        assert!(config.actions.is_some());
        assert!(config.branch_protection.is_some());
    }

    #[test]
    fn parse_deprecated_syntax_returns_error_when_not_allowed() {
        let parser = GlobalDefaultsParser::with_options(true, false); // Don't allow deprecated
        let toml_content = r#"
        # Old syntax - deprecated
        repository_visibility = "private"
        branch_protection_enabled = true
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert!(error.reason.contains("deprecated"));
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn parse_deprecated_syntax_returns_warning_when_allowed() {
        let parser = GlobalDefaultsParser::with_options(true, true); // Allow deprecated
        let toml_content = r#"
        # Old syntax - deprecated but allowed
        repository_visibility = "private"
        branch_protection_enabled = true
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        assert!(!result.warnings.is_empty());
        assert!(result.metadata.has_deprecated_syntax);

        let warning = &result.warnings[0];
        assert!(warning.message.contains("deprecated"));
        assert!(warning.recommendation.is_some());
    }

    #[test]
    fn validate_policies_succeeds_for_secure_configuration() {
        let parser = GlobalDefaultsParser::new();
        let mut config = GlobalDefaults::new();

        // Add secure configuration
        config.branch_protection_enabled = Some(OverridableValue::fixed(true));
        config.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let errors = parser.validate_policies(&config, "global/defaults.toml");

        assert!(errors.is_empty());
    }

    #[test]
    fn validate_policies_fails_for_insecure_configuration() {
        let parser = GlobalDefaultsParser::new();
        let mut config = GlobalDefaults::new();

        // Add insecure configuration - branch protection disabled
        config.branch_protection_enabled = Some(OverridableValue::fixed(false));

        let errors = parser.validate_policies(&config, "global/defaults.toml");

        assert!(!errors.is_empty());
        let error = &errors[0];
        assert_eq!(error.field_path, "branch_protection_enabled");
        assert!(error.reason.contains("security"));
    }

    #[test]
    fn add_custom_validator_and_use_in_parsing() {
        let mut parser = GlobalDefaultsParser::new();

        // Add custom validator for webhook URLs
        parser.add_custom_validator(
            "webhooks.*.url",
            Box::new(|url: &str| {
                if url.contains("company.com") {
                    Ok(())
                } else {
                    Err("Webhook URLs must use company domain".to_string())
                }
            }),
        );

        let toml_content = r#"
        organization_webhooks = {
            value = [
                {
                    url = "https://external.example.com/webhook",
                    events = ["push"],
                    active = true
                }
            ],
            override_allowed = false
        }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());

        let error = &result.errors[0];
        assert_eq!(error.field_path, "organization_webhooks.value[0].url");
        assert!(error.reason.contains("company domain"));
    }

    #[test]
    fn parse_complex_complete_configuration_succeeds() {
        let parser = GlobalDefaultsParser::new();
        let toml_content = r#"
        branch_protection_enabled = { value = true, override_allowed = false }
        repository_visibility = { value = "private", override_allowed = true }

        organization_webhooks = {
            value = [
                {
                    url = "https://security.company.com/webhook",
                    events = ["push", "pull_request", "issues"],
                    active = true,
                    secret = "webhook_secret"
                },
                {
                    url = "https://notifications.company.com/webhook",
                    events = ["release"],
                    active = true
                }
            ],
            override_allowed = false
        }

        default_labels = {
            value = [
                {
                    name = "security",
                    description = "Security-related issue or PR",
                    color = "ff0000"
                },
                {
                    name = "documentation",
                    description = "Documentation improvements",
                    color = "0052cc"
                }
            ],
            override_allowed = true
        }
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());

        let config = result.config.unwrap();
        assert!(config.default_labels.is_some());
        assert!(config.organization_webhooks.is_some());
        assert!(config.branch_protection_enabled.is_some());

        // Verify specific values
        let webhooks = config.organization_webhooks.as_ref().unwrap();
        assert_eq!(webhooks.value().len(), 2);

        let labels = config.default_labels.as_ref().unwrap();
        assert_eq!(labels.value().len(), 2);

        // Verify parsing metadata
        assert!(result.metadata.fields_parsed > 0);
        assert_eq!(result.metadata.file_path, "global/defaults.toml");
        assert_eq!(result.metadata.repository_context, "org/config");
    }
}

#[cfg(test)]
mod parsing_utils_tests {
    use super::*;

    #[test]
    fn validate_toml_type_succeeds_for_correct_types() {
        let boolean_value = toml::Value::Boolean(true);
        let result = parsing_utils::validate_toml_type(&boolean_value, "boolean", "test.field");
        assert!(result.is_ok());

        let string_value = toml::Value::String("test".to_string());
        let result = parsing_utils::validate_toml_type(&string_value, "string", "test.field");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_toml_type_fails_for_incorrect_types() {
        let string_value = toml::Value::String("test".to_string());
        let result = parsing_utils::validate_toml_type(&string_value, "boolean", "test.field");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field_path, "test.field");
        assert!(error.reason.contains("boolean"));
    }

    #[test]
    fn extract_override_policy_succeeds_for_valid_structure() {
        let toml_content = r#"{ value = true, override_allowed = false }"#;
        let parsed: toml::Value = toml::from_str(toml_content).unwrap();

        let result = parsing_utils::extract_override_policy(&parsed, "test.field");

        assert!(result.is_ok());
        let (value, override_allowed) = result.unwrap();
        assert_eq!(value, toml::Value::Boolean(true));
        assert!(!override_allowed);
    }

    #[test]
    fn extract_override_policy_fails_for_missing_fields() {
        let toml_content = r#"{ value = true }"#; // Missing override_allowed
        let parsed: toml::Value = toml::from_str(toml_content).unwrap();

        let result = parsing_utils::extract_override_policy(&parsed, "test.field");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field_path, "test.field");
        assert!(error.reason.contains("override_allowed"));
    }

    #[test]
    fn validate_secure_url_succeeds_for_https_urls() {
        let result = parsing_utils::validate_secure_url("https://example.com", "webhooks.url");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_secure_url_fails_for_http_urls() {
        let result = parsing_utils::validate_secure_url("http://example.com", "webhooks.url");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.field_path, "webhooks.url");
        assert!(error.reason.contains("secure protocol"));
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn validate_secure_url_fails_for_invalid_urls() {
        let result = parsing_utils::validate_secure_url("not-a-url", "webhooks.url");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.reason.contains("valid URL"));
    }
}
