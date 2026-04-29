---
title: "The permissions model"
description: "How team and collaborator permissions are accumulated, protected, and audited across config levels."
audience: "platform-engineer"
type: "explanation"
---

# The permissions model

## How permissions accumulate

When a repository is created, RepoRoller builds a final permissions map by processing four sources in order:

1. **Global defaults** — `[[default_teams]]` and `[[default_collaborators]]` in `global/defaults.toml`
2. **Repository type** — `[[default_teams]]` in the type config
3. **Template** — `[[teams]]` and `[[collaborators]]` in the template's `template.toml`
4. **Creation request** — teams and collaborators specified in the API call or CLI flags

Sources are processed in increasing order of specificity. Each source can upgrade (raise) the access level established by a previous source, subject to governance rules.

## No-demotion rule

An entry established at a lower level can never have its access level lowered by a higher level. If global config grants the `platform` team `write` access and a template tries to set `platform` to `read`, the template's instruction is rejected with a hard error and the repository is not created.

This makes a lower-level grant a guaranteed minimum. Setting `write` for the `platform` team in global config means every repository in the organisation will have platform at `write` or above, regardless of what templates declare.

## Locked entries

An entry can be marked `locked = true` to make it immutable at config-resolution time (levels 1–3) and silently non-overridable at request time (level 4).

A locked `security-ops=admin` entry in global config means:

- Templates that try to change `security-ops` cause a hard error
- Creation requests that include `security-ops` with a different level are silently ignored (the locked level is preserved, a warning is logged)

## Access level ceilings

The `[permissions]` section in global config caps the maximum access level that a **creation request** may grant. Config-established entries (from global, type, or template) are not subject to ceilings.

If the ceiling is `maintain` and a request includes `"some-team": "admin"`, the team receives `maintain` access and a warning is logged. The repository is still created.

## Why this design

These three mechanisms together enforce a governance model where:

- Security-critical teams always have the access they need (locked admin entries)
- Access escalates predictably as repositories become more specialised (no-demotion)
- Users cannot grant excessive access through the creation request (ceiling)

The model is deliberately asymmetric: lower levels can establish and protect grants, higher levels can only raise not lower them. Requests can only work within the space that configuration has defined.

## Audit logging

Every significant permission decision emits a structured event on the `repo_roller_core::permission_audit` tracing target:

- A repository was created and permissions were applied (`outcome = "applied"`)
- A request triggered the policy engine for review (`outcome = "requires_approval"`)
- A permission error blocked creation (`outcome = "denied"`)

Filter these events for SIEM ingestion:

```bash
RUST_LOG="repo_roller_core::permission_audit=info" ./repo_roller_api
```

With a JSON tracing subscriber, the events include the organisation, repository, requesting user, and counts of teams and collaborators applied, skipped, and failed.
