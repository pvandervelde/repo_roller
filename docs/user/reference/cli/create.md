---
title: "`repo-roller create` — create a repository"
description: "Full flag reference for the repo-roller create command."
audience: "all"
type: "reference"
---

# `repo-roller create` — create a repository

Creates a new GitHub repository using a template, as an empty repository, with minimal
initialisation, as a fork of an existing repository, or as a mirror of an existing
repository's contents.

## Synopsis

```
repo-roller create --org <ORGANIZATION> --repo <NAME> [OPTIONS]
```

## Flags

| Flag | Type | Required | Default | Description |
|---|---|---|---|---|
| `--org <ORG>` | string | Yes | — | GitHub organisation in which to create the repository |
| `--repo <NAME>` | string | Yes | — | Repository name. 1–100 characters; lowercase letters, numbers, hyphens, underscores, periods. Cannot start with `.` or `-`. |
| `--template <TMPL>` | string | Conditional | — | Name of the template repository. Required when `--empty` and `--init-readme`/`--init-gitignore` are not set. |
| `--empty` | flag | No | — | Create an empty repository with no files. Mutually exclusive with `--template`. |
| `--init-readme` | flag | No | — | Seed the repository with a generated `README.md`. Can be combined with `--init-gitignore`. Mutually exclusive with `--template` and `--empty`. |
| `--init-gitignore` | flag | No | — | Seed the repository with a `.gitignore`. Can be combined with `--init-readme`. Mutually exclusive with `--template` and `--empty`. |
| `--fork-from <OWNER/REPO>` | string | No | — | Fork an existing repository. Format `owner/repo` (exactly one `/`). Mutually exclusive with `--mirror-from`, `--empty`, `--init-readme`, `--init-gitignore`. |
| `--mirror-from <OWNER/REPO>` | string | No | — | Mirror an existing repository. Format `owner/repo` (exactly one `/`). Mutually exclusive with `--fork-from`, `--empty`, `--init-readme`, `--init-gitignore`. |
| `--mirror-history` | flag | No | — | With `--mirror-from`, preserve full commit history (source+history mirror, direct push). Without it, only the source's current file contents are mirrored via a pull request for review (source-only mirror). Requires `--mirror-from`. |
| `--description <DESC>` | string | No | — | Repository description shown on GitHub |
| `--visibility <VIS>` | string | No | `private` | Repository visibility: `private` or `public`. Subject to organisation policy. |
| `--repository-type <TYPE>` | string | No | — | Repository type name for applying type-level configuration (e.g. `library`, `service`) |
| `--team <TEAM>` | string | No | — | Team slug for applying team-level configuration |
| `--variables <JSON>` | string | No | — | JSON object of template variable values, e.g. `'{"service_name":"my-svc"}'`. Only valid with `--template`. |
| `--format <FMT>` | string | No | `pretty` | Output format: `pretty` or `json` |

## Content strategies

Exactly one of the following must be specified:

| Strategy | Flags | Description |
|---|---|---|
| Template | `--template <NAME>` | Copy files from a template repository, substitute variables |
| Empty | `--empty` | No files; organisation defaults only |
| Custom init | `--init-readme` and/or `--init-gitignore` | Minimal files only; organisation defaults applied |

## Creation modes (fork and mirror)

`--fork-from` and `--mirror-from` select a creation mode that bypasses the content
strategies above entirely — GitHub (for a fork) or RepoRoller's own git mirroring (for a
mirror) supplies the repository's contents instead. A template is optional with either
flag and, if given, only ever contributes its *settings* (branch protection, labels,
etc.), never files. Both flags require the target capability
(`fork_enabled`/`source_history_mirror_enabled`/`source_only_mirror_enabled`) to be
enabled and the source owner to be allow-listed — see
[Configure fork and mirror repository creation](../../how-to/configure/fork-and-mirror-creation.md).

| Mode | Flags | Description |
|---|---|---|
| Fork | `--fork-from <OWNER/REPO>` | Create a standard GitHub fork of the source repository. Joins GitHub's fork network; full history. |
| Mirror (source + history) | `--mirror-from <OWNER/REPO> --mirror-history` | Create an independent copy of the source repository with full commit history, `HEAD` set to the source's default branch. Pushed directly. |
| Mirror (source only) | `--mirror-from <OWNER/REPO>` | Create an independent repository seeded from the source's current files only (no history), delivered via a pull request for review. |

## Examples

### Create from a template

```bash
repo-roller create \
  --org myorg \
  --repo payment-service \
  --template rust-service \
  --description "Payment processing microservice" \
  --visibility private \
  --repository-type service \
  --team payments-team \
  --variables '{"service_name":"payment-service","service_port":"8080"}'
```

### Create an empty repository

```bash
repo-roller create \
  --org myorg \
  --repo legacy-importer \
  --empty \
  --description "Import target for legacy codebase migration"
```

### Create with README and .gitignore

```bash
repo-roller create \
  --org myorg \
  --repo python-scripts \
  --init-readme \
  --init-gitignore \
  --description "Utility scripts" \
  --repository-type library
```

### Fork an existing repository

```bash
repo-roller create \
  --org myorg \
  --repo payment-service \
  --fork-from upstream-org/payment-service \
  --description "Fork of upstream-org/payment-service"
```

### Mirror an existing repository (full history)

```bash
repo-roller create \
  --org myorg \
  --repo payment-service-mirror \
  --mirror-from upstream-org/payment-service \
  --mirror-history \
  --description "History mirror of upstream-org/payment-service"
```

### Mirror an existing repository (source files only, via PR)

```bash
repo-roller create \
  --org myorg \
  --repo payment-service-snapshot \
  --mirror-from upstream-org/payment-service \
  --description "Snapshot of upstream-org/payment-service"
```

### Get JSON output

```bash
repo-roller create \
  --org myorg \
  --repo my-repo \
  --template rust-service \
  --format json
```

Response shape:

```json
{
  "repository": {
    "name": "my-repo",
    "fullName": "myorg/my-repo",
    "url": "https://github.com/myorg/my-repo",
    "visibility": "private"
  },
  "appliedConfiguration": {
    "features": {},
    "branchProtection": {},
    "labels": []
  }
}
```
