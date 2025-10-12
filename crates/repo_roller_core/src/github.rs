//! GitHub integration types
//!
//! Types representing GitHub-specific concepts.
//! See specs/interfaces/shared-types.md for complete specifications.

use serde::{Deserialize, Serialize};

use crate::errors::ValidationError;

/// GitHub App installation ID
///
/// Represents a GitHub App installation identifier.
/// See specs/interfaces/shared-types.md#installationid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstallationId(u64);

impl InstallationId {
    /// Create a new installation ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the installation ID as a u64
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for InstallationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for InstallationId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

/// GitHub access token (secure, not logged)
///
/// Represents a GitHub API token with security measures.
/// See specs/interfaces/shared-types.md#githubtoken
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubToken(String);

impl GitHubToken {
    /// Create a new GitHub token with validation
    ///
    /// # Validation Rules
    /// - Must not be empty
    /// - Minimum length: 10 characters
    ///
    /// # Errors
    /// Returns `ValidationError` if validation fails
    pub fn new(token: impl Into<String>) -> Result<Self, ValidationError> {
        let token = token.into();

        if token.is_empty() {
            return Err(ValidationError::empty_field("github_token"));
        }

        if token.len() < 10 {
            return Err(ValidationError::too_short("github_token", token.len(), 10));
        }

        Ok(Self(token))
    }

    /// Get the token as a string slice
    ///
    /// # Security
    /// Use with caution - prefer passing the GitHubToken itself
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the token length (for logging without exposing value)
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if token is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// Security: Don't log the actual token value
impl std::fmt::Debug for GitHubToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GitHubToken([REDACTED {} chars])", self.0.len())
    }
}

impl std::fmt::Display for GitHubToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_id() {
        let id = InstallationId::new(12345);
        assert_eq!(id.as_u64(), 12345);
        assert_eq!(id.to_string(), "12345");
    }

    #[test]
    fn test_github_token_valid() {
        assert!(GitHubToken::new("ghp_1234567890").is_ok());
    }

    #[test]
    fn test_github_token_invalid() {
        assert!(GitHubToken::new("").is_err());
        assert!(GitHubToken::new("short").is_err());
    }

    #[test]
    fn test_github_token_security() {
        let token = GitHubToken::new("ghp_secret_token_value").unwrap();
        let debug_output = format!("{:?}", token);
        assert!(!debug_output.contains("secret"));
        assert!(debug_output.contains("REDACTED"));

        let display_output = format!("{}", token);
        assert_eq!(display_output, "[REDACTED]");
    }
}
