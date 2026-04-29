---
title: "Authentication and authorization"
description: "How the GitHub App and GitHub OAuth authenticate RepoRoller against GitHub and users against the web UI."
audience: "operator"
type: "explanation"
---

# Authentication and authorization

RepoRoller uses two separate authentication mechanisms that serve different purposes. Understanding both is important for deployment and troubleshooting.

## The GitHub App — service identity

RepoRoller acts as a GitHub App when creating repositories and reading organisation settings. The App is a service identity: its actions appear in GitHub audit logs as the App itself, not as any individual user. This provides a clean, traceable record of every repository created by RepoRoller regardless of who requested it.

The App authenticates to GitHub using public-key cryptography. At startup, the backend loads a private key (from `GITHUB_APP_PRIVATE_KEY`) and the App ID (`GITHUB_APP_ID`). When it needs to call the GitHub API, it derives a short-lived **installation token** by signing a JWT with the private key and exchanging it with GitHub's Apps API.

Installation tokens:

- Are scoped to the specific GitHub organisation where the App is installed
- Expire automatically (typically within one hour)
- Grant only the permissions declared in the App registration, nothing more

This means: if the private key is compromised, an attacker can act as the App — but only within the organisation and only with the declared permissions. Rotate the private key immediately if you suspect exposure.

## GitHub OAuth — user authentication

The web UI authenticates end-users with GitHub OAuth. This confirms the user's GitHub identity and provides the verified username used in audit logs.

The flow:

1. User clicks *Sign in with GitHub* — the frontend redirects to GitHub's OAuth authorisation page.
2. GitHub redirects back to `/auth/callback` with an authorization code.
3. The frontend exchanges the code for a **GitHub user token** using the OAuth App credentials (`GITHUB_CLIENT_ID` + `GITHUB_CLIENT_SECRET`).
4. The frontend sends the GitHub user token to the backend's `/api/v1/auth/token` endpoint.
5. The backend validates the token with GitHub (one call), extracts the user's organisation membership, and issues a signed **backend JWT** valid for eight hours.
6. The frontend stores the backend JWT in an HMAC-signed session cookie (`SESSION_SECRET`).
7. All subsequent API calls carry the backend JWT. The backend verifies it locally — no GitHub API call per request.

This design means GitHub credentials are never held beyond the initial exchange, and the per-request overhead of authentication is a local signature verification (microseconds) rather than a network call.

## What is checked

- **Organisation membership**: implied by VPN access. The OAuth flow confirms a valid GitHub identity.
- **Token scope**: the backend JWT is scoped to the organisation it was issued for. A token issued for `myorg` cannot be used to create repositories in `otherorg`.
- **Repository creation permissions**: the GitHub App must be installed in the organisation with the required permissions. If the App lacks `Contents: write` or `Administration: write`, repository creation fails.

## Session lifetime

Sessions expire after eight hours (the backend JWT TTL). After expiry, the user must sign in again. You cannot change the TTL without recompiling the backend.
