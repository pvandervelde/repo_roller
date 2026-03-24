# Navigation Map

## Route Inventory

| Route | Screen | Auth required | Notes |
|---|---|---|---|
| `/` | — | — | Redirects to `/create` if session exists, else `/sign-in` |
| `/sign-in` | SCR-001 | No | GitHub OAuth initiation |
| `/auth/callback` | SCR-002 | No | OAuth callback handler; transitions to `/create` or `/auth/denied` |
| `/auth/denied` | SCR-003 | No | Auth failure; `reason` query parameter selects copy variant |
| `/create` | SCR-004 | Yes | Multi-step creation wizard (step state held in memory, not URL) |
| `/create/success` | SCR-005 | Yes | Post-creation confirmation; `repo` query parameter carries full repo name |
| `/error` | SCR-006 | Optional | System error fallback |

---

## Authentication Guard

Any attempt to access a route marked "Auth required" without a valid session results in:

- Immediate redirect to `/sign-in`
- No 401 error page shown to the user
- The originally requested URL is **not** preserved (all authenticated users land at `/create`)

---

## Step Navigation Within SCR-004

Step navigation is managed entirely by component state inside `/create`. No URL changes occur
between steps.

| From | To | Trigger |
|---|---|---|
| Step 1 | Step 2 | User clicks "Next: Repository settings" (template selected + details loaded) |
| Step 2 | Step 3 | User clicks "Next: Variables" (template has variables + name is valid) |
| Step 2 | Creation overlay | User clicks "Create Repository" (template has no variables + name is valid + required fields complete) |
| Step 3 | Creation overlay | User clicks "Create Repository" (all required variables filled) |
| Step 2 | Step 1 | User clicks "← Back" |
| Step 3 | Step 2 | User clicks "← Back" |
| Any step | Browser history | Browser back button; unsaved-data guard fires if data entered |

---

## Post-Creation Navigation

| From | To | Trigger |
|---|---|---|
| SCR-004 (creation overlay) | SCR-005 | 201 Created response from API |
| SCR-004 (creation overlay) | SCR-004 (Step 2 with error) | 422 name taken |
| SCR-004 (creation overlay) | SCR-004 (Step 1 with error) | 422 template not found |
| SCR-005 | SCR-004 | "Create another repository" button |
| SCR-005 | GitHub (new tab) | "View repository on GitHub" button |

---

## Sign-Out Navigation

Sign-out is available on all authenticated screens via the UserBadge dropdown in the AppShell.

| From | To | Trigger |
|---|---|---|
| Any authenticated screen | `/sign-in` | User clicks "Sign out" in UserBadge dropdown |

---

## Back Button Behaviour

| Screen | Browser back behaviour |
|---|---|
| `/sign-in` | Goes to browser history (wherever user came from) |
| `/auth/callback` | Should have no back history entry (replace rather than push); navigating back goes to `/sign-in` |
| `/auth/denied` | "Try again" button is the intended recovery action; browser back not expected |
| `/create` (any step) | Triggers unsaved-data guard if data entered; user confirms to leave |
| `/create/success` | Goes to `/create` (intentional — user may want to create another) |
| `/error` | Goes to browser history |

---

## External Links

| Link | Destination | Behaviour |
|---|---|---|
| Repository link on SCR-005 | `https://github.com/{org}/{name}` | Opens in new tab (`target="_blank"`, `rel="noopener"`) |
