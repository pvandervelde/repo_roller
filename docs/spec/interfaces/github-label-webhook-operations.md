# GitHub Label and Webhook Operations

**Architectural Layer**: Infrastructure / External Systems
**Module Paths**:

- `crates/github_client/src/lib.rs` - GitHub API methods
- `crates/repo_roller_core/src/label_manager.rs` - Label orchestration (new)
- `crates/repo_roller_core/src/webhook_manager.rs` - Webhook orchestration (new)

**Responsibilities** (from RDD):

- **Knows**: GitHub API endpoints for labels/webhooks, API response formats, error codes
- **Does**: Creates/updates/lists labels and webhooks via GitHub REST API, handles idempotency, validates configurations
- **Collaborates**: GitHubClient (API operations), ConfigurationManager (label/webhook configs)

## Dependencies

- Types: `LabelConfig`, `WebhookConfig` ([crates/config_manager/src/settings](../../crates/config_manager/src/settings))
- GitHub API Client: `GitHubClient` ([crates/github_client/src/lib.rs](../../crates/github_client/src/lib.rs))
- Error Types: `RepoRollerError` ([specs/interfaces/error-types.md](error-types.md))
- Shared: `Result<T, E>` ([specs/interfaces/shared-types.md](shared-types.md))

## Architecture Context

This interface defines the operations for managing GitHub repository labels and webhooks. Following clean architecture principles:

- **Business Logic** (LabelManager, WebhookManager): Orchestrates label/webhook application with business rules
- **Interface/Port**: Trait methods defining what operations are needed
- **Infrastructure/Adapter**: GitHubClient implements the actual GitHub API calls

Labels and webhooks are part of repository configuration and are applied after repository creation as part of the configuration workflow.

## GitHub API Methods (GitHubClient)

### Label Operations

#### list_labels

**Status**: ✅ Already implemented (as `list_repository_labels`)

```rust
/// Lists all labels in a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
///
/// # Returns
/// Vector of label names currently defined in the repository.
///
/// # Errors
/// * `Error::ApiError` - GitHub API returned an error
/// * `Error::InvalidResponse` - Response parsing failed
///
/// # GitHub API
/// GET /repos/{owner}/{repo}/labels
pub async fn list_repository_labels(
    &self,
    owner: &str,
    repo: &str,
) -> Result<Vec<String>, Error>;
```

#### create_label

**Status**: ✅ Already implemented (idempotent - updates if exists)

```rust
/// Creates a label in a repository, or updates it if it already exists.
///
/// This method is idempotent - if the label already exists, it will be updated
/// with the provided color and description instead of failing.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `name` - Label name
/// * `color` - Label color (hex code without #, e.g., "d73a4a")
/// * `description` - Label description
///
/// # Returns
/// `Ok(())` on successful creation or update
///
/// # Errors
/// * `Error::InvalidResponse` - API call failed or response invalid
/// * `Error::ApiError` - GitHub API error (non-422)
///
/// # Behavior
/// 1. Attempts to create label via POST
/// 2. If 422 "label already exists" error, updates via PATCH
/// 3. Returns success if either operation succeeds
///
/// # GitHub API
/// POST /repos/{owner}/{repo}/labels
/// PATCH /repos/{owner}/{repo}/labels/{name}
pub async fn create_label(
    &self,
    owner: &str,
    repo: &str,
    name: &str,
    color: &str,
    description: &str,
) -> Result<(), Error>;
```

#### update_label

**Status**: 🆕 **New method needed**

```rust
/// Updates an existing label in a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `name` - Current label name
/// * `new_name` - New label name (if renaming, otherwise same as `name`)
/// * `color` - New label color (hex code without #)
/// * `description` - New label description
///
/// # Returns
/// `Ok(())` on successful update
///
/// # Errors
/// * `Error::NotFound` - Label does not exist
/// * `Error::InvalidResponse` - API call failed
/// * `Error::ApiError` - GitHub API error
///
/// # GitHub API
/// PATCH /repos/{owner}/{repo}/labels/{name}
pub async fn update_label(
    &self,
    owner: &str,
    repo: &str,
    name: &str,
    new_name: &str,
    color: &str,
    description: &str,
) -> Result<(), Error>;
```

#### delete_label

**Status**: 🆕 **New method needed** (optional, for cleanup/testing)

```rust
/// Deletes a label from a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `name` - Label name to delete
///
/// # Returns
/// `Ok(())` on successful deletion
///
/// # Errors
/// * `Error::NotFound` - Label does not exist (may be considered success)
/// * `Error::InvalidResponse` - API call failed
///
/// # GitHub API
/// DELETE /repos/{owner}/{repo}/labels/{name}
pub async fn delete_label(
    &self,
    owner: &str,
    repo: &str,
    name: &str,
) -> Result<(), Error>;
```

### Webhook Operations

#### list_webhooks

**Status**: 🆕 **New method needed**

```rust
/// Lists all webhooks configured for a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
///
/// # Returns
/// Vector of `Webhook` structs containing webhook details.
///
/// # Errors
/// * `Error::InvalidResponse` - API call failed or response parsing failed
/// * `Error::ApiError` - GitHub API error
///
/// # GitHub API
/// GET /repos/{owner}/{repo}/hooks
pub async fn list_webhooks(
    &self,
    owner: &str,
    repo: &str,
) -> Result<Vec<Webhook>, Error>;
```

#### create_webhook

**Status**: 🆕 **New method needed**

```rust
/// Creates a webhook in a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `config` - Webhook configuration (URL, events, secret, etc.)
///
/// # Returns
/// The created `Webhook` with its ID and configuration.
///
/// # Errors
/// * `Error::InvalidResponse` - API call failed
/// * `Error::ApiError` - GitHub API error (duplicate webhook, invalid config, etc.)
///
/// # Behavior
/// This method does NOT check for existing webhooks - caller should verify
/// uniqueness if needed (e.g., check if webhook with same URL already exists).
///
/// # GitHub API
/// POST /repos/{owner}/{repo}/hooks
pub async fn create_webhook(
    &self,
    owner: &str,
    repo: &str,
    config: &WebhookConfig,
) -> Result<Webhook, Error>;
```

#### update_webhook

**Status**: 🆕 **New method needed**

```rust
/// Updates an existing webhook in a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `webhook_id` - GitHub webhook ID (from list_webhooks or create_webhook)
/// * `config` - New webhook configuration
///
/// # Returns
/// The updated `Webhook` with its configuration.
///
/// # Errors
/// * `Error::NotFound` - Webhook ID does not exist
/// * `Error::InvalidResponse` - API call failed
/// * `Error::ApiError` - GitHub API error
///
/// # GitHub API
/// PATCH /repos/{owner}/{repo}/hooks/{hook_id}
pub async fn update_webhook(
    &self,
    owner: &str,
    repo: &str,
    webhook_id: u64,
    config: &WebhookConfig,
) -> Result<Webhook, Error>;
```

#### delete_webhook

**Status**: 🆕 **New method needed** (optional, for cleanup/testing)

```rust
/// Deletes a webhook from a repository.
///
/// # Arguments
/// * `owner` - Repository owner (organization or user)
/// * `repo` - Repository name
/// * `webhook_id` - GitHub webhook ID
///
/// # Returns
/// `Ok(())` on successful deletion
///
/// # Errors
/// * `Error::NotFound` - Webhook does not exist (may be considered success)
/// * `Error::InvalidResponse` - API call failed
///
/// # GitHub API
/// DELETE /repos/{owner}/{repo}/hooks/{hook_id}
pub async fn delete_webhook(
    &self,
    owner: &str,
    repo: &str,
    webhook_id: u64,
) -> Result<(), Error>;
```

## Supporting Types

### Webhook

**Status**: 🆕 **New type needed** (return type for webhook operations)

```rust
/// GitHub webhook representation.
///
/// Contains the complete webhook configuration including its GitHub-assigned ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// GitHub-assigned webhook ID
    pub id: u64,

    /// Webhook URL
    pub url: String,

    /// Whether the webhook is active
    pub active: bool,

    /// Events that trigger the webhook
    pub events: Vec<String>,

    /// Webhook configuration details
    pub config: WebhookDetails,

    /// When the webhook was created
    pub created_at: String,

    /// When the webhook was last updated
    pub updated_at: String,
}

/// Webhook configuration details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDetails {
    /// Webhook URL
    pub url: String,

    /// Content type (json or form)
    pub content_type: String,

    /// Whether to verify SSL certificates
    #[serde(default = "default_insecure_ssl")]
    pub insecure_ssl: String, // "0" or "1" as string per GitHub API
}

fn default_insecure_ssl() -> String {
    "0".to_string() // Verify SSL by default
}
```

## Orchestration Components (Business Logic)

### LabelManager

**Status**: 🆕 **New component needed**

**Purpose**: Orchestrates label operations with business logic, idempotency, and error handling.

**Location**: `crates/repo_roller_core/src/label_manager.rs`

```rust
/// Manages label operations for repositories.
///
/// This manager provides high-level label management operations that handle
/// idempotency, bulk operations, and proper error handling. It orchestrates
/// the GitHubClient label operations with business logic.
pub struct LabelManager {
    /// GitHub client for API operations
    github_client: GitHubClient,
}

impl LabelManager {
    /// Creates a new LabelManager.
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Applies labels to a repository, creating or updating as needed.
    ///
    /// This method ensures that all labels from the configuration are present
    /// in the repository. It is idempotent and safe to call multiple times.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `labels` - Map of label name to LabelConfig
    ///
    /// # Returns
    /// `Ok(ApplyLabelsResult)` with details of operations performed
    ///
    /// # Errors
    /// Returns `RepoRollerError::System` if label operations fail.
    ///
    /// # Behavior
    /// 1. Iterates through all labels in configuration
    /// 2. Creates or updates each label (create_label is idempotent)
    /// 3. Logs each operation (created/updated/skipped)
    /// 4. Returns summary of operations
    ///
    /// # Error Handling
    /// - Continues on individual label failures (logs warning)
    /// - Returns error only if all labels fail
    /// - Partial success is considered success
    pub async fn apply_labels(
        &self,
        owner: &str,
        repo: &str,
        labels: &std::collections::HashMap<String, LabelConfig>,
    ) -> Result<ApplyLabelsResult, RepoRollerError>;

    /// Lists all labels currently defined in a repository.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Vector of label names
    ///
    /// # Errors
    /// Returns `RepoRollerError::System` if API call fails.
    pub async fn list_labels(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<String>, RepoRollerError>;
}

/// Result of applying labels to a repository.
#[derive(Debug, Clone)]
pub struct ApplyLabelsResult {
    /// Number of labels created
    pub created: usize,

    /// Number of labels updated
    pub updated: usize,

    /// Number of labels that already existed with correct configuration
    pub skipped: usize,

    /// Number of labels that failed to apply
    pub failed: usize,

    /// Names of labels that failed (for error reporting)
    pub failed_labels: Vec<String>,
}
```

### WebhookManager

**Status**: 🆕 **New component needed**

**Purpose**: Orchestrates webhook operations with validation, idempotency, and secret management.

**Location**: `crates/repo_roller_core/src/webhook_manager.rs`

```rust
/// Manages webhook operations for repositories.
///
/// This manager provides high-level webhook management operations with
/// validation, idempotency checks, and proper error handling.
pub struct WebhookManager {
    /// GitHub client for API operations
    github_client: GitHubClient,
}

impl WebhookManager {
    /// Creates a new WebhookManager.
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Applies webhooks to a repository, creating or updating as needed.
    ///
    /// This method ensures that all webhooks from the configuration are present
    /// in the repository. It checks for existing webhooks to avoid duplicates.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `webhooks` - Vector of WebhookConfig
    ///
    /// # Returns
    /// `Ok(ApplyWebhooksResult)` with details of operations performed
    ///
    /// # Errors
    /// Returns `RepoRollerError::System` if webhook operations fail.
    ///
    /// # Behavior
    /// 1. Lists existing webhooks in repository
    /// 2. For each webhook in configuration:
    ///    - Checks if webhook with same URL already exists
    ///    - Creates if not exists
    ///    - Updates if exists but configuration differs
    /// 3. Logs each operation (created/updated/skipped)
    /// 4. Returns summary of operations
    ///
    /// # Error Handling
    /// - Continues on individual webhook failures (logs warning)
    /// - Returns error only if all webhooks fail
    /// - Partial success is considered success
    ///
    /// # Security
    /// Webhook secrets are handled securely and never logged.
    pub async fn apply_webhooks(
        &self,
        owner: &str,
        repo: &str,
        webhooks: &[WebhookConfig],
    ) -> Result<ApplyWebhooksResult, RepoRollerError>;

    /// Lists all webhooks currently configured in a repository.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// Vector of Webhook structs
    ///
    /// # Errors
    /// Returns `RepoRollerError::System` if API call fails.
    pub async fn list_webhooks(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Webhook>, RepoRollerError>;

    /// Validates webhook configuration before applying.
    ///
    /// # Arguments
    /// * `config` - Webhook configuration to validate
    ///
    /// # Returns
    /// `Ok(())` if valid
    ///
    /// # Errors
    /// Returns `RepoRollerError::Validation` if configuration is invalid.
    ///
    /// # Validation Rules
    /// - URL must be valid HTTPS URL
    /// - Events list must not be empty
    /// - Content type must be "json" or "form"
    /// - Secret (if provided) must meet minimum length requirements
    pub fn validate_webhook_config(
        &self,
        config: &WebhookConfig,
    ) -> Result<(), RepoRollerError>;
}

/// Result of applying webhooks to a repository.
#[derive(Debug, Clone)]
pub struct ApplyWebhooksResult {
    /// Number of webhooks created
    pub created: usize,

    /// Number of webhooks updated
    pub updated: usize,

    /// Number of webhooks that already existed with correct configuration
    pub skipped: usize,

    /// Number of webhooks that failed to apply
    pub failed: usize,

    /// URLs of webhooks that failed (for error reporting)
    pub failed_webhooks: Vec<String>,
}
```

## Integration with Repository Creation Workflow

Labels and webhooks are applied as part of the `apply_repository_configuration()` function in `crates/repo_roller_core/src/configuration.rs`.

**Current Implementation** (labels already integrated):

```rust
pub(crate) async fn apply_repository_configuration(
    installation_repo_client: &GitHubClient,
    owner: &str,
    repo_name: &str,
    merged_config: &config_manager::MergedConfiguration,
) -> RepoRollerResult<()> {
    // Labels are already applied via direct GitHubClient calls
    if !merged_config.labels.is_empty() {
        for (label_name, label_config) in &merged_config.labels {
            installation_repo_client
                .create_label(owner, repo_name, &label_config.name, ...)
                .await?;
        }
    }

    // Webhooks are stubbed
    if !merged_config.webhooks.is_empty() {
        warn!("Webhook creation not yet implemented");
    }

    // ... other configuration
}
```

**Proposed Enhanced Implementation** (using managers):

```rust
pub(crate) async fn apply_repository_configuration(
    installation_repo_client: &GitHubClient,
    owner: &str,
    repo_name: &str,
    merged_config: &config_manager::MergedConfiguration,
) -> RepoRollerResult<()> {
    // Apply labels using LabelManager
    if !merged_config.labels.is_empty() {
        let label_manager = LabelManager::new(installation_repo_client.clone());
        let result = label_manager
            .apply_labels(owner, repo_name, &merged_config.labels)
            .await
            .map_err(|e| {
                error!("Failed to apply labels: {}", e);
                e
            })?;

        info!(
            "Applied labels: {} created, {} updated, {} skipped, {} failed",
            result.created, result.updated, result.skipped, result.failed
        );
    }

    // Apply webhooks using WebhookManager
    if !merged_config.webhooks.is_empty() {
        let webhook_manager = WebhookManager::new(installation_repo_client.clone());
        let result = webhook_manager
            .apply_webhooks(owner, repo_name, &merged_config.webhooks)
            .await
            .map_err(|e| {
                error!("Failed to apply webhooks: {}", e);
                e
            })?;

        info!(
            "Applied webhooks: {} created, {} updated, {} skipped, {} failed",
            result.created, result.updated, result.skipped, result.failed
        );
    }

    // ... other configuration
}
```

## Error Handling

### Label Errors

- `Error::InvalidResponse` - GitHub API error creating/updating label
- `Error::ApiError` - GitHub service error
- `RepoRollerError::System` - Higher-level system error from manager

### Webhook Errors

- `Error::InvalidResponse` - GitHub API error creating/updating webhook
- `Error::NotFound` - Webhook ID not found (for update/delete)
- `Error::ApiError` - GitHub service error
- `RepoRollerError::Validation` - Webhook configuration invalid
- `RepoRollerError::System` - Higher-level system error from manager

### Error Handling Strategy

1. **Individual Operations**: Log warnings, continue with remaining items
2. **Complete Failure**: Return error only if ALL operations fail
3. **Partial Success**: Consider successful, log details in result
4. **Non-Fatal**: Label/webhook failures should NOT fail repository creation

## Security Considerations

### Webhook Secrets

- Secrets are stored in `WebhookConfig.secret` (Option<String>)
- Secrets must never be logged or included in error messages
- Secrets should be validated for minimum length (e.g., 20 characters)
- Consider integration with secret management services (Azure Key Vault, etc.)

### Logging

- Log label operations (name, color, description)
- Log webhook operations (URL, events) but NEVER secrets
- Use `Debug` trait carefully on types containing secrets

## Testing Strategy

### Unit Tests

- GitHubClient methods with wiremock for API responses
- Manager validation logic with mock GitHub clients
- Idempotency verification (multiple calls produce same result)
- Error handling (API failures, invalid configurations)

### Integration Tests

- Real GitHub API operations against glitchgrove test repositories
- End-to-end label creation/update flow
- End-to-end webhook creation/update flow
- Verification of labels/webhooks existence after operations
- Cleanup of test labels/webhooks after tests

### Test Patterns

```rust
#[tokio::test]
async fn test_apply_labels_idempotent() {
    // Apply labels twice, verify only created once
}

#[tokio::test]
async fn test_webhook_secret_not_logged() {
    // Verify secrets don't appear in logs
}

#[tokio::test]
async fn test_partial_label_failure_continues() {
    // Some labels fail, others succeed
}
```

## Documentation Requirements

### User Documentation

- How to configure labels in metadata repository
- How to configure webhooks in metadata repository
- Example label configurations for common scenarios
- Example webhook configurations for CI/CD integration
- Security best practices for webhook secrets
- Troubleshooting common label/webhook issues

### Developer Documentation

- Rustdoc for all public types and methods
- Examples in doc comments
- Implementation notes for idempotency
- GitHub API reference links
- Error handling patterns

## Implementation Notes

### Idempotency

- `create_label()` is already idempotent (updates if exists)
- `apply_webhooks()` checks existing before creating (URL-based deduplication)
- Safe to call multiple times without side effects

### Performance

- Batch operations where possible (list all, then create/update)
- Avoid unnecessary API calls (check existence before creating)
- Consider rate limiting for organizations with many repositories

### GitHub API Limitations

- Labels: No bulk create API, must iterate
- Webhooks: No bulk create API, must iterate
- Rate limiting: 5000 requests/hour for authenticated apps
- Webhook uniqueness: By URL (same URL = duplicate)

## Success Criteria

- ✅ All GitHubClient webhook methods implemented
- ✅ LabelManager and WebhookManager components created
- ✅ Integration with repository creation workflow complete
- ✅ Comprehensive unit tests with wiremock (90%+ coverage)
- ✅ Integration tests pass against real GitHub API
- ✅ Webhook secrets never logged or exposed
- ✅ Operations are idempotent and safe to retry
- ✅ Documentation complete with examples
