//! Crate for interacting with the GitHub REST API.
//!
//! This crate provides a client for making authenticated requests to GitHub,
//! authenticating as a GitHub App using its ID and private key.

use async_trait::async_trait;
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
#[derive(Debug)]
pub struct GitHubClient {
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

    /// Creates a new `GitHubClient` instance authenticated as a GitHub App.
    ///
    /// # Arguments
    ///
    /// * `app_id` - The ID of the GitHub App.
    /// * `private_key` - The private key associated with the GitHub App, in PEM format.
    ///
    /// # Errors
    /// Returns an `Error::AuthError` if authentication or client building fails.
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
        let path = format!("/orgs/{}/repos", org_name);
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
        let path = format!("/repos/{}/{}", owner, repo);
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
        // Delegate to the existing implementation
        self.get_installation_token_for_org(org_name).await
    }

    async fn get_organization_default_branch(&self, org_name: &str) -> Result<String, Error> {
        info!(
            org_name = org_name,
            "Getting default branch setting for organization"
        );

        let path = format!("/orgs/{}", org_name);

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

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    iat: u64,
    exp: u64,
    iss: u64,
}

/// Represents the payload for creating a new repository via the REST API.
/// Use `Default::default()` or builder pattern and modify fields as needed.
#[derive(Serialize, Default, Debug, Clone)] // Added Clone
pub struct RepositoryCreatePayload {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>, // Defaults to false if None

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>, // Defaults to true if None

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>, // Defaults to true if None

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>, // Defaults to true if None

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_template: Option<bool>, // Defaults to false if None

                                   // Add other creation fields like team_id, auto_init, gitignore_template etc. as needed
}

/// Trait for repository operations (creation, file push, etc.).
#[async_trait]
pub trait RepositoryClient: Send + Sync {
    async fn create_org_repository(
        &self,
        owner: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<models::Repository, Error>;

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

/// Represents the settings that can be updated for a repository.
/// Use `Default::default()` and modify fields as needed.
#[derive(Serialize, Default, Debug)]
pub struct RepositorySettingsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    // Add other updatable fields like topics, default_branch etc. as needed
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
        Error::AuthError(
            format!("Failed to translate the private key. Error was: {}", e).to_string(),
        )
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

#[instrument(skip(token))]
pub fn create_token_client(token: &str) -> Result<Octocrab, Error> {
    Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|_| Error::ApiError())
}

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
