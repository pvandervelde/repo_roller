# Event Secret Resolution Interface

**Architectural Layer**: Infrastructure Abstraction
**Module Path**: `repo_roller_core/src/event_secrets.rs`
**Specification**: [outbound-event-notifications.md](../design/outbound-event-notifications.md#ad-4-container-native-secret-management)

## Overview

The SecretResolver trait defines the interface for resolving webhook signing secrets from container-native secret stores. This abstraction allows the business logic to remain independent of specific secret storage mechanisms (environment variables, Kubernetes secrets, Azure Key Vault, AWS Secrets Manager, etc.).

## Responsibilities (RDD)

**Knows**:

- Secret reference format (names/keys)
- Secret resolution strategies

**Does**:

- Resolves secret references to actual secret values
- Abstracts secret storage mechanism from business logic

**Collaborates With**:

- EventPublisher (consumer of resolved secrets)
- Container runtime / cloud platform (secret source)

## Dependencies

### External Dependencies

- Implementation-specific (varies by resolver)
- `async-trait` for trait definition
- `thiserror` for error types

## Public Interface

### SecretResolver Trait

**Purpose**: Abstraction for resolving webhook signing secrets.

**Signature**:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SecretResolver: Send + Sync {
    /// Resolves a secret reference to its actual value.
    ///
    /// # Arguments
    /// * `secret_ref` - Secret identifier (e.g., env var name, secret path)
    ///
    /// # Returns
    /// * `Ok(String)` - Resolved secret value
    /// * `Err(SecretResolutionError)` - Resolution failed
    ///
    /// # Security
    /// - Secret values MUST NOT be logged or included in error messages
    /// - Errors should be sanitized to avoid leaking secret names/paths
    async fn resolve_secret(&self, secret_ref: &str) -> Result<String, SecretResolutionError>;
}
```

**Contract**:

- MUST return actual secret value (not reference)
- MUST NOT log secret values
- MUST NOT include secret values in error messages
- MUST be thread-safe (Send + Sync)
- MUST be async (supports network/IO operations)

**Error Conditions**:

- `SecretResolutionError::NotFound` - Secret reference not found
- `SecretResolutionError::AccessDenied` - Permission denied
- `SecretResolutionError::InvalidFormat` - Secret reference malformed
- `SecretResolutionError::NetworkError` - Network failure (cloud APIs)

### SecretResolutionError

**Purpose**: Error type for secret resolution failures.

**Signature**:

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SecretResolutionError {
    #[error("Secret not found: {reference}")]
    NotFound {
        reference: String,
    },

    #[error("Access denied to secret: {reference}")]
    AccessDenied {
        reference: String,
    },

    #[error("Invalid secret reference format: {reference}")]
    InvalidFormat {
        reference: String,
        reason: String,
    },

    #[error("Network error resolving secret: {message}")]
    NetworkError {
        message: String,
    },

    #[error("Secret resolution error: {message}")]
    Other {
        message: String,
    },
}
```

**Behavior**:

- Does NOT include secret values
- Includes sanitized error context
- Loggable without security concerns

## Implementations

### EnvironmentSecretResolver

**Purpose**: Resolves secrets from environment variables.

**Use Case**: Local development, simple deployments, container environments

**Behavior**:

- `secret_ref` is environment variable name
- Reads from `std::env::var()`
- Returns `NotFound` if variable not set
- No network calls (synchronous internally, async wrapper)

**Example**:

```rust
let resolver = EnvironmentSecretResolver::new();
let secret = resolver.resolve_secret("WEBHOOK_SECRET_PROD").await?;
```

**Configuration**:

```toml
# In notifications.toml
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "WEBHOOK_SECRET_PROD"  # Environment variable name
```

**Deployment**:

```yaml
# Docker
environment:
  - WEBHOOK_SECRET_PROD=actual-secret-value

# Kubernetes
env:
  - name: WEBHOOK_SECRET_PROD
    valueFrom:
      secretKeyRef:
        name: reporoller-secrets
        key: webhook-prod
```

### FilesystemSecretResolver

**Purpose**: Resolves secrets from filesystem paths (volume mounts).

**Use Case**: Kubernetes secrets, Docker secrets

**Behavior**:

- `secret_ref` is file path
- Reads file content as secret
- Trims whitespace/newlines
- Returns `NotFound` if file doesn't exist
- Returns `AccessDenied` if permission error

**Example**:

```rust
let resolver = FilesystemSecretResolver::new("/secrets");
let secret = resolver.resolve_secret("webhook-prod").await?;
// Reads from /secrets/webhook-prod
```

**Configuration**:

```toml
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "webhook-prod"  # File name in mounted directory
```

**Deployment**:

```yaml
# Kubernetes
volumeMounts:
  - name: webhook-secrets
    mountPath: /secrets
    readOnly: true
volumes:
  - name: webhook-secrets
    secret:
      secretName: reporoller-webhook-secrets

# Docker Swarm
secrets:
  - source: webhook-prod
    target: /secrets/webhook-prod
```

### AzureKeyVaultResolver (Future)

**Purpose**: Resolves secrets from Azure Key Vault via Managed Identity.

**Use Case**: Azure Container Apps, Azure Functions, AKS with workload identity

**Behavior**:

- `secret_ref` format: `secret-name` or `vault/secret-name`
- Authenticates via Azure Managed Identity
- Calls Azure Key Vault REST API
- Caches secrets with TTL
- Returns `NetworkError` if API fails

**Example**:

```rust
let resolver = AzureKeyVaultResolver::new("https://myvault.vault.azure.net");
let secret = resolver.resolve_secret("webhook-prod").await?;
```

**Configuration**:

```toml
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "webhook-prod"  # Key Vault secret name
```

**Deployment**:

```yaml
# Azure Container Apps
identity:
  type: SystemAssigned
secretStoreComponent: azurekeyvault
```

### AwsSecretsManagerResolver (Future)

**Purpose**: Resolves secrets from AWS Secrets Manager via IAM role.

**Use Case**: AWS ECS, EKS with IAM roles for service accounts

**Behavior**:

- `secret_ref` format: `secret-name` or ARN
- Authenticates via IAM task role
- Calls AWS Secrets Manager API
- Caches secrets with TTL
- Returns `NetworkError` if API fails

**Example**:

```rust
let resolver = AwsSecretsManagerResolver::new("us-east-1");
let secret = resolver.resolve_secret("webhook-prod").await?;
```

**Configuration**:

```toml
[[outbound_webhooks]]
url = "https://example.com/webhook"
secret = "webhook-prod"  # Secrets Manager secret name
```

**Deployment**:

```json
// AWS ECS
{
  "taskRoleArn": "arn:aws:iam::account:role/reporoller-task-role",
  "secrets": [
    {
      "name": "WEBHOOK_SECRET_PROD",
      "valueFrom": "arn:aws:secretsmanager:region:account:secret:webhook-prod"
    }
  ]
}
```

## Integration Pattern

### Initialization

The application selects an appropriate resolver at startup:

```rust
// In main.rs or application initialization

let secret_resolver: Arc<dyn SecretResolver> = if cfg!(feature = "azure-keyvault") {
    Arc::new(AzureKeyVaultResolver::new(vault_url))
} else if cfg!(feature = "aws-secrets") {
    Arc::new(AwsSecretsManagerResolver::new(region))
} else if Path::new("/secrets").exists() {
    Arc::new(FilesystemSecretResolver::new("/secrets"))
} else {
    Arc::new(EnvironmentSecretResolver::new())
};
```

### Usage in EventPublisher

```rust
// In event_publisher.rs

async fn deliver_to_endpoint(
    endpoint: &NotificationEndpoint,
    secret_resolver: &dyn SecretResolver,
    // ... other params
) -> DeliveryResult {
    // Resolve secret
    let secret = match secret_resolver.resolve_secret(&endpoint.secret).await {
        Ok(s) => s,
        Err(e) => {
            warn!(
                endpoint_url = %endpoint.url,
                error = %e,  // Sanitized, no secret value
                "Failed to resolve webhook secret"
            );
            return DeliveryResult {
                success: false,
                error_message: Some("Secret resolution failed".to_string()),
                // ...
            };
        }
    };

    // Use secret for signing
    let signed_request = sign_webhook_request(request, payload, &secret);

    // ... continue with delivery
}
```

## Testing Strategy

### Unit Tests

**EnvironmentSecretResolver**:

- ✅ Resolves existing environment variable
- ✅ Returns NotFound for missing variable
- ✅ Handles empty values
- ✅ Thread-safe concurrent access

**FilesystemSecretResolver**:

- ✅ Resolves secret from file
- ✅ Trims whitespace/newlines
- ✅ Returns NotFound for missing file
- ✅ Returns AccessDenied for permission errors
- ✅ Handles base path correctly

### Integration Tests

**Mock SecretResolver**:

```rust
pub struct MockSecretResolver {
    secrets: HashMap<String, String>,
}

impl MockSecretResolver {
    pub fn with_secrets(secrets: HashMap<String, String>) -> Self {
        Self { secrets }
    }
}

#[async_trait]
impl SecretResolver for MockSecretResolver {
    async fn resolve_secret(&self, secret_ref: &str) -> Result<String, SecretResolutionError> {
        self.secrets
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| SecretResolutionError::NotFound {
                reference: secret_ref.to_string(),
            })
    }
}
```

**Usage in Tests**:

```rust
#[tokio::test]
async fn test_event_delivery_with_secret_resolution() {
    let resolver = MockSecretResolver::with_secrets(
        [("test-secret".to_string(), "secret-value".to_string())]
            .iter()
            .cloned()
            .collect(),
    );

    let result = publish_repository_created(
        &result,
        &request,
        &config,
        "user",
        &config_manager,
        &resolver,
        &metrics,
    ).await;

    // Verify delivery used resolved secret
}
```

## Security Considerations

### Secret Handling

**MUST**:

- Never log secret values
- Never include secret values in error messages
- Use constant-time comparison for signature verification
- Clear secrets from memory when done (drop)
- Use secure connections for network-based resolvers

**MUST NOT**:

- Store secrets in configuration files
- Log secret references in production
- Include secrets in stack traces
- Return secrets in API responses

### Error Messages

**Good** (sanitized):

```
"Failed to resolve secret: WEBHOOK_SECRET_PROD not found"
"Access denied to secret: webhook-prod"
```

**Bad** (leaks information):

```
"Secret value 'abc123...' is invalid"
"Failed to resolve secret from /secrets/webhook-prod: abc123..."
```

### Resolver Selection

Choose resolver based on deployment environment:

- **Local dev**: EnvironmentSecretResolver
- **Docker**: FilesystemSecretResolver with volume mounts
- **Kubernetes**: FilesystemSecretResolver with secret volumes
- **Azure**: AzureKeyVaultResolver with managed identity
- **AWS**: AwsSecretsManagerResolver with IAM roles

## Performance Considerations

**Caching**:

- Network-based resolvers (Azure, AWS) SHOULD cache secrets
- Cache TTL: 5-15 minutes (balance freshness vs. API calls)
- Environment/filesystem resolvers: no caching needed (OS handles it)

**Latency**:

- Environment variables: < 1ms
- Filesystem: < 10ms
- Network (Azure/AWS): 50-200ms (first call), < 1ms (cached)

**Resource Usage**:

- No persistent connections needed
- Minimal memory overhead
- Stateless (except caching)

## Future Enhancements

Out of scope for initial implementation:

- Secret rotation handling
- Secret versioning support
- Fallback resolver chain
- Secret pre-warming/prefetch
- Circuit breaker for network resolvers

## References

- [Event Publisher Interface](event-publisher.md)
- [Design Document](../design/outbound-event-notifications.md#ad-4-container-native-secret-management)
- [Vocabulary](../vocabulary.md#event-publishing-domain)
