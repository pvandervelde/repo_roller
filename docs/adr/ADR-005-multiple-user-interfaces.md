# ADR-005: Multiple User Interface Strategy

Status: Accepted
Date: 2026-01-31
Owners: RepoRoller team

## Context

RepoRoller needs to serve different user scenarios and workflows. Developers want CLI tools for automation and scripting. Users prefer web interfaces for interactive repository creation. External tools need programmatic API access. AI/LLM agents need structured tool interfaces. Each interface has different requirements but the underlying repository creation logic should be identical.

User scenarios:

- **Developers**: Command-line tool for local use and CI/CD pipelines
- **Users**: Web UI for guided repository creation with forms and validation
- **Automation**: REST API for integration with other tools and platforms
- **AI Agents**: MCP (Model Context Protocol) server for LLM workflows
- **Cloud Deployments**: Azure Functions for serverless execution

Challenge: Maintain consistent behavior across all interfaces while avoiding code duplication. Each interface has unique concerns (HTTP handling, CLI argument parsing, terminal output, JSON formatting) that shouldn't leak into business logic.

## Decision

Implement **multiple thin interface layers** sharing a single business logic core:

**Core Business Logic:**

- `repo_roller_core` - All repository creation orchestration and business rules

**Interface Implementations:**

- `repo_roller_cli` - Command-line interface (Clap argument parsing, terminal output)
- `repo_roller_api` - REST API (Axum HTTP server, JSON request/response)
- `repo_roller_mcp` - MCP server (JSON-RPC tool definitions)
- `repo_roller_azure_fn` - Azure Functions wrapper (Azure bindings, HTTP triggers)

Each interface:

1. Handles its input format (CLI args, HTTP requests, JSON-RPC calls)
2. Translates to domain types
3. Calls `repo_roller_core` business logic
4. Translates domain results back to interface format
5. Handles interface-specific concerns (status codes, colored output, progress bars)

## Consequences

**Enables:**

- Single business logic implementation shared by all interfaces
- Consistent behavior across CLI, API, MCP, and Azure Functions
- Independent evolution of interface UX without affecting others
- Easy testing of business logic without interface concerns
- Adding new interfaces without modifying core logic
- Interface-specific optimizations (progress bars in CLI, streaming in API)

**Forbids:**

- Business logic in interface crates
- Direct HTTP, CLI, or JSON-RPC types in `repo_roller_core`
- Interface-specific behavior in shared business logic
- Duplicating orchestration logic across interfaces

**Trade-offs:**

- Maintenance overhead for multiple interface codebases
- Need for translation layer between interface types and domain types
- Some interface-specific features require careful design to keep core generic
- More complex deployment story (multiple binaries/deployments to maintain)

## Alternatives considered

### Option A: CLI only with separate API implementation

**Why not**: Duplicates business logic between CLI and API. Changes require updates in multiple places. Behavior drift over time. Testing burden doubled.

### Option B: Web UI only (no CLI)

**Why not**: Developers need automation. CLI essential for CI/CD integration. Cannot script repository creation. Poor developer experience.

### Option C: API only (CLI calls API)

**Why not**: CLI requires API server running. Extra complexity for local development. Network dependency for simple operations. Deployment burden for CLI users.

### Option D: Monolithic binary with all interfaces

**Why not**: Large binary size. Cannot deploy only needed interfaces. All dependencies included everywhere. Complicates cloud deployment.

## Implementation notes

**Dependency structure:**

```
repo_roller_cli ─┐
repo_roller_api ─┼─> repo_roller_core ─> [adapters: github_client, config_manager, template_engine]
repo_roller_mcp ─┤
repo_roller_azure_fn ┘
```

**Translation pattern:**
Each interface implements translation between its types and domain types:

```rust
// In repo_roller_api
fn http_request_to_domain(req: CreateRepoApiRequest) -> Result<CreateRepositoryRequest, ApiError>
fn domain_result_to_http(result: Repository) -> CreateRepoApiResponse

// In repo_roller_cli
fn cli_args_to_domain(args: CreateRepoArgs) -> Result<CreateRepositoryRequest, CliError>
fn domain_result_to_output(result: Repository) -> CliOutput
```

**Interface responsibilities:**

| Interface | Input Format | Output Format | Specific Concerns |
|-----------|--------------|---------------|-------------------|
| CLI | Command args | Terminal text/JSON | Colors, progress bars, interactive prompts |
| API | HTTP JSON | HTTP JSON | Status codes, headers, content negotiation |
| MCP | JSON-RPC | JSON-RPC | Tool schemas, parameter validation |
| Azure Fn | HTTP (Azure bindings) | HTTP (Azure bindings) | Azure context, logging, response formatting |

**Shared business logic examples:**

- Repository creation orchestration
- Configuration resolution
- Template processing
- Visibility policy enforcement
- Error handling and recovery

**Interface-specific features:**

- CLI: Colored output, progress indicators, interactive mode
- API: Rate limiting, API versioning, batch operations
- MCP: Tool discovery, schema generation, streaming results
- Azure Fn: Cold-start optimization, Azure Monitor integration

## Examples

**CLI interface (`repo_roller_cli`):**

```rust
// CLI-specific: argument parsing
let args = CreateRepoArgs::parse();

// Translation to domain types
let request = CreateRepositoryRequest {
    name: RepositoryName::new(&args.name)?,
    organization: OrganizationName::new(&args.org)?,
    template: TemplateName::new(&args.template)?,
    visibility: args.visibility.map(|v| v.into()),
};

// Shared business logic
let result = orchestrator.create_repository(request).await?;

// CLI-specific: formatted output
println!("{} Repository created: {}", "✓".green(), result.url);
```

**REST API interface (`repo_roller_api`):**

```rust
// API-specific: HTTP request handling
async fn create_repository(
    State(state): State<AppState>,
    Json(req): Json<CreateRepoApiRequest>,
) -> Result<Json<CreateRepoApiResponse>, ApiError> {
    // Translation to domain types
    let request = translate_api_request_to_domain(req)?;

    // Shared business logic
    let result = state.orchestrator.create_repository(request).await?;

    // API-specific: HTTP response
    Ok(Json(CreateRepoApiResponse {
        repository_url: result.url.to_string(),
        created_at: result.created_at,
        status: "success",
    }))
}
```

**MCP interface (`repo_roller_mcp`):**

```rust
// MCP-specific: tool definition
#[tool]
async fn create_repository(
    name: String,
    organization: String,
    template: String,
) -> Result<RepositoryCreationResult, McpError> {
    // Translation to domain types
    let request = CreateRepositoryRequest {
        name: RepositoryName::new(&name)?,
        organization: OrganizationName::new(&organization)?,
        template: TemplateName::new(&template)?,
        visibility: None,
    };

    // Shared business logic
    let result = get_orchestrator().create_repository(request).await?;

    // MCP-specific: structured result
    Ok(RepositoryCreationResult {
        url: result.url,
        name: result.name,
        created: true,
    })
}
```

**Azure Function wrapper (`repo_roller_azure_fn`):**

```rust
// Azure-specific: function binding
#[azure_function]
async fn create_repository_http(
    req: HttpRequest,
    _context: Context,
) -> HttpResponse {
    // Parse Azure HTTP request
    let body: CreateRepoApiRequest = req.json().await?;

    // Translation to domain types (reuse API translation)
    let request = translate_api_request_to_domain(body)?;

    // Shared business logic
    let result = get_orchestrator().create_repository(request).await?;

    // Azure-specific: HTTP response formatting
    HttpResponse::Ok().json(CreateRepoApiResponse::from(result))
}
```

## References

- [System Architecture Overview](../spec/architecture/system-overview.md)
- [Component Details](../spec/architecture/components.md)
- [Hexagonal Architecture ADR](ADR-001-hexagonal-architecture.md)
- [API Documentation](../../crates/repo_roller_api/README.md)
- [CLI Documentation](../../crates/repo_roller_cli/README.md)
