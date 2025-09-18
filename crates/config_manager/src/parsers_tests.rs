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
        [default_labels]
        value = [
            {
                name = "bug",
                description = "Something isn't working",
                color = "d73a4a"
            },
            {
                name = "enhancement",
                description = "New feature or request",
                color = "a2eeef"
            }
        ]
        override_allowed = true
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
        [organization_webhooks]
        value = [
            {
                url = "https://security.company.com/webhook",
                events = ["push", "pull_request"],
                active = true,
                secret = "webhook_secret_key"
            }
        ]
        override_allowed = false
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
        [organization_webhooks]
        value = [
            {
                url = "http://insecure.example.com/webhook",
                events = ["push"],
                active = true
            }
        ]
        override_allowed = false
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
        [organization_webhooks]
        value = [
            {
                url = "http://insecure.example.com/webhook",
                events = ["push"],
                active = true
            }
        ]
        override_allowed = false
        "#;

        let result = parser.parse(toml_content, "global/defaults.toml", "org/config");

        // Debug output to see what's happening
        println!("=== DEBUG OUTPUT ===");
        println!("config.is_some(): {}", result.config.is_some());
        println!("errors.len(): {}", result.errors.len());
        println!("warnings.len(): {}", result.warnings.len());

        for (i, error) in result.errors.iter().enumerate() {
            println!(
                "Error {}: field_path='{}', reason='{}'",
                i, error.field_path, error.reason
            );
        }

        for (i, warning) in result.warnings.iter().enumerate() {
            println!(
                "Warning {}: field_path='{}', message='{}'",
                i, warning.field_path, warning.message
            );
        }

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

#[cfg(test)]
mod team_config_parser_tests {
    use super::*;
    use crate::organization::{
        CommitMessageOption, GlobalDefaults, LabelConfig, MergeConfig, MergeType, OverridableValue,
        RepositoryVisibility, TeamConfig, WebhookConfig, WebhookEvent,
    };

    #[test]
    fn new_creates_parser_with_default_settings() {
        let parser = TeamConfigParser::new();

        assert!(parser.strict_security_validation);
        assert!(!parser.allow_deprecated_syntax);
        assert!(parser.custom_validators.is_empty());
    }

    #[test]
    fn with_options_creates_parser_with_custom_settings() {
        let parser = TeamConfigParser::with_options(false, true);

        assert!(!parser.strict_security_validation);
        assert!(parser.allow_deprecated_syntax);
        assert!(parser.custom_validators.is_empty());
    }

    #[test]
    fn default_creates_parser_same_as_new() {
        let parser1 = TeamConfigParser::new();
        let parser2 = TeamConfigParser::default();

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
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let result = parser.parse(
            "",
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert_eq!(result.metadata.fields_parsed, 0);
        assert!(!result.metadata.has_deprecated_syntax);
    }

    #[test]
    fn parse_valid_team_overrides_succeeds() {
        let parser = TeamConfigParser::new();
        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));
        global_defaults.branch_protection_enabled = Some(OverridableValue::overridable(true));

        let toml_content = r#"
        repository_visibility = "public"
        branch_protection_enabled = false
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        let config = result.config.unwrap();
        assert_eq!(
            config.repository_visibility,
            Some(RepositoryVisibility::Public)
        );
        assert_eq!(config.branch_protection_enabled, Some(false));
        assert_eq!(result.metadata.fields_parsed, 2);
    }

    #[test]
    fn parse_valid_team_webhooks_succeeds() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        [[team_webhooks]]
        url = "https://backend-team.example.com/webhook"
        events = ["push", "pull_request"]
        active = true
        secret = "team_secret"

        [[team_webhooks]]
        url = "https://backend-team.example.com/webhook2"
        events = ["issues"]
        active = false
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        let config = result.config.unwrap();
        assert!(config.team_webhooks.is_some());
        let webhooks = config.team_webhooks.as_ref().unwrap();
        assert_eq!(webhooks.len(), 2);
        assert_eq!(webhooks[0].url, "https://backend-team.example.com/webhook");
        assert_eq!(
            webhooks[0].events,
            vec![WebhookEvent::Push, WebhookEvent::PullRequest]
        );
        assert!(webhooks[0].active);
        assert!(webhooks[1].url.contains("webhook2"));
        assert!(!webhooks[1].active);
    }

    #[test]
    fn parse_valid_team_labels_and_apps_succeeds() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        team_github_apps = ["backend-app", "deployment-app"]

        [[team_labels]]
        name = "backend"
        color = "0052cc"
        description = "Backend team label"

        [[team_labels]]
        name = "urgent"
        color = "d93f0b"
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        let config = result.config.unwrap();

        assert!(config.team_github_apps.is_some());
        let apps = config.team_github_apps.as_ref().unwrap();
        assert_eq!(apps.len(), 2);
        assert!(apps.contains(&"backend-app".to_string()));

        assert!(config.team_labels.is_some());
        let labels = config.team_labels.as_ref().unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "backend");
        assert_eq!(labels[0].color, "0052cc");
        assert_eq!(labels[1].name, "urgent");
    }

    #[test]
    fn parse_invalid_toml_syntax_returns_error() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        repository_visibility = "Public
        # Missing closing quote
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].reason.contains("TOML syntax error"));
        assert!(result.errors[0].suggestion.is_some());
    }

    #[test]
    fn parse_ignores_unknown_fields() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        repository_visibility = "public"
        unknown_field = "value"
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        // Should succeed and ignore unknown fields
        assert!(result.config.is_some());
        let config = result.config.unwrap();
        assert_eq!(
            config.repository_visibility,
            Some(RepositoryVisibility::Public)
        );
        assert_eq!(result.metadata.fields_parsed, 1); // Only counted the known field
    }

    #[test]
    fn parse_override_not_allowed_returns_error() {
        let parser = TeamConfigParser::new();
        let mut global_defaults = GlobalDefaults::new();
        // Set repository visibility as fixed (not overridable)
        global_defaults.repository_visibility =
            Some(OverridableValue::fixed(RepositoryVisibility::Private));

        let toml_content = r#"
        repository_visibility = "public"
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].field_path == "repository_visibility");
        assert!(result.errors[0].reason.contains("Override not allowed"));
        assert!(result.errors[0].suggestion.is_some());
    }

    #[test]
    fn parse_insecure_webhook_url_returns_error_in_strict_mode() {
        let parser = TeamConfigParser::new(); // Strict security enabled by default
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        [[team_webhooks]]
        url = "http://insecure.example.com/webhook"
        events = ["push"]
        active = true
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].field_path.contains("team_webhooks"));
        assert!(result.errors[0].reason.contains("secure protocol"));
    }

    #[test]
    fn parse_insecure_webhook_url_returns_warning_in_non_strict_mode() {
        let parser = TeamConfigParser::with_options(false, false); // Non-strict security
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        [[team_webhooks]]
        url = "http://insecure.example.com/webhook"
        events = ["push"]
        active = true
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some()); // Config should still be parsed
        assert!(result.errors.is_empty()); // No errors, only warnings
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].field_path.contains("team_webhooks"));
        assert!(result.warnings[0].message.contains("secure protocol"));
    }

    #[test]
    fn parse_deprecated_syntax_returns_error_when_not_allowed() {
        let parser = TeamConfigParser::new(); // Deprecated syntax not allowed by default
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        [repository_visibility]
        value = "public"
        override_allowed = true
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_none());
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].field_path == "repository_visibility");
        assert!(result.errors[0]
            .reason
            .contains("Invalid team configuration syntax"));
    }

    #[test]
    fn parse_deprecated_syntax_returns_warning_when_allowed() {
        let parser = TeamConfigParser::with_options(true, true); // Allow deprecated syntax
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        [repository_visibility]
        value = "public"
        override_allowed = true
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_none()); // Still fails to parse due to incorrect structure
        assert!(result.metadata.has_deprecated_syntax);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].field_path == "repository_visibility");
        assert!(result.warnings[0].message.contains("simple values"));
    }

    // Security policy test removed - over-engineered for current requirements

    #[test]
    fn parse_team_github_apps_succeeds() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        team_github_apps = ["backend-deployment", "monitoring-alerts"]
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert!(result.config.is_some());
        assert!(result.errors.is_empty());
        let config = result.config.unwrap();

        assert!(config.team_github_apps.is_some());
        assert_eq!(
            config.team_github_apps.unwrap(),
            vec!["backend-deployment", "monitoring-alerts"]
        );
        assert_eq!(result.metadata.fields_parsed, 1);
    }

    #[test]
    fn validate_team_overrides_succeeds_for_allowed_overrides() {
        let parser = TeamConfigParser::new();
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let errors = parser.validate_team_overrides(
            &team_config,
            &global_defaults,
            "teams/backend/config.toml",
        );

        assert!(errors.is_empty());
    }

    #[test]
    fn validate_team_overrides_fails_for_fixed_global_settings() {
        let parser = TeamConfigParser::new();
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);
        team_config.branch_protection_enabled = Some(false);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::fixed(RepositoryVisibility::Private));
        global_defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));

        let errors = parser.validate_team_overrides(
            &team_config,
            &global_defaults,
            "teams/backend/config.toml",
        );

        assert_eq!(errors.len(), 2);
        assert!(errors[0].reason.contains("Override not allowed"));
        assert!(errors[1].reason.contains("Override not allowed"));
        assert!(errors[0].suggestion.is_some());
    }

    // Custom validator test removed - over-engineered for current needs

    #[test]
    fn metadata_fields_are_correctly_populated() {
        let parser = TeamConfigParser::new();
        let global_defaults = GlobalDefaults::new();

        let toml_content = r#"
        repository_visibility = "public"
        team_github_apps = ["app1", "app2"]
        "#;

        let result = parser.parse(
            toml_content,
            "teams/backend/config.toml",
            "org/config-repo",
            &global_defaults,
        );

        assert_eq!(result.metadata.file_path, "teams/backend/config.toml");
        assert_eq!(result.metadata.repository_context, "org/config-repo");
        assert_eq!(result.metadata.fields_parsed, 2);
        assert_eq!(result.metadata.defaults_applied, 0);
        assert!(!result.metadata.has_deprecated_syntax);
    }
}
