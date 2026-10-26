---
title: "Tour of the web interface"
description: "A walkthrough of every screen in the RepoRoller web interface, explaining what you see and what you can do."
audience: "repository-creator"
type: "tutorial"
---

# Tour of the web interface

This tutorial walks you through every screen in the RepoRoller web interface. By the end you will know what each screen is for and what to do if something goes wrong.

---

## Screen 1: Sign In (SCR-001)

**URL:** `/` or `/sign-in` when not authenticated

The first screen you see when you open RepoRoller without an active session.

**What you see:**

- The RepoRoller wordmark (or your organisation's custom logo)
- A brief description of the tool
- A single button: **Sign in with GitHub**

**What you do:**
Click **Sign in with GitHub**. Your browser is redirected to `github.com/login/oauth/authorize` where GitHub asks you to authorise the RepoRoller OAuth App. Review the permissions and click **Authorize**.

**What happens next:**
GitHub redirects back to the OAuth Callback screen (SCR-002).

> **Tip:** If you are on a shared workstation, sign out after you finish to clear the session cookie.

---

## Screen 2: OAuth Callback (SCR-002)

**URL:** `/auth/callback`

This screen is shown briefly while RepoRoller exchanges the OAuth code for a GitHub token and establishes your session. You will usually only see a loading spinner for one or two seconds.

**What you see:**

- A loading spinner
- Text: "Signing you in…"

**What happens next:**

- On success: you are redirected to the **Create Repository** wizard (SCR-004).
- On failure: you are redirected to the **Access Denied** screen (SCR-003) or the **Error** screen (SCR-006).

**If it takes too long:**
Reload the page. If the loop continues, check that you are connected to the corporate VPN.

---

## Screen 3: Access Denied (SCR-003)

**URL:** `/access-denied`

You see this screen when you successfully authenticated with GitHub but do not have permission to use RepoRoller.

**What you see:**

- A lock icon
- Text: "Access denied"
- An explanation: "Your GitHub account is not a member of the `myorg` organisation, or access requires additional approval."
- A **Sign in with a different account** button

**What you do:**

- If you believe this is an error, contact your platform team and ask them to verify your organisation membership.
- If you signed in with a personal GitHub account instead of your work account, click **Sign in with a different account** and use the correct account.

---

## Screen 4: Create Repository wizard (SCR-004)

**URL:** `/create`

The main screen. This is a three- or four-step wizard: three steps if the template you choose has
no configurable variables, four if it does. Either way, the wizard always ends on a **Review and
create** step — that's the only place the final **Create Repository** button appears.

### Step 1 of 3 or 4: Choose a template

**What you see:**

- A page that's noticeably wider than the rest of the wizard on large screens — Step 1 alone uses
  the extra room for a two-column layout (grid on the left, details panel on the right).
- A grid of template cards, each showing:
  - Template name (the human-readable display name configured by the template author; falls
    back to the repository slug if the template doesn't set one)
  - Description, clamped to two lines with a **Show more** link beneath it. Clicking **Show
    more** expands the card to show the full description in place (the card's selection state
    is unaffected); clicking again (**Show less**) collapses it back to two lines.
  - Up to four of the template's tags (e.g. `rust`, `microservice`)
- A search field to filter by name, display name, description, or tag (case-insensitive,
  partial match)
- A **template details panel** to the right of the grid (it moves below the grid instead, on
  narrower screens). This panel automatically shows the full, uncapped details — complete
  description, every tag, the repository type badge (when the template has one), and the full
  list of variables the template defines — for whichever card you currently have selected. Before
  you select anything it just says "Select a template to see its details here."; while the
  details for your selection are loading, it shows a small spinner and "Loading template
  details…".

**What you do:**

1. Browse or search for the template you want (searching for a tag such as `rust` narrows the
   grid to templates that have that tag).
2. Click a card to select it (the card highlights with a border, and the details panel on the
   right updates automatically to show that template's full information). Clicking **Show
   more**/**Show less** does not change which card is selected.
3. Use the details panel to review the template's full description and variable list before
   committing to it, without needing to advance past this step.
4. Click **Next** when you have selected a template.

### Step 2 of 3 or 4: Repository details

**What you see:**

- **Repository name** — a text field with live validation. A green checkmark appears when the name is valid and available; a red warning appears if the name is taken or invalid.
- **Description** — an optional text field.
- **Visibility** — a dropdown. Options depend on organisation policy. If the template's repository type has a fixed visibility policy, this field may be pre-set and read-only.
- **Repository type** — shown if the template specifies a type. May be read-only if the type is fixed by the template.

**What you do:**

1. Type the repository name in lowercase with hyphens (example: `payment-service`).
2. Optionally add a description.
3. Set visibility if the field is editable.
4. Click **Next** to proceed.

### Step 3 of 4: Variables

This step only appears if the template you chose declares at least one variable; otherwise the
wizard goes straight from Repository details to Review and create (making that step 3 of 3
instead).

**What you see:**

- A form with one field per variable declared by the template.
- Each field shows:
  - Variable name (human-readable label)
  - A short description
  - A placeholder showing an example value
  - A `*` indicator for required fields
  - Inline validation messages
- A summary panel showing the repository that will be created so far (name, template, type,
  visibility)

**What you do:**

1. Fill in all required fields (marked with `*`).
2. Optionally fill in optional fields (defaults are shown as placeholders).
3. Click **Next: Review** to continue — this step no longer has its own **Create Repository**
   button; Create has moved to the final Review step, where you'll see every value you just
   entered listed back to you before you commit.

### Step 3 of 3 (no variables) or 4 of 4 (with variables): Review and create

The wizard's final step, always. This is new: previously, there was nowhere in the wizard that
showed you every variable value you had entered before you clicked Create — they were used, but
never displayed back to you. Now you can check everything in one place first.

**What you see:**

- `[your org] / [repository name]`
- Chips for the selected template, repository type, and visibility
- The template's full description and tags (if the template has any)
- One row per configured variable, showing its name and the value you entered (or its default, if
  you left it unchanged) — only shown if the template has variables

**What you do:**

1. Review everything listed. If anything is wrong, click **← Back** to return to the previous
   step — all of your entries are preserved, so you don't need to re-enter anything.
2. Click **Create Repository** when you're satisfied. This button only appears here; clicking it
   twice in quick succession is safe — the second click is ignored while the first request is
   still in flight.

---

## Screen 5: Repository Created (SCR-005)

**URL:** `/created` (after successful creation)

**What you see:**

- A success banner: "Repository created successfully"
- Repository name as a clickable link: `https://github.com/myorg/payment-service`
- A summary list of what was applied:
  - Number of files from the template
  - Branch-protection rulesets applied
  - Labels applied
  - Teams granted access
- Buttons:
  - **Open on GitHub** — opens the new repository
  - **Create another** — returns to the wizard to create another repository

**What you do:**
Click **Open on GitHub** to visit the newly created repository. Your work here is done.

---

## Screen 6: Error (SCR-006)

**URL:** `/error` or shown inline in the wizard

**What you see:**

- An error icon
- A short error message (e.g. "Repository creation failed")
- A longer explanation where available (e.g. "A repository named `payment-service` already exists in `myorg`")
- A **Try again** button that returns you to the wizard with your previous entries pre-filled

**Common errors and what to do:**

| Error message | Likely cause | Action |
|---|---|---|
| "Repository already exists" | A repo with that name already exists | Choose a different name |
| "Template not found" | The selected template was deleted or renamed | Contact your platform team |
| "Access denied to organisation" | Token scope is insufficient | Re-authenticate or contact platform team |
| "Internal server error" | Unexpected backend failure | Note the request ID shown and contact your platform team |

---

## Next steps

- [Create your first repository in 5 minutes](quickstart.md) — follow the full wizard walkthrough
- [Create a repository using the CLI](create-first-repository.md) — terminal alternative
