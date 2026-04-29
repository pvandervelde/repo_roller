---
title: "Automate repository creation with the CLI"
description: "Use repo-roller create in CI pipelines and scripts to create repositories programmatically."
audience: "repository-creator"
type: "how-to"
---

# Automate repository creation with the CLI

Use `repo-roller create` in automation scripts, CI pipelines, and GitHub Actions workflows where you need to create repositories without interactive input.

## Prerequisites

- `repo-roller` binary on `$PATH`
- `GITHUB_TOKEN` environment variable set to a GitHub App installation token

## Create a repository non-interactively

All `repo-roller create` commands work without prompts when you supply all required flags. No confirmation step exists in the CLI.

```bash
repo-roller create \
  --org myorg \
  --repo payment-service \
  --template rust-service \
  --description "Payment processing microservice" \
  --visibility private
```

## Pass template variables from environment

Use the `--variables` flag to provide a JSON object of variable values. Quoting syntax varies by shell:

```bash
# bash / zsh
repo-roller create \
  --org myorg \
  --repo payment-service \
  --template rust-service \
  --variables '{"service_name":"payment-service","service_port":"8080"}'
```

```powershell
# PowerShell
repo-roller create `
  --org myorg `
  --repo payment-service `
  --template rust-service `
  --variables '{"service_name":"payment-service","service_port":"8080"}'
```

## Capture JSON output for downstream steps

Use `--format json` to get a machine-readable response and parse it with `jq`:

```bash
result=$(repo-roller create \
  --org myorg \
  --repo payment-service \
  --template rust-service \
  --format json)

repo_url=$(echo "$result" | jq -r '.repository.url')
echo "Created: $repo_url"
```

## Check exit codes

`repo-roller` exits with:

| Exit code | Meaning |
|---|---|
| `0` | Repository created successfully |
| `1` | Creation failed (details in stderr) |
| `2` | Invalid arguments |

Use the exit code to detect failures in scripts:

```bash
if ! repo-roller create --org myorg --repo my-repo --template rust-service; then
  echo "Repository creation failed" >&2
  exit 1
fi
```

## GitHub Actions workflow example

```yaml
name: Create service repository

on:
  workflow_dispatch:
    inputs:
      repo_name:
        description: "Repository name"
        required: true
      service_name:
        description: "Service name (for template variables)"
        required: true

jobs:
  create-repo:
    runs-on: ubuntu-latest
    steps:
      - name: Download repo-roller
        run: |
          curl -sL https://github.com/myorg/repo_roller/releases/latest/download/repo-roller-linux-amd64 \
            -o /usr/local/bin/repo-roller
          chmod +x /usr/local/bin/repo-roller

      - name: Create repository
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_APP_TOKEN }}
        run: |
          repo-roller create \
            --org myorg \
            --repo "${{ inputs.repo_name }}" \
            --template rust-service \
            --visibility private \
            --variables "{\"service_name\":\"${{ inputs.service_name }}\"}" \
            --format json | tee result.json

      - name: Output repository URL
        run: |
          echo "Repository URL: $(jq -r '.repository.url' result.json)"
          echo "repo_url=$(jq -r '.repository.url' result.json)" >> "$GITHUB_OUTPUT"
```

## Set GITHUB_TOKEN from environment

The CLI reads the token from `GITHUB_TOKEN`. Set it before running commands:

```bash
export GITHUB_TOKEN="ghs_your_installation_token"
repo-roller create --org myorg --repo my-repo --template rust-service
```

> **Note:** Use a GitHub App installation token rather than a personal access token. Installation tokens are scoped to the organisation and expire automatically.
