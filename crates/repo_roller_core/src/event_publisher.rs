// GENERATED FROM: docs/spec/interfaces/event-publisher.md
// Event publishing operations for repository lifecycle events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::ValidationError;
use crate::{RepositoryCreationRequest, RepositoryCreationResult};

/// Event published when a repository is successfully created.
///
/// See docs/spec/interfaces/event-publisher.md#repositorycreatedevent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryCreatedEvent {
    /// Event type identifier (always "repository.created")
    pub event_type: String,

    /// Unique identifier for this event (UUID v4)
    pub event_id: String,

    /// Timestamp when event occurred (ISO 8601 UTC)
    pub timestamp: DateTime<Utc>,

    /// Organization where repository was created
    pub organization: String,

    /// Name of created repository
    pub repository_name: String,

    /// Full HTTPS URL to repository
    pub repository_url: String,

    /// GitHub node ID of repository
    pub repository_id: String,

    /// User who requested repository creation
    pub created_by: String,

    /// Repository type classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,

    /// Template used for creation (null if empty/custom-init)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_name: Option<String>,

    /// Content strategy used
    pub content_strategy: String,

    /// Repository visibility
    pub visibility: String,

    /// Team name if specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Custom properties applied to repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_properties: Option<HashMap<String, String>>,

    /// Applied repository settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_settings: Option<AppliedSettings>,
}

impl RepositoryCreatedEvent {
    /// Creates a new event from repository creation result and request.
    ///
    /// See docs/spec/interfaces/event-publisher.md#repositorycreatedevent
    pub fn from_result_and_request(
        _result: &RepositoryCreationResult,
        _request: &RepositoryCreationRequest,

        _created_by: &str,
    ) -> Self {
        unimplemented!("See docs/spec/interfaces/event-publisher.md#repositorycreatedevent")
    }
}

/// Repository settings that were applied during creation.
///
/// See docs/spec/interfaces/event-publisher.md#appliedsettings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedSettings {
    pub has_issues: Option<bool>,
    pub has_wiki: Option<bool>,
    pub has_projects: Option<bool>,
    pub has_discussions: Option<bool>,
}

/// Configuration for a single webhook endpoint.
///
/// See docs/spec/interfaces/event-publisher.md#notificationendpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEndpoint {
    /// HTTPS URL to POST events to
    pub url: String,

    /// Reference to shared secret (resolved via SecretResolver)
    pub secret: String,

    /// Event types to send to this endpoint
    pub events: Vec<String>,

    /// Whether this endpoint is active
    pub active: bool,

    /// HTTP request timeout in seconds (default: 5)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_timeout() -> u64 {
    5
}

impl NotificationEndpoint {
    /// Validates endpoint configuration.
    ///
    /// # Errors
    /// Returns `ValidationError::InvalidField` if validation fails.
    ///
    /// See docs/spec/interfaces/event-publisher.md#notificationendpoint
    pub fn validate(&self) -> Result<(), ValidationError> {
        unimplemented!("See docs/spec/interfaces/event-publisher.md#notificationendpoint")
    }

    /// Checks if this endpoint should receive a specific event type.
    ///
    /// See docs/spec/interfaces/event-publisher.md#notificationendpoint
    pub fn accepts_event(&self, _event_type: &str) -> bool {
        unimplemented!("See docs/spec/interfaces/event-publisher.md#notificationendpoint")
    }
}

/// Configuration file structure for notifications.
///
/// See docs/spec/interfaces/event-publisher.md#notificationsconfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    /// List of webhook endpoints
    #[serde(default)]
    pub outbound_webhooks: Vec<NotificationEndpoint>,
}

/// Result of delivering an event to one endpoint.
///
/// See docs/spec/interfaces/event-publisher.md#deliveryresult
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub endpoint_url: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

/// Publishes a repository creation event to all configured endpoints.
///
/// This function is called after successful repository creation to notify
/// external systems. It runs asynchronously and does not block the repository
/// creation workflow.
///
/// # Arguments
/// * `result` - Successful repository creation outcome
/// * `request` - Original creation request
/// * `created_by` - User who requested creation
/// * `secret_resolver` - Secret resolution service
/// * `metrics` - Metrics collection service
///
/// # Returns
/// Vector of delivery results (one per endpoint)
///
/// # Error Handling
/// - Configuration load failures: Log WARN, return empty vector
/// - Invalid endpoints: Log WARN, skip invalid endpoint
/// - Serialization failures: Log ERROR, return empty vector
/// - Secret resolution failures: Log WARN, skip endpoint
/// - Network/timeout errors: Log WARN, record in DeliveryResult
///
/// See docs/spec/interfaces/event-publisher.md#publish_repository_created
///
/// NOTE: Full implementation will integrate with ConfigurationManager trait
/// once event notification configuration methods are added to that trait.
pub async fn publish_repository_created(
    _result: &RepositoryCreationResult,
    _request: &RepositoryCreationRequest,
    _created_by: &str,
    _secret_resolver: &dyn crate::event_secrets::SecretResolver,
    _metrics: &dyn crate::event_metrics::EventMetrics,
) -> Vec<DeliveryResult> {
    unimplemented!("See docs/spec/interfaces/event-publisher.md#publish_repository_created")
}

/// Computes HMAC-SHA256 signature for webhook payload.
///
/// # Arguments
/// * `payload` - Raw payload bytes
/// * `secret` - Shared secret for signing
///
/// # Returns
/// Signature in format: `"sha256=<hex-encoded-signature>"`
///
/// See docs/spec/interfaces/event-publisher.md#compute_hmac_sha256
pub fn compute_hmac_sha256(_payload: &[u8], _secret: &str) -> String {
    unimplemented!("See docs/spec/interfaces/event-publisher.md#compute_hmac_sha256")
}

/// Adds HMAC signature header to HTTP request.
///
/// # Arguments
/// * `request` - Request builder
/// * `payload` - Raw payload bytes
/// * `secret` - Shared secret for signing
///
/// # Returns
/// Modified request builder with signature header
///
/// See docs/spec/interfaces/event-publisher.md#sign_webhook_request
pub fn sign_webhook_request(
    _request: reqwest::RequestBuilder,
    _payload: &[u8],
    _secret: &str,
) -> reqwest::RequestBuilder {
    unimplemented!("See docs/spec/interfaces/event-publisher.md#sign_webhook_request")
}

/// Collects and deduplicates endpoints from all configuration levels.
///
/// # Arguments
/// * `org_config` - Organization-level configuration
/// * `team_config` - Team-level configuration (optional)
/// * `template_config` - Template-level configuration (optional)
///
/// # Returns
/// Deduplicated list of notification endpoints
///
/// See docs/spec/interfaces/event-publisher.md#collect_notification_endpoints
pub fn collect_notification_endpoints(
    _org_config: &NotificationsConfig,
    _team_config: Option<&NotificationsConfig>,
    _template_config: Option<&NotificationsConfig>,
) -> Vec<NotificationEndpoint> {
    unimplemented!("See docs/spec/interfaces/event-publisher.md#collect_notification_endpoints")
}

#[cfg(test)]
#[path = "event_publisher_tests.rs"]
mod tests;
