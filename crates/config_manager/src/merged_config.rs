//! Merged configuration representing resolved settings.
//!
//! MergedConfiguration is the result of merging all configuration sources
//! (Global → Repository Type → Team → Template) into a single, resolved
//! configuration that will be applied to a repository.
//!
//! # Configuration Sources
//!
//! The merge follows a strict precedence hierarchy:
//! 1. **Template** (highest precedence)
//! 2. **Team**
//! 3. **Repository Type**
//! 4. **Global** (lowest precedence)
//!
//! # Source Tracing
//!
//! MergedConfiguration includes a `source_trace` field that tracks which
//! configuration source provided each setting, enabling:
//! - Audit logging
//! - Debugging configuration issues
//! - Understanding configuration precedence
//!
//! # Examples
//!
//! ```rust
//! use config_manager::{MergedConfiguration, ConfigurationSource};
//! use std::collections::HashMap;
//!
//! // MergedConfiguration is typically created by a ConfigurationMerger
//! let mut config = MergedConfiguration::new();
//!
//! // Source trace tracks where each setting came from
//! config.record_source("repository.issues", ConfigurationSource::Global);
//! config.record_source("pull_requests.required_approving_review_count", ConfigurationSource::Template);
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::{
    BranchProtectionSettings, CustomProperty, EnvironmentConfig, GitHubAppConfig, LabelConfig,
    NotificationsConfig, PullRequestSettings, RepositorySettings, RulesetConfig, WebhookConfig,
};
use std::collections::HashMap;

/// Merged configuration representing the final resolved settings.
///
/// This structure contains the result of merging configuration from all sources
/// (global, repository type, team, and template) according to the precedence
/// hierarchy and override policies.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct MergedConfiguration {
    /// Repository feature settings.
    ///
    /// Final resolved settings for repository features (issues, wiki, etc.)
    pub repository: RepositorySettings,

    /// Pull request configuration.
    ///
    /// Final resolved PR policies (required reviewers, merge types, etc.)
    pub pull_requests: PullRequestSettings,

    /// Branch protection settings.
    ///
    /// Final resolved branch protection rules.
    pub branch_protection: BranchProtectionSettings,

    /// Labels to be created in the repository.
    ///
    /// Merged from all sources, using label name as the key.
    /// Later sources override earlier sources for the same label name.
    pub labels: HashMap<String, LabelConfig>,

    /// Webhooks to be created in the repository.
    ///
    /// Merged from all sources (additive - all webhooks from all sources).
    pub webhooks: Vec<WebhookConfig>,

    /// Custom properties to set on the repository.
    ///
    /// Merged from all sources (additive).
    pub custom_properties: Vec<CustomProperty>,

    /// Environments to create in the repository.
    ///
    /// Merged from all sources (additive).
    pub environments: Vec<EnvironmentConfig>,

    /// GitHub Apps to install on the repository.
    ///
    /// Merged from all sources (additive).
    pub github_apps: Vec<GitHubAppConfig>,

    /// Repository rulesets to apply.
    ///
    /// Merged from all sources (additive - all rulesets from all sources).
    pub rulesets: Vec<RulesetConfig>,

    /// Outbound event notification endpoints.
    ///
    /// Merged from all sources (additive - all endpoints from all sources).
    /// These webhooks receive notifications when RepoRoller performs operations.
    pub notifications: NotificationsConfig,

    /// Source trace tracking which configuration source provided each setting.
    ///
    /// Used for auditing, debugging, and understanding configuration precedence.
    pub source_trace: ConfigurationSourceTrace,
}

impl MergedConfiguration {
    /// Creates a new empty MergedConfiguration.
    ///
    /// All settings start with default values. The ConfigurationMerger
    /// will populate this with merged settings from all sources.
    pub fn new() -> Self {
        Self {
            repository: RepositorySettings::default(),
            pull_requests: PullRequestSettings::default(),
            branch_protection: BranchProtectionSettings::default(),
            labels: HashMap::new(),
            webhooks: Vec::new(),
            custom_properties: Vec::new(),
            environments: Vec::new(),
            github_apps: Vec::new(),
            rulesets: Vec::new(),
            notifications: NotificationsConfig {
                outbound_webhooks: Vec::new(),
            },
            source_trace: ConfigurationSourceTrace::new(),
        }
    }

    /// Records the source of a configuration setting.
    ///
    /// This is used during the merge process to track which configuration
    /// source (Global, RepositoryType, Team, Template) provided each setting.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the setting (e.g., "repository.issues")
    /// * `source` - The configuration source that provided this setting
    pub fn record_source(&mut self, field_path: &str, source: ConfigurationSource) {
        self.source_trace.add_source(field_path, source);
    }

    /// Gets the source of a configuration setting.
    ///
    /// Returns the configuration source that provided the specified setting,
    /// or None if the setting hasn't been configured.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the setting
    pub fn get_source(&self, field_path: &str) -> Option<ConfigurationSource> {
        self.source_trace.get_source(field_path)
    }
}

impl Default for MergedConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks which configuration source provided each setting.
///
/// Used for auditing, debugging, and understanding configuration precedence.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct ConfigurationSourceTrace {
    /// Map of field path to configuration source.
    sources: HashMap<String, ConfigurationSource>,
}

impl ConfigurationSourceTrace {
    /// Creates a new empty source trace.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Records the source of a configuration setting.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the setting
    /// * `source` - The configuration source that provided this setting
    pub fn add_source(&mut self, field_path: &str, source: ConfigurationSource) {
        self.sources.insert(field_path.to_string(), source);
    }

    /// Gets the source of a configuration setting.
    ///
    /// Returns the configuration source that provided the specified setting,
    /// or None if the setting hasn't been configured.
    pub fn get_source(&self, field_path: &str) -> Option<ConfigurationSource> {
        self.sources.get(field_path).copied()
    }

    /// Returns all field paths that have been configured.
    pub fn configured_fields(&self) -> Vec<&str> {
        self.sources.keys().map(|s| s.as_str()).collect()
    }

    /// Returns the number of configured fields.
    pub fn field_count(&self) -> usize {
        self.sources.len()
    }
}

/// Configuration source in the hierarchy.
///
/// Represents which level of the configuration hierarchy provided a setting.
/// The precedence order is: Template > Team > RepositoryType > Global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ConfigurationSource {
    /// Global organization defaults (lowest precedence).
    Global,

    /// Repository type-specific configuration.
    RepositoryType,

    /// Team-specific configuration.
    Team,

    /// Template-specific configuration (highest precedence).
    Template,
}

impl std::fmt::Display for ConfigurationSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationSource::Global => write!(f, "Global"),
            ConfigurationSource::RepositoryType => write!(f, "RepositoryType"),
            ConfigurationSource::Team => write!(f, "Team"),
            ConfigurationSource::Template => write!(f, "Template"),
        }
    }
}

#[cfg(test)]
#[path = "merged_config_tests.rs"]
mod tests;
