# REST API Error Handling

**Module**: HTTP API Error Handling
**Location**: `repo_roller_api/src/errors.rs`
**Purpose**: Define error response patterns and domain-to-HTTP error mapping

---

## Overview

This specification defines how domain errors are translated to HTTP error responses in the RepoRoller REST API. It establishes consistent error response structures, HTTP status code mapping, and security guidelines for error details.

**Architectural Layer**: HTTP Interface (Infrastructure)
**Crate**: `repo_roller_api`

**Key Principles**:

- Domain errors must never leak to HTTP responses directly
- HTTP layer translates domain errors to appropriate HTTP statuses
- Error responses provide actionable information without exposing internals
- Security-sensitive information never included in error responses

---

## Error Response Structure

### Standard Error Response

All error responses use this consistent structure:

```rust
/// Standard error response for all API errors.
///
/// See: specs/interfaces/api-error-handling.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetails {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional context (optional, type varies by error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

**Example**:

```json
{
  "error": {
    "code": "ValidationError",
    "message": "Repository name contains invalid characters",
    "details": {
      "field": "name",
      "value": "My@Repo",
      "constraint": "alphanumeric with hyphens only"
    }
  }
}
```

---

## Domain Error to HTTP Status Mapping

### Mapping Table

| Domain Error | HTTP Status | Use Case | Example Message |
|--------------|-------------|----------|-----------------|
| `ValidationError` | 400 Bad Request | Invalid input format/values | "Repository name contains invalid characters" |
| `RepositoryError::AlreadyExists` | 409 Conflict | Repository name taken | "Repository 'myorg/my-repo' already exists" |
| `RepositoryError::NotFound` | 404 Not Found | Repository doesn't exist | "Repository 'myorg/my-repo' not found" |
| `RepositoryError::CreationFailed` | 500 Internal Server Error | GitHub creation failed | "Failed to create repository" |
| `RepositoryError::PushFailed` | 502 Bad Gateway | Push to GitHub failed | "Failed to push content to repository" |
| `RepositoryError::SettingsApplicationFailed` | 502 Bad Gateway | Settings application failed | "Failed to apply repository settings" |
| `RepositoryError::OperationTimeout` | 504 Gateway Timeout | Operation took too long | "Repository creation timed out" |
| `ConfigurationError::FileNotFound` | 404 Not Found | Configuration file missing | "Configuration file not found" |
| `ConfigurationError::ParseError` | 400 Bad Request | Invalid configuration syntax | "Failed to parse configuration" |
| `ConfigurationError::InvalidConfiguration` | 400 Bad Request | Invalid configuration values | "Invalid configuration: visibility must be 'public' or 'private'" |
| `ConfigurationError::OverrideNotPermitted` | 403 Forbidden | Override policy violation | "Configuration override not permitted" |
| `ConfigurationError::RequiredConfigMissing` | 500 Internal Server Error | Missing required config | "Required configuration missing" |
| `ConfigurationError::MetadataRepositoryNotFound` | 404 Not Found | Metadata repo missing | "Metadata repository not found for organization" |
| `TemplateError::TemplateNotFound` | 404 Not Found | Template doesn't exist | "Template 'rust-library' not found" |
| `TemplateError::FetchFailed` | 502 Bad Gateway | Template fetch failed | "Failed to fetch template" |
| `TemplateError::SyntaxError` | 400 Bad Request | Template syntax error | "Template syntax error in template.toml" |
| `TemplateError::SubstitutionFailed` | 500 Internal Server Error | Variable substitution failed | "Variable substitution failed" |
| `TemplateError::RequiredVariableMissing` | 400 Bad Request | Missing required variable | "Required template variable missing: project_name" |
| `TemplateError::ProcessingTimeout` | 504 Gateway Timeout | Template processing timeout | "Template processing timed out" |
| `TemplateError::SecurityViolation` | 400 Bad Request | Security constraint violated | "Security violation: path traversal attempt" |
| `TemplateError::PathTraversalAttempt` | 400 Bad Request | Path traversal detected | "Path traversal attempt detected" |
| `AuthenticationError::InvalidToken` | 401 Unauthorized | Token invalid/expired | "Invalid or expired authentication token" |
| `AuthenticationError::AuthenticationFailed` | 401 Unauthorized | Authentication failed | "Authentication failed" |
| `AuthenticationError::InsufficientPermissions` | 403 Forbidden | Missing permissions | "Insufficient permissions to create repository" |
| `AuthenticationError::UserNotFound` | 404 Not Found | User doesn't exist | "User not found" |
| `AuthenticationError::OrganizationAccessDenied` | 403 Forbidden | No org access | "Access denied to organization 'myorg'" |
| `AuthenticationError::SessionExpired` | 401 Unauthorized | Session expired | "Session expired, please log in again" |
| `AuthenticationError::TokenRefreshFailed` | 401 Unauthorized | Token refresh failed | "Failed to refresh authentication token" |
| `GitHubError::ApiRequestFailed` | 502 Bad Gateway | GitHub API error | "GitHub API request failed" |
| `GitHubError::RateLimitExceeded` | 429 Too Many Requests | Rate limit hit | "GitHub rate limit exceeded" |
| `GitHubError::ResourceNotFound` | 404 Not Found | GitHub resource missing | "GitHub resource not found" |
| `GitHubError::AuthenticationFailed` | 401 Unauthorized | GitHub auth failed | "GitHub authentication failed" |
| `GitHubError::NetworkError` | 502 Bad Gateway | Network issue | "Network error communicating with GitHub" |
| `GitHubError::InvalidResponse` | 502 Bad Gateway | Invalid GitHub response | "Invalid response from GitHub API" |
| `GitHubError::AppNotInstalled` | 403 Forbidden | GitHub App not installed | "GitHub App not installed on organization" |
| `SystemError::FileSystem` | 500 Internal Server Error | File system error | "File system error occurred" |
| `SystemError::GitOperation` | 500 Internal Server Error | Git operation failed | "Git operation failed" |
| `SystemError::Network` | 500 Internal Server Error | Network error | "Network error occurred" |
| `SystemError::Serialization` | 500 Internal Server Error | Serialization failed | "Serialization error" |
| `SystemError::Deserialization` | 400 Bad Request | Deserialization failed | "Invalid JSON format" |
| `SystemError::Internal` | 500 Internal Server Error | Unexpected internal error | "An internal error occurred" |
| `SystemError::ResourceUnavailable` | 503 Service Unavailable | Resource unavailable | "Required resource is unavailable" |

---

## Error Conversion Implementation

### Conversion Trait

```rust
use axum::http::StatusCode;
use repo_roller_core::errors::*;

/// Convert domain errors to HTTP responses.
pub trait IntoHttpError {
    fn into_http_error(self) -> (StatusCode, ErrorResponse);
}

impl IntoHttpError for RepoRollerError {
    fn into_http_error(self) -> (StatusCode, ErrorResponse) {
        match self {
            RepoRollerError::Validation(e) => e.into_http_error(),
            RepoRollerError::Repository(e) => e.into_http_error(),
            RepoRollerError::Configuration(e) => e.into_http_error(),
            RepoRollerError::Template(e) => e.into_http_error(),
            RepoRollerError::Authentication(e) => e.into_http_error(),
            RepoRollerError::GitHub(e) => e.into_http_error(),
            RepoRollerError::System(e) => e.into_http_error(),
        }
    }
}
```

### ValidationError Conversion

```rust
impl IntoHttpError for ValidationError {
    fn into_http_error(self) -> (StatusCode, ErrorResponse) {
        let (code, message, details) = match self {
            ValidationError::InvalidRepositoryName { reason } => (
                "ValidationError",
                format!("Invalid repository name: {}", reason),
                serde_json::json!({
                    "field": "name",
                    "reason": reason
                }),
            ),
            ValidationError::InvalidOrganizationName { reason } => (
                "ValidationError",
                format!("Invalid organization name: {}", reason),
                serde_json::json!({
                    "field": "organization",
                    "reason": reason
                }),
            ),
            ValidationError::PatternMismatch { field, pattern, value } => (
                "ValidationError",
                format!("Field '{}' does not match required pattern", field),
                serde_json::json!({
                    "field": field,
                    "pattern": pattern,
                    "value": value
                }),
            ),
            // ... other variants
        };

        (
            StatusCode::BAD_REQUEST,
            ErrorResponse {
                error: ErrorDetails {
                    code: code.to_string(),
                    message,
                    details: Some(details),
                },
            },
        )
    }
}
```

### AuthenticationError Conversion

```rust
impl IntoHttpError for AuthenticationError {
    fn into_http_error(self) -> (StatusCode, ErrorResponse) {
        let (status, code, message, details) = match self {
            AuthenticationError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationError",
                "Invalid or expired authentication token".to_string(),
                Some(serde_json::json!({
                    "header": "Authorization",
                    "scheme": "Bearer"
                })),
            ),
            AuthenticationError::InsufficientPermissions { operation, required } => (
                StatusCode::FORBIDDEN,
                "AuthenticationError",
                format!("Insufficient permissions: {} permission required", required),
                Some(serde_json::json!({
                    "operation": operation,
                    "required_permission": required
                })),
            ),
            AuthenticationError::OrganizationAccessDenied { org } => (
                StatusCode::FORBIDDEN,
                "AuthenticationError",
                format!("Access denied to organization '{}'", org),
                Some(serde_json::json!({
                    "organization": org
                })),
            ),
            // ... other variants
        };

        (
            status,
            ErrorResponse {
                error: ErrorDetails {
                    code: code.to_string(),
                    message,
                    details,
                },
            },
        )
    }
}
```

### GitHubError Conversion

```rust
impl IntoHttpError for GitHubError {
    fn into_http_error(self) -> (StatusCode, ErrorResponse) {
        let (status, code, message, details) = match self {
            GitHubError::RateLimitExceeded { reset_at } => (
                StatusCode::TOO_MANY_REQUESTS,
                "GitHubRateLimitExceeded",
                "GitHub API rate limit exceeded".to_string(),
                Some(serde_json::json!({
                    "reset_at": reset_at
                })),
            ),
            GitHubError::ApiRequestFailed { status: gh_status, message: gh_message } => (
                StatusCode::BAD_GATEWAY,
                "GitHubApiError",
                format!("GitHub API request failed: {}", gh_message),
                Some(serde_json::json!({
                    "github_status": gh_status
                })),
            ),
            GitHubError::AppNotInstalled { org } => (
                StatusCode::FORBIDDEN,
                "GitHubAppNotInstalled",
                format!("GitHub App not installed on organization '{}'", org),
                Some(serde_json::json!({
                    "organization": org
                })),
            ),
            // ... other variants
        };

        (
            status,
            ErrorResponse {
                error: ErrorDetails {
                    code: code.to_string(),
                    message,
                    details,
                },
            },
        )
    }
}
```

---

## Security Guidelines

### Information Disclosure

**Never include in error responses**:

- Authentication tokens or secrets
- Internal file paths (sanitize to relative paths)
- Stack traces (except development mode)
- Database query details
- Internal service URLs or IPs
- User IDs or internal identifiers (use public identifiers only)

**Safe to include**:

- Validation error details (field names, constraints)
- Public resource identifiers (repository names, organization names)
- GitHub API error messages (already public)
- Operation names and parameters (sanitized)

### Example Unsafe vs Safe

❌ **Unsafe** (leaks internal details):

```json
{
  "error": {
    "code": "SystemError",
    "message": "Failed to load configuration",
    "details": {
      "file_path": "C:\\Users\\admin\\secrets\\config.toml",
      "token": "ghp_1234567890abcdef",
      "stack_trace": "..."
    }
  }
}
```

✅ **Safe** (provides useful info without leaks):

```json
{
  "error": {
    "code": "ConfigurationError",
    "message": "Failed to load organization configuration",
    "details": {
      "organization": "myorg",
      "configuration_file": "config.toml"
    }
  }
}
```

### Development vs Production Error Details

**Development Mode**:

```rust
#[cfg(debug_assertions)]
fn add_debug_details(details: &mut serde_json::Value, error: &dyn Error) {
    if let Some(obj) = details.as_object_mut() {
        obj.insert("debug_info".to_string(), serde_json::json!({
            "error_chain": format_error_chain(error),
            "backtrace": format_backtrace(),
        }));
    }
}
```

**Production Mode**: Never include debug information.

---

## Authentication Error Patterns

### Missing Token

**Request**:

```
GET /api/v1/orgs/myorg/templates
(No Authorization header)
```

**Response** (401):

```json
{
  "error": {
    "code": "AuthenticationError",
    "message": "Authentication required",
    "details": {
      "header": "Authorization",
      "scheme": "Bearer"
    }
  }
}
```

### Invalid Token

**Request**:

```
GET /api/v1/orgs/myorg/templates
Authorization: Bearer invalid_token
```

**Response** (401):

```json
{
  "error": {
    "code": "AuthenticationError",
    "message": "Invalid or expired authentication token",
    "details": {
      "header": "Authorization"
    }
  }
}
```

### Insufficient Permissions

**Request**:

```
POST /api/v1/repositories
Authorization: Bearer valid_token_but_no_write_permission
```

**Response** (403):

```json
{
  "error": {
    "code": "AuthenticationError",
    "message": "Insufficient permissions: repository_write permission required",
    "details": {
      "operation": "create_repository",
      "required_permission": "repository_write"
    }
  }
}
```

---

## Validation Error Patterns

### Multiple Validation Errors

When multiple validation errors occur, include all of them:

```json
{
  "error": {
    "code": "ValidationError",
    "message": "Request validation failed",
    "details": {
      "errors": [
        {
          "field": "name",
          "message": "Repository name must be lowercase",
          "constraint": "lowercase-only"
        },
        {
          "field": "variables.project_name",
          "message": "Required variable is missing",
          "constraint": "required"
        }
      ]
    }
  }
}
```

### Field-Specific Validation

```json
{
  "error": {
    "code": "ValidationError",
    "message": "Field 'visibility' must be one of ['public', 'private'], got: 'internal'",
    "details": {
      "field": "visibility",
      "value": "internal",
      "allowed_values": ["public", "private"]
    }
  }
}
```

---

## Rate Limiting Error Pattern

When GitHub rate limit is exceeded:

**Response** (429):

```json
{
  "error": {
    "code": "GitHubRateLimitExceeded",
    "message": "GitHub API rate limit exceeded",
    "details": {
      "reset_at": "2025-11-12T11:00:00Z",
      "retry_after_seconds": 1800
    }
  }
}
```

**Response Headers**:

```
X-RateLimit-Limit: 5000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1731412800
Retry-After: 1800
```

---

## Timeout Error Pattern

When operations timeout:

**Response** (504):

```json
{
  "error": {
    "code": "OperationTimeout",
    "message": "Repository creation timed out after 120 seconds",
    "details": {
      "operation": "create_repository",
      "timeout_seconds": 120
    }
  }
}
```

---

## Not Found Error Patterns

### Resource Not Found

```json
{
  "error": {
    "code": "TemplateError",
    "message": "Template 'nonexistent-template' not found",
    "details": {
      "resource_type": "template",
      "organization": "myorg",
      "template": "nonexistent-template"
    }
  }
}
```

### Configuration Not Found

```json
{
  "error": {
    "code": "ConfigurationError",
    "message": "Metadata repository not found for organization 'myorg'",
    "details": {
      "organization": "myorg",
      "expected_repository": ".reporoller",
      "discovery_method": "convention"
    }
  }
}
```

---

## Internal Server Error Pattern

For unexpected errors:

**Response** (500):

```json
{
  "error": {
    "code": "InternalError",
    "message": "An internal error occurred",
    "details": {
      "request_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**Note**: Always log full error details server-side but only return generic message to client.

---

## Axum Integration

### Error Handler Middleware

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// Axum response wrapper for API errors
pub struct ApiError(RepoRollerError);

impl From<RepoRollerError> for ApiError {
    fn from(err: RepoRollerError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = self.0.into_http_error();

        // Log error server-side
        log_error(&self.0, status);

        (status, Json(error_response)).into_response()
    }
}

fn log_error(error: &RepoRollerError, status: StatusCode) {
    match status {
        StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_GATEWAY => {
            tracing::error!("API error: {} - {}", status, error);
        }
        StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
            tracing::warn!("API error: {} - {}", status, error);
        }
        _ => {
            tracing::info!("API error: {} - {}", status, error);
        }
    }
}
```

### Handler Example

```rust
async fn create_repository(
    State(app_state): State<AppState>,
    Json(request): Json<CreateRepositoryRequest>,
) -> Result<Json<CreateRepositoryResponse>, ApiError> {
    // Business logic call returns Result<T, RepoRollerError>
    let result = app_state
        .repository_service
        .create_repository(request.try_into()?)
        .await?;  // ? automatically converts to ApiError

    Ok(Json(result.into()))
}
```

---

## Testing Requirements

### Error Conversion Tests

Test each domain error type converts to correct HTTP status:

```rust
#[test]
fn test_validation_error_conversion() {
    let error = ValidationError::InvalidRepositoryName {
        reason: "contains special characters".to_string(),
    };

    let (status, response) = RepoRollerError::Validation(error).into_http_error();

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response.error.code, "ValidationError");
}
```

### Security Tests

Verify no sensitive data leaks:

```rust
#[test]
fn test_error_response_no_token_leak() {
    let error = AuthenticationError::InvalidToken;
    let (_, response) = RepoRollerError::Authentication(error).into_http_error();

    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("ghp_"));  // No token in response
    assert!(!json.contains("Bearer"));  // No auth header values
}
```

---

## Dependencies

```toml
[dependencies]
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
repo_roller_core = { path = "../repo_roller_core" }
```

---

## Related Specifications

- **specs/interfaces/api-request-types.md** - HTTP request models
- **specs/interfaces/api-response-types.md** - HTTP response models
- **specs/interfaces/error-types.md** - Domain error types
- **specs/constraints.md** - Security constraints

---

**Status**: Interface Specification
**Next Steps**: Implement error handling in `repo_roller_api/src/errors.rs`
