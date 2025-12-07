//! # Template Engine for RepoRoller
//!
//! This crate provides comprehensive template processing capabilities for RepoRoller,
//! including variable substitution, file filtering, and content transformation.
//! It handles the conversion of template repositories into customized project repositories.
//!
//! ## Overview
//!
//! The template engine supports:
//! - **Variable Substitution**: Replace `{{variable}}` placeholders with actual values
//! - **File Filtering**: Include/exclude files based on patterns
//! - **Built-in Variables**: Automatic generation of common variables (timestamps, repo info)
//! - **Validation**: Ensure variables meet specified requirements (patterns, length, etc.)
//! - **Multiple Sources**: Fetch templates from GitHub repositories or other sources
//!
//! ## Core Components
//!
//! - [`TemplateProcessor`] - Main processor for variable substitution and file handling
//! - [`TemplateFetcher`] - Trait for retrieving template files from various sources
//! - [`GitHubTemplateFetcher`] - GitHub-specific implementation for template fetching
//! - [`VariableConfig`] - Configuration for variable validation and defaults
//! - [`TemplateProcessingRequest`] - Request structure containing all processing parameters
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use template_engine::{TemplateProcessor, GitHubTemplateFetcher, TemplateProcessingRequest};
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create template processor
//! let processor = TemplateProcessor::new();
//!
//! // Set up variables for substitution
//! let mut variables = HashMap::new();
//! variables.insert("project_name".to_string(), "my-awesome-app".to_string());
//! variables.insert("author".to_string(), "John Doe".to_string());
//!
//! // Create processing request
//! let request = TemplateProcessingRequest {
//!     variables,
//!     built_in_variables: HashMap::new(),
//!     variable_configs: HashMap::new(),
//!     templating_config: None,
//! };
//!
//! // Fetch template files (in this example, from a list of files)
//! let template_files = vec![
//!     ("README.md".to_string(), b"# {{project_name}}\n\nBy {{author}}".to_vec()),
//!     ("src/main.rs".to_string(), b"// {{project_name}} by {{author}}".to_vec()),
//! ];
//!
//! // Process the template
//! let result = processor.process_template(&template_files, &request, ".")?;
//!
//! // The result contains processed files with variables substituted
//! for (path, content) in result.files {
//!     println!("Processed file: {}", path);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Variable Substitution
//!
//! The engine uses Handlebars-style syntax for variable substitution:
//! - `{{variable}}` - Simple variable substitution
//! - `{{#if condition}}...{{/if}}` - Conditional blocks
//! - `{{#each items}}...{{/each}}` - Iteration over lists
//!
//! ## Built-in Variables
//!
//! The engine automatically provides several built-in variables:
//! - `timestamp` - Current timestamp in RFC3339 format
//! - `timestamp_unix` - Current Unix timestamp
//! - `user_login` - GitHub user login
//! - `user_name` - GitHub user display name
//! - `repo_name` - Repository name
//! - `org_name` - Organization name
//! - `template_name` - Template name used
//! - `default_branch` - Default branch name

use async_trait::async_trait;
use github_client::GitHubClient;
use glob::{MatchOptions, Pattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod errors;
pub use errors::Error;

pub mod handlebars_engine;
pub use handlebars_engine::*;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

/// Trait for fetching template files from various sources.
///
/// This trait abstracts the process of retrieving template files, allowing
/// different implementations for various sources (GitHub repositories, local
/// filesystems, remote archives, etc.). Implementations should handle the
/// specific details of connecting to and downloading from their respective sources.
///
/// ## Method Requirements
///
/// * `fetch_template_files` - Retrieve all files from a template source
///
/// ## Source Format
///
/// The `source` parameter format depends on the implementation:
/// - GitHub: Repository URLs like <https://github.com/owner/repo> or "owner/repo"
/// - Local: File system paths like "/path/to/template" or "C:\\templates\\basic"
/// - HTTP: URLs to downloadable archives like <https://example.com/templates/rust.zip>
///
/// ## Return Format
///
/// Returns a vector of tuples where each tuple contains:
/// - File path relative to the template root
/// - File content as bytes
///
/// ## Error Handling
///
/// Implementations should return descriptive error messages for common failure scenarios:
/// - Network connectivity issues
/// - Authentication failures
/// - Template not found
/// - Malformed source identifiers
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::TemplateFetcher;
/// use async_trait::async_trait;
///
/// struct LocalTemplateFetcher;
///
/// #[async_trait]
/// impl TemplateFetcher for LocalTemplateFetcher {
///     async fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
///         // Implementation for local filesystem
///         let mut files = Vec::new();
///         // ... read files from local path ...
///         Ok(files)
///     }
/// }
///
/// // Usage
/// let fetcher = LocalTemplateFetcher;
/// let files = fetcher.fetch_template_files("/templates/rust-app").await?;
/// println!("Fetched {} files", files.len());
/// ```
#[async_trait]
pub trait TemplateFetcher: Send + Sync {
    async fn fetch_template_files(&self, source: &str) -> Result<Vec<(String, Vec<u8>)>, String>;
}

/// Parameters for generating built-in template variables.
///
/// This structure contains all the contextual information needed to generate
/// built-in variables that are automatically available in templates. These
/// parameters provide information about the repository being created, the
/// user creating it, and the template being used.
///
/// # Lifetime
///
/// The struct uses borrowed string slices to avoid unnecessary allocations
/// when the data is already available elsewhere in the application.
///
/// # Examples
///
/// ```rust
/// use template_engine::{BuiltInVariablesParams, TemplateProcessor};
///
/// # fn main() -> Result<(), template_engine::Error> {
/// let params = BuiltInVariablesParams {
///     repo_name: "my-awesome-project",
///     org_name: "my-organization",
///     template_name: "rust-library",
///     template_repo: "templates/rust-library",
///     user_login: "developer123",
///     user_name: "Jane Developer",
///     default_branch: "main",
/// };
///
/// let processor = TemplateProcessor::new()?;
/// let built_ins = processor.generate_built_in_variables(&params);
///
/// assert_eq!(built_ins.get("repo_name"), Some(&"my-awesome-project".to_string()));
/// assert_eq!(built_ins.get("org_name"), Some(&"my-organization".to_string()));
/// # Ok(())
/// # }
/// ```
///
/// # Type Safety Note
///
/// This struct uses `&str` types to avoid circular dependencies between crates.
/// While branded types (like `RepositoryName`, `OrganizationName`) exist in
/// `repo_roller_core`, using them here would create a circular dependency since
/// `repo_roller_core` depends on `template_engine`.
///
/// Call sites should use the `.as_ref()` method on branded types to get `&str`:
///
/// ```rust,ignore
/// use repo_roller_core::{RepositoryName, OrganizationName, TemplateName};
///
/// let repo_name = RepositoryName::new("my-repo")?;
/// let org_name = OrganizationName::new("my-org")?;
/// let template_name = TemplateName::new("my-template")?;
///
/// let params = BuiltInVariablesParams {
///     repo_name: repo_name.as_ref(),
///     org_name: org_name.as_ref(),
///     template_name: template_name.as_ref(),
///     // ... other fields
/// };
/// ```
pub struct BuiltInVariablesParams<'a> {
    /// The name of the repository being created
    pub repo_name: &'a str,
    /// The name of the organization where the repository is being created
    pub org_name: &'a str,
    /// The name of the template being used
    pub template_name: &'a str,
    /// The source repository path of the template
    pub template_repo: &'a str,
    /// The GitHub login/username of the user creating the repository
    pub user_login: &'a str,
    /// The display name of the user creating the repository
    pub user_name: &'a str,
    /// The default branch name for the new repository
    pub default_branch: &'a str,
}

/// GitHub-based implementation for fetching template files from a repository.
///
/// This fetcher is responsible for downloading template files from GitHub repositories
/// and preparing them for template processing. It handles various GitHub URL formats
/// and filters out repository metadata files (like .git, .github directories).
///
/// # Fields
///
/// * `github_client` - A GitHub API client used for repository operations. This will
///   be used in future implementations for authenticated access and advanced repository
///   features. Currently marked as dead_code but preserved for future use.
///
/// # Examples
///
/// ```rust,ignore
/// use template_engine::GitHubTemplateFetcher;
/// use github_client::GitHubClient;
///
/// let client = GitHubClient::new(octocrab_instance);
/// let fetcher = GitHubTemplateFetcher::new(client);
///
/// // Fetch template files from a repository
/// # async {
/// let files = fetcher.fetch_template_files("https://github.com/owner/template-repo").await?;
/// # Ok::<(), String>(())
/// # };
/// ```
pub struct GitHubTemplateFetcher {
    #[allow(dead_code)] // Will be used in future implementation
    github_client: GitHubClient,
}

impl GitHubTemplateFetcher {
    /// Creates a new GitHub template fetcher with the provided client.
    ///
    /// # Arguments
    ///
    /// * `github_client` - An authenticated GitHub client for API operations
    ///
    /// # Returns
    ///
    /// A new `GitHubTemplateFetcher` instance ready to fetch template files.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::GitHubTemplateFetcher;
    /// use github_client::GitHubClient;
    ///
    /// let client = GitHubClient::new(octocrab_instance);
    /// let fetcher = GitHubTemplateFetcher::new(client);
    /// ```
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Clones a Git repository and reads all files from it.
    ///
    /// This method performs a shallow clone of the repository to a temporary
    /// directory, then reads all files and returns them as a vector of
    /// (path, content) tuples. The temporary directory is automatically
    /// cleaned up when the operation completes.
    async fn fetch_repository_files(&self, url: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        use std::process::Command;
        use tempfile::TempDir;

        // Create a temporary directory for cloning
        let temp_dir =
            TempDir::new().map_err(|e| format!("Failed to create temporary directory: {e}"))?;

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
            .map_err(|e| format!("Failed to execute git clone: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git clone failed: {stderr}"));
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
                .map_err(|e| format!("Failed to get relative path: {e}"))?;

            // Skip .git directory files
            let path_str = relative_path.to_string_lossy();
            if path_str.starts_with(".git/") {
                continue;
            }

            // Read file content
            let content =
                std::fs::read(path).map_err(|e| format!("Failed to read file {path:?}: {e}"))?;

            files.push((path_str.to_string(), content));
        }

        if files.is_empty() {
            return Err(format!("No files found in repository {url}"));
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
                // Use OS-agnostic path checking for directories we want to exclude
                let path_lower = path.to_lowercase();
                let is_git_dir =
                    path_lower.starts_with(".git/") || path_lower.starts_with(".git\\");
                let is_github_dir =
                    path_lower.starts_with(".github/") || path_lower.starts_with(".github\\");
                let is_gitignore = path == ".gitignore";

                !is_git_dir && !is_github_dir && !is_gitignore
                // We'll handle .gitignore specially
            })
            .collect();

        if filtered_files.is_empty() {
            return Err(format!("No template files found in repository {source}"));
        }

        Ok(filtered_files)
    }
}

/// Result of template processing containing the processed files.
///
/// This structure represents the output of template processing, containing
/// all files with variable substitutions applied and ready to be written
/// to the target repository.
///
/// ## Fields
///
/// * `files` - Vector of tuples where each tuple contains:
///   - `String` - Relative file path from the repository root
///   - `Vec<u8>` - File content as bytes (supports both text and binary files)
///
/// ## Usage
///
/// The processed files maintain their original directory structure and can be
/// written directly to a target directory. The paths are relative and use
/// forward slashes regardless of the operating system.
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::ProcessedTemplate;
/// use std::fs;
///
/// // Process a template and write the results
/// let processed = ProcessedTemplate {
///     files: vec![
///         ("README.md".to_string(), b"# My Awesome Project\n\nCreated by John Doe".to_vec()),
///         ("src/main.rs".to_string(), b"// My Awesome Project\nfn main() { }".to_vec()),
///         ("Cargo.toml".to_string(), b"[package]\nname = \"my-awesome-project\"".to_vec()),
///     ],
/// };
///
/// // Write processed files to disk
/// for (file_path, content) in processed.files {
///     let target_path = format!("output/{}", file_path);
///     if let Some(parent) = std::path::Path::new(&target_path).parent() {
///         fs::create_dir_all(parent)?;
///     }
///     fs::write(&target_path, content)?;
///     println!("Written: {}", target_path);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProcessedTemplate {
    pub files: Vec<(String, Vec<u8>)>, // (path, content)
}

/// Configuration for controlling which files are processed during template rendering.
///
/// This structure allows fine-grained control over which files in a template
/// repository should be processed for variable substitution. Files can be included
/// or excluded based on glob patterns.
///
/// ## Fields
///
/// * `include_patterns` - Glob patterns for files that should be processed
/// * `exclude_patterns` - Glob patterns for files that should be skipped
///
/// ## Pattern Matching
///
/// - Patterns follow standard glob syntax (`*`, `?`, `**`, etc.)
/// - Patterns are applied relative to the template repository root
/// - Exclude patterns take precedence over include patterns
/// - If no include patterns are specified, all files are included by default
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::TemplatingConfig;
///
/// // Process all Rust and markdown files, but skip test files
/// let config = TemplatingConfig {
///     include_patterns: vec![
///         "**/*.rs".to_string(),
///         "**/*.md".to_string(),
///         "Cargo.toml".to_string(),
///     ],
///     exclude_patterns: vec![
///         "**/*_test.rs".to_string(),
///         "**/*_tests.rs".to_string(),
///         "target/**".to_string(),
///     ],
/// };
///
/// // Process everything except binary files and build artifacts
/// let liberal_config = TemplatingConfig {
///     include_patterns: vec!["**/*".to_string()],
///     exclude_patterns: vec![
///         "**/*.exe".to_string(),
///         "**/*.dll".to_string(),
///         "**/*.so".to_string(),
///         "**/target/**".to_string(),
///         "**/node_modules/**".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplatingConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// Complete request for processing a template with variable substitution.
///
/// This structure contains all the information needed to process a template,
/// including user-provided variables, built-in variables, validation configurations,
/// and file filtering rules.
///
/// ## Fields
///
/// * `variables` - User-provided variables for substitution (key-value pairs)
/// * `built_in_variables` - System-generated variables (timestamps, repo info, etc.)
/// * `variable_configs` - Validation and default configurations for each variable
/// * `templating_config` - File inclusion/exclusion patterns (optional, processes all files if None)
///
/// ## Variable Resolution Priority
///
/// When processing templates, variables are resolved in this order:
/// 1. User-provided variables (`variables` field)
/// 2. Built-in variables (`built_in_variables` field)
/// 3. Default values from variable configurations
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::{TemplateProcessingRequest, VariableConfig, TemplatingConfig};
/// use std::collections::HashMap;
///
/// // Create a processing request with user variables
/// let mut variables = HashMap::new();
/// variables.insert("project_name".to_string(), "my-rust-app".to_string());
/// variables.insert("author_email".to_string(), "dev@example.com".to_string());
///
/// let mut built_in_vars = HashMap::new();
/// built_in_vars.insert("timestamp".to_string(), "2023-12-01T10:00:00Z".to_string());
/// built_in_vars.insert("repo_name".to_string(), "my-rust-app".to_string());
///
/// let mut configs = HashMap::new();
/// configs.insert("project_name".to_string(), VariableConfig {
///     description: "Name of the project".to_string(),
///     required: Some(true),
///     pattern: Some(r"^[a-z][a-z0-9-]*$".to_string()),
///     ..Default::default()
/// });
///
/// let request = TemplateProcessingRequest {
///     variables,
///     built_in_variables: built_in_vars,
///     variable_configs: configs,
///     templating_config: Some(TemplatingConfig {
///         include_patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
///         exclude_patterns: vec!["target/**".to_string()],
///     }),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TemplateProcessingRequest {
    pub variables: HashMap<String, String>,
    pub built_in_variables: HashMap<String, String>,
    pub variable_configs: HashMap<String, VariableConfig>,
    pub templating_config: Option<TemplatingConfig>,
}

/// Template processor that handles variable substitution and file processing.
///
/// This processor is the core component that takes template files and applies variable
/// substitution to create customized output files. It uses the Handlebars templating
/// engine to provide advanced template processing capabilities including control structures,
/// nested objects, and custom helpers.
///
/// The processor provides full Handlebars syntax support for advanced template processing.
/// It can generate built-in variables like timestamps and user information, and provides
/// file path templating with security validation.
///
/// # Features
///
/// - **Variable Substitution**: Complete Handlebars syntax support
/// - **Control Structures**: `{{#if}}`, `{{#each}}`, `{{#unless}}`, `{{#with}}`
/// - **Custom Helpers**: Repository-specific transformations like `{{snake_case}}`
/// - **File Path Templating**: Template file names and directory paths
/// - **Security**: Path validation and template sandboxing
/// - **HashMap Integration**: Seamless conversion from HashMap<String, String> to JSON context
///
/// # Examples
///
/// ```rust,ignore
/// use template_engine::{TemplateProcessor, TemplateProcessingRequest, TemplatingConfig};
/// use std::collections::HashMap;
/// use std::path::Path;
///
/// let processor = TemplateProcessor::new()?;
///
/// let mut variables = HashMap::new();
/// variables.insert("project_name".to_string(), "my-project".to_string());
///
/// let request = TemplateProcessingRequest {
///     variables,
///     built_in_variables: HashMap::new(),
///     variable_configs: HashMap::new(),
///     templating_config: Some(TemplatingConfig {
///         include_patterns: vec!["**/*.rs".to_string()],
///         exclude_patterns: vec!["target/**".to_string()],
///     }),
/// };
///
/// let files = vec![
///     ("README.md".to_string(), b"# {{project_name}}".to_vec()),
///     ("{{snake_case project_name}}/Cargo.toml".to_string(), b"[package]\nname = \"{{kebab_case project_name}}\"".to_vec()),
/// ];
///
/// let output_dir = Path::new("/tmp/output");
/// let result = processor.process_template(&files, &request, output_dir)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TemplateProcessor {
    /// The underlying Handlebars template engine for advanced templating
    handlebars_engine: HandlebarsTemplateEngine,
}

impl Default for TemplateProcessor {
    /// Creates a new TemplateProcessor with default settings.
    ///
    /// This is equivalent to calling `TemplateProcessor::new()`.
    ///
    /// # Panics
    ///
    /// This method will panic if the Handlebars engine cannot be initialized.
    /// This could happen if:
    /// - System is out of memory
    /// - Required dependencies are missing or corrupted
    /// - Custom helpers fail to register due to naming conflicts
    ///
    /// For error handling, use `TemplateProcessor::new()` instead.
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateProcessor: ensure sufficient memory and valid Handlebars dependencies")
    }
}

impl TemplateProcessor {
    /// Creates a new template processor instance with Handlebars engine.
    ///
    /// The processor initializes a Handlebars template engine with all custom helpers
    /// registered and default security settings. It's safe to share across threads.
    ///
    /// # Returns
    ///
    /// Returns a new `TemplateProcessor` with Handlebars engine configured, or an error
    /// if the engine initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::TemplateProcessor;
    ///
    /// let processor = TemplateProcessor::new()?;
    /// // Use processor for multiple template operations
    /// # Ok::<(), template_engine::Error>(())
    /// ```
    pub fn new() -> Result<Self, Error> {
        let mut handlebars_engine = HandlebarsTemplateEngine::new().map_err(|e| {
            Error::EngineInitialization(format!("Failed to initialize Handlebars engine: {}", e))
        })?;

        handlebars_engine.register_custom_helpers().map_err(|e| {
            Error::EngineInitialization(format!("Failed to register custom helpers: {}", e))
        })?;

        Ok(Self { handlebars_engine })
    }

    /// Convert HashMap variables to JSON format for Handlebars
    fn convert_variables_to_json(
        &self,
        variables: &HashMap<String, String>,
        built_in_variables: &HashMap<String, String>,
        variable_configs: &HashMap<String, VariableConfig>,
    ) -> Result<serde_json::Value, Error> {
        let mut all_variables = variables.clone();

        // Apply default values from variable configs for missing variables
        for (var_name, config) in variable_configs {
            if !all_variables.contains_key(var_name) {
                if let Some(ref default_value) = config.default {
                    all_variables.insert(var_name.clone(), default_value.clone());
                }
            }
        }

        // Built-in variables override user variables and defaults
        for (key, value) in built_in_variables {
            all_variables.insert(key.clone(), value.clone());
        }

        // Convert to JSON Value
        let json_map: serde_json::Map<String, serde_json::Value> = all_variables
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        Ok(serde_json::Value::Object(json_map))
    }

    /// Processes template files by applying variable substitution and filtering.
    ///
    /// This method takes a collection of template files and applies variable substitution
    /// to create customized output files. It handles both text and binary files appropriately,
    /// validates variables according to their configurations, and applies file filtering
    /// based on include/exclude patterns.
    ///
    /// # Arguments
    ///
    /// * `files` - Collection of template files as (path, content) tuples
    /// * `request` - Processing request containing variables and configuration
    /// * `_output_dir` - Target output directory (currently unused but reserved for future use)
    ///
    /// # Returns
    ///
    /// Returns a `ProcessedTemplate` containing all processed files with variables substituted,
    /// or an `Error` if processing fails.
    ///
    /// # Processing Steps
    ///
    /// 1. Validates all variables according to their configurations
    /// 2. Combines user variables with built-in variables
    /// 3. Filters files based on include/exclude patterns
    /// 4. Applies variable substitution to text files
    /// 5. Copies binary files unchanged
    /// 6. Removes `.template` suffixes from file names
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Variable validation fails (missing required variables, pattern mismatches, etc.)
    /// - File content cannot be processed due to encoding issues
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::{TemplateProcessor, TemplateProcessingRequest};
    /// use std::collections::HashMap;
    /// use std::path::Path;
    ///
    /// let processor = TemplateProcessor::new();
    ///
    /// let files = vec![
    ///     ("README.md".to_string(), b"# {{project_name}}\n\nBy {{author}}".to_vec()),
    ///     ("src/main.rs".to_string(), b"// {{project_name}} by {{author}}".to_vec()),
    /// ];
    ///
    /// let mut variables = HashMap::new();
    /// variables.insert("project_name".to_string(), "My Project".to_string());
    /// variables.insert("author".to_string(), "John Doe".to_string());
    ///
    /// let request = TemplateProcessingRequest {
    ///     variables,
    ///     built_in_variables: HashMap::new(),
    ///     variable_configs: HashMap::new(),
    ///     templating_config: None,
    /// };
    ///
    /// let result = processor.process_template(&files, &request, Path::new("./output"))?;
    /// println!("Processed {} files", result.files.len());
    /// # Ok::<(), template_engine::Error>(())
    /// ```
    pub fn process_template(
        &self,
        files: &[(String, Vec<u8>)],
        request: &TemplateProcessingRequest,
        _output_dir: &Path,
    ) -> Result<ProcessedTemplate, Error> {
        // Validate variables first
        self.validate_variables(request)?;

        // Convert HashMap variables to JSON format for Handlebars
        let all_variables = self.convert_variables_to_json(
            &request.variables,
            &request.built_in_variables,
            &request.variable_configs,
        )?;
        let context = TemplateContext::new(all_variables);

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

            // Process file path template (with security validation)
            let processed_path = self
                .handlebars_engine
                .template_file_path(file_path, &context)
                .map_err(|e| Error::VariableValidation {
                    variable: "file_path".to_string(),
                    reason: format!("File path templating failed: {}", e),
                })?;

            let processed_content = if self.is_text_file(content) {
                // Apply Handlebars template processing to text files
                let content_str = String::from_utf8_lossy(content);
                let processed_str = self
                    .handlebars_engine
                    .render_template(&content_str, &context)
                    .map_err(|e| Error::VariableValidation {
                        variable: "template_content".to_string(),
                        reason: format!("Template rendering failed: {}", e),
                    })?;
                processed_str.into_bytes()
            } else {
                // Binary files are copied as-is
                content.clone()
            };

            // Handle .template suffix removal
            let final_path = if processed_path.ends_with(".template") {
                processed_path.trim_end_matches(".template").to_string()
            } else {
                processed_path
            };

            processed_files.push((final_path, processed_content));
        }

        Ok(ProcessedTemplate {
            files: processed_files,
        })
    }

    /// Generate built-in variables for template processing.
    ///
    /// Creates a set of predefined variables that are automatically available in all templates.
    /// These variables provide context about the repository creation process, user information,
    /// and timestamp data.
    ///
    /// # Built-in Variables
    ///
    /// * `timestamp` - Current UTC timestamp in RFC3339 format (e.g., "2023-01-01T12:00:00Z")
    /// * `timestamp_unix` - Current UTC timestamp as Unix epoch seconds
    /// * `user_login` - GitHub login/username of the user creating the repository
    /// * `user_name` - Display name of the user
    /// * `org_name` - Organization name where the repository is being created
    /// * `repo_name` - Name of the new repository being created
    /// * `template_name` - Name of the template being used
    /// * `template_repo` - Full repository path of the template source
    /// * `default_branch` - Default branch name for the new repository
    ///
    /// # Future Enhancements
    ///
    /// Additional variables could include:
    /// - Repository description from the creation request
    /// - User-provided custom variables from the request
    /// - Git configuration details
    /// - License information
    /// - Project-specific metadata
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters containing user and repository information
    ///
    /// # Returns
    ///
    /// A HashMap containing variable names as keys and their string values
    pub fn generate_built_in_variables(
        &self,
        params: &BuiltInVariablesParams,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        let now = chrono::Utc::now();

        // Generate timestamp variables for template use
        variables.insert("timestamp".to_string(), now.to_rfc3339());
        variables.insert("timestamp_unix".to_string(), now.timestamp().to_string());

        // User and repository context variables
        variables.insert("user_login".to_string(), params.user_login.to_string());
        variables.insert("user_name".to_string(), params.user_name.to_string());
        variables.insert("org_name".to_string(), params.org_name.to_string());
        variables.insert("repo_name".to_string(), params.repo_name.to_string());

        // Template metadata variables
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

    /// Validates template variables against their configured constraints.
    ///
    /// This method checks all variables in the processing request against their
    /// corresponding configurations to ensure they meet requirements like:
    /// - Required variables are provided
    /// - Values match specified regex patterns
    /// - String lengths are within configured bounds
    /// - Values are from allowed option lists
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
                    reason: format!("Invalid regex pattern: {pattern}"),
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
                        reason: format!("Value too short, minimum length: {min_len}"),
                    });
                }
            }

            if let Some(max_len) = config.max_length {
                if value.len() > max_len {
                    return Err(Error::VariableValidation {
                        variable: var_name.clone(),
                        reason: format!("Value too long, maximum length: {max_len}"),
                    });
                }
            }

            // Validate against allowed options
            if let Some(ref options) = config.options {
                if !options.contains(&value) {
                    return Err(Error::VariableValidation {
                        variable: var_name.clone(),
                        reason: format!("Invalid option, allowed values: {options:?}"),
                    });
                }
            }
        }

        Ok(())
    }

    /// Determines if a file should be excluded from processing based on glob patterns.
    ///
    /// This method checks if the file path matches any of the provided exclude patterns.
    /// If any pattern matches, the file should be excluded from template processing.
    fn should_exclude_file(&self, file_path: &str, exclude_patterns: &[String]) -> bool {
        exclude_patterns.iter().any(|pattern| {
            // Simple glob matching for MVP - could be enhanced with a proper glob library
            self.simple_glob_match(pattern, file_path)
        })
    }

    /// Determines if a file should be included in processing based on glob patterns.
    ///
    /// This method checks if the file path matches any of the provided include patterns.
    /// If any pattern matches, the file should be included in template processing.
    fn should_include_file(&self, file_path: &str, include_patterns: &[String]) -> bool {
        include_patterns
            .iter()
            .any(|pattern| self.simple_glob_match(pattern, file_path))
    }

    /// Performs basic glob pattern matching for file path filtering.
    ///
    /// This implementation provides comprehensive glob pattern matching:
    /// - `**` and `**/*` match everything
    /// - `**` in the middle handles recursive directory matching
    /// - `*` handles single-level wildcards
    /// - `?` matches a single character
    /// - `[...]` matches character classes
    /// - Exact string matching for literal patterns
    ///
    /// # Arguments
    ///
    /// * `pattern` - The glob pattern to match against
    /// * `path` - The file path to test
    ///
    /// # Returns
    ///
    /// `true` if the path matches the pattern, `false` otherwise
    fn simple_glob_match(&self, pattern: &str, path: &str) -> bool {
        match Pattern::new(pattern) {
            Ok(glob_pattern) => {
                // Use custom match options to handle directory separators correctly
                let options = MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: true,
                    require_literal_leading_dot: false,
                };
                glob_pattern.matches_with(path, options)
            }
            Err(_) => {
                // If the pattern is invalid, fall back to exact string matching
                pattern == path
            }
        }
    }

    /// Determines if file content should be treated as text for variable substitution.
    ///
    /// This method uses a simple heuristic to detect text files:
    /// - Content must be valid UTF-8
    /// - Content must not contain null bytes (which typically indicate binary data)
    ///
    /// Text files will have variable substitution applied, while binary files
    /// are copied unchanged to preserve their integrity.
    fn is_text_file(&self, content: &[u8]) -> bool {
        // Simple heuristic: if we can decode as UTF-8 and it doesn't contain null bytes
        match std::str::from_utf8(content) {
            Ok(text) => !text.contains('\0'),
            Err(_) => false,
        }
    }
}

/// Configuration for template variable validation and behavior.
///
/// This structure defines how a template variable should be validated and what
/// default values or constraints apply. It allows template authors to specify
/// requirements for variables that users must provide when using the template.
///
/// ## Fields
///
/// * `description` - Human-readable description of what this variable represents
/// * `example` - Optional example value to help users understand the expected format
/// * `required` - Whether this variable must be provided (defaults to `false` if not specified)
/// * `pattern` - Optional regex pattern that the variable value must match
/// * `min_length` - Minimum length for string variables
/// * `max_length` - Maximum length for string variables
/// * `options` - List of allowed values (for enumerated variables)
/// * `default` - Default value to use if the variable is not provided
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::VariableConfig;
///
/// // Simple string variable with description
/// let name_config = VariableConfig {
///     description: "The name of the project".to_string(),
///     example: Some("my-awesome-project".to_string()),
///     required: Some(true),
///     pattern: Some(r"^[a-z][a-z0-9-]*$".to_string()), // kebab-case
///     min_length: Some(3),
///     max_length: Some(50),
///     options: None,
///     default: None,
/// };
///
/// // Enumerated variable with predefined options
/// let license_config = VariableConfig {
///     description: "The license for the project".to_string(),
///     example: Some("MIT".to_string()),
///     required: Some(false),
///     pattern: None,
///     min_length: None,
///     max_length: None,
///     options: Some(vec!["MIT".to_string(), "Apache-2.0".to_string(), "GPL-3.0".to_string()]),
///     default: Some("MIT".to_string()),
/// };
/// ```
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
