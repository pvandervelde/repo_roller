# GitHub Integration Interfaces

**Architectural Layer**: Infrastructure adapter
**Crate**: `github_client`
**Responsibilities**:

- **Knows**: GitHub API operations, authentication, and API error handling
- **Does**: Creates and updates repositories, retrieves organization settings, and applies repository access

---

## Overview

`GitHubClient` wraps the GitHub API client used by RepoRoller. Repository
creation is exposed through the `RepositoryClient` trait; organization
lookups and other GitHub operations are methods on `GitHubClient`. The
interfaces use strings at the GitHub API boundary. Core workflows validate
names with `OrganizationName` and `RepositoryName` before calling them.

There are no separate `OrganizationProvider` or `InstallationProvider`
traits in the current API.

## Repository operations

```rust
#[async_trait]
pub trait RepositoryClient: Send + Sync {
    async fn create_org_repository(
        &self,
        owner: &str,
        payload: &RepositoryCreatePayload,
    ) -> Result<Repository, Error>;

    async fn create_user_repository(
        &self,
        payload: &RepositoryCreatePayload,
    ) -> Result<Repository, Error>;

    async fn get_organization_default_branch(
        &self,
        org_name: &str,
    ) -> Result<String, Error>;
}
```

`RepositoryCreatePayload` contains the repository name, optional
description/homepage, privacy, and selected GitHub repository feature flags.
The `Repository` returned by the client represents the GitHub response.
`RepositoryClient` also exposes repository update operations; see its
definition in `crates/github_client/src/lib.rs` for the complete method list.

## Organization and installation operations

`GitHubClient` provides concrete methods for organization-related operations,
including:

- `get_installation_token_for_org(org_name: &str) -> Result<String, Error>`
- `get_organization_default_branch(org_name: &str) -> Result<String, Error>`

These are API-boundary methods and accept string identifiers. The core wraps
and validates organization and installation values in its domain types where
those types are part of the workflow; the GitHub client does not define a
separate organization-details or installation-provider abstraction.

## Permissions

Repository permissions are coordinated by
`repo_roller_core::permission_manager::PermissionManager`, not a GitHub
permissions-provider trait. The manager evaluates requests using the core
policy engine, then applies approved team and collaborator access using
`GitHubClient` methods such as `add_team_to_repository`,
`set_team_repository_permission`, `add_repository_collaborator`,
`set_collaborator_permission`, and `remove_repository_collaborator`.

- Team access is applied through the GitHub team-repository API.
- Collaborator access is applied through the collaborator API.
- A collaborator request with `AccessLevel::None` removes an existing
  collaborator; team removal is not currently exposed by `GitHubClient`.
- Permission decisions and application outcomes are recorded by the core
  permission audit logger.

Permission policy and request types are defined in
`repo_roller_core::permissions`; see the
[permission design specification](../design/multi-level-permissions.md).

## Errors and behavior

GitHub operations return `github_client::Error`. Callers should propagate
these errors through their domain error type rather than treating an API
failure as a successful operation. GitHub API authentication and rate limits
are managed by the client and the specific API operation.

## Related Specifications

- [Repository Domain](repository-domain.md) — Repository creation orchestration
- [Shared Types](shared-types.md) — Validated names and token handling
- [Permission Design](../design/multi-level-permissions.md) — Permission policy and application
