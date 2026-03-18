//! Authentication and Authorization handler for RepoRoller
//!
//! This crate provides centralized authentication and authorization services to eliminate
//! code duplication and security risks from ad-hoc implementations.
//!
//! ## Architecture
//!
//! This crate defines interface traits that infrastructure implements:
//! - Business logic depends on these traits
//! - Infrastructure (GitHub, database) implements the traits
//! - Main application wires everything together

use async_trait::async_trait;

mod github_auth_service;

pub use github_auth_service::GitHubAuthService;

/// Result type for authentication operations
pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Errors that can occur during authentication/authorization operations
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("GitHub API error: {0}")]
    GitHubError(String),

    #[error("Authentication error: {0}")]
    Other(String),
}

/// User authentication service interface
///
/// Handles user identity verification and session management.
#[async_trait]
pub trait UserAuthenticationService: Send + Sync {
    /// Get installation token for an organization
    ///
    /// # Parameters
    /// - `org_name`: Organization name
    ///
    /// # Returns
    /// Installation token with organization permissions
    ///
    /// # Errors
    /// Returns `AuthError::GitHubError` if GitHub API fails
    async fn get_installation_token_for_org(&self, org_name: &str) -> AuthResult<String>;
}
