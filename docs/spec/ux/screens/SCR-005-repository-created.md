# SCR-005: Repository Created

**Route**: `/create/success`
**Goal served**: Confirm that the repository was created and give the user a direct link to it
**Entry points**: Redirect from `/create` after successful `POST /api/v1/repositories`
**Exit points**: GitHub repository URL (external, new tab); `/create` (create another)

---

## Wireframe

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|                         ✓                                    |
|                    (success icon)                            |
|                                                              |
|                   Repository created!                        |
|                                                              |
|               myorg/my-new-service                           |
|   https://github.com/myorg/my-new-service (clickable link)   |
|                                                              |
|  +----------------------------------------------------------+|
|  |           View repository on GitHub ↗                   ||
|  +----------------------------------------------------------+|
|                                                              |
|             [ Create another repository ]                    |
|                                                              |
|  ▸ What was applied                                          |
|    (collapsed; click to expand applied config summary)       |
|                                                              |
+==============================================================+
```

Expanded "What was applied" section:

```
|  ▾ What was applied                                          |
|  +----------------------------------------------------------+|
|  | Template: rust-library                                  ||
|  | Repository type: library                                ||
|  | Team: platform                                          ||
|  | Visibility: private                                     ||
|  | Created: 2026-03-24 14:32:07 UTC                        ||
|  +----------------------------------------------------------+|
```

---

## Purpose

Clearly confirm success and make it trivially easy to open the new repository. Provide a
"Create another repository" action so users who need to create several repositories in sequence
do not have to re-navigate.

The route carries the created repository's full name as a query parameter so the page can
display correct details even on a page refresh.

**Route**: `/create/success?repo={org}/{name}`

---

## Layout

- AppShell (header with logo + UserBadge)
- Centred content area, single column
- Large success icon (checkmark) above the heading
- Repository name prominently displayed
- Two action buttons: "View repository on GitHub" (primary, opens new tab) + "Create another repository" (secondary)
- Applied configuration summary (collapsed by default, expandable)

---

## States

### Success (query parameter present and valid)

- Success icon: visible
- Heading: "Repository created!"
- Repository full name: "[org]/[name]" displayed in monospace/code style
- Repository URL as a clickable link beneath the full name
- "View repository on GitHub" button: opens new tab to the GitHub URL
- "Create another repository" button: navigates to `/create` (resets the wizard)
- Applied configuration summary: collapsed section titled "What was applied", showing:
  - Template used
  - Repository type (if set)
  - Team (if set)
  - Visibility
  - Creation timestamp

### Invalid / missing query parameter

If the user navigates to `/create/success` directly without the `repo` parameter, or the parameter
is malformed:

- No success icon or repository name shown
- Info message: "Your repository was created. Check your GitHub organization to find it."
- "Create another repository" button: visible

---

## Interactions

| Element | Action | Outcome |
|---|---|---|
| "View repository on GitHub" button | Click | Opens GitHub repository URL in a new tab |
| Repository URL link | Click | Opens GitHub repository URL in a new tab |
| "Create another repository" button | Click | Navigate to `/create`; wizard resets to Step 1 |
| Applied config summary toggle | Click | Expands/collapses the applied configuration section |

---

## Accessibility

- `<h1>`: "Repository created!"
- Success icon has `aria-hidden="true"` (decorative)
- Repository name and link have clear label context
- Applied config summary uses `<details>` / `<summary>` for semantic expand/collapse
- "View repository on GitHub" button has `aria-label="View [org/name] on GitHub"` to give screen
  readers the repository name in context; also has `target="_blank"` with `rel="noopener"`
  and a visible "(opens in new tab)" indicator

---

## Copy

| Element | Copy |
|---|---|
| Page `<title>` | Repository created — RepoRoller |
| Heading `<h1>` | Repository created! |
| Repository link label | View on GitHub |
| "View on GitHub" button | View repository on GitHub |
| "Create another" button | Create another repository |
| Applied config summary label | What was applied |
| Invalid state message | Your repository was created. Check your GitHub organization to find it. |

---

## Data Requirements

- **Inputs**: `repo` query parameter (format: `{org}/{name}`) + full creation response stored in
  session/navigation state from the creation redirect
- **No additional API calls** on this screen
- The applied configuration details come from the `CreateRepositoryResponse.configuration`
  returned by the creation API call, passed via navigation state (not re-fetched)
