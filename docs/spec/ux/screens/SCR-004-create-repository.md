# SCR-004: Create Repository

**Route**: `/create`
**Goal served**: User creates a fully-configured GitHub repository
**Entry points**: Redirect from `/auth/callback` (on first sign-in); direct navigation by returning users
**Exit points**: `/create/success` (creation succeeded); `/sign-in` (session expired or sign-out)

---

## Wireframes

### AppShell + Step 1: Choose a Template

Step 1 renders in a widened, two-column layout: the template grid on the left, and a
`TemplateDetailsPanel` sidebar on the right showing full details for whichever card is currently
selected. Below a 60rem viewport width the sidebar stacks beneath the grid instead of beside it.
Nothing is selected yet in this first wireframe, so the sidebar shows its empty state:

```
+======================================================================================+
|  [ Logo ]  [App Name]                                            [ @username v ]      |
+======================================================================================+
|                                                                                      |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  ---  (4) Review         |
|   ●                             ○                   ○                  ○             |
|                                                                                      |
|  Choose a template                                                                   |
|                                                                                      |
|  +----------------------------------------------+  +------------------------------+  |
|  | Search templates...                           |  | Template details            |  |
|  +----------------------------------------------+  |                              |  |
|                                                      |  Select a template to see    |  |
|  +-------------------------+ +-------------------+  |  its details here.           |  |
|  |  O  rust-library        | |  O  python-service|  |                              |  |
|  |  Rust library with      | |  Python microserv.|  |                              |  |
|  |  CI/CD and testing      | |  with FastAPI     |  |                              |  |
|  |  [Show more]             | |  [Show more]      |  |                              |  |
|  |  [rust] [library]       | |  [python] [service]| |                              |  |
|  |  type: library          | |  type: service    |  |                              |  |
|  +-------------------------+ +-------------------+  |                              |  |
|                                                      +------------------------------+  |
|  +-------------------------+ +-------------------+                                    |
|  |  O  github-action       | |  O  docs-site     |                                    |
|  |  GitHub Action          | |  Documentation site|                                   |
|  |  template               | |  with MkDocs      |                                    |
|  |  [actions] [ci]         | |  [docs] [mkdocs]  |                                    |
|  +-------------------------+ +-------------------+                                    |
|                                                                                      |
|                                              [ Next: Repository settings → ]          |
|                                                                                      |
+======================================================================================+
```

Selected state (rust-library chosen, details loaded — sidebar now shows the full,
untruncated description, all tags, the type badge, and the template's variables):

```
|  +-------------------------+ +-------------------+  +------------------------------+  |
|  |  ✓  rust-library        | |  O  python-service|  | rust-library                 |  |
|  |  (highlighted border,   | |                   |  |                              |  |
|  |   checkmark top-right)  | |                   |  | Rust library with CI/CD,     |  |
|  +-------------------------+ +-------------------+  | testing, and release         |  |
|                                                      | automation configured.      |  |
|                                                      |                              |  |
|                                                      | type: library                |  |
|                                                      | [rust] [library] [ci] [cd]   |  |
|                                                      |                              |  |
|                                                      | Variables                    |  |
|                                                      |  service_name  Required      |  |
|                                                      |  author        Optional      |  |
|                                                      +------------------------------+  |
|                                                                                      |
|                                              [ Next: Repository settings → ]          |  <- enabled
```

> Note: the type badge in the sidebar is shown only when the template details response includes
> a `repository_type`. As of this release the backend's `TemplateDetailsResponse` never actually
> populates that field, so this badge does not currently render in production — a pre-existing
> backend gap, out of scope for this change.

---

### Step 2: Repository Settings (template has variables — shows "Next" button)

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  ---  (4) Review            |
|   ✓                             ●                   ○                  ○                 |
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

Note: Settings now always advances (it never creates directly) — the button reads "Next:
Variables →" when the template has variables, or "Next: Review →" otherwise (see the no-variables
wireframe below). The Create action lives exclusively on the new Review step.

---

### Step 2: Repository Settings (template has NO variables — shows "Next: Review" button + summary)

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Review     |
|   ✓                             ●                   ○        |
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
|  [ ← Back ]                           [ Next: Review → ]    |
|                                                              |
+==============================================================+
```

Note: when the template has no variables, the wizard is 3 steps total (Choose template, Settings,
Review) — Variables is skipped entirely. Settings' action button now always reads "Next: Review
→"; it no longer creates the repository directly (see Step 3/4: Review and Create, below).

---

### Step 3: Template Variables

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  ---  (4) Review            |
|   ✓                             ✓                   ●                  ○                 |
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
|  [ ← Back ]                          [ Next: Review → ]     |
|                                                              |
+==============================================================+
```

Note: Variables' action button now reads "Next: Review →" (not "Create Repository" — Create has
moved to the new Review step, below). The inline summary shown here is the same `RepositorySummary`
component used on the Review step, but without the `description`/`tags`/`variables` props, since
those are only meaningful once the user is reviewing the final configuration.

---

### Step 4 (or Step 3, no-variables path): Review and Create

```
+==============================================================+
|  [ Logo ]  [App Name]                   [ @username v ]      |
+==============================================================+
|                                                              |
|  (1) Choose template  ---  (2) Settings  ---  (3) Variables  ---  (4) Review            |
|   ✓                             ✓                   ✓                  ●                 |
|                                                              |
|  Review and create                                            |
|                                                              |
|  +----------------------------------------------------------+|
|  | myorg / my-new-service                                   ||
|  | Template: Rust Library   [library]   [private]           ||
|  |                                                            ||
|  | Rust library with CI/CD and testing configured.           ||
|  | [rust] [library] [ci] [cd]                                 ||
|  |                                                            ||
|  | service_name       my-new-service                        ||
|  | author              Platform Team                        ||
|  | description         (empty)                               ||
|  +----------------------------------------------------------+|
|                                                              |
|  [ ← Back ]                          [ Create Repository ]  |
|                                                              |
+==============================================================+
```

Note: this wireframe shows the variables-present path (Step 4 of 4). When the template has no
variables, this is Step 3 of 3 instead, and the variables list in the summary is simply empty
(the `variables` list section renders nothing — see CMP-009). In both cases Review is always the
final step, and "Create Repository" appears exclusively here.

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
4. A final review of everything collected so far, before the repository is actually created

Then submits the creation request and shows a loading overlay while the API call completes.

---

## Layout

- Full-page authenticated layout: AppShell (header with logo + UserBadge) above the wizard area
- Wizard area: centred, single column on every step **except** Step 1
  - **Step 1 only**: widened to a two-column layout, max-width **80rem**, to make room for the
    `TemplateDetailsPanel` sidebar beside the template grid (see Step 1, below)
  - **Settings / Variables / Review**: max-width **56rem** (unchanged from before this round of
    changes)
  - The page shell itself (`AppShell`'s `.app-shell__main`) was also widened, from 72rem to
    **96rem**, so the wizard's own 80rem Step-1 width has room to breathe within it
- Step progress indicator at the top of the wizard (e.g., "Step 1 of 3" or "Step 1 of 4" —
  the total is 4 when the selected template has variables, 3 when it doesn't; see Global Wizard
  Behaviour)
- Step content below the indicator
- Back / Next / Create buttons at the bottom of each step — **Create** appears only on the final
  Review step; every other step's primary button is a **Next** button that only ever advances

---

## Global Wizard Behaviour

- **Step state is held in component memory** (not in the URL). The route stays `/create` throughout.
- **Step count is 3 or 4**, never fixed: `['Choose template', 'Settings', 'Review']` when the
  selected template has no variables, or `['Choose template', 'Settings', 'Variables', 'Review']`
  when it does. A final **Review** step is *always* appended, regardless of which of the two step
  lists is in effect.
- **Create lives exclusively on the Review step.** Settings and Variables never create the
  repository directly — their primary button always reads "Next: … →" and only ever advances to
  the next step. This is a change from the previous behaviour, where "Create Repository" could
  appear on Settings (no-variables path) or Variables (has-variables path).
- **Back navigation within the wizard** is handled by the wizard's own Back button — not the browser
  back button. Browser back navigates away from `/create` entirely. Back is a simple one-step
  decrement at every step, including from Review (which always lands back on Variables when
  present, or Settings when not).
- **Unsaved-data guard**: if the user has selected a template (or entered any data) and
  attempts to close the tab or navigate away via browser controls, the browser's native leave
  confirmation dialog is shown.
- **The guard is removed** once the creation overlay activates (to avoid interfering with the
  redirect to `/create/success`) and once the user successfully completes creation.
- Returning to a previous step preserves all previously entered values in later steps — this now
  includes the Review step, whose summary (template description, tags, and every configured
  variable's name/value) is always derived live from current wizard state, never stale.
- A duplicate-submission guard (`creatingRepo`) prevents the Create action from firing twice in a
  row (e.g. a rapid double-click), now that Create is reachable from exactly one place.

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
  - Template display name (bold)
  - Description (clamped to 2 lines, with a "Show more"/"Show less" toggle to expand/collapse it
    in place — does not affect card selection)
  - Tags (small chips, max 4 shown)
  - Repository type badge (if the template has a fixed or preferable type)
- **Sidebar** (`TemplateDetailsPanel`, CMP-013): shown to the right of the grid (≥60rem viewport
  width) or stacked below it (narrower viewports). Shows the full, untruncated details — display
  name, complete description, all tags (uncapped), the repository type badge if present, and the
  full variable list — for whichever card is currently selected. Shows its own loading state
  (spinner + "Loading template details…") while the per-card details fetch is in flight, and its
  own empty state ("Select a template to see its details here.") when nothing is selected yet.
  This is new in this round of changes — previously, the only way to see a template's full
  description/variables was Step 3 itself.
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
- If the selected template has **no variables**: inline summary section at the bottom + "Next:
  Review →" primary button (this step no longer creates the repository directly — see Step 3/4:
  Review and Create)
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

Displayed below the fields before the Next button:

```
You are about to create:
[org name] / [repository name]    Template: [template name]    [Type badge]    [Visibility badge]
```

All parts update live as the user fills in the form. (This summary uses the same
`RepositorySummary` component, CMP-009, as the Review step — but without the `description`,
`tags`, or `variables` props, since Review is where those are shown.)

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
| "Next: Variables →" button | Click | Transition to Step 3 (Variables); shown only when the template has variables |
| "Next: Review →" button | Click | Transition to the Review step; shown only when the template has no variables |

### Next button activation rules

The Next button ("Next: Variables →" or "Next: Review →", depending on whether the template has
variables) is **enabled** when:

- Repository name field: format is valid AND uniqueness check result is "Available" (or "check failed" warning state)

The Next button is **disabled** when:

- Repository name is empty
- Repository name format is invalid
- Uniqueness check is in progress
- Uniqueness check returned "already taken"

> This step never creates the repository directly, regardless of whether the template has
> variables — Create lives exclusively on the Review step (Step 3 or Step 4; see below).

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
- "Next: Review →" primary button at the bottom (this step no longer creates the repository
  directly — see Step 4: Review and Create, below)
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

- **Incomplete** (some required variables empty): Next button disabled
- **Complete** (all required variables have values): Next button enabled

### Interactions

| Element | Action | Outcome |
|---|---|---|
| Variable field | Type | Value captured; Next button re-evaluates enabled state |
| "← Back" button | Click | Return to Step 2; all Step 2 field values preserved |
| "Next: Review →" button | Click | Transition to the Review step; all variable values carried forward |

### Next button activation rules

The "Next: Review →" button is **enabled** when:

- All `required: true` variables have a non-empty value
- (Step 2 conditions still met — validated before proceeding to Step 3)

---

## Step 3 or 4: Review and Create

> This is Step 3 of 3 when the selected template has no variables (Variables is skipped), or
> Step 4 of 4 when it does. It is always the wizard's final step.

### Purpose

Let the user review everything that has been configured so far — org/repo name, template, type,
visibility, team, and (when the template has variables) every variable's resolved value — in one
place before committing to repository creation. Create the repository once confirmed.

This step closes a gap in the previous design: configured variable values were captured in
wizard state as soon as the user typed them, but were never shown anywhere before creation — not
even on the step where Create was clicked. Review now surfaces them.

### Layout

- Heading: "Review and create"
- A single `RepositorySummary` (CMP-009) block showing, in order:
  - `[org] / [repository name]`
  - Template (display name), repository type, and visibility chips
  - The template's full description (when available)
  - The template's full tag list (when any)
  - Every configured variable, as a `name: value` row — this list is empty (and the whole
    section renders nothing) when the template has no variables
- "← Back" secondary button at the bottom
- "Create Repository" primary button at the bottom — **this is the only place in the wizard the
  Create action appears**

### States

- **Ready**: Create button enabled
- **Submitting** (Create clicked): `creatingRepo` is set; Create button disabled for the
  duration (prevents duplicate submissions from a rapid double-click); creation overlay shown;
  wizard content hidden behind it (`aria-hidden="true"`, not removed from the DOM)

### Interactions

| Element | Action | Outcome |
|---|---|---|
| "← Back" button | Click | Return to the previous step (Variables, if the template has any; otherwise Settings); all previously entered values preserved |
| "Create Repository" button | Click | Submits `POST /api/v1/repositories` with all collected data; shows the creation overlay; ignored if a submission is already in flight |

### Create button activation rules

The Create button is **disabled** only while a creation request is already in flight
(`creatingRepo`); otherwise it is always enabled on this step, since every required value from
earlier steps was already validated before the user could advance this far.

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

Create is now only ever triggered from the Review step, so every creation error is first raised
there. All creation errors are shown as an `InlineAlert` component with `variant="error"`. For the
two recoverable conflict cases, the user is then automatically moved back to the step that needs
correcting (Settings or Step 1) and the alert is shown there instead; for the remaining cases the
alert stays on the Review step. The creation overlay is dismissed before any error appears.

| API Error | Recovery UX |
|---|---|
| 422 — name already taken (race condition) | User returned to Settings (Step 2); name field shows taken error; InlineAlert: "A repository named '[name]' was created while you were filling in this form. Please choose a different name." |
| 422 — template not found | User returned to Step 1; template grid reloaded; InlineAlert: "The template '[name]' is no longer available. Please choose a different template." |
| 403 — permission denied | InlineAlert on the Review step: "You don't have permission to create repositories in this organization. Contact your administrator." |
| 5xx — GitHub API error | InlineAlert on the Review step: "GitHub is temporarily unavailable. Your repository was not created. Please try again." |
| Network error (no response) | InlineAlert on the Review step: "Could not reach the server. Check your connection and try again." |

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
| Create repository | On "Create Repository" click (Review step — the only step where this button appears; previously this could be Step 2 or Step 3 depending on whether the template had variables) | `POST /api/v1/repositories` |

### Inputs collected (final POST body)

The POST body itself is unchanged by this round of changes — only the step from which it is sent
(always Review, now) and the fact that every variable value is also visible to the user on that
same step before they click Create.

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
