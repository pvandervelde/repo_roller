//! Team-specific configuration overrides.
//!
//! TeamConfig allows teams within an organization to customize repository settings
//! for their specific needs, overriding global defaults where permitted.
//!
//! # Configuration Hierarchy
//!
//! In the four-level hierarchy:
//! - Template (highest precedence)
//! - **Team** ← This level
//! - Repository Type
//! - Global (lowest precedence)
//!
//! Team configurations:
//! - Override global defaults (when `override_allowed = true`)
//! - Are themselves overridden by templates
//! - Use simple TOML format (values auto-wrap with `override_allowed = true`)
//! - Support additive merging for collections (webhooks, apps, environments)
//!
//! # TOML Format
//!
//! Team configurations use simple value format:
//!
//! ```toml
//! # teams/backend-team/config.toml
//!
//! [repository]
//! discussions = false  # Override global default
//! projects = true
//!
//! [pull_requests]
//! required_approving_review_count = 2
//! require_code_owner_reviews = true
//!
//! [[webhooks]]
//! url = "https://backend-team.example.com/webhook"
//! content_type = "json"
//! events = ["push", "pull_request"]
//!
//! [[github_apps]]
//! app_id = 11111
//! permissions = { contents = "read", issues = "write" }
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::{
    ActionSettings, BranchProtectionSettings, CustomProperty, EnvironmentConfig, GitHubAppConfig,
    NotificationsConfig, PullRequestSettings, PushSettings, RepositorySettings, RulesetConfig,
    WebhookConfig,
};
use serde::{Deserialize, Serialize};

/// Team-specific configuration that overrides global defaults.
///
/// Teams can customize settings for repositories they create, subject to
/// override policies defined in GlobalDefaults. All fields are optional,
/// allowing teams to override only what they need.
///
/// # Field Deserialization
///
/// Fields use `OverridableValue<T>` types but deserialize from simple TOML format.
/// The flexible deserialization automatically wraps simple values with
/// `override_allowed = true`.
///
/// # Examples
///
/// ```rust
/// use config_manager::TeamConfig;
///
/// let toml = r#"
///     [repository]
///     discussions = false
///     projects = true
///
///     [pull_requests]
///     required_approving_review_count = 2
/// "#;
///
/// let config: TeamConfig = toml::from_str(toml).expect("Failed to parse");
/// assert!(config.repository.is_some());
/// assert!(config.pull_requests.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TeamConfig {
    /// Repository feature settings overrides.
    ///
    /// Teams can enable or disable repository features like issues, wiki,
    /// discussions, etc., subject to global override policies.
    pub repository: Option<RepositorySettings>,

    /// Pull request configuration overrides.
    ///
    /// Teams can customize PR policies like required reviewers, merge types,
    /// and review requirements.
    pub pull_requests: Option<PullRequestSettings>,

    /// Branch protection settings overrides.
    ///
    /// Teams can configure branch protection rules for their repositories.
    pub branch_protection: Option<BranchProtectionSettings>,

    /// GitHub Actions configuration overrides.
    ///
    /// Teams can control Actions enablement and permissions.
    pub actions: Option<ActionSettings>,

    /// Push restrictions overrides.
    ///
    /// Teams can set limits on branch/tag pushes.
    pub push: Option<PushSettings>,

    /// Team-specific webhooks (additive).
    ///
    /// Webhooks defined here are added to global webhooks, not replacing them.
    /// This allows teams to add their own notification endpoints.
    pub webhooks: Option<Vec<WebhookConfig>>,

    /// Team-specific custom properties (additive).
    ///
    /// Custom properties defined here are added to global properties.
    pub custom_properties: Option<Vec<CustomProperty>>,

    /// Team-specific environments (additive).
    ///
    /// Environments defined here are added to global environments.
    /// Teams can define deployment targets specific to their workflows.
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// Team-specific GitHub Apps (additive).
    ///
    /// Apps defined here are added to global required apps.
    /// Teams can enable additional apps for their repositories.
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Team-specific rulesets (additive).
    ///
    /// Rulesets defined here are added to global rulesets.
    /// Teams can define additional governance rules for their repositories.
    pub rulesets: Option<Vec<RulesetConfig>>,

    /// Outbound event notification configuration (additive).
    ///
    /// Team-level webhook endpoints for event notifications.
    /// Combined with global, repository type, and template notifications.
    pub notifications: Option<NotificationsConfig>,
}

#[cfg(test)]
#[path = "team_config_tests.rs"]
mod tests;
