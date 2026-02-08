//! GitHub webhook types and operations.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

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

impl FromStr for WebhookEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "push" => Ok(Self::Push),
            "pull_request" => Ok(Self::PullRequest),
            "pull_request_review" => Ok(Self::PullRequestReview),
            "pull_request_review_comment" => Ok(Self::PullRequestReviewComment),
            "issues" => Ok(Self::Issues),
            "issue_comment" => Ok(Self::IssueComment),
            "create" => Ok(Self::Create),
            "delete" => Ok(Self::Delete),
            "fork" => Ok(Self::Fork),
            "release" => Ok(Self::Release),
            "watch" => Ok(Self::Watch),
            "deployment" => Ok(Self::Deployment),
            "deployment_status" => Ok(Self::DeploymentStatus),
            "status" => Ok(Self::Status),
            "*" => Ok(Self::All),
            _ => Err(format!("Unknown webhook event type: {}", s)),
        }
    }
}

impl WebhookEvent {
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

/// Parameters for creating a webhook.
///
/// Groups webhook configuration into a single struct to avoid
/// functions with too many parameters.
#[derive(Debug, Clone)]
pub struct CreateWebhookParams<'a> {
    /// Webhook URL
    pub url: &'a str,
    /// Content type ("json" or "form")
    pub content_type: &'a str,
    /// Optional secret for webhook signatures
    pub secret: Option<&'a str>,
    /// Whether the webhook is active
    pub active: bool,
    /// Events that trigger this webhook
    pub events: &'a [String],
}

/// Parameters for updating a webhook.
///
/// Groups webhook configuration into a single struct to avoid
/// functions with too many parameters.
#[derive(Debug, Clone)]
pub struct UpdateWebhookParams<'a> {
    /// Webhook URL
    pub url: &'a str,
    /// Content type ("json" or "form")
    pub content_type: &'a str,
    /// Optional secret for webhook signatures
    pub secret: Option<&'a str>,
    /// Whether the webhook is active
    pub active: bool,
    /// Events that trigger this webhook
    pub events: &'a [String],
}

#[cfg(test)]
#[path = "webhook_tests.rs"]
mod tests;
