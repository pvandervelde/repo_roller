# ADR-010: Fork and Mirror Repository Creation

Status: Accepted
Date: 2026-10-31
Owners: RepoRoller team

## Context

RepoRoller's existing creation flow always produces a repository from an
RepoRoller-curated template (or empty). Issue #255 asked for two new
creation types that originate content from an **arbitrary external
repository** rather than a curated template:

- **Fork** — a standard GitHub fork of a source repository.
- **Mirror** — an independent copy of a source repository, either with its
  full commit history, or with only its current file contents (delivered
  via a PR, since this isn't a GitHub fork and so has no "fork network"
  relationship to review against).

This introduced three design questions that aren't obvious from the
acceptance criteria alone and are easy to get wrong on a re-read, so they're
recorded here rather than only in `docs/spec/interfaces/fork-mirror-operations.md`
(the full interface contract — read that for complete type signatures; this
ADR is the "why", not the "what").

1. **How does a template get associated with a fork/mirror mode**, given
   templates were previously only ever picked explicitly by name for
   ordinary creation?
2. **Which repositories is an organization allowed to fork/mirror from?**
   RepoRoller's GitHub App installation token is scoped to the managed
   organization for *destination* repositories, but has no equivalent
   scoping for arbitrary *source* repositories a fork/mirror request names.
3. **Do fork/mirror templates behave like ordinary templates** (contribute
   both settings and file content), or something narrower?

## Decision

### 1. Template designation — self-declaration, not a pointer

A template declares the special role it plays via a `designation` field in
its own `.reporoller/template.toml` (`fork` | `source_history_mirror` |
`source_only_mirror`), mirroring the existing `repository_type` self-
declaration pattern rather than introducing a new "pointer" field on
`GlobalDefaults`/`RepositoryTypeConfig` that names a template by string.
At most one template per designation per organization is enforced
(`find_designated_template` errors with `AmbiguousTemplateDesignation` if
more than one template declares the same designation); when no template
declares a designation, fork/mirror creation proceeds using only
repository-type/team/global settings, with no template involved at all.

An explicitly-named template on the creation request always wins over
auto-discovery — RepoRoller never silently substitutes the organization's
discovered template over one the caller specifically asked for.

### 2. Source authorization — an explicit, fail-closed allow-list

Each of the three capabilities (`fork_enabled`, `source_history_mirror_enabled`,
`source_only_mirror_enabled`) is paired with an `allowed_source_owners` list
in the same `creation_capabilities` config section, merged through the
identical Global→RepositoryType→Team→Template precedence as the capability
flags themselves. An **empty or unset allow-list denies every source owner**,
even when the corresponding capability is enabled — enabling a capability is
necessary but not sufficient; an administrator must also explicitly name
which source owners (GitHub orgs/users) are reachable.

This was not the original design. The feature initially shipped with only
the capability toggle, on the reasoning that it mirrors the existing
template-designation pattern (an admin-curated set of sources). A security
review during this feature's implementation identified that a GitHub App
installation is frequently granted broad (often org-wide) read access, so a
single enabled capability would let any team holding it mirror or fork
*anything the installation can read* — not just repositories the requesting
team's own GitHub permissions would allow. The allow-list closes that gap by
making the *set of forkable/mirrorable sources* an explicit, administrator-
curated list, the same way templates themselves are curated, rather than
delegating authorization entirely to the installation token's reach.

### 3. Settings-only vs. content-contributing templates, by mode

A designated template's **settings** (branch protection, topics, visibility,
etc.) are applied in all three modes when one is designated. Its **file
content** is used only in the **source-only mirror** mode, as the mirror's
first commit — in Fork and source+history-mirror mode, no template file
content is ever copied, because the content itself comes entirely from the
source repository (GitHub's fork mechanism, or a full git mirror-push,
respectively). This is the one place the three modes genuinely diverge in
template behavior, and it is easy to misread as "templates never contribute
files to a fork/mirror" if you only skim the acceptance criteria — the
source-only mirror is the deliberate exception, needed so an organization
can still prepend standard scaffolding (CI config, license headers, etc.)
before the PR with the mirrored content lands.

## Consequences

**Enables:**

- Administrators can designate exactly one template per mode without a
  separate "pointer" config surface to keep in sync with the template repo.
- Enabling a capability does not, by itself, expose the installation's full
  read reach as a mirror/fork source — the allow-list bounds the blast
  radius.
- Organizations can standardize initial scaffolding on top of mirrored
  source-only content without that scaffolding leaking into Fork or
  history-mirror modes, where it would be meaningless (those modes don't
  produce an "initial commit" slot the way source-only mirror does).

**Forbids:**

- A fork/mirror request cannot reach a source repository whose owner isn't
  on the merged allow-list, regardless of what the installation token can
  technically read.
- A fork/mirror template's files are never copied in Fork or history-mirror
  mode, even if the template repository has content — only its settings
  apply there.

**Trade-offs:**

- The allow-list is **not additive** across configuration levels (highest-
  precedence level that sets it wins, same as the capability flags) rather
  than accumulating entries from every level. This was chosen for
  predictability — an administrator reading a single level's config can see
  the complete effective list without having to mentally union every lower
  level — at the cost of needing to repeat entries at a higher level if a
  narrower level also wants a subset of a broader list.
- Self-declaring `designation` on a template (rather than a central
  registry) means discovering "which template is the fork template" requires
  enumerating and loading every template in the organization
  (`find_designated_template`), not a single config lookup. This is
  consistent with how `repository_type` discovery already works and reuses
  the same caching behavior, but it is an O(templates) operation.

## Alternatives considered

### Option A: Pointer fields on org/global config instead of self-declaration

Add `fork_template`, `source_history_mirror_template`, and
`source_only_mirror_template` name fields directly to `GlobalDefaults` (or a
new config file), naming a template by string instead of having templates
declare their own role.

**Why not**: Centralizes the pointer but creates two sources of truth that
can drift (the pointer says template X, but template X's own config has
no indication it's meant to play this role) and doesn't match the existing
`repository_type` precedent this feature otherwise mirrors closely.

**When to reconsider**: If organizations report needing a template to play
different designated roles in different repository types simultaneously —
self-declaration only supports one designation per template today.

### Option B: Authorize sources via the individual requester's own GitHub permissions

Require the requesting user's own GitHub read access to the source to be
checked (e.g. a user-scoped OAuth token), rather than an admin-curated
allow-list.

**Why not**: RepoRoller's authentication model is GitHub App installation
tokens only (ADR-006) — there is no per-user token flow to check against,
and introducing one is a materially larger architecture change than this
feature warrants. The allow-list achieves the actual goal (bounding which
sources a capability can reach) without that redesign.

**When to reconsider**: If RepoRoller ever adds a user-identity-aware
request path (e.g. for a different feature), this decision should be
revisited — the allow-list and a per-user check are not mutually exclusive.

### Option C: Fork/mirror templates always contribute content, same as ordinary templates

Treat a designated template identically to an ordinary template in all
three modes — always copy its files.

**Why not**: Contradicts the acceptance criteria for Fork and
source+history mirror directly ("no files are copied from the fork
template") and would make the mirror's actual commit history/content
ambiguous with the template's. Source-only mirror is the only mode where
"what's the first commit" is even a meaningful question independent of the
source's own content.

**When to reconsider**: Not expected to be reconsidered — this is a direct
reading of the feature's acceptance criteria, not a judgment call.

## Implementation notes

**Designation TOML:**

```toml
# .reporoller/template.toml
designation = "source_only_mirror"  # or "fork" | "source_history_mirror"

[template]
name = "mirror-scaffolding"
description = "Standard scaffolding prepended to source-only mirrors"
author = "Platform Team"
tags = []
```

**Capability + allow-list TOML:**

```toml
# global/config.toml
[creation_capabilities]
source_only_mirror_enabled = true
allowed_source_owners = ["trusted-upstream-org", "another-trusted-org"]
```

An organization that sets `source_only_mirror_enabled = true` but leaves
`allowed_source_owners` unset has effectively not enabled anything usable —
this is intentional (fail closed), not a bug, and should be called out in
onboarding documentation for administrators configuring this feature for
the first time.

**Gate ordering** (`validate_creation_mode_capability`,
`crates/repo_roller_core/src/lib.rs`): capability-enabled check, then
allow-list check, then template-designation check — all three before any
GitHub API call.

## References

- Interface contract: `docs/spec/interfaces/fork-mirror-operations.md`
- Behavioral assertions: `docs/spec/assertions.md` (RCA-004/005/006,
  RCF-005/006)
- ADR-002 (Hierarchical Configuration): the four-level precedence this
  feature's new settings participate in
- ADR-006 (GitHub App Authentication): why a per-user authorization check
  wasn't a viable alternative for source authorization
- Issue #255, PR #346
