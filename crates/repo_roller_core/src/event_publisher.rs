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
        result: &RepositoryCreationResult,
        request: &RepositoryCreationRequest,
        merged_config: &config_manager::MergedConfiguration,
        created_by: &str,
    ) -> Self {
        // Generate unique event ID (UUID v4)
        let event_id = uuid::Uuid::new_v4().to_string();

        // Current timestamp in UTC
        let timestamp = chrono::Utc::now();

        // Extract organization and repository name from request
        let organization = request.owner.as_ref().to_string();
        let repository_name = request.name.as_ref().to_string();

        // Determine content strategy string
        let content_strategy = match &request.content_strategy {
            crate::ContentStrategy::Template => "template",
            crate::ContentStrategy::Empty => "empty",
            crate::ContentStrategy::CustomInit { .. } => "custom_init",
        }
        .to_string();

        // Extract visibility (convert enum to string, default to "private" if not specified)
        let visibility = request
            .visibility
            .as_ref()
            .map(|v| match v {
                crate::RepositoryVisibility::Public => "public",
                crate::RepositoryVisibility::Private => "private",
                crate::RepositoryVisibility::Internal => "internal",
            })
            .unwrap_or("private")
            .to_string();

        // Extract template name (only if template strategy)
        let template_name = request.template.as_ref().map(|t| t.as_ref().to_string());

        // Extract repository type from request variables (if present)
        let repository_type = request.variables.get("repository_type").cloned();

        // Extract team from request variables (if present)
        let team = request.variables.get("team").cloned();

        // Extract description from request variables (if present)
        let description = request.variables.get("description").cloned();

        // Extract custom properties from merged config
        let custom_properties = if !merged_config.custom_properties.is_empty() {
            let mut props = HashMap::new();
            for prop in &merged_config.custom_properties {
                // Convert CustomPropertyValue to string
                let value_str = match &prop.value {
                    config_manager::settings::custom_property::CustomPropertyValue::String(s) => {
                        s.clone()
                    }
                    config_manager::settings::custom_property::CustomPropertyValue::SingleSelect(
                        s,
                    ) => s.clone(),
                    config_manager::settings::custom_property::CustomPropertyValue::MultiSelect(
                        vec,
                    ) => vec.join(","),
                    config_manager::settings::custom_property::CustomPropertyValue::Boolean(b) => {
                        b.to_string()
                    }
                };
                props.insert(prop.property_name.clone(), value_str);
            }
            Some(props)
        } else {
            None
        };

        // Extract applied settings from merged config
        let applied_settings = {
            let has_issues = merged_config.repository.issues.as_ref().map(|v| v.value);
            let has_wiki = merged_config.repository.wiki.as_ref().map(|v| v.value);
            let has_projects = merged_config.repository.projects.as_ref().map(|v| v.value);
            let has_discussions = merged_config
                .repository
                .discussions
                .as_ref()
                .map(|v| v.value);

            // Only include applied_settings if at least one setting is present
            if has_issues.is_some()
                || has_wiki.is_some()
                || has_projects.is_some()
                || has_discussions.is_some()
            {
                Some(AppliedSettings {
                    has_issues,
                    has_wiki,
                    has_projects,
                    has_discussions,
                })
            } else {
                None
            }
        };

        Self {
            event_type: "repository.created".to_string(),
            event_id,
            timestamp,
            organization,
            repository_name,
            repository_url: result.repository_url.clone(),
            repository_id: result.repository_id.clone(),
            created_by: created_by.to_string(),
            repository_type,
            template_name,
            content_strategy,
            visibility,
            team,
            description,
            custom_properties,
            applied_settings,
        }
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
    /// Returns `ValidationError::InvalidFormat` if validation fails.
    ///
    /// See docs/spec/interfaces/event-publisher.md#notificationendpoint
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate URL is HTTPS and well-formed
        let parsed_url =
            url::Url::parse(&self.url).map_err(|e| ValidationError::InvalidFormat {
                field: "url".to_string(),
                reason: format!("Malformed URL: {}", e),
            })?;

        if parsed_url.scheme() != "https" {
            return Err(ValidationError::InvalidFormat {
                field: "url".to_string(),
                reason: "URL must use HTTPS scheme".to_string(),
            });
        }

        // Validate secret is non-empty
        if self.secret.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "secret".to_string(),
            });
        }

        // Validate events array is non-empty
        if self.events.is_empty() {
            return Err(ValidationError::InvalidFormat {
                field: "events".to_string(),
                reason: "At least one event type must be specified".to_string(),
            });
        }

        // Validate timeout is within allowed range (1-30 seconds)
        if self.timeout_seconds < 1 || self.timeout_seconds > 30 {
            return Err(ValidationError::InvalidFormat {
                field: "timeout_seconds".to_string(),
                reason: format!(
                    "Timeout must be between 1 and 30 seconds, got {}",
                    self.timeout_seconds
                ),
            });
        }

        Ok(())
    }

    /// Checks if this endpoint should receive a specific event type.
    ///
    /// Returns true if the endpoint is active and configured to receive this event type.
    ///
    /// See docs/spec/interfaces/event-publisher.md#notificationendpoint
    pub fn accepts_event(&self, event_type: &str) -> bool {
        // Inactive endpoints don't accept any events
        if !self.active {
            return false;
        }

        // Check if event type is in the configured list (case-sensitive)
        self.events.iter().any(|e| e == event_type)
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
pub fn compute_hmac_sha256(payload: &[u8], secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    // Create HMAC instance with secret
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can accept keys of any size");

    // Update with payload
    mac.update(payload);

    // Finalize and get result
    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    // Format as "sha256=<hex>"
    format!("sha256={}", hex::encode(code_bytes))
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
    request: reqwest::RequestBuilder,
    payload: &[u8],
    secret: &str,
) -> reqwest::RequestBuilder {
    let signature = compute_hmac_sha256(payload, secret);
    request.header("X-RepoRoller-Signature-256", signature)
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
    org_config: &NotificationsConfig,
    team_config: Option<&NotificationsConfig>,
    template_config: Option<&NotificationsConfig>,
) -> Vec<NotificationEndpoint> {
    use std::collections::HashSet;

    let mut endpoints = Vec::new();
    let mut seen = HashSet::new();

    // Helper to generate deduplication key (URL + sorted events)
    let dedup_key = |endpoint: &NotificationEndpoint| -> (String, Vec<String>) {
        let mut events = endpoint.events.clone();
        events.sort(); // Sort for order-independent comparison
        (endpoint.url.clone(), events)
    };

    // Collect from organization level first
    for endpoint in &org_config.outbound_webhooks {
        let key = dedup_key(endpoint);
        if seen.insert(key) {
            endpoints.push(endpoint.clone());
        }
    }

    // Collect from team level (if present)
    if let Some(team) = team_config {
        for endpoint in &team.outbound_webhooks {
            let key = dedup_key(endpoint);
            if seen.insert(key) {
                endpoints.push(endpoint.clone());
            }
        }
    }

    // Collect from template level (if present)
    if let Some(template) = template_config {
        for endpoint in &template.outbound_webhooks {
            let key = dedup_key(endpoint);
            if seen.insert(key) {
                endpoints.push(endpoint.clone());
            }
        }
    }

    endpoints
}

#[cfg(test)]
#[path = "event_publisher_tests.rs"]
mod tests;
