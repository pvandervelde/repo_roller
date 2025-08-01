//! Tests for basic configuration types and enums.

use crate::types::*;

#[cfg(test)]
mod repository_visibility_tests {
    use super::*;

    #[test]
    fn repository_visibility_debug_format() {
        assert_eq!(format!("{:?}", RepositoryVisibility::Private), "Private");
        assert_eq!(format!("{:?}", RepositoryVisibility::Public), "Public");
        assert_eq!(format!("{:?}", RepositoryVisibility::Internal), "Internal");
    }

    #[test]
    fn repository_visibility_equality() {
        assert_eq!(RepositoryVisibility::Private, RepositoryVisibility::Private);
        assert_ne!(RepositoryVisibility::Private, RepositoryVisibility::Public);
    }
}

#[cfg(test)]
mod merge_type_tests {
    use super::*;

    #[test]
    fn merge_type_debug_format() {
        assert_eq!(format!("{:?}", MergeType::Merge), "Merge");
        assert_eq!(format!("{:?}", MergeType::Squash), "Squash");
        assert_eq!(format!("{:?}", MergeType::Rebase), "Rebase");
    }
}

#[cfg(test)]
mod webhook_event_tests {
    use super::*;

    #[test]
    fn webhook_event_debug_format() {
        assert_eq!(format!("{:?}", WebhookEvent::Push), "Push");
        assert_eq!(format!("{:?}", WebhookEvent::PullRequest), "PullRequest");
        assert_eq!(format!("{:?}", WebhookEvent::Issues), "Issues");
    }
}

#[cfg(test)]
mod label_config_tests {
    use super::*;

    #[test]
    fn label_config_new() {
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
    fn label_config_fields_accessible() {
        let label = LabelConfig {
            name: "enhancement".to_string(),
            description: Some("New feature or request".to_string()),
            color: "a2eeef".to_string(),
        };

        assert_eq!(label.name, "enhancement");
        assert!(label.description.is_some());
        assert_eq!(label.color, "a2eeef");
    }
}

#[cfg(test)]
mod webhook_config_tests {
    use super::*;

    #[test]
    fn webhook_config_new() {
        let webhook = WebhookConfig::new(
            "https://example.com/webhook".to_string(),
            vec![WebhookEvent::Push, WebhookEvent::PullRequest],
            true,
            Some("secret".to_string()),
        );

        assert_eq!(webhook.url, "https://example.com/webhook");
        assert_eq!(
            webhook.events,
            vec![WebhookEvent::Push, WebhookEvent::PullRequest]
        );
        assert!(webhook.active);
        assert_eq!(webhook.secret, Some("secret".to_string()));
    }

    #[test]
    fn webhook_config_without_secret() {
        let webhook = WebhookConfig::new(
            "https://example.com/webhook".to_string(),
            vec![WebhookEvent::Issues],
            false,
            None,
        );

        assert_eq!(webhook.url, "https://example.com/webhook");
        assert_eq!(webhook.events, vec![WebhookEvent::Issues]);
        assert!(!webhook.active);
        assert!(webhook.secret.is_none());
    }
}

#[cfg(test)]
mod merge_config_tests {
    use super::*;

    #[test]
    fn merge_config_new() {
        let config = MergeConfig::new(
            vec![MergeType::Squash],
            CommitMessageOption::PullRequestTitle,
            CommitMessageOption::PullRequestTitleAndDescription,
        );

        assert_eq!(config.allowed_types, vec![MergeType::Squash]);
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
    fn merge_config_multiple_types() {
        let config = MergeConfig {
            allowed_types: vec![MergeType::Merge, MergeType::Squash, MergeType::Rebase],
            merge_commit_message: CommitMessageOption::DefaultMessage,
            squash_commit_message: CommitMessageOption::PullRequestTitleAndCommitDetails,
        };

        assert_eq!(config.allowed_types.len(), 3);
        assert!(config.allowed_types.contains(&MergeType::Merge));
        assert!(config.allowed_types.contains(&MergeType::Squash));
        assert!(config.allowed_types.contains(&MergeType::Rebase));
    }
}

#[cfg(test)]
mod environment_config_tests {
    use super::*;

    #[test]
    fn environment_config_new() {
        let env = EnvironmentConfig::new(
            "production".to_string(),
            Some(vec!["@team-leads".to_string()]),
            Some(300),
            Some("main".to_string()),
        );

        assert_eq!(env.name, "production");
        assert_eq!(
            env.required_reviewers,
            Some(vec!["@team-leads".to_string()])
        );
        assert_eq!(env.wait_timer, Some(300));
        assert_eq!(env.deployment_branch_policy, Some("main".to_string()));
    }

    #[test]
    fn environment_config_minimal() {
        let env = EnvironmentConfig::new("staging".to_string(), None, None, None);

        assert_eq!(env.name, "staging");
        assert!(env.required_reviewers.is_none());
        assert!(env.wait_timer.is_none());
        assert!(env.deployment_branch_policy.is_none());
    }
}
