# Implementation Constraints

This document defines the architectural rules, constraints, and policies that must be enforced throughout the RepoRoller implementation. These constraints ensure system reliability, maintainability, security, and performance.

## Type System Constraints

### Branded Types Requirement

**Constraint**: All domain primitives must use branded types (newtype pattern) to prevent type confusion.

**Rationale**: Prevents accidental mixing of conceptually different values that have the same underlying type.

**Enforcement**:

- Repository names, organization names, template types must be distinct types
- User IDs, session IDs, and other identifiers must be strongly typed
- No `String` or `u64` parameters in public domain interfaces

**Examples of Required Branded Types**:

- `RepositoryName`, `OrganizationName`, `TemplateType`
- `UserId`, `SessionId`, `InstallationId`
- `GitHubToken`, `ConfigurationKey`
- `RepositoryVisibility` (enum type for type-safe visibility)

### Result Type Requirement

**Constraint**: All fallible operations must use `Result<T, E>` pattern rather than exceptions.

**Rationale**: Makes error handling explicit and prevents unexpected panics.

**Enforcement**:

- No `panic!`, `unwrap()`, or `expect()` in production code paths
- All public functions returning fallible results must use `Result<T, E>`
- Custom error types must implement appropriate traits (`Error`, `Display`, `Debug`)

### No Any/Dynamic Types

**Constraint**: No use of `Any` trait or dynamic typing in domain code.

**Rationale**: Maintains type safety and compile-time verification.

**Enforcement**:

- All template variables must be strongly typed through `serde_json::Value`
- Configuration values must have known types at compile time
- Interface boundaries must use concrete types

## Module Boundary Constraints

### Dependency Direction Rules

**Constraint**: Dependency flow must follow clean separation principles.

### HTTP API Boundary Rules

**Constraint**: HTTP API types must never leak into business logic.

**HTTP API Layer** (`repo_roller_api` crate):

- Defines HTTP request and response types
- Handles HTTP-specific concerns (headers, status codes, serialization)
- Translates between HTTP types and domain types
- Never used by business logic

**Translation Requirements**:

- HTTP request types converted to domain types at API boundary
- Domain result types converted to HTTP responses at API boundary
- Validation during translation produces domain errors
- Domain layer never imports `repo_roller_api` types

**Example Pattern**:

```rust
// HTTP Layer
pub struct CreateRepositoryHttpRequest { /* HTTP-specific fields */ }

// Domain Layer (repo_roller_core)
pub struct RepositoryCreationRequest { /* domain types */ }

// Translation (in HTTP layer only)
impl TryFrom<CreateRepositoryHttpRequest> for RepositoryCreationRequest {
    type Error = ValidationError;
    fn try_from(req: CreateRepositoryHttpRequest) -> Result<Self, Self::Error> {
        // Validate and convert
    }
}
```

**Business Logic Dependencies**:

- Business logic depends only on standard library and essential crates (`serde`, `chrono`, `uuid`)
- Business logic never imports external system integrations or user interface modules
- Application services can depend on business logic and define abstract interfaces
- External system integrations implement interfaces but never import other integrations

**User Interface Dependencies**:

- User interfaces depend on business logic through application services
- Multiple user interface implementations (CLI, Web Service, MCP Server) can coexist
- User interfaces handle protocol-specific concerns only

### Interface Pattern Enforcement

**Constraint**: All external system interactions must go through defined interfaces following hexagonal architecture principles without exposing architectural terminology in naming.

**Interface Requirements**:

- All external dependencies represented as traits (interfaces) using business domain names
- Traits define only abstract behavior, no implementation details
- Implementation (concrete types) lives in external system integration modules
- No architectural terminology (ports, adapters, domain, infrastructure) in public interfaces

**Examples of Required Interfaces**:

- `RepositoryProvider` trait for all GitHub repository operations
- `OrganizationConfigurationProvider` trait for configuration persistence
- `ComplianceAuditService` trait for audit trail recording
- `TemplateEngine` trait for template processing

**Business Domain Naming Convention**:

- Use business concepts: `UserAuthenticationService`, `TemplateSource`, `CredentialVault`
- Avoid architectural terms: `UserPort`, `TemplateAdapter`, `DomainService`, `InfrastructureClient`
- Focus on business capability: what the interface provides from a user/business perspective

### Import Restrictions

**Constraint**: Strict import rules prevent architectural violations.

**Business Logic Imports**:

- May import: `std`, `serde`, `chrono`, `uuid`, `thiserror`
- May not import: HTTP clients, database drivers, external system clients

**Application Service Imports**:

- May import: Business logic, async runtime (`tokio`), interface trait definitions
- May not import: Specific external system implementations

**External System Integration Imports**:

- May import: Application service interface traits, external service clients
- May not import: Other external system integration implementations

## Error Handling Constraints

### Error Type Hierarchy

**Constraint**: Errors must follow established hierarchy with proper context preservation.

**Hierarchy Structure**:

- Top-level: `RepoRollerError` enum with variant for each domain
- Domain-specific: `ValidationError`, `ProcessingError`, `ConfigurationError`, `SystemError`
- Context preservation: All errors include relevant context for debugging
- Conversion: Automatic conversion through `From` trait implementations

### Error Context Requirements

**Constraint**: All errors must include sufficient context for debugging and user guidance.

**Required Context**:

- Operation being performed when error occurred
- Input values that caused the error (sanitized)
- Suggestions for resolution where applicable
- Trace information for system errors

### No Error Swallowing

**Constraint**: Errors must never be silently ignored or converted to default values.

**Enforcement**:

- All `Result` values must be explicitly handled
- Error conversion must preserve original error information
- Logging of errors at appropriate levels before handling

## Asynchronous Programming Constraints

### Async Boundary Definition

**Constraint**: Clear separation between sync and async code with explicit boundaries.

**Async Operations**:

- I/O operations: GitHub API calls, file system access, network requests
- Resource acquisition: Authentication token retrieval, configuration loading
- Coordination: Cross-service communication, cache operations

**Sync Operations**:

- Pure computation: Template rendering, configuration merging
- Domain logic: Business rule validation, state transitions
- Data transformation: Serialization, format conversion

### Timeout Requirements

**Constraint**: All async operations must have explicit timeouts.

**Timeout Categories**:

- Quick operations (< 5 seconds): Token validation, cache lookups
- Standard operations (< 30 seconds): GitHub API calls, configuration loading
- Long operations (< 2 minutes): Template processing, repository creation

**Implementation**:

- Timeout values configurable through application configuration
- Graceful timeout handling with appropriate error messages
- Monitoring and alerting on timeout frequencies

## Configuration Constraints

### Hierarchical Configuration Rules

**Constraint**: Configuration resolution must follow strict precedence and override rules.

**Precedence Order** (lowest to highest):

1. Encoded defaults (cannot be overridden)
2. Application configuration (global settings)
3. Organization configuration (per-organization)
4. Team configuration (per-team within organization)
5. Template configuration (highest precedence)

**Override Policy Enforcement**:

- Security-critical settings cannot be overridden regardless of level
- Override permissions explicitly defined in configuration schema
- Violations result in configuration resolution failure

### Configuration Validation

**Constraint**: All configuration must be validated before use.

**Validation Requirements**:

- Schema validation: Structure and type correctness
- Business rule validation: Cross-field constraints and policies
- Security validation: No dangerous or prohibited values
- Completeness validation: All required fields present

### Runtime Configuration Loading

**Constraint**: Configuration must support runtime loading with appropriate fallbacks.

**Loading Strategy**:

- Encoded defaults always available (compiled into binary)
- File-based configuration with graceful degradation if unavailable
- Environment variable overrides for deployment-specific settings
- Cache with TTL and invalidation for performance

### Single Organization Scope

**Constraint**: Each RepoRoller instance serves exactly one GitHub organization.

**Organizational Boundaries**:

- GitHub App installation scoped to single organization
- Configuration system assumes single organization context
- User authentication and authorization within organization scope only
- No cross-organization data leakage or configuration mixing
- Organization-specific deployment and monitoring

## Security Constraints

### Input Validation Requirements

**Constraint**: All external input must be validated before processing.

**Validation Scope**:

- User-provided data: Repository names, template variables, configuration values
- External API responses: GitHub API data, webhook payloads
- File content: Template files, configuration files, user uploads

**Validation Rules**:

- Format validation: Data type, length, character set constraints
- Business rule validation: Domain-specific constraints and policies
- Security validation: No injection attacks, path traversal prevention
- Sanitization: Remove or escape dangerous content

### Authentication Requirements

**Constraint**: All operations must occur within authenticated context.

**Authentication Context**:

- User identity: GitHub user information and organization memberships
- Permission scope: What operations user is authorized to perform
- Token lifecycle: Proper token validation and refresh handling
- Session management: Secure session handling for web interfaces

### Audit Trail Requirements

**Constraint**: All significant operations must be logged for audit purposes.

**Audit Coverage**:

- Authentication events: Login attempts, token validation, permission checks
- Repository operations: Creation requests, configuration applied, errors encountered
- Configuration changes: What changed, who changed it, when it changed
- Security events: Failed authentication, permission violations, suspicious activity

## Performance Constraints

### Response Time Requirements

**Constraint**: Operations must complete within acceptable time boundaries.

**Performance Targets**:

- Authentication: < 200ms for token validation
- Configuration resolution: < 500ms for cached configurations
- Template processing: < 30 seconds for standard templates
- Repository creation: < 2 minutes end-to-end

### Concurrency Requirements

**Constraint**: System must handle concurrent operations safely and efficiently.

**Concurrency Safety**:

- No shared mutable state without proper synchronization
- Thread-safe caching mechanisms with consistent reads
- Database/API operations must handle concurrent access
- Configuration loading must support multiple simultaneous requests

### Resource Usage Limits

**Constraint**: Operations must respect resource consumption limits.

**Resource Limits**:

- Memory usage: Template processing < 100MB per operation
- File processing: Template files < 50MB total size
- API rate limits: Respect GitHub API rate limits with intelligent backoff
- Cache size: Configuration cache with LRU eviction and size limits

## Testing Constraints

### Test Coverage Requirements

**Constraint**: Comprehensive test coverage across all architectural layers.

**Coverage Requirements**:

- Unit tests: 90%+ coverage of domain logic
- Integration tests: All interface implementations tested against real services
- Contract tests: Port interface contracts verified
- End-to-end tests: Complete workflows from interface to external services

### Test Isolation Requirements

**Constraint**: Tests must be isolated and repeatable.

**Isolation Rules**:

- No shared state between test cases
- Mock all external dependencies in unit tests
- Integration tests use dedicated test environments
- Test data cleanup after test execution

### Test Double Strategy

**Constraint**: Consistent approach to test doubles across the system.

**Test Double Types**:

- Mocks: For behavior verification of port interactions
- Stubs: For providing predetermined responses from dependencies
- Fakes: For lightweight implementations during testing
- Test builders: For creating test data with sensible defaults

## Documentation Constraints

### API Documentation Requirements

**Constraint**: All public interfaces must be thoroughly documented.

**Documentation Scope**:

- Public traits and their implementations
- Error conditions and recovery strategies
- Configuration options and their effects
- Usage examples for common scenarios

### Architectural Decision Recording

**Constraint**: Significant architectural decisions must be documented.

**Documentation Requirements**:

- Decision context and alternatives considered
- Rationale for chosen approach
- Expected consequences and trade-offs
- Success criteria and monitoring approach

## Deployment Constraints

### Environment Parity

**Constraint**: Code must work consistently across all deployment environments.

**Environment Requirements**:

- No hardcoded environment-specific values
- Configuration externalized for different environments
- Feature flags for environment-specific behavior
- Consistent behavior regardless of deployment method

### Observability Requirements

**Constraint**: System must provide comprehensive observability.

**Observability Coverage**:

- Structured logging with consistent fields
- Metrics for business operations and system health
- Distributed tracing for request flow
- Health check endpoints for monitoring systems

## Release Management Constraints

### Versioning Requirements

**Constraint**: All version changes must follow semantic versioning (semver 2.0).

**Version Rules**:

- **MAJOR** version: Breaking changes (incompatible API changes)
- **MINOR** version: New features (backward-compatible additions)
- **PATCH** version: Bug fixes (backward-compatible fixes)
- Pre-release versions: `0.x.x` before first stable release

**Enforcement**:

- Automated version calculation from conventional commits
- Manual overrides must justify version choice
- Pre-1.0 releases signal API instability

### Commit Message Requirements

**Constraint**: All commits to master must follow conventional commits format.

**Format**:

```
<type>(<scope>): <description>

[optional body]

[optional footer with BREAKING CHANGE]
```

**Required Types**:

- `feat`: New feature (triggers MINOR version)
- `fix`: Bug fix (triggers PATCH version)
- `docs`: Documentation changes
- `style`: Code style/formatting
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions/changes
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Maintenance tasks

**Breaking Changes**:

- Mark with `!` after type/scope: `feat!:` or `feat(api)!:`
- Or include `BREAKING CHANGE:` in footer
- Triggers MAJOR version bump

**Enforcement**:

- PR title must follow format (enforced by CI)
- Squash merge recommended for clean history
- Changelog generated from commit messages

### Release Artifact Requirements

**Constraint**: All releases must include verified, reproducible artifacts.

**Required Artifacts**:

- **Container Image**: `ghcr.io/pvandervelde/repo_roller_api:v{version}`
- **CLI Binaries**: Linux, Windows, macOS (x86_64)
- **Checksums**: SHA256 for all downloadable artifacts
- **Release Notes**: Auto-generated changelog from commits

**Artifact Properties**:

- Built in GitHub Actions (reproducible builds)
- Tagged with exact version and `latest` (if stable)
- Vulnerability scanned before publication
- Size optimized (stripped binaries, multi-stage Docker)

### Release Process Requirements

**Constraint**: Release process must be automated and auditable.

**Automated Steps**:

1. Version calculation from conventional commits
2. Changelog generation from commit history
3. Release PR creation/update
4. Artifact building and publishing after PR merge
5. GitHub release creation with notes

**Manual Steps** (minimal):

- Release PR review and approval
- Optional version override via comment
- Emergency rollback if critical issues

**Audit Trail**:

- All releases tracked as Git tags
- Release PRs provide diff and changelog
- Artifact publishing logged
- Release metrics tracked

See [specs/interfaces/release-automation.md](./interfaces/release-automation.md) for complete release automation specification.

## Repository Visibility Constraints

### Visibility Policy Enforcement

**Constraint**: Organization visibility policies must be enforced at all times, with no bypass mechanism.

**Rationale**: Repository visibility is a security-critical setting that affects organization data exposure.

**Enforcement**:

- Visibility resolution must occur before any GitHub API calls
- Policy violations must prevent repository creation entirely
- Cache invalidation must not allow stale policies to be used
- All visibility decisions must be logged with full audit trail

### Visibility Hierarchy

**Constraint**: Visibility determination must follow strict precedence hierarchy.

**Decision Order** (highest to lowest priority):

1. Organization Policy (required) - Mandatory visibility enforced
2. User Preference - Explicit user choice (if allowed by policy)
3. Template Default - Template's configured visibility
4. System Default - Fallback (always Private)

**Enforcement**:

- Each level validated against organization policy before acceptance
- GitHub platform constraints checked after policy validation
- Decision source documented in all visibility decisions

### GitHub Enterprise Detection

**Constraint**: Internal repository visibility must only be available in GitHub Enterprise environments.

**Rationale**: Internal visibility is a GitHub Enterprise-only feature.

**Enforcement**:

- Environment detection must occur during visibility resolution
- Attempts to create internal repositories on github.com must fail with clear error
- Detection results must be cached (1 hour) to reduce API calls
- Detection failures must fall back safely (treat as non-Enterprise)

### Visibility Configuration Format

**Constraint**: Organization visibility configuration must follow standardized format.

**Required Fields**:

- `enforcement_level`: "required", "restricted", or "unrestricted"
- `required_visibility`: Only when enforcement_level = "required" (values: "public", "private", "internal")
- `restricted_visibilities`: Only when enforcement_level = "restricted" (array of prohibited values)
- `default_visibility`: System default when no preference specified

**Validation**:

- Invalid enforcement_level must cause configuration error
- required_visibility without "required" enforcement_level must warn
- restricted_visibilities without "restricted" enforcement_level must warn
- Empty restricted_visibilities with "restricted" enforcement must error
- default_visibility must be valid visibility value

### Visibility Caching

**Constraint**: Visibility-related data must be cached with appropriate TTLs.

**Cache Durations**:

- Organization policies: 5 minutes
- GitHub environment detection: 1 hour
- Plan limitations: 1 hour

**Cache Behavior**:

- Cache misses must fetch fresh data
- Cache hits must check expiration before use
- Cache invalidation must be explicit (no automatic expiration bypass)
- Concurrent access must be thread-safe (Arc<RwLock<>>)

### Visibility Error Messages

**Constraint**: Visibility errors must provide clear, actionable guidance.

**Error Message Requirements**:

- PolicyNotFound: "Contact organization admin to configure visibility policy"
- PolicyViolation: Explain what policy requires/prohibits
- GitHubConstraint: Explain plan limitation and upgrade path
- ConfigurationError: Identify invalid configuration and correction
- GitHubApiError: Include original error with context

**No Sensitive Data**:

- Error messages must not expose internal configuration details
- Policy details kept generic in user-facing errors
- Detailed policy info logged but not returned to user

These constraints provide the framework within which all RepoRoller components must be implemented. They ensure architectural consistency, system reliability, and maintainability while supporting the complex requirements of the repository automation domain.
