# REST API Response Types

**Module**: HTTP API Response Models
**Location**: `repo_roller_api/src/models/response.rs`
**Purpose**: Define HTTP response types for REST API endpoints

---

## Overview

This specification defines all HTTP response models for the RepoRoller REST API. These types represent the data returned to HTTP clients and are **distinct from domain result types**.

**Architectural Layer**: HTTP Interface (Infrastructure)
**Crate**: `repo_roller_api`

**Key Principles**:

- Response types include HTTP-specific metadata (timestamps, URLs, etc.)
- Domain result types are translated to HTTP responses at API boundary
- Responses are optimized for client consumption (web UI, CLI tools)
- All responses use consistent JSON structure

---

## Repository Management Responses

### CreateRepositoryResponse

Response for successful repository creation.

```rust
/// HTTP response for repository creation.
///
/// Returns repository information and applied configuration details.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryResponse {
    /// Created repository information
    pub repository: RepositoryInfo,

    /// Configuration that was applied
    pub configuration: AppliedConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository name
    pub name: String,

    /// Full repository name (org/repo)
    pub full_name: String,

    /// GitHub repository URL
    pub url: String,

    /// Repository visibility
    pub visibility: String, // "public" or "private"

    /// Repository type (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedConfiguration {
    /// The merged configuration settings that were applied
    pub applied_settings: serde_json::Value,

    /// Source attribution for each setting (field path → source level)
    pub sources: HashMap<String, String>,
}
```

**HTTP Status**: `201 Created`

**Example**:

```json
{
  "repository": {
    "name": "my-new-service",
    "full_name": "myorg/my-new-service",
    "url": "https://github.com/myorg/my-new-service",
    "visibility": "private",
    "repository_type": "service",
    "created_at": "2025-11-12T10:30:00Z"
  },
  "configuration": {
    "applied_settings": {
      "repository": {
        "has_issues": true,
        "has_wiki": false,
        "has_projects": false,
        "delete_branch_on_merge": true
      },
      "branch_protection": {
        "default_branch": "main",
        "require_pull_request": true,
        "required_approvals": 2
      }
    },
    "sources": {
      "repository.has_issues": "global",
      "repository.has_wiki": "repository_type",
      "repository.has_projects": "repository_type",
      "repository.delete_branch_on_merge": "global",
      "branch_protection.required_approvals": "team"
    }
  }
}
```

---

### ValidateRepositoryNameResponse

Response for repository name validation.

```rust
/// HTTP response for repository name validation.
///
/// Always returns 200 OK with validation results (not error status).
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRepositoryNameResponse {
    /// Whether the name is valid
    pub valid: bool,

    /// The validated name (echoed back)
    pub name: String,

    /// Validation errors (empty if valid)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Field that failed validation
    pub field: String,

    /// Human-readable error message
    pub message: String,

    /// Machine-readable constraint identifier
    pub constraint: String,
}
```

**HTTP Status**: `200 OK` (always, even for invalid names)

**Example (Valid)**:

```json
{
  "valid": true,
  "name": "my-new-repo"
}
```

**Example (Invalid)**:

```json
{
  "valid": false,
  "name": "My@Repo",
  "errors": [
    {
      "field": "name",
      "message": "Repository name can only contain lowercase letters, numbers, hyphens, and underscores",
      "constraint": "alphanumeric-with-separators"
    },
    {
      "field": "name",
      "message": "Repository name cannot contain uppercase letters",
      "constraint": "lowercase-only"
    }
  ]
}
```

---

### ValidateRepositoryRequestResponse

Response for full request validation.

```rust
/// HTTP response for complete repository request validation.
///
/// Always returns 200 OK with validation results.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRepositoryRequestResponse {
    /// Whether the entire request is valid
    pub valid: bool,

    /// Validation warnings (non-fatal issues)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,

    /// Validation errors (fatal issues)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}
```

**HTTP Status**: `200 OK` (always)

**Example (Valid with Warning)**:

```json
{
  "valid": true,
  "warnings": [
    "Template specifies repository type 'library', overriding provided value 'service'"
  ]
}
```

**Example (Invalid)**:

```json
{
  "valid": false,
  "errors": [
    {
      "field": "variables.project_name",
      "message": "Required variable is missing",
      "constraint": "required"
    },
    {
      "field": "team",
      "message": "Team 'nonexistent-team' does not exist in organization",
      "constraint": "team-exists"
    }
  ]
}
```

---

## Template Discovery Responses

### ListTemplatesResponse

Response for listing available templates.

```rust
/// HTTP response for listing templates.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTemplatesResponse {
    /// List of available templates
    pub templates: Vec<TemplateSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSummary {
    /// Template name
    pub name: String,

    /// Brief description
    pub description: String,

    /// Template author/owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Template tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Repository type policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTypePolicy {
    /// Policy: "fixed", "preferable", "optional"
    pub policy: String,

    /// Type name (if policy is not "optional")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
}
```

**HTTP Status**: `200 OK`

**Example**:

```json
{
  "templates": [
    {
      "name": "rust-library",
      "description": "Rust library template with CI/CD and testing",
      "author": "Platform Team",
      "tags": ["rust", "library", "cargo"],
      "repository_type": {
        "policy": "fixed",
        "type_name": "library"
      }
    },
    {
      "name": "python-service",
      "description": "Python microservice with FastAPI",
      "author": "Backend Team",
      "tags": ["python", "service", "api"],
      "repository_type": {
        "policy": "preferable",
        "type_name": "service"
      }
    }
  ]
}
```

---

### GetTemplateDetailsResponse

Response for template details.

```rust
/// HTTP response for template details.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTemplateDetailsResponse {
    /// Template name
    pub name: String,

    /// Template metadata
    pub metadata: TemplateMetadata,

    /// Repository type policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<RepositoryTypePolicy>,

    /// Template variables
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Detailed description
    pub description: String,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,

    /// Variable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether variable is required
    pub required: bool,

    /// Default value (if not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
```

**HTTP Status**: `200 OK`

**Example**:

```json
{
  "name": "rust-library",
  "metadata": {
    "author": "Platform Team",
    "description": "Rust library template with CI/CD, testing, and documentation setup",
    "tags": ["rust", "library", "cargo", "github-actions"]
  },
  "repository_type": {
    "policy": "fixed",
    "type_name": "library"
  },
  "variables": [
    {
      "name": "project_name",
      "description": "Human-readable project name for documentation",
      "required": true,
      "default_value": null
    },
    {
      "name": "author",
      "description": "Package author name",
      "required": false,
      "default_value": "Engineering Team"
    },
    {
      "name": "description",
      "description": "Brief project description",
      "required": false,
      "default_value": "A Rust library"
    }
  ]
}
```

---

### ValidateTemplateResponse

Response for template validation.

```rust
/// HTTP response for template validation.
///
/// Always returns 200 OK with validation results.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTemplateResponse {
    /// Whether template is valid
    pub valid: bool,

    /// Template name (echoed back)
    pub template: String,

    /// Validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<TemplateValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateValidationError {
    /// File where error occurred
    pub file: String,

    /// Error message
    pub message: String,

    /// Line number (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}
```

**HTTP Status**: `200 OK` (always)

**Example (Valid)**:

```json
{
  "valid": true,
  "template": "rust-library"
}
```

**Example (Invalid)**:

```json
{
  "valid": false,
  "template": "rust-library",
  "errors": [
    {
      "file": "template.toml",
      "message": "Missing required field 'repository_type'",
      "line": null
    },
    {
      "file": "templates/README.md",
      "message": "Template file not found",
      "line": null
    }
  ]
}
```

---

## Organization Settings Responses

### ListRepositoryTypesResponse

Response for listing repository types.

```rust
/// HTTP response for listing repository types.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoryTypesResponse {
    /// List of repository type names
    pub types: Vec<String>,
}
```

**HTTP Status**: `200 OK`

**Example**:

```json
{
  "types": ["library", "service", "documentation", "infrastructure"]
}
```

---

### GetRepositoryTypeConfigResponse

Response for repository type configuration.

```rust
/// HTTP response for repository type configuration.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryTypeConfigResponse {
    /// Repository type name
    pub type_name: String,

    /// Type configuration settings
    pub configuration: serde_json::Value,
}
```

**HTTP Status**: `200 OK`

**Example**:

```json
{
  "type_name": "library",
  "configuration": {
    "repository": {
      "has_issues": { "value": true, "override_allowed": false },
      "has_wiki": { "value": false, "override_allowed": false },
      "has_projects": { "value": false, "override_allowed": true }
    },
    "branch_protection": {
      "default_branch": "main",
      "require_pull_request": true,
      "required_approvals": 2
    },
    "labels": [
      { "name": "bug", "color": "d73a4a", "description": "Bug report" }
    ]
  }
}
```

---

### GetGlobalDefaultsResponse

Response for global defaults.

```rust
/// HTTP response for global defaults.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGlobalDefaultsResponse {
    /// Global default configuration
    pub configuration: serde_json::Value,
}
```

**HTTP Status**: `200 OK`

**Example**:

```json
{
  "configuration": {
    "repository": {
      "visibility": { "value": "private", "override_allowed": true },
      "has_issues": { "value": true, "override_allowed": true },
      "delete_branch_on_merge": { "value": true, "override_allowed": false }
    },
    "branch_protection": {
      "default_branch": "main",
      "require_pull_request": true,
      "required_approvals": 1
    }
  }
}
```

---

### PreviewConfigurationResponse

Response for configuration preview.

```rust
/// HTTP response for configuration preview.
///
/// Shows the merged configuration that would be applied.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfigurationResponse {
    /// Merged configuration
    pub merged: serde_json::Value,

    /// Source attribution (field path → source level)
    pub sources: HashMap<String, String>,

    /// Validation results
    pub validation: ValidationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether configuration is valid
    pub valid: bool,

    /// Validation warnings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,

    /// Validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}
```

**HTTP Status**: `200 OK` (always, even if invalid)

**Example (Valid)**:

```json
{
  "merged": {
    "repository": {
      "visibility": "private",
      "has_issues": true,
      "has_wiki": false
    },
    "branch_protection": {
      "default_branch": "main",
      "require_pull_request": true,
      "required_approvals": 2
    }
  },
  "sources": {
    "repository.visibility": "global",
    "repository.has_issues": "repository_type",
    "repository.has_wiki": "template",
    "branch_protection.required_approvals": "team"
  },
  "validation": {
    "valid": true,
    "warnings": []
  }
}
```

---

### ValidateOrganizationResponse

Response for organization validation.

```rust
/// HTTP response for organization validation.
///
/// Always returns 200 OK with validation results.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateOrganizationResponse {
    /// Whether organization configuration is valid
    pub valid: bool,

    /// Organization name (echoed back)
    pub organization: String,

    /// Component-level validation results
    pub components: ComponentValidationResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentValidationResults {
    /// Global defaults validation
    pub global_defaults: ComponentValidation,

    /// Repository types validation
    pub repository_types: RepositoryTypesValidation,

    /// Teams validation (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<TeamsValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentValidation {
    /// Whether component is valid
    pub valid: bool,

    /// Validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTypesValidation {
    /// Whether all types are valid
    pub valid: bool,

    /// Number of types validated
    pub types_validated: usize,

    /// Validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsValidation {
    /// Whether all teams are valid
    pub valid: bool,

    /// Number of teams validated
    pub teams_validated: usize,

    /// Validation errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
}
```

**HTTP Status**: `200 OK` (always)

**Example (Valid)**:

```json
{
  "valid": true,
  "organization": "myorg",
  "components": {
    "global_defaults": {
      "valid": true,
      "errors": []
    },
    "repository_types": {
      "valid": true,
      "types_validated": 4,
      "errors": []
    },
    "teams": {
      "valid": true,
      "teams_validated": 2,
      "errors": []
    }
  }
}
```

---

## Health Check Response

### HealthCheckResponse

Response for health check.

```rust
/// HTTP response for health check.
///
/// See: specs/interfaces/api-response-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Service status: "healthy" or "unhealthy"
    pub status: String,

    /// Service version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Current timestamp (ISO 8601)
    pub timestamp: String,

    /// Error message (if unhealthy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**HTTP Status**: `200 OK` (healthy) or `503 Service Unavailable` (unhealthy)

**Example (Healthy)**:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2025-11-12T10:30:00Z"
}
```

**Example (Unhealthy)**:

```json
{
  "status": "unhealthy",
  "timestamp": "2025-11-12T10:30:00Z",
  "error": "Failed to connect to GitHub API"
}
```

---

## Response Serialization

### JSON Formatting

All responses use:

- **camelCase** field names (Rust uses snake_case, but serialize to camelCase for JSON)
- **ISO 8601** timestamps with UTC timezone
- **Omit null fields** using `#[serde(skip_serializing_if = "Option::is_none")]`
- **Pretty-print** in development, compact in production

### Content-Type Header

All responses include:

```
Content-Type: application/json; charset=utf-8
```

### Response Headers

Include standard headers:

```
Content-Type: application/json; charset=utf-8
X-Request-ID: <uuid>  (for tracing)
X-RateLimit-Limit: <limit>  (if rate limiting enabled)
X-RateLimit-Remaining: <remaining>
X-RateLimit-Reset: <reset-timestamp>
```

---

## Error Responses

See `specs/interfaces/api-error-handling.md` for complete error response specification.

**All error responses** use consistent structure:

```json
{
  "error": {
    "code": "ErrorType",
    "message": "Human-readable error message",
    "details": {
      // Context-specific error details
    }
  }
}
```

---

## Implementation Requirements

### Serde Configuration

All response types must use:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // Use camelCase for JSON
pub struct ResponseType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<String>,

    #[serde(default)]
    pub list_field: Vec<String>,
}
```

### Timestamp Formatting

Use chrono for timestamps:

```rust
use chrono::{DateTime, Utc};

fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}
```

### Documentation

Each response type must include:

- Rustdoc comments explaining purpose
- Field-level documentation
- Example JSON in doc comments
- HTTP status code(s) for this response
- Reference to this specification

---

## Testing Requirements

Each response type must have tests for:

1. **Valid serialization** - Correct JSON output
2. **Optional field omission** - None values not serialized
3. **Default values** - Empty collections use defaults
4. **Timestamp formatting** - ISO 8601 compliance
5. **camelCase conversion** - Field names converted correctly

**Example Test**:

```rust
#[test]
fn test_create_repository_response_serialization() {
    let response = CreateRepositoryResponse {
        repository: RepositoryInfo {
            name: "my-repo".to_string(),
            full_name: "myorg/my-repo".to_string(),
            url: "https://github.com/myorg/my-repo".to_string(),
            visibility: "private".to_string(),
            repository_type: Some("library".to_string()),
            created_at: "2025-11-12T10:30:00Z".to_string(),
        },
        configuration: AppliedConfiguration {
            applied_settings: serde_json::json!({
                "has_issues": true
            }),
            sources: HashMap::from([
                ("repository.has_issues".to_string(), "global".to_string())
            ]),
        },
    };

    let json = serde_json::to_string_pretty(&response).unwrap();
    assert!(json.contains("\"repository\""));
    assert!(json.contains("\"configuration\""));
}
```

---

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

---

## Related Specifications

- **specs/interfaces/api-request-types.md** - HTTP request models
- **specs/interfaces/api-error-handling.md** - Error response patterns
- **specs/interfaces/repository-domain.md** - Domain result types
- **specs/constraints.md** - Architectural constraints

---

**Status**: Interface Specification
**Next Steps**: Implement response types in `repo_roller_api/src/models/response.rs`
