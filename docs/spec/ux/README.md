# RepoRoller Web UI — UX Specification

## Overview

The RepoRoller web UI provides a self-service interface for GitHub organization members to create
new repositories from predefined templates. It is an internal tool, accessible only on the
organization VPN.

**Primary user goal**: Create a new, fully-configured GitHub repository in minutes, without CLI
access or manual GitHub setup steps.

**Authentication**: GitHub OAuth (Model A). The VPN handles access control. OAuth captures the
creator's verified GitHub username for audit logging.

---

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Auth model | GitHub OAuth ("Sign in with GitHub") | Verified identity for audit trail; VPN is the access gate |
| Org membership enforcement | Deferred (architecture supports adding later) | VPN is the current boundary; org check can be enabled later |
| Creation UX | Stepped wizard (2–3 steps) | Template variables are dynamic; sequential steps reduce cognitive overload |
| Template browsing | Inline as Step 1 of creation wizard | Template browsing is part of the creation decision, not a separate use case |
| Org selection | None — single org configured at deployment | Removes unnecessary input; multi-org can be added later |
| Visibility input | Shown only if org configuration permits override | Avoids confusing users with options they cannot change |
| Branding | Logo, app name, primary colour configured at deployment | Organizations can white-label the tool without code changes; see [branding.md](branding.md) |

---

## Screen Inventory

| ID | Screen | Route | Authentication required |
|---|---|---|---|
| SCR-001 | Sign In | `/sign-in` | No |
| SCR-002 | OAuth Callback | `/auth/callback` | No (in-progress) |
| SCR-003 | Access Denied | `/auth/denied` | No |
| SCR-004 | Create Repository | `/create` | Yes |
| SCR-005 | Repository Created | `/create/success` | Yes |
| SCR-006 | Error | `/error` | No |

---

## User Flows

- [Authentication flow](flows/authentication.md) — sign-in, OAuth callback, session management
- [Repository creation flow](flows/repository-creation.md) — template selection through to created repository

---

## Screens

- [SCR-001: Sign In](screens/SCR-001-sign-in.md)
- [SCR-002: OAuth Callback](screens/SCR-002-oauth-callback.md)
- [SCR-003: Access Denied](screens/SCR-003-access-denied.md)
- [SCR-004: Create Repository](screens/SCR-004-create-repository.md)
- [SCR-005: Repository Created](screens/SCR-005-repository-created.md)
- [SCR-006: Error](screens/SCR-006-error.md)

---

## Components

- [Component Inventory](components/component-inventory.md)

---

## Supporting Specifications

- [Branding & White-Labelling](branding.md) — deployment-time brand configuration
- [Navigation Map](navigation.md)
- [UX Assertions](ux-assertions.md) — testable behaviour specifications
- [Copy Reference](copy.md) — all user-facing strings in one place

---

## UX Design Complete — Handoff Summary

UX specifications have been produced in `docs/spec/ux/`.

**Screens defined**: 6 (SCR-001 through SCR-006)
**User flows**: 2 (authentication, repository creation)
**Components**: 11 reusable components with prop/event contracts + 1 shared shell (BrandCard)
**UX assertions**: 27 testable behavioural specifications
**Wireframes**: All 6 screens

### Key design decisions

1. **GitHub OAuth for identity** — VPN is the access gate; OAuth captures a verified GitHub
   username for the audit log. Org membership check is deferred but the architecture (including
   `read:org` scope at sign-in time) supports enabling it without re-engineering the auth flow.
2. **2–3 step wizard** — Step 3 (variables) is conditionally skipped when the selected template
   defines no variables, keeping the common case to 2 steps.
3. **Template details fetched on card selection** — eliminates visible loading delay when the
   user clicks Next to advance from Step 1.
4. **Name uniqueness check on blur, not on type** — avoids hammering the API on every keystroke.
5. **Creation errors are inline with data preserved** — users never lose form data on a failed
   creation attempt; they can correct and retry without starting over.
6. **Deployment-time branding** — logo, app name, and primary colour are configured via
   `brand.toml` or environment variables; a CSS custom property (`--brand-primary`) propagates
   the colour to all interactive elements. See [ADR-008](../adr/ADR-008-web-ui-branding.md).

### For the Interface Designer

- Translate component contracts in [components/component-inventory.md](components/component-inventory.md)
  into typed SvelteKit component props and events.
- Define a `BrandConfig` type for the branding configuration object (see [branding.md](branding.md)).
- Define the `NameValidationResult` discriminated union used by `RepositoryNameField` (CMP-006).
- Define route parameter types for `/create/success?repo={org}/{name}`.
- Map the `TemplateSummary`, `TemplateVariable`, and `RepositoryTypeOption` shapes from the
  REST API response types to frontend model types.

### For the Tester

- Use [ux-assertions.md](ux-assertions.md) as the specification for all UI behaviour tests.
- Priority assertions for initial test coverage: 001, 002, 005, 009, 010, 011, 015, 018, 022.
- All 27 assertions should eventually have corresponding integration or component tests.
