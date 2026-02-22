// GENERATED FROM: docs/spec/interfaces/event-publisher.md
// Event publishing operations for repository lifecycle events

use chrono::{DateTime, Utc};
use config_manager::{NotificationEndpoint, NotificationsConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{RepositoryCreationRequest, RepositoryCreationResult};

/// Configuration context for event notification delivery after repository creation.
///
/// Bundles the parameters required for background event notification delivery,
/// reducing the number of arguments callers must pass to [`crate::create_repository`].
///
/// # Examples
///
/// ```no_run
/// use repo_roller_core::event_publisher::EventNotificationContext;
/// use repo_roller_core::event_secrets::EnvironmentSecretResolver;
/// use repo_roller_core::event_metrics::PrometheusEventMetrics;
/// use std::sync::Arc;
///
/// let context = EventNotificationContext::new(
///     "deployment-system",
///     Arc::new(EnvironmentSecretResolver::new()),
///     Arc::new(PrometheusEventMetrics::new(&prometheus::Registry::new())),
/// );
/// ```
pub struct EventNotificationContext {
    /// Identifier for who or what triggered the repository creation (e.g., username, system name).
    pub created_by: String,
    /// Resolver for webhook endpoint secrets (e.g., HMAC signing keys).
    pub secret_resolver: std::sync::Arc<dyn crate::event_secrets::SecretResolver>,
    /// Metrics collector for tracking event delivery outcomes.
    pub metrics: std::sync::Arc<dyn crate::event_metrics::EventMetrics>,
}

impl EventNotificationContext {
    /// Create a new event notification context.
    ///
    /// # Arguments
    ///
    /// * `created_by` - Name of the user or system triggering repository creation
    /// * `secret_resolver` - Provider for webhook signing secrets
    /// * `metrics` - Metrics collector for delivery tracking
    pub fn new(
        created_by: impl Into<String>,
        secret_resolver: std::sync::Arc<dyn crate::event_secrets::SecretResolver>,
        metrics: std::sync::Arc<dyn crate::event_metrics::EventMetrics>,
    ) -> Self {
        Self {
            created_by: created_by.into(),
            secret_resolver,
            metrics,
        }
    }
}

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
/// * `merged_config` - Merged configuration with notification settings
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
pub async fn publish_repository_created(
    result: &RepositoryCreationResult,
    request: &RepositoryCreationRequest,
    merged_config: &config_manager::MergedConfiguration,
    created_by: &str,
    secret_resolver: &dyn crate::event_secrets::SecretResolver,
    metrics: &dyn crate::event_metrics::EventMetrics,
) -> Vec<DeliveryResult> {
    use tracing::{error, info, warn};

    // Step 1: Create the event payload
    let event =
        RepositoryCreatedEvent::from_result_and_request(result, request, merged_config, created_by);

    // Step 2: Serialize event to JSON
    let payload_json = match serde_json::to_string(&event) {
        Ok(json) => json,
        Err(e) => {
            error!(
                event_id = %event.event_id,
                error = %e,
                "Failed to serialize event to JSON"
            );
            return Vec::new();
        }
    };
    let payload_bytes = payload_json.as_bytes();

    info!(
        event_id = %event.event_id,
        event_type = %event.event_type,
        organization = %event.organization,
        repository = %event.repository_name,
        "Publishing repository creation event"
    );

    // Step 3: Get endpoints from merged config (no deduplication needed - already done in merge)
    let endpoints = &merged_config.notifications.outbound_webhooks;

    // Filter for active endpoints that accept this event type
    let event_type = "repository.created";
    let matching_endpoints: Vec<&NotificationEndpoint> = endpoints
        .iter()
        .filter(|endpoint| endpoint.active && endpoint.accepts_event(event_type))
        .collect();

    info!(
        event_id = %event.event_id,
        endpoint_count = matching_endpoints.len(),
        "Collected notification endpoints"
    );

    if matching_endpoints.is_empty() {
        info!(
            event_id = %event.event_id,
            "No matching notification endpoints configured"
        );
        return Vec::new();
    }

    // Step 4: Deliver to each endpoint sequentially
    let mut results = Vec::new();
    let client = reqwest::Client::new();

    for endpoint in matching_endpoints {
        let start_time = std::time::Instant::now();

        // Resolve secret
        let secret = match secret_resolver.resolve_secret(&endpoint.secret).await {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    event_id = %event.event_id,
                    endpoint_url = %endpoint.url,
                    error = %e,
                    "Secret resolution failed, skipping endpoint"
                );
                // Skip this endpoint - secret resolution failed
                continue;
            }
        };

        // Create HTTP request
        let request_builder = client
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RepoRoller/1.0")
            .timeout(std::time::Duration::from_secs(
                endpoint.timeout_seconds as u64,
            ))
            .body(payload_bytes.to_vec());

        // Sign request
        let signed_request = sign_webhook_request(request_builder, payload_bytes, &secret);

        // Send request
        match signed_request.send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let duration_ms = start_time.elapsed().as_millis() as u64;

                if response.status().is_success() {
                    info!(
                        event_id = %event.event_id,
                        endpoint_url = %endpoint.url,
                        status_code = status_code,
                        response_time_ms = duration_ms,
                        "Event delivery successful"
                    );
                    metrics.record_delivery_success(&endpoint.url, duration_ms);

                    results.push(DeliveryResult {
                        endpoint_url: endpoint.url.clone(),
                        success: true,
                        status_code: Some(status_code),
                        response_time_ms: duration_ms,
                        error_message: None,
                    });
                } else {
                    warn!(
                        event_id = %event.event_id,
                        endpoint_url = %endpoint.url,
                        status_code = status_code,
                        response_time_ms = duration_ms,
                        "Event delivery failed with HTTP error"
                    );
                    metrics.record_delivery_failure(&endpoint.url, status_code);

                    results.push(DeliveryResult {
                        endpoint_url: endpoint.url.clone(),
                        success: false,
                        status_code: Some(status_code),
                        response_time_ms: duration_ms,
                        error_message: Some(format!("HTTP {}", status_code)),
                    });
                }
            }
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let error_msg = if e.is_timeout() {
                    "Request timeout".to_string()
                } else if e.is_connect() {
                    "Connection failed".to_string()
                } else {
                    format!("Network error: {}", e)
                };

                warn!(
                    event_id = %event.event_id,
                    endpoint_url = %endpoint.url,
                    error = %error_msg,
                    response_time_ms = duration_ms,
                    "Event delivery failed with network error"
                );
                metrics.record_delivery_error(&endpoint.url);

                results.push(DeliveryResult {
                    endpoint_url: endpoint.url.clone(),
                    success: false,
                    status_code: None,
                    response_time_ms: duration_ms,
                    error_message: Some(error_msg),
                });
            }
        }
    }

    info!(
        event_id = %event.event_id,
        success_count = results.iter().filter(|r| r.success).count(),
        failure_count = results.iter().filter(|r| !r.success).count(),
        total_endpoints = results.len(),
        "Event delivery complete"
    );

    results
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
