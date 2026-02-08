# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records documenting significant architectural decisions for RepoRoller.

## What is an ADR?

An ADR documents a significant architectural decision including:

- **Context**: What problem we're solving and why
- **Decision**: What we decided to do
- **Consequences**: What this enables, forbids, and trade-offs accepted
- **Alternatives**: What we considered and why we rejected them

## When to write an ADR?

Write an ADR for decisions that:

- Affect system architecture or major components
- Have significant consequences or trade-offs
- Future developers will need to understand
- Are difficult or expensive to reverse

## ADR Index

### Active ADRs

- [ADR-001: Hexagonal Architecture for RepoRoller](ADR-001-hexagonal-architecture.md) - Ports and adapters pattern for clean separation of business logic and infrastructure
- [ADR-002: Hierarchical Configuration System](ADR-002-hierarchical-configuration.md) - 5-level configuration hierarchy with clear precedence rules
- [ADR-003: Stateless Architecture with External Storage](ADR-003-stateless-architecture.md) - No database, all state in GitHub repositories
- [ADR-004: Repository Visibility Policy System](ADR-004-repository-visibility-policy.md) - Hierarchical visibility resolution with organization policy enforcement
- [ADR-005: Multiple User Interface Strategy](ADR-005-multiple-user-interfaces.md) - CLI, API, MCP, and Azure Functions sharing single business logic core
- [ADR-006: GitHub App Authentication](ADR-006-github-app-authentication.md) - Installation tokens for organization-scoped, auto-rotating authentication
- [ADR-007: Repository Rulesets for Branch Protection](ADR-007-repository-rulesets.md) - Additive ruleset composition for flexible repository protection

## Related Documentation

- [Technology Decisions](../../.tech-decisions.yml) - Technology choices and standards
- [Implementation Constraints](../spec/constraints.md) - Rules and policies for implementation
- [Architectural Tradeoffs](../spec/tradeoffs.md) - Detailed rationale for key decisions
- [System Overview](../spec/architecture/system-overview.md) - High-level architecture

## Naming Convention

ADR files follow the pattern: `ADR-<number>-short-title.md`

## ADR Template

Use [ADR_TEMPLATE.md](ADR_TEMPLATE.md) when creating new ADRs.
