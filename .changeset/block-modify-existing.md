---
"repo_roller_core": patch
"repo_roller_api": patch
"github_client": minor
"config_manager": minor
---

Repository creation now authoritatively refuses to touch an existing repository under any
creation mode, closing a gap where retries, race conditions, or a skipped/degraded
client-side check could previously reach repository-mutating API calls. Immediately before
dispatching to the fork, mirror, or standard creation path, `repo_roller_core` now performs
an authoritative GitHub existence check and fails the request with `409
RepositoryAlreadyExists` if the target repository already exists — this does not depend on
whether `POST /repositories/validate-name` was called first, or what it reported; that
endpoint remains a best-effort, fail-open UX helper, not the source of truth. A repository
created by someone else in the brief window between the check and this request's own create
call is now also detected (via GitHub's own "already exists" response) and mapped to the
same `409` outcome, instead of a generic network error.

Also: fork-mode creation requests must specify the same `name` as the source repository —
GitHub's fork-creation API cannot rename a repository on creation, so a mismatched name
would previously risk the existence check validating the wrong identifier. Mismatched names
are now rejected up front with a new `400 ForkTargetNameMismatch` error, before any GitHub
API call, instead of silently targeting the wrong repository. This is a known rough edge in
the current request shape (which exposes the repository name independently of the fork
source) — see [issue #378](https://github.com/pvandervelde/repo_roller/issues/378) for the
planned follow-up to remove the constraint by construction instead of by validation.

**Note on bump types**: `github_client` and `config_manager` are bumped `minor` rather than
`patch`, even though this change is additive validation/bugfix hardening from a user's
perspective. `github_client::RepositoryClient` (a public trait) gained a new required
method, `get_repository`, with no default implementation — a breaking change for any
external implementor of that trait, consistent with how this project versioned the
equivalent `RepositoryClient` trait-widening change in
`.changeset/widen-repositoryclient-trait-for-config-dispatch.md`. `github_client::Error` and
`config_manager::ConfigurationError` are both plain (non-`#[non_exhaustive]`) public enums
that gained a new variant (`Error::AlreadyExists` and
`ConfigurationError::ForkTargetNameMismatch` respectively), which is likewise breaking for
any external code that exhaustively matches on them — the same treatment this project gave
the new `ConfigurationError` variants added alongside fork/mirror creation in
`.changeset/add-fork-mirror-repository-creation.md`. `repo_roller_core` and `repo_roller_api`
have no corresponding public API surface change (the new gating functions are private, and
the new HTTP error code is purely additive on the wire), so they remain `patch`.
