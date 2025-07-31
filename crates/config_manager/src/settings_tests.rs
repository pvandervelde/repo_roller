//! Tests for configuration settings structures.

use crate::hierarchy::*;
use crate::settings::*;
use crate::types::*;

#[cfg(test)]
mod global_defaults_tests {
    use super::*;

    #[test]
    fn global_defaults_new() {
        let defaults = GlobalDefaults::new();
        assert!(defaults.branch_protection_enabled.is_none());
        assert!(defaults.repository_visibility.is_none());
        assert!(defaults.merge_configuration.is_none());
        assert!(defaults.default_labels.is_none());
        assert!(defaults.organization_webhooks.is_none());
        assert!(defaults.required_github_apps.is_none());
    }

    #[test]
    fn global_defaults_default() {
        let defaults = GlobalDefaults::default();
        assert!(defaults.branch_protection_enabled.is_none());
        assert!(defaults.repository_visibility.is_none());
    }

    #[test]
    fn global_defaults_with_settings() {
        let mut defaults = GlobalDefaults::new();
        defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));
        defaults.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        assert!(defaults.branch_protection_enabled.is_some());
        assert_eq!(
            defaults.branch_protection_enabled.as_ref().unwrap().value(),
            true
        );
        assert!(!defaults
            .branch_protection_enabled
            .as_ref()
            .unwrap()
            .can_override());

        assert!(defaults.repository_visibility.is_some());
        assert_eq!(
            defaults.repository_visibility.as_ref().unwrap().value(),
            RepositoryVisibility::Private
        );
        assert!(defaults
            .repository_visibility
            .as_ref()
            .unwrap()
            .can_override());
    }
}

#[cfg(test)]
mod team_config_tests {
    use super::*;

    #[test]
    fn team_config_new() {
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
    fn team_config_default() {
        let team_config = TeamConfig::default();
        assert!(team_config.repository_visibility.is_none());
        assert!(team_config.team_webhooks.is_none());
    }

    #[test]
    fn team_config_has_overrides() {
        let mut team = TeamConfig::new();
        assert!(!team.has_overrides());

        team.repository_visibility = Some(RepositoryVisibility::Public);
        assert!(team.has_overrides());

        team.branch_protection_enabled = Some(false);
        assert!(team.has_overrides());
    }

    #[test]
    fn team_config_has_additions() {
        let mut team = TeamConfig::new();
        assert!(!team.has_additions());

        let webhook = WebhookConfig::new(
            "https://team.example.com/webhook".to_string(),
            vec![WebhookEvent::Push],
            true,
            None,
        );
        team.team_webhooks = Some(vec![webhook]);
        assert!(team.has_additions());

        team.team_github_apps = Some(vec!["team-app".to_string()]);
        assert!(team.has_additions());
    }

    #[test]
    fn team_config_validate_overrides_success() {
        let mut global = GlobalDefaults::new();
        global.repository_visibility =
            Some(OverridableValue::overridable(RepositoryVisibility::Private));

        let mut team = TeamConfig::new();
        team.repository_visibility = Some(RepositoryVisibility::Public);

        // This should succeed because global allows override
        assert!(team.validate_overrides(&global).is_ok());
    }

    #[test]
    fn team_config_validate_overrides_failure() {
        let mut global = GlobalDefaults::new();
        global.repository_visibility = Some(OverridableValue::fixed(RepositoryVisibility::Private));

        let mut team = TeamConfig::new();
        team.repository_visibility = Some(RepositoryVisibility::Public);

        // This should fail because global doesn't allow override
        let result = team.validate_overrides(&global);
        assert!(result.is_err());

        if let Err(crate::errors::ConfigurationError::OverrideNotAllowed { field, .. }) = result {
            assert_eq!(field, "repository_visibility");
        } else {
            panic!("Expected OverrideNotAllowed error");
        }
    }
}

#[cfg(test)]
mod action_settings_tests {
    use super::*;

    #[test]
    fn action_settings_new() {
        let settings = ActionSettings::new();
        assert!(settings.enabled.is_none());
        assert!(settings.default_workflow_permissions.is_none());
    }

    #[test]
    fn action_settings_default() {
        let settings = ActionSettings::default();
        assert!(settings.enabled.is_none());
    }
}

#[cfg(test)]
mod branch_protection_settings_tests {
    use super::*;

    #[test]
    fn branch_protection_settings_new() {
        let settings = BranchProtectionSettings::new();
        assert!(settings.enabled.is_none());
        assert!(settings.require_pull_request_reviews.is_none());
        assert!(settings.required_reviewers.is_none());
    }

    #[test]
    fn branch_protection_settings_default() {
        let settings = BranchProtectionSettings::default();
        assert!(settings.enabled.is_none());
    }
}

#[cfg(test)]
mod pull_request_settings_tests {
    use super::*;

    #[test]
    fn pull_request_settings_new() {
        let settings = PullRequestSettings::new();
        assert!(settings.delete_branch_on_merge.is_none());
        assert!(settings.allow_squash_merge.is_none());
        assert!(settings.allow_merge_commit.is_none());
    }

    #[test]
    fn pull_request_settings_default() {
        let settings = PullRequestSettings::default();
        assert!(settings.delete_branch_on_merge.is_none());
    }
}

#[cfg(test)]
mod push_settings_tests {
    use super::*;

    #[test]
    fn push_settings_new() {
        let settings = PushSettings::new();
        assert!(settings.allow_force_pushes.is_none());
        assert!(settings.require_signed_commits.is_none());
    }

    #[test]
    fn push_settings_default() {
        let settings = PushSettings::default();
        assert!(settings.allow_force_pushes.is_none());
    }
}

#[cfg(test)]
mod repository_settings_tests {
    use super::*;

    #[test]
    fn repository_settings_new() {
        let settings = RepositorySettings::new();
        assert!(settings.has_issues.is_none());
        assert!(settings.has_wiki.is_none());
        assert!(settings.has_projects.is_none());
    }

    #[test]
    fn repository_settings_default() {
        let settings = RepositorySettings::default();
        assert!(settings.has_issues.is_none());
    }
}
