---
title: "Register the GitHub App"
description: "Create the GitHub App that RepoRoller uses as its service identity for creating repositories and reading organisation settings."
audience: "operator"
type: "how-to"
---

# Register the GitHub App

RepoRoller uses a GitHub App — not a personal access token — to create repositories. The App acts as the tool's service identity: its actions appear in audit logs as the App, not as an individual user.

## Create the App

1. Go to **GitHub → Organisation Settings → Developer settings → GitHub Apps → New GitHub App**.
   (Or use your personal account settings if you are not creating an organisation-owned App.)
2. Fill in the registration form:
   - **GitHub App name**: Choose a unique name, for example `RepoRoller-myorg`.
   - **Homepage URL**: Your deployment URL, e.g. `https://reporoller.myorg.example`.
   - **Webhook**: uncheck **Active** — RepoRoller does not receive webhooks from GitHub.

## Required permissions

Under **Repository permissions**, grant:

| Permission | Access |
|---|---|
| Administration | Read & write |
| Contents | Read & write |
| Metadata | Read-only |
| Pull requests | Read & write |

Under **Organization permissions**, grant:

| Permission | Access |
|---|---|
| Members | Read-only |

## Finish registration

1. Set **Where can this GitHub App be installed?** to **Only on this account** unless you need multi-organisation support.
2. Click **Create GitHub App**.

## Collect the App ID

After creation, the App ID is shown at the top of the App settings page. Record this as `GITHUB_APP_ID`.

## Generate and save the private key

1. Scroll to the **Private keys** section on the App settings page.
2. Click **Generate a private key**. A `.pem` file is downloaded.
3. Keep this file — it is the App's credential.

**Convert the PEM file to a single-line string** for use as an environment variable:

```bash
# macOS / Linux
awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' your-app.private-key.pem
```

```powershell
# PowerShell (Windows)
(Get-Content your-app.private-key.pem -Raw) -replace "`r`n","`n" -replace "`n","\\n"
```

The output string is `GITHUB_APP_PRIVATE_KEY`. Store it in your secrets manager.

> **Security:** The private key is equivalent to a password. Never commit it to source control. Store it in Azure Key Vault, AWS Secrets Manager, HashiCorp Vault, or an equivalent system.

## Install the App into your organisation

1. On the App settings page, click **Install App**.
2. Select your organisation.
3. Set **Repository access** to **All repositories** (required so RepoRoller can create new repositories and read `the .reporoller` metadata repository).
4. Note the **Installation ID** from the URL after installation: `https://github.com/organizations/myorg/settings/installations/{INSTALLATION_ID}`.

## Related guides

- [Deploy with Docker Compose](deploy-with-docker.md) — use `GITHUB_APP_ID` and `GITHUB_APP_PRIVATE_KEY`
- [Environment variables reference](../../reference/environment-variables.md)
