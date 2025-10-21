//! Webhook configuration.

use serde::{Deserialize, Serialize};

/// Webhook configuration.
///
/// Defines a webhook that will be created in the repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,

    /// Content type (json or form)
    pub content_type: String,

    /// Secret for webhook validation
    pub secret: Option<String>,

    /// Whether the webhook is active
    pub active: bool,

    /// Events that trigger the webhook
    pub events: Vec<String>,
}

#[cfg(test)]
#[path = "webhook_tests.rs"]
mod tests;
