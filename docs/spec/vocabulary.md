# Domain Vocabulary

This document establishes the ubiquitous language for the RepoRoller system, defining all core domain concepts and their relationships. This vocabulary guides interface design, implementation, and communication across the development team.

## Core Domain Entities

### Repository Creation Domain

**Repository**
A GitHub repository managed by the RepoRoller system.

- **Identity**: Identified by combination of organization and repository name
- **Properties**: Name, description, visibility, settings, creation metadata
- **Lifecycle**: Requested → Validated → Created → Configured → Active
- **Constraints**: Name must be unique within organization, must follow GitHub naming rules

**Template**
A source repository that serves as the foundation for creating new repositories.

- **Identity**: Identified by template type name and source repository reference
- **Properties**: Source repository, branch, processing rules, variable definitions
- **Behavior**: Can be processed to generate customized repository content
- **Constraints**: Must be accessible via GitHub API, must contain valid template configuration

**Organization**
A GitHub organization that uses RepoRoller for repository creation.

- **Identity**: GitHub organization name
- **Properties**: Configuration repository, security policies, team definitions
- **Behavior**: Provides configuration context for repository creation
- **Constraints**: Must have RepoRoller GitHub App installed

### Configuration Domain

**ConfigurationHierarchy**
The layered configuration system that determines repository settings.

- **Levels**: Encoded Defaults → App Config → Organization Config → Team Config → Template Config
- **Behavior**: Higher levels override lower levels, with override policy enforcement
- **Constraints**: Security policies cannot be overridden, audit trail required

**RepositoryType**
A classification system for repositories within an organization.

- **Identity**: Type name (e.g., "library", "service", "documentation")
- **Properties**: Default settings, constraints, required configurations
- **Behavior**: Automatically applied when repository type is specified
- **Constraints**: Must be defined in organization configuration

**Team**
A group of users within an organization with shared repository creation patterns.

- **Identity**: Team name within organization
- **Properties**: Configuration overrides, default settings, permissions
- **Behavior**: Provides team-specific defaults for repository creation
- **Constraints**: Must exist in organization's team definitions

### Authentication Domain

**AuthenticationContext**
Information about the authenticated user making a repository creation request.

- **Properties**: User identity, permissions, organization memberships, token information
- **Behavior**: Used for authorization and audit trail
- **Lifecycle**: Created during authentication → Validated → Used for authorization
- **Constraints**: Must be valid GitHub user with appropriate permissions

**WebSession**
Browser-based user session for web interface interactions.

- **Properties**: Session ID, user authentication context, CSRF tokens, expiration time
- **Behavior**: Maintains user state across web requests and provides security context
- **Lifecycle**: Created on login → Active during use → Expires or logout
- **Constraints**: Secure session management with timeout and CSRF protection

**Permission**
Authorization to perform specific operations within RepoRoller.

- **Types**: Repository creation, template access, configuration override, administrative
- **Scope**: Organization-level, team-level, repository-type-level
- **Behavior**: Hierarchical evaluation with precedence rules
- **Constraints**: Cannot exceed GitHub API permissions

### Processing Domain

**TemplateProcessing**
The transformation of template content into repository-ready content.

- **Input**: Template repository content, variable values, processing rules
- **Output**: Processed files, directories, repository metadata
- **Behavior**: Variable substitution, file path templating, content transformation
- **Constraints**: Must complete within timeout, secure path validation required

**Variable**
A placeholder in template content that gets replaced with user-provided values.

- **Types**: String, number, boolean, object
- **Properties**: Name, description, default value, validation rules
- **Behavior**: Substituted during template processing using Handlebars engine
- **Constraints**: Must follow naming conventions, validation rules enforced

## Value Objects

### Request Objects

**CreateRepositoryRequest**
Initial request to create a new repository.

- **Properties**: Repository name, owner organization, template type, variables, options
- **Validation**: Name format, organization access, template availability
- **Immutable**: Once created, cannot be modified (create new request instead)

**ValidatedRepositoryRequest**
A repository creation request that has passed all validation checks.

- **Properties**: All CreateRepositoryRequest properties plus validation metadata
- **Guarantees**: Organization access confirmed, template exists, variables validated
- **Lifecycle**: Created from CreateRepositoryRequest after successful validation

### Result Objects

**RepositoryCreationResult**
The outcome of a repository creation operation.

- **Success Properties**: Repository URL, creation timestamp, applied settings summary
- **Failure Properties**: Error details, partial completion status, rollback information
- **Metadata**: Processing duration, configuration sources used, audit trail

**TemplateProcessingResult**
The outcome of template processing operations.

- **Success Properties**: Processed files, directory structure, metadata
- **Failure Properties**: Processing errors, failed file details, security violations
- **Constraints**: All file paths validated, content size limits enforced

### Configuration Objects

**ResolvedConfiguration**
The final configuration after hierarchical resolution and merge.

- **Properties**: Repository settings, permissions, resources, source trace
- **Behavior**: Applied to repository during creation
- **Immutable**: Snapshot of configuration state at resolution time
- **Audit**: Complete trace of configuration sources and override decisions

## Error Domain

### Error Categories

**ValidationError**
Errors in input validation or business rule violations.

- **Types**: Format errors, constraint violations, policy violations
- **Context**: Field path, invalid value, expected format, suggestions
- **Recovery**: User can correct input and retry

**ProcessingError**
Errors during template processing or repository creation.

- **Types**: Template syntax errors, GitHub API failures, timeout errors
- **Context**: Processing stage, affected files, error details
- **Recovery**: May require template fixes or retry with different parameters

**ConfigurationError**
Errors in configuration loading, merging, or validation.

- **Types**: Missing configuration, override violations, schema errors
- **Context**: Configuration source, field path, policy details
- **Recovery**: Configuration updates required

**SystemError**
Unexpected system failures or infrastructure issues.

- **Types**: Network failures, service unavailability, resource exhaustion
- **Context**: Component involved, operation attempted, retry information
- **Recovery**: Automatic retry or infrastructure remediation

## Behavioral Concepts

### State Transitions

**RepositoryCreationFlow**
The ordered sequence of operations for repository creation.

- **Stages**: Request → Validation → Authentication → Configuration → Processing → Creation → Finalization
- **Invariants**: Cannot skip stages, each stage must complete successfully before proceeding
- **Error Handling**: Failure at any stage prevents progression, cleanup may be required

**ConfigurationResolution**
The process of merging hierarchical configuration layers.

- **Order**: Encoded defaults → App → Organization → Team → Template
- **Rules**: Later layers override earlier layers, override policies enforced
- **Output**: Single resolved configuration with audit trail

### Quality Attributes

**Deterministic Processing**
Template processing produces identical output for identical input.

- **Guarantee**: Same template + same variables = same result
- **Requirements**: No random elements, stable ordering, reproducible file generation

**Security Boundaries**
Clear separation between trusted and untrusted operations.

- **Trusted**: System configuration, authentication tokens, audit logs
- **Untrusted**: User input, template content, external API responses
- **Validation**: All untrusted input validated before use in trusted operations

**Audit Trail**
Complete record of all significant operations and decisions.

- **Coverage**: Authentication, authorization, configuration resolution, processing steps
- **Immutable**: Audit records cannot be modified after creation
- **Compliance**: Meets organizational audit and compliance requirements

## Naming Conventions

### Identifiers

**Repository Names**

- Format: Lowercase with hyphens (kebab-case)
- Examples: `user-service`, `api-documentation`, `shared-library`
- Constraints: GitHub repository naming rules, organization policies

**Template Types**

- Format: Lowercase with hyphens or underscores
- Examples: `rust-microservice`, `documentation-site`, `github-action`
- Scope: Unique within organization

**Configuration Keys**

- Format: Snake_case for TOML files, camelCase for JSON
- Examples: `required_approving_review_count`, `allowSquashMerge`
- Consistency: Must match GitHub API field names where applicable

### File Patterns

**Configuration Files**

- Global: `global/config.toml`, `global/defaults.toml`
- Team: `teams/{team-name}/config.toml`
- Types: `types/{type-name}/config.toml`
- Template: `.reporoller/template.toml`

**Cache Keys**

- Format: `{domain}:{organization}:{specific-id}`
- Examples: `config:acme-corp:global`, `team:acme-corp:backend-team`
- TTL: Domain-specific cache expiration policies

## Relationships and Dependencies

### Composition Relationships

**Organization contains Teams**

- One organization has zero or more teams
- Teams exist only within their organization context
- Team configurations inherit from organization defaults

**Template defines Repository Structure**

- One template produces one repository structure
- Templates can reference other templates (future: template inheritance)
- Template variables customize the resulting structure

### Dependency Relationships

**Repository Creation depends on Configuration Resolution**

- Configuration must be resolved before repository creation begins
- Configuration resolution requires organization metadata access
- Failed configuration resolution prevents repository creation

**Template Processing depends on Authentication Context**

- Template access requires appropriate permissions
- Processing operations require authenticated user context
- Security validations use authentication information

### Aggregation Relationships

**ConfigurationHierarchy aggregates multiple Configuration sources**

- Each level provides partial configuration
- Merge operation produces single resolved configuration
- Source attribution maintained for audit purposes

## Event Publishing Domain

### Core Concepts

**RepositoryEvent**
An occurrence in the repository lifecycle that external systems may need to know about.

- **Types**: Created, CreationFailed (future: Updated, Deleted)
- **Properties**: Event type, unique event ID (UUID), timestamp, repository metadata, actor information
- **Lifecycle**: Triggered → Serialized → Delivered → Acknowledged (logged)
- **Constraints**: Must be idempotent-safe (recipients may receive duplicates)
- **Delivery**: Asynchronous, fire-and-forget, best-effort

**EventNotificationEndpoint**
A configured destination for receiving repository event notifications via webhook.

- **Identity**: Unique combination of URL and organization scope
- **Properties**: HTTPS URL, shared secret reference, event type filters, active/inactive status, timeout configuration
- **Behavior**: Receives HTTP POST with JSON payload and HMAC-SHA256 signature
- **Constraints**: Must use HTTPS (no HTTP), secret must be non-empty, must specify at least one event type
- **Configuration**: Defined at organization, team, or template level

**EventDeliveryAttempt**
A single attempt to deliver an event notification to an endpoint.

- **Properties**: Endpoint URL, HTTP status code (if received), response time in milliseconds, success/failure status, error details
- **Behavior**: Records delivery outcome, enables observability and troubleshooting
- **Lifecycle**: Created → Attempted → Completed (success or failure) → Logged
- **Constraints**: Must complete within configured timeout (default 5 seconds, max 30 seconds)

**EventPublisher**
Component responsible for sending event notifications to configured webhook endpoints.

- **Responsibilities**: Load notification endpoints from configuration hierarchy, serialize events to JSON, sign requests with HMAC-SHA256, send HTTP POST requests, log delivery results, emit metrics
- **Behavior**: Asynchronous delivery (spawns background tasks), non-blocking (does not delay repository creation), best-effort (logs failures but does not retry)
- **Collaboration**: Works with ConfigurationManager (loads endpoints), SecretsManager (resolves signing secrets), MetricsCollector (records delivery stats)
- **Constraints**: Must not block repository creation workflow, must not propagate errors to caller, must log all delivery attempts

**NotificationConfiguration**
Hierarchical configuration for webhook notification endpoints.

- **Levels**: Organization (`.reporoller/global/notifications.toml`), Team (`.reporoller/teams/{team}/notifications.toml`), Template (`.reporoller/notifications.toml` in template repository)
- **Behavior**: Endpoints from all levels are accumulated (additive, not override), duplicates detected and removed based on URL + event type combination
- **Properties**: List of notification endpoints with URL, secret reference, event filters, timeout, active status
- **Validation**: All endpoints validated on load, invalid endpoints skipped with warnings

### Event Publishing Relationships

**EventPublisher consumes NotificationConfiguration**

- Publisher loads endpoints from all hierarchy levels
- Endpoints accumulated (not overridden) across levels
- Deduplication based on URL + event type

**RepositoryCreationOrchestrator triggers EventPublisher**

- After successful repository creation
- Fire-and-forget (non-blocking)
- Failures logged but do not affect repository creation result

**EventPublisher depends on SecretsManager**

- Resolves secret references to actual secret values
- Secrets used for HMAC-SHA256 signing
- Secret resolution failures skip that endpoint

This vocabulary provides the foundation for all interface definitions and implementation decisions. All team members must use these terms consistently, and any additions or modifications to this vocabulary must be documented and communicated to the entire team.
