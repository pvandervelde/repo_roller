# Shared Types Registry

This registry catalogs all reusable types, traits, and patterns across the RepoRoller codebase. Update this document when creating new shared abstractions or modifying existing ones.

## Purpose

- **Discovery**: Help developers find existing types instead of duplicating
- **Location Tracking**: Document where each type is defined and used
- **Consistency**: Ensure naming and usage patterns are consistent
- **Dependency Management**: Track which types depend on others

## Core Domain Types

### Branded Types (Newtype Pattern)

All domain primitives use newtype pattern for type safety.

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `RepositoryName` | `repo_roller_core/src/repository.rs` | GitHub repository name | [shared-types.md](interfaces/shared-types.md#repositoryname) |
| `OrganizationName` | `repo_roller_core/src/repository.rs` | GitHub org/user name | [shared-types.md](interfaces/shared-types.md#organizationname) |
| `TemplateName` | `repo_roller_core/src/template.rs` | Template identifier | [shared-types.md](interfaces/shared-types.md#templatename) |
| `UserId` | `repo_roller_core/src/authentication.rs` | Unique user identifier | [shared-types.md](interfaces/shared-types.md#userid) |
| `SessionId` | `repo_roller_core/src/authentication.rs` | Web session identifier | [shared-types.md](interfaces/shared-types.md#sessionid) |
| `InstallationId` | `repo_roller_core/src/github.rs` | GitHub App installation ID | [shared-types.md](interfaces/shared-types.md#installationid) |
| `GitHubToken` | `repo_roller_core/src/github.rs` | Secure token wrapper | [shared-types.md](interfaces/shared-types.md#githubtoken) |
| `Timestamp` | `repo_roller_core/src/lib.rs` | UTC timestamp wrapper | [shared-types.md](interfaces/shared-types.md#timestamp) |

**Note**: Types organized by business domain, not in a generic `types.rs` file.

### Result Types

| Type | Location | Purpose | Usage |
|------|----------|---------|-------|
| `RepoRollerResult<T>` | `repo_roller_core/src/errors.rs` | Top-level result type | All public APIs |
| `RepositoryResult<T>` | `repo_roller_core/src/errors.rs` | Repository operations | Repository domain |
| `ConfigurationResult<T>` | `config_manager/src/lib.rs` | Configuration operations | Config domain |
| `TemplateResult<T>` | `template_engine/src/lib.rs` | Template processing | Template domain |
| `AuthenticationResult<T>` | `auth_handler/src/lib.rs` | Auth operations | Auth domain |
| `GitHubResult<T>` | `github_client/src/lib.rs` | GitHub API calls | GitHub integration |

## Error Types

### Error Hierarchy

| Error Type | Location | Purpose | Spec Reference |
|------------|----------|---------|----------------|
| `RepoRollerError` | `repo_roller_core/src/errors.rs` | Top-level error enum | [error-types.md](error-types.md#reporollererror) |
| `ValidationError` | `repo_roller_core/src/errors.rs` | Input validation failures | [error-types.md](error-types.md#validationerror) |
| `RepositoryError` | `repo_roller_core/src/errors.rs` | Repository operation errors | [error-types.md](error-types.md#repositoryerror) |
| `ConfigurationError` | `config_manager/src/errors.rs` | Configuration errors | [error-types.md](error-types.md#configurationerror) |
| `TemplateError` | `template_engine/src/lib.rs` | Template processing errors | [error-types.md](error-types.md#templateerror) |
| `AuthenticationError` | `auth_handler/src/lib.rs` | Auth/authz errors | [error-types.md](error-types.md#authenticationerror) |
| `GitHubError` | `github_client/src/errors.rs` | GitHub API errors | [error-types.md](error-types.md#githuberror) |
| `SystemError` | `repo_roller_core/src/errors.rs` | System/infrastructure errors | [error-types.md](error-types.md#systemerror) |

**New Error Variants (Task 1.0)**:
- `ConfigurationError::AmbiguousMetadataRepository` - Multiple metadata repos found with same topic

## Business Logic Interfaces

### Repository Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `RepositoryCreationOrchestrator` | `repo_roller_core/src/repository.rs` | Repository creation workflow | [repository-domain.md](repository-domain.md) |
| `CreateRepositoryRequest` | `repo_roller_core/src/repository.rs` | Repository creation request | [repository-domain.md](repository-domain.md) |
| `Repository` | `repo_roller_core/src/repository.rs` | Repository entity | [repository-domain.md](repository-domain.md) |

### Configuration Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `ConfigurationManager` | `config_manager/src/lib.rs` | Hierarchical config resolution | [configuration-interfaces.md](configuration-interfaces.md) |
| `OrganizationConfigurationProvider` | `config_manager/src/lib.rs` | Org config access | [configuration-interfaces.md](configuration-interfaces.md) |
| `ConfigurationPolicyValidator` | `config_manager/src/lib.rs` | Override policy enforcement | [configuration-interfaces.md](configuration-interfaces.md) |
| `MetadataRepositoryProvider` | `config_manager/src/github_metadata_provider.rs` | Metadata repository discovery and access | [organization-repository-settings.md](../design/organization-repository-settings.md) |

### Template Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `TemplateEngine` | `template_engine/src/lib.rs` | Template variable substitution | [template-interfaces.md](template-interfaces.md) |
| `TemplateSource` | `template_engine/src/lib.rs` | Template content retrieval | [template-interfaces.md](template-interfaces.md) |
| `TemplateProcessor` | `template_engine/src/lib.rs` | Template processing orchestration | [template-interfaces.md](template-interfaces.md) |

### Authentication Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `UserAuthenticationService` | `auth_handler/src/lib.rs` | User authentication | [authentication-interfaces.md](authentication-interfaces.md) |
| `OrganizationPermissionService` | `auth_handler/src/lib.rs` | Permission resolution | [authentication-interfaces.md](authentication-interfaces.md) |
| `AuthenticationContext` | `auth_handler/src/lib.rs` | Auth context carrier | [authentication-interfaces.md](authentication-interfaces.md) |

## External Integration Interfaces

### GitHub Integration

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `RepositoryProvider` | `github_client/src/lib.rs` | GitHub repo operations | [github-interfaces.md](github-interfaces.md) |
| `RepositoryClient` (existing) | `github_client/src/lib.rs` | GitHub API client | Currently implemented |
| `GitHubClient` | `github_client/src/lib.rs` | Concrete GitHub implementation | Currently implemented |

**New Methods (Task 1.0)**:
- `search_repositories_by_topic(org: &str, topic: &str)` - Search for repositories by topic within organization ([github-repository-search.md](interfaces/github-repository-search.md))

**New Methods (Task 2.0)**:
- `list_directory_contents(owner: &str, repo: &str, path: &str, branch: &str)` - List contents of repository directory ([github-directory-listing.md](interfaces/github-directory-listing.md))

**New Types (Task 2.0)**:
- `TreeEntry` - Directory entry with type information (`github_client/src/models.rs`)
- `EntryType` - Enum for file/dir/symlink/submodule (`github_client/src/models.rs`)

## HTTP API Types

**Note**: HTTP API types are distinct from domain types and exist only in the `repo_roller_api` crate.

### Request Types (HTTP Layer)

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `CreateRepositoryHttpRequest` | `repo_roller_api/src/models/request.rs` | HTTP request for repository creation | [api-request-types.md](interfaces/api-request-types.md) |
| `ValidateRepositoryNameRequest` | `repo_roller_api/src/models/request.rs` | Name validation request | [api-request-types.md](interfaces/api-request-types.md) |
| `PreviewConfigurationRequest` | `repo_roller_api/src/models/request.rs` | Configuration preview request | [api-request-types.md](interfaces/api-request-types.md) |

### Response Types (HTTP Layer)

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `CreateRepositoryResponse` | `repo_roller_api/src/models/response.rs` | Repository creation result | [api-response-types.md](interfaces/api-response-types.md) |
| `ListTemplatesResponse` | `repo_roller_api/src/models/response.rs` | Template listing | [api-response-types.md](interfaces/api-response-types.md) |
| `GetTemplateDetailsResponse` | `repo_roller_api/src/models/response.rs` | Template details | [api-response-types.md](interfaces/api-response-types.md) |
| `ValidateRepositoryNameResponse` | `repo_roller_api/src/models/response.rs` | Name validation result | [api-response-types.md](interfaces/api-response-types.md) |
| `PreviewConfigurationResponse` | `repo_roller_api/src/models/response.rs` | Configuration preview | [api-response-types.md](interfaces/api-response-types.md) |
| `ErrorResponse` | `repo_roller_api/src/errors.rs` | Standard error response | [api-error-handling.md](interfaces/api-error-handling.md) |

**Translation Pattern**: HTTP types converted to domain types at API boundary.

## Value Objects and DTOs

### Domain Request Objects

| Type | Location | Purpose | Contains |
|------|----------|---------|----------|
| `RepositoryCreationRequest` | `repo_roller_core/src/repository.rs` | Domain repo creation input | Name, owner, template, variables (branded types) |
| `TemplateProcessingRequest` | `template_engine/src/lib.rs` | Template processing input | Variables, configs |

### Domain Response Objects

| Type | Location | Purpose | Contains |
|------|----------|---------|----------|
| `Repository` | `repo_roller_core/src/repository.rs` | Created repository info | URL, metadata, settings |
| `ProcessedTemplate` | `template_engine/src/lib.rs` | Processed template output | Files with substitutions |

## Configuration Types

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `TemplateConfig` | `config_manager/src/lib.rs` | Template configuration | [configuration-interfaces.md](configuration-interfaces.md) |
| `RepositorySettings` | `config_manager/src/lib.rs` | Repository settings | [configuration-interfaces.md](configuration-interfaces.md) |
| `VariableConfig` | `template_engine/src/lib.rs` | Variable validation rules | [template-interfaces.md](template-interfaces.md) |

## Patterns and Conventions

### Async Trait Pattern

All interface traits that perform I/O use `async_trait`:

```rust
#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    async fn create_repository(&self, request: CreateRepositoryRequest)
        -> RepositoryResult<Repository>;
}
```

**Location**: All interface traits
**Dependencies**: `async_trait` crate

### Builder Pattern

Complex types with many optional fields use builders:

```rust
CreateRepositoryRequest::builder()
    .name(repo_name)
    .owner(org_name)
    .template(template_name)
    .build()?
```

**Status**: Not yet implemented (TODO)
**Candidates**: `CreateRepositoryRequest`, `RepositorySettings`

### Newtype Pattern

All domain primitives wrapped for type safety:

```rust
pub struct RepositoryName(String);
```

**Location**: `repo_roller_core/src/types.rs`
**See**: [Shared Types](shared-types.md)

## Cross-Cutting Concerns

### Logging and Tracing

All operations use structured logging with `tracing`:

```rust
use tracing::{info, debug, error, instrument};

#[instrument(skip(self), fields(org = %org, name = %name))]
async fn create_repository(&self, org: OrganizationName, name: RepositoryName) {
    info!("Creating repository");
    debug!("Repository details: {:?}", details);
}
```

**Location**: All modules
**Dependencies**: `tracing` crate

### Serialization

Types that cross API boundaries implement `Serialize`/`Deserialize`:

```rust
#[derive(Serialize, Deserialize)]
pub struct CreateRepositoryRequest { }
```

**Location**: API-facing types
**Dependencies**: `serde` crate

## Dependency Graph

### Business Logic Dependencies

```
repo_roller_core
├── types.rs (no dependencies, just std + serde)
├── errors.rs (depends on: types.rs, thiserror)
├── repository.rs (depends on: types.rs, errors.rs, interface traits)
└── [other modules]
```

### External Integration Dependencies

```
github_client
├── implements RepositoryProvider trait
├── depends on: repo_roller_core types & errors
└── external: octocrab, tokio

template_engine
├── implements TemplateEngine trait
├── depends on: repo_roller_core types & errors
└── external: handlebars, regex

config_manager
├── implements ConfigurationManager trait
├── depends on: repo_roller_core types & errors
└── external: toml, serde

auth_handler
├── implements UserAuthenticationService trait
├── depends on: repo_roller_core types & errors
└── external: jsonwebtoken, chrono
```

## Testing Utilities

### Test Builders

**Status**: TODO - Not yet implemented

```rust
// Planned test utilities
pub mod test_utils {
    pub fn repository_name(name: &str) -> RepositoryName { }
    pub fn create_repo_request() -> CreateRepositoryRequest { }
    pub fn mock_github_client() -> MockRepositoryProvider { }
}
```

**Location**: `repo_roller_core/src/test_utils.rs` (planned)

### Mock Implementations

**Status**: TODO - Not yet implemented

Mock implementations for all interface traits to enable testing.

## Usage Guidelines

### When to Create a New Type

Create a new branded type when:

- Type represents a distinct domain concept
- Mixing with other types would be a logical error
- Type has specific validation rules
- Type needs custom formatting or parsing

### When to Use Existing Types

Use existing types when:

- Concept already exists in registry
- Type is a simple composition of existing types
- No domain-specific validation needed

### When to Create a New Trait

Create new interface trait when:

- Introducing a new external dependency
- Need to swap implementations (testing, different environments)
- Enforcing architectural boundary between layers

## Maintenance

### Adding a New Type

1. Define the type in appropriate crate
2. Add documentation and spec reference
3. Update this registry
4. Add to relevant specification document
5. Ensure type compiles and has basic tests

### Deprecating a Type

1. Mark as `#[deprecated]` in code
2. Document replacement in this registry
3. Update specification documents
4. Create migration guide if needed

### Refactoring

When refactoring existing code to use new types:

- Add TODO comments to mark conversion points
- Don't break existing functionality
- Create migration wrappers if needed
- Update registry when complete

## Quick Reference

### Most Commonly Used Types

1. `RepositoryName` - Almost every operation
2. `OrganizationName` - Repository creation, config resolution
3. `TemplateName` - Template operations
4. `RepoRollerResult<T>` - All public functions
5. `CreateRepositoryRequest` - Repository creation workflow

### Most Important Traits

1. `RepositoryProvider` - GitHub integration boundary
2. `ConfigurationManager` - Configuration system boundary
3. `TemplateEngine` - Template processing boundary
4. `UserAuthenticationService` - Authentication boundary

---

**Last Updated**: 2025-10-10
**Maintainer**: Interface Designer
**Status**: Initial version with existing code cataloged and new interfaces specified

**Next Actions**:

- Implement types in `repo_roller_core/src/types.rs`
- Implement errors in `repo_roller_core/src/errors.rs`
- Add TODO comments to existing code for migration
- Update as new types are added
