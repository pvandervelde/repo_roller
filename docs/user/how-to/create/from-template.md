---
title: "Create a repository from a template"
description: "Create a new GitHub repository from an organisation template using the web UI, CLI, or REST API."
audience: "repository-creator"
type: "how-to"
---

# Create a repository from a template

## Web UI

1. Sign in at your RepoRoller URL.
2. Click **Create Repository**.
3. Select a template from the grid and click **Next**.
4. Enter the repository name, optional description, and visibility. Click **Next**.
5. Fill in the template variables. Click **Create Repository**.
6. When the success screen appears, click **Open on GitHub**.

## CLI

```bash
repo-roller create \
  --org myorg \
  --repo payment-service \
  --template rust-service \
  --description "Payment processing microservice" \
  --visibility private \
  --variable service_name="payment-service" \
  --variable service_port="8080"
```

**Key flags:**

| Flag | Required | Description |
|---|---|---|
| `--org ORG` | Yes | GitHub organisation |
| `--repo NAME` | Yes | Repository name |
| `--template TMPL` | Yes | Template repository name |
| `--description DESC` | No | Repository description |
| `--visibility` | No | `private` (default) or `public` |
| `--repository-type TYPE` | No | Repository type override |
| `--team TEAM` | No | Team slug for team-level configuration |
| `--variable KEY=VALUE` | No | Template variable; repeat for each variable |

See [CLI Reference: repo-roller create](../../reference/cli/create.md) for all flags.

## REST API

```bash
curl -X POST https://reporoller.myorg.example/api/v1/repositories \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service",
    "organization": "myorg",
    "template": "rust-service",
    "description": "Payment processing microservice",
    "visibility": "private",
    "repositoryType": "service",
    "variables": {
      "service_name": "payment-service",
      "service_port": "8080"
    }
  }'
```

The `contentStrategy` field defaults to `"template"` when `template` is provided. You can omit it.

See [Repositories API](../../reference/api/repositories.md) for the full request schema and response format.
