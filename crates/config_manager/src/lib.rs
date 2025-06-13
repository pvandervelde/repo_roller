//! Manages loading and accessing application configuration for RepoRoller.
//!
//! This crate is responsible for reading configuration files (e.g., TOML)
//! and providing access to settings like template definitions, standard labels, etc.

use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

// Unit tests will be added in a separate file: lib_tests.rs
#[path = "lib_tests.rs"]
#[cfg(test)]
mod tests;

/// Represents a standard issue label configuration.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelConfig {
    pub name: String,
    pub color: String, // Color hex code without '#'
    pub description: Option<String>,
}

/// Represents toggles for repository features.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FeatureToggles {
    pub issues: Option<bool>,
    pub projects: Option<bool>,
    pub discussions: Option<bool>,
    pub wiki: Option<bool>,
}

/// Represents settings related to pull requests.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PullRequestSettings {
    pub allow_merge_commit: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
    pub delete_branch_on_merge: Option<bool>,
    // pub default_merge_message_format: Option<String>, // TODO: Add later if needed
}

// TODO: Define more detailed structs for Branch Protection, Actions, etc. later
/// Placeholder for branch protection rule configuration.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BranchProtectionRule {
    pub pattern: String,
    // Add specific rules like required checks, reviews, etc.
}

/// Placeholder for GitHub Actions permissions configuration.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPermissions {
    pub enabled: Option<bool>,
    // Add more granular permissions later
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

/// Represents the configuration for a single repository template, including
/// default settings to apply.
#[derive(Deserialize, Debug, Clone)]
pub struct TemplateConfig {
    /// Unique identifier for the template.
    pub name: String,
    /// URL or path to the template repository/directory.
    pub source_repo: String,
    /// Optional description for the template.
    pub description: Option<String>,
    /// List of topics to apply to the repository.
    pub topics: Option<Vec<String>>,
    /// Default feature toggles for repositories created from this template.
    pub features: Option<FeatureToggles>,
    /// Default pull request settings.
    pub pr_settings: Option<PullRequestSettings>,
    /// Standard labels to create in the repository.
    pub labels: Option<Vec<LabelConfig>>,
    /// Branch protection rules to apply (placeholder).
    pub branch_protection_rules: Option<Vec<BranchProtectionRule>>,
    /// Action permissions settings (placeholder).
    pub action_permissions: Option<ActionPermissions>,
    /// List of variable names expected by the template engine.
    pub required_variables: Option<Vec<String>>,
    // TODO: Add fields for custom properties, environments, discussion categories etc.
}

/// Represents the overall application configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// List of available repository templates.
    pub templates: Vec<TemplateConfig>,
    // TODO: Add global settings if needed (e.g., org-wide defaults, allowed templates)
}

/// Trait for loading configuration.
pub trait ConfigLoader: Send + Sync {
    fn load_config(&self, path: &str) -> Result<Config, crate::ConfigError>;
}

/// Default implementation for loading config from file.
pub struct FileConfigLoader;

impl ConfigLoader for FileConfigLoader {
    fn load_config(&self, path: &str) -> Result<Config, crate::ConfigError> {
        crate::load_config(path)
    }
}

/// Loads the application configuration from the specified TOML file.
///
/// # Arguments
///
/// * `path` - The path to the configuration file.
///
/// # Errors
///
/// Returns `ConfigError` if the file cannot be read or parsed.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
