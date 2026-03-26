# SCR-002: OAuth Callback

**Route**: `/auth/callback`
**Goal served**: Completing the GitHub OAuth exchange and establishing a session
**Entry points**: GitHub OAuth redirect (system-triggered, user cannot navigate here directly)
**Exit points**: `/create` (success); `/auth/denied` (failure or denial)

---

## Wireframe

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│              [ logo image | App Name ]                  │
│                                                         │
│                 Completing sign-in                      │
│                                                         │
│                      ( ↻ )                              │
│                 (animated spinner)                      │
│                                                         │
│         You'll be redirected in a moment.               │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

- Same centred card layout as SCR-001
- No interactive elements; spinner is the only animated element
- Logo/wordmark provides brand continuity with the sign-in screen

---

## Purpose

This is an intermediate screen the user sees briefly while the server-side OAuth token exchange
completes. It provides reassurance that something is happening. Users should spend no more than
2–3 seconds on this screen under normal conditions.

---

## Layout

- Identical visual shell to the Sign In screen (centred card, no navigation)
- Spinner (animated)
- Status message
- No user actions available

---

## States

### Completing sign-in (default — shown while token exchange is in progress)

- Spinner: visible and animating
- Message: "Completing sign-in…"
- Sub-message: "You'll be redirected in a moment."
- No buttons or links

### Error (if callback URL contains `error` parameter from GitHub, or server-side failure is detected before redirect)

This state is typically not rendered — the server handles the error and immediately redirects to
`/auth/denied`. However, if client-side JavaScript detects an `error` query parameter before the
server processes it:

- Spinner: hidden
- Message: "Sign-in could not be completed."
- Link: "Try again" → `/sign-in`

---

## Interactions

There are no user interactions on this screen. Navigation is system-driven.

---

## Accessibility

- Single `<h1>`: "Completing sign-in"
- Spinner has `role="status"` and `aria-label="Signing in"`
- No keyboard trap — if the redirect never fires, the user can navigate away using browser controls

---

## Copy

| Element | Copy |
|---|---|
| Page `<title>` | Signing in… — [App Name] |
| Heading `<h1>` | Completing sign-in |
| Status message | You'll be redirected in a moment. |
| Error heading | Sign-in could not be completed. |
| Error link | Try again |

---

## Data Requirements

- **Inputs**: `code` query parameter (from GitHub), `state` query parameter (CSRF token)
- **Server action**: Exchange code for token; retrieve GitHub user identity (`GET /user`);
  create session; redirect
- **No user-visible data** on this screen beyond the status message
