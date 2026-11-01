---
"config_manager": minor
"repo_roller_core": minor
"github_client": minor
"repo_roller_api": minor
"repo_roller_cli": minor
---

RepoRoller's configuration layer now understands two new repository-creation modes
alongside ordinary template-based creation: **forks** (a standard GitHub fork of a source
repository, staying in GitHub's fork network) and **mirrors** — either a full-history copy
of a source repository (`source_history_mirror`, set to the source's default-branch HEAD)
or a source-only copy of just the source's current files delivered via a pull request for
review (`source_only_mirror`), neither of which joins GitHub's fork network.

**Template designation**: a template's own `.reporoller/template.toml` can now declare
`designation = "fork"`, `"source_history_mirror"`, or `"source_only_mirror"` to become the
single, organisation-wide settings-only template for that mode (at most one per
organisation, per designation). A designated template contributes settings only — never
files — with one exception: a `source_only_mirror` template's files become the mirror's
first commit. `designation` cannot be combined with a `[repository_type]` section whose
`policy = "fixed"`.

**Capability toggles and source allow-list**: a new `[creation_capabilities]` configuration
section (available at global, repository-type, team, and template level, with the usual
override hierarchy) independently enables or disables each of the three modes via
`fork_enabled`, `source_history_mirror_enabled`, and `source_only_mirror_enabled`, plus a
case-insensitive `allowed_source_owners` allow-list restricting which GitHub organisations
or users may be used as a fork/mirror source. All four fields are **fail-closed by
default**: with nothing configured, all three modes stay disabled and no source owner is
permitted, even if a capability is later turned on — an administrator must explicitly opt
in to each mode and explicitly allow-list each source owner before either repository or
request path accepts it.

This release ships the domain model, configuration schema, and `github_client` primitives
(`create_fork`, `create_pull_request`, `mark_pull_request_ready_for_review`) needed for fork
and mirror creation, plus the `repo_roller_core` dispatch and capability/designation/
allow-list enforcement that runs ahead of any GitHub API call. There is no existing
behavior change for standard template-based, empty, or custom-init repository creation —
every new field and section defaults to its current/disabled behavior when absent.

**Request-time surface**: `repo_roller_cli`'s `create` command gains three new flags —
`--fork-from <OWNER/REPO>` and `--mirror-from <OWNER/REPO>` to select a fork or mirror
source, and `--mirror-history` to request a full-history mirror (direct push) instead of
the default source-only mirror (delivered via a pull request). Both source flags are
mutually exclusive with each other and with `--empty`/`--init-readme`/`--init-gitignore`.
`repo_roller_api`'s `POST /api/v1/repositories` gains an optional `creationMode` JSON
field (`{"type": "fork", "source": {"owner": ..., "name": ...}}` or `{"type": "mirror",
"source": {...}, "preserveHistory": bool}`) that reuses `repo_roller_core::CreationMode`
directly; omitting it (or sending `{"type": "standard"}`) preserves today's
template/empty/custom-init behavior exactly, so existing integrations are unaffected.
Both surfaces are gated by the same fail-closed capability/allow-list enforcement
described above — a disabled capability or non-allow-listed source owner is rejected
before any GitHub API call, from either the CLI or the API.

`repo_roller_cli`'s `template validate` command additionally recognizes `designation` and
`creation_capabilities` as known `.reporoller/template.toml` keys, so templates using them
no longer trigger a spurious "unknown key" warning.
