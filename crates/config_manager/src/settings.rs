//! Configuration settings structures.
//!
//! This module provides various settings structures for different aspects of repository
//! configuration, including GitHub Actions, branch protection, pull requests, and more.

use crate::errors::ConfigurationError;
use crate::hierarchy::OverridableValue;
use crate::types::{
    EnvironmentConfig, LabelConfig, MergeConfig, RepositoryVisibility, WebhookConfig,
    WorkflowPermission,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;

/// GitHub Actions configuration settings.
///
/// Controls workflow execution permissions, artifact retention, and security policies
/// for GitHub Actions in repositories. These settings affect how workflows can run
/// and interact with repository resources.
///
/// # Security Implications
///
/// Actions settings control workflow permissions and should be carefully configured
/// to prevent unauthorized access to repository secrets or resources.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::ActionSettings;
///
/// let settings = ActionSettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ActionSettings {
    /// Whether GitHub Actions are enabled for repositories.
    /// When disabled, no workflows can be executed.
    pub enabled: Option<OverridableValue<bool>>,

    /// Default permissions for workflow tokens.
    /// Controls what API operations workflows can perform.
    pub default_workflow_permissions: Option<OverridableValue<WorkflowPermission>>,
}

impl ActionSettings {
    /// Creates a new empty `ActionSettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `ActionSettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            enabled: None,
            default_workflow_permissions: None,
        }
    }
}

impl Default for ActionSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Branch protection configuration settings.
///
/// Defines rules and policies that protect important branches from direct pushes,
/// require review processes, and enforce status checks before merging.
///
/// # Security Implications
///
/// Branch protection is a critical security feature that should typically be
/// marked as fixed in global defaults to prevent circumvention.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::BranchProtectionSettings;
///
/// let protection = BranchProtectionSettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BranchProtectionSettings {
    /// Whether branch protection is enabled.
    pub enabled: Option<OverridableValue<bool>>,

    /// Require pull request reviews before merging.
    pub require_pull_request_reviews: Option<OverridableValue<bool>>,

    /// Number of required reviewers.
    pub required_reviewers: Option<OverridableValue<u32>>,
}

impl BranchProtectionSettings {
    /// Creates a new empty `BranchProtectionSettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `BranchProtectionSettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            enabled: None,
            require_pull_request_reviews: None,
            required_reviewers: None,
        }
    }
}

impl Default for BranchProtectionSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Pull request configuration settings.
///
/// Controls merge strategies, review requirements, branch deletion policies,
/// and other pull request workflow behaviors.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::PullRequestSettings;
///
/// let settings = PullRequestSettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PullRequestSettings {
    /// Whether to delete head branches after merge.
    pub delete_branch_on_merge: Option<OverridableValue<bool>>,

    /// Whether to allow squash merging.
    pub allow_squash_merge: Option<OverridableValue<bool>>,

    /// Whether to allow merge commits.
    pub allow_merge_commit: Option<OverridableValue<bool>>,
}

impl PullRequestSettings {
    /// Creates a new empty `PullRequestSettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `PullRequestSettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            delete_branch_on_merge: None,
            allow_squash_merge: None,
            allow_merge_commit: None,
        }
    }
}

impl Default for PullRequestSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Push and commit policy settings.
///
/// Defines restrictions on direct pushes to protected branches,
/// commit signing requirements, and push validation rules.
///
/// # Security Implications
///
/// Push settings control direct access to repository branches and
/// should be configured to enforce security policies.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::PushSettings;
///
/// let settings = PushSettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PushSettings {
    /// Whether to allow force pushes to protected branches.
    pub allow_force_pushes: Option<OverridableValue<bool>>,

    /// Whether to require signed commits.
    pub require_signed_commits: Option<OverridableValue<bool>>,
}

impl PushSettings {
    /// Creates a new empty `PushSettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `PushSettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            allow_force_pushes: None,
            require_signed_commits: None,
        }
    }
}

impl Default for PushSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository-specific feature settings.
///
/// Controls the availability and configuration of repository features
/// such as issues, wiki, projects, and security advisories.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::RepositorySettings;
///
/// let settings = RepositorySettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RepositorySettings {
    /// Whether issues are enabled for the repository.
    pub has_issues: Option<OverridableValue<bool>>,

    /// Whether the wiki is enabled for the repository.
    pub has_wiki: Option<OverridableValue<bool>>,

    /// Whether projects are enabled for the repository.
    pub has_projects: Option<OverridableValue<bool>>,
}

impl RepositorySettings {
    /// Creates a new empty `RepositorySettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `RepositorySettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            has_issues: None,
            has_wiki: None,
            has_projects: None,
        }
    }
}

impl Default for RepositorySettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Organization-wide baseline settings that apply to all repositories.
///
/// This structure defines the default configuration values that serve as the foundation
/// for all repository configurations within an organization. These settings can be
/// overridden by more specific configurations (repository type, team, template) unless
/// explicitly marked as fixed.
///
/// Global defaults are loaded from `global/defaults.toml` in the metadata repository
/// and provide organization-wide policies and standards.
///
/// # Security Considerations
///
/// Global defaults often contain security policies and compliance requirements.
/// Use `OverridableValue::fixed()` for values that must not be changed by
/// higher-level configurations.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::{GlobalDefaults};
/// use config_manager::hierarchy::OverridableValue;
/// use config_manager::types::{RepositoryVisibility, MergeConfig, MergeType, CommitMessageOption, LabelConfig};
///
/// let mut defaults = GlobalDefaults::new();
/// defaults.branch_protection_enabled = Some(OverridableValue::fixed(true));
/// defaults.repository_visibility = Some(OverridableValue::overridable(RepositoryVisibility::Private));
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct GlobalDefaults {
    /// Default branch protection settings for all repositories.
    /// When fixed, ensures minimum security standards across the organization.
    pub branch_protection_enabled: Option<OverridableValue<bool>>,

    /// Default repository visibility setting.
    /// Organizations may enforce specific visibility policies for security or compliance.
    pub repository_visibility: Option<OverridableValue<RepositoryVisibility>>,

    /// Default merge configuration for pull requests.
    /// May be fixed to enforce consistent merge policies and commit message formats across the organization.
    pub merge_configuration: Option<OverridableValue<MergeConfig>>,

    /// Default set of repository labels to apply to all repositories.
    /// Teams and templates can add additional labels if not fixed.
    pub default_labels: Option<OverridableValue<Vec<LabelConfig>>>,

    /// Organization-wide webhook configurations.
    /// Typically fixed to ensure security and compliance monitoring.
    pub organization_webhooks: Option<OverridableValue<Vec<WebhookConfig>>>,

    /// Default GitHub Apps to install on all repositories.
    /// Usually fixed to ensure consistent tooling across the organization.
    /// Contains the slug names of GitHub Apps (e.g., "dependabot", "codecov").
    pub required_github_apps: Option<OverridableValue<Vec<String>>>,
}

impl GlobalDefaults {
    /// Creates a new empty `GlobalDefaults` configuration.
    ///
    /// All fields are initialized to `None`, allowing for gradual configuration
    /// building. Use the various setter methods or direct field assignment to
    /// populate the configuration.
    ///
    /// # Returns
    ///
    /// A new `GlobalDefaults` instance with all fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::GlobalDefaults;
    ///
    /// let defaults = GlobalDefaults::new();
    /// assert!(defaults.branch_protection_enabled.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            branch_protection_enabled: None,
            repository_visibility: None,
            merge_configuration: None,
            default_labels: None,
            organization_webhooks: None,
            required_github_apps: None,
        }
    }
}

impl Default for GlobalDefaults {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced global organization-wide baseline settings matching the specification.
///
/// This structure defines the default configuration values that apply across all repositories
/// in an organization. These settings serve as the foundation layer in the hierarchical
/// configuration system and can be overridden by team-specific or template-specific
/// configurations based on the individual field configurations.
///
/// Each field in the structure is optional, allowing organizations to configure only
/// the settings that are relevant to their needs. When a field is present, it defines
/// the baseline for that configuration area.
///
/// # Configuration Hierarchy
///
/// Global defaults serve as the base layer in the configuration hierarchy:
/// 1. **Global defaults** (this structure) - organization-wide baseline
/// 2. Repository type configuration - overrides for specific repository types
/// 3. Team configuration - team-specific overrides and additions
/// 4. Template configuration - final template-specific requirements
///
/// # Security Considerations
///
/// Organization-wide default settings often contain security-critical settings.
/// Individual fields may have their own override policies to prevent weakening
/// of security requirements by higher-level configurations.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::{OrganizationDefaults, ActionSettings};
/// use config_manager::hierarchy::OverridableValue;
/// use config_manager::types::WorkflowPermission;
///
/// let mut defaults = OrganizationDefaults::new();
/// defaults.actions = Some(ActionSettings {
///     enabled: Some(OverridableValue::fixed(true)),
///     default_workflow_permissions: Some(OverridableValue::overridable(WorkflowPermission::Read)),
/// });
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct OrganizationDefaults {
    /// GitHub Actions settings for repositories.
    pub actions: Option<ActionSettings>,

    /// Branch protection settings for repositories.
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Pull request settings for repositories.
    pub pull_requests: Option<PullRequestSettings>,

    /// Push and commit policy settings.
    pub push_settings: Option<PushSettings>,

    /// Repository feature settings.
    pub repository_settings: Option<RepositorySettings>,
}

impl OrganizationDefaults {
    /// Creates a new empty `OrganizationDefaults` configuration.
    ///
    /// # Returns
    ///
    /// A new `OrganizationDefaults` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            actions: None,
            branch_protection: None,
            pull_requests: None,
            push_settings: None,
            repository_settings: None,
        }
    }
}

impl Default for OrganizationDefaults {
    fn default() -> Self {
        Self::new()
    }
}

/// Team-specific configuration overrides and additions.
///
/// The `TeamConfig` structure allows teams within an organization to customize their
/// repository settings while respecting the override policies defined in global defaults.
/// Teams can override settings that are marked as overridable in the global configuration
/// and add team-specific configurations like webhooks, GitHub Apps, and environments.
///
/// # Examples
///
/// ```rust
/// use config_manager::settings::TeamConfig;
/// use config_manager::types::{RepositoryVisibility, WebhookConfig, WebhookEvent};
///
/// let mut team_config = TeamConfig::new();
///
/// // Override global repository visibility if allowed
/// team_config.repository_visibility = Some(RepositoryVisibility::Public);
///
/// // Add team-specific webhook
/// let webhook = WebhookConfig::new(
///     "https://backend-team.example.com/webhook".to_string(),
///     vec![WebhookEvent::Push, WebhookEvent::PullRequest],
///     true,
///     Some("team_secret".to_string())
/// );
/// team_config.team_webhooks = Some(vec![webhook]);
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TeamConfig {
    /// Override for repository visibility setting.
    pub repository_visibility: Option<RepositoryVisibility>,

    /// Override for branch protection enablement.
    pub branch_protection_enabled: Option<bool>,

    /// Override for merge configuration settings.
    pub merge_configuration: Option<MergeConfig>,

    /// Team-specific webhook configurations.
    pub team_webhooks: Option<Vec<WebhookConfig>>,

    /// Team-specific GitHub App installations.
    pub team_github_apps: Option<Vec<String>>,

    /// Team-specific repository labels.
    pub team_labels: Option<Vec<LabelConfig>>,

    /// Team-specific environment configurations.
    pub team_environments: Option<Vec<EnvironmentConfig>>,
}

impl TeamConfig {
    /// Creates a new empty `TeamConfig`.
    ///
    /// All fields are initialized to `None`, representing no team-specific overrides
    /// or additions. Use the various setter methods or direct field assignment to
    /// build the team configuration.
    ///
    /// # Returns
    ///
    /// A new `TeamConfig` instance with all fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::TeamConfig;
    ///
    /// let team_config = TeamConfig::new();
    /// assert!(team_config.repository_visibility.is_none());
    /// assert!(team_config.team_webhooks.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            repository_visibility: None,
            branch_protection_enabled: None,
            merge_configuration: None,
            team_webhooks: None,
            team_github_apps: None,
            team_labels: None,
            team_environments: None,
        }
    }

    /// Validates that team configuration overrides are allowed by global defaults.
    ///
    /// This method checks that any overrides specified in the team configuration
    /// are permitted by the corresponding global default settings. It validates
    /// that only settings with `override_allowed = true` are being overridden.
    ///
    /// # Arguments
    ///
    /// * `global_defaults` - The global configuration to validate against
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all team overrides are valid
    /// * `Err(ConfigurationError)` if any overrides violate global policies
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationError::OverrideNotAllowed` if the team attempts to
    /// override a setting that is marked as fixed in the global defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::{TeamConfig, GlobalDefaults};
    /// use config_manager::hierarchy::OverridableValue;
    /// use config_manager::types::RepositoryVisibility;
    ///
    /// let mut global = GlobalDefaults::new();
    /// global.repository_visibility = Some(OverridableValue::overridable(RepositoryVisibility::Private));
    ///
    /// let mut team = TeamConfig::new();
    /// team.repository_visibility = Some(RepositoryVisibility::Public);
    ///
    /// // This should succeed because global allows override
    /// assert!(team.validate_overrides(&global).is_ok());
    /// ```
    pub fn validate_overrides(
        &self,
        global_defaults: &GlobalDefaults,
    ) -> Result<(), ConfigurationError> {
        // Check repository visibility override
        if let Some(_team_visibility) = &self.repository_visibility {
            if let Some(global_visibility) = &global_defaults.repository_visibility {
                if !global_visibility.can_override() {
                    return Err(ConfigurationError::OverrideNotAllowed {
                        field: "repository_visibility".to_string(),
                        attempted_value: format!("{:?}", _team_visibility),
                        global_value: format!("{:?}", global_visibility.value()),
                    });
                }
            }
        }

        // Check branch protection override
        if let Some(_team_protection) = &self.branch_protection_enabled {
            if let Some(global_protection) = &global_defaults.branch_protection_enabled {
                if !global_protection.can_override() {
                    return Err(ConfigurationError::OverrideNotAllowed {
                        field: "branch_protection_enabled".to_string(),
                        attempted_value: _team_protection.to_string(),
                        global_value: global_protection.value().to_string(),
                    });
                }
            }
        }

        // Check merge configuration override
        if let Some(_team_merge) = &self.merge_configuration {
            if let Some(global_merge) = &global_defaults.merge_configuration {
                if !global_merge.can_override() {
                    return Err(ConfigurationError::OverrideNotAllowed {
                        field: "merge_configuration".to_string(),
                        attempted_value: format!("{:?}", _team_merge),
                        global_value: format!("{:?}", global_merge.value()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Checks if the team configuration has any override settings specified.
    ///
    /// This method returns `true` if the team has specified any values that would
    /// override global defaults, as opposed to purely additive configurations
    /// like team-specific webhooks or labels.
    ///
    /// # Returns
    ///
    /// `true` if any override settings are specified, `false` if the configuration
    /// is purely additive or empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::TeamConfig;
    /// use config_manager::types::RepositoryVisibility;
    ///
    /// let mut team = TeamConfig::new();
    /// assert!(!team.has_overrides());
    ///
    /// team.repository_visibility = Some(RepositoryVisibility::Public);
    /// assert!(team.has_overrides());
    /// ```
    pub fn has_overrides(&self) -> bool {
        self.repository_visibility.is_some()
            || self.branch_protection_enabled.is_some()
            || self.merge_configuration.is_some()
    }

    /// Checks if the team configuration has any additive settings specified.
    ///
    /// This method returns `true` if the team has specified any additive configurations
    /// such as team-specific webhooks, GitHub Apps, labels, or environments that
    /// extend rather than override global settings.
    ///
    /// # Returns
    ///
    /// `true` if any additive settings are specified, `false` if the configuration
    /// contains only overrides or is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::settings::TeamConfig;
    /// use config_manager::types::{WebhookConfig, WebhookEvent};
    ///
    /// let mut team = TeamConfig::new();
    /// assert!(!team.has_additions());
    ///
    /// let webhook = WebhookConfig::new(
    ///     "https://team.example.com/webhook".to_string(),
    ///     vec![WebhookEvent::Push],
    ///     true,
    ///     None
    /// );
    /// team.team_webhooks = Some(vec![webhook]);
    /// assert!(team.has_additions());
    /// ```
    pub fn has_additions(&self) -> bool {
        self.team_webhooks.is_some()
            || self.team_github_apps.is_some()
            || self.team_labels.is_some()
            || self.team_environments.is_some()
    }
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self::new()
    }
}
