//! Merged configuration and hierarchical merging logic.
//!
//! This module provides the final resolved configuration structure and the logic
//! for merging configurations from different hierarchy levels.

use crate::settings::{BranchProtectionSettings, PullRequestSettings, RepositorySettings};
use crate::types::{EnvironmentConfig, GitHubAppConfig, LabelConfig, WebhookConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
#[path = "merged_tests.rs"]
mod merged_tests;

/// Source of a configuration setting in the hierarchy.
///
/// This enum identifies which level of the configuration hierarchy provided
/// a particular setting. It's used for audit trails and debugging configuration
/// resolution issues.
///
/// # Hierarchy Order (lowest to highest precedence)
///
/// 1. `Global` - Organization-wide defaults
/// 2. `RepositoryType` - Repository type-specific settings
/// 3. `Team` - Team-specific overrides and additions
/// 4. `Template` - Template-specific requirements (highest precedence)
///
/// # Examples
///
/// ```rust
/// use config_manager::merged::ConfigurationSource;
///
/// let template_source = ConfigurationSource::Template;
/// let global_source = ConfigurationSource::Global;
///
/// assert!(template_source.overrides(&global_source));
/// assert!(!global_source.overrides(&template_source));
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigurationSource {
    /// Setting comes from global organization defaults.
    Global,
    /// Setting comes from repository type configuration.
    RepositoryType,
    /// Setting comes from team configuration.
    Team,
    /// Setting comes from template configuration.
    Template,
}

impl ConfigurationSource {
    /// Returns the precedence level of this source.
    ///
    /// Higher numbers indicate higher precedence in the configuration hierarchy.
    ///
    /// # Returns
    ///
    /// The precedence level as a u8 (1-4).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::ConfigurationSource;
    ///
    /// assert_eq!(ConfigurationSource::Global.precedence(), 1);
    /// assert_eq!(ConfigurationSource::Template.precedence(), 4);
    /// ```
    pub fn precedence(&self) -> u8 {
        match self {
            ConfigurationSource::Global => 1,
            ConfigurationSource::RepositoryType => 2,
            ConfigurationSource::Team => 3,
            ConfigurationSource::Template => 4,
        }
    }

    /// Checks if this source has higher precedence than another source.
    ///
    /// # Arguments
    ///
    /// * `other` - The other configuration source to compare against
    ///
    /// # Returns
    ///
    /// `true` if this source should override the other source, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::ConfigurationSource;
    ///
    /// let template = ConfigurationSource::Template;
    /// let team = ConfigurationSource::Team;
    /// let global = ConfigurationSource::Global;
    ///
    /// assert!(template.overrides(&team));
    /// assert!(team.overrides(&global));
    /// assert!(!global.overrides(&template));
    /// ```
    pub fn overrides(&self, other: &ConfigurationSource) -> bool {
        self.precedence() > other.precedence()
    }
}

/// Tracks the source of each configuration setting during hierarchical merging.
///
/// This structure maintains an audit trail of where each configuration setting
/// originated from in the four-level hierarchy. This is crucial for debugging
/// configuration resolution, understanding why certain settings were applied,
/// and providing transparency in the configuration process.
///
/// # Examples
///
/// ```rust
/// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
///
/// let mut trace = ConfigurationSourceTrace::new();
/// trace.add_source("repository.issues".to_string(), ConfigurationSource::Template);
/// trace.add_source("labels.bug".to_string(), ConfigurationSource::Team);
///
/// assert_eq!(trace.get_source("repository.issues"), Some(&ConfigurationSource::Template));
/// assert!(trace.has_source("labels.bug"));
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationSourceTrace {
    /// Maps configuration field paths to their originating sources.
    /// The key is a dot-separated path like "repository.issues" or "labels.bug".
    sources: HashMap<String, ConfigurationSource>,
}

impl ConfigurationSourceTrace {
    /// Creates a new empty configuration source trace.
    ///
    /// # Returns
    ///
    /// A new `ConfigurationSourceTrace` with no recorded sources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::ConfigurationSourceTrace;
    ///
    /// let trace = ConfigurationSourceTrace::new();
    /// assert!(trace.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Adds a source for a configuration field.
    ///
    /// If a source already exists for the field, it will be replaced.
    /// This typically happens when a higher-precedence configuration
    /// overrides a lower-precedence one.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the configuration field
    /// * `source` - The source that provided this configuration value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// trace.add_source("pull_requests.required_reviewers".to_string(), ConfigurationSource::Global);
    /// trace.add_source("pull_requests.required_reviewers".to_string(), ConfigurationSource::Template);
    ///
    /// // Template overrides global
    /// assert_eq!(trace.get_source("pull_requests.required_reviewers"), Some(&ConfigurationSource::Template));
    /// ```
    pub fn add_source(&mut self, field_path: String, source: ConfigurationSource) {
        self.sources.insert(field_path, source);
    }

    /// Gets the source for a configuration field.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the configuration field
    ///
    /// # Returns
    ///
    /// An optional reference to the `ConfigurationSource` if the field is tracked.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// trace.add_source("webhooks.ci".to_string(), ConfigurationSource::Team);
    ///
    /// assert_eq!(trace.get_source("webhooks.ci"), Some(&ConfigurationSource::Team));
    /// assert_eq!(trace.get_source("webhooks.deploy"), None);
    /// ```
    pub fn get_source(&self, field_path: &str) -> Option<&ConfigurationSource> {
        self.sources.get(field_path)
    }

    /// Checks if a source is tracked for the given field.
    ///
    /// # Arguments
    ///
    /// * `field_path` - Dot-separated path to the configuration field
    ///
    /// # Returns
    ///
    /// `true` if the field has a tracked source, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// trace.add_source("environments.prod".to_string(), ConfigurationSource::RepositoryType);
    ///
    /// assert!(trace.has_source("environments.prod"));
    /// assert!(!trace.has_source("environments.staging"));
    /// ```
    pub fn has_source(&self, field_path: &str) -> bool {
        self.sources.contains_key(field_path)
    }

    /// Checks if the trace is empty (no sources recorded).
    ///
    /// # Returns
    ///
    /// `true` if no sources are recorded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// assert!(trace.is_empty());
    ///
    /// trace.add_source("test".to_string(), ConfigurationSource::Global);
    /// assert!(!trace.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Gets the count of tracked configuration sources.
    ///
    /// # Returns
    ///
    /// The number of configuration fields with tracked sources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// assert_eq!(trace.count(), 0);
    ///
    /// trace.add_source("setting1".to_string(), ConfigurationSource::Global);
    /// trace.add_source("setting2".to_string(), ConfigurationSource::Template);
    /// assert_eq!(trace.count(), 2);
    /// ```
    pub fn count(&self) -> usize {
        self.sources.len()
    }

    /// Gets all tracked field paths and their sources.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing all field path to source mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace = ConfigurationSourceTrace::new();
    /// trace.add_source("labels.bug".to_string(), ConfigurationSource::Global);
    /// trace.add_source("labels.feature".to_string(), ConfigurationSource::Team);
    ///
    /// let all_sources = trace.all_sources();
    /// assert_eq!(all_sources.len(), 2);
    /// ```
    pub fn all_sources(&self) -> &HashMap<String, ConfigurationSource> {
        &self.sources
    }

    /// Merges another source trace into this one.
    ///
    /// Sources from the other trace will be added to this trace. If both traces
    /// have sources for the same field path, the source with higher precedence
    /// will be retained.
    ///
    /// # Arguments
    ///
    /// * `other` - The source trace to merge into this one
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::{ConfigurationSourceTrace, ConfigurationSource};
    ///
    /// let mut trace1 = ConfigurationSourceTrace::new();
    /// trace1.add_source("setting1".to_string(), ConfigurationSource::Global);
    /// trace1.add_source("setting2".to_string(), ConfigurationSource::Team);
    ///
    /// let mut trace2 = ConfigurationSourceTrace::new();
    /// trace2.add_source("setting2".to_string(), ConfigurationSource::Template);
    /// trace2.add_source("setting3".to_string(), ConfigurationSource::RepositoryType);
    ///
    /// trace1.merge(trace2);
    ///
    /// // Template overrides Team for setting2
    /// assert_eq!(trace1.get_source("setting2"), Some(&ConfigurationSource::Template));
    /// assert_eq!(trace1.get_source("setting3"), Some(&ConfigurationSource::RepositoryType));
    /// assert_eq!(trace1.count(), 3);
    /// ```
    pub fn merge(&mut self, other: ConfigurationSourceTrace) {
        for (field_path, source) in other.sources {
            if let Some(existing_source) = self.sources.get(&field_path) {
                // Keep the source with higher precedence
                if source.overrides(existing_source) {
                    self.sources.insert(field_path, source);
                }
            } else {
                self.sources.insert(field_path, source);
            }
        }
    }
}

impl Default for ConfigurationSourceTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Final resolved configuration after hierarchical merging.
///
/// This structure represents the complete, resolved configuration that results
/// from merging the four-level hierarchy (Template > Team > Repository Type > Global).
/// It contains all the settings needed to create and configure a repository,
/// along with audit trail information about where each setting originated.
///
/// The merged configuration is the authoritative source for repository settings
/// and is used by the repository creation workflow to apply all necessary
/// configurations to the new repository.
///
/// # Examples
///
/// ```rust
/// use config_manager::merged::MergedConfiguration;
///
/// let config = MergedConfiguration::new();
/// assert!(config.labels().is_empty());
/// assert!(config.webhooks().is_empty());
/// assert!(config.source_trace().is_empty());
/// ```
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MergedConfiguration {
    /// Repository settings (issues, wiki, security, etc.).
    repository_settings: RepositorySettings,
    /// Pull request settings and policies.
    pull_request_settings: PullRequestSettings,
    /// Branch protection rules and settings.
    branch_protection: BranchProtectionSettings,
    /// Repository labels to create.
    labels: Vec<LabelConfig>,
    /// Webhook configurations to set up.
    webhooks: Vec<WebhookConfig>,
    /// GitHub Apps to install.
    github_apps: Vec<GitHubAppConfig>,
    /// Environment configurations.
    environments: Vec<EnvironmentConfig>,
    /// Audit trail of configuration sources.
    source_trace: ConfigurationSourceTrace,
}

impl MergedConfiguration {
    /// Creates a new empty merged configuration.
    ///
    /// All configuration sections are initialized to their default empty states.
    /// Use the various merging methods to populate the configuration from
    /// different hierarchy levels.
    ///
    /// # Returns
    ///
    /// A new `MergedConfiguration` with empty settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::merged::MergedConfiguration;
    ///
    /// let config = MergedConfiguration::new();
    /// assert!(config.labels().is_empty());
    /// assert!(config.webhooks().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            repository_settings: RepositorySettings::new(),
            pull_request_settings: PullRequestSettings::new(),
            branch_protection: BranchProtectionSettings::new(),
            labels: Vec::new(),
            webhooks: Vec::new(),
            github_apps: Vec::new(),
            environments: Vec::new(),
            source_trace: ConfigurationSourceTrace::new(),
        }
    }

    /// Gets the repository settings.
    ///
    /// # Returns
    ///
    /// A reference to the `RepositorySettings`.
    pub fn repository_settings(&self) -> &RepositorySettings {
        &self.repository_settings
    }

    /// Gets the pull request settings.
    ///
    /// # Returns
    ///
    /// A reference to the `PullRequestSettings`.
    pub fn pull_request_settings(&self) -> &PullRequestSettings {
        &self.pull_request_settings
    }

    /// Gets the branch protection settings.
    ///
    /// # Returns
    ///
    /// A reference to the `BranchProtectionSettings`.
    pub fn branch_protection(&self) -> &BranchProtectionSettings {
        &self.branch_protection
    }

    /// Gets the labels.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `LabelConfig`.
    pub fn labels(&self) -> &[LabelConfig] {
        &self.labels
    }

    /// Gets the webhooks.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `WebhookConfig`.
    pub fn webhooks(&self) -> &[WebhookConfig] {
        &self.webhooks
    }

    /// Gets the GitHub Apps.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `GitHubAppConfig`.
    pub fn github_apps(&self) -> &[GitHubAppConfig] {
        &self.github_apps
    }

    /// Gets the environments.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `EnvironmentConfig`.
    pub fn environments(&self) -> &[EnvironmentConfig] {
        &self.environments
    }

    /// Gets the configuration source trace.
    ///
    /// # Returns
    ///
    /// A reference to the `ConfigurationSourceTrace`.
    pub fn source_trace(&self) -> &ConfigurationSourceTrace {
        &self.source_trace
    }

    /// Merges global default settings into this configuration.
    ///
    /// This should be called first in the merging process as global defaults
    /// provide the baseline configuration.
    ///
    /// # Arguments
    ///
    /// * `global_defaults` - The global default settings to merge
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual merging logic will be
    /// implemented based on the specific requirements of each configuration field.
    pub fn merge_global_defaults(&mut self, _global_defaults: &crate::settings::GlobalDefaults) {
        // TODO: Implement actual global defaults merging logic
        // This is a placeholder implementation

        // Example of what the implementation might look like:
        // if let Some(repo_visibility) = &global_defaults.repository_visibility {
        //     self.repository_settings.visibility = Some(repo_visibility.value());
        //     self.source_trace.add_source("repository.visibility".to_string(), ConfigurationSource::Global);
        // }
    }

    /// Merges repository type settings into this configuration.
    ///
    /// Repository type settings have higher precedence than global defaults
    /// but lower than team and template settings.
    ///
    /// # Arguments
    ///
    /// * `repo_type_config` - The repository type configuration to merge
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual merging logic will be
    /// implemented based on the specific requirements of each configuration field.
    pub fn merge_repository_type(
        &mut self,
        _repo_type_config: &crate::settings::RepositorySettings,
    ) {
        // TODO: Implement actual repository type merging logic
        // This is a placeholder implementation
    }

    /// Merges team configuration into this merged configuration.
    ///
    /// Team settings have higher precedence than global defaults and repository
    /// type settings, but lower than template settings.
    ///
    /// # Arguments
    ///
    /// * `team_config` - The team configuration to merge
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual merging logic will be
    /// implemented based on the specific requirements of each configuration field.
    pub fn merge_team_config(&mut self, _team_config: &crate::settings::TeamConfig) {
        // TODO: Implement actual team configuration merging logic
        // This is a placeholder implementation
    }

    /// Merges template configuration into this merged configuration.
    ///
    /// Template settings have the highest precedence and will override
    /// settings from all other levels in the hierarchy.
    ///
    /// # Arguments
    ///
    /// * `template_config` - The template configuration to merge
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual merging logic will be
    /// implemented based on the specific requirements of each configuration field.
    pub fn merge_template_config(&mut self, _template_config: &crate::templates::TemplateConfig) {
        // TODO: Implement actual template configuration merging logic
        // This is a placeholder implementation
    }

    /// Validates the final merged configuration.
    ///
    /// Performs validation checks on the merged configuration to ensure
    /// all required settings are present and all values are valid.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid, or a `ConfigurationError` if validation fails.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. The actual validation logic will be
    /// implemented based on the specific requirements of each configuration field.
    pub fn validate(&self) -> Result<(), crate::errors::ConfigurationError> {
        // TODO: Implement actual validation logic
        // This is a placeholder implementation
        Ok(())
    }

    /// Gets a summary of the configuration sources.
    ///
    /// Returns a breakdown of how many settings came from each source level.
    ///
    /// # Returns
    ///
    /// A HashMap mapping `ConfigurationSource` to count of settings from that source.
    pub fn source_summary(&self) -> HashMap<ConfigurationSource, usize> {
        let mut summary = HashMap::new();

        for source in self.source_trace.all_sources().values() {
            *summary.entry(source.clone()).or_insert(0) += 1;
        }

        summary
    }

    /// Checks if the configuration has any settings from the specified source.
    ///
    /// # Arguments
    ///
    /// * `source` - The configuration source to check for
    ///
    /// # Returns
    ///
    /// `true` if any settings came from the specified source, `false` otherwise.
    pub fn has_settings_from_source(&self, source: &ConfigurationSource) -> bool {
        self.source_trace
            .all_sources()
            .values()
            .any(|s| s == source)
    }
}

impl Default for MergedConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
