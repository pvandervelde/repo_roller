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
