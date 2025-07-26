//! Tests for organization-specific configuration structures.
//!
//! This module contains comprehensive unit tests for the organization configuration
//! system, focusing on the `OverridableValue<T>` generic type and hierarchical
//! configuration structures.

use crate::organization::{
    CommitMessageOption, GlobalDefaults, LabelConfig, MergeConfig, MergeType, OverridableValue,
    RepositoryVisibility, WebhookConfig, WebhookEvent, WorkflowPermission,
};

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
        let env = EnvironmentConfig::new("production".to_string());
        assert_eq!(env.name, "production");
        assert!(env.protection_rules_enabled.is_none());
    }

    #[test]
    fn test_environment_config_serialization() {
        let mut env = EnvironmentConfig::new("staging".to_string());
        env.protection_rules_enabled = Some(OverridableValue::overridable(true));

        let json = serde_json::to_string(&env).unwrap();
        let deserialized: EnvironmentConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(env, deserialized);
        assert_eq!(deserialized.name, "staging");
        assert_eq!(
            deserialized
                .protection_rules_enabled
                .as_ref()
                .unwrap()
                .value,
            true
        );
        assert!(deserialized
            .protection_rules_enabled
            .as_ref()
            .unwrap()
            .can_override());
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
        let mut prod_env = EnvironmentConfig::new("production".to_string());
        prod_env.protection_rules_enabled = Some(OverridableValue::fixed(true));
        let mut staging_env = EnvironmentConfig::new("staging".to_string());
        staging_env.protection_rules_enabled = Some(OverridableValue::overridable(false));
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
        assert!(!environments[0]
            .protection_rules_enabled
            .as_ref()
            .unwrap()
            .can_override());

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
