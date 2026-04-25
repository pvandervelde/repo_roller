//! Template inspection and validation commands for the RepoRoller CLI.
//!
//! This module provides commands for inspecting and validating templates
//! in the organization's metadata repository. It allows users to:
//! - Get detailed information about a template
//! - Validate template configuration and structure
//!
//! # Examples
//!
//! ```bash
//! # Get template information
//! repo-roller template info --org myorg --template rust-library
//!
//! # Validate template
//! repo-roller template validate --org myorg --template rust-library
//! ```

use clap::Subcommand;
use colored::Colorize;
use config_manager::{
    ConfigurationError, GitHubMetadataProvider, MetadataProviderConfig, MetadataRepositoryProvider,
    TemplateConfig, TemplateVariable,
};
use github_client::GitHubClient;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::commands::auth_cmd::{
    KEY_RING_APP_ID, KEY_RING_APP_PRIVATE_KEY_PATH, KEY_RING_SERVICE_NAME,
};
use crate::config::{get_config_path, AppConfig, DEFAULT_METADATA_REPOSITORY_NAME};
use crate::errors::Error;

// ============================================================================
// CLI-Specific Types
// ============================================================================
// GENERATED FROM: specs/interfaces/cli-template-operations.md

/// Template information for CLI display.
///
/// This is a CLI-specific view that combines template metadata with
/// configuration details in a format suitable for command-line output.
///
/// See: specs/interfaces/cli-template-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name (repository name).
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// Template author or owning team.
    pub author: String,

    /// Tags for categorization.
    pub tags: Vec<String>,

    /// Repository type this template creates (if specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypeInfo>,

    /// Template variables that users must provide.
    pub variables: Vec<TemplateVariableInfo>,

    /// Number of configuration sections defined.
    pub configuration_sections: usize,
}

/// Repository type information for CLI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTypeInfo {
    /// Repository type name.
    pub type_name: String,

    /// Policy: "fixed" or "preferable".
    pub policy: String,
}

/// Template variable information for CLI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariableInfo {
    /// Variable name.
    pub name: String,

    /// Variable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether variable is required.
    pub required: bool,

    /// Default value (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Example value (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Result of template validation.
///
/// Contains validation status and detailed diagnostics about any issues found.
///
/// See: specs/interfaces/cli-template-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateValidationResult {
    /// Template name being validated.
    pub template_name: String,

    /// Overall validation status.
    pub valid: bool,

    /// Validation issues found (empty if valid).
    pub issues: Vec<ValidationIssue>,

    /// Warnings that don't prevent use but should be addressed.
    pub warnings: Vec<ValidationWarning>,
}

/// Individual validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity: "error" or "warning".
    pub severity: String,

    /// Location of issue (e.g., "template.toml", "variables.service_name").
    pub location: String,

    /// Human-readable issue description.
    pub message: String,
}

/// Validation warning (non-blocking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning category (e.g., "best_practice", "deprecated").
    pub category: String,

    /// Warning message.
    pub message: String,
}

// ============================================================================
// Command Definitions
// ============================================================================

/// Template inspection and validation subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum TemplateCommands {
    /// Get detailed information about a template.
    ///
    /// Displays template metadata including name, description, author, tags,
    /// repository type specification, and defined variables.
    Info {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Template name.
        #[arg(long)]
        template: String,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Validate a template configuration.
    ///
    /// Validates the template configuration file (template.toml) including
    /// structure, variable definitions, and repository type specification.
    ///
    /// Three invocation modes:
    /// 1. `--path <DIR>` — validates the local directory directly (no GitHub call needed).
    /// 2. `--org ORG --template NAME` — clones the template from GitHub to a temp directory
    ///    then validates locally.
    /// 3. Both `--path` and `--org`/`--template` — uses the local path; if the path does not
    ///    exist the remote template is cloned there.
    Validate {
        /// Organization name (optional when --path is given).
        #[arg(long)]
        org: Option<String>,

        /// Template name to validate (optional when --path is given).
        #[arg(long)]
        template: Option<String>,

        /// Local path to a template repository directory.
        ///
        /// When specified and the directory exists, validates the template locally
        /// without requiring GitHub credentials for structural checks. If absent or
        /// the directory does not exist, --org and --template are used to clone
        /// the remote template to a temporary directory first.
        #[arg(long)]
        path: Option<String>,

        /// Output format (json or pretty).
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

/// Executes the specified template command.
///
/// # Arguments
///
/// * `cmd` - The template command to execute
///
/// # Returns
///
/// Returns `Ok(())` on successful command execution, or an `Error` if
/// the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Authentication fails (GitHub credentials not available)
/// - The organization metadata repository is not found
/// - The specified template is not found
/// - Template configuration is invalid
/// - GitHub API operations fail
/// - Output formatting fails
#[instrument]
pub async fn execute(cmd: &TemplateCommands) -> Result<(), Error> {
    match cmd {
        TemplateCommands::Info {
            org,
            template,
            format,
        } => template_info(org, template, format).await,
        TemplateCommands::Validate {
            org,
            template,
            path,
            format,
        } => template_validate(org.as_deref(), template.as_deref(), path.as_deref(), format).await,
    }
}

/// Creates an authenticated metadata provider wrapped in Arc.
///
/// Loads GitHub App credentials from the system keyring and creates
/// an authenticated GitHubMetadataProvider instance.
async fn create_metadata_provider() -> Result<Arc<dyn MetadataRepositoryProvider>, Error> {
    // Load application config to get metadata repository name
    let config_path = get_config_path(None);
    let app_config = AppConfig::load(&config_path).unwrap_or_else(|_| AppConfig::default());

    // Load GitHub App ID from keyring
    let app_id_entry = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_ID)
        .map_err(|e| Error::Auth(format!("Failed to access keyring for app ID: {}", e)))?;

    let app_id_str = app_id_entry
        .get_password()
        .map_err(|e| Error::Auth(format!("Failed to get app ID from keyring: {}. Run 'repo-roller auth setup' to configure GitHub App credentials.", e)))?;

    let app_id: u64 = app_id_str
        .parse()
        .map_err(|e| Error::Auth(format!("Invalid app ID format: {}", e)))?;

    // Load private key path from keyring
    let key_path_entry = Entry::new(KEY_RING_SERVICE_NAME, KEY_RING_APP_PRIVATE_KEY_PATH)
        .map_err(|e| Error::Auth(format!("Failed to access keyring for key path: {}", e)))?;

    let key_path = key_path_entry
        .get_password()
        .map_err(|e| Error::Auth(format!("Failed to get key path from keyring: {}. Run 'repo-roller auth setup' to configure GitHub App credentials.", e)))?;

    // Read private key file
    let private_key = std::fs::read_to_string(&key_path).map_err(|e| {
        Error::Auth(format!(
            "Failed to read private key from {}: {}",
            key_path, e
        ))
    })?;

    // Create authenticated GitHub client
    let octocrab = github_client::create_app_client(app_id, &private_key)
        .await
        .map_err(|e| Error::Auth(format!("Failed to create GitHub App client: {}. Verify your GitHub App credentials with 'repo-roller auth setup'.", e)))?;

    let github_client = GitHubClient::new(octocrab);

    // Create metadata provider config
    let metadata_repo_name = if app_config.organization.metadata_repository_name.is_empty() {
        DEFAULT_METADATA_REPOSITORY_NAME
    } else {
        &app_config.organization.metadata_repository_name
    };

    let config = MetadataProviderConfig::explicit(metadata_repo_name);
    let provider = GitHubMetadataProvider::new(github_client, config);

    Ok(Arc::new(provider))
}

// ============================================================================
// Translation Functions
// ============================================================================
// GENERATED FROM: specs/interfaces/cli-template-operations.md

/// Convert business domain `TemplateConfig` to CLI `TemplateInfo`.
///
/// # Arguments
///
/// * `config` - Template configuration from business domain
///
/// # Returns
///
/// Returns CLI-friendly `TemplateInfo` representation.
///
/// See: specs/interfaces/cli-template-operations.md
fn template_config_to_info(config: TemplateConfig) -> TemplateInfo {
    // Count non-None configuration sections
    let mut config_sections = 0;
    if config.repository.is_some() {
        config_sections += 1;
    }
    if config.pull_requests.is_some() {
        config_sections += 1;
    }
    if config.branch_protection.is_some() {
        config_sections += 1;
    }
    if config.labels.is_some() {
        config_sections += 1;
    }
    if config.webhooks.is_some() {
        config_sections += 1;
    }
    if config.environments.is_some() {
        config_sections += 1;
    }
    if config.github_apps.is_some() {
        config_sections += 1;
    }

    // Convert repository type spec to CLI format
    let repository_type = config.repository_type.map(|rt| RepositoryTypeInfo {
        type_name: rt.repository_type,
        policy: match rt.policy {
            config_manager::RepositoryTypePolicy::Fixed => "fixed".to_string(),
            config_manager::RepositoryTypePolicy::Preferable => "preferable".to_string(),
        },
    });

    // Convert variables to CLI format
    let variables = config
        .variables
        .unwrap_or_default()
        .into_iter()
        .map(|(name, var)| template_variable_to_info(name, var))
        .collect();

    TemplateInfo {
        name: config.template.name,
        description: config.template.description,
        author: config.template.author,
        tags: config.template.tags,
        repository_type,
        variables,
        configuration_sections: config_sections,
    }
}

/// Convert business domain `TemplateVariable` to CLI `TemplateVariableInfo`.
fn template_variable_to_info(name: String, var: TemplateVariable) -> TemplateVariableInfo {
    TemplateVariableInfo {
        name,
        description: Some(var.description),
        required: var.required.unwrap_or(false),
        default_value: var.default,
        example: var.example,
    }
}

// ============================================================================
// Core Template Operations
// ============================================================================
// GENERATED FROM: specs/interfaces/cli-template-operations.md

/// List all available templates for an organization.
///
/// Discovers template repositories and loads their configurations
/// to provide summary information.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns a vector of `TemplateInfo` objects, one per discovered template.
/// Returns empty vector if no templates found.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed or GitHub App not configured
/// * `Error::Config` - Failed to load template configurations
/// * `Error::GitHub` - GitHub API errors during template discovery
///
/// See: specs/interfaces/cli-template-operations.md
#[allow(dead_code)] // Public API used from integration tests; no CLI subcommand for listing yet
pub async fn list_templates(
    org: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<Vec<TemplateInfo>, Error> {
    debug!("Listing templates for organization: {}", org);

    // Get list of template names from provider
    let template_names = provider
        .list_templates(org)
        .await
        .map_err(|e| Error::Config(format!("Failed to list templates: {}", e)))?;

    debug!(
        "Found {} template(s) for organization {}",
        template_names.len(),
        org
    );

    // Load configuration for each template and convert to TemplateInfo
    let mut templates = Vec::new();
    for name in template_names {
        match provider.load_template_configuration(org, &name).await {
            Ok(config) => {
                debug!("Successfully loaded configuration for template: {}", name);
                let info = template_config_to_info(config);
                templates.push(info);
            }
            Err(e) => {
                // Log warning and skip templates that fail to load
                // This allows listing to continue even if some templates have issues
                tracing::warn!(
                    "Skipping template '{}' due to configuration error: {}",
                    name,
                    e
                );
            }
        }
    }

    debug!(
        "Successfully processed {} template(s) for organization {}",
        templates.len(),
        org
    );

    Ok(templates)
}

/// Get detailed information about a specific template.
///
/// Loads the complete template configuration and formats it for CLI display.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template_name` - Template repository name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns complete `TemplateInfo` for the specified template.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed
/// * `Error::Config` - Template not found or configuration invalid
/// * `Error::GitHub` - GitHub API errors
///
/// See: specs/interfaces/cli-template-operations.md
pub async fn get_template_info(
    org: &str,
    template_name: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<TemplateInfo, Error> {
    debug!("Getting template info for {}/{}", org, template_name);

    // Load template configuration from provider
    let config = provider
        .load_template_configuration(org, template_name)
        .await
        .map_err(|e| match e {
            ConfigurationError::TemplateNotFound { .. } => Error::Config(format!(
                "Template '{}' not found in organization '{}'",
                template_name, org
            )),
            ConfigurationError::TemplateConfigurationMissing { .. } => Error::Config(format!(
                "Template '{}' exists but is missing .reporoller/template.toml configuration file",
                template_name
            )),
            ConfigurationError::ParseError { reason } => Error::Config(format!(
                "Failed to parse template configuration for '{}': {}",
                template_name, reason
            )),
            _ => Error::Config(format!(
                "Failed to load template '{}': {}",
                template_name, e
            )),
        })?;

    debug!(
        "Successfully loaded configuration for template: {}",
        template_name
    );

    // Convert to CLI-friendly format
    let info = template_config_to_info(config);

    Ok(info)
}

/// Validate a template's structure and configuration.
///
/// Performs comprehensive validation including:
/// - Template repository accessibility
/// - `.reporoller/template.toml` existence and parse validity
/// - Required metadata fields presence
/// - Variable definition completeness
/// - Repository type reference validity (if type specified)
/// - Configuration consistency
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template_name` - Template repository name
/// * `provider` - Metadata repository provider (authenticated)
///
/// # Returns
///
/// Returns `TemplateValidationResult` with validation status and any issues found.
///
/// # Errors
///
/// * `Error::Auth` - Authentication failed
/// * `Error::GitHub` - GitHub API errors (network, permissions)
///
/// Note: Template configuration errors are returned in the validation result,
/// not as function errors.
///
/// See: specs/interfaces/cli-template-operations.md
#[allow(dead_code)] // used from tests via `use super::*`
pub async fn validate_template(
    org: &str,
    template_name: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<TemplateValidationResult, Error> {
    debug!("Validating template {}/{}", org, template_name);

    let mut issues = Vec::new();

    // Try to load template configuration
    let config = match provider
        .load_template_configuration(org, template_name)
        .await
    {
        Ok(cfg) => cfg,
        Err(e) => {
            // Template loading failed - return validation result with error
            let issue = match e {
                ConfigurationError::TemplateNotFound { .. } => ValidationIssue {
                    severity: "error".to_string(),
                    location: "template".to_string(),
                    message: format!(
                        "Template '{}' not found in organization '{}'",
                        template_name, org
                    ),
                },
                ConfigurationError::TemplateConfigurationMissing { .. } => ValidationIssue {
                    severity: "error".to_string(),
                    location: ".reporoller/template.toml".to_string(),
                    message: "Template configuration file (.reporoller/template.toml) is missing"
                        .to_string(),
                },
                ConfigurationError::ParseError { reason } => ValidationIssue {
                    severity: "error".to_string(),
                    location: ".reporoller/template.toml".to_string(),
                    message: format!("Failed to parse template configuration: {}", reason),
                },
                _ => ValidationIssue {
                    severity: "error".to_string(),
                    location: "template".to_string(),
                    message: format!("Failed to load template: {}", e),
                },
            };
            issues.push(issue);

            return Ok(TemplateValidationResult {
                template_name: template_name.to_string(),
                valid: false,
                issues,
                warnings: vec![],
            });
        }
    };

    // Delegate all structural and remote checks to the shared helper.
    validate_loaded_template(template_name, config, Some(provider), Some(org)).await
}

/// Run structural and remote validation checks on an already-loaded `TemplateConfig`.
///
/// This is the shared validation core used by both the remote path (where the config
/// is fetched via a `MetadataRepositoryProvider`) and the local-path / clone path
/// (where the config is parsed from disk).
///
/// # Arguments
///
/// * `template_name` - Template name (used in the returned result).
/// * `config` - Already-parsed template configuration.
/// * `provider` - Optional authenticated provider for remote type-validity checks.
/// * `org` - Optional organization name, required for remote type-validity checks.
///
/// # Returns
///
/// Returns `TemplateValidationResult`. Remote type checks are skipped (with a warning)
/// when `provider` or `org` is `None`.
async fn validate_loaded_template(
    template_name: &str,
    config: TemplateConfig,
    provider: Option<Arc<dyn MetadataRepositoryProvider>>,
    org: Option<&str>,
) -> Result<TemplateValidationResult, Error> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // --- Validate template metadata ---
    if config.template.name.is_empty() {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            location: "template.name".to_string(),
            message: "Template name cannot be empty".to_string(),
        });
    }

    if config.template.description.is_empty() {
        warnings.push(ValidationWarning {
            category: "best_practice".to_string(),
            message: "Template description is empty or missing".to_string(),
        });
    }

    if config.template.author.is_empty() {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            location: "template.author".to_string(),
            message: "Template author cannot be empty".to_string(),
        });
    }

    if config.template.tags.is_empty() {
        warnings.push(ValidationWarning {
            category: "best_practice".to_string(),
            message: "No tags defined for template categorization".to_string(),
        });
    }

    // --- Validate variables ---
    if let Some(ref variables) = config.variables {
        if variables.is_empty() {
            warnings.push(ValidationWarning {
                category: "best_practice".to_string(),
                message: "No variables defined - template is not customizable".to_string(),
            });
        }

        for (var_name, var_def) in variables {
            // Variable names: alphanumeric + underscore only
            if !var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                issues.push(ValidationIssue {
                    severity: "error".to_string(),
                    location: format!("variables.{}", var_name),
                    message: format!(
                        "Variable name '{}' contains invalid characters. Only alphanumeric and underscore allowed.",
                        var_name
                    ),
                });
            }

            // Contradiction: required=true with a default value
            if var_def.required.unwrap_or(false) && var_def.default.is_some() {
                issues.push(ValidationIssue {
                    severity: "error".to_string(),
                    location: format!("variables.{}", var_name),
                    message: format!(
                        "Variable '{}' is marked as required but has a default value (contradiction)",
                        var_name
                    ),
                });
            }

            // Recommendation: required variables should have an example
            if var_def.required.unwrap_or(false) && var_def.example.is_none() {
                warnings.push(ValidationWarning {
                    category: "best_practice".to_string(),
                    message: format!("Required variable '{}' has no example value", var_name),
                });
            }
        }
    } else {
        warnings.push(ValidationWarning {
            category: "best_practice".to_string(),
            message: "No variables defined - template is not customizable".to_string(),
        });
    }

    // --- Validate repository type (remote check when provider + org available) ---
    if let Some(ref repo_type_spec) = config.repository_type {
        if let (Some(provider_ref), Some(org_str)) = (&provider, org) {
            match provider_ref.discover_metadata_repository(org_str).await {
                Ok(metadata_repo) => {
                    match provider_ref
                        .list_available_repository_types(&metadata_repo)
                        .await
                    {
                        Ok(available_types) => {
                            if !available_types.contains(&repo_type_spec.repository_type) {
                                issues.push(ValidationIssue {
                                    severity: "error".to_string(),
                                    location: "repository_type.type".to_string(),
                                    message: format!(
                                        "Repository type '{}' does not exist in organization. Available types: {}",
                                        repo_type_spec.repository_type,
                                        available_types.join(", ")
                                    ),
                                });
                            }
                        }
                        Err(e) => {
                            warnings.push(ValidationWarning {
                                category: "validation_incomplete".to_string(),
                                message: format!(
                                    "Could not verify repository type '{}': {}",
                                    repo_type_spec.repository_type, e
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    warnings.push(ValidationWarning {
                        category: "validation_incomplete".to_string(),
                        message: format!(
                            "Could not discover metadata repository to verify repository type: {}",
                            e
                        ),
                    });
                }
            }
        } else {
            // No provider or no org — skip remote type check with an informational warning.
            warnings.push(ValidationWarning {
                category: "validation_incomplete".to_string(),
                message: format!(
                    "Could not verify repository type '{}': no organization context or credentials available",
                    repo_type_spec.repository_type
                ),
            });
        }
    }

    let valid = issues.is_empty();

    Ok(TemplateValidationResult {
        template_name: template_name.to_string(),
        valid,
        issues,
        warnings,
    })
}

/// Load and parse a `TemplateConfig` from a local template repository directory.
///
/// Reads `.reporoller/template.toml` relative to `path` and deserializes it with
/// `toml::from_str`.
///
/// # Arguments
///
/// * `path` - Root directory of a local template repository.
///
/// # Errors
///
/// * `Error::Config` - The `.reporoller/template.toml` file is missing or the TOML
///   content cannot be parsed.
pub(crate) fn load_template_config_from_path(path: &Path) -> Result<TemplateConfig, Error> {
    let config_path = path.join(".reporoller").join("template.toml");
    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        Error::Config(format!(
            "Failed to read .reporoller/template.toml from {}: {}",
            path.display(),
            e
        ))
    })?;
    toml::from_str::<TemplateConfig>(&content).map_err(|e| {
        Error::Config(format!(
            "Failed to parse .reporoller/template.toml in {}: {}",
            path.display(),
            e
        ))
    })
}

/// Detect the GitHub organization and repository from a local git repository's remote URL.
///
/// Parses `.git/config` in `path` looking for a GitHub remote URL in HTTPS or SSH format:
/// - HTTPS: `https://github.com/ORG/REPO[.git]`
/// - SSH:   `git@github.com:ORG/REPO[.git]`
///
/// Returns the first match found (typically the `origin` remote).
///
/// # Returns
///
/// * `Some((org, repo))` — when a GitHub remote URL is found.
/// * `None` — when no `.git/config` exists, the file cannot be read, or it contains
///   no GitHub remote.
pub(crate) fn detect_github_remote(path: &Path) -> Option<(String, String)> {
    let git_config_path = path.join(".git").join("config");
    let content = std::fs::read_to_string(&git_config_path).ok()?;

    // HTTPS: https://github.com/ORG/REPO[.git]
    let https_re =
        regex::Regex::new(r"https://github\.com/([^/\s]+)/([^/\s]+?)(?:\.git)?(?:\s|$)").ok()?;
    if let Some(caps) = https_re.captures(&content) {
        if let (Some(org), Some(repo)) = (caps.get(1), caps.get(2)) {
            return Some((org.as_str().to_string(), repo.as_str().to_string()));
        }
    }

    // SSH: git@github.com:ORG/REPO[.git]
    let ssh_re =
        regex::Regex::new(r"git@github\.com:([^:/\s]+)/([^/\s]+?)(?:\.git)?(?:\s|$)").ok()?;
    if let Some(caps) = ssh_re.captures(&content) {
        if let (Some(org), Some(repo)) = (caps.get(1), caps.get(2)) {
            return Some((org.as_str().to_string(), repo.as_str().to_string()));
        }
    }

    None
}

/// Shell out to `git clone <url> <dest>` and map failures to `Error::GitHub`.
///
/// Used as the testable inner function behind `clone_template_to_dir`.
/// Tests can call this directly with a local `file://` URL or bare-repo path
/// without touching the network.
///
/// # Errors
///
/// * `Error::GitHub` — `git` cannot be executed or exits with a non-zero status.
pub(crate) fn run_git_clone(url: &str, dest: &Path) -> Result<(), Error> {
    let dest_str = dest.to_str().ok_or_else(|| {
        Error::GitHub("Clone destination path contains non-UTF-8 characters".to_string())
    })?;

    let output = std::process::Command::new("git")
        .args(["clone", url, dest_str])
        .output()
        .map_err(|e| Error::GitHub(format!("Failed to execute git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::GitHub(format!(
            "git clone '{}' failed: {}",
            url,
            stderr.trim()
        )));
    }

    Ok(())
}

/// Clone a GitHub template repository to a local directory.
///
/// Constructs `https://github.com/{org}/{template}` and delegates to [`run_git_clone`].
///
/// # Errors
///
/// * `Error::GitHub` — the clone failed (network error, repository not found,
///   authentication required, etc.).
pub(crate) fn clone_template_to_dir(org: &str, template: &str, dest: &Path) -> Result<(), Error> {
    let url = format!("https://github.com/{}/{}", org, template);
    run_git_clone(&url, dest)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format TemplateInfo for display.
fn format_template_info(info: &TemplateInfo, format: &str) -> Result<String, Error> {
    match format {
        "json" => serde_json::to_string_pretty(info)
            .map_err(|e| Error::Config(format!("Failed to serialize to JSON: {}", e))),
        "pretty" => Ok(format_template_info_pretty(info)),
        _ => Err(Error::InvalidArguments(format!(
            "Invalid format: '{}'. Use 'json' or 'pretty'.",
            format
        ))),
    }
}

/// Format TemplateValidationResult for display.
fn format_validation_result(
    result: &TemplateValidationResult,
    format: &str,
) -> Result<String, Error> {
    match format {
        "json" => serde_json::to_string_pretty(result)
            .map_err(|e| Error::Config(format!("Failed to serialize to JSON: {}", e))),
        "pretty" => Ok(format_validation_result_pretty(result)),
        _ => Err(Error::InvalidArguments(format!(
            "Invalid format: '{}'. Use 'json' or 'pretty'.",
            format
        ))),
    }
}

/// Format TemplateInfo in pretty/human-readable format.
fn format_template_info_pretty(info: &TemplateInfo) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("\n{}\n\n", info.name.bold().bright_cyan()));

    // Description
    output.push_str(&format!("{}: {}\n", "Description".bold(), info.description));
    output.push_str(&format!("{}: {}\n", "Author".bold(), info.author));

    // Tags
    if !info.tags.is_empty() {
        let tags_str = info
            .tags
            .iter()
            .map(|t| t.blue().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("{}: {}\n", "Tags".bold(), tags_str));
    } else {
        output.push_str(&format!("{}: {}\n", "Tags".bold(), "(none)".dimmed()));
    }

    // Repository Type
    if let Some(ref rt) = info.repository_type {
        output.push_str(&format!(
            "{}: {} ({})\n",
            "Repository Type".bold(),
            rt.type_name.green(),
            rt.policy
        ));
    }

    // Variables
    output.push_str(&format!("\n{}\n", "Variables:".bold()));
    if info.variables.is_empty() {
        output.push_str(&format!("  {}\n", "(none defined)".dimmed()));
    } else {
        for var in &info.variables {
            let req_marker = if var.required {
                "[required]".red().to_string()
            } else {
                "[optional]".dimmed().to_string()
            };

            output.push_str(&format!("  {} {}\n", "✓".green(), var.name.bold()));

            if let Some(ref desc) = var.description {
                output.push_str(&format!("    {}\n", desc));
            }

            output.push_str(&format!("    {}\n", req_marker));

            if let Some(ref default) = var.default_value {
                output.push_str(&format!("    {}: {}\n", "Default".dimmed(), default));
            }

            if let Some(ref example) = var.example {
                output.push_str(&format!("    {}: {}\n", "Example".dimmed(), example));
            }

            output.push('\n');
        }
    }

    // Configuration sections
    output.push_str(&format!(
        "{}: {} sections defined\n",
        "Configuration".bold(),
        info.configuration_sections
    ));

    output
}

/// Format TemplateValidationResult in pretty/human-readable format.
fn format_validation_result_pretty(result: &TemplateValidationResult) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "\n{} {}\n\n",
        "Validating template:".bold(),
        result.template_name.bright_cyan()
    ));

    // Overall status
    if result.valid {
        output.push_str(&format!("{}\n\n", "✓ Template is VALID".green().bold()));
    } else {
        output.push_str(&format!(
            "{}\n\n",
            "✗ Template validation FAILED".red().bold()
        ));
    }

    // Issues
    if !result.issues.is_empty() {
        output.push_str(&format!("{}:\n", "Issues".red().bold()));
        for issue in &result.issues {
            let marker = if issue.severity == "error" {
                "✗".red()
            } else {
                "⚠".yellow()
            };
            output.push_str(&format!(
                "  {} {}: {}\n",
                marker,
                issue.location.dimmed(),
                issue.message
            ));
        }
        output.push('\n');
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str(&format!(
            "{} ({}):\n",
            "Warnings".yellow().bold(),
            result.warnings.len()
        ));
        for warning in &result.warnings {
            output.push_str(&format!(
                "  {} {}: {}\n",
                "⚠".yellow(),
                warning.category.dimmed(),
                warning.message
            ));
        }
        output.push('\n');
    }

    output
}

/// Gets detailed information about a template.
///
/// # Arguments
///
/// * `org` - Organization name
/// * `template` - Template name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn template_info(org: &str, template: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Getting template information",
        org = org,
        template = template,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Get template information
    let info = get_template_info(org, template, provider).await?;

    // Format and display output
    let output = format_template_info(&info, format)?;
    println!("{}", output);

    Ok(())
}

/// Validates a template configuration.
///
/// Routes between three modes depending on the supplied arguments:
/// 1. `--path` exists on disk → load `template.toml` locally; attempt remote type checks
///    if a GitHub remote is detected or `--org` is given.
/// 2. `--path` absent/nonexistent **and** `--org` + `--template` supplied → clone to a
///    temporary directory then validate locally with optional remote type checks.
/// 3. Neither usable path nor org+template → `Error::InvalidArguments`.
///
/// # Arguments
///
/// * `org` - Organisation name (optional when `--path` is given).
/// * `template` - Template name (optional when `--path` is given).
/// * `path` - Local directory containing the template repository (optional).
/// * `format` - Output format ("json" or "pretty").
#[instrument]
async fn template_validate(
    org: Option<&str>,
    template: Option<&str>,
    path: Option<&str>,
    format: &str,
) -> Result<(), Error> {
    debug!(
        message = "Validating template",
        org = org,
        template = template,
        path = path,
        format = format
    );

    let path_dir = path.map(std::path::Path::new);

    let (template_name, validation_result) = if let Some(dir) = path_dir.filter(|p| p.exists()) {
        // --- Local-path branch (task 2.3) ---
        let config = load_template_config_from_path(dir)?;
        let name = config.template.name.clone();

        // Determine org from git remote detection (task 3.2) or explicit --org.
        let detected_org: Option<String> = detect_github_remote(dir)
            .map(|(o, _)| o)
            .or_else(|| org.map(str::to_string));

        // If an org is known, attempt a provider for remote type-validity checks.
        // Credential failures are non-fatal — skip remote checks with a warning.
        let provider_opt = if detected_org.is_some() {
            match create_metadata_provider().await {
                Ok(p) => Some(p),
                Err(_) => {
                    tracing::warn!("Skipping remote type checks: GitHub credentials not available");
                    None
                }
            }
        } else {
            None
        };

        let eff_org = detected_org.as_deref();
        let result = validate_loaded_template(&name, config, provider_opt, eff_org).await?;
        (name, result)
    } else if let (Some(org_val), Some(tmpl_val)) = (org, template) {
        // --- Clone branch (task 4.2) ---
        let tmp = tempfile::TempDir::new()
            .map_err(|e| Error::GitHub(format!("Failed to create temp directory: {}", e)))?;

        clone_template_to_dir(org_val, tmpl_val, tmp.path())?;

        let config = load_template_config_from_path(tmp.path())?;
        let name = config.template.name.clone();

        // Org is known from --org; attempt remote type-validity checks.
        let provider_opt = match create_metadata_provider().await {
            Ok(p) => Some(p),
            Err(_) => {
                tracing::warn!("Skipping remote type checks: GitHub credentials not available");
                None
            }
        };

        let result = validate_loaded_template(&name, config, provider_opt, Some(org_val)).await?;
        // `tmp` drops here, cleaning up the cloned directory.
        (name, result)
    } else {
        return Err(Error::InvalidArguments(
            "Specify either --path <DIR> or both --org <ORG> and --template <TEMPLATE>".to_string(),
        ));
    };

    let output = format_validation_result(&validation_result, format)?;
    println!("{}", output);

    if !validation_result.valid {
        return Err(Error::Config(format!(
            "Template '{}' validation failed with {} issue(s)",
            template_name,
            validation_result.issues.len()
        )));
    }

    Ok(())
}

#[cfg(test)]
#[path = "template_cmd_tests.rs"]
mod tests;
