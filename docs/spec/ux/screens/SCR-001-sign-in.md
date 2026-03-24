# SCR-001: Sign In

**Route**: `/sign-in`
**Goal served**: First-time user authenticates to establish identity
**Entry points**: Direct URL access; redirect from any protected route when no session exists
**Exit points**: `/auth/callback` (system-triggered after GitHub OAuth redirect)

---

## Wireframe

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│              [ logo image | App Name ]                  │
│                                                         │
│           Sign in to [App Name]                         │
│                                                         │
│  Create standardized GitHub repositories from your      │
│  organization's templates.                              │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │  [GitHub mark]   Sign in with GitHub              │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

- The card is centred in the viewport (horizontal + vertical)
- Logo/wordmark is centred above the card heading
- The sign-in button spans most of the card width
- No navigation, no links, no footer

---

## Purpose

Allow an org member to sign in with their GitHub account. No credentials are entered here —
the user is sent to GitHub to authenticate. This screen's sole purpose is to explain the tool
briefly and provide the single sign-in action.

---

## Layout

- Centred card, single column, vertically centred on the viewport
- RepoRoller logo/wordmark at top
- One-line description of what the tool does
- "Sign in with GitHub" button (prominent, GitHub-branded style)
- No navigation, no header, no footer (intentional — this is a full-screen gate)

---

## States

### Default

- "Sign in with GitHub" button: enabled
- No error messages visible

### Loading (after button click, before GitHub redirect)

- Button: disabled, label changes to "Redirecting to GitHub…"
- Spinner replaces the GitHub icon on the button
- Duration: typically < 500ms before browser navigates away; no timeout handling needed here

---

## Interactions

| Element | Action | Outcome |
|---|---|---|
| Sign in with GitHub button | Click | Button enters loading state; browser navigates to GitHub OAuth authorization URL |

---

## Accessibility

- Single `<h1>`: "Sign in to RepoRoller"
- Button is the page's primary action; Enter key triggers it
- No autofocus needed (single interactive element)
- Button has sufficient contrast in both default and loading states
- Minimum touch target: 44×44px

---

## Copy

| Element | Copy |
|---|---|
| Page `<title>` | Sign in — RepoRoller |
| Heading `<h1>` | Sign in to RepoRoller |
| Description | Create standardized GitHub repositories from your organization's templates. |
| Button (default) | Sign in with GitHub |
| Button (loading) | Redirecting to GitHub… |

---

## Data Requirements

- **Inputs**: none (user supplies no data on this screen)
- **Outputs (events)**: `initiateOAuth()` — triggers redirect to GitHub authorization endpoint
- **No async data loading** on this screen
