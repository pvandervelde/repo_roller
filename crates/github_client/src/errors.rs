#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed")]
    ApiError(),

    #[error("Failed to authenticate or initialize GitHub client: {0}")]
    AuthError(String),

    /// Error deserializing the response from GitHub.
    #[error("Failed to deserialize GitHub response: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Failed to create an app access token for repository: {0}/{1}. For app with ID: {2}")]
    FailedToCreateAccessToken(String, String, u64),

    #[error("Failed to find installation for repository: {0}/{1} with ID: {2}")]
    FailedToFindAppInstallation(String, String, u64),

    #[error("Invalid response format")]
    InvalidResponse,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
