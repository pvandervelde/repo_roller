//! GitHub App authentication operations.
//!
//! This module handles GitHub App authentication and token management for
//! repository operations.
//!
//! ## Overview
//!
//! GitHub App authentication follows a two-step process:
//! 1. **App Client Creation**: Authenticate as the GitHub App using app ID and private key
//! 2. **Installation Token Retrieval**: Get an installation-specific token for the organization
//!
//! ## Authentication Flow
//!
//! The [`setup_github_authentication`] function orchestrates the complete authentication
//! workflow:
//! 1. Creates a GitHub App client with the app's private key
//! 2. Retrieves the installation token for the target organization
//! 3. Creates a new client authenticated with the installation token
//! 4. Returns both the token (for git operations) and the client (for API operations)
//!
//! ## Installation Tokens
//!
//! Installation tokens provide:
//! - Organization-scoped access
//! - Limited lifetime (typically 1 hour)
//! - Specific permissions granted during app installation
//! - Audit trail for app operations
//!
//! ## Error Handling
//!
//! Authentication can fail for several reasons:
//! - Invalid app credentials (ID or private key)
//! - App not installed on the organization
//! - Network connectivity issues
//! - Insufficient permissions
//!
//! ## Examples
//!
//! ```rust,ignore
//! let (token, client) = setup_github_authentication(
//!     12345,
//!     "-----BEGIN RSA PRIVATE KEY-----...",
//!     "my-organization"
//! ).await?;
//!
//! // Use token for git operations
//! git::push_to_origin(&repo, url, "main", &token)?;
//!
//! // Use client for GitHub API operations
//! client.create_org_repository("my-org", &payload).await?;
//! ```

use crate::errors::{GitHubError, RepoRollerError, RepoRollerResult, SystemError};
use github_client::{create_app_client, GitHubClient};
use tracing::{error, info};

#[cfg(test)]
#[path = "github_auth_tests.rs"]
mod tests;

/// Setup GitHub App authentication and retrieve installation token.
///
/// This function orchestrates the complete GitHub App authentication workflow,
/// creating both an app-level client and an installation-specific client with
/// the appropriate token for repository operations.
///
/// ## Authentication Process
///
/// 1. **App Client Creation**: Uses the app ID and private key to create a GitHub App client
/// 2. **Token Retrieval**: Gets an installation token scoped to the organization
/// 3. **Installation Client Creation**: Creates a new client authenticated with the installation token
///
/// ## Token Scope
///
/// The installation token provides:
/// - Access to repositories within the organization
/// - Permissions granted when the app was installed
/// - Time-limited access (typically 1 hour)
/// - Attribution to the GitHub App in audit logs
///
/// ## Parameters
///
/// * `app_id` - GitHub App ID (from app settings)
/// * `app_key` - GitHub App private key (PEM format)
/// * `organization` - Organization name where the app is installed
///
/// ## Returns
///
/// Returns a tuple of:
/// - `String`: Installation token for git operations
/// - `GitHubClient`: Client authenticated with the installation token for API operations
///
/// ## Errors
///
/// Returns `RepoRollerError` if:
/// - App client creation fails (invalid credentials)
/// - Installation token retrieval fails (app not installed, network issues)
/// - Token client creation fails (internal error)
///
/// ## Example
///
/// ```rust,ignore
/// use repo_roller_core::github_auth::setup_github_authentication;
///
/// let app_id = 12345;
/// let private_key = std::fs::read_to_string("app-key.pem")?;
///
/// let (token, client) = setup_github_authentication(
///     app_id,
///     &private_key,
///     "acme-corp"
/// ).await?;
///
/// println!("Token length: {}", token.len());
/// // Use client for GitHub API calls
/// let repos = client.list_org_repositories("acme-corp").await?;
/// ```
///
/// ## Security Considerations
///
/// - Never log or expose the private key
/// - Never log or expose the installation token
/// - Tokens should be treated as credentials
/// - Tokens expire and should not be cached long-term
pub(crate) async fn setup_github_authentication(
    app_id: u64,
    app_key: &str,
    organization: &str,
) -> RepoRollerResult<(String, GitHubClient)> {
    info!("Creating GitHub App client for authentication");
    let app_client = create_app_client(app_id, app_key).await.map_err(|e| {
        error!("Failed to create GitHub App client: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create GitHub App client: {}", e),
        })
    })?;

    let repo_client = GitHubClient::new(app_client);

    info!(
        "Getting installation token for organization: {}",
        organization
    );
    let installation_token = repo_client
        .get_installation_token_for_org(organization)
        .await
        .map_err(|e| {
            error!(
                "Failed to get installation token for organization '{}': {}",
                organization, e
            );
            RepoRollerError::GitHub(GitHubError::AuthenticationFailed {
                reason: format!(
                    "Failed to get installation token for organization '{}': {}",
                    organization, e
                ),
            })
        })?;

    info!("Successfully retrieved installation token");

    let installation_client =
        github_client::create_token_client(&installation_token).map_err(|e| {
            error!("Failed to create installation token client: {}", e);
            RepoRollerError::System(SystemError::Internal {
                reason: format!("Failed to create installation token client: {}", e),
            })
        })?;

    let installation_repo_client = GitHubClient::new(installation_client);

    Ok((installation_token, installation_repo_client))
}
