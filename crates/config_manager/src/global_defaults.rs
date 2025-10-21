//! Global default configuration for an organization.
//!
//! This module defines the `GlobalDefaults` structure which represents
//! organization-wide baseline settings that apply to all repositories
//! unless overridden by team or template configuration.
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::*;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
}

#[cfg(test)]
#[path = "global_defaults_tests.rs"]
mod tests;
