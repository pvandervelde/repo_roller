// GENERATED FROM: docs/spec/interfaces/event-secrets.md
// Secret resolution abstraction for webhook signing secrets

use async_trait::async_trait;
use thiserror::Error;

/// Error type for secret resolution failures.
///
/// Security note: Does NOT include secret values in error messages.
///
/// See docs/spec/interfaces/event-secrets.md#secretresolutionerror
#[derive(Error, Debug, Clone)]
pub enum SecretResolutionError {
    #[error("Secret not found: {reference}")]
    NotFound { reference: String },

    #[error("Access denied to secret: {reference}")]
    AccessDenied { reference: String },

    #[error("Invalid secret reference format: {reference}")]
    InvalidFormat { reference: String, reason: String },

    #[error("Network error resolving secret: {message}")]
    NetworkError { message: String },

    #[error("Secret resolution error: {message}")]
    Other { message: String },
}

/// Abstraction for resolving webhook signing secrets.
///
/// Implementations provide access to secrets from various sources:
/// - Environment variables
/// - Filesystem (Kubernetes/Docker secrets)
/// - Azure Key Vault
/// - AWS Secrets Manager
///
/// # Security
/// - Secret values MUST NOT be logged
/// - Secret values MUST NOT be included in error messages
/// - Implementations MUST be thread-safe
///
/// See docs/spec/interfaces/event-secrets.md#secretresolver-trait
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
    /// - Secret values MUST NOT be logged
    /// - Errors should be sanitized to avoid leaking secret names/paths
    ///
    /// See docs/spec/interfaces/event-secrets.md#secretresolver-trait
    async fn resolve_secret(&self, secret_ref: &str) -> Result<String, SecretResolutionError>;
}

/// Resolves secrets from environment variables.
///
/// Use case: Local development, simple deployments, container environments.
///
/// # Example
/// ```no_run
/// use repo_roller_core::event_secrets::{SecretResolver, EnvironmentSecretResolver};
///
/// # async fn example() {
/// let resolver = EnvironmentSecretResolver::new();
/// let secret = resolver.resolve_secret("WEBHOOK_SECRET_PROD").await.unwrap();
/// # }
/// ```
///
/// See docs/spec/interfaces/event-secrets.md#environmentsecretresolver
pub struct EnvironmentSecretResolver;

impl EnvironmentSecretResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnvironmentSecretResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretResolver for EnvironmentSecretResolver {
    async fn resolve_secret(&self, _secret_ref: &str) -> Result<String, SecretResolutionError> {
        unimplemented!("See docs/spec/interfaces/event-secrets.md#environmentsecretresolver")
    }
}

/// Resolves secrets from filesystem paths (volume mounts).
///
/// Use case: Kubernetes secrets, Docker secrets.
///
/// # Example
/// ```no_run
/// use repo_roller_core::event_secrets::{SecretResolver, FilesystemSecretResolver};
///
/// # async fn example() {
/// let resolver = FilesystemSecretResolver::new("/secrets");
/// let secret = resolver.resolve_secret("webhook-prod").await.unwrap();
/// // Reads from /secrets/webhook-prod
/// # }
/// ```
///
/// See docs/spec/interfaces/event-secrets.md#filesystemsecretresolver
pub struct FilesystemSecretResolver {
    base_path: std::path::PathBuf,
}

impl FilesystemSecretResolver {
    /// Creates a new filesystem secret resolver.
    ///
    /// # Arguments
    /// * `base_path` - Directory containing secret files
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

#[async_trait]
impl SecretResolver for FilesystemSecretResolver {
    async fn resolve_secret(&self, _secret_ref: &str) -> Result<String, SecretResolutionError> {
        unimplemented!("See docs/spec/interfaces/event-secrets.md#filesystemsecretresolver")
    }
}

#[cfg(test)]
#[path = "event_secrets_tests.rs"]
mod tests;
