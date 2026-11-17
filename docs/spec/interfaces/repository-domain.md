# Repository Domain Interfaces

**Architectural Layer**: Core business logic
**Crate**: `repo_roller_core`
**Responsibilities**:

- **Knows**: Repository creation rules, validated domain values, and workflow state
- **Does**: Resolves configuration and orchestrates repository creation

---

## Overview

The repository domain accepts a typed creation request, resolves applicable
configuration, validates policy and naming constraints, and coordinates the
GitHub, template, and content providers. The public operation returns a
structured result on success and a domain error on failure.

## Request model

`RepositoryCreationRequest` is the input to repository creation:

```rust
pub struct RepositoryCreationRequest {
    pub name: RepositoryName,
    pub owner: OrganizationName,
    pub template: Option<TemplateName>,
    pub variables: HashMap<String, String>,
    pub visibility: Option<RepositoryVisibility>,
    pub content_strategy: ContentStrategy,
    pub creation_mode: CreationMode,
    pub teams: HashMap<String, AccessLevel>,
    pub collaborators: HashMap<String, AccessLevel>,
    pub actor_login: String,
}
```

`RepositoryCreationRequestBuilder` constructs requests from validated
`RepositoryName` and `OrganizationName` values and provides setters for
optional fields. `template` may be absent for empty or custom-init content.
`ContentStrategy` selects template, empty, or custom-init content generation.
`CreationMode` selects the standard create flow, a GitHub fork, or a mirror;
fork and mirror modes carry a `SourceRepositoryReference`.

## Result model

Successful creation returns `RepositoryCreationResult`:

```rust
pub struct RepositoryCreationResult {
    pub repository_url: String,
    pub repository_id: String,
    pub created_at: Timestamp,
    pub default_branch: String,
}
```

The result contains the GitHub URL and ID, creation timestamp, and default
branch. It is returned only when the requested creation workflow succeeds.

## Public operation

```rust
pub async fn create_repository(
    request: RepositoryCreationRequest,
    metadata_provider: &dyn MetadataRepositoryProvider,
    auth_service: &dyn UserAuthenticationService,
    metadata_repository_name: &str,
    visibility_policy_provider: Arc<dyn VisibilityPolicyProvider>,
    environment_detector: Arc<dyn GitHubEnvironmentDetector>,
    event_context: EventNotificationContext,
) -> RepoRollerResult<RepositoryCreationResult>;
```

The operation resolves metadata and merged configuration, validates the
repository name and any fork/mirror capability constraints before making
repository-mutating GitHub calls, then dispatches according to
`CreationMode`. The standard flow obtains template content when selected,
creates and initializes the repository, applies configured settings and
permissions, and pushes generated content. Successful creation schedules
outbound event notification on a best-effort basis.

## Error model

The operation returns `RepoRollerResult<T>`, an alias for
`Result<T, RepoRollerError>`. `RepoRollerError` wraps domain errors for:

- Input and naming validation (`ValidationError`)
- Repository operations (`RepositoryError`)
- Configuration loading, merging, and policy (`ConfigurationError`)
- Template processing (`TemplateError`)
- Authentication (`AuthenticationError`)
- GitHub API calls (`GitHubError`)
- System operations (`SystemError`)
- Permission policy or application (`PermissionError`)

Failures are returned to the caller; outbound notification delivery failures
are logged and do not change a successful repository-creation result.

## Supporting components

- `OrganizationSettingsManager` resolves metadata and merged configuration.
- `TemplateLoader` loads template configuration; `TemplateFetcher` retrieves
  template files for content generation.
- `PermissionManager` evaluates policy and applies approved team and
  collaborator access.
- `GitHubClient` performs GitHub API operations through `RepositoryClient`
  and its other concrete methods.

## Related Specifications

- [Shared Types](shared-types.md) — Validated domain values
- [Configuration Management](configuration-interfaces.md) — Configuration resolution
- [GitHub Integration](github-interfaces.md) — GitHub API boundary
- [Template Processing](template-interfaces.md) — Template file rendering
- [Template Loading](template-loading.md) — Template configuration retrieval
