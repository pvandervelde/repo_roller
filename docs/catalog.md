# Catalog (what exists / reuse map)

Purpose: prevent reinventing utilities, modules, patterns, and "hidden" features.

Add to this whenever a reusable component becomes "the standard way".

## Crate Structure

### Business Logic (Core)

- **`crates/repo_roller_core/`** — Core business logic and domain types
  Use when: Implementing repository creation orchestration, domain rules
  Key exports: `CreateRepositoryRequest`, `Repository`, `VisibilityResolver`, error types
  Notes: Never imports external SDKs (octocrab, handlebars). Only port trait definitions.

### Adapters (Infrastructure)

- **`crates/github_client/`** — GitHub API integration via octocrab
  Use when: Need to interact with GitHub API (create repos, fetch content, manage labels)
  Key exports: `GitHubClient`, `create_app_client()`, `GitHubError`
  Notes: Implements GitHub-specific operations, handles authentication

- **`crates/config_manager/`** — Configuration loading and management
  Use when: Loading organization/team/template configuration from .reporoller/ repos
  Key exports: `ConfigurationManager`, `TemplateLoader`, `VisibilityPolicyProvider`
  Notes: Implements 5-level configuration hierarchy with caching

- **`crates/template_engine/`** — Handlebars template processing
  Use when: Processing template files with variable substitution
  Key exports: `HandlebarsTemplateEngine`, `TemplateProcessor`
  Notes: Security-validated (path traversal prevention, resource limits)

- **`crates/auth_handler/`** — Authentication and authorization
  Use when: GitHub App authentication, token management
  Key exports: `GitHubAuthService`, `UserAuthenticationService`
  Notes: Handles JWT generation, installation token retrieval

### User Interfaces

- **`crates/repo_roller_cli/`** — Command-line interface
  Use when: Building CLI commands or argument parsing
  Key exports: CLI argument structures, command handlers
  Notes: Uses clap for argument parsing, system keyring for credentials

- **`crates/repo_roller_api/`** — REST API server (Axum)
  Use when: HTTP API endpoints for repository creation
  Key exports: API handlers, request/response types, middleware
  Notes: Translation layer between HTTP JSON and domain types

- **`crates/repo_roller_mcp/`** — Model Context Protocol server
  Use when: LLM/AI agent integration
  Key exports: MCP tool definitions for repository creation
  Notes: JSON-RPC interface for AI workflows

- **`crates/repo_roller_azure_fn/`** — Azure Functions wrapper
  Use when: Serverless deployment to Azure
  Key exports: Azure Function bindings and handlers
  Notes: Wraps repo_roller_api for Azure Functions runtime

### Testing Infrastructure

- **`crates/test_utils/`** — Shared test utilities and mocks
  Use when: Writing tests that need mock implementations or test fixtures
  Key exports: Mock implementations of port traits, test builders
  Notes: Reusable across all test types

- **`crates/integration_tests/`** — Integration tests with real GitHub API
  Use when: Testing full workflows with real GitHub interactions
  Key exports: Integration test helpers, test repository management
  Notes: Uses glitchgrove test organization

- **`crates/e2e_tests/`** — End-to-end tests
  Use when: Testing complete user journeys (CLI → repo creation)
  Key exports: E2E test scenarios
  Notes: Expensive, uses real GitHub API quota, requires cleanup

- **`crates/test_cleanup/`** — Test repository cleanup utilities
  Use when: Cleaning up test repositories after test runs
  Key exports: Cleanup functions for test repositories
  Notes: See also `tests/cleanup-repos.ps1`

## Common Building Blocks

### Domain Types (Branded/Newtype Pattern)

- **`repo_roller_core::RepositoryName`** — Type-safe repository name
  Use when: Passing repository names between functions
  Pattern: `RepositoryName::new("my-repo")?` validates and wraps String

- **`repo_roller_core::OrganizationName`** — Type-safe organization name
  Use when: Passing organization names between functions
  Pattern: Similar to RepositoryName, prevents mixing with repo names

- **`repo_roller_core::GitHubToken`** — Secure token wrapper with secrecy
  Use when: Storing or passing GitHub tokens
  Pattern: `GitHubToken::new(token)`, no Debug impl, uses secrecy::Secret
  Notes: Never logs token value, even in debug output

- **`repo_roller_core::TemplateName`** — Type-safe template identifier
  Use when: Referencing template repositories
  Pattern: `TemplateName::new("rust-library")?`

### Error Handling

- **Error types** — Hierarchical error types with thiserror
  Location: `repo_roller_core/src/errors.rs`
  Pattern: `RepoRollerError` → domain-specific errors (GitHubError, ConfigurationError, etc.)
  Use when: All fallible operations (use `Result<T, ErrorType>`)
  Notes: See `docs/spec/interfaces/error-types.md` for complete hierarchy

### Configuration

- **Configuration hierarchy** — 5-level configuration resolution
  Location: `config_manager/src/lib.rs`
  Pattern: System → Org → Team → Type → Template (most specific wins)
  Use when: Loading any configuration (defaults, policies, template settings)
  Notes: Cached with 5-minute TTL, see ADR-002

### Caching

- **Policy cache** — In-memory cache with TTL
  Location: Throughout config_manager and github_client
  Pattern: `Arc<RwLock<HashMap<Key, CachedValue>>>`
  TTLs: Org policies (5 min), GitHub environment (1 hour), templates (5 min)
  Notes: Thread-safe, automatic expiration, explicit invalidation

## Cross-Cutting Helpers

- **Logging**: `tracing` crate throughout
  Usage: `use tracing::{info, warn, error, debug};`
  Pattern: Structured logging with spans for request tracking
  Notes: Never log tokens or secrets (use `?` for Debug, not secrets)

- **Error handling**: `thiserror` for error derivation
  Pattern: `#[derive(Error, Debug)] enum MyError { ... }`
  Usage: All public APIs return `Result<T, E>`, no panic in production

- **Configuration loading**: `config_manager::ConfigurationManager`
  Usage: Inject as dependency, loads from GitHub .reporoller/ repos
  Pattern: Hierarchical resolution with caching

- **Testing utilities**: `test_utils` crate
  Usage: Mock implementations of `GitHubClient`, `ConfigurationProvider`, etc.
  Pattern: `MockGitHubClient::new()` for unit tests

- **Authentication**: `auth_handler::GitHubAuthService`
  Usage: GitHub App authentication with installation tokens
  Pattern: Load credentials → Generate JWT → Get installation token → API calls

## Where to Add New Stuff

- **Reusable domain logic**: Add to `repo_roller_core/src/`
  Pattern: Define port trait in core, implement in adapter crate

- **GitHub API operations**: Add to `github_client/src/`
  Pattern: Implement trait defined in repo_roller_core

- **Configuration types**: Add to `config_manager/src/`
  Pattern: TOML-deserializable structs with validation

- **Template helpers**: Add to `template_engine/src/`
  Pattern: Handlebars helper functions registered in engine

- **CLI commands**: Add to `repo_roller_cli/src/commands/`
  Pattern: Clap subcommand with translation to domain types

- **API endpoints**: Add to `repo_roller_api/src/handlers/`
  Pattern: Axum handler with JSON request/response translation

- **Test utilities**: Add to `test_utils/src/`
  Pattern: Mock implementations, test builders, fixtures

- **Experimental**: Create new branch, mark with TODO
  Pattern: Must not be depended on by production code

## Port Traits (Hexagonal Architecture)

Core business logic depends on these trait abstractions (not concrete implementations):

- **`GitHubOperations`** — GitHub API operations
  Implemented by: `github_client::GitHubClient`

- **`ConfigurationProvider`** — Configuration loading
  Implemented by: `config_manager::ConfigurationManager`

- **`TemplateProcessor`** — Template processing
  Implemented by: `template_engine::HandlebarsTemplateEngine`

- **`VisibilityPolicyProvider`** — Visibility policy resolution
  Implemented by: `config_manager` (policy provider)

- **`GitHubEnvironmentDetector`** — GitHub plan detection
  Implemented by: `github_client` (environment detector)

See `docs/adr/ADR-001-hexagonal-architecture.md` for pattern details.

## Search Keywords

`logging`, `config`, `github-api`, `cache`, `retry`, `auth`, `metrics`, `tracing`, `cli`, `validation`, `error-handling`, `templates`, `visibility`, `testing`, `mocks`, `fixtures`
