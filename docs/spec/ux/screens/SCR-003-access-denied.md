# SCR-003: Access Denied

**Route**: `/auth/denied`
**Goal served**: Inform a user why sign-in could not be completed and what they can do
**Entry points**: OAuth callback redirect (on OAuth error, denial, or org membership failure)
**Exit points**: `/sign-in` (try again); GitHub org membership request (external link, when applicable)

---

## Wireframe

**OAuth error variant** (most common at launch):

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│              [ logo image | App Name ]                  │
│                                                         │
│            ⚠  Sign-in could not be completed           │
│                                                         │
│  There was a problem connecting to GitHub. This is      │
│  usually temporary.                                     │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │                  Try again                        │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Access denied variant** (user cancelled on GitHub):

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│              [ logo image | App Name ]                  │
│                                                         │
│         ✕  GitHub authorization was cancelled           │
│                                                         │
│  RepoRoller needs permission to read your GitHub        │
│  identity to log who creates repositories.              │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │                  Try again                        │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

- Same centred card layout as SCR-001 and SCR-002
- Icon variant (⚠ vs ✕) communicates nature of the problem at a glance
- Single primary button — no secondary actions to create confusion

---

## Purpose

This screen handles all failure cases from the authentication flow. It tells the user what went
wrong without exposing internal error details, and provides a clear next action.

In the current release, users only reach this screen via OAuth errors (not org membership checks,
which are not yet enforced). The org-membership variant is specified here so it can be implemented
without redesigning the screen.

---

## Layout

- Same centred card shell as Sign In and OAuth Callback
- Icon appropriate to the reason (warning or lock)
- Heading appropriate to the reason
- Explanatory message
- One or two action buttons/links depending on reason

---

## States

### OAuth error (reason=oauth_error, reason=network_error, reason=identity_failure)

Shown when the GitHub OAuth exchange failed for technical reasons (GitHub outage, network error,
or the identity lookup failed).

- Heading: "Sign-in could not be completed"
- Message: "There was a problem connecting to GitHub. This is usually temporary."
- Primary action button: "Try again" → `/sign-in`

### User denied access on GitHub (reason=access_denied)

Shown when the user clicked "Cancel" on the GitHub authorization page.

- Heading: "GitHub authorization was cancelled"
- Message: "RepoRoller needs permission to read your GitHub identity to log who creates repositories."
- Primary action button: "Try again" → `/sign-in`

### Org membership required (reason=not_org_member) — future

Shown when the org membership check is enabled and the user is not a member of the required org.

- Heading: "Access restricted to organization members"
- Message: "RepoRoller is available to members of [Org Name] only. If you believe this is an
  error, contact your administrator."
- Primary action button: "Back to sign-in" → `/sign-in`

---

## Interactions

| Element | Action | Outcome |
|---|---|---|
| "Try again" button | Click | Navigate to `/sign-in` |
| "Back to sign-in" button | Click | Navigate to `/sign-in` |

---

## Accessibility

- `<h1>` matches the state heading
- Error message uses `role="alert"` — announced to screen readers on page load
- No timed auto-redirect (user controls when to proceed)

---

## Copy

| Element | Copy |
|---|---|
| Page `<title>` | Access denied — RepoRoller |
| OAuth error heading | Sign-in could not be completed |
| OAuth error message | There was a problem connecting to GitHub. This is usually temporary. |
| OAuth error button | Try again |
| Access denied heading | GitHub authorization was cancelled |
| Access denied message | RepoRoller needs permission to read your GitHub identity to log who creates repositories. |
| Access denied button | Try again |
| Not org member heading | Access restricted to organization members |
| Not org member message | RepoRoller is available to members of [org] only. If you believe this is an error, contact your administrator. |
| Not org member button | Back to sign-in |

---

## Data Requirements

- **Inputs**: `reason` query parameter (values: `oauth_error`, `network_error`, `identity_failure`, `access_denied`, `not_org_member`)
- **No async data loading** — all content is static based on the `reason` parameter
- **Security**: The `reason` parameter drives copy selection only. Internal error details are never
  surfaced to the client.
