# Error Types Specification

**Module**: Error hierarchy and error handling
**Location**: `repo_roller_core/src/errors.rs`
**Purpose**: Define comprehensive error types and handling patterns

## Overview

This specification defines RepoRoller's error hierarchy using a structured approach with domain-specific error types. All errors implement `std::error::Error` and use `thiserror` for ergonomic error definition.

## Architectural Layer

**Layer**: Core Domain (Cross-cutting)
**Dependencies**: `thiserror`, `std::error::Error`
**Used By**: All modules for error reporting

## Error Hierarchy

```
RepoRollerError (top-level enum)
├── Validation(ValidationError)
├── Repository(RepositoryError)
├── Configuration(ConfigurationError)
├── Template(TemplateError)
├── Authentication(AuthenticationError)
├── GitHub(GitHubError)
└── System(SystemError)
```

## Top-Level Error Type

### RepoRollerError

The top-level error enum that encompasses all error categories.

```rust
/// Top-level error type for all RepoRoller operations.
///
/// This enum categorizes errors by domain and provides context
/// for error handling and user-facing error messages.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
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

pub type RepoRollerResult<T> = Result<T, RepoRollerError>;
```

## Domain Error Types

### ValidationError

Errors related to input validation and business rule violations.

```rust
/// Validation errors for user input and business rules.
///
/// These errors indicate that user-provided data doesn't meet
/// the system's requirements and suggest how to fix the issue.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
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
```

### RepositoryError

Errors specific to repository creation and management operations.

```rust
/// Repository operation errors.
///
/// These errors occur during repository creation, configuration,
/// and management operations.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Repository '{name}' already exists in organization '{org}'")]
    AlreadyExists {
        org: String,
        name: String,
    },

    #[error("Repository not found: {org}/{name}")]
    NotFound {
        org: String,
        name: String,
    },

    #[error("Failed to create repository: {reason}")]
    CreationFailed { reason: String },

    #[error("Failed to push content: {reason}")]
    PushFailed { reason: String },

    #[error("Failed to apply settings: {setting} - {reason}")]
    SettingsApplicationFailed {
        setting: String,
        reason: String,
    },

    #[error("Repository operation timeout after {timeout_secs} seconds")]
    OperationTimeout { timeout_secs: u64 },
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
```

### ConfigurationError

Errors related to configuration loading, parsing, and validation.

```rust
/// Configuration system errors.
///
/// These errors occur when loading, parsing, or validating
/// configuration from various sources.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to parse configuration: {reason}")]
    ParseError { reason: String },

    #[error("Invalid configuration: {field} - {reason}")]
    InvalidConfiguration {
        field: String,
        reason: String,
    },

    #[error("Configuration override not permitted: {setting} - {reason}")]
    OverrideNotPermitted {
        setting: String,
        reason: String,
    },

    #[error("Required configuration missing: {key}")]
    RequiredConfigMissing { key: String },

    #[error("Configuration hierarchy resolution failed: {reason}")]
    HierarchyResolutionFailed { reason: String },

    #[error("Metadata repository not found for organization: {org}")]
    MetadataRepositoryNotFound { org: String },
}

pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
```

### TemplateError

Errors during template processing and variable substitution.

```rust
/// Template processing errors.
///
/// These errors occur during template fetching, processing,
/// and variable substitution operations.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("Failed to fetch template: {reason}")]
    FetchFailed { reason: String },

    #[error("Template syntax error in {file}: {reason}")]
    SyntaxError {
        file: String,
        reason: String,
    },

    #[error("Variable substitution failed for '{variable}': {reason}")]
    SubstitutionFailed {
        variable: String,
        reason: String,
    },

    #[error("Required template variable missing: {variable}")]
    RequiredVariableMissing { variable: String },

    #[error("Template processing timeout after {timeout_secs} seconds")]
    ProcessingTimeout { timeout_secs: u64 },

    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Path traversal attempt detected: {path}")]
    PathTraversalAttempt { path: String },
}

pub type TemplateResult<T> = Result<T, TemplateError>;
```

### AuthenticationError

Errors related to authentication and authorization.

```rust
/// Authentication and authorization errors.
///
/// These errors occur during user authentication, token validation,
/// and permission checks.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Insufficient permissions: {required} permission required for {operation}")]
    InsufficientPermissions {
        operation: String,
        required: String,
    },

    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },

    #[error("Organization access denied: {org}")]
    OrganizationAccessDenied { org: String },

    #[error("Session expired")]
    SessionExpired,

    #[error("Token refresh failed: {reason}")]
    TokenRefreshFailed { reason: String },
}

pub type AuthenticationResult<T> = Result<T, AuthenticationError>;
```

### GitHubError

Errors from GitHub API interactions.

```rust
/// GitHub API interaction errors.
///
/// These errors occur when communicating with GitHub's REST API.
/// They wrap underlying HTTP and API-specific errors.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("GitHub API request failed: {status} - {message}")]
    ApiRequestFailed {
        status: u16,
        message: String,
    },

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

pub type GitHubResult<T> = Result<T, GitHubError>;
```

### SystemError

System-level and infrastructure errors.

```rust
/// System and infrastructure errors.
///
/// These errors represent unexpected system failures, resource
/// issues, and other infrastructure problems.
///
/// See spec: specs/interfaces/error-types.md
#[derive(Debug, thiserror::Error)]
pub enum SystemError {
    #[error("File system error: {operation} - {reason}")]
    FileSystem {
        operation: String,
        reason: String,
    },

    #[error("Git operation failed: {operation} - {reason}")]
    GitOperation {
        operation: String,
        reason: String,
    },

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

pub type SystemResult<T> = Result<T, SystemError>;
```

## Error Context and Conversion

### Adding Context to Errors

```rust
use thiserror::Error;

// Helper for adding context to errors
pub trait ErrorContext<T> {
    fn context(self, context: impl Into<String>) -> Result<T, RepoRollerError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<RepoRollerError>,
{
    fn context(self, context: impl Into<String>) -> Result<T, RepoRollerError> {
        self.map_err(|e| {
            // Preserve original error while adding context
            // Implementation details would wrap error with context
            e.into()
        })
    }
}
```

### External Error Conversions

```rust
// Example conversions from external crate errors

impl From<std::io::Error> for SystemError {
    fn from(err: std::io::Error) -> Self {
        SystemError::FileSystem {
            operation: "I/O operation".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for SystemError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() {
            SystemError::Deserialization {
                reason: err.to_string(),
            }
        } else {
            SystemError::Serialization {
                reason: err.to_string(),
            }
        }
    }
}

// Git errors
impl From<git2::Error> for SystemError {
    fn from(err: git2::Error) -> Self {
        SystemError::GitOperation {
            operation: "git operation".to_string(),
            reason: err.to_string(),
        }
    }
}
```

## Usage Examples

### Basic Error Handling

```rust
use repo_roller_core::errors::*;

async fn create_repository(
    org: OrganizationName,
    name: RepositoryName,
) -> RepositoryResult<Repository> {
    // Validation might fail
    validate_repository_name(&name)?;

    // Repository creation might fail
    let repo = github_client
        .create_repository(org, name)
        .await
        .map_err(|e| RepositoryError::CreationFailed {
            reason: e.to_string(),
        })?;

    Ok(repo)
}
```

### Error Conversion and Propagation

```rust
async fn process_template(template_name: &str) -> RepoRollerResult<ProcessedTemplate> {
    // Template errors automatically convert to RepoRollerError
    let template = fetch_template(template_name).await?;

    // Configuration errors also convert automatically
    let config = load_configuration().await?;

    // Combine operations, errors propagate with context
    let processed = apply_variables(&template, &config)?;

    Ok(processed)
}
```

### User-Facing Error Messages

```rust
// Convert errors to user-friendly messages
fn format_error_for_user(err: &RepoRollerError) -> String {
    match err {
        RepoRollerError::Validation(e) => {
            format!("Invalid input: {}. Please check your request and try again.", e)
        }
        RepoRollerError::Repository(RepositoryError::AlreadyExists { org, name }) => {
            format!(
                "Repository '{}/{}' already exists. Please choose a different name.",
                org, name
            )
        }
        RepoRollerError::Authentication(AuthenticationError::InvalidToken) => {
            "Your authentication token is invalid or expired. Please log in again.".to_string()
        }
        RepoRollerError::GitHub(GitHubError::RateLimitExceeded { reset_at }) => {
            format!(
                "GitHub rate limit exceeded. Please try again after {}.",
                reset_at
            )
        }
        _ => format!("An error occurred: {}", err),
    }
}
```

## Implementation Requirements

### Error Logging

All errors should be logged with appropriate context:

```rust
use tracing::{error, warn, info};

match operation().await {
    Ok(result) => {
        info!("Operation succeeded");
        result
    }
    Err(e) => {
        match &e {
            RepoRollerError::System(_) => error!("System error: {}", e),
            RepoRollerError::GitHub(_) => warn!("GitHub error: {}", e),
            _ => info!("Operation error: {}", e),
        }
        return Err(e);
    }
}
```

### Security Considerations

Never include sensitive data in error messages:

```rust
// ❌ DON'T: Expose tokens or secrets
Err(AuthenticationError::AuthenticationFailed {
    reason: format!("Invalid token: {}", token),
})

// ✅ DO: Keep sensitive data out of errors
Err(AuthenticationError::InvalidToken)
```

### Testing Error Conditions

Every error variant should have test coverage:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_already_exists() {
        let err = RepositoryError::AlreadyExists {
            org: "my-org".to_string(),
            name: "my-repo".to_string(),
        };

        assert_eq!(
            err.to_string(),
            "Repository 'my-repo' already exists in organization 'my-org'"
        );
    }

    #[tokio::test]
    async fn test_error_conversion() {
        let result: RepositoryResult<()> = Err(RepositoryError::NotFound {
            org: "test".to_string(),
            name: "test-repo".to_string(),
        });

        let repo_roller_result: RepoRollerResult<()> = result.map_err(Into::into);
        assert!(matches!(
            repo_roller_result,
            Err(RepoRollerError::Repository(_))
        ));
    }
}
```

## Dependencies

```toml
[dependencies]
thiserror = "1.0"
tracing = "0.1"
```

## Migration from Existing Errors

Current code uses various error types. Migration path:

```rust
// Old error type
pub enum Error {
    GitOperation(String),
    FileSystem(String),
    // ...
}

// Migration: Convert to new error types
impl From<Error> for RepoRollerError {
    fn from(old_err: Error) -> Self {
        match old_err {
            Error::GitOperation(msg) => {
                RepoRollerError::System(SystemError::GitOperation {
                    operation: "legacy operation".to_string(),
                    reason: msg,
                })
            }
            Error::FileSystem(msg) => {
                RepoRollerError::System(SystemError::FileSystem {
                    operation: "legacy operation".to_string(),
                    reason: msg,
                })
            }
            // ... other conversions
        }
    }
}
```

---

**Status**: Interface Specification
**Next Steps**: Implement error types in `repo_roller_core/src/errors.rs`
**See Also**: [Shared Types](shared-types.md), [Repository Domain](repository-domain.md)
