# Copy Reference

All user-facing strings in the RepoRoller web UI, organized by screen and component.

> **`[App Name]`** is a placeholder for the value of `brand.app_name` in the deployment
> branding configuration. The default value is `"RepoRoller"`. See [branding.md](branding.md)
> for configuration details.
>
> All page `<title>` elements follow the pattern: `[screen name] — [App Name]`.

Use these strings verbatim in implementation. Do not use placeholder text like "Loading..." or
"Error" without a corresponding entry here.

---

## Global

| Element | Copy |
|---|---|
| App name | `[App Name]` (configured; default: RepoRoller) |
| Browser tab suffix | `— [App Name]` |
| Sign out (menu item) | Sign out |
| Back button | ← Back |

---

## SCR-001: Sign In

| Element | Copy |
|---|---|
| Page `<title>` | Sign in — [App Name] |
| Heading `<h1>` | Sign in to [App Name] |
| Description | Create standardized GitHub repositories from your organization's templates. |
| Sign-in button (default) | Sign in with GitHub |
| Sign-in button (loading/redirecting) | Redirecting to GitHub… |

---

## SCR-002: OAuth Callback

| Element | Copy |
|---|---|
| Page `<title>` | Signing in… — [App Name] |
| Heading `<h1>` | Completing sign-in |
| Status message | You'll be redirected in a moment. |
| Error heading (client-side fallback) | Sign-in could not be completed. |
| Error link | Try again |

---

## SCR-003: Access Denied

| State | Element | Copy |
|---|---|---|
| OAuth / network error | Page `<title>` | Access denied — [App Name] |
| OAuth / network error | Heading `<h1>` | Sign-in could not be completed |
| OAuth / network error | Message | There was a problem connecting to GitHub. This is usually temporary. |
| OAuth / network error | Button | Try again |
| User cancelled on GitHub | Heading `<h1>` | GitHub authorization was cancelled |
| User cancelled on GitHub | Message | [App Name] needs permission to read your GitHub identity to log who creates repositories. |
| User cancelled on GitHub | Button | Try again |
| Not org member (future) | Heading `<h1>` | Access restricted to organization members |
| Not org member (future) | Message | [App Name] is available to members of [org] only. If you believe this is an error, contact your administrator. |
| Not org member (future) | Button | Back to sign-in |

---

## SCR-004: Create Repository

### Wizard chrome

| Element | Copy |
|---|---|
| Page `<title>` | Create repository — [App Name] |
| Step indicator aria-label pattern | Step [n] of [total]: [step label] |

### Step 1: Choose a Template

| Element | Copy |
|---|---|
| Step heading `<h1>` | Choose a template |
| Step label (in progress indicator) | Choose template |
| Search bar placeholder | Search templates |
| Search bar label | Search templates |
| No-results message | No templates match '[query]' |
| Template list empty state | No templates are configured for this organization. Contact your platform team. |
| Template list error message | Could not load templates. |
| Template list retry button | Try again |
| Template detail fetch error | Could not load template details. |
| Template detail fetch retry link | Retry |
| Next button | Next: Repository settings → |

### Step 2: Repository Settings

| Element | Copy |
|---|---|
| Step heading `<h1>` | Repository settings |
| Step label (in progress indicator) | Settings |
| Repository name label | Repository name |
| Repository name placeholder | e.g. my-new-service |
| Repository name helper text | Lowercase letters, numbers, hyphens, underscores, and dots. Must be unique in the organization. Cannot start with a dot. |
| Name checking indicator | Checking availability… |
| Name available indicator | Available |
| Name taken error | '[name]' is already taken in this organization. |
| Name format error | Repository names may only contain lowercase letters, numbers, hyphens (-), underscores (_), and dots (.). Names cannot start with a dot. |
| Name availability check failed warning | Could not check availability. You can still proceed, but the name may already exist. |
| Repository type label | Repository type |
| Repository type fixed helper | This template requires a specific repository type. |
| Repository type preferable helper | Recommended by this template, but you can choose a different type. |
| Repository type optional default option | No specific type |
| Team label | Team (optional) |
| Team default option | No specific team |
| Team helper text | Select your team to apply team-specific configuration defaults. |
| Team loading state | Loading teams… |
| Team unavailable info | Team configuration unavailable. |
| Visibility label | Visibility |
| Visibility private option | Private |
| Visibility public option | Public |
| Visibility private helper | Only organization members can see this repository. |
| Visibility public helper | Anyone on the internet can see this repository. |
| Next button (has variables) | Next: Variables → |
| Create button (no variables) | Create Repository |
| Summary label prefix | You are about to create: |

### Step 3: Template Variables

| Element | Copy |
|---|---|
| Step heading `<h1>` | Template variables |
| Step label (in progress indicator) | Variables |
| Variable field placeholder | Enter a value |
| Required field marker (visual) | * |
| Required field aria suffix | (required) |
| Create button | Create Repository |
| Summary label prefix | You are about to create: |

### Creation overlay

| Element | Copy |
|---|---|
| Spinner aria-label | Creating repository |
| Overlay heading | Creating your repository… |
| Overlay sub-message | This may take up to a minute. Please don't close this page. |
| Overlay timeout error | Could not reach the server. Check your connection and try again. |

### Creation errors (InlineAlert messages)

| Error scenario | Copy |
|---|---|
| Name taken (race condition) | A repository named '[name]' was created while you were filling in this form. Please choose a different name. |
| Template no longer available | The template '[name]' is no longer available. Please choose a different template. |
| Permission denied | You don't have permission to create repositories in this organization. Contact your administrator. |
| GitHub API error | GitHub is temporarily unavailable. Your repository was not created. Please try again. |
| Network error | Could not reach the server. Check your connection and try again. |

---

## SCR-005: Repository Created

| Element | Copy |
|---|---|
| Page `<title>` | Repository created — [App Name] |
| Heading `<h1>` | Repository created! |
| Applied config summary toggle label | What was applied |
| "View on GitHub" button | View repository on GitHub |
| "View on GitHub" aria-label pattern | View [org/name] on GitHub (opens in new tab) |
| "Create another" button | Create another repository |
| Invalid state message | Your repository was created. Check your GitHub organization to find it. |

---

## SCR-006: Error

| State | Element | Copy |
|---|---|---|
| Generic | Page `<title>` | Error — [App Name] |
| Generic | Heading `<h1>` | Something went wrong |
| Generic | Message | An unexpected error occurred. If this keeps happening, contact your platform team. |
| Generic | Button | Try again |
| Session expired | Heading `<h1>` | Your session has expired |
| Session expired | Message | Please sign in again to continue. |
| Session expired | Button | Sign in |
