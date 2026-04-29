---
title: "`repo-roller create` — create a repository"
description: "Full flag reference for the repo-roller create command."
audience: "all"
type: "reference"
---

# `repo-roller create` — create a repository

Creates a new GitHub repository using a template, as an empty repository, or with minimal initialisation.

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
