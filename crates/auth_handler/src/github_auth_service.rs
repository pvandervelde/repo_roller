//! GitHub App authentication service implementation
//!
//! Provides concrete implementation of `UserAuthenticationService` for GitHub App authentication.

use crate::{AuthError, AuthResult, UserAuthenticationService};
use async_trait::async_trait;
use github_client::{create_app_client, GitHubClient};
use secrecy::{ExposeSecret, SecretString};

/// GitHub App authentication service
///
/// Concrete implementation of `UserAuthenticationService` that handles GitHub App
/// authentication and token management.
///
/// # Examples
///
/// ```rust,no_run
/// use auth_handler::{GitHubAuthService, UserAuthenticationService};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let app_id = 12345;
/// let private_key = std::fs::read_to_string("app-key.pem")?;
///
/// let auth_service = GitHubAuthService::new(app_id, private_key);
///
/// // Get installation token for an organization
/// let token = auth_service.get_installation_token_for_org("my-org").await?;
/// println!("Got token: {} chars", token.len());
/// # Ok(())
/// # }
/// ```
pub struct GitHubAuthService {
    app_id: u64,
    /// PEM-encoded private key.  Wrapped in `SecretString` so that the value is
    /// zeroed on drop and cannot be printed via `Display` or `Debug`.
    private_key: SecretString,
}

impl GitHubAuthService {
    /// Create a new GitHub App authentication service
    ///
    /// # Parameters
    /// - `app_id`: GitHub App ID
    /// - `private_key`: GitHub App private key in PEM format
    ///
    /// # Returns
    /// New `GitHubAuthService` instance
    pub fn new(app_id: u64, private_key: impl Into<String>) -> Self {
        Self {
            app_id,
            private_key: SecretString::from(private_key.into()),
        }
    }
}

#[async_trait]
impl UserAuthenticationService for GitHubAuthService {
    async fn get_installation_token_for_org(&self, org_name: &str) -> AuthResult<String> {
        // Create app client using stored credentials
        let app_client = create_app_client(self.app_id, self.private_key.expose_secret())
            .await
            .map_err(|_e| AuthError::InvalidCredentials)?;

        let client = GitHubClient::new(app_client);

        // Get installation token for organization
        let token = client
            .get_installation_token_for_org(org_name)
            .await
            .map_err(|e| {
                AuthError::GitHubError(format!(
                    "Failed to get installation token for org '{}': {}",
                    org_name, e
                ))
            })?;

        Ok(token)
    }
}

impl std::fmt::Debug for GitHubAuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubAuthService")
            .field("app_id", &self.app_id)
            .field("private_key", &"<REDACTED>")
            .finish()
    }
}
