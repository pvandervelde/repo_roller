# Shared Types Specification

**Module**: Core shared types across all domains
**Location**: `repo_roller_core/src/types.rs`
**Purpose**: Define reusable domain primitives and common types

## Overview

This specification defines the foundational types used throughout RepoRoller. All types use the newtype pattern (branded types) to provide compile-time type safety and prevent accidental mixing of conceptually different values.

## Architectural Layer

**Layer**: Domain Types (Core)
**Dependencies**: Standard library only (`std`, `serde`, `uuid`, `chrono`)
**Used By**: All business logic and infrastructure modules

## Core Result Types

### RepoRollerResult<T>

Primary result type for operations that can fail.

```rust
pub type RepoRollerResult<T> = Result<T, RepoRollerError>;
```

**Usage**: Top-level operations that may encounter any category of error.

**Example**:

```rust
pub async fn create_repository(request: CreateRepositoryRequest)
    -> RepoRollerResult<Repository>
```

## Domain Primitive Types

All domain primitives follow the newtype pattern for type safety.

### RepositoryName

Represents a valid GitHub repository name.

```rust
/// A validated GitHub repository name.
///
/// Repository names must:
/// - Be 1-100 characters long
/// - Contain only alphanumeric characters, hyphens, underscores, and periods
/// - Not start with a period or hyphen
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryName(String);

impl RepositoryName {
    /// Create a new RepositoryName from a string.
    ///
    /// # Errors
    /// Returns ValidationError if the name doesn't meet GitHub's requirements.
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        todo!("Validate repository name according to GitHub rules")
    }

    /// Get the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**Validation Rules**:

- Length: 1-100 characters
- Pattern: `^[a-zA-Z0-9._-]+$`
- Must not start with `.` or `-`

### OrganizationName

Represents a GitHub organization or user account name.

```rust
/// A GitHub organization or user name.
///
/// Organization names must:
/// - Be 1-39 characters long
/// - Contain only alphanumeric characters and hyphens
/// - Not start or end with a hyphen
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationName(String);

impl OrganizationName {
    /// Create a new OrganizationName from a string.
    ///
    /// # Errors
    /// Returns ValidationError if the name doesn't meet GitHub's requirements.
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        todo!("Validate organization name according to GitHub rules")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Validation Rules**:

- Length: 1-39 characters
- Pattern: `^[a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)*$`

### TemplateName

Represents a template identifier.

```rust
/// A template identifier used to reference repository templates.
///
/// Template names are organization-specific and follow kebab-case convention.
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateName(String);

impl TemplateName {
    /// Create a new TemplateName from a string.
    ///
    /// # Errors
    /// Returns ValidationError if the name contains invalid characters.
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        todo!("Validate template name (kebab-case recommended)")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**Validation Rules**:

- Length: 1-100 characters
- Pattern: `^[a-z0-9]+(?:[_-][a-z0-9]+)*$` (kebab-case or snake_case)

### UserId

Unique identifier for a GitHub user.

```rust
/// A unique identifier for a GitHub user.
///
/// Wraps a UUID to provide type safety and prevent mixing with other IDs.
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(uuid::Uuid);

impl UserId {
    /// Create a new random UserId
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}
```

### SessionId

Unique identifier for user sessions (web interface).

```rust
/// A unique identifier for a user session.
///
/// Used by web interface to track authenticated user sessions.
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
}
```

### InstallationId

GitHub App installation identifier.

```rust
/// GitHub App installation ID for an organization.
///
/// Each organization where the GitHub App is installed has a unique installation ID.
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstallationId(u64);

impl InstallationId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for InstallationId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}
```

### GitHubToken

Secure wrapper for GitHub authentication tokens.

```rust
/// A GitHub authentication token (PAT, OAuth, or Installation token).
///
/// Implements security best practices:
/// - No Debug output (prevents accidental logging)
/// - Zeroize on drop (clears memory)
/// - Explicit access required
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Clone, Serialize, Deserialize)]
pub struct GitHubToken(secrecy::Secret<String>);

impl GitHubToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(secrecy::Secret::new(token.into()))
    }

    /// Access the token value (use sparingly)
    pub fn expose_secret(&self) -> &str {
        use secrecy::ExposeSecret;
        self.0.expose_secret()
    }
}

// Explicitly no Debug implementation to prevent logging tokens
```

## Common Value Objects

### Timestamp

Wrapper for timestamps used throughout the system.

```rust
/// A UTC timestamp for events and metadata.
///
/// See spec: specs/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);

impl Timestamp {
    /// Get current UTC timestamp
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    /// Create from DateTime
    pub fn from_datetime(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt)
    }

    /// Get the underlying DateTime
    pub fn as_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }

    /// Format as RFC3339 string
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}
```

## Error Handling

All constructors that perform validation return appropriate errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid repository name: {0}")]
    InvalidRepositoryName(String),

    #[error("Invalid organization name: {0}")]
    InvalidOrganizationName(String),

    #[error("Invalid template name: {0}")]
    InvalidTemplateName(String),

    #[error("Value too short: minimum length {min}, got {actual}")]
    TooShort { min: usize, actual: usize },

    #[error("Value too long: maximum length {max}, got {actual}")]
    TooLong { max: usize, actual: usize },

    #[error("Invalid pattern: expected {expected}, got '{actual}'")]
    PatternMismatch { expected: String, actual: String },
}
```

## Usage Examples

### Creating Branded Types

```rust
// Valid repository name
let repo_name = RepositoryName::new("my-awesome-project")?;
println!("Repository: {}", repo_name);

// Invalid repository name (starts with period)
let invalid = RepositoryName::new(".invalid");
assert!(invalid.is_err());

// Organization name
let org = OrganizationName::new("my-company")?;

// Using in function signatures (type safety enforced)
fn create_repo(org: OrganizationName, name: RepositoryName) {
    // Compiler prevents passing wrong types
}

// This won't compile:
// create_repo(repo_name, org); // Error: wrong types!
```

### Working with Tokens

```rust
// Create token (never logged due to no Debug impl)
let token = GitHubToken::new("ghp_xxxxxxxxxxxxxxxxxxxx");

// Use token (explicit access required)
let token_str = token.expose_secret();
authenticate_with_github(token_str).await?;

// Token is automatically zeroized when dropped
```

### Timestamps

```rust
let created_at = Timestamp::now();
let formatted = created_at.to_rfc3339();
println!("Created: {}", formatted);

// Store in structs
struct Repository {
    name: RepositoryName,
    created_at: Timestamp,
}
```

## Implementation Notes

### Type Safety Benefits

Branded types prevent common bugs:

- Can't pass `RepositoryName` where `OrganizationName` expected
- Can't accidentally log `GitHubToken` in debug output
- Can't mix different kinds of IDs (`UserId` vs `SessionId`)

### Serialization

All types implement `Serialize` and `Deserialize` for:

- JSON API responses
- Configuration file parsing
- Database storage (future)

### Performance

Newtype pattern has zero runtime cost:

- Wrapped type has same memory layout
- No indirection or allocation overhead
- Optimizes to raw type in release builds

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
secrecy = "0.8"
thiserror = "1.0"
```

## Testing Requirements

Each branded type must have tests for:

- Valid value construction
- Invalid value rejection with appropriate errors
- Serialization/deserialization round-trip
- Display formatting
- Equality and hashing (for types used in collections)

## Migration Path

For existing code using `String` or primitive types:

```rust
// Before
fn create_repository(org: &str, name: &str) -> Result<()> { }

// After
fn create_repository(org: OrganizationName, name: RepositoryName) -> Result<()> { }

// Migration wrapper (temporary)
fn create_repository_legacy(org: &str, name: &str) -> Result<()> {
    let org_name = OrganizationName::new(org)?;
    let repo_name = RepositoryName::new(name)?;
    create_repository(org_name, repo_name)
}
```

---

**Status**: Interface Specification
**Next Steps**: Implement types in `repo_roller_core/src/types.rs`
**See Also**: [Error Types](error-types.md), [Repository Domain](repository-domain.md)
