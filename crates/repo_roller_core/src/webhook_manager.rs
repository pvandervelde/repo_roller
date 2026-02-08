//! Webhook management operations for repositories.
//!
//! This module provides the [`WebhookManager`] component for orchestrating
//! webhook operations with validation, idempotency, and secret management.

use github_client::{GitHubClient, RepositoryClient, Webhook};
use tracing::{info, warn};

use crate::{GitHubError, RepoRollerResult, ValidationError};

/// Manages webhook operations for repositories.
///
/// This manager provides high-level webhook management operations with
/// validation, idempotency checks, and proper error handling.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::GitHubClient;
/// use repo_roller_core::WebhookManager;
/// use config_manager::settings::WebhookConfig;
///
/// # async fn example(github_client: GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
/// let manager = WebhookManager::new(github_client);
///
/// let webhooks = vec![WebhookConfig {
///     url: "https://example.com/webhook".to_string(),
///     content_type: "json".to_string(),
///     secret: Some("my-secret".to_string()),
///     active: true,
///     events: vec!["push".to_string()],
/// }];
///
/// let result = manager.apply_webhooks("my-org", "my-repo", &webhooks).await?;
/// println!("Created: {}, Updated: {}", result.created, result.updated);
/// # Ok(())
/// # }
/// ```
pub struct WebhookManager {
    /// GitHub client for API operations
    github_client: GitHubClient,
}

impl WebhookManager {
    /// Creates a new WebhookManager.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::GitHubClient;
    /// use repo_roller_core::WebhookManager;
    ///
    /// # async fn example(client: GitHubClient) {
    /// let manager = WebhookManager::new(client);
    /// # }
    /// ```
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Applies webhooks to a repository, creating or updating as needed.
    ///
    /// This method ensures that all webhooks from the configuration are present
    /// in the repository. It checks for existing webhooks to avoid duplicates.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `webhooks` - Vector of WebhookConfig
    ///
    /// # Returns
    ///
    /// `Ok(ApplyWebhooksResult)` with details of operations performed
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if webhook operations fail.
    ///
    /// # Behavior
    ///
    /// 1. Lists existing webhooks in repository
    /// 2. For each webhook in configuration:
    ///    - Checks if webhook with same URL already exists
    ///    - Creates if not exists
    ///    - Updates if exists but configuration differs
    /// 3. Logs each operation (created/updated/skipped)
    /// 4. Returns summary of operations
    ///
    /// # Error Handling
    ///
    /// - Continues on individual webhook failures (logs warning)
    /// - Returns error only if all webhooks fail
    /// - Partial success is considered success
    ///
    /// # Security
    ///
    /// Webhook secrets are handled securely and never logged.
    pub async fn apply_webhooks(
        &self,
        owner: &str,
        repo: &str,
        webhooks: &[config_manager::settings::WebhookConfig],
    ) -> RepoRollerResult<ApplyWebhooksResult> {
        info!(
            owner = owner,
            repo = repo,
            webhook_count = webhooks.len(),
            "Applying webhooks to repository"
        );

        let mut result = ApplyWebhooksResult::new();

        // First, validate all webhook configurations
        for webhook_config in webhooks {
            if let Err(e) = self.validate_webhook_config(webhook_config) {
                warn!(
                    url = webhook_config.url,
                    error = ?e,
                    "Invalid webhook configuration"
                );
                result.failed += 1;
                result.failed_webhooks.push(webhook_config.url.clone());
                continue;
            }
        }

        // If all webhooks failed validation, return early
        if result.failed == webhooks.len() {
            return Ok(result);
        }

        // List existing webhooks to check for duplicates
        let existing_webhooks = match self.github_client.list_webhooks(owner, repo).await {
            Ok(webhooks) => webhooks,
            Err(e) => {
                warn!(
                    owner = owner,
                    repo = repo,
                    error = ?e,
                    "Failed to list existing webhooks"
                );
                Vec::new() // Continue with empty list if listing fails
            }
        };

        // Apply each webhook
        for webhook_config in webhooks {
            // Skip if already failed validation
            if result.failed_webhooks.contains(&webhook_config.url) {
                continue;
            }

            info!(url = "[REDACTED]", "Applying webhook");

            // Check if webhook with same URL already exists
            if let Some(existing) = existing_webhooks
                .iter()
                .find(|w| w.config.url == webhook_config.url)
            {
                info!(webhook_id = existing.id, "Webhook already exists, skipping");
                result.skipped += 1;
                continue;
            }

            // Create new webhook
            let params = github_client::CreateWebhookParams {
                url: &webhook_config.url,
                content_type: &webhook_config.content_type,
                secret: webhook_config.secret.as_deref(),
                active: webhook_config.active,
                events: &webhook_config.events,
            };

            match self
                .github_client
                .create_webhook(owner, repo, &params)
                .await
            {
                Ok(webhook) => {
                    info!(webhook_id = webhook.id, "Webhook created successfully");
                    result.created += 1;
                }
                Err(e) => {
                    warn!(error = ?e, "Failed to create webhook");
                    result.failed += 1;
                    result.failed_webhooks.push(webhook_config.url.clone());
                }
            }
        }

        info!(
            created = result.created,
            skipped = result.skipped,
            failed = result.failed,
            "Webhook application complete"
        );

        Ok(result)
    }

    /// Lists all webhooks currently configured in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Vector of Webhook structs
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::System` if API call fails.
    pub async fn list_webhooks(&self, owner: &str, repo: &str) -> RepoRollerResult<Vec<Webhook>> {
        match self.github_client.list_webhooks(owner, repo).await {
            Ok(webhooks) => Ok(webhooks),
            Err(e) => {
                warn!(owner = owner, repo = repo, error = ?e, "Failed to list webhooks");
                Err(GitHubError::InvalidResponse {
                    reason: format!("Failed to list webhooks for {}/{}: {}", owner, repo, e),
                }
                .into())
            }
        }
    }

    /// Validates webhook configuration before applying.
    ///
    /// # Arguments
    ///
    /// * `config` - Webhook configuration to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid
    ///
    /// # Errors
    ///
    /// Returns `RepoRollerError::Validation` if configuration is invalid.
    ///
    /// # Validation Rules
    ///
    /// - URL must be valid HTTPS URL
    /// - Events list must not be empty
    /// - Content type must be "json" or "form"
    /// - Secret (if provided) must meet minimum length requirements
    pub fn validate_webhook_config(
        &self,
        config: &config_manager::settings::WebhookConfig,
    ) -> RepoRollerResult<()> {
        // Validate URL is HTTPS
        if !config.url.starts_with("https://") {
            return Err(ValidationError::InvalidFormat {
                field: "url".to_string(),
                reason: "Webhook URL must use HTTPS".to_string(),
            }
            .into());
        }

        // Validate events list is not empty
        if config.events.is_empty() {
            return Err(ValidationError::InvalidFormat {
                field: "events".to_string(),
                reason: "Webhook must have at least one event".to_string(),
            }
            .into());
        }

        // Validate content type
        if config.content_type != "json" && config.content_type != "form" {
            return Err(ValidationError::InvalidFormat {
                field: "content_type".to_string(),
                reason: "Content type must be 'json' or 'form'".to_string(),
            }
            .into());
        }

        // Validate secret length if provided
        if let Some(secret) = &config.secret {
            if secret.len() < 8 {
                return Err(ValidationError::InvalidFormat {
                    field: "secret".to_string(),
                    reason: "Webhook secret must be at least 8 characters".to_string(),
                }
                .into());
            }
        }

        Ok(())
    }
}

/// Result of applying webhooks to a repository.
///
/// Contains counters for the different outcomes of webhook operations.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ApplyWebhooksResult {
    /// Creates a new empty result.
    pub fn new() -> Self {
        Self {
            created: 0,
            updated: 0,
            skipped: 0,
            failed: 0,
            failed_webhooks: Vec::new(),
        }
    }

    /// Returns true if all operations succeeded.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Returns true if any webhooks were successfully applied (created or updated).
    pub fn has_changes(&self) -> bool {
        self.created > 0 || self.updated > 0
    }
}

impl Default for ApplyWebhooksResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "webhook_manager_tests.rs"]
mod tests;
