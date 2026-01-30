# GitHub Integration Interfaces

**Architectural Layer**: Business Interface (trait) + Infrastructure (GitHub API implementation)
**Crate**: `github_client`
**Responsibilities**:

- **Knows**: GitHub API protocols, authentication flows
- **Does**: Creates repositories, fetches data, manages installations

---

## Overview

GitHub integration provides the boundary between business logic and GitHub's API. All GitHub operations go through these interfaces.

## Current State

**Existing Traits** (already well-designed):

- `RepositoryClient` - Repository operations
- `TemplateFetcher` - (actually in `template_engine` crate)

**TODO**: Add organization and permissions interfaces

---

## Existing Interface: RepositoryClient

```rust
#[async_trait]
pub trait RepositoryClient: Send + Sync {
    async fn create_user_repository(
        &self,
        payload: &RepositoryCreatePayload,
    ) -> Result<Repository, GitHubClientError>;

    async fn create_org_repository(
        &self,
        org: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<Repository, GitHubClientError>;

    async fn get_installation_token_for_org(&self, org: &str)
        -> Result<String, GitHubClientError>;

    async fn get_organization_default_branch(&self, org: &str)
        -> Result<String, GitHubClientError>;
}
```

**Status**: ✅ Well-designed, already in use

**TODO**: Migrate `org: &str` parameters to use `OrganizationName` type

---

## New Interface: OrganizationProvider

```rust
#[async_trait]
pub trait OrganizationProvider: Send + Sync {
    /// Get organization details
    async fn get_organization(&self, org: &OrganizationName)
        -> Result<Organization, GitHubClientError>;

    /// Check if organization exists
    async fn organization_exists(&self, org: &OrganizationName)
        -> Result<bool, GitHubClientError>;

    /// Get organization default branch setting
    async fn get_default_branch(&self, org: &OrganizationName)
        -> Result<String, GitHubClientError>;
}
```

---

## New Interface: InstallationProvider

```rust
#[async_trait]
pub trait InstallationProvider: Send + Sync {
    /// Get installation ID for an organization
    async fn get_installation_for_org(&self, org: &OrganizationName)
        -> Result<InstallationId, GitHubClientError>;

    /// Get installation token
    async fn get_installation_token(&self, installation_id: InstallationId)
        -> Result<GitHubToken, GitHubClientError>;
}
```

---

## Types

### RepositoryCreatePayload

```rust
pub struct RepositoryCreatePayload {
    pub name: String, // TODO: Change to RepositoryName
    pub description: Option<String>,
    pub private: bool,
    pub auto_init: bool,
}
```

### Repository (response)

```rust
pub struct Repository {
    // Fields from octocrab response
}
```

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitHubClientError {
    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

---

## Implementation Notes

- Use `octocrab` as GitHub API client
- Handle rate limiting gracefully
- Retry transient errors
- Log all API calls for debugging

---

## Related Specifications

- `specs/interfaces/repository-domain.md` - Uses these interfaces
- `specs/interfaces/authentication-interfaces.md` - Authentication integration
- `specs/interfaces/shared-types.md` - Type definitions
