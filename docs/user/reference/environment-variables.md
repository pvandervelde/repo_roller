---
title: "Environment variables"
description: "All environment variables consumed by the RepoRoller backend API and frontend."
audience: "operator"
type: "reference"
---

# Environment variables

## Backend (`repo_roller_api`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_APP_ID` | Yes | — | Numeric App ID from the GitHub App settings page |
| `GITHUB_APP_PRIVATE_KEY` | Yes | — | PEM private key with literal newlines replaced by `\n`. Equivalent to a password — store in a secrets manager in production. |
| `JWT_SECRET` | Yes | — | HS256 signing key for backend-issued JWTs. Minimum 32 characters. |
| `METADATA_REPOSITORY_NAME` | No | `.reporoller` | Name of the configuration repository inside the GitHub organisation |
| `API_HOST` | No | `0.0.0.0` | Network interface to bind to |
| `API_PORT` | No | `8080` | Port to listen on |
| `RUST_LOG` | No | `info` | Log level filter: `error`, `warn`, `info`, `debug`, `trace`. Supports per-module filters (e.g. `repo_roller_core=debug,info`). |

### Secret resolver variables

When using outbound notification webhooks, additional variables provide the signing secrets. The variable name is whatever you put in the `secret` field of `notifications.toml`:

```toml
# Example: secret = "REPOROLLER_WEBHOOK_SECRET"
# Then set:
REPOROLLER_WEBHOOK_SECRET=your-actual-secret-value
```

---

## Frontend (SvelteKit)

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_CLIENT_ID` | Yes | — | OAuth App client ID |
| `GITHUB_CLIENT_SECRET` | Yes | — | OAuth App client secret |
| `GITHUB_ORG` | Yes | — | GitHub organisation slug (e.g. `acme-corp`) |
| `ORIGIN` | Yes | — | Public URL of the frontend (e.g. `https://reporoller.acme.example`). Must exactly match the OAuth App callback URL base. Required by SvelteKit for CSRF protection. |
| `SESSION_SECRET` | Yes | — | HMAC-SHA256 key for signing session cookies. Minimum 32 characters. |
| `BACKEND_API_URL` | Yes | — | Base URL of the backend container, e.g. `http://backend:8080`. Must be reachable from the frontend container. Not exposed to browsers. |
| `NODE_ENV` | No | `production` | Set to `production` for container deployments |

### Branding variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `BRAND_APP_NAME` | No | `RepoRoller` | Application name shown in the page title and header |
| `BRAND_LOGO_URL` | No | *(none)* | URL for the light-mode logo image |
| `BRAND_LOGO_URL_DARK` | No | *(none)* | URL for the dark-mode logo. Has no effect unless `BRAND_LOGO_URL` is also set. |
| `BRAND_LOGO_ALT` | No | `<name> logo` | Alt text for the logo image |
| `BRAND_PRIMARY_COLOR` | No | `#0969da` | CSS accent colour (any valid CSS colour value: hex, rgb, hsl) |
| `BRAND_PRIMARY_COLOR_DARK` | No | *(none)* | Accent colour override for dark mode |

---

## Generating secrets

Generate random secret values with:

```bash
# Node.js
node -e "console.log(require('crypto').randomBytes(48).toString('hex'))"

# openssl
openssl rand -hex 48
```

Run twice to produce two independent values for `SESSION_SECRET` and `JWT_SECRET`.

---

## Converting the GitHub App private key

The downloaded `.pem` file contains literal newlines. To store it as a single-line environment variable:

```bash
# macOS / Linux
awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' your-app.private-key.pem

# PowerShell (Windows)
(Get-Content your-app.private-key.pem -Raw) -replace "`r`n","`n" -replace "`n","\\n"
```

Use the resulting single-line string as the value of `GITHUB_APP_PRIVATE_KEY`.
