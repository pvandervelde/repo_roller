//! Basic configuration types and enums.
//!
//! This module provides the foundational types used throughout the configuration system,
//! including GitHub-specific enums and basic configuration structures.

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

/// Repository visibility options in GitHub.
///
/// This enum represents the three possible visibility settings for GitHub repositories.
/// Organizations can enforce specific visibility policies through configuration.
///
/// # Examples
///
/// ```rust
/// use config_manager::types::RepositoryVisibility;
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
/// use config_manager::types::MergeType;
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
/// use config_manager::types::CommitMessageOption;
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
/// use config_manager::types::WorkflowPermission;
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
/// use config_manager::types::WebhookEvent;
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
/// use config_manager::types::LabelConfig;
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
    /// use config_manager::types::LabelConfig;
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
/// use config_manager::types::{WebhookConfig, WebhookEvent};
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
    /// use config_manager::types::{WebhookConfig, WebhookEvent};
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
/// use config_manager::types::{MergeConfig, MergeType, CommitMessageOption};
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
    /// use config_manager::types::{MergeConfig, MergeType, CommitMessageOption};
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
/// use config_manager::types::EnvironmentConfig;
///
/// // Production environment with strict protection
/// let production = EnvironmentConfig {
///     name: "production".to_string(),
///     required_reviewers: Some(vec!["@team-leads".to_string(), "@security-team".to_string()]),
///     wait_timer: Some(300), // 5 minute wait timer
///     deployment_branch_policy: Some("main".to_string()),
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentConfig {
    /// The name of the environment (e.g., "production", "staging", "preview").
    pub name: String,

    /// List of required reviewers for deployments to this environment.
    pub required_reviewers: Option<Vec<String>>,

    /// Wait timer in seconds before deployments can proceed.
    pub wait_timer: Option<u32>,

    /// Deployment branch policy restricting which branches can deploy.
    pub deployment_branch_policy: Option<String>,
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

/// Custom repository property definition.
///
/// Represents metadata properties that can be attached to repositories for
/// categorization, compliance tracking, and automation workflows.
///
/// # Examples
///
/// ```rust
/// use config_manager::types::CustomProperty;
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

/// GitHub App configuration for repository integration.
///
/// Defines GitHub Apps that should be installed on repositories for
/// automation, security scanning, or development workflow enhancement.
///
/// # Examples
///
/// ```rust
/// use config_manager::types::GitHubAppConfig;
/// use config_manager::hierarchy::OverridableValue;
///
/// let app = GitHubAppConfig::new("dependabot".to_string());
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct GitHubAppConfig {
    /// The slug name of the GitHub App.
    pub app_slug: String,

    /// Whether the app installation is required.
    pub required: Option<crate::hierarchy::OverridableValue<bool>>,
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
