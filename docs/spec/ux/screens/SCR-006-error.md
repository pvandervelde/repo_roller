# SCR-006: Error

**Route**: `/error`
**Goal served**: Handle unexpected system errors that cannot be recovered inline
**Entry points**: Any screen when an unrecoverable error occurs; server-side redirect on catastrophic failure
**Exit points**: `/create` (try again); `/sign-in` (if session may have been involved)

---

## Wireframe

**Generic error variant:**

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|                         ⚠                                    |
|                    (warning icon)                            |
|                                                              |
|                  Something went wrong                        |
|                                                              |
|  An unexpected error occurred. If this keeps happening,      |
|  contact your platform team.                                 |
|                                                              |
|  +----------------------------------------------------------+|
|  |                   Try again                             ||
|  +----------------------------------------------------------+|
|                                                              |
+==============================================================+
```

**Session expired variant** (header may show minimal branding only, no UserBadge):

```
+==============================================================+
|  [ Logo ]  [App Name]                                        |
+==============================================================+
|                                                              |
|                         🔒                                   |
|                    (lock icon)                               |
|                                                              |
|               Your session has expired                       |
|                                                              |
|  Please sign in again to continue.                           |
|                                                              |
|  +----------------------------------------------------------+|
|  |                    Sign in                              ||
|  +----------------------------------------------------------+|
|                                                              |
+==============================================================+
```

---

## Purpose

A fallback screen for failures that cannot be shown inline (e.g., a server-side rendering error,
failed middleware, or an error that occurs before a specific screen's error handling can run).

Most errors are handled inline on the screen where they occur. This screen is only reached when
that is not possible.

---

## Layout

- AppShell (header displayed if session available; minimal header if not)
- Centred content area, single column
- Warning/error icon
- Heading
- Explanatory message
- Action button(s)

---

## States

### Generic error (default — no `reason` parameter or unknown reason)

- Heading: "Something went wrong"
- Message: "An unexpected error occurred. If this keeps happening, contact your platform team."
- Button: "Try again" → `/create` (or `/sign-in` if no session)

### Session expired (reason=session_expired — future use)

- Heading: "Your session has expired"
- Message: "Please sign in again to continue."
- Button: "Sign in" → `/sign-in`

---

## Interactions

| Element | Action | Outcome |
|---|---|---|
| "Try again" button | Click | Navigate to `/create` if session exists, otherwise `/sign-in` |
| "Sign in" button (session_expired state) | Click | Navigate to `/sign-in` |

---

## Accessibility

- `<h1>` matches the state heading
- Error message uses `role="alert"` (announced on page load to screen readers)

---

## Copy

| Element | Copy |
|---|---|
| Page `<title>` | Error — [App Name] |
| Generic heading | Something went wrong |
| Generic message | An unexpected error occurred. If this keeps happening, contact your platform team. |
| Generic button | Try again |
| Session expired heading | Your session has expired |
| Session expired message | Please sign in again to continue. |
| Session expired button | Sign in |

---

## Data Requirements

- **Inputs**: `reason` query parameter (optional; values: `session_expired`)
- **No async data loading**
- **Security**: No internal error details, stack traces, or identifiers surfaced to the client
