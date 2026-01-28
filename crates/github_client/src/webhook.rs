//! GitHub webhook types and operations.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// GitHub webhook event types.
///
/// Represents the different events that can trigger a webhook.
/// See [GitHub webhook events documentation](https://docs.github.com/en/webhooks/webhook-events-and-payloads).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Any Git push to a repository
    Push,
    /// Activity related to pull requests
    PullRequest,
    /// Activity related to pull request reviews
    PullRequestReview,
    /// Activity related to pull request review comments
    PullRequestReviewComment,
    /// Activity related to issues
    Issues,
    /// Activity related to issue comments
    IssueComment,
    /// Activity related to repository creation
    Create,
    /// Activity related to repository deletion
    Delete,
    /// Activity related to repository forks
    Fork,
    /// Activity related to GitHub releases
    Release,
    /// Activity related to repository stars
    Watch,
    /// Activity related to deployments
    Deployment,
    /// Activity related to deployment statuses
    DeploymentStatus,
    /// Activity related to commit statuses
    Status,
    /// All events (wildcard)
    #[serde(rename = "*")]
    All,
}

impl WebhookEvent {
    /// Converts a string slice to a WebhookEvent.
    ///
    /// Returns None if the string doesn't match a known event type.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "push" => Some(Self::Push),
            "pull_request" => Some(Self::PullRequest),
            "pull_request_review" => Some(Self::PullRequestReview),
            "pull_request_review_comment" => Some(Self::PullRequestReviewComment),
            "issues" => Some(Self::Issues),
            "issue_comment" => Some(Self::IssueComment),
            "create" => Some(Self::Create),
            "delete" => Some(Self::Delete),
            "fork" => Some(Self::Fork),
            "release" => Some(Self::Release),
            "watch" => Some(Self::Watch),
            "deployment" => Some(Self::Deployment),
            "deployment_status" => Some(Self::DeploymentStatus),
            "status" => Some(Self::Status),
            "*" => Some(Self::All),
            _ => None,
        }
    }

    /// Converts the WebhookEvent to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::PullRequest => "pull_request",
            Self::PullRequestReview => "pull_request_review",
            Self::PullRequestReviewComment => "pull_request_review_comment",
            Self::Issues => "issues",
            Self::IssueComment => "issue_comment",
            Self::Create => "create",
            Self::Delete => "delete",
            Self::Fork => "fork",
            Self::Release => "release",
            Self::Watch => "watch",
            Self::Deployment => "deployment",
            Self::DeploymentStatus => "deployment_status",
            Self::Status => "status",
            Self::All => "*",
        }
    }
}

/// GitHub webhook representation.
///
/// Contains the complete webhook configuration including its GitHub-assigned ID.
///
/// # Examples
///
/// ```rust
/// use github_client::{Webhook, WebhookEvent};
///
/// // Webhook typically received from GitHub API
/// let webhook_json = r#"{
///     "id": 12345,
///     "url": "https://example.com/webhook",
///     "active": true,
///     "events": ["push", "pull_request"],
///     "config": {
///         "url": "https://example.com/webhook",
///         "content_type": "json",
///         "insecure_ssl": "0"
///     },
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
/// }"#;
///
/// let webhook: Webhook = serde_json::from_str(webhook_json).unwrap();
/// assert_eq!(webhook.id, 12345);
/// assert_eq!(webhook.url, "https://example.com/webhook");
/// assert_eq!(webhook.events.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Webhook {
    /// GitHub-assigned webhook ID
    pub id: u64,

    /// Webhook URL
    pub url: String,

    /// Whether the webhook is active
    pub active: bool,

    /// Events that trigger the webhook
    pub events: Vec<WebhookEvent>,

    /// Webhook configuration details
    pub config: WebhookDetails,

    /// When the webhook was created
    pub created_at: String,

    /// When the webhook was last updated
    pub updated_at: String,
}

/// Webhook configuration details.
///
/// # Examples
///
/// ```rust
/// use github_client::WebhookDetails;
///
/// let details = WebhookDetails {
///     url: "https://example.com/webhook".to_string(),
///     content_type: "json".to_string(),
///     insecure_ssl: false, // Verify SSL (secure)
/// };
///
/// assert_eq!(details.content_type, "json");
/// assert!(!details.insecure_ssl);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDetails {
    /// Webhook URL
    pub url: String,

    /// Content type (json or form)
    pub content_type: String,

    /// Whether to skip SSL certificate verification (insecure)
    ///
    /// - `false` (default): Verify SSL certificates (secure)
    /// - `true`: Skip SSL verification (insecure, not recommended)
    ///
    /// GitHub API uses string "0" (verify) or "1" (skip), but we expose as boolean.
    #[serde(
        default = "default_insecure_ssl",
        serialize_with = "serialize_insecure_ssl",
        deserialize_with = "deserialize_insecure_ssl"
    )]
    pub insecure_ssl: bool,
}

/// Default value for insecure_ssl field (secure by default).
fn default_insecure_ssl() -> bool {
    false // Verify SSL by default (secure)
}

/// Serialize boolean to GitHub API format ("0" or "1").
fn serialize_insecure_ssl<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(if *value { "1" } else { "0" })
}

/// Deserialize from GitHub API format ("0" or "1") to boolean.
fn deserialize_insecure_ssl<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "1")
}

#[cfg(test)]
#[path = "webhook_tests.rs"]
mod tests;
