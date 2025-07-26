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
/// # Security Implications
///
/// Environment configurations control access to deployment targets and
/// should include appropriate protection rules for production environments.
///
/// # Examples
///
/// ```rust
/// use config_manager::organization::EnvironmentConfig;
///
/// let env = EnvironmentConfig::new("production".to_string());
/// // TODO: Add example usage once implemented
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct EnvironmentConfig {
    /// The name of the environment.
    pub name: String,

    /// Whether the environment requires manual approval before deployment.
    pub protection_rules_enabled: Option<OverridableValue<bool>>,
}

impl EnvironmentConfig {
    /// Creates a new environment configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the environment
    ///
    /// # Returns
    ///
    /// A new `EnvironmentConfig` instance.
    pub fn new(name: String) -> Self {
        Self {
            name,
            protection_rules_enabled: None,
        }
    }
}

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
