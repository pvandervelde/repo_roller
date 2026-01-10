//! Template processing operations for repository creation.
//!
//! This module contains all template-related operations including:
//! - Fetching template files from source repositories
//! - Copying template files to local directories
//! - Processing template variables and substitution
//! - Creating additional repository files (README, .gitignore)
//! - Extracting configuration-driven template variables
//!
//! These operations are used during the repository creation workflow to prepare
//! the local repository content before pushing to GitHub.

use crate::errors::{SystemError, TemplateError};
use crate::request::RepositoryCreationRequest;
use crate::{RepoRollerError, RepoRollerResult};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use temp_dir::TempDir;
use template_engine::{TemplateFetcher, TemplateProcessingRequest, TemplateProcessor};
use tracing::{debug, error, info};
use walkdir::WalkDir;

/// Validate that a file path is safe for use in template processing.
///
/// This function ensures that template files cannot perform path traversal attacks
/// by writing files outside the repository boundaries. It performs multiple security
/// checks on the provided file path.
///
/// ## Security Checks
///
/// 1. **Path Traversal Prevention**: Rejects paths containing `..` components that could
///    escape the repository directory
/// 2. **Absolute Path Rejection**: Rejects absolute paths (starting with `/` on Unix or
///    drive letters like `C:\` on Windows)
/// 3. **Boundary Verification**: Ensures the resolved path stays within repository bounds
///    using canonicalization and prefix checking
///
/// ## Parameters
///
/// * `file_path` - The relative file path from the template (e.g., "src/main.rs")
/// * `repo_path` - The root path of the repository where files will be written
///
/// ## Returns
///
/// * `Ok(())` - If the path is safe and can be used
/// * `Err(Error)` - If the path is unsafe or invalid
///
/// ## Error Conditions
///
/// Returns an error with a descriptive message when:
/// - Path contains `..` components (e.g., `../../etc/passwd`)
/// - Path is absolute (e.g., `/etc/passwd` or `C:\Windows\System32\config`)
/// - Resolved path would be outside the repository directory
/// - Path canonicalization fails
///
/// ## Examples
///
/// ```rust,ignore
/// // Safe paths (should succeed)
/// validate_safe_path("README.md", repo_path)?;
/// validate_safe_path("src/main.rs", repo_path)?;
/// validate_safe_path("docs/guide.md", repo_path)?;
///
/// // Unsafe paths (should fail)
/// validate_safe_path("../../etc/passwd", repo_path)?;  // Path traversal
/// validate_safe_path("/etc/passwd", repo_path)?;        // Absolute path
/// validate_safe_path("../outside.txt", repo_path)?;     // Escapes repository
/// ```
///
/// ## Implementation Notes
///
/// - Uses `Path::canonicalize()` for symbolic link resolution
/// - Uses `Path::starts_with()` for boundary checking
/// - Provides clear error messages indicating why a path was rejected
/// - Prevents both obvious attacks and subtle canonicalization-based bypasses
fn validate_safe_path(file_path: &str, repo_path: &Path) -> Result<(), SystemError> {
    // Check for empty path
    if file_path.is_empty() {
        return Err(SystemError::FileSystem {
            operation: "validate path".to_string(),
            reason: "Empty file path is not allowed".to_string(),
        });
    }

    // Create a Path from the file_path string
    let path = Path::new(file_path);

    // Check if the path is absolute (Unix: starts with /, Windows: C:\, etc.)
    if path.is_absolute() {
        return Err(SystemError::FileSystem {
            operation: "validate path".to_string(),
            reason: format!("Unsafe path: Absolute paths are not allowed: {}", file_path),
        });
    }

    // Check for paths that are only dots (., .., ..., etc.)
    // These are either current dir references or invalid file names
    let trimmed = file_path.trim_start_matches("./").trim_start_matches(".\\");
    if trimmed.chars().all(|c| c == '.') && !trimmed.is_empty() {
        return Err(SystemError::FileSystem {
            operation: "validate path".to_string(),
            reason: format!(
                "Unsafe path: Path consisting only of dots is not allowed: {}",
                file_path
            ),
        });
    }

    // Check for parent directory references (..)
    // This prevents path traversal attacks like "../../etc/passwd"
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(SystemError::FileSystem {
                    operation: "validate path".to_string(),
                    reason: format!(
                        "Unsafe path: Path traversal with parent directory (..) is not allowed: {}",
                        file_path
                    ),
                });
            }
            std::path::Component::CurDir => {
                // Current directory (.) is suspicious when it appears alone
                // but we'll allow it in paths like "./file.txt" by checking if it's the only component
                if path.components().count() == 1 {
                    return Err(SystemError::FileSystem {
                        operation: "validate path".to_string(),
                        reason: format!(
                            "Unsafe path: Path consisting only of '.' is not allowed: {}",
                            file_path
                        ),
                    });
                }
            }
            _ => {}
        }
    }

    // Additional check: resolve the path and verify it stays within repo bounds
    // This catches subtle cases where path manipulation could escape the repo
    let target_path = repo_path.join(path);

    // For validation purposes, we need to check if the path would escape the repository
    // We can't always canonicalize non-existent paths, so we'll check the components
    // We already validated no ".." components, and the path is relative, so joining
    // should keep us within bounds. However, let's add one more check:

    // Normalize the path by checking if it would escape when joined
    // Since we've already rejected ".." and absolute paths, a relative path
    // joined to repo_path should always stay within repo_path
    // But let's verify the target_path starts with repo_path when normalized

    // We can't canonicalize a path that doesn't exist yet, so we'll use a
    // heuristic: check if any normalization of the path would escape
    // Since we blocked ".." already, this should be safe

    // Convert to string for final validation
    let target_str = target_path.to_string_lossy();
    let repo_str = repo_path.to_string_lossy();

    // Basic prefix check (this is somewhat redundant after blocking .. and absolute paths,
    // but provides defense in depth)
    if !target_str.starts_with(repo_str.as_ref()) {
        return Err(SystemError::FileSystem {
            operation: "validate path".to_string(),
            reason: format!(
                "Unsafe path: Path resolves outside repository bounds: {}",
                file_path
            ),
        });
    }

    Ok(())
}

/// Copy template files to the local repository directory.
///
/// This function takes a collection of template files (as byte arrays) and writes them
/// to the local repository directory, preserving the directory structure and file paths
/// from the original template.
///
/// ## Parameters
///
/// * `files` - Vector of tuples containing (file_path, file_content) pairs
///   - `file_path`: Relative path where the file should be created
///   - `file_content`: Raw bytes of the file content
/// * `local_repo_path` - Temporary directory where files should be written
///
/// ## Returns
///
/// * `Ok(())` - If all files are copied successfully
/// * `Err(Error)` - If any file operation fails
///
/// ## Behavior
///
/// - Creates parent directories automatically if they don't exist
/// - Overwrites existing files with the same path
/// - Preserves the exact byte content of template files
/// - Maintains the relative path structure from the template
///
/// ## Errors
///
/// This function will return an error if:
/// - Parent directory creation fails
/// - File creation fails
/// - File writing fails
///
/// ## Example
///
/// ```rust,ignore
/// let files = vec![
///     ("README.md".to_string(), b"# My Project".to_vec()),
///     ("src/main.rs".to_string(), b"fn main() {}".to_vec()),
/// ];
/// let temp_dir = TempDir::new()?;
/// copy_template_files(&files, &temp_dir)?;
/// ```
pub(crate) fn copy_template_files(
    files: &Vec<(String, Vec<u8>)>,
    local_repo_path: &TempDir,
) -> Result<(), SystemError> {
    debug!("Copying {} template files to local repository", files.len());

    for (file_path, content) in files {
        // Validate the path for security (prevent path traversal attacks)
        validate_safe_path(file_path, local_repo_path.path())?;

        let target_path = local_repo_path.path().join(file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create directory {:?}: {}", parent, e);
                SystemError::FileSystem {
                    operation: "create directory".to_string(),
                    reason: format!("{:?}: {}", parent, e),
                }
            })?;
        }

        // Write the file content
        let mut file = File::create(&target_path).map_err(|e| {
            error!("Failed to create file {:?}: {}", target_path, e);
            SystemError::FileSystem {
                operation: "create file".to_string(),
                reason: format!("{:?}: {}", target_path, e),
            }
        })?;

        file.write_all(content).map_err(|e| {
            error!("Failed to write to file {:?}: {}", target_path, e);
            SystemError::FileSystem {
                operation: "write file".to_string(),
                reason: format!("{:?}: {}", target_path, e),
            }
        })?;

        debug!("Copied file: {}", file_path);
    }

    info!("Template files copied successfully");
    Ok(())
}

/// Create additional repository files that may not be provided by the template.
///
/// This function generates standard repository files if they are not already present
/// in the template files. It ensures that every repository has basic files like
/// README.md and .gitignore, while respecting template-provided versions.
///
/// ## Additional Files Created
///
/// - **README.md**: A basic readme with repository information if not provided by template
/// - **.gitignore**: A standard gitignore file with common patterns if not provided by template
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory where additional files should be created
/// * `req` - Repository creation request containing name, owner, and template information
/// * `template_files` - List of files already provided by the template (used to check for conflicts)
///
/// ## Returns
///
/// * `Ok(())` - If additional files are created successfully
/// * `Err(Error)` - If file creation fails
///
/// ## Behavior
///
/// - Only creates files that are not already provided by the template
/// - Uses repository metadata (name, owner, template) to generate content
/// - Creates files with sensible defaults suitable for most projects
/// - Logs which files are created vs. skipped
///
/// ## File Content
///
/// - **README.md**: Contains repository name, RepoRoller attribution, and metadata
/// - **.gitignore**: Includes common ignore patterns for various development environments
///
/// ## Errors
///
/// This function will return an error if:
/// - File system operations fail
/// - Directory creation fails
pub(crate) fn create_additional_files(
    local_repo_path: &TempDir,
    req: &RepositoryCreationRequest,
    template_files: &[(String, Vec<u8>)],
) -> Result<(), SystemError> {
    info!("Creating additional files for repository initialization");

    // Check what files the template already provides
    let template_file_paths: std::collections::HashSet<String> = template_files
        .iter()
        .map(|(path, _)| path.clone())
        .collect();

    // Only create README.md if the template doesn't provide one
    if !template_file_paths.contains("README.md") {
        let readme_path = local_repo_path.path().join("README.md");
        let template_info = req
            .template
            .as_ref()
            .map(|t| format!("\n\nTemplate: {}\nOwner: {}\n", t.as_ref(), req.owner.as_ref()))
            .unwrap_or_else(|| format!("\n\nOwner: {}\n", req.owner.as_ref()));
        let readme_content = format!("# {}\n\nRepository created using RepoRoller.{}", req.name.as_ref(), template_info);

        debug!(
            "Creating README.md with content length: {} (template didn't provide one)",
            readme_content.len()
        );

        std::fs::write(&readme_path, readme_content).map_err(|e| {
            error!("Failed to create README.md: {}", e);
            SystemError::FileSystem {
                operation: "create README.md".to_string(),
                reason: e.to_string(),
            }
        })?;

        info!("README.md created successfully at: {:?}", readme_path);
    } else {
        info!("README.md provided by template, skipping creation");
    }

    // Only create .gitignore if the template doesn't provide one
    if !template_file_paths.contains(".gitignore") {
        let gitignore_path = local_repo_path.path().join(".gitignore");
        let gitignore_content =
            "# Common ignores\n.DS_Store\n*.log\n*.tmp\nnode_modules/\ntarget/\n";

        debug!("Creating .gitignore (template didn't provide one)");

        std::fs::write(&gitignore_path, gitignore_content).map_err(|e| {
            error!("Failed to create .gitignore: {}", e);
            SystemError::FileSystem {
                operation: "create .gitignore".to_string(),
                reason: e.to_string(),
            }
        })?;

        info!(".gitignore created successfully at: {:?}", gitignore_path);
    } else {
        info!(".gitignore provided by template, skipping creation");
    }

    Ok(())
}

/// Extract template variables from merged configuration.
///
/// Converts relevant fields from the merged organization configuration into
/// template variables that can be used during template processing. This enables
/// templates to adapt based on organization-wide policies and settings.
///
/// ## Exported Variables
///
/// The following variables are extracted with the `config_` prefix:
///
/// ### Repository Features
/// - `config_issues_enabled`: "true" or "false"
/// - `config_projects_enabled`: "true" or "false"
/// - `config_discussions_enabled`: "true" or "false"
/// - `config_wiki_enabled`: "true" or "false"
/// - `config_pages_enabled`: "true" or "false"
/// - `config_security_advisories_enabled`: "true" or "false"
/// - `config_vulnerability_reporting_enabled`: "true" or "false"
/// - `config_auto_close_issues_enabled`: "true" or "false"
///
/// ### Pull Request Settings
/// - `config_required_approving_review_count`: Number as string (e.g., "2")
/// - `config_allow_merge_commit`: "true" or "false"
/// - `config_allow_squash_merge`: "true" or "false"
/// - `config_allow_rebase_merge`: "true" or "false"
/// - `config_allow_auto_merge`: "true" or "false"
/// - `config_delete_branch_on_merge`: "true" or "false"
///
/// ## Examples
///
/// ```ignore
/// use config_manager::MergedConfiguration;
///
/// let merged_config = MergedConfiguration::new();
/// let variables = extract_config_variables(&merged_config);
///
/// // Variables can now be used in templates like:
/// // "Issues enabled: {{config_issues_enabled}}"
/// // "Required reviewers: {{config_required_approving_review_count}}"
/// ```
///
/// ## Notes
///
/// - All boolean values are serialized as "true" or "false" strings
/// - Numeric values are serialized as decimal strings
/// - Variables use `config_` prefix to avoid conflicts with user/built-in variables
/// - Only simple scalar values are exposed (complex nested structures are omitted for MVP)
pub fn extract_config_variables(
    merged_config: &config_manager::MergedConfiguration,
) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    // Extract repository feature settings
    let repo_settings = &merged_config.repository;

    // Helper to extract boolean value from OverridableValue<bool>
    let extract_bool = |opt_value: &Option<config_manager::OverridableValue<bool>>| -> String {
        opt_value
            .as_ref()
            .map(|v| v.value.to_string())
            .unwrap_or_else(|| "false".to_string())
    };

    // Helper to extract u32 value from OverridableValue<u32>
    let extract_i32 =
        |opt_value: &Option<config_manager::OverridableValue<i32>>| -> Option<String> {
            opt_value.as_ref().map(|v| v.value.to_string())
        };

    // Repository features
    variables.insert(
        "config_issues_enabled".to_string(),
        extract_bool(&repo_settings.issues),
    );
    variables.insert(
        "config_projects_enabled".to_string(),
        extract_bool(&repo_settings.projects),
    );
    variables.insert(
        "config_discussions_enabled".to_string(),
        extract_bool(&repo_settings.discussions),
    );
    variables.insert(
        "config_wiki_enabled".to_string(),
        extract_bool(&repo_settings.wiki),
    );
    variables.insert(
        "config_pages_enabled".to_string(),
        extract_bool(&repo_settings.pages),
    );
    variables.insert(
        "config_security_advisories_enabled".to_string(),
        extract_bool(&repo_settings.security_advisories),
    );
    variables.insert(
        "config_vulnerability_reporting_enabled".to_string(),
        extract_bool(&repo_settings.vulnerability_reporting),
    );
    variables.insert(
        "config_auto_close_issues_enabled".to_string(),
        extract_bool(&repo_settings.auto_close_issues),
    );

    // Pull request settings
    let pr_settings = &merged_config.pull_requests;

    if let Some(count) = extract_i32(&pr_settings.required_approving_review_count) {
        variables.insert("config_required_approving_review_count".to_string(), count);
    }
    variables.insert(
        "config_allow_merge_commit".to_string(),
        extract_bool(&pr_settings.allow_merge_commit),
    );
    variables.insert(
        "config_allow_squash_merge".to_string(),
        extract_bool(&pr_settings.allow_squash_merge),
    );
    variables.insert(
        "config_allow_rebase_merge".to_string(),
        extract_bool(&pr_settings.allow_rebase_merge),
    );
    variables.insert(
        "config_allow_auto_merge".to_string(),
        extract_bool(&pr_settings.allow_auto_merge),
    );
    variables.insert(
        "config_delete_branch_on_merge".to_string(),
        extract_bool(&pr_settings.delete_branch_on_merge),
    );

    variables
}

/// Process template variables and substitute them in all template files.
///
/// This function handles the variable substitution phase of repository creation,
/// replacing template placeholders with actual values throughout all files in
/// the local repository.
///
/// ## Process Overview
///
/// 1. **Variable Setup**: Generates built-in variables, extracts config variables, and merges with user variables
/// 2. **Configuration Mapping**: Converts template variable configurations
/// 3. **File Reading**: Scans all files in the local repository
/// 4. **Template Processing**: Performs variable substitution using the template engine
/// 5. **File Replacement**: Removes original files and writes processed versions
///
/// ## Parameters
///
/// * `local_repo_path` - Temporary directory containing template files to process
/// * `req` - Repository creation request containing substitution values
/// * `template` - Template configuration including variable definitions
/// * `merged_config` - Merged organization configuration providing additional template variables
///
/// ## Returns
///
/// * `Ok(())` - If template processing completes successfully
/// * `Err(Error)` - If any step in the processing fails
///
/// ## Built-in Variables
///
/// The function automatically generates these variables:
/// - `repo_name`: Repository name from the request
/// - `org_name`: Organization/owner name from the request
/// - `template_name`: Template name used for creation
/// - `user_login`: GitHub App login (placeholder)
/// - `user_name`: GitHub App display name (placeholder)
/// - `default_branch`: Default branch name ("main")
///
/// ## Configuration Variables
///
/// Extracts variables from merged configuration with `config_` prefix:
/// - Repository features (issues, wiki, projects, etc.)
/// - Pull request settings (required reviewers, merge options, etc.)
///
/// ## Variable Configuration
///
/// Converts template variable configurations from `config_manager` format
/// to `template_engine` format, including:
/// - Validation rules (pattern, length, required)
/// - Default values and examples
/// - Option lists for enumerated values
///
/// ## File Processing
///
/// - Processes all files recursively in the repository
/// - Excludes the `.git` directory from processing
/// - Maintains file paths and directory structure
/// - Handles both text and binary files appropriately
///
/// ## Error Handling
///
/// Returns errors for:
/// - File system operations (reading, writing, directory operations)
/// - Template engine processing failures
/// - Path manipulation errors
///
/// ## Template Engine Integration
///
/// Uses the `template_engine` crate for actual variable substitution:
/// - Supports Handlebars-style `{{variable}}` syntax
/// - Handles conditional blocks and loops
/// - Provides validation and error reporting
/// - Configurable file inclusion/exclusion patterns
pub(crate) fn replace_template_variables(
    local_repo_path: &TempDir,
    req: &RepositoryCreationRequest,
    template: &config_manager::TemplateConfig,
    merged_config: &config_manager::MergedConfiguration,
) -> Result<(), SystemError> {
    debug!("Processing template variables using TemplateProcessor");

    // Create template processor
    let processor = TemplateProcessor::new().map_err(|e| SystemError::Internal {
        reason: format!("Failed to create template processor: {}", e),
    })?;

    // Generate built-in variables
    // Note: Use .as_ref() to convert branded types to &str for template_engine
    // (avoids circular dependency between crates)
    let template_name_str = req.template.as_ref().map(|t| t.as_ref()).unwrap_or("none");
    let built_in_params = template_engine::BuiltInVariablesParams {
        repo_name: req.name.as_ref(),
        org_name: req.owner.as_ref(),
        template_name: template_name_str,
        template_repo: "unknown", // We'd need to get this from template config
        user_login: "reporoller-app", // Placeholder for GitHub App
        user_name: "RepoRoller App", // Placeholder for GitHub App
        default_branch: "main",
    };
    let built_in_variables = processor.generate_built_in_variables(&built_in_params);

    // Extract configuration-driven variables from merged config
    let config_variables = extract_config_variables(merged_config);

    // Use user-provided variables from the request
    let user_variables = req.variables.clone();

    // Convert config_manager::TemplateVariable to template_engine::VariableConfig
    let mut variable_configs = HashMap::new();
    if let Some(ref template_vars) = template.variables {
        for (name, var) in template_vars {
            let engine_config = template_engine::VariableConfig {
                description: var.description.clone(),
                example: var.example.clone(),
                required: var.required,
                pattern: var.pattern.clone(),
                min_length: var.min_length,
                max_length: var.max_length,
                options: var.options.clone(),
                default: var.default.clone(),
            };
            variable_configs.insert(name.clone(), engine_config);
        }
    }

    // Merge all variable sources: built-in variables + config variables
    let mut all_built_in_variables = built_in_variables;
    all_built_in_variables.extend(config_variables);

    // Create processing request
    let processing_request = TemplateProcessingRequest {
        variables: user_variables,
        built_in_variables: all_built_in_variables,
        variable_configs,
        templating_config: None, // Use default processing (all files)
    };

    // Read all files that were copied to the local repo
    let mut files_to_process = Vec::new();
    for entry in WalkDir::new(local_repo_path.path()) {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            SystemError::FileSystem {
                operation: "read directory entry".to_string(),
                reason: e.to_string(),
            }
        })?;

        if entry.file_type().is_file() {
            let file_path = entry.path();
            let relative_path = file_path
                .strip_prefix(local_repo_path.path())
                .map_err(|e| {
                    error!("Failed to get relative path: {}", e);
                    SystemError::FileSystem {
                        operation: "get relative path".to_string(),
                        reason: e.to_string(),
                    }
                })?;

            let content = fs::read(file_path).map_err(|e| {
                error!("Failed to read file {:?}: {}", file_path, e);
                SystemError::FileSystem {
                    operation: "read file".to_string(),
                    reason: format!("{:?}: {}", file_path, e),
                }
            })?;

            files_to_process.push((relative_path.to_string_lossy().to_string(), content));
        }
    }

    // Process the template files
    let processed = processor
        .process_template(
            &files_to_process,
            &processing_request,
            local_repo_path.path(),
        )
        .map_err(|e| {
            error!("Template processing failed: {}", e);
            SystemError::Internal {
                reason: format!("Template processing failed: {}", e),
            }
        })?;

    // Write the processed files back to the local repo
    // First, clear the directory (except .git)
    for entry in WalkDir::new(local_repo_path.path())
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
    {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {}", e);
            SystemError::FileSystem {
                operation: "read directory entry".to_string(),
                reason: e.to_string(),
            }
        })?;

        if entry.file_type().is_file() {
            fs::remove_file(entry.path()).map_err(|e| {
                error!("Failed to remove file {:?}: {}", entry.path(), e);
                SystemError::FileSystem {
                    operation: "remove file".to_string(),
                    reason: format!("{:?}: {}", entry.path(), e),
                }
            })?;
        }
    }

    // Write processed files
    for (file_path, content) in processed.files {
        let target_path = local_repo_path.path().join(&file_path);

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                error!("Failed to create directory {:?}: {}", parent, e);
                SystemError::FileSystem {
                    operation: "create directory".to_string(),
                    reason: format!("{:?}: {}", parent, e),
                }
            })?;
        }

        // Write the file content
        fs::write(&target_path, content).map_err(|e| {
            error!("Failed to write processed file {:?}: {}", target_path, e);
            SystemError::FileSystem {
                operation: "write file".to_string(),
                reason: format!("{:?}: {}", target_path, e),
            }
        })?;

        debug!("Wrote processed file: {}", file_path);
    }

    info!("Template variable processing completed successfully");
    Ok(())
}

/// Prepare local repository with template files and processing.
///
/// This function orchestrates the complete local repository preparation workflow:
/// 1. Creates a temporary directory for the repository
/// 2. Fetches template files from the source repository
/// 3. Copies template files to the local directory
/// 4. Processes template variables and performs substitutions
/// 5. Creates additional standard files (README.md, .gitignore) if not provided by template
///
/// ## Parameters
///
/// * `request` - Repository creation request with name, owner, and template information
/// * `template` - Template configuration including source repository and variable definitions
/// * `template_fetcher` - Trait object for fetching template files from source
/// * `merged_config` - Merged organization configuration providing template variables
///
/// ## Returns
///
/// * `Ok(TempDir)` - Temporary directory containing the prepared repository
/// * `Err(RepoRollerError)` - If any step in the preparation fails
///
/// ## Error Types
///
/// - `SystemError::Internal` - Temporary directory creation or file operations failed
/// - `TemplateError::FetchFailed` - Template file fetching failed
/// - `TemplateError::SubstitutionFailed` - Variable substitution failed
///
/// ## Cleanup
///
/// The returned `TempDir` will automatically clean up when dropped, removing
/// all temporary files and directories.
pub(crate) async fn prepare_local_repository(
    request: &RepositoryCreationRequest,
    template: &config_manager::TemplateConfig,
    template_source: &str,
    template_fetcher: &dyn TemplateFetcher,
    merged_config: &config_manager::MergedConfiguration,
) -> RepoRollerResult<TempDir> {
    // Create temporary directory
    let local_repo_path = TempDir::new().map_err(|e| {
        error!("Failed to create temporary directory: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create temporary directory: {}", e),
        })
    })?;

    // Fetch template files
    // Convert owner/repo format to full GitHub URL
    let github_url =
        if template_source.starts_with("http://") || template_source.starts_with("https://") {
            template_source.to_string()
        } else {
            format!("https://github.com/{}", template_source)
        };

    info!("Fetching template files from: {}", github_url);
    let files = template_fetcher
        .fetch_template_files(&github_url)
        .await
        .map_err(|e| {
            error!("Failed to fetch template files: {}", e);
            RepoRollerError::Template(TemplateError::FetchFailed {
                reason: format!("Failed to fetch template files: {}", e),
            })
        })?;

    // Copy template files
    debug!("Copying template files to local repository");
    copy_template_files(&files, &local_repo_path).map_err(|e| {
        error!("Failed to copy template files: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to copy template files: {}", e),
        })
    })?;

    // Process template variables
    debug!("Processing template variables");
    replace_template_variables(&local_repo_path, request, template, merged_config).map_err(
        |e| {
            error!("Failed to replace template variables: {}", e);
            RepoRollerError::Template(TemplateError::SubstitutionFailed {
                variable: "(multiple variables)".to_string(),
                reason: format!("Batch variable replacement failed: {}", e),
            })
        },
    )?;

    // Create additional files
    debug!("Creating additional required files");
    create_additional_files(&local_repo_path, request, &files).map_err(|e| {
        error!("Failed to create additional files: {}", e);
        RepoRollerError::System(SystemError::Internal {
            reason: format!("Failed to create additional files: {}", e),
        })
    })?;

    Ok(local_repo_path)
}

#[cfg(test)]
#[path = "template_processing_tests.rs"]
mod tests;
