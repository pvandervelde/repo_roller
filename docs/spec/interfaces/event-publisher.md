# Event Publisher Interface

**Architectural Layer**: Business Logic (Domain Layer)
**Module Path**: `repo_roller_core/src/event_publisher.rs`
**Specification**: [outbound-event-notifications.md](../design/outbound-event-notifications.md)

## Overview

The EventPublisher component is responsible for publishing repository lifecycle events to configured webhook endpoints. It operates asynchronously in the background to ensure event delivery does not block the primary repository creation workflow.

## Responsibilities (RDD)

**Knows**:

- Event schemas and serialization formats
- Endpoint configurations and filtering rules
- Request signing requirements
- Delivery timeouts and retry policies

**Does**:

- Publishes events to all configured endpoints
- Loads notification configurations from hierarchy
- Signs HTTP requests with HMAC-SHA256
- Logs delivery attempts and outcomes
- Records metrics for observability

**Collaborates With**:

- `ConfigurationManager` - Loads notification configurations
- `SecretResolver` - Retrieves signing secrets
- `MetricsCollector` - Records delivery statistics
- HTTP Client - Sends webhook requests

## Dependencies

### Type Dependencies

- `RepositoryCreationResult` ([repository-creation.md](repository-creation.md))
- `RepositoryCreationRequest` ([repository-creation.md](repository-creation.md))
- Shared types: `OrganizationName`, `RepositoryName`

### Interface Dependencies

- `SecretResolver` trait ([event-secrets.md](event-secrets.md))
- `EventMetrics` trait ([event-metrics.md](event-metrics.md))

### External Dependencies

- `reqwest` for HTTP client
- `tokio` for async runtime and task spawning
- `serde_json` for event serialization
- `hmac`, `sha2` for request signing
- `tracing` for structured logging

## Public Types

### RepositoryCreatedEvent

**Purpose**: Event payload for `repository.created` event type.

**Signature**:

```rust
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
```

**Behavior**:

- Serializes to JSON for webhook delivery
- Contains complete repository creation context
- Immutable after creation
- All timestamps in UTC

**Constructor**:

```rust
pub fn from_result_and_request(
    result: &RepositoryCreationResult,
    request: &RepositoryCreationRequest,
    merged_config: &MergedConfiguration,
    created_by: &str,
) -> Self
```

**Error Conditions**: None (infallible construction)

### AppliedSettings

**Purpose**: Repository settings that were applied during creation.

**Signature**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedSettings {
    pub has_issues: Option<bool>,
    pub has_wiki: Option<bool>,
    pub has_projects: Option<bool>,
    pub has_discussions: Option<bool>,
}
```

### NotificationEndpoint

**Purpose**: Configuration for a single webhook endpoint.

**Signature**:

```rust
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

fn default_timeout() -> u64 { 5 }
```

**Methods**:

```rust
/// Validates endpoint configuration
pub fn validate(&self) -> Result<(), ValidationError>

/// Checks if this endpoint should receive a specific event type
pub fn accepts_event(&self, event_type: &str) -> bool
```

**Validation Rules**:

- URL must start with `https://` (no HTTP allowed)
- URL must be well-formed (parseable by `url` crate)
- Secret must be non-empty
- Events array must contain at least one event type
- Timeout must be between 1 and 30 seconds

**Error Conditions**:

- `ValidationError::InvalidField` with field name and reason

### NotificationsConfig

**Purpose**: Configuration file structure for notifications.

**Signature**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    /// List of webhook endpoints
    #[serde(default)]
    pub outbound_webhooks: Vec<NotificationEndpoint>,
}
```

**TOML Format**:

```toml
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "WEBHOOK_SECRET_PROD"  # Environment variable name
events = ["repository.created"]
active = true
timeout_seconds = 5
description = "Production deployment system"
```

### DeliveryResult

**Purpose**: Result of delivering an event to one endpoint.

**Signature**:

```rust
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub endpoint_url: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}
```

**Behavior**:

- Captures delivery outcome for observability
- Used for logging and metrics
- Not propagated to caller (fire-and-forget)

## Public Functions

### publish_repository_created

**Purpose**: Publishes a repository creation event to all configured endpoints.

**Signature**:

```rust
pub async fn publish_repository_created(
    result: &RepositoryCreationResult,
    request: &RepositoryCreationRequest,
    merged_config: &MergedConfiguration,
    created_by: &str,
    config_manager: &dyn ConfigurationManager,
    secret_resolver: &dyn SecretResolver,
    metrics: &dyn EventMetrics,
) -> Vec<DeliveryResult>
```

**Parameters**:

- `result` - Successful repository creation outcome
- `request` - Original creation request
- `merged_config` - Merged configuration from hierarchy
- `created_by` - User who requested creation
- `config_manager` - Configuration loader
- `secret_resolver` - Secret resolution service
- `metrics` - Metrics collection service

**Returns**:

- `Vec<DeliveryResult>` - Delivery outcome for each endpoint

**Behavior**:

1. Creates `RepositoryCreatedEvent` from inputs
2. Loads notification endpoints from all config levels (org/team/template)
3. Deduplicates endpoints by URL + event type
4. Filters endpoints by active status and event type
5. Serializes event to JSON once
6. For each endpoint:
   - Resolves signing secret
   - Signs request with HMAC-SHA256
   - Sends HTTP POST with timeout
   - Logs result (INFO for success, WARN for failure)
   - Records metrics
7. Returns all delivery results

**Error Handling**:

- Configuration load failures: Log WARN, skip notifications, return empty vector
- Invalid endpoint configs: Log WARN, skip invalid endpoint
- Event serialization failure: Log ERROR, return empty vector
- Secret resolution failure: Log WARN, skip endpoint
- Network/timeout errors: Log WARN, record in DeliveryResult, continue
- HTTP 4xx/5xx: Log WARN, record in DeliveryResult, continue

**Side Effects**:

- Structured logging (INFO, WARN, ERROR levels)
- Metrics recording (attempts, successes, failures, durations)
- HTTP requests to external systems
- No side effects on repository creation workflow

**Performance**:

- Serializes event payload once, reuses for all endpoints
- Sequential delivery (not parallel) to control resource usage
- Respects per-endpoint timeout settings
- Default timeout: 5 seconds, max: 30 seconds

**Example Usage**:

```rust
let results = publish_repository_created(
    &result,
    &request,
    &merged_config,
    "jane.doe",
    &config_manager,
    &secret_resolver,
    &metrics,
).await;

// Results logged and recorded, no action needed
```

## Support Functions

### compute_hmac_sha256

**Purpose**: Computes HMAC-SHA256 signature for webhook payload.

**Signature**:

```rust
pub fn compute_hmac_sha256(payload: &[u8], secret: &str) -> String
```

**Returns**: `"sha256=<hex-encoded-signature>"`

**Behavior**:

- Uses `hmac` and `sha2` crates
- Returns signature in GitHub webhook format
- Constant-time comparison safe

**Example**:

```rust
let signature = compute_hmac_sha256(payload_bytes, secret);
// Result: "sha256=abc123..."
```

### sign_webhook_request

**Purpose**: Adds HMAC signature header to HTTP request.

**Signature**:

```rust
pub fn sign_webhook_request(
    request: reqwest::RequestBuilder,
    payload: &[u8],
    secret: &str,
) -> reqwest::RequestBuilder
```

**Behavior**:

- Computes signature via `compute_hmac_sha256`
- Adds `X-RepoRoller-Signature-256` header
- Returns modified request builder

**Example**:

```rust
let request = client.post(&endpoint_url)
    .header("Content-Type", "application/json")
    .body(payload);
let signed = sign_webhook_request(request, payload, secret);
```

### collect_notification_endpoints

**Purpose**: Collects and deduplicates endpoints from all configuration levels.

**Signature**:

```rust
pub fn collect_notification_endpoints(
    org_config: &NotificationsConfig,
    team_config: Option<&NotificationsConfig>,
    template_config: Option<&NotificationsConfig>,
) -> Vec<NotificationEndpoint>
```

**Behavior**:

- Collects endpoints from organization, team, template levels
- Deduplicates by `(url, event_types)` tuple
- Preserves order (org → team → template, with deduplication)
- Returns validated, unique endpoints

**Example**:

```rust
let endpoints = collect_notification_endpoints(
    &org_config,
    team_config.as_ref(),
    template_config.as_ref(),
);
```

## Integration Points

### Repository Creation Workflow

The EventPublisher is invoked from `create_repository` function after successful creation:

```rust
// In repo_roller_core/src/lib.rs::create_repository()

// Step 11: Return success result
let result = RepositoryCreationResult { /* ... */ };

// Step 12: Publish event (fire-and-forget)
tokio::spawn(async move {
    let results = publish_repository_created(
        &result,
        &request,
        &merged_config,
        created_by,
        config_manager,
        secret_resolver,
        metrics,
    ).await;

    // Results logged by publish_repository_created
    drop(results);
});

Ok(result)
```

**Key Properties**:

- Spawned in background task (non-blocking)
- Happens AFTER repository creation succeeds
- Failures do not affect repository creation result
- All logging/metrics handled within publisher

## Configuration Loading

Notification configurations are loaded via ConfigurationManager:

```rust
// Organization level
let org_config = config_manager
    .load_notifications_config(organization, None, None)
    .await?;

// Team level (if specified)
let team_config = if let Some(team) = &request.team {
    config_manager
        .load_notifications_config(organization, Some(team), None)
        .await
        .ok()
} else {
    None
};

// Template level (if specified)
let template_config = if let Some(template) = &request.template {
    config_manager
        .load_notifications_config(organization, None, Some(template))
        .await
        .ok()
} else {
    None
};
```

**Configuration Paths**:

- Organization: `.reporoller/global/notifications.toml`
- Team: `.reporoller/teams/{team-name}/notifications.toml`
- Template: `.reporoller/notifications.toml` (in template repository)

## Security Considerations

### Request Signing

All webhook requests include HMAC-SHA256 signature:

- Header: `X-RepoRoller-Signature-256`
- Format: `sha256=<hex-encoded-hmac>`
- Algorithm: HMAC-SHA256(payload, secret)

**Recipient Verification** (documentation for webhook consumers):

```rust
// Constant-time comparison (using subtle crate)
use subtle::ConstantTimeEq;

let computed = compute_hmac_sha256(body, shared_secret);
let is_valid = computed.as_bytes().ct_eq(signature_header.as_bytes()).into();
```

### Secret Management

Secrets are resolved at delivery time via `SecretResolver`:

- Configuration contains secret **references**, not values
- Resolution from environment (env vars, volume mounts, managed identities)
- Secrets never logged or included in error messages
- Failed resolution: skip endpoint, log warning with sanitized message

### HTTPS Enforcement

- Only HTTPS URLs accepted (validated during endpoint configuration)
- HTTP URLs rejected with `ValidationError::InvalidField`
- TLS certificate validation enforced (no insecure connections)

## Testing Requirements

### Unit Tests

**RepositoryCreatedEvent**:

- ✅ Serialization produces valid JSON
- ✅ All required fields present
- ✅ Optional fields omitted when None
- ✅ Timestamps formatted as ISO 8601 UTC

**NotificationEndpoint**:

- ✅ Validation rejects HTTP URLs
- ✅ Validation rejects malformed URLs
- ✅ Validation rejects empty secrets
- ✅ Validation rejects empty events array
- ✅ Validation rejects timeouts < 1 or > 30
- ✅ `accepts_event` filters correctly
- ✅ `accepts_event` respects active status

**compute_hmac_sha256**:

- ✅ Produces correct HMAC-SHA256 signature
- ✅ Format matches `sha256=<hex>` pattern
- ✅ Signature length is 71 characters
- ✅ Same input produces same output

**collect_notification_endpoints**:

- ✅ Accumulates endpoints from all levels
- ✅ Deduplicates by URL + event type
- ✅ Preserves order (org, team, template)
- ✅ Handles missing team/template configs

### Integration Tests

**Mock HTTP Server** (using `wiremock`):

- ✅ Successful delivery (HTTP 200)
- ✅ Failed delivery (HTTP 500)
- ✅ Network timeout
- ✅ Signature verification
- ✅ Request headers correct
- ✅ Payload structure correct

**Mock ConfigurationManager**:

- ✅ Loads from all hierarchy levels
- ✅ Handles missing configurations
- ✅ Returns accumulated endpoints

**Mock SecretResolver**:

- ✅ Resolves secrets successfully
- ✅ Handles resolution failures
- ✅ Never logs secret values

### E2E Tests

**Real HTTP Endpoints** (httpbin.org or similar):

- ✅ End-to-end delivery workflow
- ✅ Signature in request headers
- ✅ Correct payload structure
- ✅ Timeout enforcement

## Metrics

### Prometheus Metrics

**Counters**:

- `notification_delivery_attempts_total` - Total delivery attempts
- `notification_delivery_successes_total` - Successful deliveries
- `notification_delivery_failures_total` - Failed deliveries

**Histograms**:

- `notification_delivery_duration_seconds` - Delivery latency
  - Buckets: [0.1, 0.5, 1.0, 2.5, 5.0, 10.0]

**Gauges**:

- `notification_active_tasks` - Active background delivery tasks

## Logging

### Structured Logging (tracing)

**INFO Level**:

- Event publication started (event_type, event_id, organization)
- Endpoints loaded (endpoint_count, organization)
- Delivery success (endpoint_url, event_id, status_code, response_time_ms)
- Event delivery complete (event_id, success_count, failure_count, total)

**WARN Level**:

- Configuration load failure (organization, error)
- Invalid endpoint configuration (endpoint_url, validation_error)
- Secret resolution failure (endpoint_url, sanitized_error)
- Delivery failure (endpoint_url, event_id, status_code/error, response_time_ms)

**ERROR Level**:

- Event serialization failure (event_id, error)
- Background task spawn failure (error)

## Performance Constraints

**Latency**:

- Event creation: < 1ms
- Configuration loading: < 100ms
- Endpoint validation: < 1ms per endpoint
- Signature computation: < 1ms
- HTTP delivery: Bounded by endpoint timeout (5-30s)

**Resource Usage**:

- Event serialization: Once per event (shared across endpoints)
- HTTP connection pooling: Managed by `reqwest`
- Background tasks: One per repository creation
- Memory: O(n) where n = number of endpoints

**Scalability**:

- Sequential delivery to endpoints (not parallel)
- No retries (fire-and-forget)
- No buffering or queuing
- Stateless (no persistent state)

## Future Enhancements

Out of scope for initial implementation:

- Retry logic with exponential backoff
- Dead letter queue for failed events
- Event replay mechanism
- Additional event types (repository.updated, repository.deleted)
- Batch delivery (multiple events per request)
- Circuit breaker for failing endpoints
- Per-endpoint rate limiting

## References

- [Design Document](../design/outbound-event-notifications.md)
- [Requirements](../requirements/outbound-event-notifications.md)
- [Vocabulary](../vocabulary.md#event-publishing-domain)
- [Responsibilities](../responsibilities.md#eventpublisher)
- [Behavioral Assertions](../assertions.md#event-notification-assertions)
