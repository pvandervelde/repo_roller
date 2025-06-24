//! Template Engine for RepoRoller
//!
//! This crate handles template processing, variable substitution, and file transformation
//! according to the RepoRoller specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

mod errors;
use errors::Error;

/// Configuration for template variable validation
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

/// Template processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplatingConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// Request for template processing
#[derive(Debug, Clone)]
pub struct TemplateProcessingRequest {
    pub variables: HashMap<String, String>,
    pub built_in_variables: HashMap<String, String>,
    pub variable_configs: HashMap<String, VariableConfig>,
    pub templating_config: Option<TemplatingConfig>,
}

/// Result of template processing
#[derive(Debug, Clone)]
pub struct ProcessedTemplate {
    pub files: Vec<(String, Vec<u8>)>, // (path, content)
}

/// Trait for fetching template files from a source
pub trait TemplateFetcher: Send + Sync {
    fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String>;
}

/// Default implementation for fetching template files from a source repo
pub struct DefaultTemplateFetcher;

impl TemplateFetcher for DefaultTemplateFetcher {
    fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        // For MVP, return a basic README.md file with template variables
        let readme_content = format!(
            r#"# {{{{repo_name}}}}

Repository created from template: {{{{template_name}}}}
Owner: {{{{org_name}}}}
Created by: {{{{user_login}}}}
Created at: {{{{timestamp}}}}

## Description

This repository was generated using RepoRoller from template: {source}

## Getting Started

TODO: Add setup instructions here
"#,
            source = source
        );

        Ok(vec![("README.md".to_string(), readme_content.into_bytes())])
    }
}

/// Template processor that handles variable substitution and file processing
pub struct TemplateProcessor;

impl TemplateProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process template files with variable substitution
    pub fn process_template(
        &self,
        files: &[(String, Vec<u8>)],
        request: &TemplateProcessingRequest,
        output_dir: &Path,
    ) -> Result<ProcessedTemplate, Error> {
        // Validate variables first
        self.validate_variables(request)?;

        // Combine all variables (built-ins override user variables)
        let mut all_variables = request.variables.clone();
        for (key, value) in &request.built_in_variables {
            all_variables.insert(key.clone(), value.clone());
        }

        let mut processed_files = Vec::new();

        for (file_path, content) in files {
            // Skip files that match exclude patterns
            if let Some(ref config) = request.templating_config {
                if self.should_exclude_file(file_path, &config.exclude_patterns) {
                    continue;
                }

                // Only process files that match include patterns
                if !config.include_patterns.is_empty()
                    && !self.should_include_file(file_path, &config.include_patterns)
                {
                    continue;
                }
            }

            let processed_content = if self.is_text_file(content) {
                // Apply variable substitution to text files
                let content_str = String::from_utf8_lossy(content);
                let processed_str = self.substitute_variables(&content_str, &all_variables);
                processed_str.into_bytes()
            } else {
                // Binary files are copied as-is
                content.clone()
            };

            // Handle .template suffix removal
            let final_path = if file_path.ends_with(".template") {
                file_path.trim_end_matches(".template").to_string()
            } else {
                file_path.clone()
            };

            processed_files.push((final_path, processed_content));
        }

        Ok(ProcessedTemplate {
            files: processed_files,
        })
    }

    /// Generate built-in variables for template processing
    pub fn generate_built_in_variables(
        &self,
        repo_name: &str,
        org_name: &str,
        template_name: &str,
        template_repo: &str,
        user_login: &str,
        user_name: &str,
        default_branch: &str,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        let now = chrono::Utc::now();
        variables.insert("timestamp".to_string(), now.to_rfc3339());
        variables.insert("timestamp_unix".to_string(), now.timestamp().to_string());
        variables.insert("user_login".to_string(), user_login.to_string());
        variables.insert("user_name".to_string(), user_name.to_string());
        variables.insert("org_name".to_string(), org_name.to_string());
        variables.insert("repo_name".to_string(), repo_name.to_string());
        variables.insert("template_name".to_string(), template_name.to_string());
        variables.insert("template_repo".to_string(), template_repo.to_string());
        variables.insert("default_branch".to_string(), default_branch.to_string());

        variables
    }

    /// Validate variables according to their configurations
    fn validate_variables(&self, request: &TemplateProcessingRequest) -> Result<(), Error> {
        for (var_name, config) in &request.variable_configs {
            // Check if required variable is provided
            if config.required.unwrap_or(false) {
                if !request.variables.contains_key(var_name) && config.default.is_none() {
                    return Err(Error::RequiredVariableMissing(var_name.clone()));
                }
            }

            // Get the actual value (from variables or default)
            let value = match request.variables.get(var_name) {
                Some(v) => v.clone(),
                None => match &config.default {
                    Some(d) => d.clone(),
                    None => continue, // Optional variable not provided
                },
            };

            // Validate pattern if specified
            if let Some(ref pattern) = config.pattern {
                let regex = regex::Regex::new(pattern).map_err(|_| Error::VariableValidation {
                    variable: var_name.clone(),
                    reason: format!("Invalid regex pattern: {}", pattern),
                })?;

                if !regex.is_match(&value) {
                    return Err(Error::PatternValidationFailed {
                        variable: var_name.clone(),
                        pattern: pattern.clone(),
                    });
                }
            }

            // Validate length constraints
            if let Some(min_len) = config.min_length {
                if value.len() < min_len {
                    return Err(Error::VariableValidation {
                        variable: var_name.clone(),
                        reason: format!("Value too short, minimum length: {}", min_len),
                    });
                }
            }

            if let Some(max_len) = config.max_length {
                if value.len() > max_len {
                    return Err(Error::VariableValidation {
                        variable: var_name.clone(),
                        reason: format!("Value too long, maximum length: {}", max_len),
                    });
                }
            }

            // Validate against allowed options
            if let Some(ref options) = config.options {
                if !options.contains(&value) {
                    return Err(Error::VariableValidation {
                        variable: var_name.clone(),
                        reason: format!("Invalid option, allowed values: {:?}", options),
                    });
                }
            }
        }

        Ok(())
    }

    /// Substitute variables in text content
    fn substitute_variables(&self, content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        // Handle escaped braces first
        result = result.replace("\\{{", "__ESCAPED_OPEN_BRACE__");

        // Substitute variables
        for (key, value) in variables {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }

        // Restore escaped braces
        result = result.replace("__ESCAPED_OPEN_BRACE__", "{{");

        result
    }

    /// Check if a file should be excluded based on patterns
    fn should_exclude_file(&self, file_path: &str, exclude_patterns: &[String]) -> bool {
        exclude_patterns.iter().any(|pattern| {
            // Simple glob matching for MVP - could be enhanced with a proper glob library
            self.simple_glob_match(pattern, file_path)
        })
    }

    /// Check if a file should be included based on patterns
    fn should_include_file(&self, file_path: &str, include_patterns: &[String]) -> bool {
        include_patterns
            .iter()
            .any(|pattern| self.simple_glob_match(pattern, file_path))
    }

    /// Simple glob pattern matching (basic implementation for MVP)
    fn simple_glob_match(&self, pattern: &str, path: &str) -> bool {
        // Very basic implementation - should be replaced with proper glob library
        if pattern == "**" || pattern == "**/*" {
            return true;
        }

        if pattern.contains("**") {
            // Handle recursive patterns
            let prefix = pattern.split("**").next().unwrap_or("");
            let suffix = pattern.split("**").nth(1).unwrap_or("");
            return path.starts_with(prefix) && path.ends_with(suffix);
        }

        if pattern.contains('*') {
            // Handle basic wildcard
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }

        // Exact match
        pattern == path
    }

    /// Check if content appears to be a text file
    fn is_text_file(&self, content: &[u8]) -> bool {
        // Simple heuristic: if we can decode as UTF-8 and it doesn't contain null bytes
        match std::str::from_utf8(content) {
            Ok(text) => !text.contains('\0'),
            Err(_) => false,
        }
    }
}

/// Legacy function for backwards compatibility - will be removed
pub fn fetch_template_files(source_repo: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    let fetcher = DefaultTemplateFetcher;
    fetcher.fetch_template_files(source_repo)
}
