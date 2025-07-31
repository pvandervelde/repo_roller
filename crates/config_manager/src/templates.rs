//! Template configuration structures.
//!
//! This module provides template-related configuration structures that define
//! how repositories should be created from templates in the organization.

use crate::settings::{BranchProtectionSettings, PullRequestSettings, RepositorySettings};
use crate::types::{EnvironmentConfig, GitHubAppConfig, LabelConfig, WebhookConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
#[path = "templates_tests.rs"]
mod templates_tests;

/// Specification for repository type requirements.
///
/// Defines constraints and requirements for specific types of repositories
/// (e.g., "service", "library", "documentation") that templates can enforce.
///
/// # Examples
///
/// ```rust
/// use config_manager::templates::RepositoryTypeSpec;
///
/// let spec = RepositoryTypeSpec {
///     required_type: "service".to_string(),
///     description: Some("Microservice repository".to_string()),
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RepositoryTypeSpec {
    /// The required repository type identifier.
    pub required_type: String,

    /// Optional description of this repository type.
    pub description: Option<String>,
}

impl RepositoryTypeSpec {
    /// Creates a new repository type specification.
    ///
    /// # Arguments
    ///
    /// * `required_type` - The required repository type identifier
    /// * `description` - Optional description
    ///
    /// # Returns
    ///
    /// A new `RepositoryTypeSpec` instance.
    pub fn new(required_type: String, description: Option<String>) -> Self {
        Self {
            required_type,
            description,
        }
    }
}

/// Template variable configuration.
///
/// Defines variables that can be substituted in template files during
/// repository creation. Variables can have default values and validation rules.
///
/// # Examples
///
/// ```rust
/// use config_manager::templates::TemplateVariable;
///
/// let var = TemplateVariable {
///     description: Some("Service name".to_string()),
///     default_value: Some("my-service".to_string()),
///     required: true,
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TemplateVariable {
    /// Description of what this variable is used for.
    pub description: Option<String>,

    /// Default value if none is provided.
    pub default_value: Option<String>,

    /// Whether this variable must be provided.
    pub required: bool,
}

impl TemplateVariable {
    /// Creates a new template variable.
    ///
    /// # Arguments
    ///
    /// * `description` - Optional description
    /// * `default_value` - Optional default value
    /// * `required` - Whether the variable is required
    ///
    /// # Returns
    ///
    /// A new `TemplateVariable` instance.
    pub fn new(description: Option<String>, default_value: Option<String>, required: bool) -> Self {
        Self {
            description,
            default_value,
            required,
        }
    }

    /// Creates a required template variable with no default.
    ///
    /// # Arguments
    ///
    /// * `description` - Optional description
    ///
    /// # Returns
    ///
    /// A new required `TemplateVariable` instance.
    pub fn required(description: Option<String>) -> Self {
        Self::new(description, None, true)
    }

    /// Creates an optional template variable with a default value.
    ///
    /// # Arguments
    ///
    /// * `description` - Optional description
    /// * `default_value` - Default value
    ///
    /// # Returns
    ///
    /// A new optional `TemplateVariable` instance.
    pub fn optional(description: Option<String>, default_value: String) -> Self {
        Self::new(description, Some(default_value), false)
    }
}

/// Template metadata and identification.
///
/// Contains information about the template itself, including its name,
/// description, version, and other metadata used for template management.
///
/// # Examples
///
/// ```rust
/// use config_manager::templates::TemplateMetadata;
///
/// let metadata = TemplateMetadata {
///     name: "rust-service".to_string(),
///     description: Some("Rust microservice template".to_string()),
///     version: Some("1.0.0".to_string()),
///     author: Some("Platform Team".to_string()),
///     tags: Some(vec!["rust".to_string(), "service".to_string()]),
/// };
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TemplateMetadata {
    /// Template name/identifier.
    pub name: String,

    /// Human-readable description of the template.
    pub description: Option<String>,

    /// Template version.
    pub version: Option<String>,

    /// Template author or maintainer.
    pub author: Option<String>,

    /// Tags for categorizing the template.
    pub tags: Option<Vec<String>>,
}

impl TemplateMetadata {
    /// Creates new template metadata.
    ///
    /// # Arguments
    ///
    /// * `name` - Template name/identifier
    ///
    /// # Returns
    ///
    /// A new `TemplateMetadata` instance with only the name set.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            version: None,
            author: None,
            tags: None,
        }
    }

    /// Sets the description.
    ///
    /// # Arguments
    ///
    /// * `description` - Template description
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the version.
    ///
    /// # Arguments
    ///
    /// * `version` - Template version
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the author.
    ///
    /// # Arguments
    ///
    /// * `author` - Template author
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the tags.
    ///
    /// # Arguments
    ///
    /// * `tags` - Template tags
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

/// Complete template configuration.
///
/// This structure defines a complete template configuration that includes
/// all the settings, metadata, and constraints needed to create repositories
/// from the template. Templates sit at the top of the configuration hierarchy
/// and can override or add to settings from lower levels.
///
/// # Examples
///
/// ```rust
/// use config_manager::templates::{TemplateConfig, TemplateMetadata};
///
/// let metadata = TemplateMetadata::new("my-template".to_string());
/// let config = TemplateConfig::new(metadata);
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct TemplateConfig {
    /// Template metadata and identification.
    template: TemplateMetadata,

    /// Repository type specification and constraints.
    repository_type: Option<RepositoryTypeSpec>,

    /// Repository feature settings.
    repository: Option<RepositorySettings>,

    /// Pull request settings and policies.
    pull_requests: Option<PullRequestSettings>,

    /// Branch protection rules.
    branch_protection: Option<BranchProtectionSettings>,

    /// Repository labels to create.
    labels: Option<Vec<LabelConfig>>,

    /// Webhook configurations.
    webhooks: Option<Vec<WebhookConfig>>,

    /// GitHub Apps to install.
    github_apps: Option<Vec<GitHubAppConfig>>,

    /// Environment configurations.
    environments: Option<Vec<EnvironmentConfig>>,

    /// Template variables for substitution.
    variables: Option<HashMap<String, TemplateVariable>>,
}

impl TemplateConfig {
    /// Creates a new template configuration with the given metadata.
    ///
    /// All configuration sections are initially `None` and can be set
    /// using the various setter methods or by direct field assignment.
    ///
    /// # Arguments
    ///
    /// * `template` - Template metadata
    ///
    /// # Returns
    ///
    /// A new `TemplateConfig` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::templates::{TemplateConfig, TemplateMetadata};
    ///
    /// let metadata = TemplateMetadata::new("rust-service".to_string());
    /// let config = TemplateConfig::new(metadata);
    /// assert_eq!(config.template().name, "rust-service");
    /// ```
    pub fn new(template: TemplateMetadata) -> Self {
        Self {
            template,
            repository_type: None,
            repository: None,
            pull_requests: None,
            branch_protection: None,
            labels: None,
            webhooks: None,
            github_apps: None,
            environments: None,
            variables: None,
        }
    }

    /// Gets the template metadata.
    ///
    /// # Returns
    ///
    /// A reference to the `TemplateMetadata`.
    pub fn template(&self) -> &TemplateMetadata {
        &self.template
    }

    /// Gets the repository type specification if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `RepositoryTypeSpec`.
    pub fn repository_type(&self) -> Option<&RepositoryTypeSpec> {
        self.repository_type.as_ref()
    }

    /// Sets the repository type specification.
    ///
    /// # Arguments
    ///
    /// * `repository_type` - Optional repository type specification
    pub fn set_repository_type(&mut self, repository_type: Option<RepositoryTypeSpec>) {
        self.repository_type = repository_type;
    }

    /// Gets the repository settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `RepositorySettings`.
    pub fn repository(&self) -> Option<&RepositorySettings> {
        self.repository.as_ref()
    }

    /// Sets the repository settings.
    ///
    /// # Arguments
    ///
    /// * `repository` - Optional repository settings
    pub fn set_repository(&mut self, repository: Option<RepositorySettings>) {
        self.repository = repository;
    }

    /// Gets the pull request settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `PullRequestSettings`.
    pub fn pull_requests(&self) -> Option<&PullRequestSettings> {
        self.pull_requests.as_ref()
    }

    /// Sets the pull request settings.
    ///
    /// # Arguments
    ///
    /// * `pull_requests` - Optional pull request settings
    pub fn set_pull_requests(&mut self, pull_requests: Option<PullRequestSettings>) {
        self.pull_requests = pull_requests;
    }

    /// Gets the branch protection settings if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the `BranchProtectionSettings`.
    pub fn branch_protection(&self) -> Option<&BranchProtectionSettings> {
        self.branch_protection.as_ref()
    }

    /// Sets the branch protection settings.
    ///
    /// # Arguments
    ///
    /// * `branch_protection` - Optional branch protection settings
    pub fn set_branch_protection(&mut self, branch_protection: Option<BranchProtectionSettings>) {
        self.branch_protection = branch_protection;
    }

    /// Gets the labels if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `LabelConfig`.
    pub fn labels(&self) -> Option<&[LabelConfig]> {
        self.labels.as_deref()
    }

    /// Sets the labels.
    ///
    /// # Arguments
    ///
    /// * `labels` - Optional vector of label configurations
    pub fn set_labels(&mut self, labels: Option<Vec<LabelConfig>>) {
        self.labels = labels;
    }

    /// Gets the webhooks if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `WebhookConfig`.
    pub fn webhooks(&self) -> Option<&[WebhookConfig]> {
        self.webhooks.as_deref()
    }

    /// Sets the webhooks.
    ///
    /// # Arguments
    ///
    /// * `webhooks` - Optional vector of webhook configurations
    pub fn set_webhooks(&mut self, webhooks: Option<Vec<WebhookConfig>>) {
        self.webhooks = webhooks;
    }

    /// Gets the GitHub Apps if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `GitHubAppConfig`.
    pub fn github_apps(&self) -> Option<&[GitHubAppConfig]> {
        self.github_apps.as_deref()
    }

    /// Sets the GitHub Apps.
    ///
    /// # Arguments
    ///
    /// * `github_apps` - Optional vector of GitHub App configurations
    pub fn set_github_apps(&mut self, github_apps: Option<Vec<GitHubAppConfig>>) {
        self.github_apps = github_apps;
    }

    /// Gets the environments if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the vector of `EnvironmentConfig`.
    pub fn environments(&self) -> Option<&[EnvironmentConfig]> {
        self.environments.as_deref()
    }

    /// Sets the environments.
    ///
    /// # Arguments
    ///
    /// * `environments` - Optional vector of environment configurations
    pub fn set_environments(&mut self, environments: Option<Vec<EnvironmentConfig>>) {
        self.environments = environments;
    }

    /// Gets the template variables if configured.
    ///
    /// # Returns
    ///
    /// An optional reference to the HashMap of variable name to `TemplateVariable`.
    pub fn variables(&self) -> Option<&HashMap<String, TemplateVariable>> {
        self.variables.as_ref()
    }

    /// Sets the template variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - Optional HashMap of variable configurations
    pub fn set_variables(&mut self, variables: Option<HashMap<String, TemplateVariable>>) {
        self.variables = variables;
    }

    /// Adds a single template variable.
    ///
    /// If the variables HashMap doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `variable` - Variable configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::templates::{TemplateConfig, TemplateMetadata, TemplateVariable};
    ///
    /// let metadata = TemplateMetadata::new("test".to_string());
    /// let mut config = TemplateConfig::new(metadata);
    /// config.add_variable("service_name".to_string(), TemplateVariable::required(Some("Service name".to_string())));
    /// ```
    pub fn add_variable(&mut self, name: String, variable: TemplateVariable) {
        if self.variables.is_none() {
            self.variables = Some(HashMap::new());
        }
        if let Some(vars) = &mut self.variables {
            vars.insert(name, variable);
        }
    }

    /// Checks if the template has any configuration sections defined.
    ///
    /// # Returns
    ///
    /// `true` if any configuration sections are defined, `false` if all are `None`.
    pub fn has_configuration(&self) -> bool {
        self.repository_type.is_some()
            || self.repository.is_some()
            || self.pull_requests.is_some()
            || self.branch_protection.is_some()
            || self.labels.is_some()
            || self.webhooks.is_some()
            || self.github_apps.is_some()
            || self.environments.is_some()
            || self.variables.is_some()
    }

    /// Gets a count of configured sections.
    ///
    /// # Returns
    ///
    /// The number of configuration sections that are not `None`.
    pub fn configuration_count(&self) -> usize {
        let mut count = 0;
        if self.repository_type.is_some() {
            count += 1;
        }
        if self.repository.is_some() {
            count += 1;
        }
        if self.pull_requests.is_some() {
            count += 1;
        }
        if self.branch_protection.is_some() {
            count += 1;
        }
        if self.labels.is_some() {
            count += 1;
        }
        if self.webhooks.is_some() {
            count += 1;
        }
        if self.github_apps.is_some() {
            count += 1;
        }
        if self.environments.is_some() {
            count += 1;
        }
        if self.variables.is_some() {
            count += 1;
        }
        count
    }
}
