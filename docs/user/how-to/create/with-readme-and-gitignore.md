---
title: "Create a repository with README and .gitignore"
description: "Create a new GitHub repository with minimal initial files using the custom_init content strategy."
audience: "repository-creator"
type: "how-to"
---

# Create a repository with README and .gitignore

Use the `custom_init` strategy when you want a repository with a generated `README.md`, a language-appropriate `.gitignore`, or both — but without the full structure of a template.

## CLI

```bash
# With both README and .gitignore
repo-roller create \
  --org myorg \
  --repo my-python-project \
  --init-readme \
  --init-gitignore \
  --description "Python data pipeline"

# README only
repo-roller create \
  --org myorg \
  --repo my-project \
  --init-readme \
  --description "My project"

# .gitignore only
repo-roller create \
  --org myorg \
  --repo my-project \
  --init-gitignore
```

Combine with `--repository-type` and `--team`:

```bash
repo-roller create \
  --org myorg \
  --repo my-library \
  --init-readme \
  --init-gitignore \
  --repository-type library \
  --team backend \
  --visibility private
```

## REST API

Set `contentStrategy` to `"custom_init"` and use `initializeReadme` and `initializeGitignore` to select the initial files:

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-python-project",
    "organization": "myorg",
    "contentStrategy": "custom_init",
    "initializeReadme": true,
    "initializeGitignore": true,
    "description": "Python data pipeline",
    "visibility": "private",
    "repositoryType": "library"
  }'
```

## Generated README content

The generated `README.md` uses the repository name and description:

```markdown
# my-python-project

Python data pipeline
```

## What gets applied

As with any repository strategy, the full configuration hierarchy (branch protection, team assignments, labels) is applied. The template-specific sections of the hierarchy are not applicable since no template is selected.

## Related guides

- [Create an empty repository](empty-repository.md) — no initial files at all
- [Create a repository from a template](from-template.md) — full template-based creation
