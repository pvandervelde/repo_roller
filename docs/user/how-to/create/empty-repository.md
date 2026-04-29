---
title: "Create an empty repository"
description: "Create a new GitHub repository with no initial files, applying only organisation-wide security and configuration policies."
audience: "repository-creator"
type: "how-to"
---

# Create an empty repository

Use an empty repository when:

- You are importing an existing codebase from another VCS
- You need a blank slate with no template constraints
- You are creating infrastructure-only or experimental repositories

Empty repositories still apply all organisation-wide settings (branch protection, team assignments, labels) from the configuration hierarchy. Only the template files are absent.

## CLI

```bash
repo-roller create \
  --org myorg \
  --repo my-service \
  --empty \
  --description "Empty repository for code import" \
  --visibility private
```

Combine with `--repository-type` and `--team` to apply type and team-level configuration:

```bash
repo-roller create \
  --org myorg \
  --repo my-service \
  --empty \
  --repository-type service \
  --team platform \
  --description "Empty service repository with platform settings"
```

## REST API

Set `contentStrategy` to `"empty"` and omit the `template` field:

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-service",
    "organization": "myorg",
    "contentStrategy": "empty",
    "description": "Empty repository for code import",
    "visibility": "private",
    "repositoryType": "service",
    "team": "platform"
  }'
```

## What gets applied

Even though the repository is empty, RepoRoller applies:

- Repository feature settings (`has_issues`, `has_wiki`, etc.) from the configuration hierarchy
- Branch-protection rulesets from global, type, and team configuration
- Default team assignments and collaborators
- Labels from global and type configuration

It does **not** apply:

- Template files
- Template-specific variables, labels, webhooks, or rulesets

## Related guides

- [Create a repository with README and .gitignore](with-readme-and-gitignore.md) — for minimal initial content
- [Create a repository from a template](from-template.md) — for full template-based creation
