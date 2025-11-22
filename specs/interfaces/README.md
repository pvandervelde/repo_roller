# RepoRoller Interface Specifications

This directory contains comprehensive interface specifications for all RepoRoller components. These specifications define the contracts, types, and behaviors that implementations must fulfill.

## Purpose

Interface specifications serve as:

- **Design documentation** - Complete type and trait definitions
- **Implementation contracts** - Requirements that code must satisfy
- **Architectural boundaries** - Clear separation between business logic and infrastructure
- **Testing specifications** - Contract tests validate implementations

## Navigation

### Core Type Specifications

- [**Shared Types**](shared-types.md) - Core shared types and branded domain primitives
- [**Error Types**](error-types.md) - Error hierarchy and error handling patterns

### Domain Specifications

- [**Repository Domain**](repository-domain.md) - Repository creation orchestration logic
- [**Configuration Domain**](configuration-interfaces.md) - Template and settings management
- [**Template Processing**](template-interfaces.md) - Template fetching and variable substitution

### External Integration Specifications

- [**GitHub Integration**](github-interfaces.md) - GitHub API boundaries
- [**Authentication Services**](authentication-interfaces.md) - Auth and authorization interfaces
- [**External Systems Overview**](external-systems.md) - High-level system boundaries

### HTTP API Specifications

- [**API Request Types**](api-request-types.md) - HTTP request models and validation
- [**API Response Types**](api-response-types.md) - HTTP response models and serialization
- [**API Error Handling**](api-error-handling.md) - Error mapping and HTTP status codes

**Status**: ✅ 11 interface specifications complete (3 HTTP API specs added)

## Architecture Overview

RepoRoller follows clean architecture principles with clear boundaries:

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interfaces                          │
│  (CLI, API, MCP Server, Azure Function, Web UI)            │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  Business Logic Layer                       │
│  • Repository Creation Orchestration                        │
│  • Configuration Resolution                                 │
│  • Template Processing Logic                                │
│  • Depends ONLY on interface traits (never infrastructure)  │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│              Interface Traits (Boundaries)                  │
│  • RepositoryProvider                                       │
│  • TemplateSource                                           │
│  • ConfigurationManager                                     │
│  • UserAuthenticationService                                │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│            Infrastructure Implementations                   │
│  • GitHubClient (implements RepositoryProvider)            │
│  • GitHubTemplateFetcher (implements TemplateSource)       │
│  • ConfigLoader (implements ConfigurationManager)          │
└─────────────────────────────────────────────────────────────┘
```

## Dependency Rules

### ✅ Allowed Dependencies

**Business Logic** may depend on:

- Standard library types
- Interface trait definitions
- Shared domain types (branded types)
- Other business logic modules

**HTTP API Layer** may depend on:

- Interface traits
- Business domain types (for translation)
- HTTP framework types (Axum, Tokio)
- Business logic modules (calls them)

**Infrastructure** may depend on:

- Interface traits (to implement them)
- External service clients (GitHub API, Azure SDK)
- Business domain types (for implementation)

### ❌ Prohibited Dependencies

**Business Logic** must NEVER import:

- Infrastructure implementations (GitHubClient, etc.)
- External service clients directly
- Framework-specific code
- HTTP API types

**HTTP API Layer** must NEVER be imported by:

- Business logic
- Infrastructure implementations (except API-specific infra)

## Type System Conventions

### Branded Types (Newtype Pattern)

All domain primitives use the newtype pattern to prevent type confusion:

```rust
pub struct RepositoryName(String);
pub struct OrganizationName(String);
pub struct TemplateName(String);
pub struct UserId(uuid::Uuid);
```

### Result Types

All fallible operations use `Result<T, E>`:

```rust
pub type RepoRollerResult<T> = Result<T, RepoRollerError>;

// Domain-specific results
pub type RepositoryResult<T> = Result<T, RepositoryError>;
pub type ConfigurationResult<T> = Result<T, ConfigurationError>;
```

### Interface Traits

All external dependencies are represented as traits:

```rust
#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    async fn create_repository(&self, request: CreateRepositoryRequest)
        -> RepositoryResult<Repository>;
}
```

## File Organization

Each specification file follows this structure:

1. **Overview** - Purpose and responsibilities
2. **Type Definitions** - All types with complete documentation
3. **Trait Definitions** - Interface contracts with method signatures
4. **Error Conditions** - Possible errors and when they occur
5. **Usage Examples** - Pseudo-code showing typical usage
6. **Implementation Notes** - Constraints and requirements

## Reading Order for Developers

### New to the Codebase

1. Start with [Shared Types](shared-types.md) - understand core domain types
2. Read [Error Types](error-types.md) - understand error handling approach
3. Review [Repository Domain](repository-domain.md) - core business logic
4. Explore integration interfaces as needed

### Implementing a Feature

1. Identify the domain (Repository, Configuration, Template, etc.)
2. Read the relevant domain specification
3. Check [Shared Types](shared-types.md) for reusable types
4. Review error handling in [Error Types](error-types.md)
5. Check interface traits you'll need to use/implement

### Adding a New External Integration

1. Read [External System Interfaces](../external-systems.md) for patterns
2. Define interface trait in appropriate specification file
3. Document error mappings in [Error Types](error-types.md)
4. Add integration to architecture diagram above

## Validation

All specifications must satisfy:

- **Completeness** - All types, traits, and errors documented
- **Consistency** - Naming and patterns align across specifications
- **Compilability** - Source stubs based on specs must compile
- **Testability** - Specifications enable contract testing

## Source Code Mapping

Interface specifications map to source code as follows:

| Specification | Source Location |
|---------------|----------------|
| **Domain Types** | |
| Shared Types | `repo_roller_core/src/types.rs` |
| Repository Domain | `repo_roller_core/src/repository.rs` |
| Configuration | `config_manager/src/lib.rs` |
| Authentication | `auth_handler/src/lib.rs` |
| **External Integration** | |
| GitHub Integration | `github_client/src/lib.rs` |
| Template Processing | `template_engine/src/lib.rs` |
| **HTTP API** | |
| API Request Types | `repo_roller_api/src/models/request.rs` |
| API Response Types | `repo_roller_api/src/models/response.rs` |
| API Error Handling | `repo_roller_api/src/errors.rs` |

## Contributing to Specifications

When adding or updating specifications:

1. **Maintain consistency** - Follow established patterns
2. **Document thoroughly** - Every type, field, and method needs docs
3. **Provide examples** - Show typical usage patterns
4. **Link related specs** - Cross-reference related specifications
5. **Update this README** - Keep the overview current

## References

- [Architecture Overview](../architecture/system-overview.md)
- [Component Responsibilities](../responsibilities.md)
- [Domain Vocabulary](../vocabulary.md)
- [Implementation Constraints](../constraints.md)

---

*These specifications are living documents that evolve with the system. Keep them synchronized with implementation as understanding deepens.*
