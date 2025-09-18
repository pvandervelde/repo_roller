//! Organization-specific repository configuration structures.
//!
//! This module provides the foundational data structures for hierarchical configuration management
//! in the organization-specific repository settings system. It implements a four-level configuration
//! hierarchy: Template > Team > Repository Type > Global.
//!
//! The system supports sophisticated override controls, allowing templates to specify whether
//! their values can be overridden by higher-level configurations, ensuring security and
//! consistency policies can be enforced.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Unit tests for organization configuration structures
#[path = "organization_tests.rs"]
#[cfg(test)]
mod tests;

/// Repository visibility options in GitHub.
///
/// This enum represents the three possible visibility settings for GitHub repositories.
/// Organizations can enforce specific visibility policies through configuration.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::RepositoryVisibility;
///
/// let visibility = RepositoryVisibility::Private;
/// assert_eq!(format!("{:?}", visibility), "Private");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryVisibility {
    /// Repository is publicly accessible to anyone.
    Public,
    /// Repository is only accessible to organization members and collaborators.
    Private,
    /// Repository is accessible to organization members (GitHub Enterprise feature).
    Internal,
}

/// Pull request merge strategies available in GitHub.
///
/// This enum represents the different ways pull requests can be merged into the target branch.
/// Organizations can restrict which merge types are allowed to enforce consistent workflows.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::MergeType;
///
/// let merge_type = MergeType::Squash;
/// assert_eq!(format!("{:?}", merge_type), "Squash");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum MergeType {
    /// Creates a merge commit that combines all commits from the feature branch.
    Merge,
    /// Combines all commits from the feature branch into a single commit.
    Squash,
    /// Replays commits from the feature branch onto the target branch without a merge commit.
    Rebase,
}

/// Commit message options for merge and squash operations.
///
/// This enum defines how commit messages should be generated when merging or squashing
/// pull requests. Different options provide varying levels of detail and consistency.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::CommitMessageOption;
///
/// let option = CommitMessageOption::PullRequestTitle;
/// assert_eq!(format!("{:?}", option), "PullRequestTitle");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CommitMessageOption {
    /// Use GitHub's default commit message format.
    DefaultMessage,
    /// Use only the pull request title as the commit message.
    PullRequestTitle,
    /// Use the pull request title and description as the commit message.
    PullRequestTitleAndDescription,
    /// Use the pull request title and include commit details in the message.
    PullRequestTitleAndCommitDetails,
}

/// GitHub Actions workflow token permissions.
///
/// This enum represents the different permission levels that can be assigned to
/// workflow tokens by default. These permissions control what API operations
/// workflows can perform against the repository.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::WorkflowPermission;
///
/// let permission = WorkflowPermission::Read;
/// assert_eq!(format!("{:?}", permission), "Read");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowPermission {
    /// No permissions - most restrictive setting.
    None,
    /// Read access to repository contents and metadata.
    Read,
    /// Read and write access to repository contents and metadata.
    Write,
}

/// Webhook events that can trigger notifications.
///
/// This enum represents the different GitHub events that can trigger webhook notifications.
/// Organizations can specify which events should trigger their webhooks for monitoring
/// and automation purposes.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::WebhookEvent;
///
/// let event = WebhookEvent::Push;
/// assert_eq!(format!("{:?}", event), "Push");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Triggered when commits are pushed to a repository.
    Push,
    /// Triggered when a pull request is opened, closed, or updated.
    PullRequest,
    /// Triggered when an issue is opened, closed, or updated.
    Issues,
    /// Triggered when a new release is published.
    Release,
    /// Triggered when repository settings or collaborators are changed.
    Repository,
    /// Triggered when a new deployment is created.
    Deployment,
    /// Triggered when a deployment status is updated.
    DeploymentStatus,
    /// Triggered when a check run is created or updated.
    CheckRun,
    /// Triggered when a check suite is created or updated.
    CheckSuite,
    /// Triggered when repository stars change.
    Star,
    /// Triggered when repository watchers change.
    Watch,
    /// Triggered when a repository is forked.
    Fork,
    /// Triggered when a commit comment is created.
    CommitComment,
    /// Triggered when a pull request review is submitted.
    PullRequestReview,
    /// Triggered when a pull request review comment is created.
    PullRequestReviewComment,
    /// Triggered when an issue comment is created.
    IssueComment,
}

/// Configuration for a repository label.
///
/// Labels in GitHub repositories have a name, description, and color. This struct
/// provides a complete representation of label configuration that can be applied
/// during repository creation or updates.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::LabelConfig;
///
/// let bug_label = LabelConfig {
///     name: "bug".to_string(),
///     description: Some("Something isn't working".to_string()),
///     color: "d73a4a".to_string(),
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelConfig {
    /// The name of the label (e.g., "bug", "enhancement", "documentation").
    ///
    /// This is the text that will be displayed on issues and pull requests.
    /// Label names must be unique within a repository.
    pub name: String,

    /// Optional description explaining when this label should be used.
    ///
    /// Descriptions help contributors understand the purpose and appropriate
    /// usage of each label.
    pub description: Option<String>,

    /// The hexadecimal color code for the label (without '#' prefix).
    ///
    /// Must be a valid 6-character hexadecimal color code (e.g., "d73a4a" for red).
    /// The color helps visually categorize and identify different types of issues.
    pub color: String,
}

impl LabelConfig {
    /// Creates a new label configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the label
    /// * `description` - Optional description for the label
    /// * `color` - Hexadecimal color code (without '#' prefix)
    ///
    /// # Returns
    ///
    /// A new `LabelConfig` instance with the specified properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::LabelConfig;
    ///
    /// let label = LabelConfig::new(
    ///     "bug".to_string(),
    ///     Some("Something isn't working".to_string()),
    ///     "d73a4a".to_string()
    /// );
    /// assert_eq!(label.name, "bug");
    /// ```
    pub fn new(name: String, description: Option<String>, color: String) -> Self {
        Self {
            name,
            description,
            color,
        }
    }
}

/// Configuration for a webhook endpoint.
///
/// Webhooks allow external services to be notified when certain events happen
/// in a repository. This struct defines the configuration for webhook endpoints
/// that should be automatically configured on repositories.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{WebhookConfig, WebhookEvent};
///
/// let webhook = WebhookConfig {
///     url: "https://example.com/webhook".to_string(),
///     events: vec![WebhookEvent::Push, WebhookEvent::PullRequest],
///     active: true,
///     secret: Some("webhook_secret".to_string()),
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WebhookConfig {
    /// The URL endpoint that will receive webhook payloads.
    ///
    /// Must be a valid HTTP or HTTPS URL that can receive POST requests
    /// from GitHub's webhook delivery system.
    pub url: String,

    /// List of GitHub events that will trigger this webhook.
    ///
    /// Specifies which repository events should trigger webhook notifications.
    /// See `WebhookEvent` enum for available event types.
    pub events: Vec<WebhookEvent>,

    /// Whether this webhook is active and should receive events.
    ///
    /// Inactive webhooks remain configured but do not receive event notifications.
    pub active: bool,

    /// Optional secret used to secure webhook payloads.
    ///
    /// When provided, GitHub will use this secret to generate a hash signature
    /// that can be used to verify the authenticity of webhook payloads.
    pub secret: Option<String>,
}

impl WebhookConfig {
    /// Creates a new webhook configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - The webhook endpoint URL
    /// * `events` - List of GitHub events that trigger this webhook
    /// * `active` - Whether the webhook should be active
    /// * `secret` - Optional secret for payload verification
    ///
    /// # Returns
    ///
    /// A new `WebhookConfig` instance with the specified properties.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{WebhookConfig, WebhookEvent};
    ///
    /// let webhook = WebhookConfig::new(
    ///     "https://example.com/webhook".to_string(),
    ///     vec![WebhookEvent::Push, WebhookEvent::PullRequest],
    ///     true,
    ///     Some("secret".to_string())
    /// );
    /// assert_eq!(webhook.url, "https://example.com/webhook");
    /// ```
    pub fn new(
        url: String,
        events: Vec<WebhookEvent>,
        active: bool,
        secret: Option<String>,
    ) -> Self {
        Self {
            url,
            events,
            active,
            secret,
        }
    }
}

/// Configuration for pull request merge settings.
///
/// This struct defines how pull requests should be merged, including which merge types
/// are allowed and how commit messages should be formatted for each type.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{MergeConfig, MergeType, CommitMessageOption};
///
/// let merge_config = MergeConfig {
///     allowed_types: vec![MergeType::Merge, MergeType::Squash],
///     merge_commit_message: CommitMessageOption::PullRequestTitle,
///     squash_commit_message: CommitMessageOption::PullRequestTitleAndDescription,
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct MergeConfig {
    /// List of merge types allowed for pull requests.
    ///
    /// Organizations can restrict which merge strategies are available to enforce
    /// consistent workflows and maintain repository history standards.
    pub allowed_types: Vec<MergeType>,

    /// Commit message format for merge commits.
    ///
    /// Specifies how commit messages should be generated when using the merge strategy.
    /// This setting applies to merge commits that combine feature branch commits.
    pub merge_commit_message: CommitMessageOption,

    /// Commit message format for squash commits.
    ///
    /// Specifies how commit messages should be generated when using the squash strategy.
    /// This setting applies to squash commits that combine all feature branch commits into one.
    pub squash_commit_message: CommitMessageOption,
}

impl MergeConfig {
    /// Creates a new merge configuration.
    ///
    /// # Arguments
    ///
    /// * `allowed_types` - List of merge types that are allowed
    /// * `merge_commit_message` - Commit message format for merge commits
    /// * `squash_commit_message` - Commit message format for squash commits
    ///
    /// # Returns
    ///
    /// A new `MergeConfig` instance with the specified settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergeConfig, MergeType, CommitMessageOption};
    ///
    /// let config = MergeConfig::new(
    ///     vec![MergeType::Squash],
    ///     CommitMessageOption::PullRequestTitle,
    ///     CommitMessageOption::PullRequestTitleAndDescription
    /// );
    /// assert_eq!(config.allowed_types, vec![MergeType::Squash]);
    /// ```
    pub fn new(
        allowed_types: Vec<MergeType>,
        merge_commit_message: CommitMessageOption,
        squash_commit_message: CommitMessageOption,
    ) -> Self {
        Self {
            allowed_types,
            merge_commit_message,
            squash_commit_message,
        }
    }
}

/// A generic wrapper for configuration values that supports override control.
///
/// This type allows configuration values to specify whether they can be overridden
/// by higher levels in the configuration hierarchy. This is essential for security
/// and policy enforcement where certain template requirements must be preserved.
///
/// # Type Parameters
///
/// * `T` - The type of the configuration value. Must implement `Clone` for merging operations.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::OverridableValue;
///
/// // A value that can be overridden by higher levels
/// let flexible_setting = OverridableValue::new("default_value".to_string(), true);
///
/// // A security-critical value that cannot be overridden
/// let fixed_setting = OverridableValue::new("required_value".to_string(), false);
///
/// // Check if override is allowed
/// assert!(flexible_setting.can_override());
/// assert!(!fixed_setting.can_override());
///
/// // Get the actual value
/// assert_eq!(flexible_setting.value(), "default_value");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OverridableValue<T: Clone> {
    /// The actual configuration value.
    value: T,
    /// Whether this value can be overridden by higher levels in the hierarchy.
    /// When `false`, this value is considered fixed and must not be changed.
    #[serde(rename = "override_allowed")]
    can_override: bool,
}

impl<T: Clone> OverridableValue<T> {
    /// Creates a new `OverridableValue` with the specified value and override permission.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    /// * `can_override` - Whether this value can be overridden by higher hierarchy levels
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` instance containing the provided value and override setting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let setting = OverridableValue::new(42, true);
    /// assert_eq!(setting.value(), 42);
    /// assert!(setting.can_override());
    /// ```
    pub fn new(value: T, can_override: bool) -> Self {
        Self {
            value,
            can_override,
        }
    }

    /// Creates a new `OverridableValue` that allows override (convenience method).
    ///
    /// This is equivalent to calling `new(value, true)` but provides a more readable
    /// way to create overridable values.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` that can be overridden by higher hierarchy levels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let setting = OverridableValue::overridable("flexible_value".to_string());
    /// assert!(setting.can_override());
    /// ```
    pub fn overridable(value: T) -> Self {
        Self::new(value, true)
    }

    /// Creates a new `OverridableValue` that cannot be overridden (convenience method).
    ///
    /// This is equivalent to calling `new(value, false)` but provides a more readable
    /// way to create fixed values that enforce security or policy requirements.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to wrap
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` that cannot be overridden by higher hierarchy levels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let setting = OverridableValue::fixed("security_requirement".to_string());
    /// assert!(!setting.can_override());
    /// ```
    pub fn fixed(value: T) -> Self {
        Self::new(value, false)
    }

    /// Returns a reference to the wrapped configuration value.
    ///
    /// # Returns
    ///
    /// A reference to the configuration value of type `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let setting = OverridableValue::new(42, true);
    /// assert_eq!(setting.value(), 42);
    /// ```
    pub fn value(&self) -> T {
        self.value.clone()
    }

    /// Returns whether this value can be overridden by higher hierarchy levels.
    ///
    /// # Returns
    ///
    /// `true` if the value can be overridden, `false` if it is fixed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let flexible = OverridableValue::overridable("test".to_string());
    /// let fixed = OverridableValue::fixed("test".to_string());
    ///
    /// assert!(flexible.can_override());
    /// assert!(!fixed.can_override());
    /// ```
    pub fn can_override(&self) -> bool {
        self.can_override
    }

    /// Attempts to override this value with a new value from a higher hierarchy level.
    ///
    /// This operation will only succeed if `can_override` is `true`. If the value
    /// cannot be overridden, the original value is returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `new_value` - The new value to apply if override is allowed
    ///
    /// # Returns
    ///
    /// A new `OverridableValue` with the new value if override is allowed,
    /// or the original value if override is not permitted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::OverridableValue;
    ///
    /// let flexible = OverridableValue::overridable(42);
    /// let overridden = flexible.try_override(100);
    /// assert_eq!(overridden.value(), 100);
    ///
    /// let fixed = OverridableValue::fixed(42);
    /// let unchanged = fixed.try_override(100);
    /// assert_eq!(unchanged.value(), 42);
    /// ```
    pub fn try_override(&self, new_value: T) -> Self {
        if self.can_override {
            Self::new(new_value, self.can_override)
        } else {
            self.clone()
        }
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
/// use config_manager::organization::{GlobalDefaults, OverridableValue, RepositoryVisibility, MergeConfig, MergeType, CommitMessageOption, LabelConfig};
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
    /// use config_manager::organization::GlobalDefaults;
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

/// Team-specific configuration overrides and additions.
///
/// The `TeamConfig` structure allows teams within an organization to customize their
/// repository settings while respecting the override policies defined in global defaults.
/// Teams can override settings that are marked as overridable in the global configuration
/// and add team-specific configurations like webhooks, GitHub Apps, and environments.
///
/// # Configuration Hierarchy
///
/// Team configurations are applied after global defaults but before template configurations
/// in the hierarchical merging process:
/// 1. Global defaults (baseline)
/// 2. **Team configuration** (team-specific overrides and additions)
/// 3. Repository type configuration (type-specific settings)
/// 4. Template configuration (highest precedence)
///
/// # Override Control
///
/// Teams can only override global settings that have `override_allowed = true`.
/// Attempting to override fixed global settings will result in validation errors
/// during configuration merging.
///
/// # Team-Specific Features
///
/// Teams can add configurations that are purely additive and don't conflict with
/// global policies:
/// - Team-specific webhooks for notifications and automation
/// - Additional GitHub Apps for team-specific workflows
/// - Custom environments for team deployment processes
/// - Team-specific labels for issue and PR management
///
/// # Security Considerations
///
/// Team configurations are loaded from the metadata repository under `teams/{team}/config.toml`.
/// Access to modify team configurations should be restricted to team leads or
/// repository administrators to prevent unauthorized changes to team policies.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{TeamConfig, OverridableValue, RepositoryVisibility, WebhookConfig, WebhookEvent};
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
///
/// // Add team-specific GitHub Apps
/// team_config.team_github_apps = Some(vec!["team-specific-app".to_string()]);
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TeamConfig {
    /// Override for repository visibility setting.
    ///
    /// If the global default allows override (`override_allowed = true`), teams can
    /// specify a different repository visibility. This is useful for teams that need
    /// public repositories for open source projects while the organization defaults to private.
    ///
    /// # Override Policy
    ///
    /// This setting can only be applied if the global default has `override_allowed = true`.
    /// If the global setting is fixed, this field will be ignored during merging and
    /// may generate validation warnings.
    pub repository_visibility: Option<RepositoryVisibility>,

    /// Override for branch protection enablement.
    ///
    /// Teams can disable branch protection if the global policy allows it, though this
    /// is typically not recommended for production repositories. Most organizations
    /// will set this as a fixed requirement in global defaults.
    ///
    /// # Override Policy
    ///
    /// This setting can only be applied if the global default has `override_allowed = true`.
    /// Security-conscious organizations typically make this a fixed requirement.
    pub branch_protection_enabled: Option<bool>,

    /// Override for merge configuration settings.
    ///
    /// Teams can customize their merge strategies and commit message formats if the
    /// global policy allows overrides. This enables teams to choose workflows that
    /// best suit their development practices while maintaining organizational consistency.
    ///
    /// # Override Policy
    ///
    /// This setting can only be applied if the global default has `override_allowed = true`.
    /// Organizations may fix this to ensure consistent merge practices.
    pub merge_configuration: Option<MergeConfig>,

    /// Team-specific webhook configurations.
    ///
    /// These webhooks are additive to any organization-wide webhooks defined in global
    /// defaults. Teams can add webhooks for team-specific notifications, automation,
    /// or integration with team tools without affecting other teams.
    ///
    /// # Additive Behavior
    ///
    /// Team webhooks are merged additively with global webhooks. Both will be configured
    /// on repositories created by the team. Duplicate webhook URLs are not automatically
    /// deduplicated, so teams should ensure they don't conflict with global webhooks.
    ///
    /// # Security Considerations
    ///
    /// Webhook URLs should be validated to ensure they point to approved team services.
    /// Secrets should be properly secured and not logged in configuration audit trails.
    pub team_webhooks: Option<Vec<WebhookConfig>>,

    /// Team-specific GitHub App installations.
    ///
    /// These GitHub Apps are additive to any required apps defined in global defaults.
    /// Teams can specify additional apps they need for their specific workflows,
    /// such as team-specific deployment tools or code analysis services.
    ///
    /// # Additive Behavior
    ///
    /// Team GitHub Apps are merged additively with required global apps. All apps
    /// (global required + team-specific) will be installed on team repositories.
    /// Duplicate app installations are handled gracefully by the GitHub API.
    ///
    /// # App Specification
    ///
    /// GitHub Apps are specified by their slug name (e.g., "dependabot", "codecov").
    /// The actual installation and permission configuration is handled during
    /// repository creation through the GitHub Apps API.
    pub team_github_apps: Option<Vec<String>>,

    /// Team-specific repository labels.
    ///
    /// These labels are additive to any default labels defined in global defaults.
    /// Teams can define labels for team-specific issue tracking, project management,
    /// or workflow categorization that supplement the organization-wide label set.
    ///
    /// # Additive Behavior
    ///
    /// Team labels are merged additively with global default labels. If a team label
    /// has the same name as a global label, the team label takes precedence for
    /// repositories created by that team.
    ///
    /// # Label Management
    ///
    /// Labels are created and updated automatically during repository creation.
    /// Changes to team label configurations will only affect newly created repositories
    /// unless explicitly synchronized to existing repositories.
    pub team_labels: Option<Vec<LabelConfig>>,

    /// Team-specific environment configurations.
    ///
    /// Environments define deployment targets with protection rules and secrets.
    /// Teams can define environments for their specific deployment needs, such as
    /// staging environments, preview environments, or team-specific production environments.
    ///
    /// # Environment Management
    ///
    /// Environments are created during repository setup and can include protection rules
    /// such as required reviewers, wait timers, and deployment branch restrictions.
    /// Environment-specific secrets and variables must be configured separately through
    /// the GitHub API after repository creation.
    ///
    /// # Security Considerations
    ///
    /// Environment protection rules should be carefully configured to prevent unauthorized
    /// deployments. Required reviewers should include team leads or designated deployment
    /// approvers to maintain proper change control.
    pub team_environments: Option<Vec<EnvironmentConfig>>,
}

/// Configuration for a deployment environment.
///
/// Environments in GitHub provide deployment protection rules and allow teams to
/// configure deployment gates, required reviewers, and deployment branch restrictions.
/// Each environment can have its own protection rules and is managed independently.
///
/// # Environment Protection
///
/// Environments can include various protection mechanisms:
/// - Required reviewers who must approve deployments
/// - Wait timers that delay deployments for a specified period
/// - Deployment branch restrictions that limit which branches can deploy
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::EnvironmentConfig;
///
/// // Production environment with strict protection
/// let production = EnvironmentConfig {
///     name: "production".to_string(),
///     required_reviewers: Some(vec!["@team-leads".to_string(), "@security-team".to_string()]),
///     wait_timer: Some(300), // 5 minute wait timer
///     deployment_branch_policy: Some("main".to_string()),
/// };
///
/// // Staging environment with minimal protection
/// let staging = EnvironmentConfig {
///     name: "staging".to_string(),
///     required_reviewers: Some(vec!["@team-leads".to_string()]),
///     wait_timer: None,
///     deployment_branch_policy: None, // Any branch can deploy to staging
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentConfig {
    /// The name of the environment (e.g., "production", "staging", "preview").
    ///
    /// Environment names must be unique within a repository and should follow
    /// your organization's naming conventions. Common patterns include
    /// environment-purpose combinations like "prod", "staging", "dev".
    pub name: String,

    /// List of required reviewers for deployments to this environment.
    ///
    /// Reviewers can be specified as:
    /// - Individual users: "@username"
    /// - Teams: "@org/team-name"
    /// - Organization roles: "@org/role-name"
    ///
    /// At least one required reviewer must approve before deployment can proceed.
    /// This provides change control and oversight for sensitive environments.
    pub required_reviewers: Option<Vec<String>>,

    /// Wait timer in seconds before deployments can proceed.
    ///
    /// Provides a cooling-off period for last-minute reviews or to coordinate
    /// with other systems. Commonly used for production environments to allow
    /// time for final validation or to schedule deployments during maintenance windows.
    ///
    /// Set to `None` for no wait timer, or specify seconds (e.g., 300 for 5 minutes).
    pub wait_timer: Option<u32>,

    /// Deployment branch policy restricting which branches can deploy.
    ///
    /// Can specify:
    /// - Specific branch name: "main", "release", etc.
    /// - Branch pattern: "release/*", "hotfix/*"
    /// - `None` to allow deployment from any branch
    ///
    /// This helps enforce deployment practices and prevents accidental deployments
    /// from feature branches to production environments.
    pub deployment_branch_policy: Option<String>,
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
    /// use config_manager::organization::TeamConfig;
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
    /// use config_manager::organization::{TeamConfig, GlobalDefaults, OverridableValue, RepositoryVisibility};
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
        if let Some(team_visibility) = &self.repository_visibility {
            if let Some(global_visibility) = &global_defaults.repository_visibility {
                if !global_visibility.can_override() {
                    return Err(ConfigurationError::OverrideNotAllowed {
                        field: "repository_visibility".to_string(),
                        attempted_value: format!("{:?}", team_visibility),
                        global_value: format!("{:?}", global_visibility.value()),
                    });
                }
            }
        }

        // Check branch protection override
        if let Some(team_protection) = &self.branch_protection_enabled {
            if let Some(global_protection) = &global_defaults.branch_protection_enabled {
                if !global_protection.can_override() {
                    return Err(ConfigurationError::OverrideNotAllowed {
                        field: "branch_protection_enabled".to_string(),
                        attempted_value: team_protection.to_string(),
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
                        attempted_value: "custom_merge_config".to_string(),
                        global_value: "fixed_merge_config".to_string(),
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
    /// use config_manager::organization::{TeamConfig, RepositoryVisibility};
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
    /// use config_manager::organization::{TeamConfig, WebhookConfig, WebhookEvent};
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

impl EnvironmentConfig {
    /// Creates a new environment configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the environment
    /// * `required_reviewers` - Optional list of required reviewers
    /// * `wait_timer` - Optional wait timer in seconds
    /// * `deployment_branch_policy` - Optional branch restriction policy
    ///
    /// # Returns
    ///
    /// A new `EnvironmentConfig` instance with the specified settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::EnvironmentConfig;
    ///
    /// let env = EnvironmentConfig::new(
    ///     "production".to_string(),
    ///     Some(vec!["@team-leads".to_string()]),
    ///     Some(300),
    ///     Some("main".to_string())
    /// );
    /// assert_eq!(env.name, "production");
    /// ```
    pub fn new(
        name: String,
        required_reviewers: Option<Vec<String>>,
        wait_timer: Option<u32>,
        deployment_branch_policy: Option<String>,
    ) -> Self {
        Self {
            name,
            required_reviewers,
            wait_timer,
            deployment_branch_policy,
        }
    }
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration validation and merging errors.
///
/// This enum represents the various errors that can occur during configuration
/// validation, merging, and processing. Each variant provides specific context
/// about the nature of the error to help with troubleshooting and resolution.
///
/// # Error Categories
///
/// - **Override Violations**: Attempts to override fixed settings
/// - **Validation Failures**: Schema or business rule violations
/// - **Missing Dependencies**: Required configurations not found
/// - **Format Errors**: Invalid configuration file formats
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::ConfigurationError;
///
/// let error = ConfigurationError::OverrideNotAllowed {
///     field: "branch_protection_enabled".to_string(),
///     attempted_value: "false".to_string(),
///     global_value: "true".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationError {
    /// Attempted to override a setting that is marked as fixed in global defaults.
    ///
    /// This error occurs when a team or template configuration tries to override
    /// a global setting that has `override_allowed = false`. The error includes
    /// details about which field was being overridden and what values were involved.
    OverrideNotAllowed {
        /// The configuration field that cannot be overridden.
        field: String,
        /// The value that was attempted to be set.
        attempted_value: String,
        /// The fixed value from global defaults.
        global_value: String,
    },

    /// Configuration value failed validation rules.
    ///
    /// This error occurs when a configuration value doesn't meet schema requirements
    /// or business rules, such as invalid webhook URLs, malformed color codes for labels,
    /// or unsupported merge strategies.
    InvalidValue {
        /// The configuration field that has an invalid value.
        field: String,
        /// The invalid value that was provided.
        value: String,
        /// Description of why the value is invalid.
        reason: String,
    },

    /// Required configuration field is missing.
    ///
    /// This error occurs when a mandatory configuration field is not provided
    /// in contexts where it's required, such as missing global defaults that
    /// are needed for security compliance.
    RequiredFieldMissing {
        /// The name of the missing required field.
        field: String,
        /// Context about where the field was expected.
        context: String,
    },

    /// Configuration file format is invalid or corrupted.
    ///
    /// This error occurs when configuration files cannot be parsed due to
    /// syntax errors, unsupported formats, or schema violations.
    FormatError {
        /// The file that contains the format error.
        file: String,
        /// Description of the format error.
        error: String,
    },
}

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
/// use config_manager::organization::ActionSettings;
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
/// use config_manager::organization::BranchProtectionSettings;
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

/// Custom repository property definition.
///
/// Represents metadata properties that can be attached to repositories for
/// categorization, compliance tracking, and automation workflows.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::CustomProperty;
///
/// let property = CustomProperty::new(
///     "team".to_string(),
///     "backend".to_string()
/// );
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CustomProperty {
    /// The name of the custom property.
    pub property_name: String,

    /// The value of the custom property.
    pub value: String,
}

impl CustomProperty {
    /// Creates a new custom property.
    ///
    /// # Arguments
    ///
    /// * `property_name` - The name of the property
    /// * `value` - The value of the property
    ///
    /// # Returns
    ///
    /// A new `CustomProperty` instance.
    pub fn new(property_name: String, value: String) -> Self {
        Self {
            property_name,
            value,
        }
    }
}

/// Environment configuration for deployment workflows.
///
/// Defines deployment targets with protection rules, required reviewers,
/// and environment-specific secrets and variables.
///
/// GitHub App configuration for repository integration.
///
/// Defines GitHub Apps that should be installed on repositories for
/// automation, security scanning, or development workflow enhancement.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::GitHubAppConfig;
///
/// let app = GitHubAppConfig::new("dependabot".to_string());
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct GitHubAppConfig {
    /// The slug name of the GitHub App.
    pub app_slug: String,

    /// Whether the app installation is required.
    pub required: Option<OverridableValue<bool>>,
}

impl GitHubAppConfig {
    /// Creates a new GitHub App configuration.
    ///
    /// # Arguments
    ///
    /// * `app_slug` - The slug name of the GitHub App
    ///
    /// # Returns
    ///
    /// A new `GitHubAppConfig` instance.
    pub fn new(app_slug: String) -> Self {
        Self {
            app_slug,
            required: None,
        }
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
/// use config_manager::organization::PullRequestSettings;
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
/// use config_manager::organization::PushSettings;
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
/// use config_manager::organization::RepositorySettings;
///
/// let settings = RepositorySettings::new();
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RepositorySettings {
    /// Whether to enable issues for the repository.
    pub issues: Option<OverridableValue<bool>>,

    /// Whether to enable the wiki for the repository.
    pub wiki: Option<OverridableValue<bool>>,

    /// Whether to enable projects for the repository.
    pub projects: Option<OverridableValue<bool>>,

    /// Whether to enable discussions for the repository.
    pub discussions: Option<OverridableValue<bool>>,
}

impl RepositorySettings {
    /// Creates a new empty `RepositorySettings` configuration.
    ///
    /// # Returns
    ///
    /// A new `RepositorySettings` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self {
            issues: None,
            wiki: None,
            projects: None,
            discussions: None,
        }
    }
}

impl Default for RepositorySettings {
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
/// Global defaults often contain security-critical settings. Individual fields may
/// have their own override policies to prevent weakening of security requirements
/// by higher-level configurations.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{GlobalDefaultsEnhanced, ActionSettings, OverridableValue, WorkflowPermission};
///
/// let mut defaults = GlobalDefaultsEnhanced::new();
/// defaults.actions = Some(ActionSettings {
///     enabled: Some(OverridableValue::fixed(true)),
///     default_workflow_permissions: Some(OverridableValue::overridable(WorkflowPermission::Read)),
/// });
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct GlobalDefaultsEnhanced {
    /// GitHub Actions settings for workflow execution and permissions.
    /// Controls whether Actions are enabled and execution policies.
    pub actions: Option<ActionSettings>,

    /// Branch protection rules and policies.
    /// Defines required status checks, review requirements, and merge restrictions.
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Custom repository properties for organization-wide metadata.
    /// Used for categorization, compliance tracking, and automation.
    pub custom_properties: Option<Vec<CustomProperty>>,

    /// Environment configurations for deployment workflows.
    /// Defines deployment targets, protection rules, and secrets.
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// GitHub Apps to install on all repositories.
    /// Usually fixed to ensure consistent tooling across the organization.
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Pull request settings and policies.
    /// Controls merge strategies, review requirements, and automation.
    pub pull_requests: Option<PullRequestSettings>,

    /// Push and commit policies.
    /// Defines restrictions on direct pushes and commit requirements.
    pub push: Option<PushSettings>,

    /// Repository-specific settings and features.
    /// Controls issues, wiki, projects, and other repository features.
    pub repository: Option<RepositorySettings>,

    /// Organization-wide webhook configurations.
    /// Typically fixed to ensure security and compliance monitoring.
    pub webhooks: Option<Vec<WebhookConfig>>,
}

impl GlobalDefaultsEnhanced {
    /// Creates a new empty `GlobalDefaultsEnhanced` configuration.
    ///
    /// All fields are initialized to `None`, allowing for gradual configuration
    /// building. Use the various setter methods or direct field assignment to
    /// populate the configuration.
    ///
    /// # Returns
    ///
    /// A new `GlobalDefaultsEnhanced` instance with all fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::GlobalDefaultsEnhanced;
    ///
    /// let defaults = GlobalDefaultsEnhanced::new();
    /// assert!(defaults.actions.is_none());
    /// assert!(defaults.branch_protection.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            actions: None,
            branch_protection: None,
            custom_properties: None,
            environments: None,
            github_apps: None,
            pull_requests: None,
            push: None,
            repository: None,
            webhooks: None,
        }
    }
}

impl Default for GlobalDefaultsEnhanced {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository type configuration for type-specific settings.
///
/// This structure represents configuration settings that apply to repositories of a specific type
/// (e.g., documentation, actions, library, service). Repository types provide a way to group
/// repositories with similar requirements and apply consistent settings across them.
///
/// Repository type configurations are stored in the metadata repository under `types/{type}/config.toml`
/// and are loaded based on the repository type specified during repository creation or through
/// GitHub custom properties.
///
/// The configuration supports all the same settings as global defaults and team configurations,
/// allowing type-specific customization of repository behavior, security policies, and automation.
///
/// # Repository Type Hierarchy
///
/// Repository types fit into the configuration hierarchy as:
/// 1. **Template** - Highest precedence (can override repository type settings)
/// 2. **Team** - Second precedence
/// 3. **Repository Type** - Third precedence (this structure)
/// 4. **Global** - Lowest precedence (baseline defaults)
///
/// # Examples
///
/// Creating a repository type configuration for documentation repositories:
///
/// ```rust
/// use config_manager::organization::{RepositoryTypeConfig, RepositorySettings, LabelConfig};
/// use config_manager::organization::OverridableValue;
///
/// let mut doc_config = RepositoryTypeConfig::new();
///
/// // Documentation repositories typically disable wiki (docs are in the repo)
/// let mut repo_settings = RepositorySettings::new();
/// repo_settings.wiki = Some(OverridableValue::fixed(false));
/// repo_settings.issues = Some(OverridableValue::overridable(true));
/// doc_config.repository = Some(repo_settings);
///
/// // Add documentation-specific labels
/// let doc_label = LabelConfig::new(
///     "documentation".to_string(),
///     Some("Improvements or additions to documentation".to_string()),
///     "0052cc".to_string()
/// );
/// doc_config.labels = Some(vec![doc_label]);
/// ```
///
/// Loading from TOML configuration file:
///
/// ```toml
/// # types/documentation/config.toml
///
/// [repository]
/// wiki = { value = false, override_allowed = true }
/// issues = { value = true, override_allowed = true }
///
/// [[labels]]
/// name = "documentation"
/// color = "0052cc"
/// description = "Improvements or additions to documentation"
/// ```
///
/// # Security Considerations
///
/// Repository type configurations should be carefully designed to ensure they don't conflict
/// with security policies. Templates can override repository type settings, so sensitive
/// security configurations should be marked as non-overridable where appropriate.
///
/// # Error Conditions
///
/// - **Invalid Configuration**: If the repository type configuration contains invalid values
/// - **Override Conflicts**: If a template tries to override a non-overridable setting
/// - **Missing Dependencies**: If the configuration references undefined custom properties or environments
/// - **Serialization Errors**: If the configuration cannot be loaded from TOML/JSON files
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RepositoryTypeConfig {
    /// Branch protection settings for repositories of this type.
    ///
    /// These settings control branch protection rules, required status checks, and merge policies.
    /// Common patterns include stricter protection for library repositories and more flexible
    /// settings for documentation repositories.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, BranchProtectionSettings};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// config.branch_protection = Some(BranchProtectionSettings::new());
    /// ```
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Custom properties specific to this repository type.
    ///
    /// GitHub custom properties allow organizations to attach metadata to repositories.
    /// Repository types can define standard properties that should be applied to all
    /// repositories of that type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, CustomProperty};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let property = CustomProperty::new(
    ///     "type".to_string(),
    ///     "documentation".to_string()
    /// );
    /// config.custom_properties = Some(vec![property]);
    /// ```
    pub custom_properties: Option<Vec<CustomProperty>>,

    /// Deployment environments for repositories of this type.
    ///
    /// Different repository types may have different deployment patterns. For example,
    /// library repositories might have staging and production environments, while
    /// documentation repositories might only need a production environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, EnvironmentConfig};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let prod_env = EnvironmentConfig::new("production".to_string(), None, None, None);
    /// config.environments = Some(vec![prod_env]);
    /// ```
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// GitHub Apps that should be installed for repositories of this type.
    ///
    /// Repository types can specify which GitHub Apps should be automatically installed
    /// to provide type-specific functionality (e.g., security scanning for libraries,
    /// documentation generators for documentation repositories).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, GitHubAppConfig};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// config.github_apps = Some(vec![
    ///     GitHubAppConfig::new("dependabot".to_string())
    /// ]);
    /// ```
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Labels that should be created for repositories of this type.
    ///
    /// Repository types typically need specific labels for their workflows. For example,
    /// documentation repositories might need "typo" and "documentation" labels, while
    /// action repositories need "breaking-change" labels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, LabelConfig};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let label = LabelConfig::new(
    ///     "enhancement".to_string(),
    ///     Some("New feature or request".to_string()),
    ///     "a2eeef".to_string()
    /// );
    /// config.labels = Some(vec![label]);
    /// ```
    pub labels: Option<Vec<LabelConfig>>,

    /// Pull request settings for repositories of this type.
    ///
    /// Different repository types may have different pull request requirements. For example,
    /// marketplace actions might require more reviewers than internal documentation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, PullRequestSettings};
    /// use config_manager::organization::OverridableValue;
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let mut pr_settings = PullRequestSettings::new();
    /// pr_settings.allow_squash_merge = Some(OverridableValue::fixed(true));
    /// config.pull_requests = Some(pr_settings);
    /// ```
    pub pull_requests: Option<PullRequestSettings>,

    /// Repository-level settings for repositories of this type.
    ///
    /// These settings control repository features like issues, wiki, projects, and security features.
    /// Repository types can provide sensible defaults based on the repository's purpose.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, RepositorySettings};
    /// use config_manager::organization::OverridableValue;
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let mut repo_settings = RepositorySettings::new();
    /// repo_settings.issues = Some(OverridableValue::overridable(true));
    /// repo_settings.wiki = Some(OverridableValue::fixed(false));
    /// config.repository = Some(repo_settings);
    /// ```
    pub repository: Option<RepositorySettings>,

    /// Webhooks that should be configured for repositories of this type.
    ///
    /// Repository types can define standard webhooks for integration with external systems.
    /// For example, library repositories might need webhooks for package publishing,
    /// while documentation repositories might need webhooks for site deployment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, WebhookConfig, WebhookEvent};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// let webhook = WebhookConfig::new(
    ///     "https://example.com/webhook".to_string(),
    ///     vec![WebhookEvent::Push, WebhookEvent::PullRequest],
    ///     true,
    ///     None
    /// );
    /// config.webhooks = Some(vec![webhook]);
    /// ```
    pub webhooks: Option<Vec<WebhookConfig>>,
}

impl RepositoryTypeConfig {
    /// Creates a new empty `RepositoryTypeConfig`.
    ///
    /// All configuration fields are initialized to `None`, indicating that this repository type
    /// does not override any settings. Individual fields can be set as needed to customize
    /// the behavior for repositories of this type.
    ///
    /// # Returns
    ///
    /// A new `RepositoryTypeConfig` instance with all fields set to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::RepositoryTypeConfig;
    ///
    /// let config = RepositoryTypeConfig::new();
    /// assert!(config.repository.is_none());
    /// assert!(config.labels.is_none());
    /// assert!(config.webhooks.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            branch_protection: None,
            custom_properties: None,
            environments: None,
            github_apps: None,
            labels: None,
            pull_requests: None,
            repository: None,
            webhooks: None,
        }
    }

    /// Validates the repository type configuration for consistency and completeness.
    ///
    /// This method performs comprehensive validation of the configuration, checking for:
    /// - Invalid or conflicting settings
    /// - Missing required dependencies
    /// - Malformed data (e.g., invalid colors, malformed URLs)
    /// - Logical inconsistencies between related settings
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the configuration is valid
    /// - `Err(ConfigurationError)` if validation fails with details about the issues
    ///
    /// # Errors
    ///
    /// - `ConfigurationError::InvalidConfiguration`: If settings are invalid or inconsistent
    /// - `ConfigurationError::MissingDependency`: If referenced resources don't exist
    /// - `ConfigurationError::ValidationError`: If data format validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, LabelConfig};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    ///
    /// // Valid configuration
    /// let label = LabelConfig::new("bug".to_string(), None, "d73a4a".to_string());
    /// config.labels = Some(vec![label]);
    /// assert!(config.validate().is_ok());
    ///
    /// // Invalid configuration (bad color format)
    /// let bad_label = LabelConfig::new("bad".to_string(), None, "invalid-color".to_string());
    /// config.labels = Some(vec![bad_label]);
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        // Validate labels if present - basic validation for now
        if let Some(labels) = &self.labels {
            for label in labels {
                if label.name.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "label.name".to_string(),
                        value: "".to_string(),
                        reason: "Label name cannot be empty".to_string(),
                    });
                }
                if label.color.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "label.color".to_string(),
                        value: "".to_string(),
                        reason: "Label color cannot be empty".to_string(),
                    });
                }
                // Basic hex color validation (6 characters)
                if !label.color.chars().all(|c| c.is_ascii_hexdigit()) || label.color.len() != 6 {
                    return Err(ConfigurationError::InvalidValue {
                        field: "label.color".to_string(),
                        value: label.color.clone(),
                        reason: "Color must be 6-character hex string".to_string(),
                    });
                }
            }
        }

        // Validate webhooks if present - basic validation for now
        if let Some(webhooks) = &self.webhooks {
            for webhook in webhooks {
                if webhook.url.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "webhook.url".to_string(),
                        value: "".to_string(),
                        reason: "Webhook URL cannot be empty".to_string(),
                    });
                }
                if !webhook.url.starts_with("https://") {
                    return Err(ConfigurationError::InvalidValue {
                        field: "webhook.url".to_string(),
                        value: webhook.url.clone(),
                        reason: "Webhook URL must use HTTPS".to_string(),
                    });
                }
            }
        }

        // Validate environments if present - basic validation for now
        if let Some(environments) = &self.environments {
            for env in environments {
                if env.name.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "environment.name".to_string(),
                        value: "".to_string(),
                        reason: "Environment name cannot be empty".to_string(),
                    });
                }
            }
        }

        // Validate GitHub apps if present - basic validation for now
        if let Some(github_apps) = &self.github_apps {
            for app in github_apps {
                if app.app_slug.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "github_app.app_slug".to_string(),
                        value: "".to_string(),
                        reason: "GitHub App name cannot be empty".to_string(),
                    });
                }
            }
        }

        // Validate custom properties if present - basic validation for now
        if let Some(custom_properties) = &self.custom_properties {
            for property in custom_properties {
                if property.property_name.is_empty() {
                    return Err(ConfigurationError::InvalidValue {
                        field: "custom_property.property_name".to_string(),
                        value: "".to_string(),
                        reason: "Custom property name cannot be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Checks if this repository type configuration has any settings defined.
    ///
    /// This method returns `true` if any configuration field is set (not `None`),
    /// indicating that this repository type provides specific settings that differ
    /// from the defaults.
    ///
    /// # Returns
    ///
    /// `true` if any configuration field is defined, `false` if all fields are `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, RepositorySettings};
    ///
    /// let empty_config = RepositoryTypeConfig::new();
    /// assert!(!empty_config.has_settings());
    ///
    /// let mut config_with_settings = RepositoryTypeConfig::new();
    /// config_with_settings.repository = Some(RepositorySettings::new());
    /// assert!(config_with_settings.has_settings());
    /// ```
    pub fn has_settings(&self) -> bool {
        self.branch_protection.is_some()
            || self.custom_properties.is_some()
            || self.environments.is_some()
            || self.github_apps.is_some()
            || self.labels.is_some()
            || self.pull_requests.is_some()
            || self.repository.is_some()
            || self.webhooks.is_some()
    }

    /// Counts the total number of additive items (labels, webhooks, etc.) in this configuration.
    ///
    /// This method provides a count of items that are additively merged during configuration
    /// resolution. This is useful for understanding the scope of configuration and for
    /// performance optimization in merging operations.
    ///
    /// # Returns
    ///
    /// The total count of additive configuration items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeConfig, LabelConfig, WebhookConfig, WebhookEvent};
    ///
    /// let mut config = RepositoryTypeConfig::new();
    /// assert_eq!(config.count_additive_items(), 0);
    ///
    /// // Add some labels and webhooks
    /// config.labels = Some(vec![
    ///     LabelConfig::new("bug".to_string(), None, "d73a4a".to_string()),
    ///     LabelConfig::new("feature".to_string(), None, "a2eeef".to_string()),
    /// ]);
    /// config.webhooks = Some(vec![
    ///     WebhookConfig::new("https://example.com/hook".to_string(), vec![WebhookEvent::Push], true, None)
    /// ]);
    ///
    /// assert_eq!(config.count_additive_items(), 3); // 2 labels + 1 webhook
    /// ```
    pub fn count_additive_items(&self) -> usize {
        let mut count = 0;

        if let Some(labels) = &self.labels {
            count += labels.len();
        }

        if let Some(webhooks) = &self.webhooks {
            count += webhooks.len();
        }

        if let Some(environments) = &self.environments {
            count += environments.len();
        }

        if let Some(github_apps) = &self.github_apps {
            count += github_apps.len();
        }

        if let Some(custom_properties) = &self.custom_properties {
            count += custom_properties.len();
        }

        count
    }
}

impl Default for RepositoryTypeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository type specification policy for templates.
///
/// This enum defines how templates can specify repository types and whether users
/// can override the template's repository type choice during repository creation.
/// This provides flexibility while allowing templates to enforce specific requirements.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::RepositoryTypePolicy;
///
/// let policy = RepositoryTypePolicy::Fixed;
/// assert_eq!(format!("{:?}", policy), "Fixed");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryTypePolicy {
    /// User cannot override the repository type - template requirement is enforced.
    Fixed,
    /// User can override the repository type during repository creation.
    Preferable,
}

/// Repository type specification for template configurations.
///
/// This structure allows templates to specify what repository type they create
/// and whether users can override this choice. This enables templates to either
/// enforce specific repository types for compliance or suggest preferred types
/// while allowing flexibility.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{RepositoryTypeSpec, RepositoryTypePolicy};
///
/// // Template that enforces a specific repository type
/// let fixed_spec = RepositoryTypeSpec::new("microservice".to_string(), RepositoryTypePolicy::Fixed);
/// assert_eq!(fixed_spec.repository_type(), "microservice");
/// assert_eq!(fixed_spec.policy(), &RepositoryTypePolicy::Fixed);
///
/// // Template that prefers a type but allows override
/// let flexible_spec = RepositoryTypeSpec::new("library".to_string(), RepositoryTypePolicy::Preferable);
/// assert_eq!(flexible_spec.policy(), &RepositoryTypePolicy::Preferable);
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryTypeSpec {
    /// The repository type this template creates or prefers.
    repository_type: String,
    /// Whether users can override this repository type choice.
    policy: RepositoryTypePolicy,
}

impl RepositoryTypeSpec {
    /// Creates a new repository type specification.
    ///
    /// # Arguments
    ///
    /// * `repository_type` - The repository type name (e.g., "microservice", "library")
    /// * `policy` - Whether this type can be overridden by users
    ///
    /// # Returns
    ///
    /// A new `RepositoryTypeSpec` with the specified type and policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeSpec, RepositoryTypePolicy};
    ///
    /// let spec = RepositoryTypeSpec::new("service".to_string(), RepositoryTypePolicy::Fixed);
    /// assert_eq!(spec.repository_type(), "service");
    /// ```
    pub fn new(repository_type: String, policy: RepositoryTypePolicy) -> Self {
        Self {
            repository_type,
            policy,
        }
    }

    /// Gets the repository type name.
    ///
    /// # Returns
    ///
    /// A reference to the repository type string.
    pub fn repository_type(&self) -> &str {
        &self.repository_type
    }

    /// Gets the override policy for this repository type specification.
    ///
    /// # Returns
    ///
    /// A reference to the `RepositoryTypePolicy`.
    pub fn policy(&self) -> &RepositoryTypePolicy {
        &self.policy
    }

    /// Checks if users can override this repository type.
    ///
    /// # Returns
    ///
    /// `true` if the repository type can be overridden, `false` if it's fixed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{RepositoryTypeSpec, RepositoryTypePolicy};
    ///
    /// let fixed = RepositoryTypeSpec::new("service".to_string(), RepositoryTypePolicy::Fixed);
    /// let flexible = RepositoryTypeSpec::new("library".to_string(), RepositoryTypePolicy::Preferable);
    ///
    /// assert!(!fixed.can_override());
    /// assert!(flexible.can_override());
    /// ```
    pub fn can_override(&self) -> bool {
        matches!(self.policy, RepositoryTypePolicy::Preferable)
    }
}

/// Template metadata containing authoring and identification information.
///
/// This structure provides essential information about the template including
/// its name, description, author, and categorization tags. This metadata helps
/// users understand the purpose and scope of the template.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::TemplateMetadata;
///
/// let metadata = TemplateMetadata::new(
///     "rust-microservice".to_string(),
///     "Production-ready Rust microservice template".to_string(),
///     "Platform Team".to_string(),
///     vec!["rust".to_string(), "microservice".to_string()],
/// );
///
/// assert_eq!(metadata.name(), "rust-microservice");
/// assert_eq!(metadata.tags().len(), 2);
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TemplateMetadata {
    /// Unique name identifying this template.
    name: String,
    /// Human-readable description of the template's purpose.
    description: String,
    /// Author or team responsible for this template.
    author: String,
    /// Categorization tags for discovery and organization.
    tags: Vec<String>,
}

impl TemplateMetadata {
    /// Creates new template metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique template identifier
    /// * `description` - Human-readable template description
    /// * `author` - Template author or responsible team
    /// * `tags` - Categorization tags for discovery
    ///
    /// # Returns
    ///
    /// A new `TemplateMetadata` instance with the provided information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::TemplateMetadata;
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "web-app".to_string(),
    ///     "React web application template".to_string(),
    ///     "Frontend Team".to_string(),
    ///     vec!["react".to_string(), "web".to_string()],
    /// );
    ///
    /// assert_eq!(metadata.author(), "Frontend Team");
    /// ```
    pub fn new(name: String, description: String, author: String, tags: Vec<String>) -> Self {
        Self {
            name,
            description,
            author,
            tags,
        }
    }

    /// Gets the template name.
    ///
    /// # Returns
    ///
    /// A reference to the template name string.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the template description.
    ///
    /// # Returns
    ///
    /// A reference to the template description string.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Gets the template author.
    ///
    /// # Returns
    ///
    /// A reference to the author string.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Gets the template tags.
    ///
    /// # Returns
    ///
    /// A reference to the vector of tag strings.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Checks if the template has a specific tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to search for
    ///
    /// # Returns
    ///
    /// `true` if the template has the specified tag, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::TemplateMetadata;
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "api".to_string(),
    ///     "REST API template".to_string(),
    ///     "Backend Team".to_string(),
    ///     vec!["api".to_string(), "rest".to_string()],
    /// );
    ///
    /// assert!(metadata.has_tag("api"));
    /// assert!(!metadata.has_tag("frontend"));
    /// ```
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Template variable configuration with validation and metadata.
///
/// This structure defines properties and validation rules for template variables
/// used during repository creation. It extends the basic variable configuration
/// concept to support template-specific requirements and defaults.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::TemplateVariable;
///
/// // Required variable with validation
/// let service_name = TemplateVariable::new(
///     "Name of the microservice".to_string(),
///     Some("user-service".to_string()),
///     None,
///     Some(true),
/// );
///
/// assert!(service_name.required().unwrap_or(false));
/// assert_eq!(service_name.example().unwrap(), "user-service");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TemplateVariable {
    /// Human-readable description of what this variable controls.
    description: String,
    /// Optional example value to guide users.
    example: Option<String>,
    /// Optional default value if not provided by user.
    default: Option<String>,
    /// Whether this variable must be provided (overrides template engine defaults).
    required: Option<bool>,
}

impl TemplateVariable {
    /// Creates a new template variable configuration.
    ///
    /// # Arguments
    ///
    /// * `description` - Human-readable description of the variable's purpose
    /// * `example` - Optional example value for user guidance
    /// * `default` - Optional default value if not provided
    /// * `required` - Whether this variable is required (None uses template engine default)
    ///
    /// # Returns
    ///
    /// A new `TemplateVariable` with the specified configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::TemplateVariable;
    ///
    /// let var = TemplateVariable::new(
    ///     "Service port number".to_string(),
    ///     Some("8080".to_string()),
    ///     Some("8000".to_string()),
    ///     Some(false),
    /// );
    ///
    /// assert_eq!(var.default().unwrap(), "8000");
    /// ```
    pub fn new(
        description: String,
        example: Option<String>,
        default: Option<String>,
        required: Option<bool>,
    ) -> Self {
        Self {
            description,
            example,
            default,
            required,
        }
    }

    /// Gets the variable description.
    ///
    /// # Returns
    ///
    /// A reference to the description string.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Gets the example value if provided.
    ///
    /// # Returns
    ///
    /// An optional reference to the example string.
    pub fn example(&self) -> Option<&str> {
        self.example.as_deref()
    }

    /// Gets the default value if provided.
    ///
    /// # Returns
    ///
    /// An optional reference to the default string.
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    /// Gets the required flag if specified.
    ///
    /// # Returns
    ///
    /// An optional boolean indicating if the variable is required.
    pub fn required(&self) -> Option<bool> {
        self.required
    }

    /// Checks if this variable has a default value.
    ///
    /// # Returns
    ///
    /// `true` if a default value is provided, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::TemplateVariable;
    ///
    /// let with_default = TemplateVariable::new(
    ///     "Port".to_string(),
    ///     None,
    ///     Some("8080".to_string()),
    ///     None,
    /// );
    ///
    /// let without_default = TemplateVariable::new(
    ///     "Name".to_string(),
    ///     None,
    ///     None,
    ///     Some(true),
    /// );
    ///
    /// assert!(with_default.has_default());
    /// assert!(!without_default.has_default());
    /// ```
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }
}

/// Template-specific configuration for repository creation and management.
///
/// This structure defines all template-specific requirements and settings that
/// are applied during repository creation. Templates have the highest precedence
/// in the configuration hierarchy and can specify whether their settings can be
/// overridden by lower-level configurations.
///
/// The template configuration extends beyond basic repository settings to include
/// metadata, variable definitions, repository type specifications, and environment
/// configurations specific to the template's intended use case.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::{TemplateConfig, TemplateMetadata, RepositoryTypeSpec, RepositoryTypePolicy};
/// use std::collections::HashMap;
///
/// let metadata = TemplateMetadata::new(
///     "rust-service".to_string(),
///     "Rust microservice template".to_string(),
///     "Platform Team".to_string(),
///     vec!["rust".to_string(), "microservice".to_string()],
/// );
///
/// let mut config = TemplateConfig::new(metadata);
/// config.set_repository_type(Some(RepositoryTypeSpec::new(
///     "microservice".to_string(),
///     RepositoryTypePolicy::Fixed,
/// )));
///
/// assert_eq!(config.template().name(), "rust-service");
/// assert!(config.repository_type().is_some());
/// ```
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TemplateConfig {
    /// Template identification and metadata information.
    template: TemplateMetadata,
    /// Optional repository type specification and override policy.
    repository_type: Option<RepositoryTypeSpec>,
    /// Template-specific repository settings (wiki, issues, security, etc.).
    repository: Option<RepositorySettings>,
    /// Template-specific pull request settings and policies.
    pull_requests: Option<PullRequestSettings>,
    /// Template-specific branch protection settings.
    branch_protection: Option<BranchProtectionSettings>,
    /// Labels that should be created in repositories using this template.
    labels: Option<Vec<LabelConfig>>,
    /// Webhooks that should be configured for repositories using this template.
    webhooks: Option<Vec<WebhookConfig>>,
    /// GitHub Apps that should be installed for repositories using this template.
    github_apps: Option<Vec<GitHubAppConfig>>,
    /// Environment configurations for deployment and automation.
    environments: Option<Vec<EnvironmentConfig>>,
    /// Template variable definitions with validation and defaults.
    variables: Option<HashMap<String, TemplateVariable>>,
}

impl TemplateConfig {
    /// Creates a new template configuration with the specified metadata.
    ///
    /// All optional configuration sections are initialized to None and can be
    /// set using the provided setter methods.
    ///
    /// # Arguments
    ///
    /// * `template` - Template metadata including name, description, author, and tags
    ///
    /// # Returns
    ///
    /// A new `TemplateConfig` with the provided metadata and empty optional sections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{TemplateConfig, TemplateMetadata};
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "web-app".to_string(),
    ///     "React web application".to_string(),
    ///     "Frontend Team".to_string(),
    ///     vec!["react".to_string()],
    /// );
    ///
    /// let config = TemplateConfig::new(metadata);
    /// assert_eq!(config.template().name(), "web-app");
    /// assert!(config.repository_type().is_none());
    /// ```
    pub fn new(template: TemplateMetadata) -> Self {
        Self {
            template,
            repository_type: None,
            repository: None,
            pull_requests: None,
            branch_protection: None,
            labels: None,
            webhooks: None,
            github_apps: None,
            environments: None,
            variables: None,
        }
    }

    /// Gets the template metadata.
    ///
    /// # Returns
    ///
    /// A reference to the `TemplateMetadata`.
    pub fn template(&self) -> &TemplateMetadata {
        &self.template
    }

    /// Gets the repository type specification if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `RepositoryTypeSpec`.
    pub fn repository_type(&self) -> Option<&RepositoryTypeSpec> {
        self.repository_type.as_ref()
    }

    /// Sets the repository type specification.
    ///
    /// # Arguments
    ///
    /// * `repository_type` - Optional repository type specification
    pub fn set_repository_type(&mut self, repository_type: Option<RepositoryTypeSpec>) {
        self.repository_type = repository_type;
    }

    /// Gets the repository settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `RepositorySettings`.
    pub fn repository(&self) -> Option<&RepositorySettings> {
        self.repository.as_ref()
    }

    /// Sets the repository settings.
    ///
    /// # Arguments
    ///
    /// * `repository` - Optional repository settings
    pub fn set_repository(&mut self, repository: Option<RepositorySettings>) {
        self.repository = repository;
    }

    /// Gets the pull request settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `PullRequestSettings`.
    pub fn pull_requests(&self) -> Option<&PullRequestSettings> {
        self.pull_requests.as_ref()
    }

    /// Sets the pull request settings.
    ///
    /// # Arguments
    ///
    /// * `pull_requests` - Optional pull request settings
    pub fn set_pull_requests(&mut self, pull_requests: Option<PullRequestSettings>) {
        self.pull_requests = pull_requests;
    }

    /// Gets the branch protection settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `BranchProtectionSettings`.
    pub fn branch_protection(&self) -> Option<&BranchProtectionSettings> {
        self.branch_protection.as_ref()
    }

    /// Sets the branch protection settings.
    ///
    /// # Arguments
    ///
    /// * `branch_protection` - Optional branch protection settings
    pub fn set_branch_protection(&mut self, branch_protection: Option<BranchProtectionSettings>) {
        self.branch_protection = branch_protection;
    }

    /// Gets the labels if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `LabelConfig`.
    pub fn labels(&self) -> Option<&[LabelConfig]> {
        self.labels.as_deref()
    }

    /// Sets the labels.
    ///
    /// # Arguments
    ///
    /// * `labels` - Optional vector of label configurations
    pub fn set_labels(&mut self, labels: Option<Vec<LabelConfig>>) {
        self.labels = labels;
    }

    /// Gets the webhooks if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `WebhookConfig`.
    pub fn webhooks(&self) -> Option<&[WebhookConfig]> {
        self.webhooks.as_deref()
    }

    /// Sets the webhooks.
    ///
    /// # Arguments
    ///
    /// * `webhooks` - Optional vector of webhook configurations
    pub fn set_webhooks(&mut self, webhooks: Option<Vec<WebhookConfig>>) {
        self.webhooks = webhooks;
    }

    /// Gets the GitHub Apps if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `GitHubAppConfig`.
    pub fn github_apps(&self) -> Option<&[GitHubAppConfig]> {
        self.github_apps.as_deref()
    }

    /// Sets the GitHub Apps.
    ///
    /// # Arguments
    ///
    /// * `github_apps` - Optional vector of GitHub App configurations
    pub fn set_github_apps(&mut self, github_apps: Option<Vec<GitHubAppConfig>>) {
        self.github_apps = github_apps;
    }

    /// Gets the environments if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `EnvironmentConfig`.
    pub fn environments(&self) -> Option<&[EnvironmentConfig]> {
        self.environments.as_deref()
    }

    /// Sets the environments.
    ///
    /// # Arguments
    ///
    /// * `environments` - Optional vector of environment configurations
    pub fn set_environments(&mut self, environments: Option<Vec<EnvironmentConfig>>) {
        self.environments = environments;
    }

    /// Gets the template variables if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the HashMap of variable name to `TemplateVariable`.
    pub fn variables(&self) -> Option<&HashMap<String, TemplateVariable>> {
        self.variables.as_ref()
    }

    /// Sets the template variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - Optional HashMap of variable configurations
    pub fn set_variables(&mut self, variables: Option<HashMap<String, TemplateVariable>>) {
        self.variables = variables;
    }

    /// Adds a single template variable.
    ///
    /// If the variables HashMap doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `variable` - Variable configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{TemplateConfig, TemplateMetadata, TemplateVariable};
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "test".to_string(),
    ///     "Test template".to_string(),
    ///     "Test Team".to_string(),
    ///     vec![],
    /// );
    ///
    /// let mut config = TemplateConfig::new(metadata);
    /// let variable = TemplateVariable::new(
    ///     "Service name".to_string(),
    ///     Some("example-service".to_string()),
    ///     None,
    ///     Some(true),
    /// );
    ///
    /// config.add_variable("service_name".to_string(), variable);
    /// assert!(config.variables().is_some());
    /// assert!(config.variables().unwrap().contains_key("service_name"));
    /// ```
    pub fn add_variable(&mut self, name: String, variable: TemplateVariable) {
        if self.variables.is_none() {
            self.variables = Some(HashMap::new());
        }
        if let Some(ref mut vars) = self.variables {
            vars.insert(name, variable);
        }
    }

    /// Checks if this template specifies a repository type.
    ///
    /// # Returns
    ///
    /// `true` if a repository type is specified, `false` otherwise.
    pub fn has_repository_type(&self) -> bool {
        self.repository_type.is_some()
    }

    /// Checks if the template's repository type can be overridden.
    ///
    /// # Returns
    ///
    /// `true` if the repository type can be overridden or no type is specified,
    /// `false` if the type is fixed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{TemplateConfig, TemplateMetadata, RepositoryTypeSpec, RepositoryTypePolicy};
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "test".to_string(),
    ///     "Test".to_string(),
    ///     "Test Team".to_string(),
    ///     vec![],
    /// );
    ///
    /// let mut config = TemplateConfig::new(metadata);
    ///
    /// // No repository type specified - can override
    /// assert!(config.can_override_repository_type());
    ///
    /// // Fixed repository type - cannot override
    /// config.set_repository_type(Some(RepositoryTypeSpec::new(
    ///     "service".to_string(),
    ///     RepositoryTypePolicy::Fixed,
    /// )));
    /// assert!(!config.can_override_repository_type());
    ///
    /// // Preferable repository type - can override
    /// config.set_repository_type(Some(RepositoryTypeSpec::new(
    ///     "library".to_string(),
    ///     RepositoryTypePolicy::Preferable,
    /// )));
    /// assert!(config.can_override_repository_type());
    /// ```
    pub fn can_override_repository_type(&self) -> bool {
        match &self.repository_type {
            Some(spec) => spec.can_override(),
            None => true, // No restriction if not specified
        }
    }

    /// Counts the total number of additive items (labels, webhooks, etc.) in this configuration.
    ///
    /// This method provides a count of items that are additively merged during configuration
    /// resolution. This is useful for understanding the scope of configuration and for
    /// performance optimization in merging operations.
    ///
    /// # Returns
    ///
    /// The total count of additive configuration items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{TemplateConfig, TemplateMetadata, LabelConfig, WebhookConfig, WebhookEvent};
    ///
    /// let metadata = TemplateMetadata::new(
    ///     "test".to_string(),
    ///     "Test".to_string(),
    ///     "Test Team".to_string(),
    ///     vec![],
    /// );
    ///
    /// let mut config = TemplateConfig::new(metadata);
    /// assert_eq!(config.count_additive_items(), 0);
    ///
    /// // Add some labels and webhooks
    /// config.set_labels(Some(vec![
    ///     LabelConfig::new("bug".to_string(), None, "d73a4a".to_string()),
    ///     LabelConfig::new("feature".to_string(), None, "a2eeef".to_string()),
    /// ]));
    /// config.set_webhooks(Some(vec![
    ///     WebhookConfig::new("https://example.com/hook".to_string(), vec![WebhookEvent::Push], true, None)
    /// ]));
    ///
    /// assert_eq!(config.count_additive_items(), 3); // 2 labels + 1 webhook
    /// ```
    pub fn count_additive_items(&self) -> usize {
        let mut count = 0;

        if let Some(labels) = &self.labels {
            count += labels.len();
        }

        if let Some(webhooks) = &self.webhooks {
            count += webhooks.len();
        }

        if let Some(environments) = &self.environments {
            count += environments.len();
        }

        if let Some(github_apps) = &self.github_apps {
            count += github_apps.len();
        }

        if let Some(variables) = &self.variables {
            count += variables.len();
        }

        count
    }
}

/// Source of a configuration setting in the hierarchical merge process.
///
/// This enum tracks where each configuration setting originates from in the
/// four-level hierarchy, enabling audit trails and debugging of configuration
/// resolution. It's essential for understanding why certain settings were
/// applied and for troubleshooting configuration conflicts.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::ConfigurationSource;
///
/// let source = ConfigurationSource::Template;
/// assert_eq!(format!("{:?}", source), "Template");
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigurationSource {
    /// Setting originated from global defaults.
    Global,
    /// Setting originated from repository type configuration.
    RepositoryType,
    /// Setting originated from team configuration.
    Team,
    /// Setting originated from template configuration (highest precedence).
    Template,
}

impl ConfigurationSource {
    /// Gets the precedence level of this configuration source.
    ///
    /// Higher numbers indicate higher precedence in the configuration hierarchy.
    /// This is used for determining which configuration takes priority during merging.
    ///
    /// # Returns
    ///
    /// The precedence level as an integer:
    /// - Global: 1 (lowest precedence)
    /// - RepositoryType: 2
    /// - Team: 3
    /// - Template: 4 (highest precedence)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::ConfigurationSource;
    ///
    /// assert_eq!(ConfigurationSource::Global.precedence(), 1);
    /// assert_eq!(ConfigurationSource::Template.precedence(), 4);
    /// assert!(ConfigurationSource::Template.precedence() > ConfigurationSource::Team.precedence());
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
    /// use config_manager::organization::ConfigurationSource;
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
/// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
    /// use config_manager::organization::ConfigurationSourceTrace;
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
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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

    /// Gets all tracked field paths and their sources.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing all field path to source mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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

    /// Checks if the trace is empty (no sources recorded).
    ///
    /// # Returns
    ///
    /// `true` if no sources are recorded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
    /// use config_manager::organization::{ConfigurationSourceTrace, ConfigurationSource};
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
/// use config_manager::organization::{MergedConfiguration, ConfigurationSourceTrace};
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
    /// Repository labels to be created or maintained.
    labels: HashMap<String, LabelConfig>,
    /// Webhooks to be configured on the repository.
    webhooks: Vec<WebhookConfig>,
    /// GitHub Apps to be installed or configured for the repository.
    github_apps: Vec<GitHubAppConfig>,
    /// Environment configurations for deployment and automation.
    environments: Vec<EnvironmentConfig>,
    /// Audit trail of configuration sources for transparency and debugging.
    source_trace: ConfigurationSourceTrace,
}

impl MergedConfiguration {
    /// Creates a new empty merged configuration.
    ///
    /// All configuration sections are initialized to their default values,
    /// and the source trace is empty. This serves as the base for the
    /// hierarchical merging process.
    ///
    /// # Returns
    ///
    /// A new `MergedConfiguration` with default values and empty collections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::MergedConfiguration;
    ///
    /// let config = MergedConfiguration::new();
    /// assert!(config.labels().is_empty());
    /// assert!(config.webhooks().is_empty());
    /// assert!(config.source_trace().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            repository_settings: RepositorySettings::default(),
            pull_request_settings: PullRequestSettings::default(),
            branch_protection: BranchProtectionSettings::default(),
            labels: HashMap::new(),
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

    /// Gets a mutable reference to the repository settings.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `RepositorySettings`.
    pub fn repository_settings_mut(&mut self) -> &mut RepositorySettings {
        &mut self.repository_settings
    }

    /// Gets the pull request settings.
    ///
    /// # Returns
    ///
    /// A reference to the `PullRequestSettings`.
    pub fn pull_request_settings(&self) -> &PullRequestSettings {
        &self.pull_request_settings
    }

    /// Gets a mutable reference to the pull request settings.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `PullRequestSettings`.
    pub fn pull_request_settings_mut(&mut self) -> &mut PullRequestSettings {
        &mut self.pull_request_settings
    }

    /// Gets the branch protection settings.
    ///
    /// # Returns
    ///
    /// A reference to the `BranchProtectionSettings`.
    pub fn branch_protection(&self) -> &BranchProtectionSettings {
        &self.branch_protection
    }

    /// Gets a mutable reference to the branch protection settings.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `BranchProtectionSettings`.
    pub fn branch_protection_mut(&mut self) -> &mut BranchProtectionSettings {
        &mut self.branch_protection
    }

    /// Gets the repository labels.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap of label names to `LabelConfig`.
    pub fn labels(&self) -> &HashMap<String, LabelConfig> {
        &self.labels
    }

    /// Gets a mutable reference to the repository labels.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the HashMap of labels.
    pub fn labels_mut(&mut self) -> &mut HashMap<String, LabelConfig> {
        &mut self.labels
    }

    /// Adds a label to the configuration.
    ///
    /// If a label with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `label` - The label configuration to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, LabelConfig};
    ///
    /// let mut config = MergedConfiguration::new();
    /// let label = LabelConfig::new("bug".to_string(), Some("Something isn't working".to_string()), "d73a4a".to_string());
    ///
    /// config.add_label(label);
    /// assert_eq!(config.labels().len(), 1);
    /// assert!(config.labels().contains_key("bug"));
    /// ```
    pub fn add_label(&mut self, label: LabelConfig) {
        self.labels.insert(label.name.clone(), label);
    }

    /// Gets the webhooks.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `WebhookConfig`.
    pub fn webhooks(&self) -> &[WebhookConfig] {
        &self.webhooks
    }

    /// Gets a mutable reference to the webhooks.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the vector of webhooks.
    pub fn webhooks_mut(&mut self) -> &mut Vec<WebhookConfig> {
        &mut self.webhooks
    }

    /// Adds a webhook to the configuration.
    ///
    /// # Arguments
    ///
    /// * `webhook` - The webhook configuration to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, WebhookConfig, WebhookEvent};
    ///
    /// let mut config = MergedConfiguration::new();
    /// let webhook = WebhookConfig::new(
    ///     "https://example.com/hook".to_string(),
    ///     vec![WebhookEvent::Push],
    ///     true,
    ///     None
    /// );
    ///
    /// config.add_webhook(webhook);
    /// assert_eq!(config.webhooks().len(), 1);
    /// ```
    pub fn add_webhook(&mut self, webhook: WebhookConfig) {
        self.webhooks.push(webhook);
    }

    /// Gets the GitHub Apps.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `GitHubAppConfig`.
    pub fn github_apps(&self) -> &[GitHubAppConfig] {
        &self.github_apps
    }

    /// Gets a mutable reference to the GitHub Apps.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the vector of GitHub Apps.
    pub fn github_apps_mut(&mut self) -> &mut Vec<GitHubAppConfig> {
        &mut self.github_apps
    }

    /// Adds a GitHub App to the configuration.
    ///
    /// # Arguments
    ///
    /// * `github_app` - The GitHub App configuration to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, GitHubAppConfig};
    ///
    /// let mut config = MergedConfiguration::new();
    /// let app = GitHubAppConfig::new("dependabot".to_string());
    ///
    /// config.add_github_app(app);
    /// assert_eq!(config.github_apps().len(), 1);
    /// ```
    pub fn add_github_app(&mut self, github_app: GitHubAppConfig) {
        self.github_apps.push(github_app);
    }

    /// Gets the environments.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `EnvironmentConfig`.
    pub fn environments(&self) -> &[EnvironmentConfig] {
        &self.environments
    }

    /// Gets a mutable reference to the environments.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the vector of environments.
    pub fn environments_mut(&mut self) -> &mut Vec<EnvironmentConfig> {
        &mut self.environments
    }

    /// Adds an environment to the configuration.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment configuration to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, EnvironmentConfig};
    ///
    /// let mut config = MergedConfiguration::new();
    /// let env = EnvironmentConfig::new("production".to_string(), None, None, None);
    ///
    /// config.add_environment(env);
    /// assert_eq!(config.environments().len(), 1);
    /// ```
    pub fn add_environment(&mut self, environment: EnvironmentConfig) {
        self.environments.push(environment);
    }

    /// Gets the configuration source trace.
    ///
    /// # Returns
    ///
    /// A reference to the `ConfigurationSourceTrace`.
    pub fn source_trace(&self) -> &ConfigurationSourceTrace {
        &self.source_trace
    }

    /// Gets a mutable reference to the configuration source trace.
    ///
    /// This is typically used during the configuration merging process.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ConfigurationSourceTrace`.
    pub fn source_trace_mut(&mut self) -> &mut ConfigurationSourceTrace {
        &mut self.source_trace
    }

    /// Sets the configuration source trace.
    ///
    /// This replaces the entire source trace with the provided one.
    ///
    /// # Arguments
    ///
    /// * `source_trace` - The new source trace to set
    pub fn set_source_trace(&mut self, source_trace: ConfigurationSourceTrace) {
        self.source_trace = source_trace;
    }

    /// Counts the total number of configuration items.
    ///
    /// This includes labels, webhooks, GitHub Apps, and environments, providing
    /// a quick overview of the scope of the merged configuration.
    ///
    /// # Returns
    ///
    /// The total count of additive configuration items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, LabelConfig, WebhookConfig, WebhookEvent};
    ///
    /// let mut config = MergedConfiguration::new();
    /// assert_eq!(config.count_items(), 0);
    ///
    /// let label = LabelConfig::new("bug".to_string(), None, "d73a4a".to_string());
    /// config.add_label(label);
    ///
    /// let webhook = WebhookConfig::new("https://example.com".to_string(), vec![WebhookEvent::Push], true, None);
    /// config.add_webhook(webhook);
    ///
    /// assert_eq!(config.count_items(), 2);
    /// ```
    pub fn count_items(&self) -> usize {
        self.labels.len() + self.webhooks.len() + self.github_apps.len() + self.environments.len()
    }

    /// Checks if the configuration is empty (has no items configured).
    ///
    /// # Returns
    ///
    /// `true` if no labels, webhooks, GitHub Apps, or environments are configured, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::{MergedConfiguration, LabelConfig};
    ///
    /// let mut config = MergedConfiguration::new();
    /// assert!(config.is_empty());
    ///
    /// let label = LabelConfig::new("bug".to_string(), None, "d73a4a".to_string());
    /// config.add_label(label);
    /// assert!(!config.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count_items() == 0
    }

    /// Validates the merged configuration for consistency and completeness.
    ///
    /// This method performs various validation checks to ensure the merged
    /// configuration is valid and can be applied to a repository successfully.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid, or a `ConfigurationError` if validation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing
    /// - Configuration values are invalid
    /// - Internal consistency checks fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::organization::MergedConfiguration;
    ///
    /// let config = MergedConfiguration::new();
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        // Validate repository settings
        // (This is a placeholder - actual validation logic would be more comprehensive)

        // Validate all labels have valid colors (hex color format)
        for (name, label) in &self.labels {
            if !label.color.chars().all(|c| c.is_ascii_hexdigit()) || label.color.len() != 6 {
                return Err(ConfigurationError::InvalidValue {
                    field: format!("labels.{}.color", name),
                    value: label.color.clone(),
                    reason: "Must be 6-character hex color".to_string(),
                });
            }
        }

        // Validate webhook URLs
        for (i, webhook) in self.webhooks.iter().enumerate() {
            if !webhook.url.starts_with("https://") {
                return Err(ConfigurationError::InvalidValue {
                    field: format!("webhooks[{}].url", i),
                    value: webhook.url.clone(),
                    reason: "Must use HTTPS".to_string(),
                });
            }
        }

        // Validate environment names are not empty
        for (i, env) in self.environments.iter().enumerate() {
            if env.name.trim().is_empty() {
                return Err(ConfigurationError::InvalidValue {
                    field: format!("environments[{}].name", i),
                    value: env.name.clone(),
                    reason: "Environment name cannot be empty".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for MergedConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl MergedConfiguration {
    /// Creates a merged configuration from a hierarchy of configurations.
    ///
    /// Merges configurations in order of precedence:
    /// 1. Template-specific configuration (highest precedence)
    /// 2. Team-specific configuration
    /// 3. Repository type configuration
    /// 4. Global defaults (lowest precedence)
    ///
    /// # Arguments
    ///
    /// * `global` - Global default configuration
    /// * `repo_type` - Repository type-specific configuration (optional)
    /// * `team` - Team-specific configuration (optional)
    /// * `template` - Template-specific configuration (optional)
    ///
    /// # Returns
    ///
    /// A `Result` containing the merged configuration or validation errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use config_manager::organization::{
    ///     MergedConfiguration, GlobalDefaults, RepositoryTypeConfig,
    ///     TeamConfig, TemplateConfig, TemplateMetadata
    /// };
    ///
    /// let global = GlobalDefaults::new();
    /// let repo_type = RepositoryTypeConfig::new();
    /// let team = TeamConfig::new();
    /// let metadata = TemplateMetadata::new(
    ///     "rust-service".to_string(),
    ///     "A Rust service template".to_string(),
    ///     "author".to_string(),
    ///     vec!["rust".to_string(), "service".to_string()]
    /// );
    /// let template = TemplateConfig::new(metadata);
    ///
    /// let merged = MergedConfiguration::merge_from_hierarchy(
    ///     &global,
    ///     Some(&repo_type),
    ///     Some(&team),
    ///     Some(&template)
    /// );
    /// assert!(merged.is_ok());
    /// ```
    pub fn merge_from_hierarchy(
        global: &GlobalDefaults,
        repo_type: Option<&RepositoryTypeConfig>,
        team: Option<&TeamConfig>,
        template: Option<&TemplateConfig>,
    ) -> Result<Self, ConfigurationError> {
        let mut merged = Self::new();

        // Merge in order of precedence (lowest to highest)
        merged.merge_global_defaults(global)?;

        if let Some(repo_type_cfg) = repo_type {
            merged.merge_repository_type_config(repo_type_cfg)?;
        }

        if let Some(team_cfg) = team {
            merged.merge_team_config(team_cfg)?;
        }

        if let Some(template_cfg) = template {
            merged.merge_template_config(template_cfg)?;
        }

        // Validate the final merged configuration
        merged.validate()?;

        Ok(merged)
    }

    /// Merges global default configuration.
    fn merge_global_defaults(&mut self, global: &GlobalDefaults) -> Result<(), ConfigurationError> {
        // TODO: Merge repository settings from global defaults
        // For now, this is a placeholder implementation

        // Record that global defaults were applied
        self.source_trace.add_source(
            "applied_global_defaults".to_string(),
            ConfigurationSource::Global,
        );

        Ok(())
    }

    /// Merges repository type configuration.
    fn merge_repository_type_config(
        &mut self,
        repo_type: &RepositoryTypeConfig,
    ) -> Result<(), ConfigurationError> {
        // TODO: Merge repository type configuration
        // For now, this is a placeholder implementation

        // Record that repository type config was applied
        self.source_trace.add_source(
            format!("applied_repository_type_{}", "config"),
            ConfigurationSource::RepositoryType,
        );

        Ok(())
    }

    /// Merges team configuration.
    fn merge_team_config(&mut self, team: &TeamConfig) -> Result<(), ConfigurationError> {
        // TODO: Merge team configuration
        // For now, this is a placeholder implementation

        // Record that team config was applied
        self.source_trace
            .add_source("applied_team_config".to_string(), ConfigurationSource::Team);

        Ok(())
    }

    /// Merges template configuration.
    fn merge_template_config(
        &mut self,
        template: &TemplateConfig,
    ) -> Result<(), ConfigurationError> {
        // TODO: Merge template configuration
        // For now, this is a placeholder implementation

        // Record that template config was applied
        self.source_trace.add_source(
            "applied_template_config".to_string(),
            ConfigurationSource::Template,
        );

        Ok(())
    }

    /// Merges an overridable value respecting override settings.
    fn merge_overridable_value<T: Clone>(
        &mut self,
        target: &mut Option<OverridableValue<T>>,
        source: &OverridableValue<T>,
        source_type: ConfigurationSource,
    ) {
        match target {
            Some(existing) => {
                // Only merge if existing allows override or if source has higher precedence
                if existing.can_override || self.has_higher_precedence(source_type) {
                    let mut new_value = source.clone();
                    new_value.can_override = existing.can_override && source.can_override;
                    *target = Some(new_value);
                }
            }
            None => {
                *target = Some(source.clone());
            }
        }
    }

    /// Checks if a source has higher precedence than the existing value.
    fn has_higher_precedence(&self, source_type: ConfigurationSource) -> bool {
        // Template configuration has highest precedence, then team, then repository type, then global
        matches!(source_type, ConfigurationSource::Template)
    }

    /// Adds or replaces a label, tracking its source.
    fn add_or_replace_label(&mut self, label: LabelConfig, source: ConfigurationSource) {
        let name = label.name.clone();
        self.labels.insert(name.clone(), label);
        self.source_trace
            .add_source(format!("labels.{}", name), source);
    }

    /// Adds or replaces a webhook, tracking its source.
    fn add_or_replace_webhook(&mut self, webhook: WebhookConfig, source: ConfigurationSource) {
        // Replace existing webhook with same URL or add new one
        if let Some(pos) = self.webhooks.iter().position(|w| w.url == webhook.url) {
            self.webhooks[pos] = webhook.clone();
        } else {
            self.webhooks.push(webhook.clone());
        }
        self.source_trace
            .add_source(format!("webhooks.{}", webhook.url), source);
    }

    /// Adds or replaces an environment, tracking its source.
    fn add_or_replace_environment(
        &mut self,
        environment: EnvironmentConfig,
        source: ConfigurationSource,
    ) {
        // Replace existing environment with same name or add new one
        if let Some(pos) = self
            .environments
            .iter()
            .position(|e| e.name == environment.name)
        {
            self.environments[pos] = environment.clone();
        } else {
            self.environments.push(environment.clone());
        }
        self.source_trace
            .add_source(format!("environments.{}", environment.name), source);
    }

    /// Adds or replaces a GitHub App, tracking its source.
    fn add_or_replace_github_app(&mut self, app: GitHubAppConfig, source: ConfigurationSource) {
        // Replace existing app with same slug or add new one
        if let Some(pos) = self
            .github_apps
            .iter()
            .position(|a| a.app_slug == app.app_slug)
        {
            self.github_apps[pos] = app.clone();
        } else {
            self.github_apps.push(app.clone());
        }
        self.source_trace
            .add_source(format!("github_apps.{}", app.app_slug), source);
    }
}
