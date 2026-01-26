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
use tracing::{debug, error, info, instrument, warn};

pub mod errors;
pub use errors::Error;

// Domain-specific modules
pub mod branch_protection;
pub mod contents;
pub mod environment;
pub mod environment_detector;
pub mod installation;
pub mod label;
pub mod repository;
pub mod user;
pub mod webhook;

// Re-export types for convenient access
pub use branch_protection::BranchProtection;
pub use contents::{EntryType, TreeEntry};
pub use environment::{GitHubEnvironmentDetector, PlanLimitations};
pub use environment_detector::GitHubApiEnvironmentDetector;
pub use installation::{Account, Installation};
pub use label::Label;
pub use repository::{Organization, Repository};
pub use user::User;
pub use webhook::{Webhook, WebhookDetails, WebhookEvent};

pub mod custom_property_payload;
pub use custom_property_payload::CustomPropertiesPayload;

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
#[derive(Debug, Clone)]
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
                Error::AuthError(format!(
                    "GitHub App not installed on organization '{}'",
                    org_name
                ))
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
                    error = %e,
                    "Failed to get installation token from GitHub API"
                );
                log_octocrab_error("Failed to get installation token", e);
                Error::AuthError(format!(
                    "Failed to get installation token for organization '{}'",
                    org_name
                ))
            })?;

        info!(
            org_name = org_name,
            installation_id = installation.id,
            "Successfully retrieved installation token"
        );
        Ok(token.expose_secret().to_string())
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
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository, Error> {
        let result = self.client.repos(owner, repo).get().await;
        match result {
            Ok(r) => Ok(Repository::from(r)),
            Err(e) => {
                // Pattern match on octocrab error to check status code
                match &e {
                    octocrab::Error::GitHub { source, .. } => {
                        if source.status_code == http::StatusCode::NOT_FOUND {
                            debug!("Repository not found: {}/{}", owner, repo);
                            return Err(Error::NotFound);
                        }
                        error!(
                            owner = owner,
                            repo = repo,
                            status_code = %source.status_code,
                            message = %source.message,
                            "GitHub API error getting repository"
                        );
                        log_octocrab_error("Failed to get repository", e);
                        Err(Error::ApiError())
                    }
                    _ => {
                        eprintln!(
                            "DIAGNOSTIC: Non-GitHub error getting repository {}/{}: error={}",
                            owner, repo, e
                        );
                        error!(
                            owner = owner,
                            repo = repo,
                            error = %e,
                            "Non-GitHub error getting repository (parsing, network, etc.)"
                        );
                        log_octocrab_error("Failed to get repository", e);
                        Err(Error::InvalidResponse)
                    }
                }
            }
        }
    }

    /// Searches for repositories in an organization that have a specific topic.
    ///
    /// This method constructs a GitHub search query to find repositories within
    /// the specified organization that are tagged with the given topic. It uses
    /// GitHub's repository search API with query syntax: `org:{org} topic:{topic}`
    ///
    /// # Arguments
    ///
    /// * `org` - The organization name to search within
    /// * `topic` - The topic tag to search for
    ///
    /// # Returns
    ///
    /// A vector of `Repository` objects matching the search criteria.
    /// Returns an empty vector if no repositories match.
    ///
    /// # Errors
    ///
    /// * `Error::ApiError` - GitHub API request failed
    /// * `Error::InvalidResponse` - Search response could not be parsed
    ///
    /// # Behavior
    ///
    /// 1. Constructs search query: `org:{org} topic:{topic}`
    /// 2. Calls GitHub search API via octocrab
    /// 3. Parses search results into `Repository` objects
    /// 4. Returns all matching repositories (handles pagination automatically)
    ///
    /// # GitHub API Rate Limits
    ///
    /// This method counts against GitHub's search API rate limits:
    /// - Authenticated: 30 requests per minute
    /// - Unauthenticated: 10 requests per minute
    ///
    /// # Example Usage
    ///
    /// ```rust,no_run
    /// use github_client::GitHubClient;
    ///
    /// # async fn example(client: &GitHubClient) -> Result<(), github_client::Error> {
    /// // Find all repositories in "my-org" with topic "reporoller-metadata"
    /// let repos = client.search_repositories_by_topic("my-org", "reporoller-metadata").await?;
    ///
    /// if repos.is_empty() {
    ///     println!("No repositories found with this topic");
    /// } else {
    ///     for repo in repos {
    ///         println!("Found: {}", repo.name());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Integration with MetadataRepositoryProvider
    ///
    /// This method is used by `GitHubMetadataProvider::discover_by_topic()` to
    /// implement topic-based metadata repository discovery. The metadata provider
    /// validates that exactly one repository is found and returns appropriate
    /// errors for 0 or multiple matches.
    ///
    /// See docs/spec/interfaces/github-repository-search.md for full contract
    #[instrument(skip(self), fields(org = %org, topic = %topic))]
    pub async fn search_repositories_by_topic(
        &self,
        org: &str,
        topic: &str,
    ) -> Result<Vec<Repository>, Error> {
        let query = format!("org:{} topic:{}", org, topic);
        self.search_repositories(&query).await
    }

    /// Lists contents of a directory in a GitHub repository.
    ///
    /// Uses the GitHub Contents API to retrieve directory listings. Automatically
    /// handles pagination for directories with more than 1000 entries.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user name)
    /// * `repo` - Repository name
    /// * `path` - Directory path to list (relative to repository root)
    /// * `branch` - Branch/ref to query (e.g., "main", "master", "develop")
    ///
    /// # Returns
    ///
    /// `Vec<TreeEntry>` - Vector of directory entries with type information
    ///
    /// # Errors
    ///
    /// * `Error::NotFound` - Path doesn't exist in repository
    /// * `Error::InvalidResponse` - Path is a file, not a directory
    /// * `Error::AuthError` - Authentication failure or insufficient permissions
    /// * `Error::RateLimitExceeded` - GitHub API rate limit exceeded
    /// * `Error::ApiError` - Other GitHub API errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use github_client::{GitHubClient, create_app_client};
    /// # use github_client::EntryType;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let app_id = 123456;
    /// #     let private_key = "-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----";
    /// #     let octocrab_client = create_app_client(app_id, private_key).await?;
    /// #     let client = GitHubClient::new(octocrab_client);
    ///
    /// // List repository types from metadata repository
    /// let entries = client
    ///     .list_directory_contents("my-org", ".reporoller-test", "types", "main")
    ///     .await?;
    ///
    /// // Filter to only directories
    /// let types: Vec<String> = entries
    ///     .iter()
    ///     .filter(|e| matches!(e.entry_type, EntryType::Dir))
    ///     .map(|e| e.name.clone())
    ///     .collect();
    ///
    /// println!("Found repository types: {:?}", types);
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # GitHub API Details
    ///
    /// - Endpoint: `GET /repos/{owner}/{repo}/contents/{path}?ref={branch}`
    /// - Response: Array of content objects for directories, single object for files
    /// - Pagination: Uses Link header for large directories (>1000 entries)
    /// - Rate Limiting: Counts against authenticated rate limit (5000/hour)
    ///
    /// # Integration with GitHubMetadataProvider
    ///
    /// This method is used by `GitHubMetadataProvider::list_available_repository_types()`
    /// to discover repository types from the metadata repository's `types/` directory.
    /// The metadata provider filters results to only include directories.
    ///
    /// See specs/interfaces/github-directory-listing.md for full contract
    #[instrument(skip(self), fields(owner = %owner, repo = %repo, path = %path, branch = %branch))]
    pub async fn list_directory_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: &str,
    ) -> Result<Vec<TreeEntry>, Error> {
        info!(
            owner = %owner,
            repo = %repo,
            path = %path,
            branch = %branch,
            "Listing directory contents"
        );

        // Use the repos API to get directory contents
        let result = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await;

        match result {
            Ok(content) => {
                let items = content.items;

                // Convert octocrab content items to TreeEntry objects
                // Note: For files, GitHub API returns single item with content field
                // For directories, GitHub API returns multiple items (directory entries)
                // The file vs directory check is done by looking at individual entry types
                let entries: Vec<TreeEntry> = items
                    .into_iter()
                    .map(|item| {
                        let entry_type = match item.r#type.as_str() {
                            "file" => EntryType::File,
                            "dir" => EntryType::Dir,
                            "symlink" => EntryType::Symlink,
                            "submodule" => EntryType::Submodule,
                            _ => {
                                warn!(
                                    item_type = %item.r#type,
                                    "Unknown content type, defaulting to File"
                                );
                                EntryType::File
                            }
                        };

                        TreeEntry {
                            name: item.name,
                            path: item.path,
                            entry_type,
                            sha: item.sha,
                            size: item.size as u64,
                            download_url: item.download_url.map(|u| u.to_string()),
                        }
                    })
                    .collect();

                debug!(
                    entry_count = entries.len(),
                    "Successfully retrieved directory entries"
                );

                // Log entry type breakdown
                let file_count = entries
                    .iter()
                    .filter(|e| matches!(e.entry_type, EntryType::File))
                    .count();
                let dir_count = entries
                    .iter()
                    .filter(|e| matches!(e.entry_type, EntryType::Dir))
                    .count();
                debug!(
                    files = file_count,
                    directories = dir_count,
                    "Entry type breakdown"
                );

                Ok(entries)
            }
            Err(e) => {
                // Map octocrab errors to appropriate Error types using pattern matching
                match &e {
                    octocrab::Error::GitHub { source, .. } => {
                        // Check for 404 Not Found
                        if source.status_code == http::StatusCode::NOT_FOUND {
                            error!(
                                owner = %owner,
                                repo = %repo,
                                path = %path,
                                "Directory not found"
                            );
                            log_octocrab_error("Directory not found", e);
                            return Err(Error::NotFound);
                        }

                        // Check for 401 Unauthorized
                        if source.status_code == http::StatusCode::UNAUTHORIZED {
                            error!(
                                owner = %owner,
                                repo = %repo,
                                "Authentication failed"
                            );
                            log_octocrab_error("Authentication failed", e);
                            return Err(Error::AuthError("Authentication failed".to_string()));
                        }

                        // Check for 403 Forbidden - could be rate limit or permissions
                        if source.status_code == http::StatusCode::FORBIDDEN {
                            let msg_lower = source.message.to_lowercase();

                            // Check if it's a rate limit error
                            if msg_lower.contains("rate limit") {
                                error!("GitHub API rate limit exceeded");
                                log_octocrab_error("Rate limit exceeded", e);
                                return Err(Error::RateLimitExceeded);
                            }

                            // Otherwise it's a permissions error
                            error!(
                                owner = %owner,
                                repo = %repo,
                                "Access forbidden - check permissions"
                            );
                            log_octocrab_error("Access forbidden", e);
                            return Err(Error::AuthError(
                                "Access forbidden - insufficient permissions".to_string(),
                            ));
                        }

                        // Other GitHub API errors
                        eprintln!("DIAGNOSTIC: GitHub API error listing directory {} in {}/{}: status={}, message={}",
                            path, owner, repo, source.status_code, source.message);
                        error!(
                            owner = %owner,
                            repo = %repo,
                            path = %path,
                            status_code = %source.status_code,
                            message = %source.message,
                            "GitHub API error listing directory contents"
                        );
                        log_octocrab_error("Failed to list directory contents", e);
                        Err(Error::ApiError())
                    }
                    _ => {
                        // Non-GitHub errors (network, parsing, etc.)
                        eprintln!(
                            "DIAGNOSTIC: Non-GitHub error listing directory {} in {}/{}: error={}",
                            path, owner, repo, e
                        );
                        error!(
                            owner = %owner,
                            repo = %repo,
                            path = %path,
                            error = %e,
                            "Non-GitHub error listing directory contents (parsing, network, etc.)"
                        );
                        log_octocrab_error("Failed to list directory contents", e);
                        Err(Error::InvalidResponse)
                    }
                }
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
    pub async fn list_installations(&self) -> Result<Vec<Installation>, Error> {
        info!("Listing installations for GitHub App using JWT authentication");

        // Use direct REST API call instead of octocrab's high-level method
        let result: OctocrabResult<Vec<octocrab::models::Installation>> =
            self.client.get("/app/installations", None::<&()>).await;

        match result {
            Ok(installations) => {
                let converted_installations: Vec<Installation> =
                    installations.into_iter().map(Installation::from).collect();

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
                                    "Failed to decode base64 content: {}",
                                    e
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
                // Map octocrab errors to appropriate Error types
                match &e {
                    octocrab::Error::GitHub { source, .. } => {
                        // Check for 404 Not Found
                        if source.status_code == http::StatusCode::NOT_FOUND {
                            error!(owner = owner, repo = repo, path = path, "File not found");
                            log_octocrab_error("File not found", e);
                            return Err(Error::NotFound);
                        }

                        // Other GitHub API errors
                        eprintln!("DIAGNOSTIC: GitHub API error getting file {} in {}/{}: status={}, message={}",
                            path, owner, repo, source.status_code, source.message);
                        error!(
                            owner = owner,
                            repo = repo,
                            path = path,
                            status_code = %source.status_code,
                            message = %source.message,
                            "GitHub API error getting file content"
                        );
                        log_octocrab_error("Failed to get file content", e);
                        Err(Error::ApiError())
                    }
                    _ => {
                        // Non-GitHub errors (network, parsing, etc.)
                        eprintln!(
                            "DIAGNOSTIC: Non-GitHub error getting file {} in {}/{}: error={}",
                            path, owner, repo, e
                        );
                        error!(
                            owner = owner,
                            repo = repo,
                            path = path,
                            error = %e,
                            "Non-GitHub error getting file content (parsing, network, etc.)"
                        );
                        log_octocrab_error("Failed to get file content", e);
                        Err(Error::InvalidResponse)
                    }
                }
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
    ) -> Result<Repository, Error> {
        let path = format!("/orgs/{org_name}/repos");
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.post(path, Some(payload)).await;
        match response {
            Ok(r) => Ok(Repository::from(r)),
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
    ) -> Result<Repository, Error> {
        let path = "/user/repos";
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.post(path, Some(payload)).await;
        match response {
            Ok(r) => Ok(Repository::from(r)),
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
    ) -> Result<Repository, Error> {
        let path = format!("/repos/{owner}/{repo}");
        // Use client.patch for updating repository settings via the REST API
        let response: OctocrabResult<octocrab::models::Repository> =
            self.client.patch(path, Some(settings)).await;
        match response {
            Ok(r) => Ok(Repository::from(r)),
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
                eprintln!(
                    "DIAGNOSTIC: Error getting org {} default branch: error={}",
                    org_name, e
                );
                error!(
                    org_name = org_name,
                    "Failed to get organization information: {}", e
                );
                log_octocrab_error("Failed to get organization information", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn set_repository_custom_properties(
        &self,
        owner: &str,
        repo: &str,
        payload: &CustomPropertiesPayload,
    ) -> Result<(), Error> {
        info!(
            owner = owner,
            repo = repo,
            property_count = payload.properties.len(),
            "Setting custom properties on repository"
        );

        let path = format!("/repos/{owner}/{repo}/custom-properties");

        debug!("Making API call to: {}", path);
        // Use Option<serde_json::Value> to handle 204 No Content responses
        let response: OctocrabResult<Option<serde_json::Value>> =
            self.client.patch(path, Some(payload)).await;

        match response {
            Ok(_) => {
                info!(
                    owner = owner,
                    repo = repo,
                    "Successfully set custom properties on repository"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    owner = owner,
                    repo = repo,
                    "Failed to set custom properties: {}",
                    e
                );
                log_octocrab_error("Failed to set repository custom properties", e);
                Err(Error::ApiError())
            }
        }
    }

    async fn search_repositories(&self, query: &str) -> Result<Vec<Repository>, Error> {
        info!(query = query, "Searching for repositories");

        let search_result = self
            .client
            .search()
            .repositories(query)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to search repositories: {}", e);
                log_octocrab_error("Failed to search repositories", e);
                Error::ApiError()
            })?;

        // Convert octocrab repositories to our Repository using From trait
        let repositories: Vec<Repository> = search_result
            .items
            .into_iter()
            .map(Repository::from)
            .collect();

        info!(
            query = query,
            count = repositories.len(),
            "Found repositories"
        );

        Ok(repositories)
    }

    async fn get_custom_properties(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<std::collections::HashMap<String, String>, Error> {
        info!("Fetching custom properties for repository");

        // GitHub API endpoint: GET /repos/{owner}/{repo}/properties/values
        let route = format!("/repos/{}/{}/properties/values", owner, repo);
        let result: OctocrabResult<serde_json::Value> = self.client.get(&route, None::<&()>).await;

        match result {
            Ok(response) => {
                // Parse the response array of {property_name, value} objects
                let properties = response.as_array().ok_or_else(|| {
                    eprintln!(
                        "DIAGNOSTIC: Custom properties response is not an array for {}/{}",
                        owner, repo
                    );
                    error!("Custom properties response is not an array");
                    Error::InvalidResponse
                })?;

                let mut property_map = std::collections::HashMap::new();
                for prop in properties {
                    if let (Some(name), Some(value)) = (
                        prop.get("property_name").and_then(|v| v.as_str()),
                        prop.get("value").and_then(|v| v.as_str()),
                    ) {
                        property_map.insert(name.to_string(), value.to_string());
                    }
                }

                info!(
                    count = property_map.len(),
                    "Successfully retrieved custom properties"
                );
                Ok(property_map)
            }
            Err(e) => {
                eprintln!(
                    "DIAGNOSTIC: Error getting custom properties for {}/{}: error={}",
                    owner, repo, e
                );
                log_octocrab_error("Failed to get custom properties", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn list_repository_labels(&self, owner: &str, repo: &str) -> Result<Vec<String>, Error> {
        info!("Listing repository labels");

        let result = self
            .client
            .issues(owner, repo)
            .list_labels_for_repo()
            .send()
            .await;

        match result {
            Ok(labels) => {
                let label_names: Vec<String> =
                    labels.items.into_iter().map(|label| label.name).collect();

                info!(count = label_names.len(), "Successfully listed labels");
                Ok(label_names)
            }
            Err(e) => {
                // Match on the error type to provide detailed diagnostics
                match &e {
                    octocrab::Error::GitHub { source, .. } => {
                        eprintln!(
                            "DIAGNOSTIC: GitHub API error listing labels for {}/{}: status={}, message={}",
                            owner, repo, source.status_code, source.message
                        );
                        error!(
                            owner = owner,
                            repo = repo,
                            status_code = %source.status_code,
                            message = %source.message,
                            "GitHub API error listing repository labels"
                        );
                        log_octocrab_error("Failed to list repository labels", e);
                        Err(Error::ApiError())
                    }
                    _ => {
                        eprintln!(
                            "DIAGNOSTIC: Non-GitHub error listing labels for {}/{}: error={}",
                            owner, repo, e
                        );
                        error!(
                            owner = owner,
                            repo = repo,
                            error = %e,
                            "Non-GitHub error listing repository labels"
                        );
                        log_octocrab_error("Failed to list repository labels", e);
                        Err(Error::InvalidResponse)
                    }
                }
            }
        }
    }

    async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        color: &str,
        description: &str,
    ) -> Result<(), Error> {
        info!(name = name, "Creating repository label");

        // Construct the full API URL (octocrab's _post requires full URL, not relative path)
        let url = format!("https://api.github.com/repos/{}/{}/labels", owner, repo);
        let body = serde_json::json!({
            "name": name,
            "color": color,
            "description": description,
        });

        // Send the request and get the raw response
        let result = self.client._post(&url, Some(&body)).await;

        match result {
            Ok(_response) => {
                info!(name = name, "Successfully created label");
                Ok(())
            }
            Err(e) => {
                // Log the error details for debugging
                debug!(
                    name = name,
                    error = ?e,
                    "Label creation failed, checking if it already exists"
                );

                // Check if this is a "label already exists" error (422 Unprocessable Entity)
                // In that case, update the existing label instead
                if is_label_already_exists_error(&e) {
                    info!(
                        name = name,
                        "Label already exists, updating instead of creating"
                    );

                    // Update the existing label using PATCH
                    let update_url = format!(
                        "https://api.github.com/repos/{}/{}/labels/{}",
                        owner, repo, name
                    );
                    let update_result = self.client._patch(&update_url, Some(&body)).await;

                    match update_result {
                        Ok(_) => {
                            info!(name = name, "Successfully updated existing label");
                            Ok(())
                        }
                        Err(update_e) => {
                            eprintln!(
                                "DIAGNOSTIC: Error updating label {} in {}/{}: error={}",
                                name, owner, repo, update_e
                            );
                            log_octocrab_error("Failed to update existing label", update_e);
                            Err(Error::InvalidResponse)
                        }
                    }
                } else {
                    eprintln!(
                        "DIAGNOSTIC: Error creating label {} in {}/{}: error={}",
                        name, owner, repo, e
                    );
                    log_octocrab_error("Failed to create label", e);
                    Err(Error::InvalidResponse)
                }
            }
        }
    }

    async fn get_repository_settings(&self, owner: &str, repo: &str) -> Result<Repository, Error> {
        info!("Getting repository settings");

        let result = self.client.repos(owner, repo).get().await;

        match result {
            Ok(repo) => {
                info!("Successfully retrieved repository settings");
                Ok(repo.into())
            }
            Err(e) => {
                log_octocrab_error("Failed to get repository settings", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn get_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Option<BranchProtection>, Error> {
        info!(branch = branch, "Getting branch protection rules");

        // GitHub API endpoint: GET /repos/{owner}/{repo}/branches/{branch}/protection
        let url = format!("repos/{}/{}/branches/{}/protection", owner, repo, branch);

        let result: Result<serde_json::Value, octocrab::Error> =
            self.client.get(url, None::<&()>).await;

        match result {
            Ok(protection_data) => {
                info!("Successfully retrieved branch protection rules");

                // Extract review requirements if present
                let review_count = protection_data
                    .get("required_pull_request_reviews")
                    .and_then(|reviews| reviews.get("required_approving_review_count"))
                    .and_then(|count| count.as_u64())
                    .map(|c| c as u32);

                let code_owner_reviews = protection_data
                    .get("required_pull_request_reviews")
                    .and_then(|reviews| reviews.get("require_code_owner_reviews"))
                    .and_then(|v| v.as_bool());

                let dismiss_stale = protection_data
                    .get("required_pull_request_reviews")
                    .and_then(|reviews| reviews.get("dismiss_stale_reviews"))
                    .and_then(|v| v.as_bool());

                Ok(Some(BranchProtection {
                    required_approving_review_count: review_count,
                    require_code_owner_reviews: code_owner_reviews,
                    dismiss_stale_reviews: dismiss_stale,
                }))
            }
            Err(e) if is_not_found_error(&e) => {
                info!("No branch protection configured");
                Ok(None)
            }
            Err(e) => {
                log_octocrab_error("Failed to get branch protection", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn list_repository_files(&self, owner: &str, repo: &str) -> Result<Vec<String>, Error> {
        info!("Listing all files in repository");

        let mut all_files = Vec::new();
        let mut dirs_to_process = vec![String::new()]; // Start with root directory

        while let Some(path) = dirs_to_process.pop() {
            debug!(
                "Processing directory: {}",
                if path.is_empty() { "/" } else { &path }
            );

            // Get contents of current directory
            let contents = self
                .client
                .repos(owner, repo)
                .get_content()
                .path(&path)
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to get directory contents for path: {}", path);
                    log_octocrab_error("Failed to get directory contents", e);
                    Error::InvalidResponse
                })?;

            // Process each item in the directory
            for item in contents.items {
                let item_path = item.path;

                match item.r#type.as_str() {
                    "file" => {
                        // Add file to the list
                        all_files.push(item_path);
                    }
                    "dir" => {
                        // Add directory to be processed
                        dirs_to_process.push(item_path);
                    }
                    "symlink" => {
                        // Include symlinks in the file list
                        debug!("Found symlink: {}", item_path);
                        all_files.push(item_path);
                    }
                    other => {
                        debug!("Skipping item of type '{}': {}", other, item_path);
                    }
                }
            }
        }

        info!(
            "Successfully listed {} files in repository",
            all_files.len()
        );

        Ok(all_files)
    }

    async fn list_webhooks(&self, owner: &str, repo: &str) -> Result<Vec<Webhook>, Error> {
        info!(owner = owner, repo = repo, "Listing repository webhooks");

        let url = format!("https://api.github.com/repos/{}/{}/hooks", owner, repo);

        let result: OctocrabResult<Vec<Webhook>> = self.client._get(&url, None::<&()>).await;

        match result {
            Ok(webhooks) => {
                info!(
                    owner = owner,
                    repo = repo,
                    count = webhooks.len(),
                    "Successfully listed webhooks"
                );
                Ok(webhooks)
            }
            Err(e) => {
                log_octocrab_error("Failed to list webhooks", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        url: &str,
        content_type: &str,
        secret: Option<&str>,
        active: bool,
        events: &[String],
    ) -> Result<Webhook, Error> {
        info!(
            owner = owner,
            repo = repo,
            url = url,
            "Creating repository webhook"
        );

        let api_url = format!("https://api.github.com/repos/{}/{}/hooks", owner, repo);

        let mut config = serde_json::json!({
            "url": url,
            "content_type": content_type,
            "insecure_ssl": "0"
        });

        if let Some(secret_value) = secret {
            config["secret"] = serde_json::json!(secret_value);
        }

        let body = serde_json::json!({
            "name": "web",
            "active": active,
            "events": events,
            "config": config
        });

        let result: OctocrabResult<Webhook> = self.client._post(&api_url, Some(&body)).await;

        match result {
            Ok(webhook) => {
                info!(
                    owner = owner,
                    repo = repo,
                    webhook_id = webhook.id,
                    "Successfully created webhook"
                );
                Ok(webhook)
            }
            Err(e) => {
                log_octocrab_error("Failed to create webhook", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn update_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_id: u64,
        url: &str,
        content_type: &str,
        secret: Option<&str>,
        active: bool,
        events: &[String],
    ) -> Result<Webhook, Error> {
        info!(
            owner = owner,
            repo = repo,
            webhook_id = webhook_id,
            "Updating repository webhook"
        );

        let api_url = format!("repos/{}/{}/hooks/{}", owner, repo, webhook_id);

        let mut config = serde_json::json!({
            "url": url,
            "content_type": content_type,
            "insecure_ssl": "0"
        });

        if let Some(secret_value) = secret {
            config["secret"] = serde_json::json!(secret_value);
        }

        let body = serde_json::json!({
            "active": active,
            "events": events,
            "config": config
        });

        let result: OctocrabResult<Webhook> = self.client.patch(&api_url, Some(&body)).await;

        match result {
            Ok(webhook) => {
                info!(
                    owner = owner,
                    repo = repo,
                    webhook_id = webhook_id,
                    "Successfully updated webhook"
                );
                Ok(webhook)
            }
            Err(e) => {
                log_octocrab_error("Failed to update webhook", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn delete_webhook(&self, owner: &str, repo: &str, webhook_id: u64) -> Result<(), Error> {
        info!(
            owner = owner,
            repo = repo,
            webhook_id = webhook_id,
            "Deleting repository webhook"
        );

        let url = format!("repos/{}/{}/hooks/{}", owner, repo, webhook_id);

        let result: OctocrabResult<()> = self.client.delete(&url, None::<&()>).await;

        match result {
            Ok(_) => {
                info!(
                    owner = owner,
                    repo = repo,
                    webhook_id = webhook_id,
                    "Successfully deleted webhook"
                );
                Ok(())
            }
            Err(e) => {
                log_octocrab_error("Failed to delete webhook", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn update_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        new_name: &str,
        color: &str,
        description: &str,
    ) -> Result<(), Error> {
        info!(owner = owner, repo = repo, name = name, "Updating label");

        let url = format!("repos/{}/{}/labels/{}", owner, repo, name);

        let body = serde_json::json!({
            "new_name": new_name,
            "color": color,
            "description": description,
        });

        let result: OctocrabResult<()> = self.client.patch(&url, Some(&body)).await;

        match result {
            Ok(_) => {
                info!(
                    owner = owner,
                    repo = repo,
                    name = name,
                    "Successfully updated label"
                );
                Ok(())
            }
            Err(e) => {
                log_octocrab_error("Failed to update label", e);
                Err(Error::InvalidResponse)
            }
        }
    }

    async fn delete_label(&self, owner: &str, repo: &str, name: &str) -> Result<(), Error> {
        info!(owner = owner, repo = repo, name = name, "Deleting label");

        let url = format!("repos/{}/{}/labels/{}", owner, repo, name);

        let result: OctocrabResult<()> = self.client.delete(&url, None::<&()>).await;

        match result {
            Ok(_) => {
                info!(
                    owner = owner,
                    repo = repo,
                    name = name,
                    "Successfully deleted label"
                );
                Ok(())
            }
            Err(e) => {
                log_octocrab_error("Failed to delete label", e);
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
    ) -> Result<Repository, Error>;

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
    ) -> Result<Repository, Error>;

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
    ) -> Result<Repository, Error>;

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

    /// Sets custom properties on a repository.
    ///
    /// This method updates repository custom properties using the GitHub API.
    /// Custom property definitions must already exist at the organization level
    /// before they can be assigned to repositories.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    /// * `payload` - The custom properties payload containing properties to set
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the custom properties were successfully updated.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The repository doesn't exist or is not accessible
    /// - The authenticated app lacks permission to set custom properties
    /// - A referenced custom property doesn't exist at the organization level
    /// - The GitHub API request fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::{GitHubClient, CustomPropertiesPayload, RepositoryClient};
    /// use serde_json::json;
    ///
    /// # async fn example(client: GitHubClient) -> Result<(), github_client::Error> {
    /// let payload = CustomPropertiesPayload::new(vec![
    ///     json!({
    ///         "property_name": "repository_type",
    ///         "value": "library"
    ///     })
    /// ]);
    ///
    /// client.set_repository_custom_properties("my-org", "my-repo", &payload).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_repository_custom_properties(
        &self,
        owner: &str,
        repo: &str,
        payload: &CustomPropertiesPayload,
    ) -> Result<(), Error>;

    /// Search for repositories matching a query.
    ///
    /// This method uses GitHub's repository search API to find repositories
    /// matching the provided search query. The query supports GitHub's search
    /// syntax including qualifiers like `org:`, `user:`, `topic:`, etc.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (e.g., "org:myorg topic:template")
    ///
    /// # Returns
    ///
    /// Returns a vector of matching repositories.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    /// - The search query is invalid
    /// - The GitHub API request fails
    /// - Rate limits are exceeded
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use github_client::{GitHubClient, RepositoryClient};
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Search for template repositories in an organization
    /// let repos = client.search_repositories("org:myorg topic:reporoller-template").await?;
    ///
    /// for repo in repos {
    ///     println!("Found template: {}", repo.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn search_repositories(&self, query: &str) -> Result<Vec<Repository>, Error>;

    /// Gets custom properties for a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    ///
    /// # Returns
    ///
    /// A map of custom property names to their values.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if the API call fails.
    async fn get_custom_properties(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<std::collections::HashMap<String, String>, Error>;

    /// Lists labels for a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    ///
    /// # Returns
    ///
    /// A vector of label names.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if the API call fails.
    async fn list_repository_labels(&self, owner: &str, repo: &str) -> Result<Vec<String>, Error>;

    /// Creates a label in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    /// * `name` - The name of the label
    /// * `color` - The color of the label (hex code without #)
    /// * `description` - The description of the label
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if the API call fails.
    async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        color: &str,
        description: &str,
    ) -> Result<(), Error>;

    /// Gets repository settings including feature flags.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    ///
    /// # Returns
    ///
    /// The repository model containing all settings.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if the API call fails.
    async fn get_repository_settings(&self, owner: &str, repo: &str) -> Result<Repository, Error>;

    /// Gets branch protection rules for a specific branch.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    /// * `branch` - The branch name to get protection rules for
    ///
    /// # Returns
    ///
    /// Optional branch protection data if protection is enabled.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if the API call fails.
    async fn get_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Option<BranchProtection>, Error>;

    /// Lists all files in a repository.
    ///
    /// This method retrieves a flat list of all file paths in the repository
    /// using the GitHub Contents API. It recursively traverses directories
    /// to build a complete file listing.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the repository (user or organization name)
    /// * `repo` - The name of the repository
    ///
    /// # Returns
    ///
    /// A vector of file paths relative to the repository root.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidResponse` if:
    /// - The repository doesn't exist or is not accessible
    /// - The API request fails
    /// - Directory traversal fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use github_client::{GitHubClient, RepositoryClient};
    ///
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let files = client.list_repository_files("my-org", "my-repo").await?;
    ///
    /// for file in files {
    ///     println!("File: {}", file);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_repository_files(&self, owner: &str, repo: &str) -> Result<Vec<String>, Error>;

    /// Lists all webhooks configured for a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Vector of `Webhook` structs containing webhook details.
    ///
    /// # Errors
    ///
    /// * `Error::InvalidResponse` - API call failed or response parsing failed
    /// * `Error::ApiError` - GitHub API error
    ///
    /// # GitHub API
    ///
    /// GET /repos/{owner}/{repo}/hooks
    async fn list_webhooks(&self, owner: &str, repo: &str) -> Result<Vec<Webhook>, Error>;

    /// Creates a webhook in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    /// * `url` - Webhook URL
    /// * `content_type` - Content type ("json" or "form")
    /// * `secret` - Optional webhook secret
    /// * `active` - Whether the webhook is active
    /// * `events` - Events that trigger the webhook
    ///
    /// # Returns
    ///
    /// The created `Webhook` with its ID and configuration.
    ///
    /// # Errors
    ///
    /// * `Error::InvalidResponse` - API call failed
    /// * `Error::ApiError` - GitHub API error (duplicate webhook, invalid config, etc.)
    ///
    /// # Behavior
    ///
    /// This method does NOT check for existing webhooks - caller should verify
    /// uniqueness if needed (e.g., check if webhook with same URL already exists).
    ///
    /// # GitHub API
    ///
    /// POST /repos/{owner}/{repo}/hooks
    async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        url: &str,
        content_type: &str,
        secret: Option<&str>,
        active: bool,
        events: &[String],
    ) -> Result<Webhook, Error>;

    /// Updates an existing webhook in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    /// * `webhook_id` - GitHub webhook ID (from list_webhooks or create_webhook)
    /// * `url` - Webhook URL
    /// * `content_type` - Content type ("json" or "form")
    /// * `secret` - Optional webhook secret
    /// * `active` - Whether the webhook is active
    /// * `events` - Events that trigger the webhook
    ///
    /// # Returns
    ///
    /// The updated `Webhook` with its configuration.
    ///
    /// # Errors
    ///
    /// * `Error::NotFound` - Webhook ID does not exist
    /// * `Error::InvalidResponse` - API call failed
    /// * `Error::ApiError` - GitHub API error
    ///
    /// # GitHub API
    ///
    /// PATCH /repos/{owner}/{repo}/hooks/{hook_id}
    async fn update_webhook(
        &self,
        owner: &str,
        repo: &str,
        webhook_id: u64,
        url: &str,
        content_type: &str,
        secret: Option<&str>,
        active: bool,
        events: &[String],
    ) -> Result<Webhook, Error>;

    /// Deletes a webhook from a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    /// * `webhook_id` - GitHub webhook ID
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful deletion
    ///
    /// # Errors
    ///
    /// * `Error::NotFound` - Webhook does not exist (may be considered success)
    /// * `Error::InvalidResponse` - API call failed
    ///
    /// # GitHub API
    ///
    /// DELETE /repos/{owner}/{repo}/hooks/{hook_id}
    async fn delete_webhook(&self, owner: &str, repo: &str, webhook_id: u64) -> Result<(), Error>;

    /// Updates an existing label in a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    /// * `name` - Current label name
    /// * `new_name` - New label name (if renaming, otherwise same as `name`)
    /// * `color` - New label color (hex code without #)
    /// * `description` - New label description
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful update
    ///
    /// # Errors
    ///
    /// * `Error::NotFound` - Label does not exist
    /// * `Error::InvalidResponse` - API call failed
    /// * `Error::ApiError` - GitHub API error
    ///
    /// # GitHub API
    ///
    /// PATCH /repos/{owner}/{repo}/labels/{name}
    async fn update_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        new_name: &str,
        color: &str,
        description: &str,
    ) -> Result<(), Error>;

    /// Deletes a label from a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organization or user)
    /// * `repo` - Repository name
    /// * `name` - Label name to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful deletion
    ///
    /// # Errors
    ///
    /// * `Error::NotFound` - Label does not exist (may be considered success)
    /// * `Error::InvalidResponse` - API call failed
    ///
    /// # GitHub API
    ///
    /// DELETE /repos/{owner}/{repo}/labels/{name}
    async fn delete_label(&self, owner: &str, repo: &str, name: &str) -> Result<(), Error>;
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

/// Checks if an octocrab error indicates a label already exists (HTTP 422 with specific message).
fn is_label_already_exists_error(e: &octocrab::Error) -> bool {
    match e {
        octocrab::Error::GitHub { source, .. } => {
            // GitHub returns 422 Unprocessable Entity when a label with the same name already exists
            // Check both status code and error message
            let is_422 = source.status_code == http::StatusCode::UNPROCESSABLE_ENTITY;
            let msg_lower = source.message.to_lowercase();
            let has_already_exists =
                msg_lower.contains("already exists") || msg_lower.contains("already_exists");

            // Also check for common validation error patterns from GitHub
            let has_validation_error = msg_lower.contains("validation failed")
                && (msg_lower.contains("label") || msg_lower.contains("name"));

            is_422 && (has_already_exists || has_validation_error)
        }
        _ => false,
    }
}

/// Checks if an octocrab error is a 404 Not Found error.
///
/// # Arguments
///
/// * `e` - The octocrab error to check
///
/// # Returns
///
/// `true` if the error is a 404 Not Found, `false` otherwise
fn is_not_found_error(e: &octocrab::Error) -> bool {
    match e {
        octocrab::Error::GitHub { source, .. } => {
            // Check if the error message or status indicates 404
            source.message.contains("404") || source.status_code == http::StatusCode::NOT_FOUND
        }
        _ => false,
    }
}
