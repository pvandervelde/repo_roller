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
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::config::{get_config_path, AppConfig, DEFAULT_METADATA_REPOSITORY_NAME};
use crate::errors::Error;

// Keyring constants (shared with create_cmd and auth_cmd)
// Currently unused but will be needed when template loading is implemented
#[allow(dead_code)]
const KEY_RING_SERVICE_NAME: &str = "repo_roller";
#[allow(dead_code)]
const KEY_RING_APP_ID: &str = "github_app_id";
#[allow(dead_code)]
const KEY_RING_APP_PRIVATE_KEY_PATH: &str = "github_app_private_key_path";

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
    Validate {
        /// Organization name.
        #[arg(long)]
        org: String,

        /// Template name to validate.
        #[arg(long)]
        template: String,

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
            format,
        } => template_validate(org, template, format).await,
    }
}

/// Creates an authenticated metadata provider wrapped in Arc.
///
/// Loads GitHub App credentials from the system keyring and creates
/// an authenticated GitHubMetadataProvider instance.
///
/// TODO: This will be needed when template loading from template repositories is implemented
#[allow(dead_code)]
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
#[allow(dead_code)] // Used in tests
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
#[allow(dead_code)] // Used in tests
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
#[allow(dead_code)] // Used in tests and future CLI commands
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
#[allow(dead_code)] // Used in tests and future CLI commands
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
#[allow(dead_code)] // Used in tests and future CLI commands
pub async fn validate_template(
    org: &str,
    template_name: &str,
    provider: Arc<dyn MetadataRepositoryProvider>,
) -> Result<TemplateValidationResult, Error> {
    debug!("Validating template {}/{}", org, template_name);

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

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
                warnings,
            });
        }
    };

    // Validate template metadata
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

    // Validate variables
    if let Some(ref variables) = config.variables {
        if variables.is_empty() {
            warnings.push(ValidationWarning {
                category: "best_practice".to_string(),
                message: "No variables defined - template is not customizable".to_string(),
            });
        }

        for (var_name, var_def) in variables {
            // Validate variable name (alphanumeric + underscore only)
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

            // Check for contradiction: required + default
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

            // Warn about required variables without examples
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

    // Validate repository type if specified
    if let Some(ref repo_type_spec) = config.repository_type {
        // We need to discover the metadata repository to check available types
        // For now, we'll try to get available types if possible
        // If it fails, we'll note it as a warning rather than an error
        match provider.discover_metadata_repository(org).await {
            Ok(metadata_repo) => {
                match provider
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
    }

    let valid = issues.is_empty();

    Ok(TemplateValidationResult {
        template_name: template_name.to_string(),
        valid,
        issues,
        warnings,
    })
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
/// # Arguments
///
/// * `org` - Organization name
/// * `template` - Template name
/// * `format` - Output format ("json" or "pretty")
#[instrument]
async fn template_validate(org: &str, template: &str, format: &str) -> Result<(), Error> {
    debug!(
        message = "Validating template",
        org = org,
        template = template,
        format = format
    );

    // Create authenticated metadata provider
    let provider = create_metadata_provider().await?;

    // Validate template
    let result = validate_template(org, template, provider).await?;

    // Format and display output
    let output = format_validation_result(&result, format)?;
    println!("{}", output);

    // Return error if validation failed
    if !result.valid {
        return Err(Error::Config(format!(
            "Template '{}' validation failed with {} issue(s)",
            template,
            result.issues.len()
        )));
    }

    Ok(())
}

#[cfg(test)]
#[path = "template_cmd_tests.rs"]
mod tests;
