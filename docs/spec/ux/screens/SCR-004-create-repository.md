# SCR-004: Create Repository

**Route**: `/create`
**Goal served**: User creates a fully-configured GitHub repository
**Entry points**: Redirect from `/auth/callback` (on first sign-in); direct navigation by returning users
**Exit points**: `/create/success` (creation succeeded); `/sign-in` (session expired or sign-out)

---

## Wireframes

### AppShell + Step 1: Choose a Template

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  |
|   ●                             ○                   ○        |
|                                                              |
|  Choose a template                                           |
|                                                              |
|  +----------------------------------------------------------+|
|  | Search templates...                                      ||
|  +----------------------------------------------------------+|
|                                                              |
|  +-------------------------+  +-------------------------+   |
|  |  O  rust-library        |  |  O  python-service      |   |
|  |  Rust library with      |  |  Python microservice    |   |
|  |  CI/CD and testing      |  |  with FastAPI           |   |
|  |  [rust] [library]       |  |  [python] [service]     |   |
|  |  type: library          |  |  type: service          |   |
|  +-------------------------+  +-------------------------+   |
|                                                              |
|  +-------------------------+  +-------------------------+   |
|  |  O  github-action       |  |  O  docs-site           |   |
|  |  GitHub Action          |  |  Documentation site     |   |
|  |  template               |  |  with MkDocs            |   |
|  |  [actions] [ci]         |  |  [docs] [mkdocs]        |   |
|  +-------------------------+  +-------------------------+   |
|                                                              |
|                              [ Next: Repository settings → ] |
|                                                              |
+==============================================================+
```

Selected state (rust-library chosen, details loaded):

```
|  +-------------------------+  +-------------------------+   |
|  |  ✓  rust-library        |  |  O  python-service      |   |
|  |  (highlighted border,   |  |                         |   |
|  |   checkmark top-right)  |  |                         |   |
|  +-------------------------+  +-------------------------+   |
|                                                              |
|                              [ Next: Repository settings → ] |  <- enabled
```

---

### Step 2: Repository Settings (template has variables — shows "Next" button)

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  |
|   ✓                             ●                   ○        |
|                                                              |
|  Repository settings                                         |
|                                                              |
|  Repository name *                                           |
|  +----------------------------------------------------------+|
|  | e.g. my-new-service                                      ||
|  +----------------------------------------------------------+|
|  Lowercase letters, numbers, hyphens, underscores, dots.     |
|                after blur → [ ✓ Available ]                  |
|                                                              |
|  Repository type                                             |
|  +----------------------------------------------------------+|
|  | library  (pre-selected, preferable policy)           v   ||
|  +----------------------------------------------------------+|
|  Recommended by this template, but you can choose a          |
|  different type.                                             |
|                                                              |
|  Team (optional)                                             |
|  +----------------------------------------------------------+|
|  | No specific team                                     v   ||
|  +----------------------------------------------------------+|
|                                                              |
|  [ ← Back ]                       [ Next: Variables → ]     |
|                                                              |
+==============================================================+
```

---

### Step 2: Repository Settings (template has NO variables — shows "Create" button + summary)

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings                      |
|   ✓                             ●                            |
|                                                              |
|  Repository settings                                         |
|                                                              |
|  Repository name *                                           |
|  +----------------------------------------------------------+|
|  | my-docs-site                                             ||
|  +----------------------------------------------------------+|
|  ✓ Available                                                 |
|                                                              |
|  Team (optional)                                             |
|  +----------------------------------------------------------+|
|  | No specific team                                     v   ||
|  +----------------------------------------------------------+|
|                                                              |
|  +----------------------------------------------------------+|
|  | You are about to create:                                 ||
|  | myorg / my-docs-site   Template: docs-site   [private]  ||
|  +----------------------------------------------------------+|
|                                                              |
|  [ ← Back ]                          [ Create Repository ]  |
|                                                              |
+==============================================================+
```

---

### Step 3: Template Variables

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  |
|   ✓                             ✓                   ●        |
|                                                              |
|  Template variables                                          |
|                                                              |
|  Service name *                                              |
|  +----------------------------------------------------------+|
|  |                                                          ||
|  +----------------------------------------------------------+|
|  The name of the service, used throughout the template.      |
|                                                              |
|  Author                                                      |
|  +----------------------------------------------------------+|
|  | Platform Team       (pre-filled default value)           ||
|  +----------------------------------------------------------+|
|                                                              |
|  Description (optional)                                      |
|  +----------------------------------------------------------+|
|  |                                                          ||
|  +----------------------------------------------------------+|
|                                                              |
|  +----------------------------------------------------------+|
|  | You are about to create:                                 ||
|  | myorg / my-new-service   Template: rust-library          ||
|  | [library] [private]                                      ||
|  +----------------------------------------------------------+|
|                                                              |
|  [ ← Back ]                          [ Create Repository ]  |
|                                                              |
+==============================================================+
```

---

### Creation overlay (active during API call)

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  ////////////////////////////////////////////////////////// |
|  //  (wizard content dimmed beneath overlay)             // |
|  //  +----------------------------------------------+   // |
|  //  |                                              |   // |
|  //  |       Creating your repository…              |   // |
|  //  |                                              |   // |
|  //  |               ( ⟳ )                          |   // |
|  //  |          (animated spinner)                  |   // |
|  //  |                                              |   // |
|  //  |  This may take up to a minute. Please        |   // |
|  //  |  don't close this page.                      |   // |
|  //  |                                              |   // |
|  //  +----------------------------------------------+   // |
|  ////////////////////////////////////////////////////////// |
|                                                              |
+==============================================================+
```

---

## Purpose

The primary screen of the application. A stepped wizard that collects:

1. Template selection
2. Repository settings (name, type, team, visibility)
3. Template variable values (conditional — only shown when the template defines variables)

Then submits the creation request and shows a loading overlay while the API call completes.

---

## Layout

- Full-page authenticated layout: AppShell (header with logo + UserBadge) above the wizard area
- Wizard area: centred, single column, max-width ~700px
- Step progress indicator at the top of the wizard (e.g., "Step 1 of 3")
- Step content below the indicator
- Back / Next / Create buttons at the bottom of each step

---

## Global Wizard Behaviour

- **Step state is held in component memory** (not in the URL). The route stays `/create` throughout.
- **Back navigation within the wizard** is handled by the wizard's own Back button — not the browser
  back button. Browser back navigates away from `/create` entirely.
- **Unsaved-data guard**: if the user has selected a template (or entered any data) and
  attempts to close the tab or navigate away via browser controls, the browser's native leave
  confirmation dialog is shown.
- **The guard is removed** once the creation overlay activates (to avoid interfering with the
  redirect to `/create/success`) and once the user successfully completes creation.
- Returning to a previous step preserves all previously entered values in later steps.

---

## Step 1: Choose a Template

### Purpose

Let the user select the template that will define the structure and configuration of their new
repository. This step also loads the template's variable definitions in the background, so Step 3
is ready without delay.

### Layout

- Heading: "Choose a template"
- Optional search/filter bar (filters the displayed cards client-side, no API call)
- Template card grid (2 columns desktop, 1 column mobile)
- Each template card shows:
  - Template name (bold)
  - Description (clamped to 2 lines)
  - Tags (small chips, max 4 shown)
  - Repository type badge (if the template has a fixed or preferable type)
- "Next: Repository settings →" button at the bottom: **disabled** until a card is selected

### States

#### Loading (fetching template list)

- Search bar: disabled
- Card grid replaced with 4 skeleton card placeholders (ghost loading)
- Next button: disabled

#### Loaded (template list available, no selection)

- Template cards interactive
- Next button: disabled

#### Card selected — fetch in progress

- Selected card: highlighted border, checkmark in top-right corner
- Template details fetch begins immediately (`GET /api/v1/orgs/{org}/templates/{template}`)
- Next button: **disabled** while the fetch is in-flight
- Previously selected template (if user came back from Step 2): pre-selected on return

#### Card selected — fetch complete

- Next button: **enabled** — transitions automatically when the fetch succeeds; no user action required

#### Template details fetch error (after card selection)

- Error banner below the selected card: "Could not load template details. [Retry]"
- Next button: disabled until fetch succeeds or user selects a different template

#### Empty (no templates in the org)

- No cards shown
- Info message: "No templates are configured for this organization. Contact your platform team."
- Next button: disabled permanently

#### API error (failed to fetch template list)

- No cards shown
- Error message: "Could not load templates."
- "Try again" button — triggers refetch of template list

### Interactions

| Element | Action | Outcome |
|---|---|---|
| Template card | Click | Card selected; details fetch begins; Next remains disabled until fetch succeeds |
| Different card (when one already selected) | Click | New card selected; details refetched; previous selection cleared; Next disabled during refetch |
| Search/filter bar | Type | Template cards filtered client-side (by name, description, tags); no API call |
| Search bar cleared | Clear | All templates shown again |
| Retry button (API error state) | Click | Template list refetched |
| Retry link (details error state) | Click | Template details refetched for currently selected template |
| "Next: Repository settings" button | Click | Transition to Step 2 |

---

## Step 2: Repository Settings

### Purpose

Collect the repository name and other settings that control how the repository is created.
The name field validates availability in real-time. Repository type and other fields are
populated with defaults from the template or org configuration.

### Layout

- Heading: "Repository settings"
- Fields in order (see below)
- If the selected template has **no variables**: inline summary section at the bottom + "Create Repository" primary button
- If the selected template **has variables**: "Next: Variables →" button at the bottom
- "← Back" secondary button at the bottom

### Fields

#### Repository name (required)

- Label: "Repository name"
- Input type: text
- Placeholder: "e.g. my-new-service"
- Helper text: "Lowercase letters, numbers, hyphens, underscores, and dots. Must be unique in the organization. Cannot start with a dot."
- Validation behaviour:
  - **On type** (debounced 300ms): client-side format check only. Removes the "already taken" error
    immediately if the name changes.
  - **On blur** (when the field loses focus, if format is valid): calls `POST /repositories/validate-name`
    to check uniqueness
  - **Format invalid** (client-side): shows error below field; no API call made
  - **Format valid, checking** (API call in flight): shows spinner + "Checking availability…" below field; Next/Create button disabled
  - **Valid and available**: shows green checkmark + "Available" below field
  - **Valid but taken**: shows warning icon + "'[name]' is already taken in this organization." below field; Next/Create button disabled
  - **Format valid, API check failed**: shows warning icon + "Could not check availability. You can still proceed, but the name may already exist." below field; Next/Create button enabled (with caveat)

#### Repository type (conditional)

Shown only when repository types are defined in the org configuration.

- Label: "Repository type"
- Three sub-cases based on the selected template's `repository_type.policy`:

  **Policy: `fixed`**
  - Not an editable input — displayed as a read-only info row:
    "Repository type: [type name]" with a lock icon
  - Helper text: "This template requires a specific repository type."

  **Policy: `preferable`**
  - Dropdown, pre-selected to the template's preferred type
  - User can change it
  - Helper text: "Recommended by this template, but you can choose a different type."
  - Dropdown options: all available repository types from `GET /api/v1/orgs/{org}/types`

  **Policy: `optional` or no policy**
  - Dropdown, default selection: "No specific type"
  - Dropdown options: "No specific type" + all available repository types

#### Team (optional)

- Label: "Team (optional)"
- Input: dropdown
- Default: "No specific team"
- Options: org's teams (if team list API is available) — fetched on step entry, not on page load
- Helper text: "Select your team to apply team-specific configuration defaults."
- Loading state when fetching teams: dropdown shows "Loading teams…" and is disabled
- Error state if teams fetch fails: dropdown hidden, replaced with info: "Team configuration unavailable."

#### Visibility (conditional)

Shown only if the org configuration permits user visibility override. Hidden by default in the
initial release (assume org-enforced visibility).

- Label: "Visibility"
- Input: radio group
  - "Private" (default)
  - "Public"
- Helper text for Private: "Only organization members can see this repository."
- Helper text for Public: "Anyone on the internet can see this repository."

### Inline Summary (when template has no variables)

Displayed below the fields before the Create button:

```
You are about to create:
[org name] / [repository name]    Template: [template name]    [Type badge]    [Visibility badge]
```

All parts update live as the user fills in the form.

### Interactions

| Element | Action | Outcome |
|---|---|---|
| Repository name field | Type | Client-side format validation on debounce; "taken" error cleared |
| Repository name field | Blur (valid format) | API uniqueness check triggered |
| Repository name field | Blur (invalid format) | No API call; format error remains |
| Repository type dropdown | Change | Selected type updates; summary updates |
| Team dropdown | Change | Selected team updates |
| Visibility radio | Change | Visibility updates; summary updates |
| "← Back" button | Click | Return to Step 1; previously selected template card still selected |
| "Next: Variables" button | Click | Transition to Step 3; button visible only when template has variables |
| "Create Repository" button | Click | Validation check → if all valid, show creation overlay and submit |

### Next / Create button activation rules

The Next / Create button is **enabled** when:

- Repository name field: format is valid AND uniqueness check result is "Available" (or "check failed" warning state)

The Next / Create button is **disabled** when:

- Repository name is empty
- Repository name format is invalid
- Uniqueness check is in progress
- Uniqueness check returned "already taken"

---

## Step 3: Template Variables

### Purpose

Collect values for the template-specific variables that will be substituted into the template
content. Only shown when the selected template defines at least one variable.

### Layout

- Heading: "Template variables"
- Variable fields in order: required variables first, then optional
- Required variables marked with an asterisk (*) in the label
- Inline summary section at the bottom (same as Step 2's summary when no variables)
- "Create Repository" primary button at the bottom
- "← Back" secondary button at the bottom

### Variable Field Rendering

Each variable from the template's `variables[]` array is rendered as a `VariableField` component:

- **Label**: variable `name` formatted as a human-readable label (e.g., `service_name` → "Service name")
- **Required marker**: asterisk if `required: true`
- **Description**: if `variable.description` is provided, shown as helper text below the field
- **Placeholder**: variable description (shortened) or "Enter a value"
- **Default value**: if `variable.default_value` is provided, pre-filled in the input
- **Input type**: always `text` in this release (no typed fields)

### Inline Summary

Same component as used in Step 2 when template has no variables:

```
You are about to create:
[org] / [name]    Template: [template]    [Type badge]    [Visibility badge]
```

### States

- **Incomplete** (some required variables empty): Create button disabled
- **Complete** (all required variables have values): Create button enabled
- **Submitting** (Create clicked): creation overlay shown; wizard hidden behind overlay

### Interactions

| Element | Action | Outcome |
|---|---|---|
| Variable field | Type | Value captured; Create button re-evaluates enabled state |
| "← Back" button | Click | Return to Step 2; all Step 2 field values preserved |
| "Create Repository" button | Click | All required fields validated → creation overlay shows → POST to API |

### Create button activation rules

The Create button is **enabled** when:

- All `required: true` variables have a non-empty value
- (Step 2 conditions still met — validated before proceeding to Step 3)

---

## Creation Overlay

Displayed over the entire wizard while `POST /api/v1/repositories` is in flight.

### Layout

- Full wizard area covered with a semi-transparent overlay
- Centred spinner (animated)
- Heading: "Creating your repository…"
- Sub-message: "This may take up to a minute. Please don't close this page."
- No buttons (intentional — creation cannot be cancelled mid-flight)

### Behaviour

- Activated immediately when the user clicks "Create Repository"
- Wizard step content is visually obscured (not removed from DOM — allows error recovery)
- If the API call succeeds: navigates to `/create/success`
- If the API call fails: overlay is dismissed; appropriate inline error is shown (see below)
- **Timeout**: if no response is received within **90 seconds**, treat as a network error; dismiss
  the overlay and show the "Could not reach the server. Check your connection and try again."
  InlineAlert on the current step. The sub-message "This may take up to a minute" implies a
  ~60-second expected maximum; the 90-second timeout adds a safety margin before giving up.

---

## Creation Error States

All creation errors are shown as an `InlineAlert` component with `variant="error"` above the
Create button (Step 2 or Step 3 depending on where creation was triggered). The creation overlay
is dismissed before the error appears.

| API Error | Recovery UX |
|---|---|
| 422 — name already taken (race condition) | User returned to Step 2; name field shows taken error; InlineAlert: "A repository named '[name]' was created while you were filling in this form. Please choose a different name." |
| 422 — template not found | User returned to Step 1; template grid reloaded; InlineAlert: "The template '[name]' is no longer available. Please choose a different template." |
| 403 — permission denied | InlineAlert in current step: "You don't have permission to create repositories in this organization. Contact your administrator." |
| 5xx — GitHub API error | InlineAlert in current step: "GitHub is temporarily unavailable. Your repository was not created. Please try again." |
| Network error (no response) | InlineAlert in current step: "Could not reach the server. Check your connection and try again." |

---

## Accessibility

- Wizard is a single `<form>` element; Create triggers form submission
- Step heading is an `<h1>` that updates as steps advance
- Step progress indicator has `aria-label="Step [n] of [total]"`
- Template card grid: each card is a `<label>` wrapping a radio `<input>` for screen reader support
- All form fields have explicit `<label>` associations
- Required fields use `aria-required="true"` in addition to the visual asterisk
- Validation error messages use `role="alert"` (announced on appearance)
- InlineAlert errors use `role="alert"`
- Spinner in creation overlay has `role="status"` and `aria-label="Creating repository"`
- Tab order follows visual reading order (top to bottom)
- Minimum touch target: 44×44px for all interactive elements
- Focus management: when advancing to a new step, focus moves to the step's `<h1>`

---

## Data Requirements

### API calls made by this screen

| Call | When | Endpoint |
|---|---|---|
| Fetch template list | On step entry (page load) | `GET /api/v1/orgs/{org}/templates` |
| Fetch template details | When user selects a template card | `GET /api/v1/orgs/{org}/templates/{template}` |
| Fetch repository types | On step 2 entry (first time only) | `GET /api/v1/orgs/{org}/types` |
| Validate name uniqueness | On blur of name field (if format valid) | `POST /api/v1/repositories/validate-name` |
| Create repository | On "Create Repository" click | `POST /api/v1/repositories` |

### Inputs collected (final POST body)

```json
{
  "organization": "<configured org>",
  "name": "<user input>",
  "template": "<selected template name>",
  "visibility": "<"private"|"public">",
  "team": "<selected team name or omitted>",
  "repository_type": "<selected type name or omitted>",
  "variables": {
    "<var_name>": "<user value>",
    ...
  }
}
```
