# Authentication Flow

## Overview

RepoRoller uses GitHub OAuth to establish user identity. The VPN provides access control;
OAuth provides the verified GitHub username that is recorded in the creation audit log.

Org membership is **not enforced at sign-in** in this release. The architecture is designed so
that a membership check can be added to the OAuth callback handler later without changes to
the front-end flow.

---

## User Goal: Sign in and reach the creation form

### Flow Diagram

```mermaid
flowchart TD
    A([User opens RepoRoller]) --> B{Has valid session?}
    B -->|Yes| C([Create Repository screen])
    B -->|No| D[Sign In screen]

    D --> E{User clicks 'Sign in with GitHub'}
    E --> F[Redirect to GitHub OAuth authorization page]

    F --> G{User action on GitHub}
    G -->|Authorizes the app| H[GitHub redirects to /auth/callback with code]
    G -->|Denies authorization| I[GitHub redirects to /auth/callback with error=access_denied]
    G -->|No action / closes tab| J([User abandoned — no redirect])

    H --> K[OAuth Callback screen: 'Completing sign-in...']
    K --> L{Backend: exchange code for token}

    L -->|Token exchange succeeds| M{Backend: get GitHub user identity}
    M -->|Identity retrieved| N{Future: check org membership}
    N -->|Member — or check not enforced| O[Create session with GitHub username]
    N -->|Not a member| P[Redirect to /auth/denied]
    M -->|Identity lookup fails| Q[Redirect to /auth/denied with reason=identity_failure]

    L -->|Token exchange fails — GitHub error| R[Redirect to /auth/denied with reason=oauth_error]
    L -->|Token exchange fails — network error| S[Redirect to /auth/denied with reason=network_error]

    O --> C

    P --> T[Access Denied screen]
    Q --> T
    R --> T
    I --> T
    S --> T
```

---

## Session Lifecycle

| Event | Behaviour |
|---|---|
| Successful OAuth | Session cookie set (HTTP-only, Secure, SameSite=Lax); user redirected to `/create` |
| Session active | All requests to `/create` and `/create/success` proceed without re-authentication |
| Session expired (future) | User redirected to `/sign-in`; redirect back to `/create` after successful sign-in |
| User clicks Sign out | Session destroyed; user redirected to `/sign-in` |
| OAuth error or denial | Session not created; user redirected to `/auth/denied` with `reason` parameter |

---

## Assumptions

- Session management is handled by the SvelteKit backend (server-side sessions or signed cookies).
- GitHub OAuth App credentials (client ID, client secret) are configured at deployment.
- The `read:user` scope is requested to retrieve the GitHub login (`GET /user`).
- The `read:org` scope is requested to support future org membership checks without a re-auth flow.
- The OAuth callback URL registered with the GitHub App is `/auth/callback`.
