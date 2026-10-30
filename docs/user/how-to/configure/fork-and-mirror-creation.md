---
title: "Configure fork and mirror repository creation"
description: "Enable fork and mirror creation modes, restrict which source repositories may be used, and designate settings-only templates for them."
audience: "platform-engineer"
type: "how-to"
---

# Configure fork and mirror repository creation

RepoRoller supports three ways to create a repository beyond the standard
template/empty/custom-init flow:

| Mode | What it creates | Part of GitHub's fork network? | File history |
|---|---|---|---|
| **Fork** | A standard GitHub fork of a source repository | Yes | Full history (GitHub performs the fork) |
| **Mirror (source + history)** | An independent copy of the source repository | No | Full history, `HEAD` set to the source's default branch |
| **Mirror (source only)** | An independent repository seeded from the source's current files, delivered via a pull request for review | No | None — a single snapshot commit, no source history |

All three are **disabled by default** and must be explicitly turned on by an administrator.
Each one is controlled independently, so you can enable forks without enabling mirrors, or
enable only the source-only mirror, and so on.

> **Current limitation:** as of this release, enabling and configuring these modes is only
> possible through the configuration files described below. There is no CLI flag, REST API
> field, or web UI control yet for actually *requesting* a fork or mirror creation — that
> request-time surface has not shipped. This guide covers the administrative configuration
> that is already in place and ready for that surface when it arrives.

## 1. Enable the capabilities you want

Add a `[creation_capabilities]` section to any configuration level — global, repository
type, team, or template — following the same override hierarchy as every other setting
(Template > Team > Repository type > Global).

```toml
# global/defaults.toml
[creation_capabilities]
fork_enabled                   = true
source_history_mirror_enabled  = true
source_only_mirror_enabled     = false
```

Leaving a field unset at every level means that capability stays **disabled** — this is the
opposite default from most other settings in this system (compare to `[actions] enabled`,
which defaults to `true`). See [Global configuration schema: `[creation_capabilities]`](../../reference/configuration/global-config.md#creation_capabilities--fork-and-mirror-creation-toggles)
for the full field list.

To lock a capability so lower levels cannot re-enable it after an administrator disables it
(or vice versa):

```toml
[creation_capabilities]
fork_enabled = { value = false, override_allowed = false }
```

## 2. Allow-list the source repositories' owners

Enabling a capability is not enough on its own. Every fork/mirror request names a *source*
repository, and the GitHub account or organisation that owns that source repository must
appear in `allowed_source_owners` — at whichever configuration level ultimately wins —
before the request is accepted.

```toml
# global/defaults.toml
[creation_capabilities]
fork_enabled           = true
allowed_source_owners  = ["upstream-org", "some-vendor-org"]
```

- Matching is **case-insensitive**.
- `allowed_source_owners` is unset (empty) by default, which denies every source owner —
  even if `fork_enabled` is `true`. This is intentional: your GitHub App installation token
  often has read access across the whole organisation, and that reach is not by itself
  treated as authorization to fork or mirror an arbitrary repository.
- Unlike `[[labels]]` or `[[webhooks]]`, this list is **not additive** across configuration
  levels. The highest-precedence level that sets `allowed_source_owners` replaces the list
  entirely; it does not merge with lists set at lower levels.

A request whose source repository is owned by an account not on the resolved allow-list is
rejected before any GitHub API call is made, regardless of capability state.

## 3. (Optional) Designate a settings-only template

You can optionally designate one template per organisation to supply **settings** (not
files) for each mode, via a top-level `designation` field in that template's own
`.reporoller/template.toml`:

```toml
# .reporoller/template.toml, inside the template repository
designation = "fork"   # or "source_history_mirror" | "source_only_mirror"

[template]
name        = "fork-settings"
description = "Branch protection and labels applied to every fork"
author      = "Platform Team"
tags        = []

[repository]
wiki = false

[pull_requests]
required_approving_review_count = 1
```

Rules:

- **At most one** template per organisation may declare a given `designation` value. If two
  or more templates declare the same one, resolving it fails until the duplicate is removed.
- A designated template contributes only its settings sections (`[repository]`,
  `[pull_requests]`, labels, rulesets, etc.) — **never its files** — with one exception:
  a `source_only_mirror`-designated template's file contents become the mirror's **first
  commit**. Fork and source+history mirror templates never contribute files, because
  GitHub (for forks) or the git mirror operation (for history mirrors) already supplies the
  full file set from the source.
- `designation` cannot be combined with a `[repository_type]` section whose
  `policy = "fixed"` — the two are mutually exclusive roles for a template. Combining
  `designation` with `policy = "preferable"`, or omitting `[repository_type]` entirely, is
  fine.
- A designated template is optional. If none is designated for a mode, that mode falls back
  to the repository-type/team/global settings, same as any other repository.

See [Template configuration schema: `designation`](../../reference/configuration/template-config.md#designation--forkmirror-settings-role)
for the complete field reference.

## Troubleshooting

| Symptom | Cause |
|---|---|
| Fork/mirror request rejected even though the capability looks enabled | Check `allowed_source_owners` at every level — an empty or unset list denies every source, independently of the `*_enabled` flags. |
| Two templates both declaring `designation = "fork"` | Only one template per organisation may declare the same `designation`. Remove or change the duplicate. |
| A template with `designation` set also forces `repository_type` | Remove the `policy = "fixed"` repository type policy from that template, or remove `designation` — the two cannot coexist on the same template. |
| Allow-listed an owner but it's still rejected | Matching is case-insensitive but exact otherwise — check for typos, and confirm which configuration level actually wins for `allowed_source_owners` (it replaces, not merges). |

## Related reference

- [Global configuration schema](../../reference/configuration/global-config.md#creation_capabilities--fork-and-mirror-creation-toggles)
- [Template configuration schema](../../reference/configuration/template-config.md#designation--forkmirror-settings-role)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
