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

This guide covers both sides: the administrative configuration (capability toggles,
source allow-list, settings-only templates) and, once that configuration is in place,
how a repository creator actually requests a fork or mirror via the CLI or REST API.

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

## 4. Request a fork or mirror creation

Once a capability is enabled and its source owner is allow-listed (steps 1–2 above), any
repository creator can request that mode via the CLI's `create` command or the REST API's
`POST /api/v1/repositories` endpoint. A template is optional for all three modes — it is
only ever consulted for its settings (via `designation`, step 3), never for files, with the
one exception noted above for a `source_only_mirror`-designated template.

### Fork (CLI)

```bash
repo-roller create \
  --org myorg \
  --repo payment-service \
  --fork-from upstream-org/payment-service \
  --description "Fork of upstream-org/payment-service"
```

`--fork-from` takes an `OWNER/REPO` value naming the source repository. GitHub performs the
fork server-side, so the new repository joins GitHub's fork network with full history.
`--fork-from` cannot be combined with `--mirror-from`, `--empty`, `--init-readme`, or
`--init-gitignore` — the CLI rejects the combination before any GitHub API call.

### Fork (REST API)

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service",
    "organization": "myorg",
    "creationMode": {
      "type": "fork",
      "source": { "owner": "upstream-org", "name": "payment-service" }
    },
    "description": "Fork of upstream-org/payment-service"
  }'
```

### Mirror with full history (CLI)

```bash
repo-roller create \
  --org myorg \
  --repo payment-service-mirror \
  --mirror-from upstream-org/payment-service \
  --mirror-history \
  --description "History mirror of upstream-org/payment-service"
```

With `--mirror-history`, RepoRoller clones the source as a bare mirror (all refs) and
pushes it into the new, independent repository — full commit history, `HEAD` set to the
source's default branch. The new repository is **not** part of GitHub's fork network.

### Mirror with full history (REST API)

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service-mirror",
    "organization": "myorg",
    "creationMode": {
      "type": "mirror",
      "source": { "owner": "upstream-org", "name": "payment-service" },
      "preserveHistory": true
    },
    "description": "History mirror of upstream-org/payment-service"
  }'
```

### Mirror of source files only, no history (CLI)

```bash
repo-roller create \
  --org myorg \
  --repo payment-service-snapshot \
  --mirror-from upstream-org/payment-service \
  --description "Snapshot of upstream-org/payment-service's current files"
```

Omitting `--mirror-history` produces a source-only mirror: RepoRoller takes a snapshot of
the source's current files only (no commit history) and delivers it as a pull request
against the new repository for review, rather than pushing directly.

### Mirror of source files only, no history (REST API)

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service-snapshot",
    "organization": "myorg",
    "creationMode": {
      "type": "mirror",
      "source": { "owner": "upstream-org", "name": "payment-service" },
      "preserveHistory": false
    },
    "description": "Snapshot of current files from upstream-org/payment-service"
  }'
```

Equivalently, omit `preserveHistory` entirely — it defaults to `false`.

### Notes that apply to both surfaces

- `creationMode`/`--fork-from`/`--mirror-from` are mutually exclusive with an explicit
  `contentStrategy`/`--empty`/`--init-readme`/`--init-gitignore`. The CLI rejects the
  combination via `clap`'s own conflict checking; the API rejects it with a
  `400 VALIDATION_ERROR` on the `content_strategy` field.
- A request naming a source repository whose owner is not on the resolved
  `allowed_source_owners` list, or whose capability (`fork_enabled`,
  `source_history_mirror_enabled`, `source_only_mirror_enabled`) is disabled, is rejected
  before any GitHub API call — see the Troubleshooting table below.

## Troubleshooting

| Symptom | Cause |
|---|---|
| Fork/mirror request rejected even though the capability looks enabled | Check `allowed_source_owners` at every level — an empty or unset list denies every source, independently of the `*_enabled` flags. |
| Two templates both declaring `designation = "fork"` | Only one template per organisation may declare the same `designation`. Remove or change the duplicate. |
| A template with `designation` set also forces `repository_type` | Remove the `policy = "fixed"` repository type policy from that template, or remove `designation` — the two cannot coexist on the same template. |
| Allow-listed an owner but it's still rejected | Matching is case-insensitive but exact otherwise — check for typos, and confirm which configuration level actually wins for `allowed_source_owners` (it replaces, not merges). |
| CLI rejects `--fork-from`/`--mirror-from` together with `--empty`/`--init-readme`/`--init-gitignore` | These flags are mutually exclusive — pick one content/creation strategy per request. |
| API returns `400 VALIDATION_ERROR` on `content_strategy` alongside `creationMode` | Omit `contentStrategy` entirely when `creationMode.type` is `"fork"` or `"mirror"` — it is only meaningful for `"standard"` creation. |
| `--mirror-from` request produced a pull request instead of pushing directly | Expected for a source-only mirror (`--mirror-history` not set, or `preserveHistory: false`). Add `--mirror-history` / `preserveHistory: true` for a direct-push, full-history mirror instead. |

## Related reference

- [Global configuration schema](../../reference/configuration/global-config.md#creation_capabilities--fork-and-mirror-creation-toggles)
- [Template configuration schema](../../reference/configuration/template-config.md#designation--forkmirror-settings-role)
- [How configuration is resolved](../../explanation/configuration-hierarchy.md)
- [`repo-roller create` CLI reference](../../reference/cli/create.md)
- [Repositories API reference](../../reference/api/repositories.md)
