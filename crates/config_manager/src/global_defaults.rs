//! Global default configuration for an organization.
//!
//! This module defines the `GlobalDefaults` structure which represents
//! organization-wide baseline settings that apply to all repositories
//! unless overridden by team or template configuration.
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::*;
use crate::visibility::VisibilityPolicyConfig;
use serde::{Deserialize, Serialize};

/// Organization-wide global default configuration.
///
/// Defines baseline settings that apply to all repositories in the organization.
/// Settings use `OverridableValue<T>` to control whether teams and templates
/// can override them.
///
/// # Structure
///
/// The global defaults are loaded from `global/defaults.toml` in the metadata repository.
///
/// # Examples
///
/// ```rust
/// use config_manager::{GlobalDefaults, OverridableValue, settings::RepositorySettings};
///
/// let defaults = GlobalDefaults {
///     repository: Some(RepositorySettings {
///         issues: Some(OverridableValue::allowed(true)),
///         wiki: Some(OverridableValue::fixed(false)), // Policy
///         ..Default::default()
///     }),
///     ..Default::default()
/// };
/// ```
///
/// # TOML Format
///
/// ```toml
/// [repository]
/// issues = { value = true, override_allowed = true }
/// wiki = { value = false, override_allowed = false }
///
/// [pull_requests]
/// allow_merge_commit = { value = false, override_allowed = true }
/// required_approving_review_count = { value = 1, override_allowed = true }
///
/// [[webhooks]]
/// url = "https://example.com/webhook"
/// content_type = "json"
/// active = true
/// events = ["push", "pull_request"]
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GlobalDefaults {
    /// Repository feature settings
    pub repository: Option<RepositorySettings>,

    /// Pull request settings
    pub pull_requests: Option<PullRequestSettings>,

    /// Branch protection settings
    pub branch_protection: Option<BranchProtectionSettings>,

    /// GitHub Actions settings
    pub actions: Option<ActionSettings>,

    /// Push restriction settings
    pub push: Option<PushSettings>,

    /// Organization-wide webhooks
    pub webhooks: Option<Vec<WebhookConfig>>,

    /// Custom properties to set
    pub custom_properties: Option<Vec<CustomProperty>>,

    /// Environment configurations
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// Required GitHub Apps
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Repository visibility policy configuration
    ///
    /// Controls organization-wide visibility policies. This defines whether
    /// repositories must use specific visibility, are restricted from certain
    /// visibilities, or have unrestricted choice.
    ///
    /// # Examples
    ///
    /// Require all repositories to be private:
    /// ```toml
    /// [repository_visibility]
    /// enforcement_level = "required"
    /// required_visibility = "private"
    /// ```
    ///
    /// Prohibit public repositories:
    /// ```toml
    /// [repository_visibility]
    /// enforcement_level = "restricted"
    /// restricted_visibilities = ["public"]
    /// ```
    pub repository_visibility: Option<VisibilityPolicyConfig>,

    /// Repository rulesets (additive).
    ///
    /// Rulesets defined here are added to rulesets from other levels.
    /// This allows global governance rules to be defined and enforced
    /// across all repositories.
    ///
    /// # Examples
    ///
    /// ```toml
    /// [[rulesets]]
    /// name = "main-protection"
    /// target = "branch"
    /// enforcement = "active"
    ///
    /// [rulesets.conditions.ref_name]
    /// include = ["refs/heads/main"]
    ///
    /// [[rulesets.rules]]
    /// type = "deletion"
    /// ```
    pub rulesets: Option<Vec<RulesetConfig>>,
}

#[cfg(test)]
#[path = "global_defaults_tests.rs"]
mod tests;
