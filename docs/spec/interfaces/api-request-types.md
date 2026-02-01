# REST API Request Types

**Module**: HTTP API Request Models
**Location**: `repo_roller_api/src/models/request.rs`
**Purpose**: Define HTTP request types for REST API endpoints

---

## Overview

This specification defines all HTTP request models for the RepoRoller REST API. These types are **distinct from domain types** and exist in the HTTP layer only. They are translated to domain types before invoking business logic.

**Architectural Layer**: HTTP Interface (Infrastructure)
**Crate**: `repo_roller_api`

**Key Principles**:

- HTTP request types have optional fields and string types for flexibility
- Domain types use branded types and have stricter validation
- Translation from HTTP to domain happens at API boundary
- Validation errors during translation use domain error types

---

## Repository Management Requests

### CreateRepositoryRequest

HTTP request for creating a new repository.

```rust
/// HTTP request to create a repository.
///
/// This type accepts flexible input from HTTP clients and is translated
/// to RepositoryCreationRequest (domain type) after validation.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRepositoryRequest {
    /// Organization name (GitHub organization)
    pub organization: String,

    /// Repository name (must follow GitHub naming rules)
    pub name: String,

    /// Template name to use for repository creation
    pub template: String,

    /// Repository visibility (optional, defaults from configuration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>, // "public" or "private"

    /// Team name for team-specific configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Repository type override (optional, template may specify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,

    /// Template variables for substitution
    #[serde(default)]
    pub variables: HashMap<String, String>,
}
```

**Validation Rules**:

- `organization`: Non-empty, valid GitHub organization name format
- `name`: Non-empty, GitHub repository name format (lowercase, hyphens, no special chars)
- `template`: Non-empty, must exist in organization's metadata repository
- `visibility`: If provided, must be "public" or "private"
- `team`: If provided, must exist in organization
- `repository_type`: If provided, must be defined in organization configuration
- `variables`: Keys must match template's required variables

**Translation to Domain**:

```rust
impl TryFrom<CreateRepositoryRequest> for repo_roller_core::RepositoryCreationRequest {
    type Error = ValidationError;

    fn try_from(req: CreateRepositoryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            name: RepositoryName::new(req.name)?,
            owner: OrganizationName::new(req.organization)?,
            template: TemplateName::new(req.template)?,
            variables: req.variables,
        })
    }
}
```

**Example**:

```json
{
  "organization": "myorg",
  "name": "my-new-service",
  "template": "rust-microservice",
  "visibility": "private",
  "team": "backend-team",
  "repository_type": "service",
  "variables": {
    "service_name": "MyService",
    "author": "Backend Team",
    "description": "A new microservice"
  }
}
```

---

### ValidateRepositoryNameRequest

HTTP request to validate a repository name.

```rust
/// HTTP request to validate repository name format and availability.
///
/// This endpoint checks both:
/// 1. Name format (GitHub naming rules)
/// 2. Availability (name not already taken in organization)
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidateRepositoryNameRequest {
    /// Organization name
    pub organization: String,

    /// Repository name to validate
    pub name: String,
}
```

**Validation Rules**:

- `organization`: Non-empty, valid GitHub organization name
- `name`: Any string (validation is the purpose of this endpoint)

**Behavior**:

1. Validates name format against GitHub rules
2. Checks if repository already exists in organization via GitHub API
3. Returns validation result (not an error - 200 OK response)

**Example**:

```json
{
  "organization": "myorg",
  "name": "my-new-repo"
}
```

---

### ValidateRepositoryRequestRequest

HTTP request to validate a complete repository creation request.

**Note**: This uses the same structure as `CreateRepositoryRequest` but returns validation results instead of creating a repository.

```rust
/// HTTP request to validate a complete repository creation request.
///
/// This validates:
/// - Repository name format and availability
/// - Template existence and accessibility
/// - Required variables are provided
/// - Optional fields are valid if provided
/// - Configuration override permissions
///
/// Type alias for CreateRepositoryRequest since structure is identical.
///
/// See: specs/interfaces/api-request-types.md
pub type ValidateRepositoryRequestRequest = CreateRepositoryRequest;
```

**Validation Performed**:

- All CreateRepositoryRequest validation rules
- Template variable completeness (all required variables provided)
- Configuration override permissions (if visibility/settings overridden)
- Template accessibility with current authentication

**Example**: Same as `CreateRepositoryRequest`

---

## Template Discovery Requests

### ListTemplatesRequest

HTTP request to list available templates (no request body - path parameter only).

```rust
/// Path parameters for listing templates.
///
/// Templates are discovered from the organization's metadata repository.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTemplatesParams {
    /// Organization name
    pub org: String,
}
```

**Endpoint**: `GET /api/v1/orgs/{org}/templates`

**No request body** - organization is path parameter only.

---

### GetTemplateDetailsRequest

HTTP request to get template details (no request body - path parameters only).

```rust
/// Path parameters for getting template details.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTemplateDetailsParams {
    /// Organization name
    pub org: String,

    /// Template name
    pub template: String,
}
```

**Endpoint**: `GET /api/v1/orgs/{org}/templates/{template}`

**No request body** - parameters are in path only.

---

### ValidateTemplateRequest

HTTP request to validate a template (no request body - path parameters only).

```rust
/// Path parameters for validating a template.
///
/// See: specs/interfaces/api-request-types.md
pub type ValidateTemplateParams = GetTemplateDetailsParams;
```

**Endpoint**: `POST /api/v1/orgs/{org}/templates/{template}/validate`

**No request body** - validation target is in path parameters.

---

## Organization Settings Requests

### ListRepositoryTypesRequest

HTTP request to list repository types (no request body - path parameter only).

```rust
/// Path parameters for listing repository types.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoryTypesParams {
    /// Organization name
    pub org: String,
}
```

**Endpoint**: `GET /api/v1/orgs/{org}/types`

**No request body** - organization is path parameter only.

---

### GetRepositoryTypeConfigRequest

HTTP request to get repository type configuration (no request body - path parameters only).

```rust
/// Path parameters for getting repository type configuration.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryTypeConfigParams {
    /// Organization name
    pub org: String,

    /// Repository type name
    pub type_name: String,
}
```

**Endpoint**: `GET /api/v1/orgs/{org}/types/{type}`

**No request body** - parameters are in path only.

---

### GetGlobalDefaultsRequest

HTTP request to get global defaults (no request body - path parameter only).

```rust
/// Path parameters for getting global defaults.
///
/// See: specs/interfaces/api-request-types.md
pub type GetGlobalDefaultsParams = ListRepositoryTypesParams;
```

**Endpoint**: `GET /api/v1/orgs/{org}/global`

**No request body** - organization is path parameter only.

---

### PreviewConfigurationRequest

HTTP request to preview merged configuration.

```rust
/// HTTP request to preview merged configuration.
///
/// This shows what configuration will be applied based on the
/// hierarchical merge of global → org → team → type → template.
///
/// See: specs/interfaces/api-request-types.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewConfigurationRequest {
    /// Template name (required)
    pub template: String,

    /// Team name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Repository type (optional - template may specify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_type: Option<String>,
}
```

**Validation Rules**:

- `template`: Must exist in organization's metadata repository
- `team`: If provided, must exist in organization
- `repository_type`: If provided, must be defined in organization configuration

**Behavior**:

1. Resolves configuration hierarchy
2. Applies merge logic with precedence rules
3. Validates override permissions
4. Returns merged configuration with source attribution

**Example**:

```json
{
  "template": "rust-library",
  "team": "platform",
  "repository_type": "library"
}
```

---

### ValidateOrganizationRequest

HTTP request to validate organization settings (no request body - path parameter only).

```rust
/// Path parameters for validating organization settings.
///
/// See: specs/interfaces/api-request-types.md
pub type ValidateOrganizationParams = ListRepositoryTypesParams;
```

**Endpoint**: `POST /api/v1/orgs/{org}/validate`

**No request body** - organization is path parameter only.

**Validation Performed**:

- Global defaults schema validation
- All repository type configurations validation
- All team configurations validation
- Override policy consistency checks

---

## Health Check Request

### HealthCheckRequest

Health check endpoint (no request body or parameters).

**Endpoint**: `GET /health`

**No request body, no parameters** - always accessible without authentication.

---

## Request Validation Patterns

### Common Validation Rules

**Organization Names**:

- Pattern: `^[a-z0-9][a-z0-9-]*[a-z0-9]$`
- Length: 1-39 characters
- Must exist and be accessible to authenticated user

**Repository Names**:

- Pattern: `^[a-z0-9._-]+$`
- Length: 1-100 characters
- Cannot start with `.`
- Cannot contain `..` sequence

**Template Names**:

- Pattern: `^[a-z0-9-_]+$`
- Length: 1-100 characters
- Must exist in organization's metadata repository

**Team Names**:

- Pattern: `^[a-z0-9-_]+$`
- Length: 1-100 characters
- Must exist in organization's GitHub teams

**Repository Type Names**:

- Pattern: `^[a-z0-9-_]+$`
- Length: 1-50 characters
- Must be defined in organization configuration

### Validation Error Responses

When validation fails during request parsing or translation, return `400 Bad Request`:

```json
{
  "error": {
    "code": "ValidationError",
    "message": "Repository name contains invalid characters",
    "details": {
      "field": "name",
      "value": "My@Repo",
      "constraint": "alphanumeric with hyphens, underscores, and dots only"
    }
  }
}
```

---

## Security Considerations

### Input Sanitization

All string fields must be sanitized before use:

- Trim whitespace
- Validate character sets
- Check length constraints
- Prevent injection attacks (SQL, path traversal, etc.)

### Sensitive Data

**Never include in request types**:

- GitHub tokens (passed via Authorization header)
- Passwords or secrets
- Private keys

**Template variables** may contain sensitive values - handle with care:

- Do not log variable values
- Sanitize before error messages
- Consider marking certain variables as sensitive in template config

---

## Implementation Requirements

### Serde Configuration

All request types must use:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]  // Reject unknown fields
pub struct RequestType {
    // ...
}
```

This prevents:

- Typos in field names being silently ignored
- Forward compatibility issues
- Confusion about what fields are supported

### Documentation

Each request type must include:

- Rustdoc comments explaining purpose
- Field-level documentation
- Example JSON in doc comments
- Reference to this specification

---

## Testing Requirements

Each request type must have tests for:

1. **Valid deserialization** - Correct JSON deserializes successfully
2. **Invalid field rejection** - Unknown fields cause error
3. **Missing required fields** - Error with clear message
4. **Invalid values** - Type mismatches caught
5. **Translation to domain types** - Conversion logic tested
6. **Validation errors** - Proper ValidationError returned

**Example Test**:

```rust
#[test]
fn test_create_repository_request_deserialization() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "variables": {}
    }"#;

    let req: CreateRepositoryRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.organization, "myorg");
    assert_eq!(req.name, "my-repo");
    assert_eq!(req.template, "rust-library");
}

#[test]
fn test_create_repository_request_unknown_field() {
    let json = r#"{
        "organization": "myorg",
        "name": "my-repo",
        "template": "rust-library",
        "unknown_field": "value"
    }"#;

    let result = serde_json::from_str::<CreateRepositoryRequest>(json);
    assert!(result.is_err());
}
```

---

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Related Specifications

- **specs/interfaces/api-response-types.md** - HTTP response models
- **specs/interfaces/api-error-handling.md** - Error response patterns
- **specs/interfaces/repository-domain.md** - Domain types for translation
- **specs/interfaces/shared-types.md** - Common domain types
- **specs/constraints.md** - Architectural constraints

---

**Status**: Interface Specification
**Next Steps**: Implement request types in `repo_roller_api/src/models/request.rs`
