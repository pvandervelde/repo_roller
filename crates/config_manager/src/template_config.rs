//! Template-specific configuration.
//!
//! TemplateConfig defines the configuration embedded in template repositories,
//! specifying repository settings, required variables, and repository type policies.
//!
//! # Configuration Hierarchy
//!
//! In the four-level hierarchy:
//! - **Template** ← This level (highest precedence)
//! - Team
//! - Repository Type
//! - Global (lowest precedence)
//!
//! Template configurations:
//! - Have highest precedence (override all other levels)
//! - Define repository type with policy controls
//! - Specify template variables for customization
//! - Use simple TOML format (values auto-wrap with `override_allowed = true`)
//! - Support additive merging for collections (labels, webhooks, apps, environments)
//! - Stored in `.reporoller/template.toml` in template repositories
//!
//! # TOML Format
//!
//! Template configurations use metadata and optional settings:
//!
//! ```toml
//! # .reporoller/template.toml
//!
//! [template]
//! name = "rust-microservice"
//! description = "Production-ready Rust microservice"
//! author = "Platform Team"
//! tags = ["rust", "microservice", "backend"]
//! default_visibility = "private"  # Optional: "public", "private", "internal"
//!
//! # Repository type specification
//! [repository_type]
//! type = "service"
//! policy = "fixed"  # or "preferable" to allow user override
//!
//! [repository]
//! has_wiki = false
//! security_advisories = true
//!
//! [pull_requests]
//! required_approving_review_count = 2
//!
//! # Template variables
//! [variables.service_name]
//! description = "Name of the microservice"
//! example = "user-service"
//! required = true
//!
//! [[github_apps]]
//! app_id = 55555
//! permissions = { actions = "write", deployments = "write" }
//! ```
//!
//! See: specs/design/organization-repository-settings.md

use crate::settings::{
    BranchProtectionSettings, EnvironmentConfig, GitHubAppConfig, LabelConfig, PullRequestSettings,
    RepositorySettings, WebhookConfig,
};
use crate::visibility::RepositoryVisibility;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template-specific configuration embedded in template repositories.
///
/// Defines the configuration requirements and defaults for repositories
/// created from this template. Templates have the highest precedence in
/// the configuration hierarchy.
///
/// # Examples
///
/// ```rust
/// use config_manager::template_config::{TemplateConfig, TemplateMetadata};
///
/// let toml = r#"
///     [template]
///     name = "rust-library"
///     description = "Rust library template"
///     author = "Platform Team"
///     tags = ["rust", "library"]
///
///     [repository]
///     wiki = false
/// "#;
///
/// let config: TemplateConfig = toml::from_str(toml).expect("Failed to parse");
/// assert_eq!(config.template.name, "rust-library");
/// assert!(config.repository.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template metadata (required).
    ///
    /// Provides information about the template, including name, description,
    /// author, and tags for categorization.
    pub template: TemplateMetadata,

    /// Repository type specification (optional).
    ///
    /// Defines the repository type this template creates and whether users
    /// can override it during repository creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypeSpec>,

    /// Template variables (optional).
    ///
    /// Variables that users must provide when creating a repository from
    /// this template. Used for template processing and customization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, TemplateVariable>>,

    /// Repository feature settings (optional).
    ///
    /// Template-specific repository settings that override team and global defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySettings>,

    /// Pull request configuration (optional).
    ///
    /// Template-specific PR policies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_requests: Option<PullRequestSettings>,

    /// Branch protection settings (optional).
    ///
    /// Template-specific branch protection rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_protection: Option<BranchProtectionSettings>,

    /// Template-specific labels (additive).
    ///
    /// Labels defined here are added to labels from other configuration levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelConfig>>,

    /// Template-specific webhooks (additive).
    ///
    /// Webhooks defined here are added to webhooks from other levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<WebhookConfig>>,

    /// Template-specific environments (additive).
    ///
    /// Environments defined here are added to environments from other levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<Vec<EnvironmentConfig>>,

    /// Template-specific GitHub Apps (additive).
    ///
    /// Apps defined here are added to apps from other levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_apps: Option<Vec<GitHubAppConfig>>,

    /// Default visibility for repositories created from this template (optional).
    ///
    /// Specifies the preferred visibility level (Public, Private, or Internal)
    /// for repositories created from this template. This serves as a template
    /// default in the visibility resolution hierarchy:
    ///
    /// 1. Organization Policy (highest priority)
    /// 2. User Preference
    /// 3. **Template Default** ← This field
    /// 4. System Default (Private)
    ///
    /// If not specified, falls back to system default (Private). The template
    /// default is subject to organization visibility policies and GitHub platform
    /// constraints (e.g., Internal requires GitHub Enterprise).
    ///
    /// # TOML Format
    ///
    /// ```toml
    /// [template]
    /// name = "rust-library"
    /// description = "Rust library template"
    /// author = "Platform Team"
    /// default_visibility = "private"  # Options: "public", "private", "internal"
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use config_manager::{TemplateConfig, RepositoryVisibility};
    ///
    /// let toml = r#"
    ///     [template]
    ///     name = "public-docs-template"
    ///     description = "Public documentation template"
    ///     author = "Docs Team"
    ///     default_visibility = "public"
    /// "#;
    ///
    /// let config: TemplateConfig = toml::from_str(toml).expect("Parse failed");
    /// assert_eq!(config.default_visibility, Some(RepositoryVisibility::Public));
    /// ```
    ///
    /// # Backward Compatibility
    ///
    /// Templates without this field continue to work normally - visibility
    /// resolution falls through to system default (Private).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_visibility: Option<RepositoryVisibility>,
}

/// Template metadata providing information about the template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name.
    pub name: String,

    /// Human-readable description of the template.
    pub description: String,

    /// Template author or owning team.
    pub author: String,

    /// Tags for template categorization and discovery.
    pub tags: Vec<String>,
}

/// Repository type specification for the template.
///
/// Defines which repository type this template creates and whether
/// users can override it during repository creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryTypeSpec {
    /// The repository type this template creates.
    ///
    /// Must match a defined repository type in the organization's
    /// configuration repository.
    #[serde(rename = "type")]
    pub repository_type: String,

    /// Policy controlling whether users can override the repository type.
    pub policy: RepositoryTypePolicy,
}

/// Policy for repository type override during repository creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryTypePolicy {
    /// User cannot override the repository type.
    ///
    /// The template always creates repositories of the specified type.
    Fixed,

    /// User can override the repository type during creation.
    ///
    /// The template suggests a preferred type but allows user choice.
    Preferable,
}

/// Template variable definition for user-provided values.
///
/// Defines a variable that users must or can provide when creating
/// a repository from this template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Human-readable description of the variable.
    pub description: String,

    /// Example value for the variable (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,

    /// Whether the variable is required (optional, defaults to false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Regex pattern for value validation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Minimum length for string values (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length for string values (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// List of allowed values (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,

    /// Default value if not provided by user (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[cfg(test)]
#[path = "template_config_tests.rs"]
mod tests;
