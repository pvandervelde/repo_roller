use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

/// DEPRECATED: Internal-only error type for lib.rs orchestration code.
///
/// This type exists temporarily to support the legacy orchestration code in lib.rs.
/// It will be removed when lib.rs is refactored (Tasks 1.8.10 and 2.0+).
///
/// DO NOT USE in new code. Use the domain-specific error types instead:
/// - ValidationError for input validation
/// - RepositoryError for repository operations
/// - SystemError for infrastructure failures
/// - etc.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Git operation failed: {0}")]
    GitOperation(String),

    #[error("File system operation failed: {0}")]
    FileSystem(String),

    #[error("Template processing failed: {0}")]
    TemplateProcessing(String),
}

/// Validation errors for user input and business rules.
///
/// These errors indicate that user-provided data doesn't meet
/// the system's requirements and suggest how to fix the issue.
///
/// See specs/interfaces/error-types.md#validationerror for complete specification
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Field '{field}' cannot be empty")]
    EmptyField { field: String },

    #[error("Field '{field}' is too long: {actual} characters (max: {max})")]
    TooLong {
        field: String,
        actual: usize,
        max: usize,
    },

    #[error("Field '{field}' is too short: {actual} characters (min: {min})")]
    TooShort {
        field: String,
        actual: usize,
        min: usize,
    },

    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Invalid repository name: {reason}")]
    InvalidRepositoryName { reason: String },

    #[error("Invalid organization name: {reason}")]
    InvalidOrganizationName { reason: String },

    #[error("Invalid template name: {reason}")]
    InvalidTemplateName { reason: String },

    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Field '{field}' must match pattern '{pattern}', got: '{value}'")]
    PatternMismatch {
        field: String,
        pattern: String,
        value: String,
    },

    #[error("Field '{field}' length must be between {min} and {max}, got {actual}")]
    LengthConstraint {
        field: String,
        min: usize,
        max: usize,
        actual: usize,
    },

    #[error("Field '{field}' must be one of {options:?}, got: '{value}'")]
    InvalidOption {
        field: String,
        options: Vec<String>,
        value: String,
    },
}

impl ValidationError {
    pub fn empty_field(field: impl Into<String>) -> Self {
        Self::EmptyField {
            field: field.into(),
        }
    }

    pub fn too_long(field: impl Into<String>, actual: usize, max: usize) -> Self {
        Self::TooLong {
            field: field.into(),
            actual,
            max,
        }
    }

    pub fn too_short(field: impl Into<String>, actual: usize, min: usize) -> Self {
        Self::TooShort {
            field: field.into(),
            actual,
            min,
        }
    }

    pub fn invalid_format(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

/// Repository operation errors.
///
/// These errors occur during repository creation, configuration,
/// and management operations.
///
/// See specs/interfaces/error-types.md#repositoryerror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RepositoryError {
    #[error("Repository '{name}' already exists in organization '{org}'")]
    AlreadyExists { org: String, name: String },

    #[error("Repository not found: {org}/{name}")]
    NotFound { org: String, name: String },

    #[error("Failed to create repository: {reason}")]
    CreationFailed { reason: String },

    #[error("Failed to push content: {reason}")]
    PushFailed { reason: String },

    #[error("Failed to apply settings: {setting} - {reason}")]
    SettingsApplicationFailed { setting: String, reason: String },

    #[error("Repository operation timeout after {timeout_secs} seconds")]
    OperationTimeout { timeout_secs: u64 },
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type RepositoryResult<T> = Result<T, RepositoryError>;

// Re-export ConfigurationError from config_manager (proper architectural layering)
pub use config_manager::{ConfigurationError, ConfigurationResult};

/// Template processing errors.
///
/// These errors occur during template fetching, processing,
/// and variable substitution operations.
///
/// See specs/interfaces/error-types.md#templateerror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TemplateError {
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("Failed to fetch template: {reason}")]
    FetchFailed { reason: String },

    #[error("Template syntax error in {file}: {reason}")]
    SyntaxError { file: String, reason: String },

    #[error("Variable substitution failed for '{variable}': {reason}")]
    SubstitutionFailed { variable: String, reason: String },

    #[error("Required template variable missing: {variable}")]
    RequiredVariableMissing { variable: String },

    #[error("Template processing timeout after {timeout_secs} seconds")]
    ProcessingTimeout { timeout_secs: u64 },

    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Path traversal attempt detected: {path}")]
    PathTraversalAttempt { path: String },
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Authentication and authorization errors.
///
/// These errors occur during user authentication, token validation,
/// and permission checks.
///
/// See specs/interfaces/error-types.md#authenticationerror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AuthenticationError {
    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Insufficient permissions: {required} permission required for {operation}")]
    InsufficientPermissions { operation: String, required: String },

    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },

    #[error("Organization access denied: {org}")]
    OrganizationAccessDenied { org: String },

    #[error("Session expired")]
    SessionExpired,

    #[error("Token refresh failed: {reason}")]
    TokenRefreshFailed { reason: String },
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type AuthenticationResult<T> = Result<T, AuthenticationError>;

/// GitHub API interaction errors.
///
/// These errors occur when communicating with GitHub's REST API.
/// They wrap underlying HTTP and API-specific errors.
///
/// See specs/interfaces/error-types.md#githuberror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug, Clone, PartialEq)]
pub enum GitHubError {
    #[error("GitHub API request failed: {status} - {message}")]
    ApiRequestFailed { status: u16, message: String },

    #[error("GitHub API rate limit exceeded, resets at {reset_at}")]
    RateLimitExceeded { reset_at: String },

    #[error("GitHub resource not found: {resource}")]
    ResourceNotFound { resource: String },

    #[error("GitHub authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Network error communicating with GitHub: {reason}")]
    NetworkError { reason: String },

    #[error("Invalid GitHub API response: {reason}")]
    InvalidResponse { reason: String },

    #[error("GitHub App not installed on organization: {org}")]
    AppNotInstalled { org: String },
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type GitHubResult<T> = Result<T, GitHubError>;

/// System and infrastructure errors.
///
/// These errors represent unexpected system failures, resource
/// issues, and other infrastructure problems.
///
/// See specs/interfaces/error-types.md#systemerror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SystemError {
    #[error("File system error: {operation} - {reason}")]
    FileSystem { operation: String, reason: String },

    #[error("Git operation failed: {operation} - {reason}")]
    GitOperation { operation: String, reason: String },

    #[error("Network error: {reason}")]
    Network { reason: String },

    #[error("Serialization error: {reason}")]
    Serialization { reason: String },

    #[error("Deserialization error: {reason}")]
    Deserialization { reason: String },

    #[error("Internal error: {reason}")]
    Internal { reason: String },

    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type SystemResult<T> = Result<T, SystemError>;

/// Top-level error type for all RepoRoller operations.
///
/// This enum categorizes errors by domain and provides context
/// for error handling and user-facing error messages.
///
/// See specs/interfaces/error-types.md#reporollererror
#[allow(dead_code)] // Will be used in Task 1.8.3+
#[derive(Error, Debug)]
pub enum RepoRollerError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("Template processing error: {0}")]
    Template(#[from] TemplateError),

    #[error("Authentication error: {0}")]
    Authentication(#[from] AuthenticationError),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),

    #[error("System error: {0}")]
    System(#[from] SystemError),
}

#[allow(dead_code)] // Will be used in Task 1.8.3+
pub type RepoRollerResult<T> = Result<T, RepoRollerError>;
