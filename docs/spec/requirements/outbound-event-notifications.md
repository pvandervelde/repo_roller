# Outbound Event Notifications Requirements

## Overview

RepoRoller SHALL publish event notifications to configured webhook endpoints when repository lifecycle events occur. This enables external systems to react to repository creation and chain additional automation workflows.

## Use Cases

### UC-001: CI/CD Pipeline Initialization

**Actor**: CI/CD System (Jenkins, GitHub Actions, Azure DevOps)

**Goal**: Automatically configure build pipelines when new repository is created

**Flow**:

1. RepoRoller creates new repository
2. RepoRoller sends notification to CI/CD webhook endpoint
3. CI/CD system receives notification with repository details
4. CI/CD system automatically creates pipeline configuration
5. CI/CD system sets up build triggers and deployment workflows

**Value**: Eliminates manual CI/CD setup, ensures consistent pipeline configuration

### UC-002: Inventory System Updates

**Actor**: IT Asset Management / CMDB System

**Goal**: Maintain up-to-date inventory of all repositories

**Flow**:

1. RepoRoller creates new repository
2. RepoRoller sends notification to inventory system webhook
3. Inventory system records new repository with metadata
4. Inventory system updates dependency graphs and ownership records
5. Inventory system generates compliance reports

**Value**: Automated asset tracking, compliance reporting, ownership visibility

### UC-003: Team Notifications

**Actor**: Communication Platform (Slack, Microsoft Teams)

**Goal**: Notify team members when new repositories are created

**Flow**:

1. RepoRoller creates new repository for a team
2. RepoRoller sends notification to team communication webhook
3. Communication platform posts message to team channel
4. Team members receive notification with repository link
5. Team members can immediately access and contribute to new repository

**Value**: Team awareness, faster onboarding, improved collaboration

### UC-004: Security Scanning Automation

**Actor**: Security Scanning Service (Snyk, SonarQube, GitHub Advanced Security)

**Goal**: Automatically enroll new repositories in security scanning

**Flow**:

1. RepoRoller creates new repository
2. RepoRoller sends notification to security scanning webhook
3. Security service receives notification
4. Security service configures scanning rules based on repository type
5. Security service initiates initial security scan

**Value**: Security coverage from day one, consistent security policies

### UC-005: Documentation Generation

**Actor**: Documentation Platform (Read the Docs, Confluence)

**Goal**: Create documentation spaces for new repositories

**Flow**:

1. RepoRoller creates new documentation repository
2. RepoRoller sends notification to documentation platform
3. Documentation platform creates new documentation project
4. Documentation platform configures build hooks and rendering
5. Documentation platform generates initial documentation structure

**Value**: Consistent documentation, automated documentation lifecycle

## Functional Requirements

### FR-001: Event Publishing

**Requirement**: The system SHALL publish event notifications when repository creation succeeds.

**Details**:

- Event type: `repository.created`
- Delivery method: HTTP POST to configured webhook endpoints
- Delivery timing: After repository creation completes successfully, before returning result to caller
- Delivery semantics: Best-effort, asynchronous, fire-and-forget

**Acceptance Criteria**:

- Event notification sent to all active webhook endpoints
- Delivery attempts logged with success/failure status
- Failed deliveries do not block or fail repository creation
- Event includes complete repository metadata

### FR-002: Event Payload

**Requirement**: Event notifications SHALL include comprehensive repository context.

**Required Fields**:

- `event_type`: Always "repository.created"
- `event_id`: Unique identifier for this event (UUID)
- `timestamp`: ISO 8601 UTC timestamp of event occurrence
- `organization`: Organization name where repository was created
- `repository_name`: Name of created repository
- `repository_url`: Full HTTPS URL to repository
- `repository_id`: GitHub node ID of created repository
- `created_by`: Username/identifier of user who requested creation
- `repository_type`: Type classification (e.g., "service", "library", "documentation")
- `template_name`: Template used for creation (null if empty/custom-init repository)
- `content_strategy`: How repository was initialized ("template", "empty", "custom_init")
- `visibility`: Repository visibility ("public", "private", "internal")

**Optional Fields**:

- `team`: Team name if specified
- `description`: Repository description if provided
- `custom_properties`: Map of custom properties applied
- `applied_settings`: Summary of settings applied (has_issues, has_wiki, etc.)

**Acceptance Criteria**:

- All required fields present in every event
- Optional fields included when data is available
- Field values accurate and match actual repository state
- Payload serialized as JSON

### FR-003: Multi-Level Configuration

**Requirement**: Webhook endpoints SHALL be configurable at organization, team, and template levels.

**Details**:

- Organization-level: `.reporoller/global/notifications.toml`
- Team-level: `.reporoller/teams/{team-name}/notifications.toml`
- Template-level: `.reporoller/` in template repository `notifications.toml`

**Behavior**:

- **Additive**: Endpoints from all levels are accumulated (not overridden)
- **Example**: If org defines 2 endpoints, team defines 1, template defines 1, then 4 notifications are sent
- **Deduplication**: Endpoints with identical URL and event filter are deduplicated

**Acceptance Criteria**:

- Configuration loaded from all hierarchy levels
- Endpoints from all levels accumulated into single delivery list
- Duplicate endpoints detected and removed
- Each unique endpoint receives exactly one notification

### FR-004: Endpoint Configuration

**Requirement**: Each webhook endpoint configuration SHALL specify delivery parameters.

**Configuration Fields**:

- `url` (required): HTTPS URL to POST events to
- `secret` (required): Shared secret for HMAC signing (reference to secret store)
- `events` (required): Array of event types to send to this endpoint (e.g., ["repository.created"])
- `active` (required): Boolean flag to enable/disable endpoint
- `timeout_seconds` (optional): HTTP request timeout (default: 5 seconds)
- `description` (optional): Human-readable endpoint description

**Validation**:

- URL must use HTTPS protocol
- URL must be valid and well-formed
- Secret must be non-empty
- Events array must contain at least one event type
- Timeout must be between 1 and 30 seconds

**Acceptance Criteria**:

- Invalid configurations rejected during configuration loading
- Validation errors logged with clear messages
- Invalid endpoints skipped (do not block repository creation)

### FR-005: Asynchronous Delivery

**Requirement**: Event notifications SHALL be delivered asynchronously without blocking repository creation.

**Details**:

- Delivery happens in background task spawned after repository creation
- Repository creation returns immediately to caller
- Delivery results logged but not returned to caller
- Delivery failures do not propagate errors to caller

**Acceptance Criteria**:

- Repository creation latency not impacted by notification delivery
- Background task completes delivery attempts within reasonable time (< 1 minute)
- All delivery attempts logged regardless of outcome
- Caller receives repository creation result without waiting for notifications

### FR-006: Request Signing

**Requirement**: Outbound webhook requests SHALL be cryptographically signed.

**Details**:

- Signing algorithm: HMAC-SHA256
- Signature header: `X-RepoRoller-Signature-256`
- Signature format: `sha256=<hex_digest>`
- Signed payload: Raw JSON request body
- Secret: Per-endpoint shared secret from configuration

**Verification**:

- Recipients can verify authenticity by recomputing HMAC with shared secret
- Prevents unauthorized parties from spoofing RepoRoller events

**Acceptance Criteria**:

- All webhook requests include signature header
- Signature computed correctly using HMAC-SHA256
- Signature verifiable by recipient using shared secret
- Signature header format matches specification

### FR-007: Delivery Logging

**Requirement**: All delivery attempts SHALL be logged with structured context.

**Log Fields**:

- Timestamp of delivery attempt
- Endpoint URL (sanitized, no secrets)
- Event type and event ID
- HTTP status code (if response received)
- Response time in milliseconds
- Success/failure status
- Error message (if failed)
- Organization and repository name

**Log Levels**:

- INFO: Successful delivery (HTTP 2xx)
- WARN: Failed delivery (HTTP 4xx, 5xx, timeout, network error)
- DEBUG: Detailed request/response information (when debug logging enabled)

**Acceptance Criteria**:

- Every delivery attempt produces at least one log entry
- Log entries include all required context fields
- Failed deliveries clearly logged with error details
- Logs enable troubleshooting delivery issues

## Non-Functional Requirements

### NFR-001: Reliability

**Requirement**: Notification delivery failures SHALL NOT cause repository creation to fail.

**Rationale**: Repository creation is the primary operation; notifications are secondary.

**Acceptance Criteria**:

- Repository creation succeeds even if all notification deliveries fail
- Failed notifications logged as warnings (not errors)
- Delivery failures tracked in metrics for monitoring

### NFR-002: Performance

**Requirement**: Asynchronous notification delivery SHALL NOT significantly impact system throughput.

**Details**:

- Background task spawning overhead: < 1ms
- Maximum concurrent notification tasks: Configurable (default 100)
- Task cleanup: Completed tasks cleaned up promptly

**Acceptance Criteria**:

- Repository creation throughput not reduced by notification system
- System resource usage (memory, file descriptors) remains stable under load
- No task/thread leaks over extended operation

### NFR-003: Security

**Requirement**: Notification delivery SHALL protect sensitive information.

**Details**:

- Webhook secrets never logged or exposed
- HTTPS required for all webhook endpoints (no HTTP)
- TLS certificate validation enforced
- Secrets stored in secure secret management system (Azure Key Vault)

**Acceptance Criteria**:

- Secrets redacted from all logs and error messages
- HTTP endpoints rejected during configuration validation
- TLS errors logged when certificate validation fails
- Secrets loaded securely from secret store

### NFR-004: Observability

**Requirement**: Notification delivery SHALL produce metrics for monitoring and alerting.

**Metrics**:

- `notification_delivery_attempts_total` - Counter by endpoint, event_type, status (success/failure)
- `notification_delivery_duration_seconds` - Histogram by endpoint
- `notification_endpoints_configured` - Gauge by organization
- `notification_active_tasks` - Gauge of background tasks in flight

**Acceptance Criteria**:

- Metrics emitted for every delivery attempt
- Metrics enable monitoring delivery success rates
- Metrics enable alerting on delivery failures
- Metrics enable capacity planning

### NFR-005: Timeout and Resource Management

**Requirement**: Notification delivery SHALL enforce timeouts and resource limits.

**Details**:

- Per-request timeout: Configurable per endpoint (default 5 seconds)
- Maximum payload size: 1 MB
- Connection pooling: HTTP client reuses connections
- DNS caching: Endpoint URLs resolved and cached

**Acceptance Criteria**:

- Requests exceeding timeout are cancelled
- Timeout errors logged with timing information
- HTTP connections properly closed and pooled
- No resource leaks (sockets, file descriptors)

## Configuration Example

```toml
# .reporoller/global/notifications.toml (organization level)
[[outbound_webhooks]]
url = "https://ci.example.com/hooks/repo-created"
secret = "azurekeyvault://prod-keyvault/ci-webhook-secret"
events = ["repository.created"]
active = true
timeout_seconds = 10
description = "CI/CD system integration"

[[outbound_webhooks]]
url = "https://inventory.example.com/api/repositories"
secret = "azurekeyvault://prod-keyvault/inventory-webhook-secret"
events = ["repository.created"]
active = true
description = "Asset inventory system"

# .reporoller/teams/platform-team/notifications.toml (team level)
[[outbound_webhooks]]
url = "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXX"
secret = "azurekeyvault://prod-keyvault/slack-webhook-secret"
events = ["repository.created"]
active = true
description = "Platform team Slack notifications"

# .reporoller/notifications.toml (template level, in template repo)
[[outbound_webhooks]]
url = "https://docs.example.com/api/create-project"
secret = "azurekeyvault://prod-keyvault/docs-webhook-secret"
events = ["repository.created"]
active = true
description = "Documentation platform integration"
```

## Event Payload Example

```json
{
  "event_type": "repository.created",
  "event_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "timestamp": "2026-02-09T14:30:00.000Z",
  "organization": "acme-corp",
  "repository_name": "awesome-service",
  "repository_url": "https://github.com/acme-corp/awesome-service",
  "repository_id": "R_kgDOH1234567",
  "created_by": "john.doe",
  "repository_type": "service",
  "template_name": "microservice-template",
  "content_strategy": "template",
  "visibility": "private",
  "team": "platform-team",
  "description": "New microservice for handling payments",
  "custom_properties": {
    "cost_center": "engineering",
    "compliance_level": "high"
  },
  "applied_settings": {
    "has_issues": true,
    "has_wiki": false,
    "has_projects": false,
    "has_discussions": false
  }
}
```

## Success Criteria

The outbound event notifications feature is successful when:

1. ✅ External systems can reliably receive repository creation events
2. ✅ Notification delivery never blocks or fails repository creation
3. ✅ Events contain sufficient context for downstream automation
4. ✅ Webhook endpoints can verify authenticity of events via signatures
5. ✅ Failed deliveries are observable through logs and metrics
6. ✅ Configuration is flexible (org/team/template levels)
7. ✅ System performs efficiently under load (async, non-blocking)

## Out of Scope

The following are explicitly out of scope for initial implementation:

- **Retry logic**: Failed deliveries are not retried (best-effort only)
- **Dead letter queue**: Failed events are not queued for later retry
- **Delivery confirmation**: No mechanism to confirm receipt by endpoint
- **Event ordering guarantees**: No guarantee on delivery order if multiple events
- **Rate limiting**: No rate limiting on outbound requests (assumed low volume)
- **Event replay**: No mechanism to replay historical events
- **Additional event types**: Only `repository.created` initially
- **Webhook management UI**: Configuration only via TOML files
- **Batch delivery**: Events sent individually (no batching)

These features may be considered for future enhancements based on real-world usage and requirements.
