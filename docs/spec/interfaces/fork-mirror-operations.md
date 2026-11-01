# Fork & Mirror Operations Specification

**Status**: Interface Design Complete
**Version**: 1.0
**Last Updated**: 2026-10-26
**Tracks**: GitHub issue #255

## Overview

Extends repository creation with two new creation modes that sit alongside the
existing template-based flow documented in
[repository-creation-modes.md](repository-creation-modes.md):

- **Fork** — a standard GitHub fork of a source repository (stays in the GitHub
  fork network). Settings only (no files) may be taken from an optional,
  explicitly designated fork template.
- **Mirror (source + history)** — an independent repository containing the
  full commit history of the source, set to the source's default-branch HEAD.
  Not part of the GitHub fork network.
- **Mirror (source only)** — an independent repository containing only the
  source's current file contents (no history), delivered via a PR for review.
  Not part of the GitHub fork network.

Both capabilities are independently toggleable (global/repository-type/team
override hierarchy) and, when enabled, may optionally use an explicitly
designated template for settings-only application.

## Architecture Context

**Layers touched**:

- `config_manager` — new settings struct, new template self-declaration field.
- `repo_roller_core` — new `CreationMode` enum, new `ContentProvider`-adjacent
  dispatch, new validation/error variants.
- `github_client` — new `RepositoryClient` methods (fork, PR create, PR
  ready-for-review) and payload/response types.
- `repo_roller_core::git` — new mirror-clone/mirror-push and
  content-only-copy helpers (local git operations, no GitHub API).

## 1. Template Designation (config_manager)

Templates self-declare the single special role they play, mirroring the
existing `repository_type: Option<RepositoryTypeSpec>` self-declaration on
`TemplateConfig` (`crates/config_manager/src/template_config.rs:110`).

```rust
/// Special role a template plays in repository creation, beyond
/// ordinary template-based creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateDesignation {
    /// This template's settings apply to newly created forks.
    Fork,
    /// This template's settings apply to source+history mirrors.
    SourceHistoryMirror,
    /// This template's settings apply to source-only mirrors; its file
    /// contents are committed as the mirror's first commit.
    SourceOnlyMirror,
}
```

`TemplateConfig` (`template_config.rs`) gains:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub designation: Option<TemplateDesignation>,
```

TOML (`.reporoller/template.toml`):

```toml
designation = "fork"  # or "source_history_mirror" | "source_only_mirror"

[template]
name = "fork-settings"
description = "Settings applied to every fork"
author = "Platform Team"
tags = []
```

**Constraint**: a template MUST NOT declare both `repository_type` policy
`Fixed`-for-a-type AND a `designation` — these are orthogonal roles and mixing
them is rejected at template-load validation time with
`ConfigurationError::InvalidConfiguration { field: "designation", reason: ".." }`
only if the template also sets `repository_type`; declaring just `designation`
alone is the common case and must not require `repository_type`.

### Discovery

Add to the component that already enumerates an org's templates for
resolution (the `OrganizationSettingsManager` /
`TemplateLoader`/`TemplateRepository` machinery described in
`docs/spec/interfaces/template-loading.md`):

```rust
/// Finds the single template designated for `designation`, if any, within
/// `org`. Returns the bare template name (config_manager has no branded
/// `TemplateName` type — `ConfigurationContext::template()` already treats
/// template identity as a plain `&str`/`String`, and this follows that
/// existing convention rather than introducing a new branded type or
/// depending on `repo_roller_core::TemplateName`, which would create a
/// circular dependency).
///
/// # Errors
/// - `ConfigurationError::AmbiguousTemplateDesignation` if more than one
///   template in `org` declares the same `designation`.
pub async fn find_designated_template(
    &self,
    org: &str,
    designation: TemplateDesignation,
) -> ConfigurationResult<Option<String>>;
```

New `ConfigurationError` variants (`crates/config_manager/src/errors.rs`),
following the existing `AmbiguousMetadataRepository` pattern:

```rust
#[error("Multiple templates declare designation '{designation:?}' for organization '{org}': {templates:?}. Expected at most one.")]
AmbiguousTemplateDesignation {
    org: String,
    designation: String,
    templates: Vec<String>,
},

#[error("Template '{template}' is not designated for '{expected_designation:?}' (actual: {actual_designation:?})")]
TemplateNotDesignated {
    template: String,
    expected_designation: String,
    actual_designation: Option<String>,
},

#[error("Capability '{capability}' is disabled by configuration")]
CapabilityDisabled { capability: String },
```

**Rejection rule (AC: "non-designated template prevented")**: if a caller
explicitly names a template for a fork/mirror request and that template's
`designation` does not match the expected one (including `None`), reject with
`TemplateNotDesignated` before any GitHub API calls are made.

Result is cached with the existing `template_configurations` TTL (5 minutes,
per `.tech-decisions.yml` caching strategy) since it is derived from template
configuration the same way other resolved template data is cached.

## 2. Enable/Disable Toggle (config_manager)

New settings struct, placed alongside `ActionSettings`
(`crates/config_manager/src/settings/`), following its exact shape:

```rust
/// Fork/mirror capability toggles with override controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreationCapabilitiesSettings {
    /// Enable GitHub fork creation.
    pub fork_enabled: Option<OverridableValue<bool>>,
    /// Enable source+history mirror creation.
    pub source_history_mirror_enabled: Option<OverridableValue<bool>>,
    /// Enable source-only mirror creation.
    pub source_only_mirror_enabled: Option<OverridableValue<bool>>,
    /// Allow-list of source owners (GitHub org/user account names,
    /// case-insensitive) permitted as `SourceRepositoryReference.owner` for
    /// fork/mirror requests. Merged through the same precedence as the
    /// `*_enabled` flags above (highest-precedence level that sets this
    /// wins, subject to `override_allowed`) — it is NOT additive across
    /// levels.
    ///
    /// **Security-critical default**: `None`/empty means no source owner is
    /// permitted, even when a capability above is `true`. An administrator
    /// must explicitly enumerate allowed source owners before fork/mirror
    /// becomes usable. This bounds the blast radius of enabling a
    /// capability: the GitHub App installation token's read reach (often
    /// org-wide) is not, by itself, sufficient authorization to mirror or
    /// fork an arbitrary repository — see threat-model T4.2/T1.2.
    pub allowed_source_owners: Option<OverridableValue<Vec<String>>>,
}
```

**Enforcement**: `validate_creation_mode_capability` (§6) checks
`allowed_source_owners` immediately after the capability-enabled check and
before the designation check, case-insensitively comparing
`source.owner.as_str()` against the merged list. Empty/unset list →
reject every source, regardless of capability state. New error:

```rust
#[error("Source owner '{owner}' is not in the configured allow-list for '{capability}'")]
SourceOwnerNotAllowed { owner: String, capability: String },
```

(`ConfigurationError` variant, same disclosure-level precedent as
`CapabilityDisabled`.)

Add `pub creation_capabilities: Option<CreationCapabilitiesSettings>` to
`GlobalDefaults`, `RepositoryTypeConfig`, `TeamConfig`, and `TemplateConfig`
(the same four structs that carry `repository`/`pull_requests`/
`branch_protection`). Merge using the same scalar-field + `OverridableValue`
precedence-and-override-enforcement pattern already implemented for
`merge_repository_settings` (`crates/config_manager/src/merger.rs:689`), not
the additive-collection pattern — add
`merge_creation_capabilities_settings` alongside it and call it from
`apply_global_defaults`, `apply_repository_type_overrides`,
`apply_team_overrides`, and `apply_template_configuration` exactly where
`merge_repository_settings` is called for each phase. `MergedConfiguration`
(`crates/config_manager/src/merged_config.rs`) gains
`pub creation_capabilities: CreationCapabilitiesSettings`.

**Default when unset at every level**: `false` (fail closed) — fork and
mirror creation must be explicitly enabled by an administrator. This differs
from `ActionSettings` (which has no documented default) and must be
implemented as the `Default` for the three fields being
`Some(OverridableValue::fixed(false))`... concretely: treat a fully-`None`
`CreationCapabilitiesSettings` at Global level as "disabled", i.e. the
resolution helper used by `repo_roller_core` must read
`merged.creation_capabilities.fork_enabled.as_ref().map(|v| *v.get()).unwrap_or(false)`
(and equivalently for the two mirror flags), not `unwrap_or(true)`.

**Enforcement point**: `repo_roller_core::create_repository` (or
`create_repository_inner`) must check the resolved capability flag for the
requested `CreationMode` immediately after configuration resolution and
before any GitHub API call, returning
`RepoRollerError::Configuration(ConfigurationError::CapabilityDisabled { capability })`
with `capability` one of `"fork"`, `"source_history_mirror"`,
`"source_only_mirror"` if disabled.

## 3. Source Repository Reference (repo_roller_core)

No new branded type is needed — `OrganizationName`'s validation rules
(`crates/repo_roller_core/src/repository.rs:89-139`: 1-39 chars, alphanumeric
+ hyphen, no leading/trailing/double hyphen) already match GitHub's shared
username/org naming rules, so it can represent the owner of an arbitrary
source repository (which may be outside the managed organization — e.g. the
upstream of a fork).

```rust
/// Identifies a repository to fork or mirror from. May reference a
/// repository outside the organization RepoRoller manages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRepositoryReference {
    pub owner: OrganizationName,
    pub name: RepositoryName,
}
```

## 4. Creation Mode (repo_roller_core)

`ContentStrategy` (`request.rs:50-68`) is left unchanged — it governs how
file content is produced for the *Standard* creation path. A new, orthogonal
enum governs *how the GitHub repository itself comes into existence*:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreationMode {
    /// Existing behavior: create (org/user) repo, then run `content_strategy`.
    Standard,

    /// Create a GitHub fork of `source`. Bypasses `content_strategy` and
    /// local git entirely — GitHub performs the fork (content + history).
    Fork { source: SourceRepositoryReference },

    /// Create an independent repository that is not part of the GitHub
    /// fork network.
    Mirror {
        source: SourceRepositoryReference,
        /// `true`: full commit history mirror, HEAD = source default branch.
        /// `false`: source-only mirror, delivered via PR, no history.
        preserve_history: bool,
    },
}

impl Default for CreationMode {
    fn default() -> Self {
        Self::Standard
    }
}
```

`RepositoryCreationRequest` (`request.rs:204`) gains:

```rust
pub creation_mode: CreationMode,
```

(`RepositoryCreationRequestBuilder` gains a `creation_mode()` setter mirroring
`content_strategy()`.)

### Validation (build-time, mirrors existing `Template`-requires-template rule)

- `CreationMode::Fork` / `CreationMode::Mirror` + `content_strategy` set to
  anything other than the `Default` (`Template` when no template, otherwise
  caller must not pass a non-default `content_strategy`) is a builder-time
  panic: `"content_strategy is not used with CreationMode::Fork/Mirror"`.
  (Content for Fork and history-preserving Mirror comes from GitHub/git, not
  a `ContentProvider`; content for source-only Mirror is derived internally —
  see §6.)
- `template`, when `Some` together with `CreationMode::Fork`, must resolve to
  a template whose `designation == Some(TemplateDesignation::Fork)` — checked
  in `create_repository_inner`, not at builder time (requires a config
  lookup). Same rule for `Mirror{preserve_history:true}` →
  `SourceHistoryMirror`, and `Mirror{preserve_history:false}` →
  `SourceOnlyMirror`.
- `template == None` is valid for all three non-Standard modes (AC: "works
  correctly when no template is configured").

## 5. GitHub Client Additions (github_client)

Add to `RepositoryClient` (`crates/github_client/src/lib.rs:2896`), following
the existing method doc style exactly:

```rust
/// Creates a fork of `source_repo` owned by `source_owner`, optionally into
/// `target_owner` (organization fork) or the authenticated identity's own
/// account when `target_owner` is `None`.
///
/// # Errors
/// - The source repository doesn't exist or is not accessible
/// - A repository with the fork's resulting name already exists at the
///   target owner
/// - The GitHub API request fails
async fn create_fork(
    &self,
    source_owner: &str,
    source_repo: &str,
    target_owner: Option<&str>,
) -> Result<Repository, Error>;

/// Opens a pull request.
///
/// # Errors
/// - Either branch does not exist
/// - A pull request between these branches already exists
/// - The GitHub API request fails
async fn create_pull_request(
    &self,
    owner: &str,
    repo: &str,
    payload: &PullRequestCreatePayload,
) -> Result<PullRequest, Error>;

/// Transitions a draft pull request to ready-for-review. No-op (returns the
/// current state) if the PR is already not a draft.
///
/// # Errors
/// - The pull request does not exist
/// - The GitHub API request fails
async fn mark_pull_request_ready_for_review(
    &self,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<PullRequest, Error>;
```

New types (same module as `RepositoryCreatePayload`/`Repository`):

```rust
#[derive(Debug, Clone, Default)]
pub struct PullRequestCreatePayload {
    pub title: String,
    pub head: String,   // branch containing the changes
    pub base: String,    // branch to merge into (typically default branch)
    pub body: Option<String>,
    pub draft: bool,
}

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub html_url: String,
    pub draft: bool,
    pub state: String,
}
```

`create_fork` wraps octocrab's `repos(owner, repo).create_fork()` (sets
`organization` when `target_owner` is `Some`); `create_pull_request` wraps
`pulls(owner, repo).create(title, head, base)`;
`mark_pull_request_ready_for_review` wraps
`pulls(owner, repo).update(pr_number).draft(false)`. Octocrab is pinned at
`0.54` in the workspace root `Cargo.toml` (note: `.tech-decisions.yml` says
`0.47` — stale, do not rely on it for the exact version).

## 6. Orchestration Dispatch (repo_roller_core)

`create_repository_inner` (`lib.rs:1206`) currently always creates the GitHub
repository via `create_org_repository`/`create_user_repository` and then runs
`generate_repository_content` → `push_to_origin`
(`lib.rs:536-589`). Add a dispatch on `request.creation_mode` *before* that
sequence:

- **`Standard`**: existing behavior, unchanged.
- **`Fork { source }`**:
  1. Resolve capability (`fork_enabled`) and, if `template` is `Some`,
     resolve + validate its `Fork` designation (§4).
  2. Call `create_fork(source.owner, source.name, Some(target_org))`.
  3. Apply resolved settings (branch protection, topics, visibility, etc. —
     the same `update_repository_settings` step `Standard` already performs)
     from the merged configuration (template settings only if a fork template
     was designated; otherwise repository-type/team/global defaults per AC).
  4. **Do not** run any `ContentProvider` — no files are copied from the fork
     template.
- **`Mirror { source, preserve_history: true }`**:
  1. Resolve capability (`source_history_mirror_enabled`) + `SourceHistoryMirror`
     designation if `template` given.
  2. Create a plain (non-fork) repository via the existing
     `create_org_repository`/`create_user_repository` path.
  3. New `git.rs` operation: mirror-clone `source` (full history,
     `git clone --mirror`) into a temp dir, then mirror-push
     (`git push --mirror`) to the new repository's remote. Default branch of
     the new repository must end up equal to `source`'s default branch HEAD.
  4. Apply resolved settings as in Fork (template settings only, no file
     content from the template).
- **`Mirror { source, preserve_history: false }`** (source-only):
  1. Resolve capability (`source_only_mirror_enabled`) + `SourceOnlyMirror`
     designation if `template` given.
  2. Create the repository; if `template` is `Some`, run
     `TemplateBasedContentProvider` and push its output as the **first
     commit** (reusing the existing `generate_repository_content` +
     `push_to_origin` path with `ContentStrategy::Template`); if `template`
     is `None`, push an empty initial commit (reuse
     `ZeroContentProvider`/`push_to_origin`, matching `ContentStrategy::Empty`
     behavior).
  3. New `git.rs` operation: create a branch off the new repository's default
     branch, copy `source`'s current working-tree file contents onto it
     (no history — a plain tree snapshot, not a merge/cherry-pick), commit,
     push the branch.
  4. Call `create_pull_request` (head = the new branch, base = default
     branch) then `mark_pull_request_ready_for_review` — the PR must be
     opened as ready-for-review, not as a draft (`draft: false` on create is
     sufficient; the explicit ready-for-review call exists for templates/API
     versions that default PRs to draft).

### Testability seam (required for unit-level capability/designation gate tests)

`create_github_repository`, `create_repository_inner`, and their helpers
(`crates/repo_roller_core/src/lib.rs:672`, `958`, `1011`, `1206`) currently
take `installation_repo_client: &GitHubClient` (concrete type), even though
`GitHubClient` already implements `RepositoryClient`. To unit-test that the
capability gate (RCF-005) and designation gate (RCF-006) reject before any
GitHub call — without a live test org — change these parameters to
`&dyn RepositoryClient` (the trait is already `Send + Sync` and
object-safe). This is a pure widening (every current call site passes a
`&GitHubClient`, which already satisfies `&dyn RepositoryClient`) — no
behavior change for the `Standard` path, and it lets tests inject the mock
`RepositoryClient` from `crates/test_utils/` and assert zero calls were made
when a request is rejected.

## 7. Testing Requirements

### Unit tests

- `TemplateDesignation` serde round-trip for all three variants.
- `CreationCapabilitiesSettings` merge precedence (global → repository-type →
  team → template), including `override_allowed = false` rejection, mirroring
  existing `merge_repository_settings` test structure.
- `find_designated_template`: zero matches → `None`; one match → `Some`; two+
  matches → `AmbiguousTemplateDesignation`.
- `CreationMode` builder validation: panics when `content_strategy` is
  non-default alongside `Fork`/`Mirror`.
- `SourceRepositoryReference` construction validation (delegates to existing
  `OrganizationName`/`RepositoryName` validation — no new rules).

### Integration tests

- Fork creation: with designated fork template (settings applied, no files
  copied) and without (repository-type/team/global settings applied).
- Source+history mirror: full history present, HEAD matches source default
  branch, repository not in the fork network, with/without designated
  template.
- Source-only mirror: PR opened with source contents, PR not draft, initial
  commit from template (when configured) or empty, with/without designated
  template.
- Each capability disabled → `CapabilityDisabled` error, no GitHub API calls
  made (verify via mock call-count assertions).
- Explicit non-designated template passed for fork/mirror → `TemplateNotDesignated`,
  no GitHub API calls made.
- Capability enabled but `source.owner` absent from `allowed_source_owners`
  (including the all-levels-unset case) → `SourceOwnerNotAllowed`, no GitHub
  API calls made. Allow-list match is case-insensitive.
- Source-only mirror's working-tree copy (`copy_dir_contents`) rejects or
  skips symlink entries rather than dereferencing them — regression test:
  a cloned source tree containing a symlink to a path outside the clone must
  not have that target's content appear in the resulting commit/PR.

## 8a. CLI and API Request Surface

The domain model (§4) and its enforcement (§6) exist, but nothing yet lets
a caller actually select `CreationMode::Fork`/`Mirror` — this section closes
that gap.

### CLI (`crates/repo_roller_cli/src/commands/create_cmd.rs`)

Add to `CreateArgs` (alongside the existing `empty`/`init_readme`/
`init_gitignore` flags at ~line 128), following the same
`conflicts_with_all` pattern:

```rust
/// Fork an existing repository (format: "owner/repo").
#[arg(long, value_name = "OWNER/REPO", conflicts_with_all = ["mirror_from", "empty", "init_readme", "init_gitignore"])]
pub fork_from: Option<String>,

/// Mirror an existing repository (format: "owner/repo").
#[arg(long, value_name = "OWNER/REPO", conflicts_with_all = ["fork_from", "empty", "init_readme", "init_gitignore"])]
pub mirror_from: Option<String>,

/// With --mirror-from, preserve full commit history (source+history mirror).
/// Without it, only current file contents are mirrored via a PR (source-only mirror).
#[arg(long, requires = "mirror_from")]
pub mirror_history: bool,
```

In `handle_create_command`, parse `"owner/repo"` (split on exactly one `/`,
reject zero or multiple slashes with `Error::InvalidArguments`) into
`OrganizationName`/`RepositoryName` using the exact same validation calls
already used for `--team`/`--collaborator` parsing, assemble a
`SourceRepositoryReference`, and call
`builder.creation_mode(CreationMode::Fork { source })` or
`CreationMode::Mirror { source, preserve_history: args.mirror_history }`.
Do not call `.content_strategy(...)` when a fork/mirror flag is set (already
enforced by the `conflicts_with_all` lists above — the builder's
content_strategy/creation_mode panic guard, `request.rs`, is now unreachable
from the CLI by construction, not just by convention).

### REST API (`crates/repo_roller_api/src/models/request.rs`, `src/translation.rs`)

Add to `CreateRepositoryRequest`:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub creation_mode: Option<repo_roller_core::CreationMode>,
```

Reusing the core enum directly (not a separate HTTP-layer copy) — this
matches the existing precedent of embedding `content_strategy:
ContentStrategy` directly from `repo_roller_core` in this same struct.
`CreationMode`'s own `#[serde(tag = "type", rename_all = "snake_case")]`
already defines the wire shape:

```json
{ "creationMode": { "type": "fork", "source": { "owner": "upstream-org", "name": "some-repo" } } }
{ "creationMode": { "type": "mirror", "source": { "owner": "upstream-org", "name": "some-repo" }, "preserveHistory": true } }
```

**Correction (found during RED)**: `OrganizationName`/`RepositoryName`
derive `Deserialize` directly on their tuple struct, which bypasses `::new()`
validation entirely — a malformed `source.owner`/`source.name` string
deserializes successfully with the invalid value intact. The HTTP boundary
MUST explicitly re-validate `creation_mode`'s nested `source.owner`/
`source.name` inside `http_create_repository_request_to_domain`, calling
`OrganizationName::new`/`RepositoryName::new` on the raw strings and mapping
failures to `ApiError::validation_error`, exactly like the existing `name`/
`owner`/`template` fields on the same struct already do. This is not
optional — do not skip it on the assumption deserialize-time validation
already covers it.

In `http_create_repository_request_to_domain`
(`translation.rs:36-157`): if `creation_mode` is `Some` and not
`CreationMode::Standard`, validate that `content_strategy` was NOT also
explicitly set to a non-default value — return
`ApiError::validation_error("content_strategy", "not valid with a fork/mirror creation_mode")`
rather than letting the builder panic (API handlers must never panic on
caller input — this is a required check here specifically because the
builder's own guard is a `panic!`, not a `Result`, per `request.rs`).
Otherwise, pass `creation_mode` straight to `builder.creation_mode(...)`.

### Testing Requirements (addendum to §7)

- CLI: `--fork-from owner/repo`, `--mirror-from owner/repo` (with and
  without `--mirror-history`), malformed `owner/repo` (no slash, multiple
  slashes, invalid owner/repo chars) → `Error::InvalidArguments`;
  `conflicts_with_all` enforcement (clap-level, verify via its own
  generated error, not custom code).
- API: valid `creationMode` JSON for all three variants round-trips to the
  correct domain `CreationMode`; invalid nested `source.owner`/`source.name`
  → `ApiError::validation_error` (not a panic, not a 500); `creationMode`
  fork/mirror + explicit non-default `content_strategy` in the same request
  → `ApiError::validation_error`, no builder panic reaches the handler.

## 8. Related Specifications

- [Repository Creation Modes](repository-creation-modes.md) — `ContentStrategy`
  and `RepositoryCreationRequest`, left structurally unchanged by this spec.
- [Content Providers](content-providers.md) — `ContentProvider` trait, reused
  unmodified for the source-only mirror's template-or-empty initial commit.
- [Template Loading](template-loading.md) — template enumeration machinery
  extended by `find_designated_template`.
- [GitHub Interfaces](github-interfaces.md) — `RepositoryClient` trait
  extended by §5.

## Change History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-10-26 | Initial specification for fork and mirror creation (issue #255) |
