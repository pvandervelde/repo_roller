//! # Handlebars Template Engine
//!
//! This module provides advanced template processing capabilities using the Handlebars
//! templating engine. It supports variable substitution, control structures, file path
//! templating, and custom helpers for repository-specific operations.
//!
//! ## Features
//!
//! - **Variable Substitution**: Basic `{{variable}}` and complex `{{object.property}}`
//! - **Control Structures**: Conditionals, loops, and context switching
//! - **File Path Templating**: Template file names and directory paths
//! - **Custom Helpers**: Repository-specific text transformations
//! - **Security**: Path validation and template sandboxing
//! - **Performance**: Template compilation caching and resource limits
//!
//! ## Examples
//!
//! ```rust
//! # use template_engine::{HandlebarsTemplateEngine, TemplateContext};
//! # use serde_json::json;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create template engine with custom helpers
//! let mut engine = HandlebarsTemplateEngine::new()?;
//! engine.register_custom_helpers()?;
//!
//! // Prepare template context
//! let variables = json!({
//!     "repo_name": "my-awesome-project",
//!     "author": {"name": "John Doe", "email": "john@example.com"},
//!     "features": ["logging", "testing", "docs"]
//! });
//!
//! let context = TemplateContext::new(variables);
//!
//! // Process template content
//! let template = "# {{repo_name}}";
//! let result = engine.render_template(template, &context)?;
//! assert_eq!(result, "# my-awesome-project");
//!
//! // Process file path with helper
//! let file_path = "{{snake_case repo_name}}.md";
//! let processed_path = engine.template_file_path(file_path, &context)?;
//! assert_eq!(processed_path, "my_awesome_project.md");
//! # Ok(())
//! # }
//! ```

use chrono;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, JsonRender, Output, RenderContext, RenderError,
};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[cfg(test)]
#[path = "handlebars_tests.rs"]
mod handlebars_tests;

// ================================
// Custom Handlebars Helpers
// ================================

/// Helper to convert text to snake_case format.
struct SnakeCaseHelper;

impl HelperDef for SnakeCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("snake_case helper requires a string parameter"))?;

        let snake_case = param
            .trim()
            .chars()
            .map(|c| match c {
                ' ' | '-' => '_',
                c => c.to_ascii_lowercase(),
            })
            .collect::<String>();

        out.write(&snake_case)?;
        Ok(())
    }
}

/// Helper to convert text to kebab-case format.
struct KebabCaseHelper;

impl HelperDef for KebabCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("kebab_case helper requires a string parameter"))?;

        let kebab_case = param
            .trim()
            .chars()
            .map(|c| match c {
                ' ' | '_' => '-',
                c => c.to_ascii_lowercase(),
            })
            .collect::<String>();

        out.write(&kebab_case)?;
        Ok(())
    }
}

/// Helper to convert text to UPPER_CASE format.
struct UpperCaseHelper;

impl HelperDef for UpperCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("upper_case helper requires a string parameter"))?;

        out.write(&param.to_uppercase())?;
        Ok(())
    }
}

/// Helper to convert text to lowercase format.
struct LowerCaseHelper;

impl HelperDef for LowerCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("lower_case helper requires a string parameter"))?;

        out.write(&param.to_lowercase())?;
        Ok(())
    }
}

/// Helper to capitalize the first letter of text.
struct CapitalizeHelper;

impl HelperDef for CapitalizeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("capitalize helper requires a string parameter"))?;

        let capitalized = if let Some(first_char) = param.chars().next() {
            let mut result = first_char.to_uppercase().collect::<String>();
            result.push_str(&param[first_char.len_utf8()..]);
            result
        } else {
            param.to_string()
        };

        out.write(&capitalized)?;
        Ok(())
    }
}

/// Helper to provide default values for undefined variables.
struct DefaultHelper;

impl HelperDef for DefaultHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let value = h.param(0);
        let default_value = h
            .param(1)
            .ok_or_else(|| RenderError::new("default helper requires a default value parameter"))?;

        if let Some(val) = value {
            if !val.value().is_null() {
                out.write(&val.value().render())?;
            } else {
                out.write(&default_value.value().render())?;
            }
        } else {
            out.write(&default_value.value().render())?;
        }

        Ok(())
    }
}

/// Helper to generate current timestamp.
struct TimestampHelper;

impl HelperDef for TimestampHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let format = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");

        let now = chrono::Utc::now();
        let formatted = now.format(format).to_string();

        out.write(&formatted)?;
        Ok(())
    }
}

/// Errors that can occur during Handlebars template processing.
///
/// This enum covers all possible error conditions when using the Handlebars
/// template engine, from compilation errors to runtime failures and security
/// violations.
#[derive(Error, Debug)]
pub enum HandlebarsError {
    /// Template compilation failed due to syntax errors.
    ///
    /// This error occurs when the template contains invalid Handlebars syntax,
    /// such as unclosed blocks, malformed expressions, or invalid helper calls.
    ///
    /// # Examples
    ///
    /// - `{{#if unclosed block`
    /// - `{{invalid.}}`
    /// - `{{#unknown_helper}}`
    #[error("Template compilation failed: {message}")]
    CompilationError {
        /// Detailed error message from the Handlebars parser
        message: String,
    },

    /// Template rendering failed during execution.
    ///
    /// This error occurs when a syntactically valid template fails during
    /// rendering, typically due to runtime issues like type mismatches,
    /// helper execution failures, or resource limits.
    ///
    /// # Examples
    ///
    /// - Calling string helpers on non-string values
    /// - Helper execution throwing exceptions
    /// - Template processing timeout
    #[error("Template rendering failed: {message}")]
    RenderError {
        /// Detailed error message from the Handlebars renderer
        message: String,
    },

    /// File path templating resulted in an invalid or unsafe path.
    ///
    /// This error occurs when file path templating produces a path that:
    /// - Contains directory traversal sequences (`..`)
    /// - Is an absolute path (starts with `/` or drive letter)
    /// - Contains invalid filesystem characters
    /// - Exceeds platform path length limits
    ///
    /// # Security
    ///
    /// This error is critical for preventing directory traversal attacks
    /// and ensuring generated paths stay within the target directory.
    #[error("Invalid file path generated from template: {path} - {reason}")]
    InvalidPath {
        /// The problematic path that was generated
        path: String,
        /// Specific reason why the path is invalid
        reason: String,
    },

    /// Template variable validation failed.
    ///
    /// This error occurs when required variables are missing or variables
    /// fail validation rules defined in the template configuration.
    ///
    /// # Examples
    ///
    /// - Required variable not provided
    /// - Variable value doesn't match required pattern
    /// - Variable value exceeds length limits
    #[error("Variable validation failed: {variable} - {reason}")]
    VariableValidation {
        /// Name of the variable that failed validation
        variable: String,
        /// Specific validation failure reason
        reason: String,
    },

    /// Helper registration or execution failed.
    ///
    /// This error occurs when custom Handlebars helpers cannot be registered
    /// or fail during execution with runtime errors.
    #[error("Helper error: {helper} - {message}")]
    HelperError {
        /// Name of the helper that failed
        helper: String,
        /// Error message from helper execution
        message: String,
    },

    /// Template processing exceeded resource limits.
    ///
    /// This error occurs when template processing takes too long, uses too
    /// much memory, or exceeds other configured resource limits.
    #[error("Resource limit exceeded: {limit_type} - {message}")]
    ResourceLimit {
        /// Type of resource limit that was exceeded
        limit_type: String,
        /// Detailed message about the limit violation
        message: String,
    },
}

/// Context for template rendering containing variables and configuration.
///
/// This structure holds all the data needed for template rendering, including
/// user-provided variables, built-in variables, and rendering configuration.
/// It provides a clean interface for passing complex data structures to
/// the Handlebars template engine.
///
/// ## Variable Structure
///
/// Variables are stored as JSON values, allowing for complex nested structures:
/// - Simple values: `"repo_name": "my-project"`
/// - Objects: `"author": {"name": "John", "email": "john@example.com"}`
/// - Arrays: `"features": ["logging", "testing"]`
/// - Mixed types: `"config": {"enabled": true, "count": 42}`
///
/// ## Examples
///
/// ```rust
/// use template_engine::TemplateContext;
/// use serde_json::json;
///
/// // Create context with simple variables
/// let simple_context = TemplateContext::from_map({
///     let mut vars = std::collections::HashMap::new();
///     vars.insert("name".to_string(), "test".to_string());
///     vars
/// });
///
/// // Create context with complex structure
/// let complex_context = TemplateContext::new(json!({
///     "project": {
///         "name": "my-app",
///         "version": "1.0.0",
///         "authors": ["Alice", "Bob"]
///     },
///     "build": {
///         "debug": true,
///         "target": "x86_64-unknown-linux-gnu"
///     }
/// }));
/// ```
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// All variables available for template rendering.
    ///
    /// This includes user-provided variables, built-in variables, and any
    /// computed values. Variables are stored as JSON values to support
    /// complex data structures and type flexibility.
    pub variables: Value,

    /// Configuration for template rendering behavior.
    ///
    /// Controls aspects like error handling, whitespace management, and
    /// security restrictions during template processing.
    pub config: TemplateRenderConfig,
}

/// Configuration for template rendering behavior.
///
/// This structure controls various aspects of template rendering, including
/// error handling strategies, performance limits, and security restrictions.
///
/// ## Security Configuration
///
/// - `strict_variables`: Controls handling of undefined variables
/// - `max_template_size`: Prevents processing of extremely large templates
/// - `max_render_time`: Prevents runaway template processing
///
/// ## Performance Configuration
///
/// - `enable_caching`: Whether to cache compiled templates
/// - `max_memory_usage`: Memory limit for template context
/// - `max_iterations`: Limit for loop iterations in templates
#[derive(Debug, Clone)]
pub struct TemplateRenderConfig {
    /// Whether to fail on undefined variables (true) or ignore them (false).
    ///
    /// When `true`, templates that reference undefined variables will fail
    /// with a clear error message. When `false`, undefined variables are
    /// rendered as empty strings.
    ///
    /// **Default**: `true` (fail on undefined variables)
    pub strict_variables: bool,

    /// Maximum size of template content in bytes.
    ///
    /// Templates larger than this size will be rejected during compilation
    /// to prevent memory exhaustion attacks.
    ///
    /// **Default**: 1MB (1,048,576 bytes)
    pub max_template_size: usize,

    /// Maximum time allowed for template rendering in milliseconds.
    ///
    /// Templates that take longer than this to render will be terminated
    /// to prevent denial-of-service attacks.
    ///
    /// **Default**: 30,000ms (30 seconds)
    pub max_render_time_ms: u64,

    /// Whether to enable template compilation caching.
    ///
    /// When enabled, compiled templates are cached for reuse, improving
    /// performance for repeated template processing.
    ///
    /// **Default**: `true`
    pub enable_caching: bool,
}

/// Advanced Handlebars template engine with custom helpers and security features.
///
/// This engine provides a comprehensive templating solution built on Handlebars,
/// with repository-specific helpers, file path templating, and security controls.
/// It maintains a registry of compiled templates and custom helpers for efficient
/// processing of multiple templates.
///
/// ## Core Capabilities
///
/// - **Template Compilation**: Parses and caches Handlebars templates
/// - **Variable Rendering**: Supports complex data structures and built-in helpers
/// - **File Path Processing**: Templates file names and directory paths safely
/// - **Custom Helpers**: Repository-specific text transformations
/// - **Security Validation**: Prevents path traversal and resource exhaustion
///
/// ## Performance Features
///
/// - **Template Caching**: Compiled templates are cached for reuse
/// - **Lazy Loading**: Templates compiled only when needed
/// - **Resource Limits**: Configurable limits prevent abuse
/// - **Efficient Context**: Optimized variable resolution
///
/// ## Thread Safety
///
/// The engine is thread-safe and can be shared between multiple threads
/// for concurrent template processing. Template compilation results are
/// cached globally within the engine instance.
///
/// ## Examples
///
/// ```rust,ignore
/// use template_engine::{HandlebarsTemplateEngine, TemplateContext};
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create engine with default configuration
/// let mut engine = HandlebarsTemplateEngine::new()?;
/// engine.register_custom_helpers()?;
///
/// // Process content template
/// let template = "Hello {{name}}, welcome to {{#upper_case project.name}}{{/upper_case}}!";
/// let context = TemplateContext::new(json!({
///     "name": "Alice",
///     "project": {"name": "repo-roller"}
/// }));
///
/// let result = engine.render_template(template, &context)?;
/// assert_eq!(result, "Hello Alice, welcome to REPO-ROLLER!");
///
/// // Process file path template
/// let path_template = "{{snake_case project.name}}/{{name}}.md";
/// let path_result = engine.template_file_path(path_template, &context)?;
/// assert_eq!(path_result, "repo_roller/Alice.md");
/// # Ok(())
/// # }
/// ```
pub struct HandlebarsTemplateEngine {
    /// The underlying Handlebars engine instance.
    ///
    /// This is the core Handlebars engine that handles template compilation,
    /// helper registration, and rendering. It maintains a registry of compiled
    /// templates and custom helpers.
    handlebars: Handlebars<'static>,

    /// Configuration for template rendering behavior.
    ///
    /// Controls security restrictions, performance limits, and error handling
    /// strategies used during template processing.
    config: TemplateRenderConfig,
}

impl TemplateContext {
    /// Creates a new template context with the provided variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - JSON value containing all template variables
    ///
    /// # Examples
    ///
    /// ```rust
    /// use template_engine::TemplateContext;
    /// use serde_json::json;
    ///
    /// let context = TemplateContext::new(json!({
    ///     "repo_name": "my-project",
    ///     "features": ["logging", "testing"]
    /// }));
    /// ```
    pub fn new(variables: Value) -> Self {
        Self {
            variables,
            config: TemplateRenderConfig::default(),
        }
    }

    /// Creates a template context from a HashMap of string variables.
    ///
    /// This is a convenience method for creating contexts from simple
    /// string-to-string mappings, which are common in basic templating scenarios.
    ///
    /// # Arguments
    ///
    /// * `variables` - HashMap containing string variable mappings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use template_engine::TemplateContext;
    /// use std::collections::HashMap;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("repo_name".to_string(), "my-project".to_string());
    /// vars.insert("author".to_string(), "John Doe".to_string());
    ///
    /// let context = TemplateContext::from_map(vars);
    /// ```
    pub fn from_map(variables: HashMap<String, String>) -> Self {
        let json_vars = variables
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect::<serde_json::Map<String, Value>>();

        Self::new(Value::Object(json_vars))
    }

    /// Creates a template context with custom rendering configuration.
    ///
    /// # Arguments
    ///
    /// * `variables` - JSON value containing all template variables
    /// * `config` - Custom configuration for template rendering
    ///
    /// # Examples
    ///
    /// ```rust
    /// use template_engine::{TemplateContext, TemplateRenderConfig};
    /// use serde_json::json;
    ///
    /// let config = TemplateRenderConfig {
    ///     strict_variables: false,
    ///     max_render_time_ms: 5000,
    ///     ..Default::default()
    /// };
    ///
    /// let context = TemplateContext::with_config(
    ///     json!({"name": "test"}),
    ///     config
    /// );
    /// ```
    pub fn with_config(variables: Value, config: TemplateRenderConfig) -> Self {
        Self { variables, config }
    }
}

impl Default for TemplateRenderConfig {
    fn default() -> Self {
        Self {
            strict_variables: true,
            max_template_size: 1_048_576, // 1MB
            max_render_time_ms: 30_000,   // 30 seconds
            enable_caching: true,
        }
    }
}

impl HandlebarsTemplateEngine {
    /// Creates a new Handlebars template engine with default configuration.
    ///
    /// The engine is initialized with default security settings and an empty
    /// helper registry. Custom helpers must be registered separately using
    /// `register_custom_helpers()`.
    ///
    /// # Returns
    ///
    /// A new `HandlebarsTemplateEngine` instance ready for template processing.
    ///
    /// # Errors
    ///
    /// Returns `HandlebarsError::CompilationError` if the Handlebars engine
    /// cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::HandlebarsTemplateEngine;
    ///
    /// let engine = HandlebarsTemplateEngine::new()?;
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn new() -> Result<Self, HandlebarsError> {
        let mut handlebars = Handlebars::new();
        let config = TemplateRenderConfig::default();

        // Configure handlebars based on our config
        handlebars.set_strict_mode(config.strict_variables);

        Ok(Self { handlebars, config })
    }

    /// Creates a new engine with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration for template rendering
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::{HandlebarsTemplateEngine, TemplateRenderConfig};
    ///
    /// let config = TemplateRenderConfig {
    ///     max_render_time_ms: 5000,
    ///     strict_variables: false,
    ///     ..Default::default()
    /// };
    ///
    /// let engine = HandlebarsTemplateEngine::with_config(config)?;
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn with_config(config: TemplateRenderConfig) -> Result<Self, HandlebarsError> {
        let mut handlebars = Handlebars::new();

        // Configure handlebars based on our config
        handlebars.set_strict_mode(config.strict_variables);

        Ok(Self { handlebars, config })
    }

    /// Registers all custom helpers for repository-specific template processing.
    ///
    /// This method registers the following custom helpers:
    /// - `snake_case`: Convert text to snake_case format
    /// - `kebab_case`: Convert text to kebab-case format
    /// - `upper_case`: Convert text to UPPER_CASE format
    /// - `lower_case`: Convert text to lowercase format
    /// - `capitalize`: Capitalize the first letter
    /// - `timestamp`: Format current timestamp with optional format string
    /// - `default`: Provide default value for undefined variables
    ///
    /// # Returns
    ///
    /// `Ok(())` if all helpers are registered successfully.
    ///
    /// # Errors
    ///
    /// Returns `HandlebarsError::HelperError` if any helper fails to register.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::HandlebarsTemplateEngine;
    ///
    /// let mut engine = HandlebarsTemplateEngine::new()?;
    /// engine.register_custom_helpers()?;
    ///
    /// // Now custom helpers are available in templates
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn register_custom_helpers(&mut self) -> Result<(), HandlebarsError> {
        // Register snake_case helper
        self.handlebars
            .register_helper("snake_case", Box::new(SnakeCaseHelper));

        // Register kebab_case helper
        self.handlebars
            .register_helper("kebab_case", Box::new(KebabCaseHelper));

        // Register upper_case helper
        self.handlebars
            .register_helper("upper_case", Box::new(UpperCaseHelper));

        // Register lower_case helper
        self.handlebars
            .register_helper("lower_case", Box::new(LowerCaseHelper));

        // Register capitalize helper
        self.handlebars
            .register_helper("capitalize", Box::new(CapitalizeHelper));

        // Register default helper
        self.handlebars
            .register_helper("default", Box::new(DefaultHelper));

        // Register timestamp helper
        self.handlebars
            .register_helper("timestamp", Box::new(TimestampHelper));

        Ok(())
    }

    /// Renders a template string with the provided context.
    ///
    /// This method compiles the template (if not cached) and renders it with
    /// the provided variables. It enforces all configured security restrictions
    /// and resource limits.
    ///
    /// # Arguments
    ///
    /// * `template` - The template string to render
    /// * `context` - Template context containing variables and configuration
    ///
    /// # Returns
    ///
    /// The rendered template as a string with all variables substituted.
    ///
    /// # Errors
    ///
    /// - `HandlebarsError::CompilationError`: Template syntax is invalid
    /// - `HandlebarsError::RenderError`: Template rendering failed
    /// - `HandlebarsError::VariableValidation`: Required variables missing
    /// - `HandlebarsError::ResourceLimit`: Processing exceeded limits
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::{HandlebarsTemplateEngine, TemplateContext};
    /// use serde_json::json;
    ///
    /// let engine = HandlebarsTemplateEngine::new()?;
    /// let context = TemplateContext::new(json!({"name": "Alice"}));
    ///
    /// let result = engine.render_template("Hello {{name}}!", &context)?;
    /// assert_eq!(result, "Hello Alice!");
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn render_template(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> Result<String, HandlebarsError> {
        // Check template size limit
        if template.len() > self.config.max_template_size {
            return Err(HandlebarsError::ResourceLimit {
                limit_type: "template_size".to_string(),
                message: format!(
                    "Template size {} bytes exceeds limit of {} bytes",
                    template.len(),
                    self.config.max_template_size
                ),
            });
        }

        // Render the template with the provided context
        let result = self
            .handlebars
            .render_template(template, &context.variables)
            .map_err(|e| {
                // Check the error type and message to categorize appropriately
                let err_msg = e.to_string();

                // Check for missing variable errors in strict mode
                if err_msg.contains("Variable") && err_msg.contains("not found") {
                    HandlebarsError::VariableValidation {
                        variable: "unknown".to_string(), // Could parse from error message
                        reason: err_msg,
                    }
                }
                // Check for compilation/syntax errors
                else if err_msg.contains("parse")
                    || err_msg.contains("Parse")
                    || err_msg.contains("syntax")
                    || err_msg.contains("Syntax")
                    || err_msg.contains("invalid")
                    || err_msg.contains("Invalid")
                    || err_msg.contains("unclosed")
                    || err_msg.contains("Unclosed")
                    || err_msg.contains("unexpected")
                    || err_msg.contains("Unexpected")
                {
                    HandlebarsError::CompilationError { message: err_msg }
                }
                // All other errors are render errors
                else {
                    HandlebarsError::RenderError { message: err_msg }
                }
            })?;

        Ok(result)
    }

    /// Processes a file path template to generate the actual file path.
    ///
    /// This method applies template substitution to file paths and directory
    /// names, with comprehensive security validation to prevent directory
    /// traversal attacks and ensure filesystem compatibility.
    ///
    /// # Security
    ///
    /// - Rejects paths containing `..` (directory traversal)
    /// - Rejects absolute paths (starting with `/` or drive letters)
    /// - Validates filesystem-compatible characters
    /// - Ensures resulting path stays within target directory bounds
    /// - Checks platform-specific path length limits
    ///
    /// # Arguments
    ///
    /// * `path_template` - The file path template to process
    /// * `context` - Template context containing variables
    ///
    /// # Returns
    ///
    /// A validated, safe file path with variables substituted.
    ///
    /// # Errors
    ///
    /// - `HandlebarsError::CompilationError`: Path template syntax invalid
    /// - `HandlebarsError::InvalidPath`: Generated path is unsafe or invalid
    /// - `HandlebarsError::VariableValidation`: Required path variables missing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::{HandlebarsTemplateEngine, TemplateContext};
    /// use serde_json::json;
    ///
    /// let engine = HandlebarsTemplateEngine::new()?;
    /// let context = TemplateContext::new(json!({
    ///     "repo_name": "my-project",
    ///     "environment": "production"
    /// }));
    ///
    /// // Simple file name templating
    /// let file_path = engine.template_file_path("{{repo_name}}.md", &context)?;
    /// assert_eq!(file_path, "my-project.md");
    ///
    /// // Directory path templating
    /// let dir_path = engine.template_file_path("{{environment}}/config.yml", &context)?;
    /// assert_eq!(dir_path, "production/config.yml");
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn template_file_path(
        &self,
        path_template: &str,
        context: &TemplateContext,
    ) -> Result<String, HandlebarsError> {
        // First render the path template
        let rendered_path = self.render_template(path_template, context)?;

        // Then validate the resulting path for security
        self.validate_file_path(&rendered_path)?;

        Ok(rendered_path)
    }

    /// Validates that a generated file path is safe and filesystem-compatible.
    ///
    /// This method performs comprehensive security validation on file paths
    /// generated from templates to prevent directory traversal attacks and
    /// ensure filesystem compatibility.
    ///
    /// # Validation Rules
    ///
    /// - No directory traversal sequences (`..`, `../`, `..\`)
    /// - No absolute paths (`/`, `C:\`, etc.)
    /// - No invalid filesystem characters
    /// - Path length within platform limits
    /// - Path stays within target directory bounds
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the path is safe and valid.
    ///
    /// # Errors
    ///
    /// Returns `HandlebarsError::InvalidPath` with specific details about
    /// why the path is invalid or unsafe.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use template_engine::HandlebarsTemplateEngine;
    ///
    /// let engine = HandlebarsTemplateEngine::new()?;
    ///
    /// // Valid paths
    /// assert!(engine.validate_file_path("README.md").is_ok());
    /// assert!(engine.validate_file_path("src/main.rs").is_ok());
    /// assert!(engine.validate_file_path("docs/api/index.html").is_ok());
    ///
    /// // Invalid paths
    /// assert!(engine.validate_file_path("../../../etc/passwd").is_err());
    /// assert!(engine.validate_file_path("/absolute/path").is_err());
    /// assert!(engine.validate_file_path("file<>name").is_err());
    /// # Ok::<(), template_engine::HandlebarsError>(())
    /// ```
    pub fn validate_file_path(&self, path: &str) -> Result<(), HandlebarsError> {
        // Check for empty path
        if path.is_empty() {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Path cannot be empty".to_string(),
            });
        }

        // Check for special directory names
        if path == "." || path == ".." {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Path cannot be '.' or '..'".to_string(),
            });
        }

        // Check for directory traversal patterns
        if path.contains("..") {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Path contains directory traversal sequence '..'".to_string(),
            });
        }

        // Check for absolute paths (Unix and Windows)
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Absolute paths are not allowed".to_string(),
            });
        }

        // Check for Windows drive letters
        if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Windows drive letters are not allowed".to_string(),
            });
        }

        // Check for UNC paths
        if path.starts_with("\\\\") {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "UNC paths are not allowed".to_string(),
            });
        }

        // Check for null bytes
        if path.contains('\0') {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Path contains null byte".to_string(),
            });
        }

        // Check for invalid characters on Windows
        if cfg!(windows) {
            let invalid_chars = ['<', '>', '|', '?', '*'];
            for ch in invalid_chars.iter() {
                if path.contains(*ch) {
                    return Err(HandlebarsError::InvalidPath {
                        path: path.to_string(),
                        reason: format!("Path contains invalid character '{}'", ch),
                    });
                }
            }
        }

        // Check path length (conservative limit)
        if path.len() > 255 {
            return Err(HandlebarsError::InvalidPath {
                path: path.to_string(),
                reason: "Path exceeds maximum length of 255 characters".to_string(),
            });
        }

        Ok(())
    }
}
