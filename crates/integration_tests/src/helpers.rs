//! Helper functions for integration tests.
//!
//! This module provides common patterns and utilities for writing integration tests,
//! including setup, teardown, assertions, and test data creation.

use anyhow::{Context, Result};
use github_client::GitHubClient;
use repo_roller_core::{OrganizationName, RepositoryName, TemplateName};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

/// Retry configuration for flaky operations.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// Maximum delay between retries.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
        }
    }
}

/// Retry an async operation with exponential backoff.
///
/// This is useful for operations that may fail transiently, such as
/// GitHub API calls that may hit rate limits or network issues.
///
/// # Example
///
/// ```no_run
/// use integration_tests::helpers::{retry_with_backoff, RetryConfig};
/// use anyhow::anyhow;
///
/// # async fn example() -> anyhow::Result<()> {
/// let result = retry_with_backoff(
///     RetryConfig::default(),
///     || async {
///         // Operation that might fail
///         Ok::<i32, anyhow::Error>(42)
///     }
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(config: RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    let mut last_error: Option<String> = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(value) => {
                if attempt > 1 {
                    info!(
                        attempt = attempt,
                        total_attempts = config.max_attempts,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(value);
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                last_error = Some(error_msg.clone());

                if attempt < config.max_attempts {
                    debug!(
                        attempt = attempt,
                        total_attempts = config.max_attempts,
                        delay_ms = delay.as_millis(),
                        error = error_msg,
                        "Operation failed, retrying after delay"
                    );

                    sleep(delay).await;

                    // Exponential backoff
                    delay = Duration::from_millis(
                        ((delay.as_millis() as f64) * config.backoff_multiplier) as u64,
                    );
                    if delay > config.max_delay {
                        delay = config.max_delay;
                    }
                } else {
                    debug!(
                        attempt = attempt,
                        error = error_msg,
                        "Operation failed on final attempt"
                    );
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Operation failed after {} attempts. Last error: {}",
        config.max_attempts,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}

/// Wait for a repository to be accessible via GitHub API.
///
/// This is useful after creating a repository, as there may be a slight
/// delay before the repository is fully available via the API.
pub async fn wait_for_repository_available(
    client: &GitHubClient,
    owner: &str,
    repo_name: &str,
    timeout: Duration,
) -> Result<()> {
    let start = tokio::time::Instant::now();
    let check_interval = Duration::from_millis(500);

    loop {
        match client.get_repository(owner, repo_name).await {
            Ok(_) => {
                info!(
                    owner = owner,
                    repo_name = repo_name,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Repository is available"
                );
                return Ok(());
            }
            Err(e) => {
                if start.elapsed() >= timeout {
                    return Err(anyhow::anyhow!(
                        "Repository {}/{} not available after {}s: {}",
                        owner,
                        repo_name,
                        timeout.as_secs(),
                        e
                    ));
                }

                debug!(
                    owner = owner,
                    repo_name = repo_name,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Repository not yet available, retrying"
                );

                sleep(check_interval).await;
            }
        }
    }
}

/// Assert that an error message contains expected keywords.
///
/// This is useful for validating error messages are helpful and contain
/// relevant information for debugging and user guidance.
pub fn assert_error_contains(error: &anyhow::Error, expected_keywords: &[&str]) {
    let error_msg = error.to_string().to_lowercase();

    for keyword in expected_keywords {
        assert!(
            error_msg.contains(&keyword.to_lowercase()),
            "Error message should contain '{}'. Got: {}",
            keyword,
            error
        );
    }
}

/// Assert that an error message does NOT contain sensitive patterns.
///
/// This is critical for security - errors should never leak secrets,
/// tokens, or private keys.
pub fn assert_error_no_secrets(error: &anyhow::Error, forbidden_patterns: &[&str]) {
    let error_msg = error.to_string();

    for pattern in forbidden_patterns {
        assert!(
            !error_msg.contains(pattern),
            "Error message should not contain sensitive pattern '{}'. Got: {}",
            pattern,
            error
        );
    }
}

/// Create a test repository name with timestamp and random suffix.
///
/// This ensures unique repository names for concurrent test execution.
pub fn create_test_repo_name(test_name: &str) -> Result<RepositoryName> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let random_suffix = uuid::Uuid::new_v4().simple().to_string()[..6].to_lowercase();
    let name = format!("test-{}-{}-{}", test_name, timestamp, random_suffix);

    RepositoryName::new(&name).context("Failed to create test repository name")
}

/// Create test organization name from environment variable.
pub fn create_test_org_name() -> Result<OrganizationName> {
    let org = std::env::var("TEST_ORG").context("TEST_ORG environment variable not set")?;
    OrganizationName::new(&org).context("Failed to create organization name")
}

/// Create test template name.
pub fn create_test_template_name(template: &str) -> Result<TemplateName> {
    TemplateName::new(template).context("Failed to create template name")
}

/// Verify that a collection contains all expected items.
///
/// This is useful for verifying label merging, webhook accumulation, etc.
pub fn assert_contains_all<T: PartialEq + std::fmt::Debug>(
    collection: &[T],
    expected: &[T],
    collection_name: &str,
) {
    for item in expected {
        assert!(
            collection.contains(item),
            "{} should contain {:?}. Got: {:?}",
            collection_name,
            item,
            collection
        );
    }
}

/// Verify that a collection does NOT contain any forbidden items.
pub fn assert_contains_none<T: PartialEq + std::fmt::Debug>(
    collection: &[T],
    forbidden: &[T],
    collection_name: &str,
) {
    for item in forbidden {
        assert!(
            !collection.contains(item),
            "{} should not contain {:?}. Got: {:?}",
            collection_name,
            item,
            collection
        );
    }
}

/// Compare two values with a detailed diff on failure.
///
/// This provides better error messages when assertions fail.
pub fn assert_eq_detailed<T: PartialEq + std::fmt::Debug>(actual: &T, expected: &T, context: &str) {
    if actual != expected {
        panic!(
            "{} mismatch:\nExpected: {:#?}\nActual: {:#?}",
            context, expected, actual
        );
    }
}

/// Sleep for a short duration to allow for eventual consistency.
///
/// Some operations (like GitHub webhooks) may take a moment to propagate.
pub async fn wait_for_eventual_consistency() {
    sleep(Duration::from_millis(500)).await;
}

/// Create a temporary directory for test files.
pub fn create_temp_dir(prefix: &str) -> Result<tempfile::TempDir> {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .context("Failed to create temporary directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_repo_name() {
        let name = create_test_repo_name("basic").unwrap();
        let name_str = name.as_str();
        assert!(name_str.starts_with("test-basic-"));
        assert!(name_str.len() > 20); // Should include timestamp and random suffix
    }

    #[test]
    fn test_assert_error_contains() {
        let error = anyhow::anyhow!("Repository not found in organization");
        assert_error_contains(&error, &["repository", "not found"]);
    }

    #[test]
    #[should_panic(expected = "should contain 'missing'")]
    fn test_assert_error_contains_fails_when_missing() {
        let error = anyhow::anyhow!("Something went wrong");
        assert_error_contains(&error, &["missing"]);
    }

    #[test]
    fn test_assert_error_no_secrets() {
        let error = anyhow::anyhow!("Authentication failed");
        assert_error_no_secrets(&error, &["password", "secret", "token="]);
    }

    #[test]
    #[should_panic(expected = "should not contain")]
    fn test_assert_error_no_secrets_fails_when_present() {
        let error = anyhow::anyhow!("Failed with token=abc123");
        assert_error_no_secrets(&error, &["token="]);
    }

    #[test]
    fn test_assert_contains_all() {
        let collection = vec![1, 2, 3, 4, 5];
        assert_contains_all(&collection, &[1, 3, 5], "numbers");
    }

    #[test]
    #[should_panic(expected = "should contain")]
    fn test_assert_contains_all_fails_when_missing() {
        let collection = vec![1, 2, 3];
        assert_contains_all(&collection, &[1, 4], "numbers");
    }

    #[test]
    fn test_assert_contains_none() {
        let collection = vec![1, 2, 3];
        assert_contains_none(&collection, &[4, 5, 6], "numbers");
    }

    #[test]
    #[should_panic(expected = "should not contain")]
    fn test_assert_contains_none_fails_when_present() {
        let collection = vec![1, 2, 3];
        assert_contains_none(&collection, &[2, 4], "numbers");
    }

    #[tokio::test]
    async fn test_retry_with_backoff_succeeds_immediately() {
        let result = retry_with_backoff(RetryConfig::default(), || async { Ok::<_, String>(42) })
            .await
            .unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_succeeds_after_retries() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(10),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_millis(100),
            },
            move || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    if count < 3 {
                        Err("Temporary failure")
                    } else {
                        Ok(42)
                    }
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_fails_after_max_attempts() {
        let result = retry_with_backoff(
            RetryConfig {
                max_attempts: 2,
                initial_delay: Duration::from_millis(10),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_millis(100),
            },
            || async { Err::<i32, _>("Permanent failure") },
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed after 2 attempts"));
    }
}
