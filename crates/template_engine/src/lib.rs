//! Template Engine for RepoRoller
//!
//! This crate handles template processing, variable substitution, and file transformation
//! according to the RepoRoller specification.

use async_trait::async_trait;
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
#[async_trait]
pub trait TemplateFetcher: Send + Sync {
    async fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String>;
}

/// GitHub-based implementation for fetching template files from a repository
pub struct GitHubTemplateFetcher {
    #[allow(dead_code)] // Will be used in future implementation
    github_client: github_client::GitHubClient,
}

impl GitHubTemplateFetcher {
    /// Create a new GitHub template fetcher with the provided client
    pub fn new(github_client: github_client::GitHubClient) -> Self {
        Self { github_client }
    }

    /// Parse GitHub repository URL to extract owner and repo name
    #[allow(dead_code)] // Will be used in future implementation
    fn parse_github_url(&self, url: &str) -> Result<(String, String), String> {
        // Handle both HTTPS and SSH GitHub URLs
        // Examples:
        // - https://github.com/owner/repo
        // - https://github.com/owner/repo.git
        // - git@github.com:owner/repo.git

        let url = url.trim_end_matches(".git");

        if let Some(captures) = regex::Regex::new(r"github\.com[:/]([^/]+)/([^/]+)/?$")
            .map_err(|e| format!("Failed to create regex: {}", e))?
            .captures(url)
        {
            let owner = captures
                .get(1)
                .ok_or("No owner found in URL")?
                .as_str()
                .to_string();
            let repo = captures
                .get(2)
                .ok_or("No repository name found in URL")?
                .as_str()
                .to_string();

            Ok((owner, repo))
        } else {
            Err(format!("Invalid GitHub URL format: {}", url))
        }
    }

    /// Clone the repository and read all files
    async fn fetch_repository_files(&self, url: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        use std::process::Command;
        use tempfile::TempDir;

        // Create a temporary directory for cloning
        let temp_dir =
            TempDir::new().map_err(|e| format!("Failed to create temporary directory: {}", e))?;

        // Clone the repository
        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                url,
                temp_dir.path().to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute git clone: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Read all files from the cloned repository
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(temp_dir.path())
                .map_err(|e| format!("Failed to get relative path: {}", e))?;

            // Skip .git directory files
            let path_str = relative_path.to_string_lossy();
            if path_str.starts_with(".git/") {
                continue;
            }

            // Read file content
            let content = std::fs::read(path)
                .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;

            files.push((path_str.to_string(), content));
        }

        if files.is_empty() {
            return Err(format!("No files found in repository {}", url));
        }

        Ok(files)
    }
}

#[async_trait]
impl TemplateFetcher for GitHubTemplateFetcher {
    async fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        // Fetch all files from the repository using git clone
        let files = self.fetch_repository_files(source).await?;

        // Filter out unwanted files
        let filtered_files: Vec<(String, Vec<u8>)> = files
            .into_iter()
            .filter(|(path, _)| {
                !path.starts_with(".git/") && !path.starts_with(".github/") && path != ".gitignore"
                // We'll handle .gitignore specially
            })
            .collect();

        if filtered_files.is_empty() {
            return Err(format!("No template files found in repository {}", source));
        }

        Ok(filtered_files)
    }
}

/// Template processor that handles variable substitution and file processing
pub struct TemplateProcessor;

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process template files with variable substitution
    pub fn process_template(
        &self,
        files: &[(String, Vec<u8>)],
        request: &TemplateProcessingRequest,
        _output_dir: &Path,
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
        params: &BuiltInVariablesParams,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        let now = chrono::Utc::now();
        variables.insert("timestamp".to_string(), now.to_rfc3339());
        variables.insert("timestamp_unix".to_string(), now.timestamp().to_string());
        variables.insert("user_login".to_string(), params.user_login.to_string());
        variables.insert("user_name".to_string(), params.user_name.to_string());
        variables.insert("org_name".to_string(), params.org_name.to_string());
        variables.insert("repo_name".to_string(), params.repo_name.to_string());
        variables.insert(
            "template_name".to_string(),
            params.template_name.to_string(),
        );
        variables.insert(
            "template_repo".to_string(),
            params.template_repo.to_string(),
        );
        variables.insert(
            "default_branch".to_string(),
            params.default_branch.to_string(),
        );

        variables
    }

    /// Validate variables according to their configurations
    fn validate_variables(&self, request: &TemplateProcessingRequest) -> Result<(), Error> {
        for (var_name, config) in &request.variable_configs {
            // Check if required variable is provided
            if config.required.unwrap_or(false)
                && !request.variables.contains_key(var_name)
                && config.default.is_none()
            {
                return Err(Error::RequiredVariableMissing(var_name.clone()));
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

/// Parameters for generating built-in template variables.
pub struct BuiltInVariablesParams<'a> {
    pub repo_name: &'a str,
    pub org_name: &'a str,
    pub template_name: &'a str,
    pub template_repo: &'a str,
    pub user_login: &'a str,
    pub user_name: &'a str,
    pub default_branch: &'a str,
}

/// Legacy function for backwards compatibility - will be removed
pub async fn fetch_template_files(_source_repo: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    Err("DefaultTemplateFetcher is deprecated. Use GitHubTemplateFetcher instead.".to_string())
}
