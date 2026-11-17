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

### Enum Types

| Type | Location | Re-exported From | Purpose | Spec Reference |
|------|----------|------------------|---------|----------------|
| `RepositoryVisibility` | `config_manager/src/visibility.rs` | `repo_roller_core::RepositoryVisibility` | Repository visibility level (Public, Private, Internal) | [repository-visibility.md](interfaces/repository-visibility.md#repositoryvisibility) |
| `VisibilityPolicy` | `config_manager/src/visibility.rs` | `repo_roller_core::VisibilityPolicy` | Organization visibility policy (Required, Restricted, Unrestricted) | [repository-visibility.md](interfaces/repository-visibility.md#visibilitypolicy) |
| `PolicyConstraint` | `config_manager/src/visibility.rs` | `repo_roller_core::PolicyConstraint` | Applied visibility constraints | [repository-visibility.md](interfaces/repository-visibility.md#policyconstraint) |
| `DecisionSource` | `repo_roller_core/src/visibility.rs` | - | Source of visibility decision | [repository-visibility.md](interfaces/repository-visibility.md#decisionsource) |
| `PlanLimitations` | `github_client/src/environment.rs` | - | GitHub plan limitations for visibility | [repository-visibility.md](interfaces/repository-visibility.md#planlimitations) |
| `ContentStrategy` | `repo_roller_core/src/request.rs` | - | Content generation strategy (Template, Empty, CustomInit) | [repository-creation-modes.md](interfaces/repository-creation-modes.md#contentstrategy-enum) |

**Note**: Policy types defined in `config_manager` to avoid circular dependencies. See [repository-visibility.md](interfaces/repository-visibility.md#circular-dependency-resolution) for architectural rationale.

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
| `RepoRollerError` | `repo_roller_core/src/errors.rs` | Top-level error enum | [error-types.md](interfaces/error-types.md#reporollererror) |
| `ValidationError` | `repo_roller_core/src/errors.rs` | Input validation failures | [error-types.md](interfaces/error-types.md#validationerror) |
| `RepositoryError` | `repo_roller_core/src/errors.rs` | Repository operation errors | [error-types.md](interfaces/error-types.md#repositoryerror) |
| `ConfigurationError` | `config_manager/src/errors.rs` | Configuration errors | [error-types.md](interfaces/error-types.md#configurationerror) |
| `TemplateError` | `template_engine/src/lib.rs` | Template processing errors | [error-types.md](interfaces/error-types.md#templateerror) |
| `AuthenticationError` | `auth_handler/src/lib.rs` | Auth/authz errors | [error-types.md](interfaces/error-types.md#authenticationerror) |
| `GitHubError` | `github_client/src/errors.rs` | GitHub API errors | [error-types.md](interfaces/error-types.md#githuberror) |
| `SystemError` | `repo_roller_core/src/errors.rs` | System/infrastructure errors | [error-types.md](interfaces/error-types.md#systemerror) |

**Metadata discovery error**:

- `ConfigurationError::AmbiguousMetadataRepository` - Multiple metadata repos found with same topic

**Template loading errors**:

- `ConfigurationError::TemplateNotFound` - Template repository not found or not accessible
- `ConfigurationError::TemplateConfigurationMissing` - Template repository exists but missing `.reporoller/template.toml`

## Business Logic Interfaces

### Repository Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `create_repository` | `repo_roller_core/src/lib.rs` | Repository creation workflow | [repository-domain.md](interfaces/repository-domain.md) |
| `RepositoryCreationRequest` | `repo_roller_core/src/request.rs` | Typed repository creation request | [repository-domain.md](interfaces/repository-domain.md) |
| `RepositoryCreationResult` | `repo_roller_core/src/request.rs` | Successful repository creation result | [repository-domain.md](interfaces/repository-domain.md) |

### Configuration Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `OrganizationSettingsManager` | `config_manager/src/organization_settings_manager.rs` | Hierarchical configuration resolution | [configuration-interfaces.md](interfaces/configuration-interfaces.md) |
| `MetadataRepositoryProvider` | `config_manager/src/metadata_provider.rs` | Metadata repository discovery and access abstraction | [organization-repository-settings.md](design/organization-repository-settings.md) |
| `ConfigurationMerger` | `config_manager/src/merger.rs` | Merge applicable configuration levels | [configuration-interfaces.md](interfaces/configuration-interfaces.md) |
| `ConfigurationValidator` | `config_manager/src/validator.rs` | Validate configuration | [configuration-interfaces.md](interfaces/configuration-interfaces.md) |
| `TemplateRepository` | `config_manager/src/template_loader.rs` | Template configuration access abstraction | [template-loading.md](interfaces/template-loading.md) |
| `TemplateLoader` | `config_manager/src/template_loader.rs` | Template config loading with caching | [template-loading.md](interfaces/template-loading.md) |

### Event Notification Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `RepositoryCreatedEvent` | `repo_roller_core/src/event_publisher.rs` | Event payload for repository.created | [event-publisher.md](interfaces/event-publisher.md#repositorycreatedevent) |
| `AppliedSettings` | `repo_roller_core/src/event_publisher.rs` | Repository settings in event payload | [event-publisher.md](interfaces/event-publisher.md#appliedsettings) |
| `NotificationEndpoint` | `config_manager/src/settings/notifications.rs` | Outbound webhook endpoint configuration | [event-publisher.md](interfaces/event-publisher.md#notificationendpoint) |
| `NotificationsConfig` | `config_manager/src/settings/notifications.rs` | Outbound notification configuration | [event-publisher.md](interfaces/event-publisher.md#notificationsconfig) |
| `DeliveryResult` | `repo_roller_core/src/event_publisher.rs` | Delivery outcome | [event-publisher.md](interfaces/event-publisher.md#deliveryresult) |
| `SecretResolver` | `repo_roller_core/src/event_secrets.rs` | Secret resolution abstraction | [event-secrets.md](interfaces/event-secrets.md#secretresolver-trait) |
| `EventMetrics` | `repo_roller_core/src/event_metrics.rs` | Metrics recording abstraction | [event-metrics.md](interfaces/event-metrics.md#eventmetrics-trait) |

### Template Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `TemplateFetcher` | `template_engine/src/lib.rs` | Template file retrieval | [template-interfaces.md](interfaces/template-interfaces.md) |
| `TemplateProcessor` | `template_engine/src/lib.rs` | Template variable substitution | [template-interfaces.md](interfaces/template-interfaces.md) |
| `TemplateProcessingRequest` | `template_engine/src/lib.rs` | Template processor input | [template-interfaces.md](interfaces/template-interfaces.md) |

### Authentication Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `UserAuthenticationService` | `auth_handler/src/lib.rs` | User authentication | — |

### Visibility Domain

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `VisibilityResolver` | `repo_roller_core/src/visibility.rs` | Visibility decision orchestration | [repository-visibility.md](interfaces/repository-visibility.md#visibilityresolver) |
| `VisibilityPolicyProvider` | `config_manager/src/visibility.rs` | Organization policy access (trait) | [repository-visibility.md](interfaces/repository-visibility.md#visibilitypolicyprovider) |
| `ConfigBasedPolicyProvider` | `config_manager/src/visibility_policy_provider.rs` | Config-based policy implementation | [repository-visibility.md](interfaces/repository-visibility.md#visibilitypolicyprovider) |
| `GitHubEnvironmentDetector` | `github_client/src/environment.rs` | GitHub environment detection (trait) | [repository-visibility.md](interfaces/repository-visibility.md#githubenvironmentdetector) |
| `GitHubApiEnvironmentDetector` | `github_client/src/environment_detector.rs` | GitHub API-based environment detection | [repository-visibility.md](interfaces/repository-visibility.md#githubenvironmentdetector) |
| `VisibilityDecision` | `repo_roller_core/src/visibility.rs` | Resolved visibility with audit trail | [repository-visibility.md](interfaces/repository-visibility.md#visibilitydecision) |
| `VisibilityRequest` | `repo_roller_core/src/visibility.rs` | Visibility resolution input | [repository-visibility.md](interfaces/repository-visibility.md#visibilityrequest) |
| `PlanLimitations` | `github_client/src/environment.rs` | GitHub plan constraints | [repository-visibility.md](interfaces/repository-visibility.md#planlimitations) |
| `VisibilityError` | `config_manager/src/visibility.rs` | Visibility-specific errors | [repository-visibility.md](interfaces/repository-visibility.md#error-types) |

**Architecture Note**: Policy types in `config_manager` to avoid circular dependencies with `repo_roller_core`. See [circular dependency resolution](interfaces/repository-visibility.md#circular-dependency-resolution).

## External Integration Interfaces

### GitHub Integration

| Interface | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| `RepositoryClient` | `github_client/src/lib.rs` | Repository creation and update operations | [github-interfaces.md](interfaces/github-interfaces.md) |
| `GitHubClient` | `github_client/src/lib.rs` | Concrete GitHub API client | [github-interfaces.md](interfaces/github-interfaces.md) |

**Repository and metadata operations include**:

- `get_installation_token_for_org` and `get_organization_default_branch` for organization-scoped operations
- Repository content access for configuration and template loading

## HTTP API Types

**Note**: HTTP API types are distinct from domain types and exist only in the `repo_roller_api` crate.

### Request Types (HTTP Layer)

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `CreateRepositoryRequest` | `repo_roller_api/src/models/request.rs` | HTTP request for repository creation | [api-request-types.md](interfaces/api-request-types.md) |
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

## CLI-Specific Types

**Note**: CLI types are distinct from domain and HTTP API types, existing only in `repo_roller_cli`.

### Template Display Types

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `TemplateInfo` | `repo_roller_cli/src/commands/template_cmd.rs` | CLI-friendly template information | [cli-template-operations.md](interfaces/cli-template-operations.md) |
| `RepositoryTypeInfo` | `repo_roller_cli/src/commands/template_cmd.rs` | CLI repository type display | [cli-template-operations.md](interfaces/cli-template-operations.md) |
| `TemplateVariableInfo` | `repo_roller_cli/src/commands/template_cmd.rs` | CLI variable display | [cli-template-operations.md](interfaces/cli-template-operations.md) |

### Validation Types

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `TemplateValidationResult` | `repo_roller_cli/src/commands/template_cmd.rs` | Template validation result | [cli-template-operations.md](interfaces/cli-template-operations.md) |
| `ValidationIssue` | `repo_roller_cli/src/commands/template_cmd.rs` | Individual validation issue | [cli-template-operations.md](interfaces/cli-template-operations.md) |
| `ValidationWarning` | `repo_roller_cli/src/commands/template_cmd.rs` | Validation warning | [cli-template-operations.md](interfaces/cli-template-operations.md) |

**Translation Pattern**: Domain `TemplateConfig` types converted to CLI `TemplateInfo` at CLI boundary.
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
| `TemplateConfig` | `config_manager/src/template_config.rs` | Template configuration | [template-loading.md](interfaces/template-loading.md) |
| `TemplatingConfig` | `template_engine/src/lib.rs` | File filtering configuration (Task 7.0) | [template-file-filtering.md](interfaces/template-file-filtering.md) |
| `RepositorySettings` | `config_manager/src/settings/repository.rs` | Repository settings | [configuration-interfaces.md](interfaces/configuration-interfaces.md) |
| `CacheStatistics` | `config_manager/src/template_loader.rs` | Template cache metrics | [template-loading.md](interfaces/template-loading.md) |
| `VariableConfig` | `template_engine/src/lib.rs` | Variable validation rules | [template-interfaces.md](interfaces/template-interfaces.md) |
| `CustomInitOptions` | `repo_roller_core/src/content_providers.rs` | Custom initialization options | [content-providers.md](interfaces/content-providers.md) |
| `LabelConfig` | `config_manager/src/settings/label.rs` | Label configuration | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `WebhookConfig` | `config_manager/src/settings/webhook.rs` | Webhook configuration | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |

## GitHub API Types (Task 10.0)

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `Label` | `github_client/src/label.rs` | GitHub label representation | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `Webhook` | `github_client/src/webhook.rs` | GitHub webhook with ID | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `WebhookDetails` | `github_client/src/webhook.rs` | Webhook config details | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |

## Manager Components (Task 10.0)

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `LabelManager` | `repo_roller_core/src/label_manager.rs` | Label orchestration | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `WebhookManager` | `repo_roller_core/src/webhook_manager.rs` | Webhook orchestration | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `ApplyLabelsResult` | `repo_roller_core/src/label_manager.rs` | Label operation result | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |
| `ApplyWebhooksResult` | `repo_roller_core/src/webhook_manager.rs` | Webhook operation result | [github-label-webhook-operations.md](interfaces/github-label-webhook-operations.md) |

## Content Provider Types

| Type | Location | Purpose | Spec Reference |
|------|----------|---------|----------------|
| `ContentProvider` | `repo_roller_core/src/content_providers.rs` | Trait for repository content generation | [content-providers.md](interfaces/content-providers.md#contentprovider-trait) |
| `TemplateBasedContentProvider` | `repo_roller_core/src/content_providers.rs` | Fetch and process template files | [content-providers.md](interfaces/content-providers.md#templatebasedcontentprovider) |
| `ZeroContentProvider` | `repo_roller_core/src/content_providers.rs` | Create empty repository | [content-providers.md](interfaces/content-providers.md#zerocontentprovider) |
| `CustomInitContentProvider` | `repo_roller_core/src/content_providers.rs` | Create custom initialization files | [content-providers.md](interfaces/content-providers.md#custominitcontentprovider) |
| `CustomInitOptions` | `repo_roller_core/src/content_providers.rs` | Options for custom initialization | [content-providers.md](interfaces/content-providers.md#custominitcontentprovider) |
| `ContentStrategy` | `repo_roller_core/src/request.rs` | Content generation strategy enum | [repository-creation-modes.md](interfaces/repository-creation-modes.md#contentstrategy-enum) |

## Patterns and Conventions

### Async Trait Pattern

I/O abstractions use asynchronous traits where an implementation boundary is
needed. For example, `TemplateRepository` abstracts template configuration
storage:

```rust
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn load_template_config(
        &self,
        org: &str,
        template_name: &str,
    ) -> ConfigurationResult<TemplateConfig>;
}
```

**Location**: All interface traits
**Dependencies**: `async_trait` crate

### Builder Pattern

`RepositoryCreationRequestBuilder` constructs typed repository creation
requests:

```rust
RepositoryCreationRequestBuilder::new(repo_name, organization)
    .template(template_name)
    .build()
```

### Newtype Pattern

All domain primitives wrapped for type safety:

```rust
pub struct RepositoryName(String);
```

**Location**: `repo_roller_core/src/repository.rs`, `template.rs`, and `github.rs`
**See**: [Shared Types](interfaces/shared-types.md)

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

API request/response DTOs derive `Serialize` and `Deserialize` where they
cross JSON boundaries. Domain types that are serialized also derive the
appropriate serde traits. These definitions are maintained in their
respective API and domain modules.

## Dependency Graph

### Business Logic Dependencies

```
repo_roller_core
├── repository.rs, template.rs, github.rs, authentication.rs (domain types)
├── request.rs (creation requests and results)
├── errors.rs (domain errors and result aliases)
└── [orchestration, permissions, visibility, notification modules]
```

### External Integration Dependencies

```
github_client
├── provides GitHubClient and RepositoryClient
└── external: octocrab, tokio

template_engine
├── provides TemplateFetcher and TemplateProcessor
└── external: handlebars, regex

config_manager
├── provides configuration models, loaders, merger, and validation
└── external: toml, serde

auth_handler
├── provides UserAuthenticationService
└── external: jsonwebtoken, chrono
```

## Testing Utilities

### Test helpers and mocks

Tests define local mocks and fixtures alongside the modules they exercise.
The `test_utils` crate provides shared helpers such as test repository-name
generation; there is no central set of builders or mocks for every service.

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

When changing interfaces or introducing new types:

- Update affected call sites and their tests together.
- Preserve existing behavior unless the change explicitly alters it.
- Document any required migration in a linked issue or specification.
- Update this registry with the implemented interface.

## Quick Reference

### Most Commonly Used Types

1. `RepositoryName` - Almost every operation
2. `OrganizationName` - Repository creation, config resolution
3. `TemplateName` - Template operations
4. `RepoRollerResult<T>` - All public functions
5. `RepositoryCreationRequest` - Repository creation workflow

### Most Important Traits

1. `RepositoryClient` - GitHub repository operations
2. `MetadataRepositoryProvider` - Configuration source access
3. `TemplateRepository` - Template configuration access
4. `UserAuthenticationService` - Authentication boundary

## Infrastructure Components

### CI/CD Automation

| Component | Location | Purpose | Spec Reference |
|-----------|----------|---------|----------------|
| Release PR Workflow | `.github/workflows/release-pr.yml` | Auto-create/update release PRs | [release-automation.md](interfaces/release-automation.md#release-pr-management-workflow) |
| Release Publish Workflow | `.github/workflows/release-publish.yml` | Publish releases and artifacts | [release-automation.md](interfaces/release-automation.md#release-publication-workflow) |
| Version Comment Handler | `.github/workflows/release-comment.yml` | Handle version override comments | [release-automation.md](interfaces/release-automation.md#version-comment-handler-workflow) |

**Version Conventions**:

- Semantic versioning (semver 2.0)
- Conventional commits for version calculation
- Pre-1.0 versions signal API instability

**Artifact Publishing**:

- Container image: `ghcr.io/pvandervelde/repo_roller_api:v{version}`
- CLI binaries: Linux, Windows, macOS (x86_64)
- GitHub Releases with changelog

**See**: [constraints.md](constraints.md#release-management-constraints) for release management rules

---

Update this registry when implemented interfaces or their locations change.
