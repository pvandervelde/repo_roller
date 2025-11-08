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
//!
//! See specs/interfaces/authentication-interfaces.md for complete specifications.

use async_trait::async_trait;

mod github_auth_service;

pub use github_auth_service::GitHubAuthService;

/// Result type for authentication operations
pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Errors that can occur during authentication/authorization operations
///
/// TODO: Implement complete error hierarchy from specs/interfaces/error-types.md
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
/// See specs/interfaces/authentication-interfaces.md#userauthenticationservice
///
/// TODO: Implement this trait for GitHub App authentication
#[async_trait]
pub trait UserAuthenticationService: Send + Sync {
    /// Authenticate a GitHub App installation and get an installation token
    ///
    /// # Parameters
    /// - `app_id`: GitHub App ID
    /// - `private_key`: GitHub App private key (PEM format)
    /// - `installation_id`: Installation ID for the organization
    ///
    /// # Returns
    /// Installation token for API operations
    ///
    /// # Errors
    /// Returns `AuthError::InvalidCredentials` if authentication fails
    async fn authenticate_installation(
        &self,
        app_id: u64,
        private_key: &str,
        installation_id: u64,
    ) -> AuthResult<String>;

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

/// Organization permission checking service interface
///
/// Validates user permissions for organization-level operations.
/// See specs/interfaces/authentication-interfaces.md#organizationpermissionservice
///
/// TODO: Implement this trait using GitHub API permissions checks
#[async_trait]
pub trait OrganizationPermissionService: Send + Sync {
    /// Check if a user has permission to create repositories in an organization
    ///
    /// # Parameters
    /// - `user_id`: User identifier
    /// - `org_name`: Organization name
    ///
    /// # Returns
    /// `true` if user has repository creation permission
    ///
    /// # Errors
    /// Returns `AuthError::GitHubError` if permission check fails
    async fn can_create_repository(&self, user_id: &str, org_name: &str) -> AuthResult<bool>;

    /// Check if a user is an admin of an organization
    ///
    /// # Parameters
    /// - `user_id`: User identifier
    /// - `org_name`: Organization name
    ///
    /// # Returns
    /// `true` if user is an organization admin
    ///
    /// # Errors
    /// Returns `AuthError::GitHubError` if permission check fails
    async fn is_organization_admin(&self, user_id: &str, org_name: &str) -> AuthResult<bool>;

    /// Get user's role in an organization
    ///
    /// # Parameters
    /// - `user_id`: User identifier
    /// - `org_name`: Organization name
    ///
    /// # Returns
    /// User's role (e.g., "admin", "member", "none")
    ///
    /// # Errors
    /// Returns `AuthError::GitHubError` if role lookup fails
    async fn get_organization_role(&self, user_id: &str, org_name: &str) -> AuthResult<String>;
}
