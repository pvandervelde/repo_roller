//! Repository type-specific configuration.
//!
//! RepositoryTypeConfig allows defining configuration profiles for different
//! types of repositories (e.g., "library", "service", "documentation").
//!
//! # Configuration Hierarchy
//!
//! In the four-level hierarchy:
//! - Template (highest precedence)
//! - Team
//! - **Repository Type** ← This level
//! - Global (lowest precedence)
//!
//! Repository type configurations:
//! - Override global defaults (when `override_allowed = true`)
//! - Are themselves overridden by team and template configurations
//! - Use simple TOML format (values auto-wrap with `override_allowed = true`)
//! - Support additive merging for collections (labels, webhooks, apps, environments)
//! - Stored in `repository-types/<type-name>/config.toml`
//!
//! # TOML Format
//!
//! Repository type configurations use simple value format:
//!
//! ```toml
//! # repository-types/library/config.toml
//!
//! [repository]
//! has_wiki = false  # Libraries typically don't need wikis
//! has_projects = false
//! allow_squash_merge = true
//!
//! [pull_requests]
//! required_approving_review_count = 2
//! require_code_owner_reviews = true
//!
//! [branch_protection]
//! require_pull_request_reviews = true
//! required_approving_review_count = 2
//!
//! [[labels]]
//! name = "breaking-change"
//! color = "FF0000"
//! description = "Breaking API change requiring major version bump"
//!
//! [[webhooks]]
//! url = "https://ci.example.com/library-webhook"
//! content_type = "json"
//! events = ["push", "release"]
//! active = true
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::{
    BranchProtectionSettings, CustomProperty, EnvironmentConfig, GitHubAppConfig, LabelConfig,
    NotificationsConfig, PullRequestSettings, RepositorySettings, RulesetConfig, WebhookConfig,
};
use serde::{Deserialize, Serialize};

/// Repository type-specific configuration profile.
///
/// Defines configuration settings for a specific type of repository
/// (e.g., "library", "service", "documentation"). Repository types allow
/// organizations to standardize settings across similar repositories.
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
/// use config_manager::RepositoryTypeConfig;
///
/// let toml = r#"
///     [repository]
///     has_wiki = false
///     allow_squash_merge = true
///
///     [pull_requests]
///     required_approving_review_count = 2
/// "#;
///
/// let config: RepositoryTypeConfig = toml::from_str(toml).expect("Failed to parse");
/// assert!(config.repository.is_some());
/// assert!(config.pull_requests.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RepositoryTypeConfig {
    /// Repository feature settings for this type.
    ///
    /// Controls which features are enabled for repositories of this type
    /// (issues, wiki, discussions, merge strategies, etc.).
    pub repository: Option<RepositorySettings>,

    /// Pull request configuration for this type.
    ///
    /// Defines PR policies like required reviewers, merge types, and
    /// review requirements appropriate for this repository type.
    pub pull_requests: Option<PullRequestSettings>,

    /// Branch protection settings for this type.
    ///
    /// Configures branch protection rules appropriate for this repository type.
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Type-specific labels (additive).
    ///
    /// Labels defined here are added to global labels. This allows repository
    /// types to define standard issue/PR labels (e.g., "breaking-change" for
    /// libraries, "deployment" for services).
    pub labels: Option<Vec<LabelConfig>>,

    /// Type-specific webhooks (additive).
    ///
    /// Webhooks defined here are added to global webhooks. This allows repository
    /// types to integrate with type-specific tooling (e.g., library CI/CD pipelines).
    pub webhooks: Option<Vec<WebhookConfig>>,

    /// Type-specific custom properties (additive).
    ///
    /// Custom properties defined here are added to global properties.
    pub custom_properties: Option<Vec<CustomProperty>>,

    /// Type-specific environments (additive).
    ///
    /// Environments defined here are added to global environments.
    /// Allows defining deployment targets appropriate for this type.
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// Type-specific GitHub Apps (additive).
    ///
    /// Apps defined here are added to global required apps.
    /// Enables type-specific integrations (e.g., documentation generators
    /// for documentation repositories).
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Type-specific rulesets (additive).
    ///
    /// Rulesets defined here are added to global rulesets.
    /// Allows repository types to define governance rules appropriate
    /// for the type (e.g., stricter rules for library repositories).
    pub rulesets: Option<Vec<RulesetConfig>>,

    /// Outbound event notification configuration (additive).
    ///
    /// Type-specific webhook endpoints for event notifications.
    /// Combined with global, team, and template notifications.
    pub notifications: Option<NotificationsConfig>,
}

#[cfg(test)]
#[path = "repository_type_config_tests.rs"]
mod tests;
