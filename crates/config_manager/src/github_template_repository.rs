//! GitHub implementation of TemplateRepository trait.
//!
//! Loads template configurations from `.reporoller/template.toml` files
//! in GitHub template repositories using the GitHub Contents API.

use crate::{ConfigurationError, ConfigurationResult, TemplateConfig, TemplateRepository};
use async_trait::async_trait;
use github_client::GitHubClient;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// GitHub implementation of TemplateRepository.
///
/// Loads template configurations from `.reporoller/template.toml` files
/// in GitHub template repositories using the GitHub Contents API.
///
/// # Examples
///
/// ```rust,no_run
/// use config_manager::{GitHubTemplateRepository, TemplateRepository};
/// use github_client::GitHubClient;
/// use std::sync::Arc;
///
/// # async fn example(github_client: Arc<GitHubClient>) -> Result<(), Box<dyn std::error::Error>> {
/// let repo = GitHubTemplateRepository::new(github_client);
///
/// let config = repo
///     .load_template_config("my-org", "rust-service-template")
///     .await?;
///
/// println!("Template: {}", config.template.name);
/// # Ok(())
/// # }
/// ```
pub struct GitHubTemplateRepository {
    /// GitHub client for API access
    github_client: Arc<GitHubClient>,
}

impl GitHubTemplateRepository {
    /// Create a new GitHub template repository accessor.
    ///
    /// # Arguments
    ///
    /// * `github_client` - GitHub client for API access
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use config_manager::GitHubTemplateRepository;
    /// use github_client::GitHubClient;
    /// use std::sync::Arc;
    ///
    /// # fn example(github_client: Arc<GitHubClient>) {
    /// let repo = GitHubTemplateRepository::new(github_client);
    /// # }
    /// ```
    pub fn new(github_client: Arc<GitHubClient>) -> Self {
        Self { github_client }
    }
}

#[async_trait]
impl TemplateRepository for GitHubTemplateRepository {
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig> {
        debug!("Loading template configuration: {}/{}", org, template_name);

        // Step 1: Check if template repository exists
        let exists = self.template_exists(org, template_name).await?;
        if !exists {
            return Err(ConfigurationError::TemplateNotFound {
                org: org.to_string(),
                template: template_name.to_string(),
            });
        }

        // Step 2: Fetch .reporoller/template.toml file
        let config_path = ".reporoller/template.toml";
        let content = self
            .github_client
            .get_file_content(org, template_name, config_path)
            .await
            .map_err(|e| {
                let error_msg = e.to_string();
                // Map GitHub 404 to TemplateConfigurationMissing
                if error_msg.contains("404") || error_msg.contains("Not Found") {
                    warn!(
                        "Template configuration file missing: {}/{} ({})",
                        org, template_name, config_path
                    );
                    ConfigurationError::TemplateConfigurationMissing {
                        org: org.to_string(),
                        template: template_name.to_string(),
                    }
                } else {
                    warn!(
                        "Failed to access template configuration: {}/{} - {}",
                        org, template_name, e
                    );
                    ConfigurationError::FileAccessError {
                        path: format!("{}/{}/{}", org, template_name, config_path),
                        reason: e.to_string(),
                    }
                }
            })?;

        // Step 3: Parse TOML content
        let config: TemplateConfig = toml::from_str(&content).map_err(|e| {
            warn!(
                "Failed to parse template configuration in {}/{}: {}",
                org, template_name, e
            );
            ConfigurationError::ParseError {
                reason: format!(
                    "Failed to parse template configuration in {}/{}: {}",
                    org, template_name, e
                ),
            }
        })?;

        info!(
            "Template configuration loaded: {}/{} ({})",
            org, template_name, config.template.name
        );

        Ok(config)
    }

    async fn template_exists(&self, org: &str, template_name: &str) -> ConfigurationResult<bool> {
        debug!("Checking if template exists: {}/{}", org, template_name);

        // Use GitHub API to check if repository exists
        match self.github_client.get_repository(org, template_name).await {
            Ok(_) => {
                debug!("Template exists: {}/{}", org, template_name);
                Ok(true)
            }
            Err(e) => {
                let error_msg = e.to_string();
                // 404 means doesn't exist (return false, not error)
                if error_msg.contains("404") || error_msg.contains("Not Found") {
                    debug!("Template not found: {}/{}", org, template_name);
                    Ok(false)
                } else {
                    // Other errors are actual failures
                    warn!(
                        "Failed to check template existence: {}/{} - {}",
                        org, template_name, e
                    );
                    Err(ConfigurationError::FileAccessError {
                        path: format!("{}/{}", org, template_name),
                        reason: e.to_string(),
                    })
                }
            }
        }
    }
}
