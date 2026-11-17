---
"repo_roller_api": minor
---

Adds a new read-only discovery endpoint,
`GET /api/v1/orgs/:org/creation-capabilities?team=<team>&repositoryType=<type>`, that
reports — for an organisation, optionally scoped by `team` and/or `repositoryType` —
whether fork creation, source+history-mirror creation, and source-only-mirror creation are
enabled (resolved through the existing Global → RepositoryType → Team configuration
precedence), the resolved `allowedSourceOwners` allow-list, and the designated
settings-only template name (if any) for each of the three modes.

No prior endpoint could answer this before a client composes a fork/mirror creation
request: `GET /api/v1/orgs/:org/defaults` only reflects the Global level, and
`POST /api/v1/orgs/:org/configuration/preview` requires a `template` name up front, which
fork/mirror creation intentionally makes optional. This endpoint is purely informational —
it performs zero mutations and issues no GitHub write calls — and reports a disabled
capability as `false`/`200`, never as an error. It returns `400` for an unknown
`repositoryType` query value and `409 AmbiguousTemplateDesignation` if two or more
templates in the organisation declare the same designation.

This is an additive, backward-compatible change with no impact on any existing endpoint or
request/response shape.
