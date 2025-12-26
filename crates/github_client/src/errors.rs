//! Error types for GitHub client operations.
//!
//! This module defines the error types that can occur when interacting with the GitHub API
//! through the github_client crate. It provides comprehensive error context for debugging
//! and error handling in applications using this client.

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// Errors that can occur during GitHub client operations.
///
/// This enum represents all possible error conditions when working with the GitHub API,
/// including authentication failures, API errors, rate limiting, and data processing issues.
/// Each variant provides specific context about what went wrong and relevant details
/// for debugging and error handling.
///
/// ## Examples
///
/// ```rust,ignore
/// use github_client::Error;
///
/// // Handle different error types
/// match github_client.create_repository(&payload).await {
///     Ok(repo) => println!("Repository created: {}", repo.name()),
///     Err(Error::AuthError(msg)) => eprintln!("Authentication failed: {}", msg),
///     Err(Error::RateLimitExceeded) => eprintln!("Rate limit exceeded, retry later"),
///     Err(err) => eprintln!("Other error: {}", err),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A generic API request failure.
    ///
    /// This error occurs when a GitHub API request fails for unspecified reasons.
    /// Check the GitHub API status and ensure your request parameters are correct.
    #[error("API request failed")]
    ApiError(),

    /// Authentication or GitHub client initialization failure.
    ///
    /// This error occurs when:
    /// - GitHub App credentials are invalid or expired
    /// - Network connectivity issues prevent authentication
    /// - The GitHub App lacks necessary permissions
    ///
    /// The contained string provides specific details about the authentication failure.
    #[error("Failed to authenticate or initialize GitHub client: {0}")]
    AuthError(String),

    /// Error deserializing the response from GitHub.
    ///
    /// This error occurs when the GitHub API returns a response that cannot be
    /// parsed into the expected data structure. This may indicate:
    /// - API version changes
    /// - Unexpected response format
    /// - Corrupted response data
    #[error("Failed to deserialize GitHub response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// Failed to create an app access token for a specific repository.
    ///
    /// This error occurs when the GitHub App cannot obtain an installation token
    /// for the specified repository. Common causes include:
    /// - The app is not installed on the repository/organization
    /// - The app lacks necessary permissions
    /// - The repository doesn't exist or is not accessible
    ///
    /// Parameters: (owner, repository, app_id)
    #[error("Failed to create an app access token for repository: {0}/{1}. For app with ID: {2}")]
    FailedToCreateAccessToken(String, String, u64),

    /// Failed to find app installation for a specific repository.
    ///
    /// This error occurs when the GitHub App installation cannot be found for
    /// the specified repository. This typically means:
    /// - The app is not installed on the target repository or organization
    /// - The installation was removed or suspended
    /// - The repository has been moved or deleted
    ///
    /// Parameters: (owner, repository, installation_id)
    #[error("Failed to find installation for repository: {0}/{1} with ID: {2}")]
    FailedToFindAppInstallation(String, String, u64),

    /// The GitHub API returned a response in an unexpected format.
    ///
    /// This error indicates that the API response structure doesn't match
    /// what the client expects. This may occur due to:
    /// - GitHub API changes or deprecations
    /// - Client library being out of date
    /// - Malformed API responses
    #[error("Invalid response format")]
    InvalidResponse,

    /// The requested resource was not found.
    ///
    /// This error occurs when a GitHub API request returns a 404 status code,
    /// indicating that the requested resource (repository, file, directory, etc.)
    /// does not exist or is not accessible with the current authentication.
    #[error("Resource not found")]
    NotFound,

    /// GitHub API rate limit has been exceeded.
    ///
    /// This error occurs when the client has made too many requests in a given
    /// time window. The client should implement exponential backoff and retry
    /// logic when encountering this error. Check the `X-RateLimit-Reset` header
    /// in the response to determine when to retry.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
