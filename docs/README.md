# RepoRoller Documentation

Welcome to the RepoRoller documentation. Start here to understand the system and find what you need.

## Quick Start

- **New to RepoRoller?** Read: [Problem Statement](spec/overview/problem-statement.md) and [Solution Overview](spec/overview/solution-overview.md)
- **Building a feature?** Check: [Constraints](constraints.md) → [Catalog](catalog.md) → [ADRs](adr/README.md)
- **Writing tests?** Read: [Testing Standards](standards/testing.md) and [Testing Strategy](../tests/TESTING_STRATEGY.md)
- **Fixing a bug?** Check: [Catalog](catalog.md) for existing code, [Error Types](spec/interfaces/error-types.md) for error handling

## Documentation Structure

### Core Decision Documents

- **[constraints.md](constraints.md)** — Quick reference of hard rules and strong preferences (read this first!)
- **[catalog.md](catalog.md)** — What exists and where to find it (prevent duplication)
- **[ADRs](adr/README.md)** — Architecture Decision Records explaining why we built it this way
- **[.tech-decisions.yml](../.tech-decisions.yml)** — Technology choices and enforceable standards

### Detailed Specifications

- **[spec/](spec/)** — Detailed technical specifications
  - [overview/](spec/overview/) — Problem statement, solution overview, design goals
  - [architecture/](spec/architecture/) — System architecture, components, data flow
  - [interfaces/](spec/interfaces/) — API contracts, type definitions, interface specifications
  - [design/](spec/design/) — Feature-specific design documents
  - [requirements/](spec/requirements/) — Detailed requirements for features
  - [operations/](spec/operations/) — Observability, deployment, operational concerns
  - [security/](spec/security/) — Security architecture and threat model

### Standards and Conventions

- **[standards/](standards/)** — Coding standards, testing conventions, style guides
  - [code.md](standards/code.md) — Coding conventions for Rust
  - [testing.md](standards/testing.md) — Testing organization and requirements

## Architecture at a Glance

RepoRoller uses **Hexagonal Architecture** (Ports and Adapters):

```
User Interfaces          Business Logic          Adapters
┌─────────────────┐      ┌─────────────────┐     ┌─────────────────┐
│ CLI             │──────│                 │     │ GitHub Client   │
│ REST API        │──────│ repo_roller_    │─────│ Config Manager  │
│ MCP Server      │──────│ core            │─────│ Template Engine │
│ Azure Functions │──────│                 │     │ Auth Handler    │
└─────────────────┘      └─────────────────┘     └─────────────────┘
```

- **Core**: Pure business logic, no external dependencies
- **Ports**: Trait definitions for external operations
- **Adapters**: Concrete implementations (GitHub API, handlebars, etc.)
- **Interfaces**: Thin layers translating between user input and domain types

See [ADR-001](adr/ADR-001-hexagonal-architecture.md) for details.

## Key Architectural Decisions

1. **[Hexagonal Architecture](adr/ADR-001-hexagonal-architecture.md)** — Separation of business logic from infrastructure
2. **[Hierarchical Configuration](adr/ADR-002-hierarchical-configuration.md)** — 5-level config system (System → Org → Team → Type → Template)
3. **[Stateless Architecture](adr/ADR-003-stateless-architecture.md)** — No database, all state in GitHub
4. **[Visibility Policy System](adr/ADR-004-repository-visibility-policy.md)** — Org-enforced security policies
5. **[Multiple User Interfaces](adr/ADR-005-multiple-user-interfaces.md)** — CLI, API, MCP, Azure Functions share core logic
6. **[GitHub App Authentication](adr/ADR-006-github-app-authentication.md)** — Installation tokens for secure, org-scoped auth

## Key Technology Choices

- **Language**: Rust (performance, safety, single binary deployment)
- **Error Handling**: `Result<T, E>` throughout (explicit, no panic)
- **Type Safety**: Branded types (newtype pattern) for all domain primitives
- **GitHub Client**: octocrab (mature Rust GitHub API client)
- **Template Engine**: handlebars (familiar syntax, secure)
- **Web Framework**: axum (async, type-safe HTTP)
- **Logging**: tracing (structured, async-aware)
- **Testing**: Unit + Integration + E2E (see [testing standards](standards/testing.md))

See [.tech-decisions.yml](../.tech-decisions.yml) for complete list.

## Common Tasks

### Adding a New Feature

1. Check [constraints.md](constraints.md) for hard rules
2. Check [catalog.md](catalog.md) for existing code to reuse
3. Read relevant ADRs ([ADR index](adr/README.md))
4. Define types in `repo_roller_core` (domain types)
5. Implement adapters if needed (GitHub API, config, templates)
6. Add interface layer (CLI, API, etc.)
7. Write tests (unit → integration → e2e)
8. Update [catalog.md](catalog.md) if you created reusable components

### Understanding Error Handling

1. Read [Error Types Specification](spec/interfaces/error-types.md)
2. All operations return `Result<T, ErrorType>`
3. Use `thiserror` for error type derivation
4. Never panic in production code
5. Map adapter errors to domain errors

### Working with Configuration

1. Read [ADR-002: Hierarchical Configuration](adr/ADR-002-hierarchical-configuration.md)
2. Configuration stored in `.reporoller/` metadata repos (GitHub)
3. 5 levels: System → Org → Team → Type → Template
4. Cached in-memory with 5-minute TTL
5. Organization policies enforced (cannot be overridden)

### Adding Tests

1. Read [Testing Standards](standards/testing.md)
2. Unit tests: Co-located `*_tests.rs` files
3. Integration tests: `crates/integration_tests/` (real GitHub API)
4. E2E tests: `crates/e2e_tests/` (complete user journeys)
5. Use `test_utils` for mocks and fixtures
6. Cleanup test repos after tests (use `test_cleanup`)

## For AI Agents

If you're an AI agent working on this codebase:

1. **Start here**: Read [AGENTS.md](../AGENTS.md) for agent-specific guidelines
2. **Check constraints**: [constraints.md](constraints.md) has non-negotiable rules
3. **Find existing code**: [catalog.md](catalog.md) prevents reinventing the wheel
4. **Understand decisions**: [ADRs](adr/README.md) explain why things are the way they are
5. **Follow standards**: [standards/](standards/) for coding and testing conventions

## Getting Help

- **Architecture questions**: Read relevant [ADRs](adr/README.md)
- **What exists?**: Check [catalog.md](catalog.md)
- **Coding standards**: See [standards/code.md](standards/code.md)
- **Testing questions**: See [standards/testing.md](standards/testing.md)
- **Spec details**: Browse [spec/](spec/) for detailed documentation
