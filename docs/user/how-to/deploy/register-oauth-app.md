---
title: "Register the GitHub OAuth App"
description: "Create the GitHub OAuth App that the RepoRoller web UI uses to authenticate end-users."
audience: "operator"
type: "how-to"
---

# Register the GitHub OAuth App

The RepoRoller frontend uses a GitHub OAuth App to authenticate end-users. This is separate from the GitHub App used for repository creation.

## Create the OAuth App

1. Go to **GitHub → Organisation Settings → Developer settings → OAuth Apps → New OAuth App**.
   (Or use **Personal → Settings → Developer settings → OAuth Apps** for personal OAuth Apps.)
2. Fill in the form:

   | Field | Value |
   |---|---|
   | **Application name** | `RepoRoller` (or your branded name) |
   | **Homepage URL** | `https://reporoller.myorg.example` |
   | **Authorization callback URL** | `https://reporoller.myorg.example/auth/callback` |

3. Click **Register application**.

> **Warning:** The **Authorization callback URL** must exactly match `${ORIGIN}/auth/callback`, where `ORIGIN` is the environment variable you set on the frontend container. A mismatch causes GitHub to reject the OAuth flow with a redirect URI mismatch error. For local testing, use `http://localhost:3001/auth/callback`.

## Collect the credentials

1. After registration, the **Client ID** is shown on the App settings page. Record this as `GITHUB_CLIENT_ID`.
2. Click **Generate a new client secret**. Record the value as `GITHUB_CLIENT_SECRET`.

> **Security:** The client secret cannot be retrieved after it is shown. Store it in your secrets manager immediately. If lost, generate a new one and update your deployment configuration.

## Related guides

- [Deploy with Docker Compose](deploy-with-docker.md) — use `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET`
- [Environment variables reference](../../reference/environment-variables.md)
