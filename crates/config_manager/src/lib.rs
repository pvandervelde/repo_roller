//! Configuration management for RepoRoller
//!
//! TODO (Interface Design): This crate will be completely redesigned based on interface specifications.
//! The types below are temporary stubs to maintain compatibility with existing code.
//!
//! See specs/interfaces/configuration-interfaces.md for the new interface design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ================================================================================================
// TEMPORARY COMPATIBILITY TYPES
// These types maintain compatibility with existing code while interface design is in progress
// TODO: Replace with new types from specs/interfaces/configuration-interfaces.md
// ================================================================================================

/// Temporary Config structure for existing code compatibility
///
/// TODO: Replace with new ConfigurationManager trait and types
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub templates: Vec<TemplateConfig>,
}

/// Temporary TemplateConfig structure for existing code compatibility
///
/// TODO: Replace with new template configuration types
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub name: String,
    pub source_repo: String,
    pub variable_configs: Option<HashMap<String, VariableConfig>>,

    // Additional fields for integration tests compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_protection_rules: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_permissions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_variables: Option<Vec<String>>,
}

/// Temporary VariableConfig structure for existing code compatibility
///
/// TODO: Replace with new variable configuration types  
/// See: specs/interfaces/configuration-interfaces.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariableConfig {
    pub description: String,
    pub example: Option<String>,
    pub required: Option<bool>,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
}
