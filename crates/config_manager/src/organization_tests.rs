//! Tests for organization-specific configuration structures.
//!
//! This module contains comprehensive unit tests for the organization configuration
//! system, focusing on the `OverridableValue<T>` generic type and hierarchical
//! configuration structures.

use crate::organization::{
    CommitMessageOption, GlobalDefaults, LabelConfig, MergeConfig, MergeType, OverridableValue,
    RepositoryVisibility, WebhookConfig, WebhookEvent,
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
