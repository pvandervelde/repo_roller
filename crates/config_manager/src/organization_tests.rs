//! Tests for organization-specific configuration structures.
//!
//! This module contains comprehensive unit tests for the organization configuration
//! system, focusing on the `OverridableValue<T>` generic type and hierarchical
//! configuration structures.

use crate::organization::{
    CommitMessageOption, ConfigurationError, EnvironmentConfig, GlobalDefaults, LabelConfig,
    MergeConfig, MergeType, OverridableValue, RepositoryTypeConfig, RepositoryTypePolicy,
    RepositoryTypeSpec, RepositoryVisibility, TeamConfig, TemplateConfig, TemplateMetadata,
    TemplateVariable, WebhookConfig, WebhookEvent, WorkflowPermission,
};
use std::collections::HashMap;

#[cfg(test)]
mod overridable_value_tests {
    use super::*;

    #[test]
    fn new_creates_overridable_value_with_correct_properties() {
        let value = OverridableValue::new(42, true);
        assert_eq!(value.value(), 42);
        assert!(value.can_override());

        let fixed_value = OverridableValue::new("test".to_string(), false);
        assert_eq!(fixed_value.value(), "test");
        assert!(!fixed_value.can_override());
    }

    #[test]
    fn overridable_creates_value_that_can_be_overridden() {
        let value = OverridableValue::overridable(100);
        assert_eq!(value.value(), 100);
        assert!(value.can_override());
    }

    #[test]
    fn fixed_creates_value_that_cannot_be_overridden() {
        let value = OverridableValue::fixed("security_setting".to_string());
        assert_eq!(value.value(), "security_setting");
        assert!(!value.can_override());
    }

    #[test]
    fn try_override_succeeds_when_override_allowed() {
        let original = OverridableValue::overridable(10);
        let overridden = original.try_override(20);

        assert_eq!(overridden.value(), 20);
        assert!(overridden.can_override());

        // Original should be unchanged
        assert_eq!(original.value(), 10);
    }

    #[test]
    fn try_override_fails_when_override_not_allowed() {
        let original = OverridableValue::fixed(10);
        let unchanged = original.try_override(20);

        assert_eq!(unchanged.value(), 10);
        assert!(!unchanged.can_override());

        // Should return a new instance with same values
        assert_eq!(original.value(), unchanged.value());
        assert_eq!(original.can_override(), unchanged.can_override());
    }

    #[test]
    fn try_override_preserves_override_permission() {
        let overridable = OverridableValue::overridable("original".to_string());
        let overridden = overridable.try_override("new".to_string());

        assert_eq!(overridden.value(), "new");
        assert!(overridden.can_override());
    }

    #[test]
    fn overridable_value_supports_complex_types() {
        let labels = vec![
            LabelConfig {
                name: "bug".to_string(),
                description: Some("Something isn't working".to_string()),
                color: "d73a4a".to_string(),
            },
            LabelConfig {
                name: "enhancement".to_string(),
                description: Some("New feature or request".to_string()),
                color: "a2eeef".to_string(),
            },
        ];

        let value = OverridableValue::overridable(labels.clone());
        assert_eq!(value.value(), labels);
        assert!(value.can_override());

        let new_labels = vec![LabelConfig {
            name: "documentation".to_string(),
            description: Some("Improvements or additions to documentation".to_string()),
            color: "0075ca".to_string(),
        }];

        let overridden = value.try_override(new_labels.clone());
        assert_eq!(overridden.value(), new_labels);
        assert!(overridden.can_override());
    }

    #[test]
    fn overridable_value_supports_vectors() {
        let original_vec = vec!["item1".to_string(), "item2".to_string()];
        let value = OverridableValue::fixed(original_vec.clone());

        assert_eq!(value.value(), original_vec);
        assert!(!value.can_override());

        let new_vec = vec!["item3".to_string(), "item4".to_string()];
        let unchanged = value.try_override(new_vec);

        assert_eq!(unchanged.value(), original_vec);
        assert!(!unchanged.can_override());
    }

    #[test]
    fn overridable_value_equality_works_correctly() {
        let value1 = OverridableValue::new(42, true);
        let value2 = OverridableValue::new(42, true);
        let value3 = OverridableValue::new(42, false);
        let value4 = OverridableValue::new(43, true);

        assert_eq!(value1, value2);
        assert_ne!(value1, value3); // Different override permission
        assert_ne!(value1, value4); // Different value
    }

    #[test]
    fn overridable_value_cloning_works_correctly() {
        let original = OverridableValue::overridable("test".to_string());
        let cloned = original.clone();

        assert_eq!(original.value(), cloned.value());
        assert_eq!(original.can_override(), cloned.can_override());

        // Ensure they are independent
        let modified = cloned.try_override("modified".to_string());
        assert_ne!(original.value(), modified.value());
    }
}

#[cfg(test)]
mod global_defaults_tests {
    use super::*;

    #[test]
    fn new_creates_empty_global_defaults() {
        let defaults = GlobalDefaults::new();

        assert!(defaults.branch_protection_enabled.is_none());
        assert!(defaults.repository_visibility.is_none());
        assert!(defaults.merge_configuration.is_none());
        assert!(defaults.default_labels.is_none());
        assert!(defaults.organization_webhooks.is_none());
        assert!(defaults.required_github_apps.is_none());
    }

    #[test]
    fn default_trait_implementation_works() {
        let defaults = GlobalDefaults::default();
        let manual_new = GlobalDefaults::new();

        assert_eq!(
            defaults.branch_protection_enabled,
            manual_new.branch_protection_enabled
        );
        assert_eq!(
            defaults.repository_visibility,
            manual_new.repository_visibility
        );
        assert_eq!(defaults.merge_configuration, manual_new.merge_configuration);
        assert_eq!(defaults.default_labels, manual_new.default_labels);
        assert_eq!(
            defaults.organization_webhooks,
            manual_new.organization_webhooks
        );
        assert_eq!(
            defaults.required_github_apps,
            manual_new.required_github_apps
        );
    }

    #[test]
    fn global_defaults_can_be_populated_with_fixed_values() {
        let mut defaults = GlobalDefaults::new();

        defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));
        defaults.repository_visibility =
            Some(OverridableValue::fixed(RepositoryVisibility::Private));

        let bp_enabled = defaults.branch_protection_enabled.as_ref().unwrap();
        assert!(bp_enabled.value());
        assert!(!bp_enabled.can_override());

        let visibility = defaults.repository_visibility.as_ref().unwrap();
        assert_eq!(visibility.value(), RepositoryVisibility::Private);
        assert!(!visibility.can_override());
    }

    #[test]
    fn global_defaults_can_be_populated_with_overridable_values() {
        let mut defaults = GlobalDefaults::new();

        let merge_config = MergeConfig {
            allowed_types: vec![MergeType::Merge, MergeType::Squash],
            merge_commit_message: CommitMessageOption::PullRequestTitle,
            squash_commit_message: CommitMessageOption::PullRequestTitleAndDescription,
        };
        defaults.merge_configuration = Some(OverridableValue::overridable(merge_config.clone()));

        let labels = vec![
            LabelConfig {
                name: "bug".to_string(),
                description: Some("Something isn't working".to_string()),
                color: "d73a4a".to_string(),
            },
            LabelConfig {
                name: "enhancement".to_string(),
                description: Some("New feature or request".to_string()),
                color: "a2eeef".to_string(),
            },
        ];
        defaults.default_labels = Some(OverridableValue::overridable(labels.clone()));

        let merge_setting = defaults.merge_configuration.as_ref().unwrap();
        assert_eq!(merge_setting.value(), merge_config);
        assert!(merge_setting.can_override());

        let label_setting = defaults.default_labels.as_ref().unwrap();
        assert_eq!(label_setting.value(), labels);
        assert!(label_setting.can_override());
    }

    #[test]
    fn global_defaults_supports_mixed_override_settings() {
        let mut defaults = GlobalDefaults::new();

        // Security-critical settings that cannot be overridden
        defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));
        defaults.organization_webhooks = Some(OverridableValue::fixed(vec![WebhookConfig {
            url: "https://security.example.com/webhook".to_string(),
            events: vec![WebhookEvent::Push, WebhookEvent::PullRequest],
            active: true,
            secret: Some("webhook_secret".to_string()),
        }]));

        // Flexible settings that teams can customize
        defaults.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));
        defaults.merge_configuration = Some(OverridableValue::overridable(MergeConfig {
            allowed_types: vec![MergeType::Squash],
            merge_commit_message: CommitMessageOption::PullRequestTitle,
            squash_commit_message: CommitMessageOption::PullRequestTitleAndDescription,
        }));

        // Verify security settings are fixed
        assert!(!defaults
            .branch_protection_enabled
            .as_ref()
            .unwrap()
            .can_override());
        assert!(!defaults
            .organization_webhooks
            .as_ref()
            .unwrap()
            .can_override());

        // Verify flexible settings are overridable
        assert!(defaults
            .repository_visibility
            .as_ref()
            .unwrap()
            .can_override());
        assert!(defaults
            .merge_configuration
            .as_ref()
            .unwrap()
            .can_override());
    }

    #[test]
    fn global_defaults_cloning_works_correctly() {
        let mut original = GlobalDefaults::new();
        original.branch_protection_enabled = Some(OverridableValue::fixed(true));
        original.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let cloned = original.clone();

        assert_eq!(
            original.branch_protection_enabled.as_ref().unwrap().value(),
            cloned.branch_protection_enabled.as_ref().unwrap().value()
        );
        assert_eq!(
            original.repository_visibility.as_ref().unwrap().value(),
            cloned.repository_visibility.as_ref().unwrap().value()
        );
    }

    #[test]
    fn global_defaults_equality_works_correctly() {
        let mut defaults1 = GlobalDefaults::new();
        defaults1.branch_protection_enabled = Some(OverridableValue::fixed(true));

        let mut defaults2 = GlobalDefaults::new();
        defaults2.branch_protection_enabled = Some(OverridableValue::fixed(true));

        let mut defaults3 = GlobalDefaults::new();
        defaults3.branch_protection_enabled = Some(OverridableValue::overridable(true));

        assert_eq!(defaults1, defaults2);
        assert_ne!(defaults1, defaults3); // Different override permission
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use serde_json;

    #[test]
    fn overridable_value_serializes_and_deserializes_correctly() {
        let original = OverridableValue::new("test_value".to_string(), true);

        let serialized = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: OverridableValue<String> =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(original.value(), deserialized.value());
        assert_eq!(original.can_override(), deserialized.can_override());
    }

    #[test]
    fn global_defaults_serializes_and_deserializes_correctly() {
        let mut original = GlobalDefaults::new();
        original.branch_protection_enabled = Some(OverridableValue::fixed(true));
        original.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let serialized = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: GlobalDefaults =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(
            original.branch_protection_enabled.as_ref().unwrap().value(),
            deserialized
                .branch_protection_enabled
                .as_ref()
                .unwrap()
                .value()
        );
        assert_eq!(
            original.repository_visibility.as_ref().unwrap().value(),
            deserialized.repository_visibility.as_ref().unwrap().value()
        );
    }

    #[test]
    fn overridable_value_with_complex_types_serializes_correctly() {
        let labels = vec![
            LabelConfig {
                name: "bug".to_string(),
                description: Some("Something isn't working".to_string()),
                color: "d73a4a".to_string(),
            },
            LabelConfig {
                name: "enhancement".to_string(),
                description: Some("New feature or request".to_string()),
                color: "a2eeef".to_string(),
            },
        ];

        let original = OverridableValue::overridable(labels.clone());

        let serialized = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: OverridableValue<Vec<LabelConfig>> =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(original.value(), deserialized.value());
        assert_eq!(original.can_override(), deserialized.can_override());
    }
}

#[cfg(test)]
mod enum_and_struct_tests {
    use super::*;

    #[test]
    fn repository_visibility_enum_works_correctly() {
        let public = RepositoryVisibility::Public;
        let private = RepositoryVisibility::Private;
        let internal = RepositoryVisibility::Internal;

        assert_eq!(format!("{:?}", public), "Public");
        assert_eq!(format!("{:?}", private), "Private");
        assert_eq!(format!("{:?}", internal), "Internal");
    }

    #[test]
    fn merge_type_enum_works_correctly() {
        let merge = MergeType::Merge;
        let squash = MergeType::Squash;
        let rebase = MergeType::Rebase;

        assert_eq!(format!("{:?}", merge), "Merge");
        assert_eq!(format!("{:?}", squash), "Squash");
        assert_eq!(format!("{:?}", rebase), "Rebase");
    }

    #[test]
    fn commit_message_option_enum_works_correctly() {
        let default = CommitMessageOption::DefaultMessage;
        let title = CommitMessageOption::PullRequestTitle;
        let title_desc = CommitMessageOption::PullRequestTitleAndDescription;
        let title_commits = CommitMessageOption::PullRequestTitleAndCommitDetails;

        assert_eq!(format!("{:?}", default), "DefaultMessage");
        assert_eq!(format!("{:?}", title), "PullRequestTitle");
        assert_eq!(
            format!("{:?}", title_desc),
            "PullRequestTitleAndDescription"
        );
        assert_eq!(
            format!("{:?}", title_commits),
            "PullRequestTitleAndCommitDetails"
        );
    }

    #[test]
    fn webhook_event_enum_works_correctly() {
        let push = WebhookEvent::Push;
        let pr = WebhookEvent::PullRequest;
        let issues = WebhookEvent::Issues;

        assert_eq!(format!("{:?}", push), "Push");
        assert_eq!(format!("{:?}", pr), "PullRequest");
        assert_eq!(format!("{:?}", issues), "Issues");
    }

    #[test]
    fn workflow_permission_enum_works_correctly() {
        let none = WorkflowPermission::None;
        let read = WorkflowPermission::Read;
        let write = WorkflowPermission::Write;

        assert_eq!(format!("{:?}", none), "None");
        assert_eq!(format!("{:?}", read), "Read");
        assert_eq!(format!("{:?}", write), "Write");
    }

    #[test]
    fn label_config_creation_and_access_works() {
        let label = LabelConfig::new(
            "bug".to_string(),
            Some("Something isn't working".to_string()),
            "d73a4a".to_string(),
        );

        assert_eq!(label.name, "bug");
        assert_eq!(
            label.description,
            Some("Something isn't working".to_string())
        );
        assert_eq!(label.color, "d73a4a");
    }

    #[test]
    fn webhook_config_creation_and_access_works() {
        let webhook = WebhookConfig::new(
            "https://example.com/webhook".to_string(),
            vec![WebhookEvent::Push, WebhookEvent::PullRequest],
            true,
            Some("secret_key".to_string()),
        );

        assert_eq!(webhook.url, "https://example.com/webhook");
        assert_eq!(
            webhook.events,
            vec![WebhookEvent::Push, WebhookEvent::PullRequest]
        );
        assert!(webhook.active);
        assert_eq!(webhook.secret, Some("secret_key".to_string()));
    }

    #[test]
    fn merge_config_creation_and_access_works() {
        let config = MergeConfig::new(
            vec![MergeType::Squash, MergeType::Merge],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        );

        assert_eq!(
            config.allowed_types,
            vec![MergeType::Squash, MergeType::Merge]
        );
        assert_eq!(
            config.merge_commit_message,
            CommitMessageOption::PullRequestTitle
        );
        assert_eq!(
            config.squash_commit_message,
            CommitMessageOption::PullRequestTitleAndDescription
        );
    }

    #[test]
    fn enums_serialize_and_deserialize_correctly() {
        let visibility = RepositoryVisibility::Private;
        let serialized = serde_json::to_string(&visibility).expect("Should serialize");
        let deserialized: RepositoryVisibility =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(visibility, deserialized);

        let merge_type = MergeType::Squash;
        let serialized = serde_json::to_string(&merge_type).expect("Should serialize");
        let deserialized: MergeType =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(merge_type, deserialized);

        let commit_msg = CommitMessageOption::PullRequestTitleAndDescription;
        let serialized = serde_json::to_string(&commit_msg).expect("Should serialize");
        let deserialized: CommitMessageOption =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(commit_msg, deserialized);

        let webhook_event = WebhookEvent::PullRequest;
        let serialized = serde_json::to_string(&webhook_event).expect("Should serialize");
        let deserialized: WebhookEvent =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(webhook_event, deserialized);

        let workflow_permission = WorkflowPermission::Read;
        let serialized = serde_json::to_string(&workflow_permission).expect("Should serialize");
        let deserialized: WorkflowPermission =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(workflow_permission, deserialized);
    }

    #[test]
    fn structs_serialize_and_deserialize_correctly() {
        let label = LabelConfig {
            name: "bug".to_string(),
            description: Some("Something isn't working".to_string()),
            color: "d73a4a".to_string(),
        };

        let serialized = serde_json::to_string(&label).expect("Should serialize");
        let deserialized: LabelConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(label, deserialized);

        let webhook = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            events: vec![WebhookEvent::Push],
            active: true,
            secret: None,
        };

        let serialized = serde_json::to_string(&webhook).expect("Should serialize");
        let deserialized: WebhookConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(webhook, deserialized);

        let merge_config = MergeConfig {
            allowed_types: vec![MergeType::Squash, MergeType::Merge],
            merge_commit_message: CommitMessageOption::PullRequestTitle,
            squash_commit_message: CommitMessageOption::PullRequestTitleAndDescription,
        };

        let serialized = serde_json::to_string(&merge_config).expect("Should serialize");
        let deserialized: MergeConfig =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(merge_config, deserialized);
    }
}

// Tests for enhanced GlobalDefaults structure matching specification

#[cfg(test)]
mod enhanced_configuration_tests {
    use super::*;
    use crate::organization::{
        ActionSettings, BranchProtectionSettings, CustomProperty, EnvironmentConfig,
        GitHubAppConfig, GlobalDefaultsEnhanced, PullRequestSettings, PushSettings,
        RepositorySettings, WorkflowPermission,
    };

    #[test]
    fn test_action_settings_creation() {
        let settings = ActionSettings::new();
        assert!(settings.enabled.is_none());
        assert!(settings.default_workflow_permissions.is_none());
    }

    #[test]
    fn test_action_settings_default() {
        let settings = ActionSettings::default();
        assert!(settings.enabled.is_none());
        assert!(settings.default_workflow_permissions.is_none());
    }

    #[test]
    fn test_action_settings_serialization() {
        let mut settings = ActionSettings::new();
        settings.enabled = Some(OverridableValue::fixed(true));
        settings.default_workflow_permissions =
            Some(OverridableValue::overridable(WorkflowPermission::Read));

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: ActionSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
        assert_eq!(deserialized.enabled.as_ref().unwrap().value, true);
        assert!(!deserialized.enabled.as_ref().unwrap().can_override());
        assert_eq!(
            deserialized
                .default_workflow_permissions
                .as_ref()
                .unwrap()
                .value,
            WorkflowPermission::Read
        );
        assert!(deserialized
            .default_workflow_permissions
            .as_ref()
            .unwrap()
            .can_override());
    }

    #[test]
    fn test_branch_protection_settings_creation() {
        let protection = BranchProtectionSettings::new();
        assert!(protection.enabled.is_none());
        assert!(protection.require_pull_request_reviews.is_none());
        assert!(protection.required_reviewers.is_none());
    }

    #[test]
    fn test_branch_protection_settings_default() {
        let protection = BranchProtectionSettings::default();
        assert!(protection.enabled.is_none());
        assert!(protection.require_pull_request_reviews.is_none());
        assert!(protection.required_reviewers.is_none());
    }

    #[test]
    fn test_branch_protection_settings_serialization() {
        let mut protection = BranchProtectionSettings::new();
        protection.enabled = Some(OverridableValue::fixed(true));
        protection.require_pull_request_reviews = Some(OverridableValue::fixed(true));
        protection.required_reviewers = Some(OverridableValue::overridable(2));

        let json = serde_json::to_string(&protection).unwrap();
        let deserialized: BranchProtectionSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(protection, deserialized);
        assert_eq!(deserialized.enabled.as_ref().unwrap().value, true);
        assert!(!deserialized.enabled.as_ref().unwrap().can_override());
        assert_eq!(deserialized.required_reviewers.as_ref().unwrap().value, 2);
        assert!(deserialized
            .required_reviewers
            .as_ref()
            .unwrap()
            .can_override());
    }

    #[test]
    fn test_custom_property_creation() {
        let property = CustomProperty::new("team".to_string(), "backend".to_string());
        assert_eq!(property.property_name, "team");
        assert_eq!(property.value, "backend");
    }

    #[test]
    fn test_custom_property_serialization() {
        let property = CustomProperty::new("repository_type".to_string(), "service".to_string());

        let json = serde_json::to_string(&property).unwrap();
        let deserialized: CustomProperty = serde_json::from_str(&json).unwrap();

        assert_eq!(property, deserialized);
        assert_eq!(deserialized.property_name, "repository_type");
        assert_eq!(deserialized.value, "service");
    }

    #[test]
    fn test_environment_config_creation() {
        let env = EnvironmentConfig::new("production".to_string(), None, None, None);
        assert_eq!(env.name, "production");
        assert!(env.required_reviewers.is_none());
        assert!(env.wait_timer.is_none());
        assert!(env.deployment_branch_policy.is_none());
    }

    #[test]
    fn test_environment_config_serialization() {
        let env = EnvironmentConfig::new(
            "staging".to_string(),
            Some(vec!["@team-leads".to_string()]),
            Some(300),
            Some("main".to_string()),
        );

        let json = serde_json::to_string(&env).unwrap();
        let deserialized: EnvironmentConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(env, deserialized);
        assert_eq!(deserialized.name, "staging");
        assert_eq!(
            deserialized.required_reviewers,
            Some(vec!["@team-leads".to_string()])
        );
        assert_eq!(deserialized.wait_timer, Some(300));
        assert_eq!(
            deserialized.deployment_branch_policy,
            Some("main".to_string())
        );
    }

    #[test]
    fn test_github_app_config_creation() {
        let app = GitHubAppConfig::new("dependabot".to_string());
        assert_eq!(app.app_slug, "dependabot");
        assert!(app.required.is_none());
    }

    #[test]
    fn test_github_app_config_serialization() {
        let mut app = GitHubAppConfig::new("codecov".to_string());
        app.required = Some(OverridableValue::fixed(true));

        let json = serde_json::to_string(&app).unwrap();
        let deserialized: GitHubAppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(app, deserialized);
        assert_eq!(deserialized.app_slug, "codecov");
        assert_eq!(deserialized.required.as_ref().unwrap().value, true);
        assert!(!deserialized.required.as_ref().unwrap().can_override());
    }

    #[test]
    fn test_pull_request_settings_creation() {
        let settings = PullRequestSettings::new();
        assert!(settings.delete_branch_on_merge.is_none());
        assert!(settings.allow_squash_merge.is_none());
        assert!(settings.allow_merge_commit.is_none());
    }

    #[test]
    fn test_pull_request_settings_default() {
        let settings = PullRequestSettings::default();
        assert!(settings.delete_branch_on_merge.is_none());
        assert!(settings.allow_squash_merge.is_none());
        assert!(settings.allow_merge_commit.is_none());
    }

    #[test]
    fn test_pull_request_settings_serialization() {
        let mut settings = PullRequestSettings::new();
        settings.delete_branch_on_merge = Some(OverridableValue::overridable(true));
        settings.allow_squash_merge = Some(OverridableValue::fixed(true));
        settings.allow_merge_commit = Some(OverridableValue::fixed(false));

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: PullRequestSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
        assert_eq!(
            deserialized.delete_branch_on_merge.as_ref().unwrap().value,
            true
        );
        assert!(deserialized
            .delete_branch_on_merge
            .as_ref()
            .unwrap()
            .can_override());
        assert_eq!(
            deserialized.allow_squash_merge.as_ref().unwrap().value,
            true
        );
        assert!(!deserialized
            .allow_squash_merge
            .as_ref()
            .unwrap()
            .can_override());
        assert_eq!(
            deserialized.allow_merge_commit.as_ref().unwrap().value,
            false
        );
        assert!(!deserialized
            .allow_merge_commit
            .as_ref()
            .unwrap()
            .can_override());
    }

    #[test]
    fn test_push_settings_creation() {
        let settings = PushSettings::new();
        assert!(settings.allow_force_pushes.is_none());
        assert!(settings.require_signed_commits.is_none());
    }

    #[test]
    fn test_push_settings_default() {
        let settings = PushSettings::default();
        assert!(settings.allow_force_pushes.is_none());
        assert!(settings.require_signed_commits.is_none());
    }

    #[test]
    fn test_push_settings_serialization() {
        let mut settings = PushSettings::new();
        settings.allow_force_pushes = Some(OverridableValue::fixed(false));
        settings.require_signed_commits = Some(OverridableValue::overridable(true));

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: PushSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
        assert_eq!(
            deserialized.allow_force_pushes.as_ref().unwrap().value,
            false
        );
        assert!(!deserialized
            .allow_force_pushes
            .as_ref()
            .unwrap()
            .can_override());
        assert_eq!(
            deserialized.require_signed_commits.as_ref().unwrap().value,
            true
        );
        assert!(deserialized
            .require_signed_commits
            .as_ref()
            .unwrap()
            .can_override());
    }

    #[test]
    fn test_repository_settings_creation() {
        let settings = RepositorySettings::new();
        assert!(settings.issues.is_none());
        assert!(settings.wiki.is_none());
        assert!(settings.projects.is_none());
        assert!(settings.discussions.is_none());
    }

    #[test]
    fn test_repository_settings_default() {
        let settings = RepositorySettings::default();
        assert!(settings.issues.is_none());
        assert!(settings.wiki.is_none());
        assert!(settings.projects.is_none());
        assert!(settings.discussions.is_none());
    }

    #[test]
    fn test_repository_settings_serialization() {
        let mut settings = RepositorySettings::new();
        settings.issues = Some(OverridableValue::overridable(true));
        settings.wiki = Some(OverridableValue::fixed(false));
        settings.projects = Some(OverridableValue::overridable(true));
        settings.discussions = Some(OverridableValue::fixed(true));

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: RepositorySettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
        assert_eq!(deserialized.issues.as_ref().unwrap().value, true);
        assert!(deserialized.issues.as_ref().unwrap().can_override());
        assert_eq!(deserialized.wiki.as_ref().unwrap().value, false);
        assert!(!deserialized.wiki.as_ref().unwrap().can_override());
        assert_eq!(deserialized.projects.as_ref().unwrap().value, true);
        assert!(deserialized.projects.as_ref().unwrap().can_override());
        assert_eq!(deserialized.discussions.as_ref().unwrap().value, true);
        assert!(!deserialized.discussions.as_ref().unwrap().can_override());
    }

    #[test]
    fn test_global_defaults_enhanced_creation() {
        let defaults = GlobalDefaultsEnhanced::new();
        assert!(defaults.actions.is_none());
        assert!(defaults.branch_protection.is_none());
        assert!(defaults.custom_properties.is_none());
        assert!(defaults.environments.is_none());
        assert!(defaults.github_apps.is_none());
        assert!(defaults.pull_requests.is_none());
        assert!(defaults.push.is_none());
        assert!(defaults.repository.is_none());
        assert!(defaults.webhooks.is_none());
    }

    #[test]
    fn test_global_defaults_enhanced_default() {
        let defaults = GlobalDefaultsEnhanced::default();
        assert!(defaults.actions.is_none());
        assert!(defaults.branch_protection.is_none());
        assert!(defaults.custom_properties.is_none());
        assert!(defaults.environments.is_none());
        assert!(defaults.github_apps.is_none());
        assert!(defaults.pull_requests.is_none());
        assert!(defaults.push.is_none());
        assert!(defaults.repository.is_none());
        assert!(defaults.webhooks.is_none());
    }

    #[test]
    fn test_global_defaults_enhanced_comprehensive_configuration() {
        let mut defaults = GlobalDefaultsEnhanced::new();

        // Configure Actions
        let mut actions = ActionSettings::new();
        actions.enabled = Some(OverridableValue::fixed(true));
        actions.default_workflow_permissions =
            Some(OverridableValue::overridable(WorkflowPermission::Read));
        defaults.actions = Some(actions);

        // Configure Branch Protection
        let mut protection = BranchProtectionSettings::new();
        protection.enabled = Some(OverridableValue::fixed(true));
        protection.require_pull_request_reviews = Some(OverridableValue::fixed(true));
        protection.required_reviewers = Some(OverridableValue::overridable(2));
        defaults.branch_protection = Some(protection);

        // Configure Custom Properties
        defaults.custom_properties = Some(vec![
            CustomProperty::new("team".to_string(), "platform".to_string()),
            CustomProperty::new("classification".to_string(), "internal".to_string()),
        ]);

        // Configure Environments
        let prod_env = EnvironmentConfig::new(
            "production".to_string(),
            Some(vec!["@security-team".to_string()]),
            Some(300),
            Some("main".to_string()),
        );
        let staging_env = EnvironmentConfig::new(
            "staging".to_string(),
            Some(vec!["@team-leads".to_string()]),
            None,
            None,
        );
        defaults.environments = Some(vec![prod_env, staging_env]);

        // Configure GitHub Apps
        let mut dependabot = GitHubAppConfig::new("dependabot".to_string());
        dependabot.required = Some(OverridableValue::fixed(true));
        let mut codecov = GitHubAppConfig::new("codecov".to_string());
        codecov.required = Some(OverridableValue::overridable(false));
        defaults.github_apps = Some(vec![dependabot, codecov]);

        // Configure Pull Requests
        let mut pr_settings = PullRequestSettings::new();
        pr_settings.delete_branch_on_merge = Some(OverridableValue::overridable(true));
        pr_settings.allow_squash_merge = Some(OverridableValue::fixed(true));
        pr_settings.allow_merge_commit = Some(OverridableValue::fixed(false));
        defaults.pull_requests = Some(pr_settings);

        // Configure Push settings
        let mut push_settings = PushSettings::new();
        push_settings.allow_force_pushes = Some(OverridableValue::fixed(false));
        push_settings.require_signed_commits = Some(OverridableValue::overridable(true));
        defaults.push = Some(push_settings);

        // Configure Repository settings
        let mut repo_settings = RepositorySettings::new();
        repo_settings.issues = Some(OverridableValue::overridable(true));
        repo_settings.wiki = Some(OverridableValue::fixed(false));
        repo_settings.projects = Some(OverridableValue::overridable(true));
        repo_settings.discussions = Some(OverridableValue::fixed(false));
        defaults.repository = Some(repo_settings);

        // Configure Webhooks
        let webhook = WebhookConfig::new(
            "https://security.example.com/webhook".to_string(),
            vec![WebhookEvent::Push, WebhookEvent::PullRequest],
            true,
            Some("webhook_secret".to_string()),
        );
        defaults.webhooks = Some(vec![webhook]);

        // Verify all fields are configured
        assert!(defaults.actions.is_some());
        assert!(defaults.branch_protection.is_some());
        assert!(defaults.custom_properties.is_some());
        assert!(defaults.environments.is_some());
        assert!(defaults.github_apps.is_some());
        assert!(defaults.pull_requests.is_some());
        assert!(defaults.push.is_some());
        assert!(defaults.repository.is_some());
        assert!(defaults.webhooks.is_some());

        // Verify specific configurations
        let actions = defaults.actions.as_ref().unwrap();
        assert_eq!(actions.enabled.as_ref().unwrap().value, true);
        assert!(!actions.enabled.as_ref().unwrap().can_override());

        let custom_props = defaults.custom_properties.as_ref().unwrap();
        assert_eq!(custom_props.len(), 2);
        assert_eq!(custom_props[0].property_name, "team");
        assert_eq!(custom_props[0].value, "platform");

        let environments = defaults.environments.as_ref().unwrap();
        assert_eq!(environments.len(), 2);
        assert_eq!(environments[0].name, "production");
        assert_eq!(
            environments[0].required_reviewers,
            Some(vec!["@security-team".to_string()])
        );
        assert_eq!(environments[0].wait_timer, Some(300));
        assert_eq!(
            environments[0].deployment_branch_policy,
            Some("main".to_string())
        );

        let github_apps = defaults.github_apps.as_ref().unwrap();
        assert_eq!(github_apps.len(), 2);
        assert_eq!(github_apps[0].app_slug, "dependabot");
        assert!(!github_apps[0].required.as_ref().unwrap().can_override());

        let webhooks = defaults.webhooks.as_ref().unwrap();
        assert_eq!(webhooks.len(), 1);
        assert_eq!(webhooks[0].url, "https://security.example.com/webhook");
    }

    #[test]
    fn test_global_defaults_enhanced_serialization() {
        let mut defaults = GlobalDefaultsEnhanced::new();

        // Configure a subset of fields for serialization test
        let mut actions = ActionSettings::new();
        actions.enabled = Some(OverridableValue::fixed(true));
        defaults.actions = Some(actions);

        defaults.custom_properties = Some(vec![CustomProperty::new(
            "team".to_string(),
            "backend".to_string(),
        )]);

        let json = serde_json::to_string(&defaults).unwrap();
        let deserialized: GlobalDefaultsEnhanced = serde_json::from_str(&json).unwrap();

        assert_eq!(defaults, deserialized);
        assert!(deserialized.actions.is_some());
        assert_eq!(
            deserialized
                .actions
                .as_ref()
                .unwrap()
                .enabled
                .as_ref()
                .unwrap()
                .value,
            true
        );
        assert!(deserialized.custom_properties.is_some());
        assert_eq!(deserialized.custom_properties.as_ref().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod team_config_tests {
    use super::*;

    #[test]
    fn new_creates_empty_team_config() {
        let team_config = TeamConfig::new();

        assert!(team_config.repository_visibility.is_none());
        assert!(team_config.branch_protection_enabled.is_none());
        assert!(team_config.merge_configuration.is_none());
        assert!(team_config.team_webhooks.is_none());
        assert!(team_config.team_github_apps.is_none());
        assert!(team_config.team_labels.is_none());
        assert!(team_config.team_environments.is_none());
    }

    #[test]
    fn default_creates_empty_team_config() {
        let team_config = TeamConfig::default();
        let new_config = TeamConfig::new();

        assert_eq!(team_config, new_config);
    }

    #[test]
    fn has_overrides_returns_false_for_empty_config() {
        let team_config = TeamConfig::new();
        assert!(!team_config.has_overrides());
    }

    #[test]
    fn has_overrides_returns_true_when_repository_visibility_set() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);

        assert!(team_config.has_overrides());
    }

    #[test]
    fn has_overrides_returns_true_when_branch_protection_set() {
        let mut team_config = TeamConfig::new();
        team_config.branch_protection_enabled = Some(false);

        assert!(team_config.has_overrides());
    }

    #[test]
    fn has_overrides_returns_true_when_merge_configuration_set() {
        let mut team_config = TeamConfig::new();
        team_config.merge_configuration = Some(MergeConfig::new(
            vec![MergeType::Squash],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        ));

        assert!(team_config.has_overrides());
    }

    #[test]
    fn has_overrides_returns_false_for_additive_only_config() {
        let mut team_config = TeamConfig::new();
        team_config.team_webhooks = Some(vec![WebhookConfig::new(
            "https://team.example.com/webhook".to_string(),
            vec![WebhookEvent::Push],
            true,
            None,
        )]);
        team_config.team_github_apps = Some(vec!["team-app".to_string()]);

        assert!(!team_config.has_overrides());
    }

    #[test]
    fn has_additions_returns_false_for_empty_config() {
        let team_config = TeamConfig::new();
        assert!(!team_config.has_additions());
    }

    #[test]
    fn has_additions_returns_true_when_team_webhooks_set() {
        let mut team_config = TeamConfig::new();
        team_config.team_webhooks = Some(vec![WebhookConfig::new(
            "https://team.example.com/webhook".to_string(),
            vec![WebhookEvent::Push],
            true,
            None,
        )]);

        assert!(team_config.has_additions());
    }

    #[test]
    fn has_additions_returns_true_when_team_github_apps_set() {
        let mut team_config = TeamConfig::new();
        team_config.team_github_apps = Some(vec!["team-app".to_string()]);

        assert!(team_config.has_additions());
    }

    #[test]
    fn has_additions_returns_true_when_team_labels_set() {
        let mut team_config = TeamConfig::new();
        team_config.team_labels = Some(vec![LabelConfig::new(
            "team-specific".to_string(),
            Some("Team specific label".to_string()),
            "ff0000".to_string(),
        )]);

        assert!(team_config.has_additions());
    }

    #[test]
    fn has_additions_returns_true_when_team_environments_set() {
        let mut team_config = TeamConfig::new();
        team_config.team_environments = Some(vec![EnvironmentConfig::new(
            "team-staging".to_string(),
            Some(vec!["@team-leads".to_string()]),
            None,
            None,
        )]);

        assert!(team_config.has_additions());
    }

    #[test]
    fn has_additions_returns_false_for_overrides_only_config() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);
        team_config.branch_protection_enabled = Some(true);

        assert!(!team_config.has_additions());
    }

    #[test]
    fn validate_overrides_succeeds_with_empty_team_config() {
        let team_config = TeamConfig::new();
        let global_defaults = GlobalDefaults::new();

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_overrides_succeeds_when_global_allows_repository_visibility_override() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_overrides_fails_when_global_prohibits_repository_visibility_override() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::fixed(RepositoryVisibility::Private));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_err());

        if let Err(ConfigurationError::OverrideNotAllowed {
            field,
            attempted_value,
            global_value,
        }) = result
        {
            assert_eq!(field, "repository_visibility");
            assert_eq!(attempted_value, "Public");
            assert_eq!(global_value, "Private");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }

    #[test]
    fn validate_overrides_succeeds_when_global_allows_branch_protection_override() {
        let mut team_config = TeamConfig::new();
        team_config.branch_protection_enabled = Some(false);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.branch_protection_enabled = Some(OverridableValue::overridable(true));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_overrides_fails_when_global_prohibits_branch_protection_override() {
        let mut team_config = TeamConfig::new();
        team_config.branch_protection_enabled = Some(false);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_err());

        if let Err(ConfigurationError::OverrideNotAllowed {
            field,
            attempted_value,
            global_value,
        }) = result
        {
            assert_eq!(field, "branch_protection_enabled");
            assert_eq!(attempted_value, "false");
            assert_eq!(global_value, "true");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }

    #[test]
    fn validate_overrides_succeeds_when_global_allows_merge_configuration_override() {
        let team_merge_config = MergeConfig::new(
            vec![MergeType::Squash],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        );
        let mut team_config = TeamConfig::new();
        team_config.merge_configuration = Some(team_merge_config.clone());

        let global_merge_config = MergeConfig::new(
            vec![MergeType::Merge, MergeType::Squash],
            CommitMessageOption::DefaultMessage,
            CommitMessageOption::DefaultMessage,
        );
        let mut global_defaults = GlobalDefaults::new();
        global_defaults.merge_configuration =
            Some(OverridableValue::overridable(global_merge_config));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_overrides_fails_when_global_prohibits_merge_configuration_override() {
        let team_merge_config = MergeConfig::new(
            vec![MergeType::Squash],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        );
        let mut team_config = TeamConfig::new();
        team_config.merge_configuration = Some(team_merge_config);

        let global_merge_config = MergeConfig::new(
            vec![MergeType::Merge],
            CommitMessageOption::DefaultMessage,
            CommitMessageOption::DefaultMessage,
        );
        let mut global_defaults = GlobalDefaults::new();
        global_defaults.merge_configuration = Some(OverridableValue::fixed(global_merge_config));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_err());

        if let Err(ConfigurationError::OverrideNotAllowed { field, .. }) = result {
            assert_eq!(field, "merge_configuration");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }

    #[test]
    fn validate_overrides_ignores_additive_configurations() {
        let mut team_config = TeamConfig::new();
        team_config.team_webhooks = Some(vec![WebhookConfig::new(
            "https://team.example.com/webhook".to_string(),
            vec![WebhookEvent::Push],
            true,
            None,
        )]);
        team_config.team_github_apps = Some(vec!["team-app".to_string()]);
        team_config.team_labels = Some(vec![LabelConfig::new(
            "team-specific".to_string(),
            None,
            "ff0000".to_string(),
        )]);
        team_config.team_environments = Some(vec![EnvironmentConfig::new(
            "team-staging".to_string(),
            None,
            None,
            None,
        )]);

        let global_defaults = GlobalDefaults::new();

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_overrides_handles_multiple_override_violations() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);
        team_config.branch_protection_enabled = Some(false);

        let mut global_defaults = GlobalDefaults::new();
        global_defaults.repository_visibility =
            Some(OverridableValue::fixed(RepositoryVisibility::Private));
        global_defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));

        let result = team_config.validate_overrides(&global_defaults);
        assert!(result.is_err());

        // Should fail on the first violation encountered
        if let Err(ConfigurationError::OverrideNotAllowed { field, .. }) = result {
            // Either field could be reported first, both are violations
            assert!(field == "repository_visibility" || field == "branch_protection_enabled");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }

    #[test]
    fn serialization_roundtrip_preserves_team_config() {
        let mut team_config = TeamConfig::new();
        team_config.repository_visibility = Some(RepositoryVisibility::Public);
        team_config.branch_protection_enabled = Some(true);
        team_config.merge_configuration = Some(MergeConfig::new(
            vec![MergeType::Squash, MergeType::Merge],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        ));
        team_config.team_webhooks = Some(vec![
            WebhookConfig::new(
                "https://team.example.com/webhook1".to_string(),
                vec![WebhookEvent::Push, WebhookEvent::PullRequest],
                true,
                Some("secret1".to_string()),
            ),
            WebhookConfig::new(
                "https://team.example.com/webhook2".to_string(),
                vec![WebhookEvent::Issues],
                false,
                None,
            ),
        ]);
        team_config.team_github_apps =
            Some(vec!["team-app-1".to_string(), "team-app-2".to_string()]);
        team_config.team_labels = Some(vec![
            LabelConfig::new(
                "team-bug".to_string(),
                Some("Team specific bug label".to_string()),
                "ff0000".to_string(),
            ),
            LabelConfig::new(
                "team-feature".to_string(),
                Some("Team specific feature label".to_string()),
                "00ff00".to_string(),
            ),
        ]);
        team_config.team_environments = Some(vec![
            EnvironmentConfig::new(
                "team-staging".to_string(),
                Some(vec!["@team-leads".to_string()]),
                None,
                Some("main".to_string()),
            ),
            EnvironmentConfig::new(
                "team-production".to_string(),
                Some(vec![
                    "@team-leads".to_string(),
                    "@security-team".to_string(),
                ]),
                Some(300),
                Some("main".to_string()),
            ),
        ]);

        // Serialize to JSON
        let json = serde_json::to_string(&team_config).expect("Failed to serialize to JSON");
        let deserialized: TeamConfig =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        assert_eq!(team_config, deserialized);

        // Serialize to TOML
        let toml = toml::to_string(&team_config).expect("Failed to serialize to TOML");
        let deserialized: TeamConfig =
            toml::from_str(&toml).expect("Failed to deserialize from TOML");
        assert_eq!(team_config, deserialized);
    }
}

#[cfg(test)]
mod environment_config_tests {
    use super::*;

    #[test]
    fn new_creates_environment_with_all_fields() {
        let env = EnvironmentConfig::new(
            "production".to_string(),
            Some(vec![
                "@team-leads".to_string(),
                "@security-team".to_string(),
            ]),
            Some(300),
            Some("main".to_string()),
        );

        assert_eq!(env.name, "production");
        assert_eq!(
            env.required_reviewers,
            Some(vec![
                "@team-leads".to_string(),
                "@security-team".to_string()
            ])
        );
        assert_eq!(env.wait_timer, Some(300));
        assert_eq!(env.deployment_branch_policy, Some("main".to_string()));
    }

    #[test]
    fn new_creates_environment_with_minimal_configuration() {
        let env = EnvironmentConfig::new("staging".to_string(), None, None, None);

        assert_eq!(env.name, "staging");
        assert!(env.required_reviewers.is_none());
        assert!(env.wait_timer.is_none());
        assert!(env.deployment_branch_policy.is_none());
    }

    #[test]
    fn environment_config_serialization_roundtrip() {
        let env = EnvironmentConfig::new(
            "test-env".to_string(),
            Some(vec!["@reviewer1".to_string(), "@reviewer2".to_string()]),
            Some(180),
            Some("release/*".to_string()),
        );

        // JSON roundtrip
        let json = serde_json::to_string(&env).expect("Failed to serialize to JSON");
        let deserialized: EnvironmentConfig =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        assert_eq!(env, deserialized);

        // TOML roundtrip
        let toml = toml::to_string(&env).expect("Failed to serialize to TOML");
        let deserialized: EnvironmentConfig =
            toml::from_str(&toml).expect("Failed to deserialize from TOML");
        assert_eq!(env, deserialized);
    }
}

#[cfg(test)]
mod configuration_error_tests {
    use super::*;

    #[test]
    fn override_not_allowed_error_contains_correct_details() {
        let error = ConfigurationError::OverrideNotAllowed {
            field: "repository_visibility".to_string(),
            attempted_value: "Public".to_string(),
            global_value: "Private".to_string(),
        };

        if let ConfigurationError::OverrideNotAllowed {
            field,
            attempted_value,
            global_value,
        } = error
        {
            assert_eq!(field, "repository_visibility");
            assert_eq!(attempted_value, "Public");
            assert_eq!(global_value, "Private");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }

    #[test]
    fn invalid_value_error_contains_correct_details() {
        let error = ConfigurationError::InvalidValue {
            field: "webhook_url".to_string(),
            value: "not-a-url".to_string(),
            reason: "Invalid URL format".to_string(),
        };

        if let ConfigurationError::InvalidValue {
            field,
            value,
            reason,
        } = error
        {
            assert_eq!(field, "webhook_url");
            assert_eq!(value, "not-a-url");
            assert_eq!(reason, "Invalid URL format");
        } else {
            panic!("Expected InvalidValue error");
        }
    }

    #[test]
    fn required_field_missing_error_contains_correct_details() {
        let error = ConfigurationError::RequiredFieldMissing {
            field: "branch_protection_enabled".to_string(),
            context: "global defaults".to_string(),
        };

        if let ConfigurationError::RequiredFieldMissing { field, context } = error {
            assert_eq!(field, "branch_protection_enabled");
            assert_eq!(context, "global defaults");
        } else {
            panic!("Expected RequiredFieldMissing error");
        }
    }

    #[test]
    fn format_error_contains_correct_details() {
        let error = ConfigurationError::FormatError {
            file: "team-config.toml".to_string(),
            error: "Invalid TOML syntax at line 5".to_string(),
        };

        if let ConfigurationError::FormatError { file, error } = error {
            assert_eq!(file, "team-config.toml");
            assert_eq!(error, "Invalid TOML syntax at line 5");
        } else {
            panic!("Expected FormatError error");
        }
    }

    #[test]
    fn configuration_errors_are_cloneable() {
        let error = ConfigurationError::OverrideNotAllowed {
            field: "test".to_string(),
            attempted_value: "value1".to_string(),
            global_value: "value2".to_string(),
        };

        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn configuration_errors_are_debuggable() {
        let error = ConfigurationError::InvalidValue {
            field: "test_field".to_string(),
            value: "test_value".to_string(),
            reason: "test_reason".to_string(),
        };

        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("InvalidValue"));
        assert!(debug_string.contains("test_field"));
        assert!(debug_string.contains("test_value"));
        assert!(debug_string.contains("test_reason"));
    }
}

#[cfg(test)]
mod repository_type_config_tests {
    use super::*;
    use crate::organization::{
        BranchProtectionSettings, CustomProperty, GitHubAppConfig, PullRequestSettings,
        RepositorySettings,
    };

    #[test]
    fn new_creates_empty_repository_type_config() {
        let config = RepositoryTypeConfig::new();

        assert!(config.branch_protection.is_none());
        assert!(config.custom_properties.is_none());
        assert!(config.environments.is_none());
        assert!(config.github_apps.is_none());
        assert!(config.labels.is_none());
        assert!(config.pull_requests.is_none());
        assert!(config.repository.is_none());
        assert!(config.webhooks.is_none());
    }

    #[test]
    fn default_creates_empty_repository_type_config() {
        let config = RepositoryTypeConfig::default();
        let new_config = RepositoryTypeConfig::new();

        assert_eq!(config, new_config);
    }

    #[test]
    fn has_settings_returns_false_for_empty_config() {
        let config = RepositoryTypeConfig::new();
        assert!(!config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_branch_protection_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.branch_protection = Some(BranchProtectionSettings::new());
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_custom_properties_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.custom_properties = Some(vec![]);
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_environments_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.environments = Some(vec![]);
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_github_apps_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.github_apps = Some(vec![]);
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_labels_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![]);
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_pull_requests_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.pull_requests = Some(PullRequestSettings::new());
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_repository_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.repository = Some(RepositorySettings::new());
        assert!(config.has_settings());
    }

    #[test]
    fn has_settings_returns_true_when_webhooks_is_set() {
        let mut config = RepositoryTypeConfig::new();
        config.webhooks = Some(vec![]);
        assert!(config.has_settings());
    }

    #[test]
    fn count_additive_items_returns_zero_for_empty_config() {
        let config = RepositoryTypeConfig::new();
        assert_eq!(config.count_additive_items(), 0);
    }

    #[test]
    fn count_additive_items_counts_labels() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![
            LabelConfig::new(
                "bug".to_string(),
                Some("Bug report".to_string()),
                "d73a4a".to_string(),
            ),
            LabelConfig::new(
                "enhancement".to_string(),
                Some("Enhancement request".to_string()),
                "a2eeef".to_string(),
            ),
        ]);

        assert_eq!(config.count_additive_items(), 2);
    }

    #[test]
    fn count_additive_items_counts_webhooks() {
        let mut config = RepositoryTypeConfig::new();
        config.webhooks = Some(vec![
            WebhookConfig {
                url: "https://example.com/hook1".to_string(),
                events: vec![WebhookEvent::Push],
                active: true,
                secret: None,
            },
            WebhookConfig {
                url: "https://example.com/hook2".to_string(),
                events: vec![WebhookEvent::PullRequest],
                active: true,
                secret: None,
            },
        ]);

        assert_eq!(config.count_additive_items(), 2);
    }

    #[test]
    fn count_additive_items_counts_environments() {
        let mut config = RepositoryTypeConfig::new();
        config.environments = Some(vec![
            EnvironmentConfig::new("production".to_string(), None, None, None),
            EnvironmentConfig::new("staging".to_string(), None, None, None),
        ]);

        assert_eq!(config.count_additive_items(), 2);
    }

    #[test]
    fn count_additive_items_counts_github_apps() {
        let mut config = RepositoryTypeConfig::new();
        config.github_apps = Some(vec![
            GitHubAppConfig::new("dependabot".to_string()),
            GitHubAppConfig::new("security-scan".to_string()),
        ]);

        assert_eq!(config.count_additive_items(), 2);
    }

    #[test]
    fn count_additive_items_counts_custom_properties() {
        let mut config = RepositoryTypeConfig::new();
        config.custom_properties = Some(vec![
            CustomProperty::new("type".to_string(), "documentation".to_string()),
            CustomProperty::new("owner".to_string(), "platform-team".to_string()),
        ]);

        assert_eq!(config.count_additive_items(), 2);
    }

    #[test]
    fn count_additive_items_counts_all_types() {
        let mut config = RepositoryTypeConfig::new();

        // Add 2 labels
        config.labels = Some(vec![
            LabelConfig::new("bug".to_string(), None, "d73a4a".to_string()),
            LabelConfig::new("enhancement".to_string(), None, "a2eeef".to_string()),
        ]);

        // Add 1 webhook
        config.webhooks = Some(vec![WebhookConfig {
            url: "https://example.com/hook".to_string(),
            events: vec![WebhookEvent::Push],
            active: true,
            secret: None,
        }]);

        // Add 1 environment
        config.environments = Some(vec![EnvironmentConfig::new(
            "production".to_string(),
            None,
            None,
            None,
        )]);

        // Add 2 GitHub apps
        config.github_apps = Some(vec![
            GitHubAppConfig::new("dependabot".to_string()),
            GitHubAppConfig::new("security-scan".to_string()),
        ]);

        // Add 1 custom property
        config.custom_properties = Some(vec![CustomProperty::new(
            "type".to_string(),
            "documentation".to_string(),
        )]);

        // Total: 2 + 1 + 1 + 2 + 1 = 7
        assert_eq!(config.count_additive_items(), 7);
    }

    #[test]
    fn validate_succeeds_for_empty_config() {
        let config = RepositoryTypeConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_succeeds_for_valid_labels() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![
            LabelConfig::new("bug".to_string(), None, "d73a4a".to_string()),
            LabelConfig::new(
                "enhancement".to_string(),
                Some("New feature".to_string()),
                "a2eeef".to_string(),
            ),
        ]);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_empty_label_name() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "".to_string(),
            None,
            "d73a4a".to_string(),
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_fails_for_empty_label_color() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "bug".to_string(),
            None,
            "".to_string(),
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_fails_for_invalid_label_color_format() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "bug".to_string(),
            None,
            "invalid-color".to_string(),
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_fails_for_short_label_color() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "bug".to_string(),
            None,
            "123".to_string(),
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_succeeds_for_valid_webhooks() {
        let mut config = RepositoryTypeConfig::new();
        config.webhooks = Some(vec![WebhookConfig {
            url: "https://example.com/hook".to_string(),
            events: vec![WebhookEvent::Push],
            active: true,
            secret: Some("secret".to_string()),
        }]);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_empty_webhook_url() {
        let mut config = RepositoryTypeConfig::new();
        config.webhooks = Some(vec![WebhookConfig {
            url: "".to_string(),
            events: vec![WebhookEvent::Push],
            active: true,
            secret: None,
        }]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_fails_for_non_https_webhook_url() {
        let mut config = RepositoryTypeConfig::new();
        config.webhooks = Some(vec![WebhookConfig {
            url: "http://example.com/hook".to_string(),
            events: vec![WebhookEvent::Push],
            active: true,
            secret: None,
        }]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_succeeds_for_valid_environments() {
        let mut config = RepositoryTypeConfig::new();
        config.environments = Some(vec![
            EnvironmentConfig::new("production".to_string(), None, None, None),
            EnvironmentConfig::new("staging".to_string(), None, None, None),
        ]);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_empty_environment_name() {
        let mut config = RepositoryTypeConfig::new();
        config.environments = Some(vec![EnvironmentConfig::new(
            "".to_string(),
            None,
            None,
            None,
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_succeeds_for_valid_github_apps() {
        let mut config = RepositoryTypeConfig::new();
        config.github_apps = Some(vec![
            GitHubAppConfig::new("dependabot".to_string()),
            GitHubAppConfig::new("security-scan".to_string()),
        ]);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_empty_github_app_name() {
        let mut config = RepositoryTypeConfig::new();
        config.github_apps = Some(vec![GitHubAppConfig::new("".to_string())]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn validate_succeeds_for_valid_custom_properties() {
        let mut config = RepositoryTypeConfig::new();
        config.custom_properties = Some(vec![CustomProperty::new(
            "type".to_string(),
            "documentation".to_string(),
        )]);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_fails_for_empty_custom_property_name() {
        let mut config = RepositoryTypeConfig::new();
        config.custom_properties = Some(vec![CustomProperty::new(
            "".to_string(),
            "documentation".to_string(),
        )]);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigurationError::InvalidValue { .. }
        ));
    }

    #[test]
    fn serialization_roundtrip_toml() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "bug".to_string(),
            Some("Something isn't working".to_string()),
            "d73a4a".to_string(),
        )]);
        config.webhooks = Some(vec![WebhookConfig {
            url: "https://example.com/hook".to_string(),
            events: vec![WebhookEvent::Push, WebhookEvent::PullRequest],
            active: true,
            secret: Some("webhook_secret".to_string()),
        }]);

        let toml_str = toml::to_string(&config).expect("Should serialize to TOML");
        let deserialized: RepositoryTypeConfig =
            toml::from_str(&toml_str).expect("Should deserialize from TOML");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn serialization_roundtrip_json() {
        let mut config = RepositoryTypeConfig::new();
        config.environments = Some(vec![EnvironmentConfig::new(
            "production".to_string(),
            None,
            None,
            None,
        )]);
        config.github_apps = Some(vec![GitHubAppConfig::new("dependabot".to_string())]);
        config.custom_properties = Some(vec![CustomProperty::new(
            "type".to_string(),
            "library".to_string(),
        )]);

        let json_str = serde_json::to_string(&config).expect("Should serialize to JSON");
        let deserialized: RepositoryTypeConfig =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn complex_repository_type_config() {
        let mut config = RepositoryTypeConfig::new();

        // Repository settings
        let mut repo_settings = RepositorySettings::new();
        repo_settings.issues = Some(OverridableValue::overridable(true));
        repo_settings.wiki = Some(OverridableValue::fixed(false));
        config.repository = Some(repo_settings);

        // Pull request settings
        let mut pr_settings = PullRequestSettings::new();
        pr_settings.allow_squash_merge = Some(OverridableValue::fixed(false));
        config.pull_requests = Some(pr_settings);

        // Labels
        config.labels = Some(vec![
            LabelConfig::new(
                "breaking-change".to_string(),
                Some("Breaking API changes".to_string()),
                "d73a4a".to_string(),
            ),
            LabelConfig::new(
                "enhancement".to_string(),
                Some("New feature or request".to_string()),
                "a2eeef".to_string(),
            ),
        ]);

        // Environments
        config.environments = Some(vec![EnvironmentConfig::new(
            "production".to_string(),
            None,
            None,
            None,
        )]);

        // GitHub Apps
        config.github_apps = Some(vec![
            GitHubAppConfig::new("dependabot".to_string()),
            GitHubAppConfig::new("security-scan".to_string()),
        ]);

        // Validation should pass
        assert!(config.validate().is_ok());

        // Should have settings
        assert!(config.has_settings());

        // Should count additive items: 2 labels + 1 environment + 2 GitHub apps = 5
        assert_eq!(config.count_additive_items(), 5);

        // Should serialize/deserialize correctly
        let json_str = serde_json::to_string(&config).expect("Should serialize to JSON");
        let deserialized: RepositoryTypeConfig =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn repository_type_config_is_cloneable() {
        let mut config = RepositoryTypeConfig::new();
        config.labels = Some(vec![LabelConfig::new(
            "test".to_string(),
            None,
            "ffffff".to_string(),
        )]);

        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn repository_type_config_is_debuggable() {
        let config = RepositoryTypeConfig::new();
        let debug_string = format!("{:?}", config);
        assert!(debug_string.contains("RepositoryTypeConfig"));
    }
}

#[cfg(test)]
mod template_config_tests {
    use super::*;

    #[test]
    fn repository_type_policy_serialization() {
        // Test Fixed policy
        let fixed = RepositoryTypePolicy::Fixed;
        let json_fixed = serde_json::to_string(&fixed).expect("Should serialize Fixed");
        assert_eq!(json_fixed, "\"fixed\"");

        let deserialized_fixed: RepositoryTypePolicy =
            serde_json::from_str(&json_fixed).expect("Should deserialize Fixed");
        assert_eq!(deserialized_fixed, RepositoryTypePolicy::Fixed);

        // Test Preferable policy
        let preferable = RepositoryTypePolicy::Preferable;
        let json_preferable =
            serde_json::to_string(&preferable).expect("Should serialize Preferable");
        assert_eq!(json_preferable, "\"preferable\"");

        let deserialized_preferable: RepositoryTypePolicy =
            serde_json::from_str(&json_preferable).expect("Should deserialize Preferable");
        assert_eq!(deserialized_preferable, RepositoryTypePolicy::Preferable);
    }

    #[test]
    fn repository_type_spec_new_and_accessors() {
        let spec = RepositoryTypeSpec::new("microservice".to_string(), RepositoryTypePolicy::Fixed);

        assert_eq!(spec.repository_type(), "microservice");
        assert_eq!(spec.policy(), &RepositoryTypePolicy::Fixed);
        assert!(!spec.can_override());
    }

    #[test]
    fn repository_type_spec_can_override() {
        let fixed_spec =
            RepositoryTypeSpec::new("service".to_string(), RepositoryTypePolicy::Fixed);
        let flexible_spec =
            RepositoryTypeSpec::new("library".to_string(), RepositoryTypePolicy::Preferable);

        assert!(!fixed_spec.can_override());
        assert!(flexible_spec.can_override());
    }

    #[test]
    fn repository_type_spec_serialization() {
        let spec = RepositoryTypeSpec::new("api".to_string(), RepositoryTypePolicy::Preferable);

        let json_str = serde_json::to_string(&spec).expect("Should serialize to JSON");
        let deserialized: RepositoryTypeSpec =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        assert_eq!(spec, deserialized);
        assert_eq!(deserialized.repository_type(), "api");
        assert_eq!(deserialized.policy(), &RepositoryTypePolicy::Preferable);
    }

    #[test]
    fn template_metadata_new_and_accessors() {
        let metadata = TemplateMetadata::new(
            "rust-service".to_string(),
            "Production Rust service template".to_string(),
            "Platform Team".to_string(),
            vec![
                "rust".to_string(),
                "microservice".to_string(),
                "backend".to_string(),
            ],
        );

        assert_eq!(metadata.name(), "rust-service");
        assert_eq!(metadata.description(), "Production Rust service template");
        assert_eq!(metadata.author(), "Platform Team");
        assert_eq!(metadata.tags().len(), 3);
        assert_eq!(metadata.tags(), &["rust", "microservice", "backend"]);
    }

    #[test]
    fn template_metadata_has_tag() {
        let metadata = TemplateMetadata::new(
            "web-app".to_string(),
            "React web application".to_string(),
            "Frontend Team".to_string(),
            vec![
                "react".to_string(),
                "web".to_string(),
                "frontend".to_string(),
            ],
        );

        assert!(metadata.has_tag("react"));
        assert!(metadata.has_tag("web"));
        assert!(metadata.has_tag("frontend"));
        assert!(!metadata.has_tag("backend"));
        assert!(!metadata.has_tag("rust"));
    }

    #[test]
    fn template_metadata_serialization() {
        let metadata = TemplateMetadata::new(
            "test-template".to_string(),
            "Test template description".to_string(),
            "Test Team".to_string(),
            vec!["test".to_string(), "example".to_string()],
        );

        let json_str = serde_json::to_string(&metadata).expect("Should serialize to JSON");
        let deserialized: TemplateMetadata =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        assert_eq!(metadata, deserialized);
        assert_eq!(deserialized.name(), "test-template");
        assert_eq!(deserialized.tags().len(), 2);
    }

    #[test]
    fn template_variable_new_and_accessors() {
        let var = TemplateVariable::new(
            "Service name for the microservice".to_string(),
            Some("user-service".to_string()),
            Some("default-service".to_string()),
            Some(true),
        );

        assert_eq!(var.description(), "Service name for the microservice");
        assert_eq!(var.example(), Some("user-service"));
        assert_eq!(var.default(), Some("default-service"));
        assert_eq!(var.required(), Some(true));
        assert!(var.has_default());
    }

    #[test]
    fn template_variable_optional_fields() {
        let minimal_var =
            TemplateVariable::new("Optional configuration".to_string(), None, None, None);

        assert_eq!(minimal_var.description(), "Optional configuration");
        assert_eq!(minimal_var.example(), None);
        assert_eq!(minimal_var.default(), None);
        assert_eq!(minimal_var.required(), None);
        assert!(!minimal_var.has_default());
    }

    #[test]
    fn template_variable_has_default() {
        let with_default = TemplateVariable::new(
            "Port number".to_string(),
            Some("8080".to_string()),
            Some("8000".to_string()),
            Some(false),
        );

        let without_default = TemplateVariable::new(
            "Service name".to_string(),
            Some("example-service".to_string()),
            None,
            Some(true),
        );

        assert!(with_default.has_default());
        assert!(!without_default.has_default());
    }

    #[test]
    fn template_variable_serialization() {
        let var = TemplateVariable::new(
            "Database URL".to_string(),
            Some("postgresql://localhost:5432/mydb".to_string()),
            Some("sqlite:///tmp/default.db".to_string()),
            Some(false),
        );

        let json_str = serde_json::to_string(&var).expect("Should serialize to JSON");
        let deserialized: TemplateVariable =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        assert_eq!(var, deserialized);
        assert_eq!(deserialized.description(), "Database URL");
        assert!(deserialized.has_default());
    }

    #[test]
    fn template_config_new_and_basic_accessors() {
        let metadata = TemplateMetadata::new(
            "api-template".to_string(),
            "REST API template".to_string(),
            "Backend Team".to_string(),
            vec!["api".to_string(), "rest".to_string()],
        );

        let config = TemplateConfig::new(metadata);

        assert_eq!(config.template().name(), "api-template");
        assert_eq!(config.template().author(), "Backend Team");
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
    fn template_config_repository_type_management() {
        let metadata = TemplateMetadata::new(
            "service-template".to_string(),
            "Microservice template".to_string(),
            "Platform Team".to_string(),
            vec!["microservice".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);

        // Initially no repository type
        assert!(!config.has_repository_type());
        assert!(config.can_override_repository_type()); // No restriction when not specified

        // Set fixed repository type
        let fixed_spec =
            RepositoryTypeSpec::new("microservice".to_string(), RepositoryTypePolicy::Fixed);
        config.set_repository_type(Some(fixed_spec));

        assert!(config.has_repository_type());
        assert!(!config.can_override_repository_type());
        assert_eq!(
            config.repository_type().unwrap().repository_type(),
            "microservice"
        );

        // Change to preferable repository type
        let preferable_spec =
            RepositoryTypeSpec::new("service".to_string(), RepositoryTypePolicy::Preferable);
        config.set_repository_type(Some(preferable_spec));

        assert!(config.has_repository_type());
        assert!(config.can_override_repository_type());
        assert_eq!(
            config.repository_type().unwrap().repository_type(),
            "service"
        );

        // Remove repository type
        config.set_repository_type(None);
        assert!(!config.has_repository_type());
        assert!(config.can_override_repository_type());
    }

    #[test]
    fn template_config_variable_management() {
        let metadata = TemplateMetadata::new(
            "app-template".to_string(),
            "Application template".to_string(),
            "Dev Team".to_string(),
            vec!["app".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);

        // Initially no variables
        assert!(config.variables().is_none());

        // Add individual variable
        let service_var = TemplateVariable::new(
            "Name of the service".to_string(),
            Some("my-service".to_string()),
            None,
            Some(true),
        );
        config.add_variable("service_name".to_string(), service_var);

        assert!(config.variables().is_some());
        assert!(config.variables().unwrap().contains_key("service_name"));
        assert_eq!(config.variables().unwrap().len(), 1);

        // Add another variable
        let port_var = TemplateVariable::new(
            "Port number".to_string(),
            Some("8080".to_string()),
            Some("8000".to_string()),
            Some(false),
        );
        config.add_variable("port".to_string(), port_var);

        assert_eq!(config.variables().unwrap().len(), 2);
        assert!(config.variables().unwrap().contains_key("port"));

        // Set entire variables HashMap
        let mut new_vars = HashMap::new();
        new_vars.insert(
            "database_url".to_string(),
            TemplateVariable::new(
                "Database connection URL".to_string(),
                None,
                Some("sqlite:///tmp/app.db".to_string()),
                Some(false),
            ),
        );
        config.set_variables(Some(new_vars));

        assert_eq!(config.variables().unwrap().len(), 1);
        assert!(config.variables().unwrap().contains_key("database_url"));
        assert!(!config.variables().unwrap().contains_key("service_name"));

        // Remove all variables
        config.set_variables(None);
        assert!(config.variables().is_none());
    }

    #[test]
    fn template_config_labels_management() {
        let metadata = TemplateMetadata::new(
            "lib-template".to_string(),
            "Library template".to_string(),
            "Platform Team".to_string(),
            vec!["library".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);

        // Initially no labels
        assert!(config.labels().is_none());

        // Set labels
        let labels = vec![
            LabelConfig::new(
                "bug".to_string(),
                Some("Bug reports".to_string()),
                "d73a4a".to_string(),
            ),
            LabelConfig::new(
                "enhancement".to_string(),
                Some("Feature requests".to_string()),
                "a2eeef".to_string(),
            ),
        ];
        config.set_labels(Some(labels));

        assert!(config.labels().is_some());
        assert_eq!(config.labels().unwrap().len(), 2);

        // Remove labels
        config.set_labels(None);
        assert!(config.labels().is_none());
    }

    #[test]
    fn template_config_count_additive_items() {
        let metadata = TemplateMetadata::new(
            "full-template".to_string(),
            "Template with all features".to_string(),
            "Full Team".to_string(),
            vec!["complete".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);

        // Initially no additive items
        assert_eq!(config.count_additive_items(), 0);

        // Add labels
        config.set_labels(Some(vec![
            LabelConfig::new("bug".to_string(), None, "d73a4a".to_string()),
            LabelConfig::new("feature".to_string(), None, "a2eeef".to_string()),
            LabelConfig::new("documentation".to_string(), None, "0075ca".to_string()),
        ]));
        assert_eq!(config.count_additive_items(), 3);

        // Add webhooks
        config.set_webhooks(Some(vec![
            WebhookConfig::new(
                "https://example.com/hook1".to_string(),
                vec![WebhookEvent::Push],
                true,
                None,
            ),
            WebhookConfig::new(
                "https://example.com/hook2".to_string(),
                vec![WebhookEvent::PullRequest],
                true,
                None,
            ),
        ]));
        assert_eq!(config.count_additive_items(), 5); // 3 labels + 2 webhooks

        // Add variables
        let mut variables = HashMap::new();
        variables.insert(
            "var1".to_string(),
            TemplateVariable::new("Variable 1".to_string(), None, None, None),
        );
        variables.insert(
            "var2".to_string(),
            TemplateVariable::new("Variable 2".to_string(), None, None, None),
        );
        config.set_variables(Some(variables));
        assert_eq!(config.count_additive_items(), 7); // 3 labels + 2 webhooks + 2 variables

        // Add environments
        config.set_environments(Some(vec![EnvironmentConfig::new(
            "staging".to_string(),
            None,
            None,
            None,
        )]));
        assert_eq!(config.count_additive_items(), 8); // 3 labels + 2 webhooks + 2 variables + 1 environment
    }

    #[test]
    fn template_config_serialization() {
        let metadata = TemplateMetadata::new(
            "serialization-test".to_string(),
            "Template for testing serialization".to_string(),
            "Test Team".to_string(),
            vec!["test".to_string(), "serialization".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);

        // Set repository type
        config.set_repository_type(Some(RepositoryTypeSpec::new(
            "library".to_string(),
            RepositoryTypePolicy::Preferable,
        )));

        // Add variables
        let mut variables = HashMap::new();
        variables.insert(
            "lib_name".to_string(),
            TemplateVariable::new(
                "Library name".to_string(),
                Some("my-lib".to_string()),
                None,
                Some(true),
            ),
        );
        config.set_variables(Some(variables));

        // Add labels
        config.set_labels(Some(vec![LabelConfig::new(
            "library".to_string(),
            Some("Library specific".to_string()),
            "0075ca".to_string(),
        )]));

        // Test serialization
        let json_str = serde_json::to_string(&config).expect("Should serialize to JSON");
        let deserialized: TemplateConfig =
            serde_json::from_str(&json_str).expect("Should deserialize from JSON");

        // Verify metadata
        assert_eq!(deserialized.template().name(), "serialization-test");
        assert_eq!(deserialized.template().author(), "Test Team");
        assert_eq!(deserialized.template().tags().len(), 2);

        // Verify repository type
        assert!(deserialized.repository_type().is_some());
        assert_eq!(
            deserialized.repository_type().unwrap().repository_type(),
            "library"
        );
        assert!(deserialized.can_override_repository_type());

        // Verify variables
        assert!(deserialized.variables().is_some());
        assert!(deserialized.variables().unwrap().contains_key("lib_name"));

        // Verify labels
        assert!(deserialized.labels().is_some());
        assert_eq!(deserialized.labels().unwrap().len(), 1);

        // Verify additive item count
        assert_eq!(deserialized.count_additive_items(), 2); // 1 variable + 1 label
    }

    #[test]
    fn template_config_is_cloneable() {
        let metadata = TemplateMetadata::new(
            "clone-test".to_string(),
            "Template for clone testing".to_string(),
            "Clone Team".to_string(),
            vec!["clone".to_string()],
        );

        let mut config = TemplateConfig::new(metadata);
        config.add_variable(
            "test_var".to_string(),
            TemplateVariable::new("Test variable".to_string(), None, None, None),
        );

        let cloned = config.clone();

        // Verify they are equal but independent
        assert_eq!(config.template().name(), cloned.template().name());
        assert_eq!(config.variables().is_some(), cloned.variables().is_some());
        assert_eq!(config.count_additive_items(), cloned.count_additive_items());
    }

    #[test]
    fn template_config_is_debuggable() {
        let metadata = TemplateMetadata::new(
            "debug-test".to_string(),
            "Template for debug testing".to_string(),
            "Debug Team".to_string(),
            vec!["debug".to_string()],
        );

        let config = TemplateConfig::new(metadata);
        let debug_string = format!("{:?}", config);

        assert!(debug_string.contains("TemplateConfig"));
        assert!(debug_string.contains("debug-test"));
    }
}
