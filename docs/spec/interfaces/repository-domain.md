# Repository Domain Interfaces

**Architectural Layer**: Core Business Logic
**Crate**: `repo_roller_core`
**Responsibilities** (from RDD):

- **Knows**: Repository creation rules, validation logic, workflow steps
- **Does**: Orchestrates repository creation, coordinates services

---

## Overview

This document defines the interfaces for the repository creation domain - the central business logic of RepoRoller. This is the core orchestration that coordinates template fetching, file processing, Git operations, and GitHub API interactions.

## Dependencies

### Types Used

- `RepositoryName` - from `repo_roller_core::repository`
- `OrganizationName` - from `repo_roller_core::repository`
- `TemplateName` - from `repo_roller_core::template`
- `GitHubToken` - from `repo_roller_core::github`
- `InstallationId` - from `repo_roller_core::github`
- `Timestamp` - from `repo_roller_core`

### Interface Traits Used

- `TemplateFetcher` - from `template_engine`
- `RepositoryClient` - from `github_client`
- `ConfigurationManager` - from `config_manager` (TODO: define)
- `UserAuthenticationService` - from `auth_handler`

---

## Core Types

### RepositoryCreationRequest

The input for repository creation operations.

```rust
pub struct RepositoryCreationRequest {
    pub name: RepositoryName,
    pub owner: OrganizationName,
    pub template: TemplateName,
    pub variables: HashMap<String, String>, // User-provided template variables
}
```

**TODO**: Current code uses `CreateRepoRequest` with raw `String` fields - migrate to this typed version.

### RepositoryCreationResult

The output of repository creation operations.

```rust
pub struct RepositoryCreationResult {
    pub repository_url: String,
    pub repository_id: String,
    pub created_at: Timestamp,
    pub default_branch: String,
}
```

**TODO**: Current code uses `CreateRepoResult { success: bool, message: String }` - migrate to Result<RepositoryCreationResult, RepoRollerError>.

---

## Main Orchestration Function

### create_repository_with_config

**Current Signature** (to be refactored):

```rust
pub async fn create_repository_with_config(
    request: CreateRepoRequest,
    config: &config_manager::Config,
    app_id: u64,
    app_key: String,
) -> CreateRepoResult
```

**Target Signature** (after refactoring):

```rust
pub async fn create_repository(
    request: RepositoryCreationRequest,
    config: &dyn ConfigurationManager,
    auth_service: &dyn UserAuthenticationService,
) -> Result<RepositoryCreationResult, RepoRollerError>
```

**Changes Required**:

1. Use branded types (RepositoryName, OrganizationName, TemplateName)
2. Accept trait objects instead of concrete implementations
3. Return Result instead of custom result type
4. Remove direct app_id/app_key parameters - use auth service

**Workflow** (remains the same):

1. Resolve template configuration
2. Fetch template files
3. Create temporary local directory
4. Copy template files
5. Process template variables
6. Create additional files (README, .gitignore)
7. Initialize Git repository
8. Commit all changes
9. Create GitHub repository
10. Push to remote
11. Configure repository settings

**Current Location**: `repo_roller_core/src/lib.rs::create_repository_with_config`

**TODO Marker Added**: See line 760 in lib.rs

---

## Supporting Functions

These internal functions support the main orchestration:

### copy_template_files

```rust
fn copy_template_files(
    files: &Vec<(String, Vec<u8>)>,
    local_repo_path: &TempDir,
) -> Result<(), Error>
```

**Status**: ✅ Already well-designed, no changes needed

### create_additional_files

```rust
fn create_additional_files(
    local_repo_path: &TempDir,
    req: &CreateRepoRequest,
    template_files: &[(String, Vec<u8>)],
) -> Result<(), Error>
```

**TODO**: Change `req` parameter to use `RepositoryCreationRequest` with branded types

### commit_all_changes

```rust
fn commit_all_changes(
    local_repo_path: &TempDir,
    commit_message: &str,
) -> Result<(), Error>
```

**Status**: ✅ Well-designed, no changes needed

### init_local_git_repo

```rust
fn init_local_git_repo(
    local_path: &TempDir,
    default_branch: &str,
) -> Result<(), Error>
```

**Status**: ✅ Well-designed, no changes needed

### push_to_origin

```rust
fn push_to_origin(
    local_repo_path: &TempDir,
    repo_url: url::Url,
    branch_name: &str,
    access_token: &str,
) -> Result<(), Error>
```

**TODO**: Change `access_token` parameter to `GitHubToken` type

### replace_template_variables

```rust
fn replace_template_variables(
    local_repo_path: &TempDir,
    req: &CreateRepoRequest,
    template: &config_manager::TemplateConfig,
) -> Result<(), Error>
```

**TODO**:

- Change `req` to use `RepositoryCreationRequest`
- Change `template` to use new configuration interfaces

---

## Organization Rules

### OrgRules

**Current Implementation**:

```rust
pub struct OrgRules {
    pub repo_name_regex: Option<String>,
}

impl OrgRules {
    pub fn new_from_text(org: &str) -> OrgRules { ... }
}
```

**TODO**: This should be moved to a configuration service or organization permission service:

- Not core business logic responsibility
- Should be loaded from configuration or GitHub org settings
- Consider making this an interface trait that infrastructure implements

---

## Error Handling

All functions return `Result<T, Error>` where `Error` is defined in `repo_roller_core::errors`.

**TODO**: Migrate to comprehensive error hierarchy from `specs/interfaces/error-types.md`

---

## Migration Path

### Phase 1: Type Safety

1. Introduce branded types alongside existing String parameters
2. Add conversion functions
3. Update internal code to use branded types
4. Keep public API backward compatible with deprecation warnings

### Phase 2: Dependency Injection

1. Extract authentication logic to use `auth_handler` traits
2. Accept trait objects instead of concrete types
3. Add integration tests with mock implementations

### Phase 3: Result Types

1. Define `RepositoryCreationResult` type
2. Migrate from `CreateRepoResult` to `Result<RepositoryCreationResult, RepoRollerError>`
3. Update all call sites

### Phase 4: Configuration

1. Define `ConfigurationManager` trait
2. Migrate from direct `config_manager::Config` dependency
3. Enable dependency injection for testability

---

## Testing Requirements

### Unit Tests

- Mock all external dependencies (GitHub, template engine, config)
- Test orchestration logic in isolation
- Test error handling and edge cases

### Integration Tests

- Test complete workflow with real GitHub test organization
- Test template processing end-to-end
- Test failure recovery and error messages

### Contract Tests

- Verify all interface trait methods are called correctly
- Test behavior with different implementations

---

## Related Specifications

- `specs/interfaces/shared-types.md` - Type definitions
- `specs/interfaces/error-types.md` - Error hierarchy
- `specs/interfaces/github-interfaces.md` - GitHub integration
- `specs/interfaces/template-interfaces.md` - Template processing
- `specs/interfaces/configuration-interfaces.md` - Configuration management
- `specs/interfaces/authentication-interfaces.md` - Authentication services
