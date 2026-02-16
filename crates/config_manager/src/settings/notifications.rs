//! Outbound event notification configuration.
//!
//! Defines configuration for webhook endpoints that receive notifications when
//! RepoRoller performs operations (e.g., repository creation). These are distinct
//! from repository webhooks which are created IN repositories.

use serde::{Deserialize, Serialize};

/// Outbound event notification configuration.
///
/// Specifies webhook endpoints that should receive notifications when
/// RepoRoller performs operations. Merged additively from all configuration levels.
///
/// # TOML Format
///
/// ```toml
/// [[notifications.outbound_webhooks]]
/// url = "https://monitoring.example.com/webhook"
/// secret = "ENV:WEBHOOK_SECRET"
/// events = ["repository.created"]
/// active = true
/// timeout_seconds = 5
/// description = "Central monitoring system"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NotificationsConfig {
    /// List of outbound webhook endpoints
    #[serde(default)]
    pub outbound_webhooks: Vec<NotificationEndpoint>,
}

/// Configuration for a single outbound notification endpoint.
///
/// Defines where and how to deliver event notifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationEndpoint {
    /// Webhook URL (must be HTTPS)
    pub url: String,

    /// Secret reference for HMAC signing (e.g., "ENV:SECRET_NAME", "FILE:/path/to/secret")
    pub secret: String,

    /// Events this endpoint should receive
    pub events: Vec<String>,

    /// Whether endpoint is active
    #[serde(default = "default_active")]
    pub active: bool,

    /// Request timeout in seconds (default: 5, max: 30)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl NotificationEndpoint {
    /// Validates endpoint configuration.
    ///
    /// # Errors
    /// Returns error if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        // Validate URL is HTTPS and well-formed
        let parsed_url = url::Url::parse(&self.url).map_err(|e| format!("Malformed URL: {}", e))?;

        if parsed_url.scheme() != "https" {
            return Err("URL must use HTTPS scheme".to_string());
        }

        // Validate secret is non-empty
        if self.secret.is_empty() {
            return Err("Secret cannot be empty".to_string());
        }

        // Validate events array is non-empty
        if self.events.is_empty() {
            return Err(
                "events array cannot be empty, at least one event type must be specified"
                    .to_string(),
            );
        }

        // Validate timeout is within allowed range (1-30 seconds)
        if self.timeout_seconds < 1 || self.timeout_seconds > 30 {
            return Err(format!(
                "timeout_seconds must be between 1 and 30 seconds, got {}",
                self.timeout_seconds
            ));
        }

        Ok(())
    }

    /// Checks if this endpoint should receive a specific event type.
    ///
    /// Returns true if the endpoint is active and configured to receive this event type.
    pub fn accepts_event(&self, event_type: &str) -> bool {
        // Inactive endpoints don't accept any events
        if !self.active {
            return false;
        }

        // Check for wildcard or specific event match (case-sensitive)
        self.events.iter().any(|e| e == "*" || e == event_type)
    }
}

fn default_active() -> bool {
    true
}

fn default_timeout() -> u32 {
    5
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
mod tests;
