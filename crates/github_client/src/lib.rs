//! Crate for interacting with the GitHub REST API.
//!
//! This crate provides a client for making authenticated requests to GitHub,
//! authenticating as a GitHub App using its ID and private key.

use async_trait::async_trait;
use base64::Engine;
use jsonwebtoken::EncodingKey;
use octocrab::{Octocrab, Result as OctocrabResult};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};

pub mod errors;
pub use errors::Error;

pub mod models;

// Reference the tests module in the separate file
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// A client for interacting with the GitHub API, authenticated as a GitHub App.
///
/// This struct provides a high-level interface for GitHub API operations using
/// GitHub App authentication. It wraps an Octocrab client and provides methods
/// for repository management, installation token retrieval, and organization queries.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::{GitHubClient, create_app_client};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let app_id = 123456;
///     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
///
///     let octocrab_client = create_app_client(app_id, private_key).await?;
///     let github_client = GitHubClient::new(octocrab_client);
///
///     let installations = github_client.list_installations().await?;
///     println!("Found {} installations", installations.len());
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct GitHubClient {
    /// The underlying Octocrab client used for API requests
    client: Octocrab,
}

impl GitHubClient {
    /// Gets an installation access token for a specific organization.
    ///
    /// This method finds the installation for the given organization and returns
    /// an access token that can be used for git operations and API calls.
    ///
    /// # Arguments
    ///
    /// * `org_name` - The name of the organization to get the installation token for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the installation access token as a string, or an error
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidResponse` if:
    /// - The API call fails
    /// - No installation is found for the organization
    /// - The token cannot be retrieved
    ///
    /// # Example
    ///
    /// ```rust
    /// # use github_client::{GitHubClient, create_app_client};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let app_id = 123456;
    /// #     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
    /// #     let client_octocrab = create_app_client(app_id, private_key).await?;
    /// #     let client = GitHubClient::new(client_octocrab);
    ///
    ///     let token = client.get_installation_token_for_org("my-org").await?;
    ///     println!("Got installation token: {}", &token[..8]); // Only show first 8 chars
    ///
    /// #     Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(org_name = %org_name))]
    pub async fn get_installation_token_for_org(&self, org_name: &str) -> Result<String, Error> {
        info!(
            org_name = org_name,
            "Getting installation token for organization"
        );

        // First, list all installations to find the one for this org
        info!("Calling list_installations to find organization installation");
        let installations = self.list_installations().await?;

        info!(
            org_name = org_name,
            installation_count = installations.len(),
            "Retrieved installations, searching for organization"
        );

        // Log all available installations for debugging
        for (i, inst) in installations.iter().enumerate() {
            info!(
                index = i,
                installation_id = inst.id,
                account_login = inst.account.login,
                account_type = ?inst.account.account_type,
                "Available installation"
            );
        }

        let installation = installations
            .into_iter()
            .find(|inst| inst.account.login.eq_ignore_ascii_case(org_name))
            .ok_or_else(|| {
                error!(
                    org_name = org_name,
                    "No installation found for organization - this means the GitHub App is not installed on this organization"
                );
                Error::InvalidResponse
            })?;

        info!(
            org_name = org_name,
            installation_id = installation.id,
            account_login = installation.account.login,
            "Found matching installation for organization"
        );

        // Get the installation access token
        info!(
            installation_id = installation.id,
            "Requesting installation token from GitHub API"
        );
        let (_, token) = self
            .client
            .installation_and_token(installation.id.into())
            .await
            .map_err(|e| {
                error!(
                    org_name = org_name,
                    installation_id = installation.id,
                    "Failed to get installation token from GitHub API"
                );
                log_octocrab_error("Failed to get installation token", e);
                Error::InvalidResponse
            })?;

        info!(
            org_name = org_name,
            installation_id = installation.id,
            "Successfully retrieved installation token"
        );
        Ok(token.expose_secret().clone())
    }

    /// Fetches details for a specific repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name).
    /// * `repo` - The name of the repository.
    ///
    /// # Errors
    /// Returns an `Error::Octocrab` if the API call fails.
    #[instrument(skip(self), fields(owner = %owner, repo = %repo))]
    pub async fn get_repository(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<models::Repository, Error> {
        let result = self.client.repos(owner, repo).get().await;
        match result {
            Ok(r) => Ok(models::Repository::from(r)),
            Err(e) => {
                log_octocrab_error("Failed to get repository", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    /// Lists all installations for the authenticated GitHub App.
    ///
    /// This method retrieves all installations where the GitHub App is installed,
    /// which can be used to find the installation ID for a specific organization.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of installation objects, or an error if the
    /// operation fails.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidResponse` if the API call fails or the response
    /// cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use github_client::{GitHubClient, create_app_client};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let app_id = 123456;
    /// #     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
    /// #     let client_octocrab = create_app_client(app_id, private_key).await?;
    /// #     let client = GitHubClient::new(client_octocrab);
    ///
    ///     let installations = client.list_installations().await?;
    ///     for installation in installations {
    ///         println!("Installation ID: {}, Account: {}", installation.id, installation.account.login);
    ///     }
    ///
    /// #     Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn list_installations(&self) -> Result<Vec<models::Installation>, Error> {
        info!("Listing installations for GitHub App using JWT authentication");

        // Use direct REST API call instead of octocrab's high-level method
        let result: OctocrabResult<Vec<octocrab::models::Installation>> =
            self.client.get("/app/installations", None::<&()>).await;

        match result {
            Ok(installations) => {
                let converted_installations: Vec<models::Installation> = installations
                    .into_iter()
                    .map(models::Installation::from)
                    .collect();

                info!(
                    count = converted_installations.len(),
                    "Successfully retrieved installations for GitHub App"
                );

                Ok(converted_installations)
            }
            Err(e) => {
                error!(
                    "Failed to list installations - this likely means JWT authentication failed"
                );
                log_octocrab_error("Failed to list installations", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    /// Gets the content of a file from a repository.
    ///
    /// This method retrieves the contents of a file from the specified path in
    /// the repository. The file content is returned as a UTF-8 string.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    /// * `path` - The path to the file within the repository (e.g., "README.md" or "src/main.rs")
    ///
    /// # Returns
    ///
    /// Returns the file content as a `String` if successful.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if:
    /// - The file doesn't exist (404)
    /// - The path points to a directory, not a file
    /// - The file content cannot be decoded as UTF-8
    /// - The API request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::{GitHubClient, create_app_client};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let app_id = 123456;
    /// #     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
    /// #     let client_octocrab = create_app_client(app_id, private_key).await?;
    /// #     let client = GitHubClient::new(client_octocrab);
    ///
    ///     let content = client.get_file_content("my-org", "my-repo", "README.md").await?;
    ///     println!("README content: {}", content);
    ///
    /// #     Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(owner = %owner, repo = %repo, path = %path))]
    pub async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<String, Error> {
        debug!(
            owner = owner,
            repo = repo,
            path = path,
            "Fetching file content from repository"
        );

        // Use the repos API to get file contents
        let result = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .send()
            .await;

        match result {
            Ok(content) => {
                // The GitHub API returns content items
                // For a file path, we expect items with content field
                let items = content.items;

                // Get the first item (should be the file)
                if let Some(item) = items.first() {
                    // Decode the base64 content
                    if let Some(encoded_content) = &item.content {
                        // Remove newlines from base64 encoding
                        let cleaned = encoded_content.replace('\n', "");

                        let decoded_bytes = base64::engine::general_purpose::STANDARD
                            .decode(&cleaned)
                            .map_err(|e| {
                                error!(
                                    owner = owner,
                                    repo = repo,
                                    path = path,
                                    "Failed to decode base64 content: {}", e
                                );
                                Error::InvalidResponse
                            })?;

                        let decoded = String::from_utf8(decoded_bytes).map_err(|_| {
                            error!(
                                owner = owner,
                                repo = repo,
                                path = path,
                                "Failed to decode file content as UTF-8"
                            );
                            Error::InvalidResponse
                        })?;

                        debug!(
                            size = decoded.len(),
                            "Successfully retrieved and decoded file content"
                        );

                        return Ok(decoded);
                    }
                }

                error!("File content not found in response");
                Err(Error::InvalidResponse)
            }
            Err(e) => {
                error!(
                    owner = owner,
                    repo = repo,
                    path = path,
                    "Failed to get file content"
                );
                log_octocrab_error("Failed to get file content", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    /// Creates a new `GitHubClient` instance with the provided Octocrab client.
    ///
    /// This constructor wraps an existing Octocrab client that should already be
    /// configured with appropriate authentication (typically GitHub App JWT).
    ///
    /// # Arguments
    ///
    /// * `client` - An authenticated Octocrab client instance
    ///
    /// # Returns
    ///
    /// Returns a new `GitHubClient` instance ready for API operations.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::{GitHubClient, create_app_client};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let app_id = 123456;
    ///     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
    ///
    ///     let octocrab_client = create_app_client(app_id, private_key).await?;
    ///     let github_client = GitHubClient::new(octocrab_client);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(client: Octocrab) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RepositoryClient for GitHubClient {
    /// Creates a new repository within a specified organization using the REST API directly.
    ///
    /// # Arguments
    ///
    /// * `org_name` - The name of the organization.
    /// * `payload` - A `RepositoryCreatePayload` struct containing the repository details.
    ///
    /// # Errors
    /// Returns `Error::Octocrab` for API errors or `Error::Deserialization` if the response cannot be parsed.
    async fn create_org_repository(
        &self,
        org_name: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<models::Repository, Error> {
        let path = format!("/orgs/{org_name}/repos");
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.post(path, Some(payload)).await;
        match response {
            Ok(r) => Ok(models::Repository::from(r)),
            Err(e) => {
                log_octocrab_error("Failed to create repository for organisation", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    /// Creates a new repository for the authenticated user (GitHub App) using the REST API directly.
    ///
    /// # Arguments
    ///
    /// * `payload` - A `RepositoryCreatePayload` struct containing the repository details.
    ///
    /// # Errors
    /// Returns `Error::Octocrab` for API errors or `Error::Deserialization` if the response cannot be parsed.
    async fn create_user_repository(
        &self,
        payload: &RepositoryCreatePayload,
    ) -> Result<models::Repository, Error> {
        let path = "/user/repos";
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.post(path, Some(payload)).await;
        match response {
            Ok(r) => Ok(models::Repository::from(r)),
            Err(e) => {
                log_octocrab_error("Failed to create repository for user", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    /// Updates settings for a specific repository using the REST API directly.
    ///
    /// Only the fields provided in the `settings` argument will be updated.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name).
    /// * `repo` - The name of the repository.    /// * `settings` - A `RepositorySettingsUpdate` struct containing the desired changes.
    ///
    /// # Errors
    /// Returns an `Error::Octocrab` if the API call fails.
    #[instrument(skip(self, settings), fields(owner = %owner, repo = %repo))]
    async fn update_repository_settings(
        &self,
        owner: &str,
        repo: &str,
        settings: &RepositorySettingsUpdate,
    ) -> Result<models::Repository, Error> {
        let path = format!("/repos/{owner}/{repo}");
        // Use client.patch for updating repository settings via the REST API
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.patch(path, Some(settings)).await;
        match response {
            Ok(r) => Ok(models::Repository::from(r)),
            Err(e) => {
                log_octocrab_error("Failed to create repository for user", e);
                return Err(Error::InvalidResponse);
            }
        }
    }

    async fn get_installation_token_for_org(&self, org_name: &str) -> Result<String, Error> {
        // This method implements the RepositoryClient trait by delegating to the
        // GitHubClient's own implementation. This allows GitHubClient to be used
        // polymorphically through the RepositoryClient trait while maintaining
        // its own direct API. The duplication is intentional for trait compliance.
        GitHubClient::get_installation_token_for_org(self, org_name).await
    }

    /// Retrieves the default branch setting for an organization.
    ///
    /// This method queries the GitHub API to get the organization's default
    /// branch setting, which is used for newly created repositories.
    async fn get_organization_default_branch(&self, org_name: &str) -> Result<String, Error> {
        info!(
            org_name = org_name,
            "Getting default branch setting for organization"
        );

        let path = format!("/orgs/{org_name}");

        debug!("Making API call to: {}", path);
        let response: OctocrabResult<serde_json::Value> = self.client.get(path, None::<&()>).await;

        match response {
            Ok(org_data) => {
                debug!("Organization API response received");

                // Extract the default_repository_branch field
                let default_branch = org_data
                    .get("default_repository_branch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("main") // Default to "main" if not specified
                    .to_string();

                info!(
                    org_name = org_name,
                    default_branch = default_branch,
                    "Successfully retrieved organization default branch"
                );

                Ok(default_branch)
            }
            Err(e) => {
                error!(
                    org_name = org_name,
                    "Failed to get organization information: {}", e
                );
                log_octocrab_error("Failed to get organization information", e);
                Err(Error::InvalidResponse)
            }
        }
    }
}

/// JWT claims structure for GitHub App authentication.
///
/// This struct represents the claims included in JSON Web Tokens used
/// for GitHub App authentication. It contains the standard JWT fields
/// required by GitHub's authentication system.
///
/// Note: Currently not used as JWT encoding is handled by octocrab internally,
/// but kept for potential future use or custom JWT implementation.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    /// Issued at time (Unix timestamp)
    iat: u64,
    /// Expiration time (Unix timestamp)
    exp: u64,
    /// Issuer (GitHub App ID)
    iss: u64,
}

/// Payload structure for creating a new repository via the GitHub REST API.
///
/// This struct contains all the configurable options for repository creation.
/// Use `Default::default()` to get sensible defaults, then modify specific fields
/// as needed. Optional fields that are `None` will use GitHub's default values.
///
/// # Examples
///
/// ```rust
/// use github_client::RepositoryCreatePayload;
///
/// let payload = RepositoryCreatePayload {
///     name: "my-new-repo".to_string(),
///     description: Some("A test repository".to_string()),
///     private: Some(true),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Default, Debug, Clone)]
pub struct RepositoryCreatePayload {
    /// The name of the repository (required)
    pub name: String,

    /// A short description of the repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A URL with more information about the repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Whether the repository is private (defaults to false if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,

    /// Whether issues are enabled for this repository (defaults to true if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,

    /// Whether projects are enabled for this repository (defaults to true if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,

    /// Whether the wiki is enabled for this repository (defaults to true if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,

    /// Whether this repository is a template repository (defaults to false if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_template: Option<bool>,
}

/// Trait for GitHub repository operations and management.
///
/// This trait defines the interface for interacting with GitHub repositories,
/// including creation, updates, and metadata retrieval. It abstracts the
/// underlying GitHub API client to allow for testing and different implementations.
///
/// All methods are async and return Results with appropriate error handling.
/// Implementations should handle GitHub API rate limiting, authentication,
/// and network errors appropriately.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::{RepositoryClient, RepositoryCreatePayload, GitHubClient, create_app_client};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let app_id = 123456;
///     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
///
///     let octocrab_client = create_app_client(app_id, private_key).await?;
///     let client: Box<dyn RepositoryClient> = Box::new(GitHubClient::new(octocrab_client));
///
///     let payload = RepositoryCreatePayload {
///         name: "test-repo".to_string(),
///         ..Default::default()
///     };
///
///     let repo = client.create_org_repository("my-org", &payload).await?;
///     println!("Created repository: {}", repo.name());
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait RepositoryClient: Send + Sync {
    /// Creates a new repository within a specified organization.
    ///
    /// This method creates a repository under the given organization using the
    /// provided payload configuration. The authenticated GitHub App must have
    /// appropriate permissions on the target organization.
    ///
    /// # Arguments
    ///
    /// * `owner` - The name of the organization where the repository will be created
    /// * `payload` - Configuration for the new repository (name, description, settings, etc.)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `Repository` on success, or an `Error`
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The organization doesn't exist or is not accessible
    /// - The authenticated app lacks permission to create repositories
    /// - A repository with the same name already exists
    /// - The GitHub API request fails
    async fn create_org_repository(
        &self,
        owner: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<models::Repository, Error>;

    /// Creates a new repository for the authenticated user.
    ///
    /// This method creates a repository under the authenticated user's account
    /// using the provided payload configuration.
    ///
    /// # Arguments
    ///
    /// * `payload` - Configuration for the new repository (name, description, settings, etc.)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the created `Repository` on success, or an `Error`
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - A repository with the same name already exists
    /// - The authenticated user lacks permission to create repositories
    /// - The GitHub API request fails
    async fn create_user_repository(
        &self,
        payload: &RepositoryCreatePayload,
    ) -> Result<models::Repository, Error>;

    /// Updates settings for a specific repository using the REST API directly.
    ///
    /// Only the fields provided in the `settings` argument will be updated.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name).
    /// * `repo` - The name of the repository.
    /// * `settings` - A `RepositorySettingsUpdate` struct containing the desired changes.
    ///
    /// # Errors
    /// Returns an `Error::Octocrab` if the API call fails.
    async fn update_repository_settings(
        &self,
        owner: &str,
        repo: &str,
        settings: &RepositorySettingsUpdate,
    ) -> Result<models::Repository, Error>;

    /// Gets an installation access token for a specific organization.
    ///
    /// This method finds the installation for the given organization and returns
    /// an access token that can be used for git operations and API calls.
    ///
    /// # Arguments
    ///
    /// * `org_name` - The name of the organization to get the installation token for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the installation access token as a string, or an error
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidResponse` if:
    /// - The API call fails
    /// - No installation is found for the organization
    /// - The token cannot be retrieved
    async fn get_installation_token_for_org(&self, org_name: &str) -> Result<String, Error>;

    /// Gets the default branch name for an organization.
    ///
    /// This method retrieves the organization's default branch setting which is used
    /// for newly created repositories in that organization.
    ///
    /// # Arguments
    ///
    /// * `org_name` - The name of the organization to get the default branch for.
    ///
    /// # Returns
    ///
    /// A `Result` containing the default branch name as a string, or an error
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidResponse` if:
    /// - The API call fails
    /// - The organization is not found
    /// - The default branch setting is not available
    async fn get_organization_default_branch(&self, org_name: &str) -> Result<String, Error>;
}

/// Settings that can be updated for an existing repository.
///
/// This struct contains all the repository settings that can be modified after
/// creation. Use `Default::default()` to get an empty update (no changes), then
/// set specific fields to update only those settings. Fields set to `None` will
/// not be updated.
///
/// # Examples
///
/// ```rust
/// use github_client::RepositorySettingsUpdate;
///
/// let settings = RepositorySettingsUpdate {
///     description: Some("Updated description".to_string()),
///     private: Some(false), // Make repository public
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Default, Debug)]
pub struct RepositorySettingsUpdate {
    /// Update the repository description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Update the repository homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Update the repository visibility (true for private, false for public)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,

    /// Update whether issues are enabled for this repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,

    /// Update whether projects are enabled for this repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,

    /// Update whether the wiki is enabled for this repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
}

/// Authenticates with GitHub using an installation access token for a specific app installation.
///
/// This function retrieves an access token for a GitHub App installation and creates a new
/// `Octocrab` client authenticated with that token. It is useful for performing API operations
/// on behalf of a GitHub App installation.
///
/// # Arguments
///
/// * `octocrab` - An existing `Octocrab` client instance.
/// * `installation_id` - The ID of the GitHub App installation.
/// * `repository_owner` - The owner of the repository associated with the installation.
/// * `source_repository` - The name of the repository associated with the installation.
///
/// # Returns
///
/// A `Result` containing a new `Octocrab` client authenticated with the installation access token,
/// or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the app installation cannot be found.
/// - If the access token cannot be created.
/// - If the new `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use github_client::{authenticate_with_access_token, Error};
/// use octocrab::Octocrab;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let octocrab = Octocrab::builder().build().unwrap();
///     let installation_id = 12345678; // Replace with your installation ID
///     let repository_owner = "example-owner";
///     let source_repository = "example-repo";
///
///     let authenticated_client = authenticate_with_access_token(
///         &octocrab,
///         installation_id,
///         repository_owner,
///         source_repository,
///     )
///     .await?;
///
///     // Use `authenticated_client` to perform API operations
///     Ok(())
/// }
/// ```
#[instrument]
pub async fn authenticate_with_access_token(
    octocrab: &Octocrab,
    installation_id: u64,
    repository_owner: &str,
    source_repository: &str,
) -> Result<Octocrab, Error> {
    debug!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Finding installation"
    );

    let (api_with_token, _) = octocrab
        .installation_and_token(installation_id.into())
        .await
        .map_err(|_| {
            error!(
                repository_owner = repository_owner,
                repository = source_repository,
                installation_id,
                "Failed to create a token for the installation",
            );

            Error::InvalidResponse
        })?;

    info!(
        repository_owner = repository_owner,
        repository = source_repository,
        installation_id,
        "Created access token for installation",
    );

    Ok(api_with_token)
}

/// Creates an `Octocrab` client authenticated as a GitHub App using a JWT token.
///
/// This function generates a JSON Web Token (JWT) for the specified GitHub App ID and private key,
/// and uses it to create an authenticated `Octocrab` client. The client can then be used to perform
/// API operations on behalf of the GitHub App.
///
/// # Arguments
///
/// * `app_id` - The ID of the GitHub App.
/// * `private_key` - The private key associated with the GitHub App, in PEM format.
///
/// # Returns
///
/// A `Result` containing an authenticated `Octocrab` client, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function returns an `Error` in the following cases:
/// - If the private key cannot be parsed.
/// - If the JWT token cannot be created.
/// - If the `Octocrab` client cannot be built.
///
/// # Example
///
/// ```rust,no_run
/// use github_client::{create_app_client, Error};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let app_id = 123456; // Replace with your GitHub App ID
///     let private_key = r#"
/// -----BEGIN RSA PRIVATE KEY-----
/// ...
/// -----END RSA PRIVATE KEY-----
/// "#; // Replace with your GitHub App private key
///
///     let client = create_app_client(app_id, private_key).await?;
///
///     // Use `client` to perform API operations
///     Ok(())
/// }
/// ```
#[instrument(skip(private_key))]
pub async fn create_app_client(app_id: u64, private_key: &str) -> Result<Octocrab, Error> {
    info!(
        app_id = app_id,
        key_length = private_key.len(),
        key_starts_with = &private_key[..27], // "-----BEGIN RSA PRIVATE KEY"
        "Creating GitHub App client with provided credentials"
    );

    let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|e| {
        error!(
            app_id = app_id,
            error = %e,
            "Failed to parse RSA private key - key format is invalid"
        );
        Error::AuthError(format!("Failed to translate the private key. Error was: {e}").to_string())
    })?;

    info!(app_id = app_id, "Successfully parsed RSA private key");

    let octocrab = Octocrab::builder()
        .app(app_id.into(), key)
        .build()
        .map_err(|e| {
            error!(
                app_id = app_id,
                error = ?e,
                "Failed to build Octocrab client with GitHub App credentials"
            );
            Error::AuthError("Failed to get a personal token for the app install.".to_string())
        })?;

    info!(app_id = app_id, "Successfully created GitHub App client");

    Ok(octocrab)
}

/// Creates an Octocrab client authenticated with a personal access token.
///
/// This function creates a GitHub API client using a personal access token
/// for authentication. This is useful for operations that don't require
/// GitHub App authentication.
///
/// # Arguments
///
/// * `token` - A GitHub personal access token
///
/// # Returns
///
/// Returns a `Result` containing an authenticated `Octocrab` client, or an `Error`
/// if the client cannot be built.
///
/// # Errors
///
/// This function returns an `Error::ApiError` if the Octocrab client cannot be
/// constructed with the provided token.
///
/// # Examples
///
/// ```rust,no_run
/// use github_client::create_token_client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let token = "ghp_xxxxxxxxxxxxxxxxxxxx"; // Your GitHub PAT
///     let client = create_token_client(token)?;
///
///     // Use client for API operations
///     Ok(())
/// }
/// ```
#[instrument(skip(token))]
pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

/// Helper function to log Octocrab errors with appropriate detail.
///
/// This function examines the type of Octocrab error and logs relevant
/// information for debugging purposes. It handles different error types
/// with appropriate context and formatting.
fn log_octocrab_error(message: &str, e: octocrab::Error) {
    match e {
        octocrab::Error::GitHub { source, backtrace } => {
            let err = source;
            error!(
                error_message = err.message,
                backtrace = backtrace.to_string(),
                "{}. Received an error from GitHub",
                message
            )
        }
        octocrab::Error::UriParse { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. Failed to parse URI.",
            message
        ),

        octocrab::Error::Uri { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}, Failed to parse URI.",
            message
        ),
        octocrab::Error::InvalidHeaderValue { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. One of the header values was invalid.",
            message
        ),
        octocrab::Error::InvalidUtf8 { source, backtrace } => error!(
            error_message = source.to_string(),
            backtrace = backtrace.to_string(),
            "{}. The message wasn't valid UTF-8.",
            message,
        ),
        _ => error!(error_message = e.to_string(), message),
    };
}
