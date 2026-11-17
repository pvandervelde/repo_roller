# Shared Types Specification

**Module**: Core shared types across all domains
**Location**: `repo_roller_core/src/repository.rs`, `template.rs`, `github.rs`, `authentication.rs`, and `lib.rs`
**Purpose**: Define reusable domain primitives and common types

## Overview

This specification describes the foundational types used throughout RepoRoller. Domain names and identifiers use newtypes to provide compile-time type safety and prevent accidental mixing of conceptually different values.

## Architectural Layer

**Layer**: Domain Types (Core)
**Dependencies**: Core domain modules use `serde`, `uuid`, `chrono`, and `thiserror` where applicable.
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
pub async fn create_repository(request: RepositoryCreationRequest)
    -> RepoRollerResult<RepositoryCreationResult>
```

## Domain Primitive Types

All domain primitives follow the newtype pattern for type safety.

### RepositoryName

Represents a repository name validated by RepoRoller.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryName(String);
```

`RepositoryName::new` rejects an empty value, values longer than 100 UTF-8
bytes, names starting with `.` or `-`, and characters other than Unicode
alphanumeric characters, hyphen, underscore, or period. It returns
`ValidationError` on failure. `as_str`, `AsRef<str>`, and `Display` expose the
validated value as text.

### OrganizationName

Represents a GitHub organization or user account name validated by RepoRoller.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrganizationName(String);
```

`OrganizationName::new` rejects an empty value, values longer than 39 UTF-8
bytes, leading or trailing hyphens, consecutive hyphens, and characters other
than Unicode alphanumeric characters or hyphens. It returns `ValidationError`
on failure.

### TemplateName

Represents a template identifier.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateName(String);
```

`TemplateName::new` accepts 1–50 UTF-8 bytes containing only lowercase ASCII
letters, digits, or hyphens. The value must not start or end with a hyphen.
Uppercase letters, underscores, spaces, and other characters are rejected.
Validation failures return `ValidationError`.

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
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubToken(String);
```

`GitHubToken::new` rejects empty tokens and tokens shorter than 10 UTF-8
bytes. `as_str` provides explicit access; its `Debug` and `Display`
implementations redact the token value. The wrapper does not zeroize memory.
Because it implements serialization, callers must not serialize it into logs
or other untrusted output.

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

The validated name and token constructors return
`repo_roller_core::ValidationError`. Depending on the failed rule, this is
`EmptyField`, `TooLong`, `TooShort`, or `InvalidFormat`, with the field name
and constraint details included. No value is normalized automatically; a
caller must correct invalid input and construct the type again.

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
// Documentation-only value; never use a real credential in examples.
let token = GitHubToken::new("PLACEHOLDER_TOKEN_NOT_A_REAL_CREDENTIAL")?;

// Use token (explicit access required)
let token_str = token.as_str();
authenticate_with_github(token_str).await?;
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

The listed domain wrappers derive `Serialize` and `Deserialize` for use by
API and configuration boundaries. In particular, serializing `GitHubToken`
exposes its credential value; do not include it in logs, diagnostics, or
untrusted output.

### Performance

The newtypes preserve the wrapped value's ordinary ownership and allocation
behavior while adding validation at construction and type distinctions at
compile time.

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
```

## Testing Requirements

Each branded type should have tests for:

- Valid value construction
- Invalid value rejection with appropriate errors
- Serialization/deserialization round-trip where supported
- Display formatting where implemented
- Equality and hashing (for types used in collections)

## Boundary Usage

External strings should be validated with the relevant constructor before
they enter domain logic. The repository-creation request uses
`RepositoryName`, `OrganizationName`, and optional `TemplateName` values;
GitHub and template-engine APIs that accept strings receive their string
representations at those boundaries.

---

**Status**: Implemented in the domain modules listed above
**See Also**: [Error Types](error-types.md), [Repository Domain](repository-domain.md)
