//! Configuration management for the repo_roller system.
//!
//! This crate provides hierarchical configuration management for organization-specific
//! repository settings, supporting a four-level hierarchy with override controls.
//!
//! # Module Organization
//!
//! - `types` - Basic configuration types and enums
//! - `hierarchy` - Hierarchical value types with override control
//! - `settings` - Configuration setting structures
//! - `templates` - Template-related structures
//! - `merged` - Final merged configuration and merging logic
//! - `errors` - Error types for configuration validation and processing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

// Configuration modules
pub mod errors;
pub mod github_provider;
pub mod hierarchy;
pub mod merged;
pub mod metadata;
pub mod organization;
pub mod parsers;
pub mod settings;
pub mod templates;
pub mod types;

// Re-export commonly used items
pub use errors::*;
pub use hierarchy::*;
pub use merged::*;
pub use metadata::*;
pub use parsers::*;
pub use settings::*;
pub use templates::*;
pub use types::*;

// Unit tests will be added in a separate file: lib_tests.rs
#[path = "lib_tests.rs"]
#[cfg(test)]
mod tests;

/// Trait for loading configuration from various sources.
///
/// This trait abstracts the configuration loading mechanism, allowing for different
/// implementations such as loading from files, environment variables, or remote sources.
/// All implementations must be thread-safe (`Send + Sync`) to support concurrent access.
///
/// # Examples
///
/// ```rust
/// use config_manager::{ConfigLoader, FileConfigLoader, Config};
///
/// let loader = FileConfigLoader;
/// // In real usage, you would pass a valid file path
/// // let config = loader.load_config("config.toml")?;
/// ```
pub trait ConfigLoader: Send + Sync {
    /// Loads configuration from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration source (e.g., file path, URL)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the loaded `Config` on success, or a `ConfigError`
    /// if the configuration could not be loaded or parsed.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The configuration source cannot be accessed (e.g., file not found)
    /// - The configuration content is malformed or cannot be parsed
    /// - Any I/O operation fails during loading
    fn load_config(&self, path: &str) -> Result<Config, crate::ConfigError>;
}

/// Configuration for GitHub Actions permissions.
///
/// This struct defines the permissions settings for GitHub Actions in a repository.
/// Currently serves as a placeholder for more granular permission controls that may
/// be added in future versions.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPermissions {
    /// Whether GitHub Actions are enabled for the repository.
    ///
    /// When `Some(true)`, Actions are enabled. When `Some(false)`, Actions are disabled.
    /// When `None`, the repository's default setting is used.
    pub enabled: Option<bool>,
    // Add more granular permissions later
}

/// Configuration for a branch protection rule.
///
/// Branch protection rules define policies that must be satisfied before code can be
/// merged into protected branches. This struct currently serves as a placeholder
/// with basic pattern matching, with more specific rules to be added later.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BranchProtectionRule {
    /// The branch name pattern that this rule applies to.
    ///
    /// Supports glob patterns (e.g., "main", "release/*", "feature/**").
    /// The pattern determines which branches are protected by this rule.
    pub pattern: String,
    // Add specific rules like required checks, reviews, etc.
}

/// Represents the overall application configuration.
///
/// This is the root configuration structure that contains all templates and
/// global settings for the RepoRoller application.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    /// List of available repository templates.
    ///
    /// Each template defines a set of default configurations that can be applied
    /// when creating or updating repositories.
    pub templates: Vec<TemplateConfig>,
}

/// Represents toggles for repository features.
///
/// This struct controls which GitHub repository features are enabled or disabled.
/// Each field corresponds to a specific repository feature that can be toggled
/// on or off independently.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FeatureToggles {
    /// Whether the Issues feature is enabled.
    ///
    /// When `Some(true)`, Issues are enabled. When `Some(false)`, Issues are disabled.
    /// When `None`, the repository's default setting is used.
    pub issues: Option<bool>,

    /// Whether the Projects feature is enabled.
    ///
    /// When `Some(true)`, Projects are enabled. When `Some(false)`, Projects are disabled.
    /// When `None`, the repository's default setting is used.
    pub projects: Option<bool>,

    /// Whether the Discussions feature is enabled.
    ///
    /// When `Some(true)`, Discussions are enabled. When `Some(false)`, Discussions are disabled.
    /// When `None`, the repository's default setting is used.
    pub discussions: Option<bool>,

    /// Whether the Wiki feature is enabled.
    ///
    /// When `Some(true)`, Wiki is enabled. When `Some(false)`, Wiki is disabled.
    /// When `None`, the repository's default setting is used.
    pub wiki: Option<bool>,
}

/// Represents a standard issue label configuration.
///
/// Issue labels are used to categorize and organize issues and pull requests.
/// This struct defines the properties of a label that will be created in the repository.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelConfig {
    /// The name of the label.
    ///
    /// This is the text that will be displayed on the label. Label names should be
    /// descriptive and follow consistent naming conventions.
    pub name: String,

    /// The color of the label as a hex code without the '#' prefix.
    ///
    /// For example: "ff0000" for red, "00ff00" for green, "0000ff" for blue.
    /// The color helps visually distinguish different types of labels.
    pub color: String,

    /// An optional description for the label.
    ///
    /// When provided, this description appears as a tooltip when hovering over
    /// the label in the GitHub interface, helping users understand the label's purpose.
    pub description: Option<String>,
}

/// Represents settings related to pull requests.
///
/// This struct configures the merge options and behavior for pull requests
/// in a repository, controlling how contributors can merge their changes.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PullRequestSettings {
    /// Whether merge commits are allowed.
    ///
    /// When `Some(true)`, contributors can use the "Create a merge commit" option.
    /// When `Some(false)`, this option is disabled. When `None`, the repository's
    /// default setting is used.
    pub allow_merge_commit: Option<bool>,

    /// Whether squash merging is allowed.
    ///
    /// When `Some(true)`, contributors can use the "Squash and merge" option.
    /// When `Some(false)`, this option is disabled. When `None`, the repository's
    /// default setting is used.
    pub allow_squash_merge: Option<bool>,

    /// Whether rebase merging is allowed.
    ///
    /// When `Some(true)`, contributors can use the "Rebase and merge" option.
    /// When `Some(false)`, this option is disabled. When `None`, the repository's
    /// default setting is used.
    pub allow_rebase_merge: Option<bool>,

    /// Whether to automatically delete head branches after pull requests are merged.
    ///
    /// When `Some(true)`, the head branch is automatically deleted after merge.
    /// When `Some(false)`, branches are preserved. When `None`, the repository's
    /// default setting is used.
    pub delete_branch_on_merge: Option<bool>,
    // pub default_merge_message_format: Option<String>, // TODO: Add later if needed
}

/// Represents the configuration for a single repository template.
///
/// A template defines a set of default configurations and settings that can be
/// applied when creating or updating repositories. Templates help ensure consistency
/// across multiple repositories by providing standardized configurations.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TemplateConfig {
    /// Unique identifier for the template.
    ///
    /// This name is used to reference the template when creating repositories
    /// and should be descriptive and unique within the configuration.
    pub name: String,

    /// URL or path to the template repository/directory.
    ///
    /// This can be a GitHub repository URL, a local file path, or any other
    /// location where the template source can be found.
    pub source_repo: String,

    /// Optional description for the template.
    ///
    /// Provides human-readable information about what this template is intended
    /// for and what configurations it includes.
    pub description: Option<String>,

    /// List of topics to apply to the repository.
    ///
    /// Topics help categorize and make repositories discoverable. They appear
    /// as tags on the repository's main page in GitHub.
    pub topics: Option<Vec<String>>,

    /// Default feature toggles for repositories created from this template.
    ///
    /// Defines which GitHub features (Issues, Projects, Discussions, Wiki) should
    /// be enabled or disabled by default for repositories using this template.
    pub features: Option<FeatureToggles>,

    /// Default pull request settings.
    ///
    /// Configures the merge options and behavior for pull requests in repositories
    /// created from this template.
    pub pr_settings: Option<PullRequestSettings>,

    /// Standard labels to create in the repository.
    ///
    /// A list of issue labels that will be automatically created when applying
    /// this template to a repository.
    pub labels: Option<Vec<LabelConfig>>,

    /// Branch protection rules to apply.
    ///
    /// Defines policies that must be satisfied before code can be merged into
    /// protected branches. Currently serves as a placeholder for future implementation.
    pub branch_protection_rules: Option<Vec<BranchProtectionRule>>,

    /// Action permissions settings.
    ///
    /// Configures GitHub Actions permissions for repositories using this template.
    /// Currently serves as a placeholder for more granular controls.
    pub action_permissions: Option<ActionPermissions>,

    /// List of variable names expected by the template engine.
    ///
    /// When processing templates, these variables must be provided to successfully
    /// generate the repository content.
    pub required_variables: Option<Vec<String>>,

    /// Template variable configurations with validation rules.
    ///
    /// Provides detailed configuration for template variables including validation
    /// rules, default values, and examples. The key is the variable name.
    pub variable_configs: Option<HashMap<String, VariableConfig>>,
}

/// Template variable configuration with validation and metadata.
///
/// This struct defines the properties and validation rules for template variables
/// used during repository creation. It supports various validation constraints
/// and provides metadata to help users understand how to use the variable.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VariableConfig {
    /// Human-readable description of the variable.
    ///
    /// Explains what this variable is used for and how it affects the generated
    /// repository content.
    pub description: String,

    /// Optional example value for the variable.
    ///
    /// Provides users with a concrete example of what a valid value looks like,
    /// helping them understand the expected format.
    pub example: Option<String>,

    /// Whether this variable is required.
    ///
    /// When `Some(true)`, the variable must be provided. When `Some(false)` or `None`,
    /// the variable is optional and may use a default value if provided.
    pub required: Option<bool>,

    /// Optional regex pattern for validation.
    ///
    /// When provided, variable values must match this pattern to be considered valid.
    /// Useful for enforcing naming conventions or format requirements.
    pub pattern: Option<String>,

    /// Minimum length constraint for string values.
    ///
    /// When provided, string values must be at least this many characters long.
    pub min_length: Option<usize>,

    /// Maximum length constraint for string values.
    ///
    /// When provided, string values must be no more than this many characters long.
    pub max_length: Option<usize>,

    /// List of valid options for the variable.
    ///
    /// When provided, the variable value must be one of the options in this list.
    /// Useful for creating dropdown selections or enforcing specific choices.
    pub options: Option<Vec<String>>,

    /// Default value for the variable.
    ///
    /// When provided, this value will be used if no value is explicitly provided
    /// for the variable during template processing.
    pub default: Option<String>,
}

/// Errors that can occur while loading or accessing configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Error reading the configuration file.
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] io::Error),

    /// Error parsing the text.
    #[error("Failed to parse the text configuration")]
    Text(String),

    /// Error parsing the TOML configuration content.
    #[error("Failed to parse TOML configuration: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Default implementation for loading config from file.
///
/// This implementation loads configuration from TOML files on the filesystem.
/// It provides a simple, file-based approach to configuration loading suitable
/// for most use cases.
pub struct FileConfigLoader;

impl ConfigLoader for FileConfigLoader {
    fn load_config(&self, path: &str) -> Result<Config, crate::ConfigError> {
        crate::load_config(path)
    }
}

/// Loads the application configuration from the specified TOML file.
///
/// This function reads a TOML configuration file from the filesystem and
/// parses it into a `Config` structure. It handles both I/O and parsing
/// errors gracefully.
///
/// # Arguments
///
/// * `path` - The path to the configuration file. Can be any type that
///   implements `AsRef<Path>` (e.g., `&str`, `String`, `PathBuf`)
///
/// # Returns
///
/// Returns a `Result` containing the parsed `Config` on success, or a
/// `ConfigError` if the file cannot be read or parsed.
///
/// # Errors
///
/// This function will return an error if:
/// - The file cannot be read (e.g., file not found, permission denied)
/// - The file content is not valid TOML syntax
/// - The TOML content doesn't match the expected configuration structure
///
/// # Examples
///
/// ```rust
/// use config_manager::{load_config, Config};
/// use std::fs;
/// use tempfile::NamedTempFile;
/// use std::io::Write;
///
/// // Create a temporary config file for testing
/// let mut temp_file = NamedTempFile::new().unwrap();
/// writeln!(temp_file, r#"
/// [[templates]]
/// name = "example"
/// source_repo = "user/repo"
/// "#).unwrap();
///
/// let config = load_config(temp_file.path()).unwrap();
/// assert_eq!(config.templates.len(), 1);
/// ```
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
