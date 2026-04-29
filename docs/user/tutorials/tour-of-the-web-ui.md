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

The main screen. This is a three-step wizard.

### Step 1 of 3: Choose a template

**What you see:**

- A grid of template cards, each showing:
  - Template name
  - Short description
  - Tags (e.g. `rust`, `microservice`)
- A search field to filter by name or tag

**What you do:**

1. Browse or search for the template you want.
2. Click a card to select it (the card highlights with a border).
3. Optionally, click the info icon on a card to see the full description, variable list, and what the template includes.
4. Click **Next** when you have selected a template.

### Step 2 of 3: Repository details

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

### Step 3 of 3: Variables

**What you see:**

- A form with one field per variable declared by the template.
- Each field shows:
  - Variable name (human-readable label)
  - A short description
  - A placeholder showing an example value
  - A `*` indicator for required fields
  - Inline validation messages
- A summary panel on the right showing the repository that will be created

**What you do:**

1. Fill in all required fields (marked with `*`).
2. Optionally fill in optional fields (defaults are shown as placeholders).
3. Review the summary panel to confirm the repository name, template, and variable values.
4. Click **Create Repository**.

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
